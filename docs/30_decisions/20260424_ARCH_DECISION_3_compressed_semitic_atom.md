# ADR-3: Compressed Semitic-Based Atom Layout

**Status:** ACCEPTED (Idan, 2026-04-24 — evening session)
**Type:** Architecture Decision Record — binding
**Empirical basis:** `docs/20_research/20260424_hebrew_root_compression_poc.md`
**Builds on:**
  - ADR-1: Atom as Sigil, Executor as Doer
  - ADR-2: Linguistic Representation (Word/Sense/Concept layers)

---

## The Decision (in one sentence)

> **An 8-byte atom uses a Semitic root + binyan + features layout as its default
> encoding, with two variant layouts for foreign words and logographic scripts.
> Hebrew/Arabic/Aramaic share a unified root pool. The root is the primary
> atom identity; inflection lives in feature bits; other languages anchor via
> pointer or direct codepoint.**

---

## Why This Decision Now

Idan's intuition (the claim tested): "אפשר לקפל הכל לפי בסיס 37 שלמדנו".

I ran an empirical POC to test this. Results:

- **80.8% of Hebrew unique words** fit the 3-letter root model (with weak variants)
- **78.1% of Hebrew tokens** covered
- **656 roots shared between Hebrew and Arabic** (33% of Hebrew roots, 41% of Arabic)
- **50-57% of tokens** in both languages covered by shared Semitic roots
- **All fields fit in 64 bits** with room for growth

The POC validated Idan's hypothesis empirically. This ADR makes it binding
architecture.

---

## The 8-Byte Atom — Three Variants

All variants occupy **exactly 64 bits**. The top 4 bits (`kind`) determine layout.

### Variant A: `HebrewWord` — Semitic root-based (default)

```
 63      60  59      56  55          44  43        40  39   36  35   31
┌──────────┬──────────┬─────────────┬──────────┬──────┬──────┐
│  kind    │  flags   │  root_id    │  binyan  │tense │ pgn  │
│   4      │    4     │    12       │    3     │  3   │  4   │
└──────────┴──────────┴─────────────┴──────────┴──────┴──────┘
 30  29     28                                              0
┌────┬────┬────────────────────────────────────────────────┐
│def │foreig│             semantic_id                       │
│ 1  │ n 1  │                  24                           │
└────┴──────┴────────────────────────────────────────────────┘

Reserved bits: 4 (between flags and root_id for alignment)

kind = 0x0 for Hebrew, 0x1 for Arabic, 0x2 for Aramaic (all share root_id pool)
```

**Semantics:**
- `root_id` (12 bits = 4096 slots) indexes the shared Semitic root pool
- `binyan` (3 bits = 8 slots): Pa'al, Nif'al, Pi'el, Pu'al, Hif'il, Huf'al, Hitpa'el, Nominal
- `tense` (3 bits = 8 slots): Past, Present, Future, Imperative, Infinitive, Participle-active, Participle-passive, Gerund
- `pgn` (4 bits = 16 slots): Person × Gender × Number combinations
- `def` (1 bit): definite article present
- `foreign` (1 bit): set to 0 in this variant (root is native)
- `semantic_id` (24 bits = 16M): homograph discriminator + variant id

**Storage efficiency:**
- All inflections of one root share root_id → 1 root atom covers ~50 inflected forms
- "כתבתי", "כותבת", "נכתב", "הכתיב", "מכתב", "כתובה" → same root_id, different feature bits

### Variant B: `ForeignWord` — for non-Semitic languages + loanwords

```
 63      60  59      56  55            48  47                      24
┌──────────┬──────────┬──────────────────┬──────────────────────────┐
│  kind    │  flags   │   language_id    │     string_ref           │
│   4      │    4     │       8          │         24               │
└──────────┴──────────┴──────────────────┴──────────────────────────┘
 23                                                              0
┌────────────────────────────────────────────────────────────────┐
│                     semantic_id                                 │
│                         24                                      │
└────────────────────────────────────────────────────────────────┘

kind = 0x3 for ForeignWord
```

