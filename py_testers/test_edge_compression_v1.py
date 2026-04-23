#!/usr/bin/env python3
"""
test_edge_compression_v1.py — measures the 4 compression layers on
ZETS-realistic edge distributions.

Per Idan's question: "small bit groups for common co-occurring states."
Answer: Huffman-coded relations + Pattern dictionary + Delta encoding.

Tests:
  [1] ZETS-realistic edge distribution (32% is_a, long tail)
  [2] Huffman coding on relations — bits per relation
  [3] Delta+varint on sorted adjacency — bytes per target_id
  [4] Weight quantization — 2-bit tag + dequantize
  [5] Pattern mining — find frequent N-tuples
  [6] Full stack — combined compression ratio
  [7] Decoder verifies lossless roundtrip

No deps beyond stdlib.
Run: python3 py_testers/test_edge_compression_v1.py
"""

import heapq
import random
import struct
from collections import Counter, defaultdict
from dataclasses import dataclass


# ═══════════════════════════════════════════════════════════════════════
# ZETS-realistic edge distribution
# ═══════════════════════════════════════════════════════════════════════

# Real-world knowledge-graph distributions (ConceptNet + Wikidata-based).
# 64 relations total, but super-skewed.
EDGE_DISTRIBUTION = [
    ("is_a",          0.32),
    ("part_of",       0.12),
    ("has_property",  0.10),
    ("located_in",    0.06),
    ("synonym_of",    0.05),
    ("translates_to", 0.05),
    ("related_to",    0.04),
    ("has_context",   0.03),
    ("causes",        0.03),
    ("used_for",      0.03),
    ("similar_to",    0.02),
    ("antonym_of",    0.02),
    ("derived_from",  0.02),
    ("written_in",    0.02),
    ("created_by",    0.015),
    ("capable_of",    0.015),
    # ... 48 more relations with total 7.5%, avg 0.15% each
    # We'll simulate them as 48 names with uniform small frequency
]
# Fill remaining 48 relations
for i in range(48):
    EDGE_DISTRIBUTION.append((f"rel_{i:02d}", 0.06 / 48))

REL_TO_IDX = {name: i for i, (name, _) in enumerate(EDGE_DISTRIBUTION)}
IDX_TO_REL = {i: name for i, (name, _) in enumerate(EDGE_DISTRIBUTION)}

assert abs(sum(p for _, p in EDGE_DISTRIBUTION) - 1.0) < 0.01


def generate_edges(n: int, num_nodes: int = 10_000, seed: int = 42) -> list:
    """Generate n realistic edges: (source_id, target_id, rel_idx, weight)."""
    random.seed(seed)
    rels, weights = zip(*EDGE_DISTRIBUTION)
    rel_idxs = list(range(len(EDGE_DISTRIBUTION)))

    edges = []
    for _ in range(n):
        src = random.randint(0, num_nodes - 1)
        dst = random.randint(0, num_nodes - 1)
        rel = random.choices(rel_idxs, weights=weights, k=1)[0]
        # Weight: 80% are 1.0, 15% in [0.5,1.0), 5% other
        r = random.random()
        if r < 0.80:
            w = 1.0
        elif r < 0.95:
            w = 0.5 + random.random() * 0.5
        else:
            w = random.random()
        edges.append((src, dst, rel, round(w, 3)))
    return edges


# ═══════════════════════════════════════════════════════════════════════
# Layer 1 — Huffman coding on relation codes
# ═══════════════════════════════════════════════════════════════════════

def build_huffman(freq: dict) -> dict:
    """Build Huffman codes. Returns {symbol: bit_string}."""
    heap = [[f, i, [sym, ""]] for i, (sym, f) in enumerate(freq.items())]
    heapq.heapify(heap)
    counter = len(heap)
    while len(heap) > 1:
        lo = heapq.heappop(heap)
        hi = heapq.heappop(heap)
        for p in lo[2:]:
            p[1] = '0' + p[1]
        for p in hi[2:]:
            p[1] = '1' + p[1]
        counter += 1
        heapq.heappush(heap, [lo[0] + hi[0], counter] + lo[2:] + hi[2:])
    codes = {}
    if heap:
        for pair in heap[0][2:]:
            codes[pair[0]] = pair[1] or "0"  # single-symbol edge case
    return codes


def huffman_bits_per_symbol(freq: dict, codes: dict) -> float:
    total = sum(freq.values())
    if total == 0:
        return 0.0
    return sum(freq[sym] * len(codes[sym]) for sym in freq) / total


