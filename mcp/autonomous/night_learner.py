#!/usr/bin/env python3
"""
night_learner.py — autonomous RSS harvesting loop, overnight.

What it does (forever, or until stopped):
  1. Every CYCLE_MINUTES (default 30), fetch a rotating batch of feeds
  2. Extract novel items, update hash_registry
  3. Log each cycle to logs/night/YYYYMMDD_HHMM.json
  4. Honor safety limits:
       - STOP file present → stop gracefully
       - Disk < 5GB free → stop
       - Items harvested > MAX_ITEMS → stop
       - Uptime > MAX_HOURS → stop
  5. Write status heartbeat every 60 seconds to status.json

What it does NOT do:
  - Bypass robots.txt
  - Hammer any single domain (built-in politeness)
  - Store full articles (only title + link + summary + hashes)
  - Run without the safety checks

Start:
  nohup python3 mcp/autonomous/night_learner.py &
  echo $! > mcp/autonomous/logs/night.pid

Stop:
  touch mcp/autonomous/STOP
  # or
  kill $(cat mcp/autonomous/logs/night.pid)
"""

import json
import os
import random
import shutil
import signal
import socket
import sys
import time
import traceback
from dataclasses import asdict
from datetime import datetime, timedelta
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from polite_fetcher import Fetcher  # noqa
from rss_ingestor import ingest_feed, HashRegistrySidecar, DATA_DIR  # noqa
from source_tiers import tier_of, domain_from_url  # noqa


# ═══════════════════════════════════════════════════════════════════════
# Configuration
# ═══════════════════════════════════════════════════════════════════════

HERE = Path(os.path.dirname(os.path.abspath(__file__)))
LOGS_DIR = HERE / 'logs' / 'night'
LOGS_DIR.mkdir(parents=True, exist_ok=True)

STATUS_FILE = HERE / 'status.json'
STOP_FILE = HERE / 'STOP'

# Safety limits
MAX_HOURS = 12.0                  # stop after 12 hours
MAX_ITEMS_TOTAL = 100_000         # stop after 100K items harvested
MIN_DISK_GB = 5.0                 # stop if less than 5GB free
CYCLE_MINUTES = 30                # how often to poll each feed

# Feed catalog — organized by tier, all respecting robots.txt
FEED_CATALOG = [
    # ─── Tier A (0.90) — scholarly ───
    {'url': 'https://rss.arxiv.org/rss/cs.AI',      'name': 'arxiv_cs_AI'},
    {'url': 'https://rss.arxiv.org/rss/cs.CL',      'name': 'arxiv_cs_CL'},
    {'url': 'https://rss.arxiv.org/rss/cs.LG',      'name': 'arxiv_cs_LG'},
    {'url': 'https://rss.arxiv.org/rss/cs.CV',      'name': 'arxiv_cs_CV'},
    {'url': 'https://rss.arxiv.org/rss/cs.NE',      'name': 'arxiv_cs_NE'},
    {'url': 'https://rss.arxiv.org/rss/cs.RO',      'name': 'arxiv_cs_RO'},
    {'url': 'https://rss.arxiv.org/rss/math.CO',    'name': 'arxiv_math_CO'},
    {'url': 'https://rss.arxiv.org/rss/math.NT',    'name': 'arxiv_math_NT'},
    {'url': 'https://rss.arxiv.org/rss/physics.gen-ph', 'name': 'arxiv_phys_gen'},
    {'url': 'https://rss.arxiv.org/rss/q-bio',      'name': 'arxiv_qbio'},
    {'url': 'https://rss.arxiv.org/rss/stat.ML',    'name': 'arxiv_stat_ML'},

    # ─── Tier C (0.75) — gov/edu ───
    {'url': 'https://www.nasa.gov/news-release/feed/',           'name': 'nasa_news'},
    {'url': 'https://science.nasa.gov/feed/',                    'name': 'nasa_science'},
    {'url': 'https://www.cdc.gov/media/rss/spotlights-rss.xml',  'name': 'cdc_spotlights'},

    # ─── Tier D (0.70) — established journalism ───
    {'url': 'https://feeds.bbci.co.uk/news/rss.xml',          'name': 'bbc_world'},
    {'url': 'https://feeds.bbci.co.uk/news/technology/rss.xml', 'name': 'bbc_tech'},
    {'url': 'https://feeds.bbci.co.uk/news/science_and_environment/rss.xml', 'name': 'bbc_science'},
    {'url': 'https://www.theguardian.com/world/rss',          'name': 'guardian_world'},
    {'url': 'https://www.theguardian.com/science/rss',        'name': 'guardian_science'},
    {'url': 'https://www.theguardian.com/technology/rss',     'name': 'guardian_tech'},

    # ─── Tier D (0.70) — Quanta (high-quality science journalism) ───
    {'url': 'https://www.quantamagazine.org/feed/', 'name': 'quanta_magazine'},

    # ─── Tier E (0.60) — community signals ───
    {'url': 'https://hnrss.org/frontpage',  'name': 'hackernews_front'},
    {'url': 'https://hnrss.org/best',       'name': 'hackernews_best'},
]