**Semantics:**
- `language_id` (8 bits = 256 languages): en, es, de, fr, ru, ja, etc.
- `string_ref` (24 bits = 16M slots): offset into per-language string pool
  - Small strings (≤8 bytes UTF-8) stored inline in companion structure
  - Longer strings via BlobStore
- `semantic_id` (24 bits): concept/sense identifier

**Covers:**
- English/German/Romance/Slavic words
- Hebrew loanwords that can't be fit into root model ("פייסבוק", "אלגוריתם")
- Proper names (English names, foreign place names)

### Variant C: `Logographic` — for CJK scripts

```
 63      60  59      56  55                         32
┌──────────┬──────────┬───────────────────────────────┐
│  kind    │  flags   │     codepoint (Unicode)       │
│   4      │    4     │            24                 │
└──────────┴──────────┴───────────────────────────────┘
 31                                                  0
┌────────────────────────────────────────────────────┐
│                 semantic_id                         │
│                     32                              │
└────────────────────────────────────────────────────┘

kind = 0x4 for Logographic
```

**Semantics:**
- `codepoint`: direct Unicode codepoint (covers all CJK Unified + extensions)
- No morphology — each character is an atom
- `semantic_id` (32 bits): used for disambiguation across reading+meaning combinations

**Covers:**
- Chinese characters (Simplified + Traditional)
- Japanese kanji
- Korean hanja (historical/scholarly)
- Does NOT cover Japanese hiragana/katakana (those go in ForeignWord variant)

### Reserved Variants (for future)

- `kind = 0x5`: Concept atom (pure language-agnostic concept, no surface form)
- `kind = 0x6`: Phrase-lemma atom (idioms, compounds)
- `kind = 0x7`: Procedure atom
- `kind = 0x8`: Action atom (for Executor invocation)
- `kind = 0x9..0xF`: reserved

---

## The Shared Semitic Root Pool

One pool serves Hebrew, Arabic, Aramaic, and other Semitic languages:

```
SemiticRootPool {
  max_roots: 4096  (12 bits addressing)
  measured_usage: 2,931 roots across HE + AR (18% headroom)
  
  each entry stores:
    - 3 consonants (18 bits raw, 15 bits packed)
    - flag: 3-letter / 4-letter / weak (2 bits)
    - usage_count per language (for heat/cold classification)
    - discovered_in (source corpus tracking)
}
```

**Why 12 bits (not smaller or larger):**
- 2^12 = 4096 slots
- Measured: 2,931 distinct roots in unified pool across 6M+ tokens
- Headroom: 38% for future corpora (Aramaic, Amharic, biblical forms, etc.)
- Not 11 bits (2,048) — too tight
- Not 13 bits (8,192) — wastes a bit

---

## Root Assignment — First-Come Canonical

Roots get IDs on first observation, in order of discovery:

```
Observation 1: root "כ.ת.ב" → ID = 0
Observation 2: root "ס.פ.ר" → ID = 1
Observation 3: root "ק.ר.א" → ID = 2
...
```

**Why:**
- Hot roots get low IDs (fits in VarInt 1 byte when atom_id referenced in edges)
- Deterministic given fixed corpus ingestion order
- No hash-based assignment (avoids collision handling complexity)

**Identity across runs:**
- Root ID table is persisted
- New roots appended, never renumbered
- Multi-instance ZETS federation uses same canonical pool (shared via sync)

---

## Binyan Encoding (3 bits = 7 Hebrew + 1 nominal)

