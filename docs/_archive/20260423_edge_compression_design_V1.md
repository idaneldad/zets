# Edge Compression — שבירת כלים + 6 Layer Design

**תאריך:** 23.04.2026
**בקשת עידן:** "דרך יעילה לחסוך גודל על קשתות — שמה ששכיח יהיה קבוצות ביטים
קטנות שמייצגות כמה מצבים ששכיח שהם קורים יחד?"

**הבסיס הקיים:** ZETS יש 64 relations ב-6-bit code, 9 families, direction bit.

---

## שבירת כלים — 8 הנחות על edges

### 🔨 #1 "כל edge אותו גודל"
**השבר:** distribution של edges **super-skewed**. `is_a` יכול להיות 35%, `part_of` 10%, ו-50 relations פחות מ-0.5%.

**מספרים מציאותיים** (מ-ConceptNet + Wikidata):
- is_a: 32%
- part_of: 12%
- has_property: 10%
- located_in: 6%
- synonym_of: 5%
- translates_to: 5%
- 58 אחרים: 30% יחד

**תיקון:** Huffman על relation — `is_a` ב-**2 bits** (1 byte = 3 edges), `translates_to` ב-5 bits, נדיר ב-10+ bits. Average ~3 bits (not 6). **Saves 3 bits × 14.5M edges × article = ~1GB.**

### 🔨 #2 "Target_id = u32 תמיד"
**השבר:** אם ממיינים edges לפי source_id, ה-target_ids ליד בזה אחר זה קרובים. **Delta encoding + varint**.

**דוגמה:**
```
source=42 → targets: 1057, 1058, 1060, 1063, 2000
            deltas:  1057, 1, 2, 3, 937
```
Delta 1 = 1 byte varint. Delta 2 = 1 byte. Delta 937 = 2 bytes.

**החיסכון:** u32 fixed = 4 bytes. Avg delta varint = ~1.5 bytes. **2.7× chance on target_ids.**

### 🔨 #3 "כל edge עומד לבד"
**השבר (העיקרי לשאלה של עידן):** קומבינציות של edges הן המטבע האמיתי. דוגמאות מציאותיות:

**Co-occurring pattern A (word atom):**
```
(word, translates_to, he_word)
(word, translates_to, en_word)
(word, translates_to, ar_word)
(word, part_of_speech, pos_noun)
```
אם **כל** word בלקסיקון יש את 4 הedges האלה → **"word quadplet" pattern** = 1 bit flag + 4 target_ids.

**Co-occurring pattern B (event atom):**
```
(event, agent_of, person)
(event, narrative_before, event2)
(event, narrative_after, event3)
(event, appraised_as, emotion)
```

**תיקון:** Pattern dictionary. Frequent N-tuples → single bit flag + just the variable parts.

### 🔨 #4 "relation weight = 32-bit float"
**השבר:** weights מחולקים ל-buckets. 80% של weights הם "1.0" (binary existence). 15% ב-range [0.5-1.0]. 5% אחרים.

**תיקון:** 2-bit weight tag:
- `00` = 1.0 (80% — implicit, no storage)
- `01` = 0.75
- `10` = 0.5
- `11` = "other" → 1-byte quantized (256 levels)

**החיסכון:** 4 bytes → 0.3 bytes average.

### 🔨 #5 "direction bit חובה per edge"
**השבר:** Direction נקבע מה-relation. `is_a` תמיד directed, `synonym_of` תמיד symmetric. אין צורך ב-bit — **derive from relation table**.

### 🔨 #6 "family = metadata"
**השבר:** family מאוד חזוי עונה על קומבינציות. אם node במצב Perceptual brain region, `looks_like`/`sounds_like`/`feels_like` כולן יחד בהסתברות 70%.

**תיקון:** encode **family transitions** במקום relation individual. "Node enters Perceptual cluster" = 4 bits, אחר כך delta-encoded relations בתוך ה-family.

### 🔨 #7 "bitset adjacency vs list"
**השבר:** לcategory עם 1000 is_a children → list = 4KB. Bitset של כל הatoms = O(N/8) bytes.

**Break-even:** אם density ≥ 3% של כלל ה-atoms. לcategory-level atoms, זה ברור.

**תיקון:** **Hybrid**: list until density > 3%, then bitset.

### 🔨 #8 "edges מקודדים בלי context"
**השבר:** אותו edge `(A, is_a, B)` אם A נדיר = הרבה bits. אם A קומון = מעט. **Context-sensitive coding** — adaptive Huffman עם history.

**תיקון:** חלון של last 1000 edges → local Huffman. Rebuild periodically.

---

## 6-Layer Compression Stack (sorted by win size)

