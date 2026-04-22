#!/usr/bin/env python3
"""
hewiki_dump_ingest.py — download + stream-parse Hebrew Wikipedia dump.

This is a DIFFERENT workflow than the RSS crawler:
  - One-time dump ingestion (not polling)
  - Content comes from Wikipedia's official dump server (polite)
  - Stream parsing (never fully expand the XML into memory)
  - Safety limits: max N articles, max disk usage, stop-file honored

Expected yield: ~360K Hebrew Wikipedia articles → ~2-5M atoms after ingestion.

Usage:
  python3 mcp/autonomous/hewiki_dump_ingest.py download  # ~800MB, 10-30 min
  python3 mcp/autonomous/hewiki_dump_ingest.py parse     # stream-parse
  python3 mcp/autonomous/hewiki_dump_ingest.py stats     # show what's in the dump
"""

import bz2
import json
import os
import re
import shutil
import sys
import time
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime
from pathlib import Path

HERE = Path(os.path.dirname(os.path.abspath(__file__)))
STOP_FILE = HERE / 'STOP'

DUMP_DIR = Path('/home/dinio/zets/data/wikipedia_dumps')
DUMP_DIR.mkdir(parents=True, exist_ok=True)

# Small dump — Hebrew Wikipedia articles only, no images or history
DUMP_URL = 'https://dumps.wikimedia.org/hewiki/latest/hewiki-latest-pages-articles.xml.bz2'
DUMP_LOCAL = DUMP_DIR / 'hewiki-latest-pages-articles.xml.bz2'

# Safety limits
MAX_ARTICLES = 50_000           # cap at 50K for overnight run
MIN_DISK_GB = 3.0               # stop if disk drops below this
CHECKPOINT_EVERY = 1000         # save progress every N articles

OUTPUT_JSONL = DUMP_DIR / 'hewiki_parsed.jsonl'
PROGRESS_FILE = DUMP_DIR / 'hewiki_progress.json'


def disk_free_gb() -> float:
    return shutil.disk_usage('/home/dinio/zets').free / (1024 ** 3)


# ═══════════════════════════════════════════════════════════════════════
# Download (with resume + progress)
# ═══════════════════════════════════════════════════════════════════════

def download():
    if DUMP_LOCAL.exists():
        sz = DUMP_LOCAL.stat().st_size / 1024 / 1024
        print(f"  Dump already exists ({sz:.1f} MB): {DUMP_LOCAL}")
        print(f"  Delete it to re-download.")
        return

    df = disk_free_gb()
    if df < 3.0:
        print(f"  ABORT: only {df:.1f}GB free. Need 3+GB for HE dump.")
        return

    print(f"  Downloading {DUMP_URL}")
    print(f"  → {DUMP_LOCAL}")
    print(f"  Expected: ~800 MB. Disk free: {df:.1f} GB")
    print()

    tmp_path = DUMP_LOCAL.with_suffix('.bz2.part')
    t0 = time.time()

    req = urllib.request.Request(DUMP_URL, headers={
        'User-Agent': 'ZETS-Learner/0.1 (+https://github.com/idaneldad/zets; contact: idan@chooz.co.il)'
    })

    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            total = int(r.headers.get('Content-Length', 0))
            print(f"  Size: {total/1024/1024:.1f} MB")
            written = 0
            with open(tmp_path, 'wb') as f:
                while True:
                    if STOP_FILE.exists():
                        print("  STOP file present, aborting download")
                        return
                    chunk = r.read(1024 * 1024)  # 1MB at a time
                    if not chunk:
                        break
                    f.write(chunk)
                    written += len(chunk)
                    pct = 100 * written / max(total, 1)
                    elapsed = time.time() - t0
                    rate_mbps = (written / 1024 / 1024) / max(elapsed, 0.1)
                    eta = (total - written) / (rate_mbps * 1024 * 1024) if rate_mbps > 0 else 0
                    if written % (10 * 1024 * 1024) < 1024 * 1024:  # every ~10MB
                        print(f"    {pct:5.1f}%  {written/1024/1024:7.1f} MB  "
                              f"{rate_mbps:.1f} MB/s  ETA {eta:.0f}s")
        tmp_path.rename(DUMP_LOCAL)
        print()
        print(f"  ✓ Done. {DUMP_LOCAL.stat().st_size/1024/1024:.1f} MB, "
              f"{time.time()-t0:.0f}s")
    except Exception as e:
        print(f"  ✗ Download failed: {e}")
        if tmp_path.exists():
            tmp_path.unlink()


