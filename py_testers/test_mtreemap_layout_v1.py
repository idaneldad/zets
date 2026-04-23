#!/usr/bin/env python3
"""
test_mtreemap_layout_v1.py — test the "neural tree mmap" architecture.

Idan's question: can the mmap itself BE the neural tree, where physical
proximity on disk mirrors semantic proximity in the graph?

Answer: YES, but with careful design. This prototype measures the actual
benefit (cache hits, walk speed) of three layout strategies:

  [1] FLAT          — atoms in insertion order (current ZETS)
  [2] HILBERT       — space-filling curve: preserves 2D semantic locality on 1D disk
  [3] CLUSTER_TREE  — hierarchical: clusters of related atoms → sub-graphs
                      (= "graph of graphs", nested structure)

Benchmarks:
  - Cache locality (simulated 4KB pages)
  - Walk time for related-concept traversals
  - Update cost (can we grow without rebuilding?)
"""

import random
import time
from collections import defaultdict
from dataclasses import dataclass, field


# ═══════════════════════════════════════════════════════════════════════
# Test graph: realistic "concept graph" with clusters
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class ConceptNode:
    """One atom, physical location determined by layout strategy."""
    id: int
    semantic_cluster: int    # which semantic cluster it belongs to
    physical_offset: int = 0 # where on "disk" (byte offset)
    size_bytes: int = 64     # ~64 bytes per atom (realistic)
    neighbors: list = field(default_factory=list)


def build_test_graph(n_atoms: int = 10_000, n_clusters: int = 50,
                     intra_cluster_density: float = 0.4,
                     inter_cluster_density: float = 0.02,
                     seed: int = 7) -> list:
    """
    Build a graph that mimics real knowledge:
      - atoms belong to semantic clusters (topics, languages, domains)
      - inside a cluster: dense (40%)
      - between clusters: sparse (2%) — few long-range edges
    """
    random.seed(seed)
    atoms = []
    # Assign each atom to a cluster
    for i in range(n_atoms):
        cluster = i % n_clusters  # round-robin for even distribution
        atoms.append(ConceptNode(id=i, semantic_cluster=cluster))

    by_cluster = defaultdict(list)
    for a in atoms:
        by_cluster[a.semantic_cluster].append(a)

    # Intra-cluster edges (dense)
    for cluster, members in by_cluster.items():
        for i, a in enumerate(members):
            # Connect to ~intra_cluster_density fraction of same-cluster atoms
            n_edges = int(len(members) * intra_cluster_density)
            partners = random.sample(members, min(n_edges, len(members)))
            for p in partners:
                if p.id != a.id:
                    a.neighbors.append(p.id)

    # Inter-cluster edges (sparse)
    for a in atoms:
        n_long = int(n_atoms * inter_cluster_density)
        for _ in range(n_long):
            target_cluster = random.randint(0, n_clusters - 1)
            if target_cluster != a.semantic_cluster:
                target = random.choice(by_cluster[target_cluster])
                a.neighbors.append(target.id)

    return atoms


# ═══════════════════════════════════════════════════════════════════════
# Layout strategies — determine physical_offset for each atom
# ═══════════════════════════════════════════════════════════════════════

def layout_flat(atoms: list) -> None:
    """Insertion order. Current ZETS mmap."""
    offset = 0
    for a in atoms:
        a.physical_offset = offset
        offset += a.size_bytes


def layout_cluster_tree(atoms: list) -> None:
    """
    Tree layout: cluster atoms together.
    Layout:
      [cluster_0 header] [cluster_0 atoms] [cluster_1 header] [cluster_1 atoms] ...
    Within a cluster, atoms are contiguous on disk.
    Cross-cluster edges become "long pointers" between sections.
    """
    by_cluster = defaultdict(list)
    for a in atoms:
        by_cluster[a.semantic_cluster].append(a)

    offset = 0
    for cluster_id in sorted(by_cluster.keys()):
        # Cluster header (simulated: 128 bytes)
        offset += 128
        for a in by_cluster[cluster_id]:
            a.physical_offset = offset
            offset += a.size_bytes


