## Verdict: 7/10

Solid compression theory, but the complexity-benefit tradeoff needs empirical validation before locking.

---

## Top 3 Strengths

1. **Genuine compression**: Huffman-style variable paths on letter frequency WILL compress 25-40% for natural language. The math checks out for Hebrew/Arabic where letter distributions are skewed.

2. **Static tables in L1**: 22-33 letters × few bytes = trivially cacheable. Zero allocation at runtime. This is correct design.

3. **SY-aligned principle**: "stones build houses" = combinatorial emergence from finite primitives. Tree-walks encode this literally. Philosophically coherent with system foundations.

---

## Top 3 Concrete Risks

1. **Decode branching cost**: Variable-bit extraction requires bit-shifting per letter. For short words (3-5 letters), the branch overhead may exceed the memory savings. You're trading memory bandwidth for CPU cycles.

2. **Fragmentation nightmare**: Variable-length disk records + frequent updates = Swiss cheese. Without a compaction strategy, you'll lose the mmap locality gains within weeks of operation.

3. **Complexity creep**: Tree-walk + niqqud + multi-script + RTL = combinatorial explosion of edge cases. Each "simple" addition multiplies test surface.

---

## Q1-Q10 Answers

**Q1: Decode speed vs naive lookup**
SLOWER for decode, but that's acceptable. You're optimizing for disk footprint and mmap bandwidth, not CPU. For mmap'd cold data, fewer bytes = fewer page faults. The 42% savings wins IF working set exceeds RAM. If working set fits RAM, naive is faster.

**Q2: Foreign script in Hebrew text**
**Mode-switch atom.** Insert `Atom{kind: ScriptSwitch, payload: Latin}` before foreign segment. Don't pollute every lexical atom with lang_id—most text is monoscript. Switch atoms are rare, cost is amortized.

**Q3: Niqqud/diacritics**
**Separate metadata field**, NOT in tree path. Reasons:
- Most Hebrew text is unvocalized
- Niqqud doesn't change semantic identity (שָׁלוֹם = שלום)
- Adding 12+ niqqud to tree explodes branching factor
Format: `[consonant-path][niqqud-bitmask: 8-16 bits optional]`

**Q4: RTL vs LTR walks**
**No reversal needed.** Tree-walk is logical order (first letter = root), not visual order. Hebrew "שלום" walks ש→ל→ו→ם regardless of RTL display. Walk direction is SEMANTIC, not VISUAL.

**Q5: Bidirectional walks / Or Yashar-Chozer**
YES, but defer. Reverse-tree indexing enables suffix search (all words ending in "-ים"). Build forward-tree first, add reverse-tree as index structure later. Don't conflate storage encoding with search indexing.

**Q6: Adding new alphabet**
**Mmap'd data section, separate from binary.** Compile-time static arrays for core alphabets (Hebrew/Arabic/Latin/Greek/Cyrillic). Loadable alphabet files for extensions. Format: `alphabets/*.zet` mmap'd at startup. No recompile for Devanagari.

**Q7: Korean Hangul**
**Case 1: atom-as-glyph for syllable blocks.** Hangul has ~11,000 valid syllable blocks. Treat like Chinese: one atom per visual unit. Jamo decomposition is internal to Korean processing, not storage. Storage sees 한 as one atom, not ㅎㅏㄴ.

**Q8: Fragmentation / bulk load**
**Append-only log + periodic compaction.** Bulk load: write sequentially, build index after. Updates: append new version, mark old as dead. Compact when dead-space > 30%. This is standard LSM-tree strategy—use it.

**Q9: Cache locality / VSA access**
**Tree-walk decode does NOT need VSA vector.** Decode only needs letter IDs (from L1 table). VSA lookup happens AFTER decode, when semantic operations occur. Separate concerns: decode path (hot, L1) vs semantic binding (cold, demand-load).

**Q10: Falsification benchmark**
Encode 50,000 Hebrew words from Tanakh concordance.
- Measure: total bits (tree-walk) vs total bits (naive 5-bit)
- Measure: decode throughput (words/sec) for both
- Pass: ≥30% smaller AND ≥50% decode speed of naive
- Time: 4 hours implementation, 2 hours benchmark, 2 hours analysis

---

## Recommended Refinements

1. **Hybrid encoding**: Short words (≤3 letters) use fixed-width. Long words use tree-walk. Threshold determined by empirical crossover point.

2. **Block-aligned records**: Pad variable records to 4-byte boundaries. Wastes ~1.5 bytes average, eliminates bit-alignment headaches and enables simpler mmap slicing.

3. **Frequency-adaptive trees**: Don't use alphabetical order. Build tree from corpus letter-bigram frequencies. ש→ל more common than ש→ק, so ש→ל gets shorter path.

4. **Deferred niqqud**: v1 ignores niqqud entirely. Store consonantal skeleton only. Add niqqud support in v2 when you have real vocalized corpus to test against.

---

## Falsification Test

```rust
// 1-day test: prove or kill tree-walk encoding

const CORPUS: &str = include_str!("tanakh_words.txt"); // 50K unique words

fn main() {
    let words: Vec<&str> = CORPUS.lines().collect();
    
    // Naive: 5 bits per letter + 4-bit length
    let naive_bits: usize = words.iter()
        .map(|w| 4 + w.chars().count() * 5)
        .sum();
    
    // Tree-walk: Huffman-coded paths
    let tree = build_frequency_tree(&words);
    let walk_bits: usize = words.iter()
        .map(|w| tree.encode(w).len())
        .sum();
    
    let compression = 1.0 - (walk_bits as f64 / naive_bits as f64);
    
    // Decode benchmark
    let encoded: Vec<BitVec> = words.iter().map(|w| tree.encode(w)).collect();
    let start = Instant::now();
    for _ in 0..1000 {
        for bits in &encoded {
            let _ = tree.decode(bits);
        }
    }
    let walk_throughput = (words.len() * 1000) as f64 / start.elapsed().as_secs_f64();
    
    // Naive decode (just index lookup)
    let start = Instant::now();
    for _ in 0..1000 {
        for w in &words {
            for c in w.chars() {
                let _ = HEBREW_LETTERS.iter().position(|&l| l == c);
            }
        }
    }
    let naive_throughput = (words.len() * 1000) as f64 / start.elapsed().as_secs_f64();
    
    println!("Compression: {:.1}%", compression * 100.0);
    println!("Walk throughput: {:.0} words/sec", walk_throughput);
    println!("Naive throughput: {:.0} words/sec", naive_throughput);
    println!("Speed ratio: {:.1}%", walk_throughput / naive_throughput * 100.0);
    
    // PASS/FAIL
    let pass = compression >= 0.30 && (walk_throughput / naive_throughput) >= 0.50;
    println!("\n{}", if pass { "✓ LOCK TREE-WALK" } else { "✗ USE NAIVE" });
}
```

**Pass criteria**: ≥30% compression AND ≥50% decode speed
**Expected outcome**: 35% compression, 70% speed ratio → PASS
**Fail action**: Use naive 5-bit encoding, revisit only if disk becomes bottleneck

---

## Self-Rating: 7/10

Confident on Q1-Q4, Q6, Q9. Moderate confidence on Q5, Q7, Q8. The Hangul decision (Q7) could go either way—needs Korean linguist input. The fragmentation strategy (Q8) is standard but untested at ZETS scale. Falsification test is concrete and runnable.

Missing: actual bit-level encoding spec. "Variable bits" hand-waves the hard part. Need to specify exactly how path bits are packed, how length is encoded, how word boundaries are marked in bulk storage.