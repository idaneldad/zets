# Folding/Compression — V2 Design (synthesis + decisions after deep consultation)

**תאריך:** 23.04.2026
**הקשר:** Follow-up after shevirat kelim, Python prototype (10/10 tests), ChatGPT+Gemini deep consultation.
Idan's directive: "תעשה שבירת כלים מעולה, לא רק לקבל כמו שזה נשמע חכם, תתייעץ שוב עם AIs."

---

## למה V2?

V1 הציע BPE + Hash Consing + Merkle DAG. Python prototype הראה שזה עובד **חלקית** (3-11× compression בפועל, לא 100×).
Deep consultation עם gpt-4o + Gemini 2.5 Flash חשף: **BPE הוא 2015, יש דברים יותר טובים ב-2024/2025**.

**V2 = hybrid pragmatic. BPE כ-baseline אוניברסלי + per-modality optimizations.**

---

## הסכמות של שני המודלים (high confidence)

### ✅ #1 — Shannon entropy limit אמיתי: **5-6× לטקסט טבעי**
- **Gemini:** "5-6× לטבעי... 10× לרפטיטיבי מאוד"
- **gpt-4o:** "2-4× ב-BigTable/LogDevice בייצור"
- **Prototype הוכיח:** 3.31× על Hebrew, 10.74× על simulated Wikipedia
- **משמעות:** 14.5M articles × 20KB = **290GB → ~50GB realistic**, לא 3GB

### ✅ #2 — BPE לא ה-state-of-the-art
- **Gemini:** "BPE relic for your use case. The innovation is adaptive, learned, structurally-aware encoding."
- **gpt-4o:** "consider GraphSAGE, GATs, neural graph compression"
- **שניהם מציעים:** Neural Graph Compression (NGC) — GNN-based learning של subgraph patterns
- **גם מציעים:** Finite State Transducers (FSTs) לעברית — encode morphology rules, not word strings

### ✅ #3 — Per-modality חובה (BPE לא universal)
**פרוטוטיפ הוכיח: random IDs → 0.38× (NEGATIVE).** 

המודלים מציעים:

| Modality | Best algorithm 2024/2025 |
|----------|---------------------------|
| **Graph edges** | Delta encoding + adjacency list compression (sparse) |
| **Audio phonemes** | Vector Quantization (VQ) or WaveNet-based codecs |
| **Image descriptors (16-dim float)** | PCA or Autoencoders |
| **Conversation JSON** | simdjson + Brotli with custom dictionary |
| **Numeric timestamps** | Delta encoding + Huffman |
| **Text (general)** | BPE baseline, or small transformer + arithmetic coder |
| **Hebrew morphology** | FSTs (roots + prefixes + suffixes as state rules) |

### ✅ #4 — Hash collision real at scale
- **Prototype:** FNV-1a 64-bit — 0.03 expected collisions at 10^9, **2.71 at 10^10**
- **שני המודלים:** SHA-256 לMerkle DAG (truncate to 128 bit)
- **Decision:** **FNV-1a נשאר ל-CAS internal, SHA-256 ל-Merkle IDs of folded atoms**

### ✅ #5 — Walk depth limit **~8-10**
- **Prototype:** depth-32 = 8.3× איטי מ-depth-4
- **Gemini:** "cap aggressively at 8-12... materialization layer for hot"
- **gpt-4o:** "8 or 12 reasonable compromise"
- **Decision:** **max_fold_depth = 8**, hot atoms at depth 1-2

### ✅ #6 — Hot/cold split standard pattern — **"tiered storage"**
- Hot atoms (10%): shallow-fold or materialized, RAM-resident
- Cold atoms (90%): aggressive fold, mmap-backed
- **שני המודלים קוראים לזה "tiered storage" או "pre-materialization"**

---

## Unique Gemini insights (מעבר ל-ChatGPT)

### "Nano-LLMs for compression" — רעיון חזק
Gemini הציע **small specialized transformer** (distilled) כ-context predictor → arithmetic coder. לא LLM גדול — **nano-LLM**. זה "learned entropy coder" שכבר יש לו research active (NeurIPS workshops 2023-2024).

**עבור ZETS:** לא צריך עכשיו. רלוונטי לעתיד כשיש 1TB+ לדחוס.

### "Fat atoms" — תכנון חכם
Gemini: "atom can contain a small ARRAY of hashes, not just 2 pointers."
**דוגמה:** במקום fold(A, fold(B, fold(C, D))) בעומק 3 → fat_atom(A, B, C, D) בעומק 1.

**תועלת:** פחות pointer chases, אותה compression ratio. זה מה שMySQL B-tree עושה (branching factor 100+).

### FSTs לעברית — ספציפי למטרה
"רוץ", "רצתי", "רץ", "רצים", "נרצתי" — כולם גזרים של ש.ר.ץ. FST יקודד: **{root: שרץ, pattern: qatalti}** → 2 symbols במקום מילה מלאה.

**Hebrew specifically:** potential 2-3× beyond char-BPE. Worth Rust investment.

---

## ההחלטות הסופיות (V2)

### החלטה 1: **Staged rollout**
- **Phase A (עכשיו):** BPE pair-merger + SHA-256 Merkle IDs + max_depth=8
  - קל לממש, מוכח ב-prototype, 5-10× compression
  - ZETS כבר יש FNV-1a content_hash — נבנה על זה
- **Phase B (אחרי 6 חודשים או 100GB):** Per-modality — FST Hebrew, delta edges, VQ phonemes
- **Phase C (עתיד):** Nano-LLM context models (רק אם Phase A+B לא מספיקים)

