#!/usr/bin/env python3
"""
rss_ingestor.py — RSS/Atom feed → facts with trust scoring + dedup.

For each feed item:
  1. Fetch the feed (politely)
  2. Parse items (title, link, date, summary, author)
  3. Compute content_hash (FNV-1a — same algorithm as Rust zets)
  4. Dedup: check if hash already in registry
  5. Score: tier × corroboration (for now just tier)
  6. Store minimal fact + citation (NOT full article)

MINIMAL scope on purpose. Not a full crawler. Proof-of-concept.

Outputs:
  - JSONL at data/autonomous/items_<date>.jsonl
  - Registry updates (tracked in JSON sidecar)
"""

import json
import re
import time
import xml.etree.ElementTree as ET
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Optional, List
import sys, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from polite_fetcher import Fetcher  # noqa
from source_tiers import domain_from_url, tier_of  # noqa


DATA_DIR = Path(os.environ.get('ZETS_AUTO_DATA',
                               '/home/dinio/zets/data/autonomous'))
DATA_DIR.mkdir(parents=True, exist_ok=True)


# ═══════════════════════════════════════════════════════════════════════
# FNV-1a 64-bit — matches Rust content_hash exactly
# ═══════════════════════════════════════════════════════════════════════

def content_hash(data: bytes) -> int:
    """Same FNV-1a used in src/atoms.rs content_hash."""
    h = 0xcbf29ce484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return h


# ═══════════════════════════════════════════════════════════════════════
# Feed catalog (seed — tier-aware)
# ═══════════════════════════════════════════════════════════════════════

DEFAULT_FEEDS = [
    # Tier A
    {'url': 'https://rss.arxiv.org/rss/cs.AI',          'name': 'arxiv_cs_ai'},
    # Tier B
    {'url': 'https://en.wikipedia.org/w/index.php?title=Special:RecentChanges&feed=rss',
     'name': 'wiki_en_recent'},
    # Tier C
    {'url': 'https://www.nasa.gov/news-release/feed/', 'name': 'nasa_news'},
    # Tier D
    {'url': 'https://feeds.reuters.com/reuters/technologyNews', 'name': 'reuters_tech'},
    {'url': 'https://feeds.bbci.co.uk/news/world/rss.xml', 'name': 'bbc_world'},
]


# ═══════════════════════════════════════════════════════════════════════
# RSS/Atom parser — robust to both formats
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class FeedItem:
    source: str              # feed name
    source_url: str          # feed URL
    source_domain: str
    source_tier: str         # A-F / X
    prior_confidence: float

    title: str
    link: str
    published_ts: Optional[float]
    author: Optional[str]
    summary: Optional[str]   # short — NO full article verbatim
    canonical: Optional[str]

    # Identity
    content_hash_title: int      # title-only (for dedup of same article)
    content_hash_quote: int      # normalized-title+first-N-summary-words
    content_hash_link: int       # canonical link


def parse_rss(xml_body: str, feed_name: str, feed_url: str,
              domain: str, tier: str, prior: float) -> List[FeedItem]:
    items: List[FeedItem] = []
    try:
        root = ET.fromstring(xml_body)
    except ET.ParseError:
        return items

    # Strip namespaces to simplify lookups
    for elem in root.iter():
        if '}' in elem.tag:
            elem.tag = elem.tag.split('}', 1)[1]

    # RSS 2.0: root > channel > item*
    # Atom: root > entry*
    item_elements = root.findall('.//item') or root.findall('.//entry')

    for it in item_elements:
        title = _text(it, 'title') or ''
        link = _text(it, 'link') or _attr(it, 'link', 'href') or ''
        if not title and not link:
            continue

        pub_raw = (_text(it, 'pubDate') or _text(it, 'published')
                   or _text(it, 'updated') or '')
        pub_ts = _parse_date(pub_raw)

        author = _text(it, 'author') or _text(it, 'creator') or _text(it, 'dc:creator')

        summary = _text(it, 'description') or _text(it, 'summary') or _text(it, 'content')
        if summary:
            # Strip HTML tags; keep only first 300 chars (minimum viable — NO full article)
            summary = re.sub(r'<[^>]+>', '', summary)
            summary = re.sub(r'\s+', ' ', summary).strip()
            summary = summary[:300]

        canonical = _canonical_url(link)

        h_title = content_hash(title.strip().encode('utf-8'))
        h_link = content_hash(canonical.encode('utf-8')) if canonical else 0
        quote_text = _normalize(title + ' ' + (summary or '')[:80])
        h_quote = content_hash(quote_text.encode('utf-8'))

        items.append(FeedItem(
            source=feed_name,
            source_url=feed_url,
            source_domain=domain,
            source_tier=tier,
            prior_confidence=prior,
            title=title.strip(),
            link=link.strip(),
            published_ts=pub_ts,
            author=author.strip() if author else None,
            summary=summary,
            canonical=canonical,
            content_hash_title=h_title,
            content_hash_quote=h_quote,
            content_hash_link=h_link,
        ))

    return items


