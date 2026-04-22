#!/usr/bin/env python3
"""
trust_scorer.py — compute per-fact confidence from source tiers + corroboration.

Formula (Bayesian-inspired, simple):
  base = max(prior_confidence over all sources)       # best single source
  corroboration_boost = 0.05 * log2(N)                 # N sources
  diversity_boost = 0.10 if >= 2 different tiers agree  # independence

  confidence = min(0.99, base + corroboration_boost + diversity_boost)

Capped at 0.99 — nothing is certain from web scraping alone.

Usage:
  python3 trust_scorer.py  # scores all items in the latest registry
"""

import json
import math
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from source_tiers import tier_of, DOMAIN_TIERS, TIER_PRIORS  # noqa

DATA_DIR = Path('/home/dinio/zets/data/autonomous')
REGISTRY_PATH = DATA_DIR / 'hash_registry_sidecar.json'


def score_fact(sources: list) -> dict:
    """Given list of {source, confidence, ts}, compute trust score."""
    if not sources:
        return {'confidence': 0.0, 'num_sources': 0, 'num_tiers': 0}

    base = max(s['confidence'] for s in sources)
    n = len(sources)
    corroboration_boost = 0.05 * math.log2(n) if n > 1 else 0.0

    # Count unique tiers
    tier_set = set()
    for s in sources:
        # source name doesn't directly tell us tier — but we can map via registry
        # For this prototype, use the confidence as a tier proxy
        tier_set.add(round(s['confidence'] * 10) / 10)
    diversity_boost = 0.10 if len(tier_set) >= 2 else 0.0

    confidence = min(0.99, base + corroboration_boost + diversity_boost)
    return {
        'confidence': round(confidence, 3),
        'base_prior': round(base, 3),
        'num_sources': n,
        'num_tiers': len(tier_set),
        'corroboration_boost': round(corroboration_boost, 3),
        'diversity_boost': round(diversity_boost, 3),
    }


def main():
    if not REGISTRY_PATH.exists():
        print(f"No registry at {REGISTRY_PATH}")
        return
    reg = json.loads(REGISTRY_PATH.read_text())
    by_hash = reg.get('by_hash', {})

    # Score every fact
    scored = []
    for h, sources in by_hash.items():
        s = score_fact(sources)
        s['hash'] = h
        s['sources'] = [x['source'] for x in sources]
        scored.append(s)

    # Distribution
    buckets = {'0.9-0.99': 0, '0.8-0.9': 0, '0.7-0.8': 0, '0.6-0.7': 0,
               '0.5-0.6': 0, '0.4-0.5': 0, '<0.4': 0}
    for s in scored:
        c = s['confidence']
        if c >= 0.9:
            buckets['0.9-0.99'] += 1
        elif c >= 0.8:
            buckets['0.8-0.9'] += 1
        elif c >= 0.7:
            buckets['0.7-0.8'] += 1
        elif c >= 0.6:
            buckets['0.6-0.7'] += 1
        elif c >= 0.5:
            buckets['0.5-0.6'] += 1
        elif c >= 0.4:
            buckets['0.4-0.5'] += 1
        else:
            buckets['<0.4'] += 1

    multi = [s for s in scored if s['num_sources'] >= 2]
    multi_sorted = sorted(multi, key=lambda s: -s['confidence'])

    print(f"━━━ Trust Score Distribution ({len(scored)} facts) ━━━")
    print()
    total = len(scored)
    for k, v in buckets.items():
        bar = '█' * int(v * 50 / max(total, 1))
        pct = 100 * v / max(total, 1)
        print(f"  conf {k:<10} {v:>5}  {pct:5.1f}%  {bar}")
    print()
    print(f"━━━ Corroborated facts ({len(multi)} with 2+ sources) ━━━")
    print()
    for s in multi_sorted[:10]:
        print(f"  conf={s['confidence']}  sources={s['num_sources']:2d}  "
              f"base={s['base_prior']}  +corr={s['corroboration_boost']}  "
              f"+div={s['diversity_boost']}")
        print(f"     from: {', '.join(s['sources'][:4])}"
              + (f" (+{s['num_sources']-4} more)" if s['num_sources'] > 4 else ''))

    # Save scored report
    out = DATA_DIR / 'trust_scores.json'
    out.write_text(json.dumps({
        'total_facts': total,
        'distribution': buckets,
        'corroborated_count': len(multi),
        'top_corroborated_sample': multi_sorted[:20],
    }, ensure_ascii=False, indent=2))
    print(f"\n  Report: {out}")


if __name__ == '__main__':
    main()