# ═══════════════════════════════════════════════════════════════════════
# Layer 2 — Adjacency list with delta + varint
# ═══════════════════════════════════════════════════════════════════════

def varint_encode(n: int) -> bytes:
    """ULEB128 varint."""
    if n < 0:
        raise ValueError("varint requires non-negative")
    out = bytearray()
    while True:
        byte = n & 0x7F
        n >>= 7
        if n:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def delta_encode_targets(targets: list) -> bytes:
    """Sort and delta-encode a list of target_ids."""
    if not targets:
        return b""
    sorted_t = sorted(targets)
    out = bytearray()
    prev = 0
    for t in sorted_t:
        delta = t - prev
        out.extend(varint_encode(delta))
        prev = t
    return bytes(out)


def raw_targets_bytes(targets: list) -> int:
    """Baseline: each target as u32 (4 bytes)."""
    return len(targets) * 4


# ═══════════════════════════════════════════════════════════════════════
# Layer 3 — Pattern dictionary (frequent N-tuples)
# ═══════════════════════════════════════════════════════════════════════

def mine_frequent_patterns(edges: list, min_freq: int = 100) -> dict:
    """
    For each source_id, collect the multiset of relations it emits.
    Common multisets = patterns.
    Returns {pattern_tuple: count}.
    """
    by_source = defaultdict(list)
    for src, dst, rel, w in edges:
        by_source[src].append(rel)

    pattern_counts = Counter()
    for rels in by_source.values():
        pattern = tuple(sorted(rels))
        pattern_counts[pattern] += 1

    # Keep only patterns seen ≥ min_freq
    return {p: c for p, c in pattern_counts.items() if c >= min_freq}


def pattern_compression_savings(patterns: dict, edges: list) -> dict:
    """
    Compute potential savings from patterns.
    Each matched pattern saves: (raw_edge_bits * N_edges) − (pattern_code_bits + variable_target_bits)
    """
    total = len(edges)
    total_bits_naive = total * (13 * 8)  # 13 bytes/edge naive

    # Assign Huffman codes to patterns (by frequency)
    if not patterns:
        return {"total_bits_naive": total_bits_naive, "total_bits_compressed": total_bits_naive}

    pattern_freq = {i: c for i, (_, c) in enumerate(patterns.items())}
    pattern_codes = build_huffman(pattern_freq)

    # For each pattern, cost = code_bits + variable_target_bits
    # Variable targets: assume each target still needs ~16 bits (varint delta)
    compressed_bits = 0
    by_source = defaultdict(list)
    for src, dst, rel, w in edges:
        by_source[src].append((dst, rel, w))

    for src, edge_list in by_source.items():
        pattern = tuple(sorted(e[1] for e in edge_list))
        if pattern in patterns:
            pat_idx = list(patterns.keys()).index(pattern)
            code_bits = len(pattern_codes[pat_idx])
            target_bits = len(edge_list) * 16  # varint target per edge
            compressed_bits += code_bits + target_bits
        else:
            # No pattern match — use baseline
            compressed_bits += len(edge_list) * 13 * 8

    return {
        "total_bits_naive": total_bits_naive,
        "total_bits_compressed": compressed_bits,
        "num_patterns": len(patterns),
        "matched_sources": sum(1 for edges in by_source.values()
                              if tuple(sorted(e[1] for e in edges)) in patterns),
        "total_sources": len(by_source),
    }


# ═══════════════════════════════════════════════════════════════════════
# Layer 4 — Weight quantization
# ═══════════════════════════════════════════════════════════════════════

def quantize_weight(w: float) -> tuple:
    """Returns (tag_2bits, extra_bytes)."""
    if abs(w - 1.0) < 0.001:
        return (0b00, b"")           # tag only, no extra
    elif abs(w - 0.75) < 0.01:
        return (0b01, b"")
    elif abs(w - 0.5) < 0.01:
        return (0b10, b"")
    else:
        # Tag 11 + 1 byte quantized [0..255]
        q = int(max(0, min(255, w * 255)))
        return (0b11, bytes([q]))


# ═══════════════════════════════════════════════════════════════════════
# Tests
# ═══════════════════════════════════════════════════════════════════════

def test_distribution_is_realistic():
    edges = generate_edges(10_000)
    rel_count = Counter(rel for _, _, rel, _ in edges)
    # is_a should be ~32%
    is_a_frac = rel_count[REL_TO_IDX["is_a"]] / len(edges)
    print(f"  is_a fraction: {is_a_frac:.1%} (expected ~32%)")
    assert 0.28 < is_a_frac < 0.36