| ID | Hebrew Binyan | Arabic Wazn | Pattern |
|---|---|---|---|
| 0 | Pa'al (קל) | Fa'ala (فعل) | Basic active |
| 1 | Nif'al (נפעל) | Fu'ila (فُعِل) | Passive/reflexive |
| 2 | Pi'el (פיעל) | Fa'ala II (فعّل) | Intensive active |
| 3 | Pu'al (פועל) | Fu'ila II (فُعِّل) | Intensive passive |
| 4 | Hif'il (הפעיל) | Af'ala (أفعل) | Causative active |
| 5 | Huf'al (הופעל) | Uf'ila (أُفعل) | Causative passive |
| 6 | Hitpa'el (התפעל) | Tafa'ala (تفعّل) | Reflexive |
| 7 | Nominal (שם) | Noun/Adj/Adv | Not a verb; derived noun/adjective |

Binyan 7 ("Nominal") covers nouns, adjectives, adverbs derived from the root.
This way **nouns and verbs share the same atom structure** — "ספר" (book, noun)
and "ספר" (counted, verb) use same root_id with different binyan.

---

## Tense/Aspect Encoding (3 bits = 8 forms)

| ID | Form | Hebrew Example | Usage |
|---|---|---|---|
| 0 | Past | כתבתי | "I wrote" |
| 1 | Present | כותב | "writing" |
| 2 | Future | אכתוב | "I will write" |
| 3 | Imperative | כתוב! | "write!" |
| 4 | Infinitive | לכתוב | "to write" |
| 5 | Participle-active | כותב | "writer" (agent noun) |
| 6 | Participle-passive | כתוב | "written" |
| 7 | Gerund/verbal-noun | כתיבה | "writing" (the act) |

For the Nominal binyan (7), tense is repurposed as noun-type category
(concrete noun / abstract noun / adjective / adverb).

---

## PGN — Person+Gender+Number (4 bits = 16 slots)

| ID | Person | Gender | Number | Example in past tense |
|---|---|---|---|---|
| 0 | 1 | Common | Singular | כתבתי |
| 1 | 2 | Masculine | Singular | כתבת |
| 2 | 2 | Feminine | Singular | כתבת (same surface, binyan-dependent) |
| 3 | 3 | Masculine | Singular | כתב |
| 4 | 3 | Feminine | Singular | כתבה |
| 5 | 1 | Common | Plural | כתבנו |
| 6 | 2 | Masculine | Plural | כתבתם |
| 7 | 2 | Feminine | Plural | כתבתן |
| 8 | 3 | Masculine | Plural | כתבו |
| 9 | 3 | Feminine | Plural | כתבו |
| 10-15 | Reserved for dual, archaic, dialectal forms | | | |

---

## Homograph Handling via `semantic_id`

Same root + same binyan can have different meanings:

```
root: ש.ל.מ (ID=47), binyan: 0 (Pa'al)
  semantic_id: 0 → "שלום" (peace, greeting)
  semantic_id: 1 → "שלם" (whole, complete)  
  semantic_id: 2 → "שילם" (paid)

root: ע.ב.ד (ID=125), binyan: 0 (Pa'al)
  semantic_id: 0 → "עבד" (served, worked)
  semantic_id: 1 → "עבד" (slave, noun via binyan-7 variant)
```

24 bits = 16,777,216 variants per kind. **Never runs out.**

---

## Integration With Earlier ADRs

### With ADR-1 (Atom as Sigil)

This ADR specifies the **layout of the 8-byte atom core** introduced in ADR-1.
All principles from ADR-1 remain:
- Atoms are sigils, not containers
- Heavy data (surface strings, embeddings, full DAGs) lives in executors/blobs
- VarInt encoding for atom IDs in edges still applies

### With ADR-2 (Linguistic Representation)

This ADR is the **concrete encoding of the Lemma layer** from ADR-2.
- Concept layer: uses `kind = 0x5`, pure semantic_id
- Sense layer: expressed as edges between lemma atoms (not own variant)
- Lemma layer: HebrewWord/ForeignWord/Logographic variants
- WordForm layer: derived at parse time via morphology rules; usually NOT stored
  as persistent atoms unless frequency > threshold