def _text(elem, tag):
    for child in elem.iter():
        if child.tag == tag and child.text:
            return child.text
    return None


def _attr(elem, tag, attr):
    for child in elem.iter():
        if child.tag == tag and attr in child.attrib:
            return child.attrib[attr]
    return None


def _parse_date(s: str) -> Optional[float]:
    if not s:
        return None
    for fmt in (
        '%a, %d %b %Y %H:%M:%S %z',
        '%a, %d %b %Y %H:%M:%S %Z',
        '%Y-%m-%dT%H:%M:%S%z',
        '%Y-%m-%dT%H:%M:%SZ',
        '%Y-%m-%dT%H:%M:%S.%fZ',
    ):
        try:
            return datetime.strptime(s, fmt).timestamp()
        except Exception:
            pass
    return None


def _canonical_url(url: str) -> str:
    """Strip common tracking parameters."""
    from urllib.parse import urlparse, parse_qs, urlencode, urlunparse
    try:
        u = urlparse(url)
        qs = {k: v for k, v in parse_qs(u.query).items()
              if not k.startswith('utm_') and k not in ('ref', 'source', 'src')}
        return urlunparse((u.scheme, u.netloc, u.path, u.params,
                          urlencode(qs, doseq=True), ''))
    except Exception:
        return url


def _normalize(s: str) -> str:
    s = s.lower()
    s = re.sub(r'[^\w\s]', '', s, flags=re.UNICODE)
    s = re.sub(r'\s+', ' ', s).strip()
    return s


# ═══════════════════════════════════════════════════════════════════════
# Hash registry sidecar (tracks what we've seen across runs)
# ═══════════════════════════════════════════════════════════════════════

class HashRegistrySidecar:
    """Simple JSON-backed registry (prototype; real system uses Rust HashRegistry)."""

    def __init__(self, path: Path):
        self.path = path
        self.data: dict = {'by_hash': {}, 'created_ts': time.time(),
                          'updated_ts': time.time()}
        if path.exists():
            try:
                self.data = json.loads(path.read_text())
            except Exception:
                pass

    def seen(self, h: int) -> bool:
        return str(h) in self.data.get('by_hash', {})

    def register(self, h: int, source: str, confidence: float):
        k = str(h)
        entries = self.data['by_hash'].get(k, [])
        # Dedup: same source already?
        if not any(e.get('source') == source for e in entries):
            entries.append({'source': source, 'confidence': confidence,
                           'ts': time.time()})
        self.data['by_hash'][k] = entries

    def corroboration_count(self, h: int) -> int:
        return len(self.data.get('by_hash', {}).get(str(h), []))

    def save(self):
        self.data['updated_ts'] = time.time()
        self.path.write_text(json.dumps(self.data, ensure_ascii=False, indent=1))

    def stats(self) -> dict:
        total = len(self.data.get('by_hash', {}))
        shared = sum(1 for v in self.data.get('by_hash', {}).values() if len(v) > 1)
        return {'total_hashes': total, 'shared_hashes': shared}