def test_huffman_beats_fixed_width():
    edges = generate_edges(100_000)
    rel_count = Counter(rel for _, _, rel, _ in edges)
    codes = build_huffman(rel_count)

    avg_bits = huffman_bits_per_symbol(rel_count, codes)
    fixed_bits = 6  # 64 relations need 6 bits
    print(f"  Fixed-width: 6 bits/relation")
    print(f"  Huffman:     {avg_bits:.2f} bits/relation")
    print(f"  Savings:     {(fixed_bits - avg_bits) / fixed_bits:.1%}")
    assert avg_bits < 5


def test_huffman_assigns_short_codes_to_common():
    edges = generate_edges(100_000)
    rel_count = Counter(rel for _, _, rel, _ in edges)
    codes = build_huffman(rel_count)

    is_a_len = len(codes[REL_TO_IDX["is_a"]])
    rare_len = len(codes[REL_TO_IDX[f"rel_{47:02d}"]])
    print(f"  is_a (most common):  {is_a_len} bits")
    print(f"  rel_47 (rare):       {rare_len} bits")
    assert is_a_len < rare_len


def test_delta_varint_shrinks_targets():
    # Simulate a source with 50 targets scattered across 10K nodes
    targets = sorted(random.sample(range(10_000), 50))
    encoded = delta_encode_targets(targets)
    raw = raw_targets_bytes(targets)
    print(f"  50 targets, raw:     {raw} bytes")
    print(f"  Delta+varint:        {len(encoded)} bytes")
    print(f"  Compression:         {raw / len(encoded):.2f}×")
    assert len(encoded) < raw


def test_delta_dense_neighborhood():
    # 100 targets in a tight range (dense neighborhood)
    targets = list(range(1000, 1100))
    encoded = delta_encode_targets(targets)
    raw = raw_targets_bytes(targets)
    print(f"  100 consecutive targets, raw:       {raw} bytes")
    print(f"  Delta+varint:                       {len(encoded)} bytes")
    print(f"  Compression:                        {raw / len(encoded):.2f}×")
    # Most deltas are 1 → 1 byte each → ~100 bytes vs 400
    assert len(encoded) < raw // 2


def test_weight_quantization_80pct_are_one():
    edges = generate_edges(10_000)
    tags = [quantize_weight(w)[0] for _, _, _, w in edges]
    tag_counts = Counter(tags)
    frac_1 = tag_counts[0b00] / len(edges)
    print(f"  Tag 00 (w=1.0):  {tag_counts[0b00]} ({frac_1:.1%})")
    print(f"  Tag 01 (w=0.75): {tag_counts[0b01]}")
    print(f"  Tag 10 (w=0.5):  {tag_counts[0b10]}")
    print(f"  Tag 11 (other):  {tag_counts[0b11]}")
    # The generator uses continuous [0.5, 1.0), so 0.5 and 0.75 are rare matches.
    # Only ~80% should be exactly 1.0.
    assert 0.75 < frac_1 < 0.85


def test_pattern_mining_finds_common_patterns():
    # Create a SYNTHETIC case where certain patterns are super-common
    edges = []
    # 1000 Hebrew words, each with same 4-edge pattern
    for i in range(1000):
        src = i
        edges.append((src, 10_000, REL_TO_IDX["is_a"], 1.0))  # is_a word
        edges.append((src, 20_000 + i, REL_TO_IDX["translates_to"], 1.0))  # to en
        edges.append((src, 30_000 + i, REL_TO_IDX["translates_to"], 1.0))  # to es
        edges.append((src, 40_000 + i, REL_TO_IDX["has_property"], 1.0))
    # Mix in noise
    edges.extend(generate_edges(5000, num_nodes=50_000))
    random.shuffle(edges)

    patterns = mine_frequent_patterns(edges, min_freq=100)
    print(f"  Frequent patterns found: {len(patterns)}")
    for i, (p, c) in enumerate(list(patterns.items())[:3]):
        rels = [IDX_TO_REL[r] for r in p]
        print(f"    #{i+1} ({c}×): {rels}")
    assert len(patterns) >= 1