**Materialization rule:** A WordForm is persisted as its own atom only if:
- Observed frequency > 10 in local corpus, OR
- Explicitly tagged (proper names, technical terms)
- Otherwise: regenerated from Lemma + features at query time

---

## Storage Implications at Scale

### Target: 10M atoms on laptop (ADR-1 goal)

- 10M atoms × 8 bytes = **80 MB** for atom core
- + 4096 root pool entries × ~32 bytes each = 128 KB (negligible)
- + edges: 1B edges × 6 bytes = 6 GB (dominant)
- **Total: ~6.1 GB** (well under 8 GB laptop budget)

### Compression vs UTF-8 strings

- Average Hebrew word UTF-8: ~10 bytes
- This atom: 8 bytes (with 1 bit room for foreign flag)
- Plus root sharing: ~50 wordforms share 1 root atom
- **Effective bytes per wordform: ~0.2 bytes** when amortized

### Compression vs raw CSV concept files

- CSV: ~40 bytes per row (surface,pos,english,definition,synonyms)
- Atom model: 8 bytes atom + pointers to blob-stored gloss/examples
- **~5x compression on hot data**

---

## What This Decision Supersedes

- Any previous proposal to use strings as primary atom identity
- Any variable-length atom layout (kept consistent at 8 bytes exactly)
- Any per-language root pool (unified Semitic pool wins)

---

## What Remains Open (for later ADRs)

### 1. Root pool synchronization across instances
When federated ZETS instances discover the same new root independently,
how do they merge root_id assignments without renumbering?
Likely answer: content-hash of (root_consonants) → deterministic preliminary ID,
then authoritative pool syncs periodically.

### 2. Language drift / dialectal variants
How to encode Yemenite Hebrew vs Modern Hebrew vs Biblical Hebrew?
Candidates: flag bit + dialect atom edge, or separate language_id values.

### 3. Romanized / Latinized forms
"Shalom" in English text — is this a foreign variant of root ש.ל.מ or a
Foreign Word with its own atom? Recommendation: dual-anchor with cross-edge.

### 4. Gematria as atom property
Every Semitic root has a gematria value (sum of letter values).
Should it be computed on demand (deterministic function of root_id → consonants → values)
or cached in atom flags?
Recommendation: computed on demand. Deterministic, cheap.

### 5. Vowelization / niqud
Modern Hebrew is typically unvocalized. Biblical/religious Hebrew uses niqud.
Binyan + tense + pgn bits implicitly encode vowels.
Full niqud rendering: via generation function, not stored bits.

### 6. Numeral atoms
Should numbers 0-100 be HebrewWord atoms, or their own variant?
Recommendation: separate kind=0xA Numeral variant with direct value encoding.

### 7. Code atoms (for AGI-generates-code use case)
How does "sum_csv_python" procedure atom use this layout?
Likely: kind=0x7 Procedure variant with pointer to DAG structure.
Separate ADR when we get there.

---

## Verification Checklist (for implementation)

Before this decision is considered implemented, code must demonstrate:

- [ ] All three variants (HebrewWord, ForeignWord, Logographic) encode/decode correctly
- [ ] Semitic root pool persists across restarts
- [ ] Root_id assignment is deterministic given ingestion order
- [ ] HebrewWord atom regenerates correct surface form from bits alone
  (for top 1000 most-frequent Hebrew words)
- [ ] ForeignWord atom retrieves correct surface from string pool
- [ ] Logographic atom renders correct codepoint
- [ ] Cross-variant edges work (e.g., HebrewWord "תפוח" linked to ForeignWord "apple")
- [ ] At 10K atoms stored, total RAM usage < 1 MB for atoms (validates 8-byte claim)

---

## Signed

**Architect:** Idan Eldad (עידן אלדד)
**Scribe:** Claude 4.7
**Date:** 2026-04-24 (evening session — end of design sprint day)

**Empirical basis:** `docs/20_research/20260424_hebrew_root_compression_poc.md`

Now the graph is sized. Now the words have shape. Now implementation can start
from ground truth, not theory.

