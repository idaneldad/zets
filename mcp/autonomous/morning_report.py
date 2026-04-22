#!/usr/bin/env python3
"""
morning_report.py — what happened last night.

Reads logs/night/*.json + status.json + hash_registry_sidecar.json
and produces a human-readable morning report for Idan.

Usage:
  python3 mcp/autonomous/morning_report.py
  python3 mcp/autonomous/morning_report.py > /tmp/last_night.txt
"""

import json
import os
import sys
from collections import Counter
from datetime import datetime
from pathlib import Path

HERE = Path(os.path.dirname(os.path.abspath(__file__)))
LOGS_DIR = HERE / 'logs' / 'night'
STATUS_FILE = HERE / 'status.json'
DATA_DIR = Path('/home/dinio/zets/data/autonomous')


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text())
    except Exception:
        return {}


def main():
    print("╔══════════════════════════════════════════════════════════════╗")
    print("║       ZETS Night Learner — Morning Report                     ║")
    print("║       " + datetime.now().strftime('%A, %Y-%m-%d %H:%M') + "                              ║")
    print("╚══════════════════════════════════════════════════════════════╝")
    print()

    # ── Overall status ──
    status = load_json(STATUS_FILE)
    if not status:
        print("  (no status file found — learner didn't run)")
        return

    print(f"  Started:     {status.get('started_at', '?')}")
    print(f"  Uptime:      {status.get('uptime_hours', 0):.2f} hours")
    print(f"  Stop reason: {status.get('stop_reason') or '(still running)'}")
    print(f"  Cycles:      {status.get('cycles_completed', 0)}")
    print()
    print(f"  Items seen:         {status.get('total_items_seen', 0):>7,}")
    print(f"  Items novel:        {status.get('total_items_novel', 0):>7,}  ← unique new facts")
    print(f"  Items corroborated: {status.get('total_items_corroborated', 0):>7,}  ← same fact, another source")
    print()
    print(f"  HTTP requests:      {status.get('total_requests', 0):>7,}")
    print(f"  robots.txt denied:  {status.get('total_robots_denied', 0):>7,}  (politeness working)")
    print(f"  429 rate-limited:   {status.get('total_rate_limited', 0):>7,}  (lower is better)")
    print(f"  Errors:             {status.get('total_errors', 0):>7,}")
    print()

    # ── Hash registry stats ──
    reg_path = DATA_DIR / 'hash_registry_sidecar.json'
    reg = load_json(reg_path)
    by_hash = reg.get('by_hash', {})
    if by_hash:
        total = len(by_hash)
        shared_2 = sum(1 for v in by_hash.values() if len(v) >= 2)
        shared_3 = sum(1 for v in by_hash.values() if len(v) >= 3)
        shared_5 = sum(1 for v in by_hash.values() if len(v) >= 5)
        print(f"━━━ Hash Registry (dedup + corroboration) ━━━")
        print(f"  Unique hashes:           {total:>7,}")
        print(f"  With 2+ sources:         {shared_2:>7,}  ({100*shared_2/max(total,1):.1f}%)")
        print(f"  With 3+ sources:         {shared_3:>7,}")
        print(f"  With 5+ sources:         {shared_5:>7,}")
        print()

        # Top corroborated facts (5+ sources)
        top = sorted(
            ((h, v) for h, v in by_hash.items() if len(v) >= 3),
            key=lambda x: -len(x[1])
        )[:10]
        if top:
            print(f"  Top cross-source facts (by # of sources):")
            for h, sources in top:
                domains = [s['source'] for s in sources]
                max_conf = max(s['confidence'] for s in sources)
                print(f"    {len(sources)}× conf={max_conf:.2f}  "
                      f"domains: {', '.join(sorted(set(domains))[:5])}")
            print()

    # ── Per-source totals ──
    source_counter = Counter()
    for v in by_hash.values():
        for s in v:
            source_counter[s['source']] += 1

    if source_counter:
        print(f"━━━ Items per source domain ━━━")
        for domain, count in source_counter.most_common(20):
            print(f"  {domain:<40}  {count:>7,}")
        print()

    # ── Cycle summary ──
    cycles = sorted(LOGS_DIR.glob('cycle_*.json'))
    if cycles:
        print(f"━━━ Cycles ({len(cycles)}) ━━━")
        total_novel = 0
        total_feeds = 0
        for cf in cycles:
            c = load_json(cf)
            total_novel += c.get('items_novel', 0)
            total_feeds += c.get('feeds_succeeded', 0)
        print(f"  Total cycles:     {len(cycles)}")
        print(f"  Total feeds run:  {total_feeds}")
        print(f"  Novel items:      {total_novel:,}")
        if cycles:
            # Show last 3 cycles
            print(f"  Last 3 cycles:")
            for cf in cycles[-3:]:
                c = load_json(cf)
                print(f"    #{c.get('cycle', '?'):>3}  "
                      f"{c.get('feeds_succeeded', 0):>2}/{c.get('feeds_attempted', 0)} feeds, "
                      f"{c.get('items_novel', 0):>4} novel, "
                      f"{c.get('items_corroborated', 0):>3} corrob")
        print()

    # ── Items by tier ──
    cycle_files = sorted(DATA_DIR.glob('night_cycle_*.jsonl'))
    if cycle_files:
        tier_count = Counter()
        tier_bytes = Counter()
        for cf in cycle_files:
            try:
                with open(cf) as f:
                    for line in f:
                        try:
                            item = json.loads(line)
                            tier_count[item.get('source_tier', '?')] += 1
                            tier_bytes[item.get('source_tier', '?')] += len(line)
                        except Exception:
                            pass
            except Exception:
                pass
        if tier_count:
            print(f"━━━ Novel items by tier ━━━")
            for tier in ['A', 'B', 'C', 'D', 'E', 'F', 'X']:
                if tier in tier_count:
                    priors = {'A': 0.90, 'B': 0.80, 'C': 0.75, 'D': 0.70,
                             'E': 0.60, 'F': 0.40, 'X': 0.20}
                    print(f"  Tier {tier} ({priors.get(tier, '?'):.2f} prior):  "
                          f"{tier_count[tier]:>5,} items  "
                          f"({tier_bytes[tier]/1024:.0f} KB)")
            print()

    # ── Estimate of knowledge added ──
    snap_path = Path('/home/dinio/zets/data/baseline/wiki_all_domains_v1.atoms')
    if snap_path.exists():
        before_bytes = snap_path.stat().st_size
        print(f"━━━ Context ━━━")
        print(f"  Current wiki snapshot:   {before_bytes/1024/1024:.1f} MB "
              f"(211,650 atoms, 13.2M edges)")
        print(f"  Night-harvested items:   {status.get('total_items_novel', 0):,}  "
              f"(not yet ingested into snapshot)")
        print(f"  → To ingest these: run `scripts/ingest_night_items.sh` (separate step)")
        print()

    # ── Honest assessment ──
    print(f"━━━ Honest Reality Check ━━━")
    hours = status.get('uptime_hours', 0)
    items = status.get('total_items_novel', 0)
    if hours > 0:
        rate = items / hours
        print(f"  Rate:                {rate:,.0f} novel items/hour")
        print(f"  Daily projection:    {rate*24:,.0f} items/day")
        print(f"  Annual projection:   {rate*24*365/1e6:.1f}M items/year")
    print(f"  'Trillion' target:   not realistic on single server")
    print(f"  'Billion' target:    reachable in ~2-5 years at this pace + Wikipedia dumps")
    print()
    print(f"  For mass ingestion: Wikipedia dumps are the right path.")
    print(f"  HE dump: ~800MB compressed. See: data/wikipedia_dumps/ (empty)")
    print(f"  EN dump: ~22GB compressed. Requires ~110GB disk.")


if __name__ == '__main__':
    main()