# ═══════════════════════════════════════════════════════════════════════
# Stream-parse (memory-efficient)
# ═══════════════════════════════════════════════════════════════════════

def strip_wikitext(text: str) -> str:
    """Very basic wikitext → plain text. Good enough for ingestion."""
    if not text:
        return ''
    # Remove templates {{...}}
    text = re.sub(r'\{\{[^{}]*\}\}', '', text)
    text = re.sub(r'\{\{[^{}]*\}\}', '', text)  # run twice for nested
    # Remove tables {|...|}
    text = re.sub(r'\{\|.*?\|\}', '', text, flags=re.DOTALL)
    # Remove references <ref>...</ref>
    text = re.sub(r'<ref[^>]*>.*?</ref>', '', text, flags=re.DOTALL)
    text = re.sub(r'<ref[^>]*/>', '', text)
    # Remove HTML tags
    text = re.sub(r'<[^>]+>', '', text)
    # Links: [[target|text]] → text; [[target]] → target
    text = re.sub(r'\[\[([^\]|]+)\|([^\]]+)\]\]', r'\2', text)
    text = re.sub(r'\[\[([^\]]+)\]\]', r'\1', text)
    # External links: [url text] → text
    text = re.sub(r'\[https?://\S+\s+([^\]]+)\]', r'\1', text)
    text = re.sub(r'\[https?://\S+\]', '', text)
    # Bold/italic: '''x''' / ''x'' → x
    text = re.sub(r"'''([^']+)'''", r'\1', text)
    text = re.sub(r"''([^']+)''", r'\1', text)
    # Headers: == x == → x
    text = re.sub(r'={2,}\s*([^=]+?)\s*={2,}', r'\1', text)
    # Collapse whitespace
    text = re.sub(r'\n{3,}', '\n\n', text)
    text = re.sub(r'[ \t]+', ' ', text)
    return text.strip()


