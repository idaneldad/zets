#!/usr/bin/env python3
"""
source_tiers.py — the prior-confidence dictionary for autonomous learning.

Each domain is tagged with a tier (A-F, X) based on structural signals:
  A  — Peer-reviewed, structured (arXiv, PubMed, OpenAlex)
  B  — Reference/encyclopedic (Wikipedia, Britannica)
  C  — Primary source, gov/edu (.gov, .edu, NASA, WHO)
  D  — Established journalism (Reuters, BBC, AP)
  E  — Industry/organizational (company blogs with editorial)
  F  — Personal/social (Medium, Twitter, Reddit)
  X  — Unknown / suspicious (default for unrecognized)

The tier gives a PRIOR confidence. Final confidence = prior × corroboration
evidence × contradiction-adjustment.

This is a SEED. The real system should learn tier assignments from signals
(HTTPS, age, PageRank, citation count). For now, 100 hand-picked domains.
"""

from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class TierInfo:
    tier: str              # 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'X'
    prior_confidence: float  # 0.0 - 1.0
    description: str


TIER_PRIORS: dict[str, TierInfo] = {
    'A': TierInfo('A', 0.90, 'Peer-reviewed scholarly'),
    'B': TierInfo('B', 0.80, 'Reference / encyclopedic'),
    'C': TierInfo('C', 0.75, 'Primary source, gov/edu'),
    'D': TierInfo('D', 0.70, 'Established journalism'),
    'E': TierInfo('E', 0.60, 'Industry / organizational'),
    'F': TierInfo('F', 0.40, 'Personal / social / community'),
    'X': TierInfo('X', 0.20, 'Unknown / unrated'),
}


# ═══════════════════════════════════════════════════════════════════════
# Seed catalog — hand-picked as of 23.04.2026
# ═══════════════════════════════════════════════════════════════════════

DOMAIN_TIERS: dict[str, str] = {
    # ─── Tier A — peer-reviewed scholarly ───
    'arxiv.org': 'A',
    'pubmed.ncbi.nlm.nih.gov': 'A',
    'pmc.ncbi.nlm.nih.gov': 'A',
    'openalex.org': 'A',
    'doi.org': 'A',
    'nature.com': 'A',
    'science.org': 'A',
    'cell.com': 'A',
    'plos.org': 'A',
    'ieeexplore.ieee.org': 'A',
    'acm.org': 'A',
    'springer.com': 'A',
    'sciencedirect.com': 'A',
    'biorxiv.org': 'A',
    'medrxiv.org': 'A',

    # ─── Tier B — reference / encyclopedic ───
    'en.wikipedia.org': 'B',
    'he.wikipedia.org': 'B',
    'wikidata.org': 'B',
    'commons.wikimedia.org': 'B',
    'conceptnet.io': 'B',
    'wordnet.princeton.edu': 'B',
    'britannica.com': 'B',
    'stanford.edu/entries': 'B',  # Stanford Encyclopedia of Philosophy
    'iep.utm.edu': 'B',            # Internet Encyclopedia of Philosophy
    'sep.stanford.edu': 'B',

    # ─── Tier C — primary source, gov/edu ───
    'nasa.gov': 'C',
    'nih.gov': 'C',
    'who.int': 'C',
    'cdc.gov': 'C',
    'fda.gov': 'C',
    'usgs.gov': 'C',
    'noaa.gov': 'C',
    'epa.gov': 'C',
    'eia.gov': 'C',                # Energy Info Admin
    'bls.gov': 'C',                # Bureau of Labor Stats
    'census.gov': 'C',
    'europa.eu': 'C',
    'knesset.gov.il': 'C',
    'cbs.gov.il': 'C',             # Israel CBS
    'gov.il': 'C',
    'data.gov': 'C',
    'data.gov.uk': 'C',
    'mit.edu': 'C',
    'stanford.edu': 'C',
    'harvard.edu': 'C',
    'cam.ac.uk': 'C',
    'ox.ac.uk': 'C',
    'tau.ac.il': 'C',
    'huji.ac.il': 'C',
    'technion.ac.il': 'C',
    'weizmann.ac.il': 'C',

    # ─── Tier D — established journalism ───
    'reuters.com': 'D',
    'apnews.com': 'D',
    'bbc.com': 'D',
    'bbc.co.uk': 'D',
    'nytimes.com': 'D',
    'washingtonpost.com': 'D',
    'theguardian.com': 'D',
    'wsj.com': 'D',
    'ft.com': 'D',
    'economist.com': 'D',
    'bloomberg.com': 'D',
    'aljazeera.com': 'D',
    'haaretz.co.il': 'D',
    'ynet.co.il': 'D',
    'timesofisrael.com': 'D',
    'npr.org': 'D',
    'pbs.org': 'D',
    'dw.com': 'D',                  # Deutsche Welle
    'france24.com': 'D',
    'politico.eu': 'D',

    # ─── Tier E — industry / orgs ───
    'stackexchange.com': 'E',
    'stackoverflow.com': 'E',
    'github.com': 'E',
    'hackernews.com': 'E',
    'news.ycombinator.com': 'E',
    'microsoft.com': 'E',
    'google.com/research': 'E',
    'openai.com/research': 'E',
    'anthropic.com/research': 'E',
    'deepmind.com/research': 'E',
    'mozilla.org': 'E',
    'linuxfoundation.org': 'E',
    'apache.org': 'E',
    'iana.org': 'E',
    'w3.org': 'E',
    'ietf.org': 'E',

    # ─── Tier F — personal / community ───
    'medium.com': 'F',
    'substack.com': 'F',
    'dev.to': 'F',
    'blogspot.com': 'F',
    'wordpress.com': 'F',
    'reddit.com': 'F',
    'twitter.com': 'F',
    'x.com': 'F',
    'mastodon.social': 'F',
    'linkedin.com': 'F',
    'youtube.com': 'F',
    'tiktok.com': 'F',
    'quora.com': 'F',

    # ─── Additions (23.04.26) — common RSS subdomains + more ───
    'bbci.co.uk': 'D',
    'feeds.bbci.co.uk': 'D',
    'feeds.reuters.com': 'D',
    'rss.nytimes.com': 'D',
    'rss.arxiv.org': 'A',
    'export.arxiv.org': 'A',
    'en.wiktionary.org': 'B',
    'he.wiktionary.org': 'B',
    'ieeespectrum.org': 'D',
    'spectrum.ieee.org': 'D',
    'nature.com/articles': 'A',
    'scientificamerican.com': 'D',
    'quantamagazine.org': 'D',
    'techcrunch.com': 'E',
    'theverge.com': 'D',
    'wired.com': 'D',
    'arstechnica.com': 'D',
    'statista.com': 'E',
    'ourworldindata.org': 'C',
    'opencagedata.com': 'E',
    'osm.org': 'B',
    'openstreetmap.org': 'B',
}