# ═══════════════════════════════════════════════════════════════════════
# Status heartbeat
# ═══════════════════════════════════════════════════════════════════════

class Status:
    def __init__(self):
        self.started_ts = time.time()
        self.cycles_completed = 0
        self.total_items_seen = 0
        self.total_items_novel = 0
        self.total_items_corroborated = 0
        self.total_requests = 0
        self.total_robots_denied = 0
        self.total_errors = 0
        self.total_rate_limited = 0
        self.last_cycle_ts = None
        self.last_cycle_report = None
        self.stop_reason = None

    def uptime_hours(self) -> float:
        return (time.time() - self.started_ts) / 3600

    def to_dict(self) -> dict:
        return {
            'started_at': datetime.fromtimestamp(self.started_ts).isoformat(),
            'uptime_hours': round(self.uptime_hours(), 2),
            'cycles_completed': self.cycles_completed,
            'total_items_seen': self.total_items_seen,
            'total_items_novel': self.total_items_novel,
            'total_items_corroborated': self.total_items_corroborated,
            'total_requests': self.total_requests,
            'total_robots_denied': self.total_robots_denied,
            'total_rate_limited': self.total_rate_limited,
            'total_errors': self.total_errors,
            'last_cycle_at': (datetime.fromtimestamp(self.last_cycle_ts).isoformat()
                             if self.last_cycle_ts else None),
            'last_cycle_report': self.last_cycle_report,
            'stop_reason': self.stop_reason,
            'updated_ts': time.time(),
        }

    def save(self):
        STATUS_FILE.write_text(json.dumps(self.to_dict(), ensure_ascii=False,
                                           indent=2))


# ═══════════════════════════════════════════════════════════════════════
# Safety checks
# ═══════════════════════════════════════════════════════════════════════

def disk_free_gb() -> float:
    usage = shutil.disk_usage('/home/dinio/zets')
    return usage.free / (1024 ** 3)


def check_stop_conditions(status: Status) -> str | None:
    if STOP_FILE.exists():
        return 'STOP_FILE_PRESENT'
    if status.uptime_hours() >= MAX_HOURS:
        return f'MAX_HOURS_{MAX_HOURS}_REACHED'
    if status.total_items_novel >= MAX_ITEMS_TOTAL:
        return f'MAX_ITEMS_{MAX_ITEMS_TOTAL}_REACHED'
    df = disk_free_gb()
    if df < MIN_DISK_GB:
        return f'DISK_LOW_{df:.1f}GB_FREE'
    return None


# ═══════════════════════════════════════════════════════════════════════
# Cycle — one pass through the feed catalog
# ═══════════════════════════════════════════════════════════════════════