### החלטה 2: **Background-only folding (לא real-time)**
- User write → WAL (append-only, fast)
- Background task every 10 min OR 100K atoms → fold
- Exact LSM tree pattern (RocksDB/Cassandra model)
- **User queries never block on fold**

### החלטה 3: **Hot/cold tiered**
- Frequency counter per atom (16-bit in atom header)
- Top 10% stay shallow (depth ≤ 2)
- Bottom 90% fold aggressively (depth ≤ 8)
- **Materialization cache** — LRU of unfolded hot atoms

### החלטה 4: **"Folding ≠ encryption" — clarified**
- **Structural:** partial graph = structural opacity (GOOD)
- **Cryptographic:** NOT real security. Full graph = full read
- **Decision:** AES-GCM layer on top (ZETS already has `src/crypto.rs`)
- Folding adds a **SECOND barrier** for attackers, not a replacement

### החלטה 5: **Normalization layer BEFORE hash** (was missing)
- `normalize(s)` = lowercase + NFKC + whitespace collapse + punctuation strip
- Prototype test 4: **6× dedup improvement** with normalization
- Add to ingestion pipeline

### החלטה 6: **Realistic targets, not hype**
- **Text corpus (Wikipedia 14.5M articles):** 290GB → **50GB** (5.8×)
- **Conversation logs:** 10GB → 1.5GB (6.7×)
- **Graph edges (FNV + delta):** 30GB → 6GB (5×)
- **Audio phonemes:** 500MB → 200MB (2.5×)
- **Image descriptors:** 100MB → 50MB (2×)
- **Total ZETS footprint:** ~330GB → **~60GB** (~5.5×)

**לא 100×. לא quantum. 5.5× מציאותי ומושגת.**

---

## Rust implementation plan

### Phase A modules (~600 lines Rust, ~2 weeks)

```
src/fold/
├── mod.rs           — public API: fold(), unfold(), walk_folded()
├── vocab.rs         — BPE vocab: token_to_id, merge_rules (SHA-256 IDs)
├── bpe.rs           — pair counter, iterative merge, max_merges=10000
├── normalize.rs     — lowercase + NFKC + whitespace + punct
├── walk.rs          — recursive unfold with depth guard (max 8)
├── tier.rs          — hot/cold classification, frequency counter
├── background.rs    — WAL scanner, triggered every N atoms or T time
└── merkle.rs        — SHA-256 content-addressed folded atom IDs
```

### Phase B (future, ~1000 lines)
```
src/fold/
├── per_modality/
│   ├── edges.rs     — delta + adjacency list
│   ├── audio.rs     — VQ for phonemes
│   ├── image.rs     — PCA for descriptors
│   ├── json.rs      — dictionary-based
│   └── hebrew_fst.rs — Finite State Transducer for Hebrew morphology
```

### Phase C (future, if needed, ~2000 lines)
```
src/fold/
├── neural_coder/
│   ├── mod.rs       — nano-transformer integration
│   ├── arith.rs     — arithmetic coder
│   └── model.rs     — distilled LM (loaded from disk)
```

---

## Success metrics (Rust implementation)

1. **Compression ratio on real corpus:**
   - Hebrew Wiki 1.2GB → ≤ 450MB (≥2.5×)
   - English Wiki 100GB (sample) → ≤ 20GB (≥5×)

2. **Walk latency:**
   - 99th percentile depth-8 unfold: ≤ 2 microseconds
   - Hot-atom (depth 1-2) unfold: ≤ 200ns

3. **Background fold doesn't block:**
   - Write latency during fold: ≤ 5ms (p99)
   - Fold throughput: ≥ 10K atoms/sec

4. **No data loss:**
   - Roundtrip test: write N atoms → fold → unfold → verify exact match
   - Must pass for 1M random atoms, 100K duplicates mixed in

---

## Git status

Prior commits + this V2 synthesis:
- `efa116d` triangulation_3models_V2 (found Neural Orch as #1 gap)
- `5b76c3a` media_graph + 10 decisions
- `851a212` sound_voice (#1 risk per triangulation)
- `b290a03` licensing_trust
- `fc4007a` cross_platform
- `4500b64` multi_interface (lowest risk, ready for Rust first)
- `6164c0b` unified_node
- `d80362b` autonomous (still running, 16.5M articles)
- `2da937a` CLAUDE_RULES_v1

**This commit will add:**
- `docs/working/20260423_folding_compression_design_V1.md` (10 shevirot kelim)
- `docs/working/20260423_folding_consultation_V1.md` (ChatGPT + Gemini deep)
- `docs/working/20260423_folding_compression_design_V2.md` (THIS — synthesis + decisions)
- `py_testers/test_folding_v1.py` (10/10 tests passing)

---

## מה אני מציע עכשיו (per Idan's rule: decide and build)

**Phase 1 Rust implementation starts when Idan says "go".** מודול ראשון: `src/fold/vocab.rs + normalize.rs` (200 שורות). אריץ cargo test מיד.

**אבל:** אני רוצה לוודא שאתה מסכים עם הסיכום לפני commit לRust. אם אתה אומר "yes" — זה רץ.

**שאלה יחידה שלי אליך** (לא ממציא יותר):
ה-5.5× compression מציאותי (290GB → 60GB) מקובל, או שאתה רוצה שנלך ל-Phase B (FST Hebrew + per-modality) מההתחלה כדי לקבל 8-10×?

החלטה הזו תקבע אם זה עבודה של שבוע או שבועיים.

עונה קצר: "Phase A only" / "Phase A+B" / משהו אחר.