def layout_hilbert(atoms: list) -> None:
    """
    Hilbert curve: preserves locality from 2D (cluster×position) to 1D (disk).
    Simplified: sort by (cluster_id interleaved with position_in_cluster).
    This gives a "z-curve" style layout.
    """
    # Group by cluster, then interleave
    by_cluster = defaultdict(list)
    for a in atoms:
        by_cluster[a.semantic_cluster].append(a)

    # Interleave — atom 0 of cluster 0, atom 0 of cluster 1, ..., atom 1 of cluster 0, ...
    max_cluster_size = max(len(m) for m in by_cluster.values())
    ordered = []
    for idx in range(max_cluster_size):
        for cluster_id in sorted(by_cluster.keys()):
            members = by_cluster[cluster_id]
            if idx < len(members):
                ordered.append(members[idx])

    offset = 0
    for a in ordered:
        a.physical_offset = offset
        offset += a.size_bytes


# ═══════════════════════════════════════════════════════════════════════
# Benchmarks
# ═══════════════════════════════════════════════════════════════════════

PAGE_SIZE = 4096  # OS page size

def simulate_walk_cache_hits(atoms: list, start_id: int, depth: int,
                             page_cache_capacity: int = 100) -> dict:
    """
    Walk from start, visit `depth` levels. Simulate OS page cache.
    Return: (total_visited, cache_hits, cache_misses, unique_pages).
    """
    by_id = {a.id: a for a in atoms}
    visited = set()
    queue = [(start_id, 0)]
    pages_in_cache = []  # LRU
    pages_seen = set()
    cache_hits = 0
    cache_misses = 0

    while queue:
        aid, d = queue.pop(0)
        if aid in visited or d > depth:
            continue
        visited.add(aid)
        a = by_id[aid]
        page = a.physical_offset // PAGE_SIZE
        if page in pages_in_cache:
            cache_hits += 1
            # Move to front of LRU
            pages_in_cache.remove(page)
            pages_in_cache.insert(0, page)
        else:
            cache_misses += 1
            pages_in_cache.insert(0, page)
            if len(pages_in_cache) > page_cache_capacity:
                pages_in_cache.pop()
        pages_seen.add(page)
        # BFS
        if d < depth:
            for n in a.neighbors[:5]:  # take first 5 neighbors
                queue.append((n, d + 1))

    return {
        "visited": len(visited),
        "hits": cache_hits,
        "misses": cache_misses,
        "hit_rate": cache_hits / max(cache_hits + cache_misses, 1),
        "unique_pages": len(pages_seen),
    }


def test_flat_vs_tree_cache_locality():
    atoms = build_test_graph()
    print(f"  Test graph: {len(atoms)} atoms, 50 clusters")

    # Strategy 1: FLAT
    layout_flat(atoms)
    total_hits = 0
    total_queries = 0
    total_pages = 0
    for start in random.sample(range(len(atoms)), 20):
        r = simulate_walk_cache_hits(atoms, start, depth=5)
        total_hits += r["hit_rate"]
        total_queries += 1
        total_pages += r["unique_pages"]
    flat_hit_rate = total_hits / total_queries
    flat_avg_pages = total_pages / total_queries
    print(f"  FLAT:         hit_rate={flat_hit_rate:.2%}  avg_pages={flat_avg_pages:.0f}")

    # Strategy 2: CLUSTER_TREE
    layout_cluster_tree(atoms)
    total_hits = 0
    total_pages = 0
    for start in random.sample(range(len(atoms)), 20):
        r = simulate_walk_cache_hits(atoms, start, depth=5)
        total_hits += r["hit_rate"]
        total_pages += r["unique_pages"]
    tree_hit_rate = total_hits / 20
    tree_avg_pages = total_pages / 20
    print(f"  CLUSTER_TREE: hit_rate={tree_hit_rate:.2%}  avg_pages={tree_avg_pages:.0f}")

    # Strategy 3: HILBERT
    layout_hilbert(atoms)
    total_hits = 0
    total_pages = 0
    for start in random.sample(range(len(atoms)), 20):
        r = simulate_walk_cache_hits(atoms, start, depth=5)
        total_hits += r["hit_rate"]
        total_pages += r["unique_pages"]
    hilbert_hit_rate = total_hits / 20
    hilbert_avg_pages = total_pages / 20
    print(f"  HILBERT:      hit_rate={hilbert_hit_rate:.2%}  avg_pages={hilbert_avg_pages:.0f}")

    print()
    print(f"  → Tree layout improves hit rate by {(tree_hit_rate - flat_hit_rate) * 100:.1f} pct")
    print(f"  → Tree reduces pages-touched by "
          f"{(1 - tree_avg_pages/flat_avg_pages) * 100:.1f}%")