# ═══════════════════════════════════════════════════════════════════════
# Main ingester
# ═══════════════════════════════════════════════════════════════════════

def ingest_feed(fetcher: Fetcher, feed: dict, registry: HashRegistrySidecar) -> dict:
    """Fetch + parse + dedup one feed."""
    url = feed['url']
    name = feed['name']
    host = domain_from_url(url)
    if not host:
        return {'feed': name, 'status': 'bad_url'}
    tier_info = tier_of(host)
    resp = fetcher.get(url)
    if not resp or not resp.get('body'):
        return {'feed': name, 'status': 'fetch_failed'}
    body = resp['body']
    items = parse_rss(body, name, url, host, tier_info.tier,
                      tier_info.prior_confidence)

    # Dedup + register
    novel = []
    duplicates = []
    for it in items:
        # Use the quote-hash as primary identity
        h = it.content_hash_quote
        # Use DOMAIN as source identity for proper corroboration counting.
        # (Re-fetching same feed doesn't inflate the count.)
        source_id = host
        if registry.seen(h):
            existing_sources = [
                e['source'] for e in registry.data.get('by_hash', {}).get(str(h), [])
            ]
            if source_id not in existing_sources:
                duplicates.append(it)
                registry.register(h, source_id, it.prior_confidence)
        else:
            novel.append(it)
            registry.register(h, source_id, it.prior_confidence)
            if it.content_hash_title != h:
                registry.register(it.content_hash_title, source_id, it.prior_confidence)
            if it.content_hash_link and it.content_hash_link != h:
                registry.register(it.content_hash_link, source_id, it.prior_confidence)

    return {
        'feed': name,
        'status': 'ok',
        'tier': tier_info.tier,
        'items_total': len(items),
        'items_novel': len(novel),
        'items_corroborated': len(duplicates),
        'novel_items': novel,
    }


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument('--feeds', choices=['default', 'custom'], default='default')
    ap.add_argument('--limit', type=int, default=0,
                   help='limit total items saved across all feeds')
    args = ap.parse_args()

    feeds = DEFAULT_FEEDS if args.feeds == 'default' else []
    if not feeds:
        print("No feeds")
        return

    fetcher = Fetcher()
    registry_path = DATA_DIR / 'hash_registry_sidecar.json'
    registry = HashRegistrySidecar(registry_path)

    print(f"━━━ Ingesting {len(feeds)} RSS feeds ━━━")
    print()

    all_novel = []
    report = []
    for feed in feeds:
        print(f"  Fetching {feed['name']:<20} ", end='', flush=True)
        r = ingest_feed(fetcher, feed, registry)
        if r['status'] == 'ok':
            print(f"tier={r['tier']}  total={r['items_total']:3d}  "
                  f"novel={r['items_novel']:3d}  corroborated={r['items_corroborated']:3d}")
            all_novel.extend(r['novel_items'])
        else:
            print(f"FAILED ({r['status']})")
        report.append({k: v for k, v in r.items() if k != 'novel_items'})

    # Save novel items (facts only, NO full article bodies)
    date_str = datetime.now().strftime('%Y%m%d_%H%M')
    items_path = DATA_DIR / f'items_{date_str}.jsonl'
    with open(items_path, 'w', encoding='utf-8') as f:
        for i, item in enumerate(all_novel):
            if args.limit and i >= args.limit:
                break
            f.write(json.dumps(asdict(item), ensure_ascii=False) + '\n')

    registry.save()

    # Summary
    print()
    print(f"━━━ Summary ━━━")
    print(f"  Items saved:     {len(all_novel)}  →  {items_path.name}")
    reg_stats = registry.stats()
    print(f"  Hash registry:   {reg_stats['total_hashes']} hashes, "
          f"{reg_stats['shared_hashes']} corroborated (2+ sources)")
    print(f"  Fetcher stats:   {fetcher.stats}")
    print()
    print(f"━━━ Per-feed ━━━")
    for r in report:
        print(f"  {r['feed']:<20}  {r.get('status')}")


if __name__ == '__main__':
    main()