def run_cycle(cycle_num: int, status: Status) -> dict:
    fetcher = Fetcher()
    reg = HashRegistrySidecar(DATA_DIR / 'hash_registry_sidecar.json')

    # Shuffle feed order — round-robin is implicit through shuffling + polite delays
    feeds = FEED_CATALOG.copy()
    random.shuffle(feeds)

    cycle_report = {
        'cycle': cycle_num,
        'started_at': datetime.now().isoformat(),
        'feeds_attempted': 0,
        'feeds_succeeded': 0,
        'items_seen': 0,
        'items_novel': 0,
        'items_corroborated': 0,
        'per_feed': [],
    }

    # We don't need to hit ALL feeds every cycle; rotate 8-12 per cycle
    budget_feeds = random.randint(8, 12)
    feeds_this_cycle = feeds[:budget_feeds]

    for feed in feeds_this_cycle:
        if STOP_FILE.exists():
            break
        cycle_report['feeds_attempted'] += 1
        try:
            r = ingest_feed(fetcher, feed, reg)
            if r['status'] == 'ok':
                cycle_report['feeds_succeeded'] += 1
                cycle_report['items_seen'] += r['items_total']
                cycle_report['items_novel'] += r['items_novel']
                cycle_report['items_corroborated'] += r['items_corroborated']
                cycle_report['per_feed'].append({
                    'name': r['feed'], 'tier': r['tier'],
                    'total': r['items_total'], 'novel': r['items_novel'],
                    'corroborated': r['items_corroborated'],
                })

                # Persist novel items per cycle (jsonl)
                if r['novel_items']:
                    feed_path = DATA_DIR / f'night_cycle_{cycle_num:04d}_{feed["name"]}.jsonl'
                    with open(feed_path, 'w', encoding='utf-8') as f:
                        for item in r['novel_items']:
                            f.write(json.dumps(asdict(item), ensure_ascii=False) + '\n')
            else:
                cycle_report['per_feed'].append({
                    'name': feed['name'], 'status': r['status']
                })
        except Exception as e:
            cycle_report['per_feed'].append({
                'name': feed['name'], 'error': str(e)[:200]
            })

    reg.save()
    cycle_report['ended_at'] = datetime.now().isoformat()
    cycle_report['fetcher_stats'] = fetcher.stats

    # Update global status
    status.cycles_completed = cycle_num
    status.total_items_seen += cycle_report['items_seen']
    status.total_items_novel += cycle_report['items_novel']
    status.total_items_corroborated += cycle_report['items_corroborated']
    status.total_requests += fetcher.stats.get('requests', 0)
    status.total_robots_denied += fetcher.stats.get('robots_denied', 0)
    status.total_rate_limited += fetcher.stats.get('rate_limited_429', 0)
    status.total_errors += fetcher.stats.get('errors', 0)
    status.last_cycle_ts = time.time()
    status.last_cycle_report = {
        'cycle': cycle_num,
        'feeds_succeeded': cycle_report['feeds_succeeded'],
        'items_novel': cycle_report['items_novel'],
        'items_corroborated': cycle_report['items_corroborated'],
    }

    # Save cycle log
    log_path = LOGS_DIR / f'cycle_{cycle_num:04d}.json'
    log_path.write_text(json.dumps(cycle_report, ensure_ascii=False, indent=1))

    return cycle_report


# ═══════════════════════════════════════════════════════════════════════
# Main loop
# ═══════════════════════════════════════════════════════════════════════

def main():
    # Ensure only one instance
    pidfile = HERE / 'logs' / 'night.pid'
    if pidfile.exists():
        try:
            old_pid = int(pidfile.read_text().strip())
            os.kill(old_pid, 0)  # check alive
            print(f"night_learner already running as pid {old_pid}", file=sys.stderr)
            sys.exit(1)
        except (ProcessLookupError, ValueError, OSError):
            pass
    pidfile.write_text(str(os.getpid()))

    # Clear STOP flag from previous run (if any)
    if STOP_FILE.exists():
        STOP_FILE.unlink()

    # Graceful shutdown on SIGTERM
    status = Status()
    shutdown_requested = {'flag': False}

    def handler(signum, frame):
        shutdown_requested['flag'] = True
        status.stop_reason = f'SIGNAL_{signum}'
    signal.signal(signal.SIGTERM, handler)
    signal.signal(signal.SIGINT, handler)

    status.save()
    print(f"━━━ ZETS Night Learner — PID {os.getpid()} ━━━")
    print(f"  Feeds in catalog: {len(FEED_CATALOG)}")
    print(f"  Cycle interval:   {CYCLE_MINUTES} min")
    print(f"  Max runtime:      {MAX_HOURS} hours")
    print(f"  Max items:        {MAX_ITEMS_TOTAL}")
    print(f"  Stop file:        {STOP_FILE}")
    print()

    cycle_num = 0
    next_cycle_ts = time.time()

    try:
        while not shutdown_requested['flag']:
            # Safety check
            reason = check_stop_conditions(status)
            if reason:
                status.stop_reason = reason
                print(f"STOPPING: {reason}")
                break

            now = time.time()
            if now >= next_cycle_ts:
                cycle_num += 1
                print(f"[{datetime.now().isoformat()}] Starting cycle {cycle_num}")
                try:
                    report = run_cycle(cycle_num, status)
                    print(f"  ✓ cycle {cycle_num}: "
                          f"{report['feeds_succeeded']}/{report['feeds_attempted']} feeds, "
                          f"{report['items_novel']} novel, "
                          f"{report['items_corroborated']} corroborated")
                except Exception as e:
                    print(f"  ✗ cycle {cycle_num} failed: {e}")
                    traceback.print_exc()
                next_cycle_ts = now + CYCLE_MINUTES * 60

            # Heartbeat + sleep
            status.save()
            time.sleep(60)  # check every minute

    finally:
        if status.stop_reason is None:
            status.stop_reason = 'SHUTDOWN_REQUESTED'
        status.save()
        try:
            pidfile.unlink()
        except Exception:
            pass
        print(f"━━━ Exited. Reason: {status.stop_reason} ━━━")


if __name__ == '__main__':
    main()