# ═══════════════════════════════════════════════════════════════════════
# Lookup with domain-suffix matching
# ═══════════════════════════════════════════════════════════════════════

def tier_of(hostname: str) -> TierInfo:
    """Return tier info for a hostname. Falls back to suffix rules, then 'X'."""
    h = hostname.lower().strip().lstrip('.')
    # Exact match
    if h in DOMAIN_TIERS:
        return TIER_PRIORS[DOMAIN_TIERS[h]]
    # Check parent domains
    parts = h.split('.')
    for i in range(len(parts)):
        candidate = '.'.join(parts[i:])
        if candidate in DOMAIN_TIERS:
            return TIER_PRIORS[DOMAIN_TIERS[candidate]]
    # Suffix rules
    if h.endswith('.gov') or '.gov.' in h:
        return TIER_PRIORS['C']
    if h.endswith('.edu') or '.ac.' in h or h.endswith('.ac'):
        return TIER_PRIORS['C']
    if h.endswith('.org') and not any(
        h.endswith(f'.{d}') for d in ('medium.com', 'blogspot.com')
    ):
        # .org is a weak signal — bump to E not D
        return TIER_PRIORS['E']
    # Everything else
    return TIER_PRIORS['X']


def domain_from_url(url: str) -> Optional[str]:
    """Extract hostname from URL."""
    from urllib.parse import urlparse
    try:
        return urlparse(url).hostname
    except Exception:
        return None


# ═══════════════════════════════════════════════════════════════════════
# Self-test
# ═══════════════════════════════════════════════════════════════════════

if __name__ == '__main__':
    test_urls = [
        'https://arxiv.org/abs/2024.12345',
        'https://en.wikipedia.org/wiki/Python',
        'https://www.nasa.gov/mission_pages/cassini',
        'https://www.reuters.com/world/us/story',
        'https://medium.com/@someone/post',
        'https://some-unknown-blog.info/article',
        'https://gov.il/ministry',
        'https://mit.edu/research',
        'https://www.ynet.co.il/news/123',
    ]
    for url in test_urls:
        h = domain_from_url(url)
        t = tier_of(h) if h else TIER_PRIORS['X']
        print(f"  {url:<55}  tier={t.tier}  prior={t.prior_confidence:.2f}")
    print(f"\n  Total cataloged domains: {len(DOMAIN_TIERS)}")