def parse():
    if not DUMP_LOCAL.exists():
        print(f"  No dump file. Run: python3 {sys.argv[0]} download")
        return

    # Resume from checkpoint if exists
    progress = {'last_article': 0, 'articles_written': 0, 'started_ts': time.time()}
    if PROGRESS_FILE.exists():
        try:
            progress = json.loads(PROGRESS_FILE.read_text())
            print(f"  Resuming from article #{progress.get('last_article', 0)}")
        except Exception:
            pass

    ns_uri = 'http://www.mediawiki.org/xml/export-0.11/'
    NS_PAGE = f'{{{ns_uri}}}page'
    NS_TITLE = f'{{{ns_uri}}}title'
    NS_TEXT = f'{{{ns_uri}}}text'
    NS_NS = f'{{{ns_uri}}}ns'
    NS_REDIRECT = f'{{{ns_uri}}}redirect'

    t0 = time.time()
    articles_seen = 0
    articles_written = progress.get('articles_written', 0)
    skip_until = progress.get('last_article', 0)

    out_mode = 'a' if skip_until > 0 else 'w'
    print(f"  Streaming parse → {OUTPUT_JSONL}")
    print(f"  Max: {MAX_ARTICLES} articles, skip first {skip_until}")
    print()

    with open(OUTPUT_JSONL, out_mode, encoding='utf-8') as out_f:
        with bz2.open(DUMP_LOCAL, 'rb') as raw:
            try:
                context = ET.iterparse(raw, events=('end',))
                for event, elem in context:
                    if elem.tag != NS_PAGE:
                        continue

                    articles_seen += 1

                    if articles_seen <= skip_until:
                        elem.clear()
                        continue

                    # Safety checks
                    if STOP_FILE.exists():
                        print("  STOP file present, halting parse")
                        break
                    if articles_written >= MAX_ARTICLES:
                        print(f"  MAX_ARTICLES={MAX_ARTICLES} reached")
                        break
                    if articles_seen % 500 == 0:
                        df = disk_free_gb()
                        if df < MIN_DISK_GB:
                            print(f"  Disk low ({df:.1f}GB), halting")
                            break

                    # Extract
                    title_el = elem.find(NS_TITLE)
                    text_el = elem.find(NS_TEXT)
                    ns_el = elem.find(NS_NS)
                    redirect_el = elem.find(NS_REDIRECT)

                    # Skip redirects
                    if redirect_el is not None:
                        elem.clear()
                        continue
                    # Only namespace 0 (main articles)
                    if ns_el is not None and ns_el.text != '0':
                        elem.clear()
                        continue

                    title = title_el.text if title_el is not None else None
                    raw_text = text_el.text if text_el is not None else None
                    if not title or not raw_text:
                        elem.clear()
                        continue

                    clean_text = strip_wikitext(raw_text)
                    # Skip stubs (< 200 chars)
                    if len(clean_text) < 200:
                        elem.clear()
                        continue

                    # Write
                    out_f.write(json.dumps({
                        'title': title,
                        'lang': 'he',
                        'source': 'wikipedia_he',
                        'source_tier': 'B',
                        'prior_confidence': 0.80,
                        'text': clean_text[:20000],  # cap per article at 20K
                        'text_length_original': len(clean_text),
                    }, ensure_ascii=False) + '\n')
                    articles_written += 1

                    # Checkpoint periodically
                    if articles_written % CHECKPOINT_EVERY == 0:
                        out_f.flush()
                        progress = {
                            'last_article': articles_seen,
                            'articles_written': articles_written,
                            'updated_ts': time.time(),
                            'elapsed_s': time.time() - t0,
                        }
                        PROGRESS_FILE.write_text(json.dumps(progress, ensure_ascii=False,
                                                             indent=1))
                        rate = articles_written / max(time.time() - t0, 0.1)
                        print(f"    [{datetime.now().strftime('%H:%M:%S')}] "
                              f"seen={articles_seen:>7,}  "
                              f"written={articles_written:>6,}  "
                              f"rate={rate:.0f}/s")

                    elem.clear()
            except ET.ParseError as e:
                print(f"  Parse error at article {articles_seen}: {e}")
            except KeyboardInterrupt:
                print("  Interrupted by user")

    # Final checkpoint
    PROGRESS_FILE.write_text(json.dumps({
        'last_article': articles_seen,
        'articles_written': articles_written,
        'completed_ts': time.time(),
        'elapsed_s': time.time() - t0,
    }, ensure_ascii=False, indent=1))

    print()
    print(f"  ✓ Done. {articles_written} articles written to {OUTPUT_JSONL}")
    print(f"    Output size: {OUTPUT_JSONL.stat().st_size/1024/1024:.1f} MB")
    print(f"    Time: {time.time()-t0:.0f}s")


# ═══════════════════════════════════════════════════════════════════════
# Stats
# ═══════════════════════════════════════════════════════════════════════

def stats():
    if PROGRESS_FILE.exists():
        p = json.loads(PROGRESS_FILE.read_text())
        print(f"  Progress:    last_article={p.get('last_article', 0):,}  "
              f"written={p.get('articles_written', 0):,}")
    if DUMP_LOCAL.exists():
        print(f"  Dump file:   {DUMP_LOCAL.stat().st_size/1024/1024:.1f} MB")
    if OUTPUT_JSONL.exists():
        print(f"  Parsed JSONL: {OUTPUT_JSONL.stat().st_size/1024/1024:.1f} MB  "
              f"({sum(1 for _ in open(OUTPUT_JSONL)):,} articles)")


if __name__ == '__main__':
    cmd = sys.argv[1] if len(sys.argv) > 1 else 'stats'
    if cmd == 'download':
        download()
    elif cmd == 'parse':
        parse()
    elif cmd == 'stats':
        stats()
    else:
        print(f"Usage: {sys.argv[0]} [download|parse|stats]")
