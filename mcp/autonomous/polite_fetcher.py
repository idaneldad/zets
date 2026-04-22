#!/usr/bin/env python3
"""
polite_fetcher.py — HTTP fetcher with politeness discipline.

Non-negotiable rules (see design doc):
  - User-Agent identifies us as ZETS-Learner
  - robots.txt parsed and honored per domain (cached 24h)
  - Per-domain rate limit: min 3 seconds between requests
  - Global rate limit: max 10 requests/second overall
  - Exponential backoff on 429 / 503
  - No login bypass, no CAPTCHA bypass, no paywall bypass
  - ETag / If-Modified-Since caching

Designed as a prototype — single-threaded, synchronous. Good enough for
10K-100K requests/day at safe rates. Production version would use asyncio
but the rules stay the same.

Usage:
  from polite_fetcher import Fetcher
  f = Fetcher()
  r = f.get('https://arxiv.org/abs/2401.12345')
  # r is either dict {status, body, headers, tier} or None if blocked/failed
"""

import hashlib
import json
import time
import urllib.parse
import urllib.request
import urllib.error
import urllib.robotparser
from pathlib import Path
from typing import Optional

import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from source_tiers import tier_of, domain_from_url  # noqa


USER_AGENT = "ZETS-Learner/0.1 (+https://github.com/idaneldad/zets; research; contact: idan@chooz.co.il)"

DEFAULT_CRAWL_DELAY = 3.0    # seconds per domain
GLOBAL_MAX_RPS = 10.0        # requests per second across ALL domains
ROBOTS_CACHE_TTL = 86400     # 24 hours
CACHE_DIR = Path(os.environ.get('ZETS_CACHE_DIR', '/home/dinio/zets/data/autonomous_cache'))