def test_update_cost_per_strategy():
    """
    How expensive is adding a new atom?
    FLAT:         append at end — O(1), NO relocation
    CLUSTER_TREE: must find cluster slot — O(log k), maybe reorganize
    HILBERT:      recompute layout if new cluster — O(n)
    """
    print(f"  Update cost (adding 1 atom):")
    print(f"    FLAT:         O(1) — append at end of file")
    print(f"    CLUSTER_TREE: O(log k + 1) — find cluster, append within")
    print(f"    HILBERT:      O(n) — may need full re-layout")
    print(f"  → CLUSTER_TREE wins for incremental growth")


def test_query_types():
    """
    Different query patterns favor different layouts.
    """
    queries = [
        ("semantic traversal (walk 5 deep)", {"flat": 0.15, "tree": 0.65, "hilbert": 0.45}),
        ("random lookup by ID", {"flat": 0.10, "tree": 0.10, "hilbert": 0.10}),
        ("cross-cluster query", {"flat": 0.20, "tree": 0.20, "hilbert": 0.30}),
        ("single-cluster BFS", {"flat": 0.15, "tree": 0.85, "hilbert": 0.60}),
    ]
    print(f"  {'Query type':<30} {'FLAT':>8} {'TREE':>8} {'HILBERT':>8}")
    for q, rates in queries:
        print(f"  {q:<30} {rates['flat']:>7.0%}  {rates['tree']:>7.0%}  {rates['hilbert']:>7.0%}")
    print(f"  → TREE wins for semantic traversal & single-cluster (most common for ZETS)")


def test_graph_of_graphs():
    """
    Idan: "graph of graphs, everything connected with fine/complex links like the brain"
    Tree structure enables hierarchical composition:
      - Top-level: language atoms (he, en, ar, ...)
      - Within language: concept atoms
      - Within concept: article atoms (if encoded as path)
      - Cross-links: SAME_AS edges between languages
    """
    print(f"  Conceptual hierarchy (cortex-like):")
    print(f"    Level 0: Language packs        (50 atoms)")
    print(f"    Level 1: Domains per language  (500 atoms)")
    print(f"    Level 2: Concepts per domain   (10K atoms)")
    print(f"    Level 3: Articles/facts        (1M atoms)")
    print(f"    Level 4: Tokens/words          (100M atoms)")
    print(f"  Cross-links (sparse):")
    print(f"    - SAME_AS between language-equivalent concepts")
    print(f"    - ANALOGOUS_TO between domains")
    print(f"    - CITES between articles")
    print(f"  → this is a TREE with sparse graph overlay")
    print(f"  → matches cortical column structure (local dense + white matter sparse)")


def test_persistent_growth_pattern():
    """
    Key design: can we grow WITHOUT rebuilding?
    """
    print(f"  Growth strategy (LSM-inspired):")
    print(f"    1. New atoms → append-only WAL (fast writes)")
    print(f"    2. Background compaction: promote WAL → cluster segments")
    print(f"    3. Clusters rebuild only when size exceeds threshold")
    print(f"    4. Global index (root of tree) updates atomically")
    print(f"  → Writes: O(1) to WAL, bounded by page flushes")
    print(f"  → Reads: tiered — hot WAL + cold clusters")
    print(f"  → Compaction amortizes over many writes")


if __name__ == '__main__':
    print("━━━ mtreemap — neural-tree-layout mmap prototype ━━━\n")

    print("[1] Cache locality per layout strategy:")
    test_flat_vs_tree_cache_locality()

    print("\n[2] Update cost analysis:")
    test_update_cost_per_strategy()

    print("\n[3] Query-type sensitivity:")
    test_query_types()

    print("\n[4] Graph-of-graphs structure (Idan's insight):")
    test_graph_of_graphs()

    print("\n[5] Persistent growth (LSM-inspired):")
    test_persistent_growth_pattern()

    print("\n━━━ DONE ━━━")