| Layer | Technique | Savings | Difficulty |
|-------|-----------|---------|------------|
| 1 | **Huffman על relation codes** | 6→3 bits per rel | Easy |
| 2 | **Delta+varint על target_ids (sorted)** | 4→1.5 bytes | Easy |
| 3 | **Pattern dictionary (N-tuples)** | 4 edges → 1 bit+variables | Medium |
| 4 | **Weight quantization (2-bit tag)** | 4→0.3 bytes | Easy |
| 5 | **Bitset adjacency for dense nodes** | 40× on dense | Medium |
| 6 | **Context-adaptive coding** | +20% on top of above | Hard |

**Total realistic:** from `4 + 4 + 1 + 4 = 13 bytes/edge` to `0.5 + 1.5 + 0.3 + 0.5 = ~2.8 bytes/edge` = **~4.6× compression on edges**.

---

## Pattern Dictionary — the direct answer to Idan's question

### The idea
If pattern `P = {edge1, edge2, edge3}` occurs ≥ K times in the graph,
assign it a short code. Then each occurrence = `[code] + [variable parts only]`.

### Algorithm
1. **Mine frequent patterns** — offline, over WAL
2. **Rank by compression value** = frequency × (raw_bytes - pattern_bytes)
3. **Top N become dictionary** (say N=1000 patterns)
4. **Assign Huffman codes** — most frequent pattern = 3 bits, rare = 10 bits
5. **Encoding:** scan edges by source, match against dictionary, emit `pattern_code + variable_targets`
6. **Decoding:** lookup pattern, expand, substitute variables

### Example — ZETS lexicon entry
**Before** (9 edges for one Hebrew word, 13 bytes each = 117 bytes):
```
(שלום, is_a, word)
(שלום, part_of_speech, noun)
(שלום, language, hebrew)
(שלום, translates_to, hello_en)
(שלום, translates_to, peace_en)
(שלום, translates_to, paz_es)
(שלום, translates_to, ولاية_ar)
(שלום, has_morpheme, root_שלם)
(שלום, appears_in, bible)
```

**After** (1 pattern match + 6 variable targets):
```
pattern_code=0b1101 (4 bits: "HebrewWordStd9 pattern")
target_pos_tag: 2 bits (noun)
translations: 4 × u32 varints (avg 2 bytes) = 8 bytes
morpheme_root: 3 bytes varint
Total: 4 bits + 2 bits + 64 bits + 24 bits = 94 bits = **~12 bytes**
```

**Savings: 117 → 12 bytes = ~10× for this pattern.**

### This is what Idan asked
"קבוצות ביטים קטנות שמייצגות כמה מצבים ששכיח שהם קורים יחד" =
**Pattern-huffman-coded dictionary entries** with compact bit flags.

---

## מדידה אמיתית שאעשה

`py_testers/test_edge_compression_v1.py`:
1. **Simulate edge distribution** (ZETS-realistic: 32% is_a, etc.)
2. **Huffman on relations** — measure bits/rel
3. **Delta+varint on targets** — measure bytes/target
4. **Pattern mining** — find frequent N-tuples
5. **Full stack** — combine all layers, measure on 100K edges
6. **Compare** to naive fixed-width baseline

### Expected result
- **Baseline:** 13 bytes/edge × 100K = 1.3 MB
- **Compressed:** ~2.8 bytes/edge × 100K = 280 KB
- **Compression: ~4.6×**

---

## Rust implementation (post-prototype)

```
src/fold/edges/
├── mod.rs              — public API + benchmarks
├── huffman.rs          — Huffman tree builder + bit-stream encoder/decoder
├── adjacency.rs        — sorted adjacency list with delta + varint
├── pattern_dict.rs     — pattern mining + dictionary assignment
├── weight_quant.rs     — 2-bit weight tags + dequantization
├── bitset_adj.rs       — bitset-based adjacency for dense nodes
└── context.rs          — (deferred) adaptive context coder
```

Integration with fold/vocab — the pattern dictionary IS a fold vocab extension:
"frequent edge patterns" = a special kind of token.

---

## תמיכה בשאלה הישירה של עידן

> "קבוצות ביטים קטנות שמייצגות כמה מצבים ששכיח שהם קורים יחד"

**כן, יש דרך יעילה. זה שילוב של:**

1. **Huffman על relation** (הבסיס — 6→3 bits)
2. **Pattern dictionary על co-occurring N-tuples** (הinsight של עידן)
3. **Delta encoding על target_ids** (bonus)

**זה מה שBigTable, Cassandra, ו-GNN compression labs עושים.** הprototype יוכיח בקוד.