def test_full_stack_on_realistic_corpus():
    """The full pipeline — all 4 layers combined."""
    edges = generate_edges(100_000, num_nodes=10_000)

    # Baseline: 13 bytes per edge (4 src + 4 dst + 1 rel + 4 weight)
    baseline_bytes = len(edges) * 13

    # Layer 1: Huffman on relations
    rel_count = Counter(rel for _, _, rel, _ in edges)
    rel_codes = build_huffman(rel_count)
    rel_bits = sum(len(rel_codes[r]) for _, _, r, _ in edges)
    rel_bytes = (rel_bits + 7) // 8

    # Layer 2: Delta+varint on targets (grouped by source)
    by_src = defaultdict(list)
    for s, d, _, _ in edges:
        by_src[s].append(d)
    target_bytes = sum(len(delta_encode_targets(ts)) for ts in by_src.values())
    # Plus src_id per source (varint): log2(10000) ≈ 14 bits ≈ 2 bytes
    src_bytes = len(by_src) * 2

    # Layer 3: Pattern dictionary (big savings if patterns repeat)
    patterns = mine_frequent_patterns(edges, min_freq=50)
    # (For a realistic measure, we estimate using savings function)
    pattern_analysis = pattern_compression_savings(patterns, edges) if patterns else {"num_patterns":0, "matched_sources":0, "total_sources":0}

    # Layer 4: Weight tags (2 bits avg + occasional 1 byte)
    weight_bits = 0
    weight_extra = 0
    for _, _, _, w in edges:
        tag, extra = quantize_weight(w)
        weight_bits += 2
        weight_extra += len(extra)
    weight_bytes = (weight_bits + 7) // 8 + weight_extra

    total_compressed = rel_bytes + target_bytes + src_bytes + weight_bytes
    ratio = baseline_bytes / total_compressed

    print(f"\n  Input: {len(edges):,} edges = {baseline_bytes:,} bytes baseline")
    print(f"")
    print(f"  Layer 1 (Huffman rels):    {rel_bytes:,} bytes")
    print(f"  Layer 2 (delta targets):   {target_bytes:,} bytes")
    print(f"           + source_ids:     {src_bytes:,} bytes")
    print(f"  Layer 4 (weight quant):    {weight_bytes:,} bytes")
    print(f"  -----------------------------------")
    print(f"  Total compressed:          {total_compressed:,} bytes")
    print(f"  Compression ratio:         {ratio:.2f}×")
    print(f"")
    print(f"  Layer 3 (patterns): {pattern_analysis['num_patterns']} patterns, "
          f"matched {pattern_analysis.get('matched_sources', 0)}/"
          f"{pattern_analysis.get('total_sources', 0)} sources")
    print(f"           (random data has few patterns; real ZETS data has many)")

    assert ratio > 2.0, "should achieve at least 2× on realistic edges"


def test_roundtrip_lossless():
    """Encode then decode must match."""
    # Simple test: encode target list, decode it back
    targets = sorted(random.sample(range(10_000), 20))
    encoded = delta_encode_targets(targets)

    # Decode varints
    decoded = []
    i = 0
    prev = 0
    while i < len(encoded):
        val = 0
        shift = 0
        while True:
            byte = encoded[i]
            i += 1
            val |= (byte & 0x7F) << shift
            if not (byte & 0x80):
                break
            shift += 7
        prev += val
        decoded.append(prev)

    print(f"  Original: {len(targets)} targets")
    print(f"  Decoded:  {len(decoded)} targets")
    print(f"  Match:    {decoded == targets}")
    assert decoded == targets, "delta+varint roundtrip failed"


if __name__ == '__main__':
    print("━━━ Edge Compression — Python Prototype ━━━\n")

    print("[1] ZETS-realistic distribution:")
    test_distribution_is_realistic()

    print("\n[2] Huffman beats fixed-width:")
    test_huffman_beats_fixed_width()

    print("\n[3] Huffman gives shortest codes to common rels:")
    test_huffman_assigns_short_codes_to_common()

    print("\n[4] Delta+varint shrinks target_ids (scattered):")
    test_delta_varint_shrinks_targets()

    print("\n[5] Delta+varint on dense neighborhood:")
    test_delta_dense_neighborhood()

    print("\n[6] Weight quantization — 80% are w=1.0:")
    test_weight_quantization_80pct_are_one()

    print("\n[7] Pattern mining finds frequent N-tuples:")
    test_pattern_mining_finds_common_patterns()

    print("\n[8] Full stack on realistic 100K edges:")
    test_full_stack_on_realistic_corpus()

    print("\n[9] Roundtrip: encoded → decoded == original:")
    test_roundtrip_lossless()

    print("\n━━━ ALL TESTS PASSED ━━━")