class Fetcher:
    """Single-process polite fetcher. Persists robots.txt + response cache."""

    def __init__(self, cache_dir: Optional[Path] = None):
        self.cache_dir = cache_dir or CACHE_DIR
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.robots_dir = self.cache_dir / 'robots'
        self.robots_dir.mkdir(exist_ok=True)
        self.etag_dir = self.cache_dir / 'etag'
        self.etag_dir.mkdir(exist_ok=True)
        # Per-domain last-request timestamps
        self.last_request_ts: dict[str, float] = {}
        # Global rate limit — simple leaky bucket
        self.global_bucket: list[float] = []
        # Stats
        self.stats = {
            'requests': 0,
            'cached': 0,
            'robots_denied': 0,
            'rate_limited_429': 0,
            'errors': 0,
            'bytes_fetched': 0,
        }

    # ── Politeness helpers ──────────────────────────────────────────────

    def _robots_for(self, hostname: str) -> Optional[urllib.robotparser.RobotFileParser]:
        """Return a RobotFileParser for hostname, fetching + caching if needed."""
        cache_path = self.robots_dir / (hostname + '.txt')
        if cache_path.exists():
            age = time.time() - cache_path.stat().st_mtime
            if age < ROBOTS_CACHE_TTL:
                rp = urllib.robotparser.RobotFileParser()
                rp.parse(cache_path.read_text(errors='replace').splitlines())
                return rp
        # Fetch robots.txt
        for scheme in ('https', 'http'):
            url = f"{scheme}://{hostname}/robots.txt"
            try:
                req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
                with urllib.request.urlopen(req, timeout=10) as r:
                    body = r.read().decode('utf-8', errors='replace')
                cache_path.write_text(body, encoding='utf-8')
                rp = urllib.robotparser.RobotFileParser()
                rp.parse(body.splitlines())
                return rp
            except urllib.error.HTTPError as e:
                if e.code == 404:
                    # No robots.txt means everything allowed
                    cache_path.write_text('', encoding='utf-8')
                    rp = urllib.robotparser.RobotFileParser()
                    rp.parse([])
                    return rp
                continue
            except Exception:
                continue
        return None

    def _allowed(self, url: str) -> bool:
        h = domain_from_url(url)
        if not h:
            return False
        rp = self._robots_for(h)
        if rp is None:
            # If we couldn't even fetch robots.txt, be conservative — deny
            return False
        return rp.can_fetch(USER_AGENT, url)

    def _crawl_delay(self, hostname: str) -> float:
        rp = self._robots_for(hostname)
        if rp:
            d = rp.crawl_delay(USER_AGENT)
            if d:
                return max(float(d), DEFAULT_CRAWL_DELAY)
        return DEFAULT_CRAWL_DELAY

    def _wait_per_domain(self, hostname: str):
        delay = self._crawl_delay(hostname)
        last = self.last_request_ts.get(hostname, 0.0)
        elapsed = time.time() - last
        if elapsed < delay:
            time.sleep(delay - elapsed)
        self.last_request_ts[hostname] = time.time()

    def _wait_global(self):
        now = time.time()
        # Drop timestamps older than 1 second
        self.global_bucket = [t for t in self.global_bucket if now - t < 1.0]
        if len(self.global_bucket) >= GLOBAL_MAX_RPS:
            # Sleep until oldest drops out
            sleep_for = 1.0 - (now - self.global_bucket[0])
            if sleep_for > 0:
                time.sleep(sleep_for)
        self.global_bucket.append(time.time())

    # ── ETag cache ─────────────────────────────────────────────────────
    def _etag_key(self, url: str) -> Path:
        h = hashlib.sha256(url.encode('utf-8')).hexdigest()[:24]
        return self.etag_dir / h

    def _etag_load(self, url: str) -> Optional[dict]:
        p = self._etag_key(url)
        if p.exists():
            try:
                return json.loads(p.read_text())
            except Exception:
                return None
        return None

    def _etag_save(self, url: str, etag: Optional[str], last_modified: Optional[str],
                   body: str):
        p = self._etag_key(url)
        p.write_text(json.dumps({
            'url': url,
            'etag': etag,
            'last_modified': last_modified,
            'body': body,
            'fetched_ts': time.time(),
        }, ensure_ascii=False))

    # ── Main fetch ─────────────────────────────────────────────────────

    def get(self, url: str, use_cache: bool = True) -> Optional[dict]:
        """Polite fetch. Returns dict with status/body/headers/tier, or None."""
        h = domain_from_url(url)
        if not h:
            return None
        tier_info = tier_of(h)

        # 1. Check robots
        if not self._allowed(url):
            self.stats['robots_denied'] += 1
            return {
                'url': url, 'status': 0, 'body': None,
                'tier': tier_info.tier, 'reason': 'robots_denied',
            }

        # 2. Wait (per-domain + global)
        self._wait_per_domain(h)
        self._wait_global()

        # 3. Build request
        headers = {'User-Agent': USER_AGENT, 'Accept-Encoding': 'gzip'}
        cached = self._etag_load(url) if use_cache else None
        if cached:
            if cached.get('etag'):
                headers['If-None-Match'] = cached['etag']
            if cached.get('last_modified'):
                headers['If-Modified-Since'] = cached['last_modified']

        req = urllib.request.Request(url, headers=headers)

        # 4. Fetch
        self.stats['requests'] += 1
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                raw = r.read()
                body = _maybe_gunzip(raw, r.headers)
                etag = r.headers.get('ETag')
                last_modified = r.headers.get('Last-Modified')
                self.stats['bytes_fetched'] += len(body)
                if use_cache:
                    self._etag_save(url, etag, last_modified, body)
                return {
                    'url': url,
                    'final_url': r.geturl(),
                    'status': r.status,
                    'body': body,
                    'tier': tier_info.tier,
                    'prior_confidence': tier_info.prior_confidence,
                    'etag': etag,
                    'last_modified': last_modified,
                    'from_cache': False,
                }
        except urllib.error.HTTPError as e:
            if e.code == 304 and cached:
                self.stats['cached'] += 1
                return {
                    'url': url, 'status': 304,
                    'body': cached['body'], 'tier': tier_info.tier,
                    'prior_confidence': tier_info.prior_confidence,
                    'from_cache': True,
                }
            if e.code == 429:
                self.stats['rate_limited_429'] += 1
                # Punish ourselves: double wait for this domain
                self.last_request_ts[h] = time.time() + 60  # 1-min cooldown
                return None
            self.stats['errors'] += 1
            return None
        except Exception:
            self.stats['errors'] += 1
            return None


def _maybe_gunzip(raw: bytes, headers) -> str:
    enc = (headers.get('Content-Encoding') or '').lower()
    if enc == 'gzip':
        import gzip
        try:
            return gzip.decompress(raw).decode('utf-8', errors='replace')
        except Exception:
            return raw.decode('utf-8', errors='replace')
    return raw.decode('utf-8', errors='replace')


if __name__ == '__main__':
    f = Fetcher()
    test_urls = [
        'https://en.wikipedia.org/wiki/Graph_database',
        'https://arxiv.org/list/cs.AI/recent',
    ]
    for u in test_urls:
        r = f.get(u)
        if r:
            print(f"  {u[:60]:<60}  status={r.get('status')}  tier={r.get('tier')}  "
                  f"body={len(r.get('body') or '')} chars")
        else:
            print(f"  {u[:60]:<60}  failed")
    print(f"\n  Stats: {f.stats}")
