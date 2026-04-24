"""
ניתוח: איך לחבר atoms בצורה רזה — pointer? קשת? תכונות?
"""

# 4 layout options
class EdgeA: SIZE = 25  # pointer-based (linked list per atom)
class EdgeB: SIZE = 7   # packed adjacency
class EdgeC: SIZE = 6   # CSR (compressed sparse row)
class EdgeD_HOT: SIZE = 6
EdgeD_COLD_AVG = 0.8

N_ATOMS = 10_000_000
N_EDGES = 100_000_000
CACHE_LINE = 64

print("═" * 78)
print("  Storage layout comparison — 10M atoms, 100M edges")
print("═" * 78)

opts = [
    ("A. Pointer linked-list",      EdgeA.SIZE,  N_EDGES * EdgeA.SIZE),
    ("B. Packed adjacency",         EdgeB.SIZE,  N_EDGES * EdgeB.SIZE + N_ATOMS * 4),
    ("C. CSR (industry std)",       EdgeC.SIZE,  N_EDGES * EdgeC.SIZE + N_ATOMS * 4),
    ("D. Hybrid hot/cold",          EdgeD_HOT.SIZE + EdgeD_COLD_AVG, 
                                    int(N_EDGES * (EdgeD_HOT.SIZE + EdgeD_COLD_AVG)) + N_ATOMS * 8),
]

print(f"\n  {'Option':<35} {'B/edge':<10} {'Total RAM':<12}")
print(f"  {'─'*35} {'─'*10} {'─'*12}")
for name, sz, total in opts:
    print(f"  {name:<35} {sz:<10.1f} {total/1e9:.2f} GB")

print(f"""
  Cache analysis ({CACHE_LINE} byte lines):
    A: {CACHE_LINE // 25} edges/line   slow, fragmented
    B: {CACHE_LINE // 7} edges/line   ×3.5 faster than A
    C: {CACHE_LINE // 6} edges/line  ×4.0 faster than A
    D: {CACHE_LINE // 7} edges/line   ×3.5 faster (hot path)

═══════════════════════════════════════════════════════════════════════════════
  Bitwise packing — edge metadata in u16 (2 bytes!)
═══════════════════════════════════════════════════════════════════════════════

  edge_meta layout (16 bits):
  ┌─────────┬──────────┬──────────┬──────────┐
  │ type 5b │ state 4b │ mem 4b   │ flags 3b │
  └─────────┴──────────┴──────────┴──────────┘
  
  - type   (5 bits): 32 edge types — fits 21 we need
  - state  (4 bits): 16 levels  -8..+7 (state_value bucketed)
  - memory (4 bits): 16 levels (Ebbinghaus is exponential)
  - flags  (3 bits): has_context, has_state_dep, deleted

  Rust implementation:
  
    fn pack(t: u8, s: i8, m: u8, f: u8) -> u16 {{
        ((t as u16 & 0x1F) << 11)
      | (((s + 8) as u16 & 0x0F) << 7)
      | ((m as u16 & 0x0F) << 3)
      | (f as u16 & 0x07)
    }}

═══════════════════════════════════════════════════════════════════════════════
  השאלה שלך: pointer? קשת? תכונות?
═══════════════════════════════════════════════════════════════════════════════

  התשובה: INDEX-BASED, לא pointers.
  
  ┌──────────────────────────────────────────────────────────────────────┐
  │  atoms: Vec<AtomHot>                  // O(1) by id                  │
  │  edges_hot: Vec<EdgeHot>              // 6 bytes each                │
  │  fwd_offsets: Vec<u32>                // CSR offsets per atom        │
  │  rev_offsets: Vec<u32>                // for reverse traversal       │
  │                                                                      │
  │  הקישור: fwd_offsets[atom_id] = (start, end)                          │
  │  edges = edges_hot[start..end]                                       │
  └──────────────────────────────────────────────────────────────────────┘
  
  למה INDICES ולא POINTERS?
  
  1. Cache-friendly  — predictable, prefetchable
  2. mmap-ready      — no virtual addresses, save direct to disk
  3. 50% smaller     — u32 (4 bytes) vs pointer (8 bytes)
  4. Safer in Rust   — no unsafe, no Box/Rc overhead
  5. Zero-copy load  — mmap a file, ready instantly

═══════════════════════════════════════════════════════════════════════════════
  המלצה סופית: Layout C+D היברידי
═══════════════════════════════════════════════════════════════════════════════
""")

print("""  Memory layout (10M atoms × 100M edges):
  
  // HOT PATH (always in RAM, mmap'd):
  atoms:        16B × 10M  = 160 MB
  edges_hot:     6B × 100M = 600 MB
  fwd_offsets:   4B × 10M  =  40 MB
  rev_offsets:   4B × 10M  =  40 MB
                          ─────────
  HOT TOTAL:               840 MB
  
  // COLD PATH (lookup tables, ~10% of edges):
  contexts:                ~80 MB
  state_deps:              ~40 MB
  features:                ~50 MB
  lemma_strings:           ~30 MB
                          ─────────
  COLD TOTAL:              200 MB
                          ─────────
  GRAND TOTAL:           1.04 GB

  Latency budget:
    Atom lookup:        < 10 ns   (array index, L1 cache)
    Edges of atom:      < 100 ns  (sequential, prefetched)
    21 parallel dives:  < 50 μs   (good cache behavior)
""")

