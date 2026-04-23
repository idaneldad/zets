# Folding / Compression Design — BPE + Hash Consing — deep shevirat kelim

**תאריך:** 23.04.2026
**מקור:** עידן קיבל הצעה (מ-Gemini או ChatGPT): BPE + Hash Consing + Merkle DAG.
עידן: "תעשה שבירת כלים מעולה, לא רק לקבל כמו שזה נשמע חכם, ותתייעץ שוב."

**חוק:** לא רק "זה כלי מפורסם, בטח יעבוד." לחשוב שוב. לבנות tester. להריץ.

---

## מה כבר קיים ב-ZETS (בדיקה אמיתית)

```
✅ src/atoms.rs:32   content_hash() FNV-1a — deduplicates atoms by bytes
✅ src/atoms.rs:146  content_hash → atom_id index (hash consing ברמת leaf)
✅ src/piece_graph.rs:23  "Pieces are content-addressed"
❌ אין BPE pair-merging
❌ אין recursive folding לunit גדול יותר מ-piece
❌ אין compression ratio tracking
```

**משמעות:** חצי מההצעה כבר מיושם. החלק החדש הוא **pair merging iterative + Merkle DAG structure**.

---

## 10 שבירות כלים על ההצעה

### 🔨 #1 "BPE תמיד עובד"
**השבר:** BPE הוא **greedy heuristic, NP-hard בצורתו האופטימלית**. הצעד שלוקח את ה-pair הכי נפוץ **יכול לחסום** merges טובים יותר בהמשך.

**דוגמה:** "בינה מלאכותית" + "בינה חישובית" — שניהם מופיעים. BPE יבחר "בינה" כ-merge ראשון. אבל אם היינו ממזגים "מלאכותית" ו"חישובית" קודם (קל יותר?) — תוצאה שונה.

**תיקון:** BPE עם **backtracking מוגבל**, או **N-gram analysis תחילה** לתכנון, או **look-ahead של 2-3 merges**.

### 🔨 #2 "Hash collisions לא קורות"
**השבר:** **Birthday paradox.** עם FNV-1a 64-bit ו-10^9 atoms, הסיכוי לconflict הוא **~2.7%**. לא zero!

**Math:**
- 64-bit hash space = 1.84 × 10^19
- Birthday collision probability at n items = 1 - e^(-n²/2H)
- n=10^9: p ≈ 0.027 (2.7%)
- n=10^10: p ≈ 94% — כמעט וודאות

**תיקון:** 
- Use **SHA-256** ל-Merkle DAG (128-bit effective → collision-safe ל-2^64 atoms)
- OR: **hash + length + first-4-bytes** של content → practically unique at 80 bits

### 🔨 #3 "Recursive folding = exponential compression"
**השבר:** לא. compression ratio מתכנס לאחוז קטן **מהשפה עצמה**, לא מהdepth.

**המציאות:**
- BPE על English: ~4× compression (GPT-4 tokenizer = 4 chars/token)
- BPE על Hebrew: ~3× (Hebrew denser)
- Merkle DAG structure: 10-30% שיפור נוסף לרפטיציות
- **Total: ~5-6× compression max על טקסט**. לא 100× ולא 1000×.

**תיקון expectations:** 14.5M articles × 20KB avg = 290GB → 50GB מציאותי, לא 3GB.

### 🔨 #4 "Leaves = flat atoms"
**השבר:** למה "מילה" leaf? למה לא character-level? Byte-level? Token-level?

**3 אסטרטגיות שונות:**
- **Byte-level** (256 leaves): max compression, slow walks (deep trees)
- **Char-level** (Unicode, ~150K chars): balanced
- **Word-level** (הdefault של BPE טקסט): surface meaning but misses morphology
- **Morpheme-level** (Hebrew ש+ל+ו+ם): linguistic compression

**תיקון:** תלוי במodality:
- Text: morpheme (Hebrew) or word (English)
- Code: token
- Audio: phoneme
- Image: region patch

### 🔨 #5 "Walk is O(depth)"
**השבר:** walk = **cache misses**. Depth-20 fold = 20 random memory accesses.

**מציאות:**
- L1 cache hit: 1ns
- L2 cache miss: 10ns
- L3 cache miss: 30ns
- RAM access: 100ns
- **Depth 20 × 100ns = 2 microseconds לקריאת atom אחד** (אם כל אחד cache miss)

**לעומת flat storage:** 1 memcpy של 100 bytes = ~50ns

**תיקון:**
- Limit fold depth (say 6-10 max)
- Inline-unfold hot atoms (frequently-walked)
- LRU of unfolded forms

### 🔨 #6 "Folding = encryption"
**עידן אמר:** "קיפול = גם עוד הצפנה כי לא פשוט להבין, נכון?"

**השבר לעיקר:** לא אמיתי. **Obfuscation ≠ Encryption.**
- Folded graph: אם יש לך **את הכל**, אתה יכול לפרש. אין key.
- Encryption: **גם אם יש לך הכל**, צריך key.

**אבל:** folding **מוסיף partial privacy** — if you steal only part of the graph (say, "atom 57493 = fold(12, 84)" without atoms 12, 84) — you can't unfold.

**תיקון:** Folding = **structural integrity + partial privacy**. לא תחליף ל-AES.

### 🔨 #7 "Fold during ingestion vs background"
**עידן שאל בסוף ההצעה.** שני המודלים (BPE-wise) **מסכימים** שבackground מנוחה נכון. אבל למה?

**טעם טכני:**
- BPE scan = O(n²) בגרף. כבד.
- Writing user request pipeline shouldn't block on BPE
- Batch is better: 1000 atoms בבת אחת → better pair statistics

**תיקון:** Write-Ahead Log (WAL) → batch fold every 10 min OR 100K atoms, whichever first. ZETS **כבר יש WAL**.

### 🔨 #8 "BPE is universal"
**השבר:** BPE נבנה לטקסט. לא אומר שהוא נכון לכל הכלים.

- **Graph edges:** BPE לא עובד ב-edges. מה זה "שני edges דומים"? אותו source? אותה relation?
- **Audio phonemes:** כבר יש פירוק. BPE ברמת triphones?
- **Image regions:** מה זה "זוג נפוץ" של regions? לא מוגדר.
- **Float vectors:** BPE לא חל כלל.

**תיקון:** BPE **per-modality**. Text gets BPE. Edges get **pattern mining** (subgraph isomorphism). Audio gets triphone compression. Images get patch quantization.

### 🔨 #9 "Hash consing means no duplicates ever"
**השבר:** רק אם ה-**input identical**. **Near-duplicates** (שינוי אות, punctuation) יוצרים atoms נפרדים.

- "Hello World" vs "Hello, World" → שני atoms
- "hello" vs "Hello" → שני atoms
- 10,000 articles שכולן מכילות "Wikipedia is a free encyclopedia" — **10,000 atoms**, כל אחד שונה ב-context

**תיקון:** **Normalization layer לפני hash**:
- Lowercase (unless proper noun)
- Whitespace normalize
- Punctuation strip
- Unicode NFKC

ZETS **כבר עושה חלק מזה** ב-ingestion, אבל לא עקבי.

### 🔨 #10 "Folding is forward-only"
**השבר:** הצעת BPE **בונה** vocabulary. מה אם שפה/תחום חדש מגיע? המילון **כבר הושלם**.

**מציאות:**
- Re-BPE = יקר (יוצר IDs חדשים → כל הedges צריכים update)
- Solution: **hierarchical vocabularies** — core BPE (frozen) + domain-specific (mutable)

**תיקון:** `src/fold/vocab.rs` יש **2 שכבות**:
- Base vocabulary (universal, 100K entries)
- Domain extensions (per-topic, growable)

---

## התכנית המתוקנת (אחרי שבירת כלים)

### Architecture
```
┌─────────────────────────────────────────────────┐
│                   WAL (append-only)              │
│     [atom_new, atom_new, atom_new, ...]         │
└──────────────────┬──────────────────────────────┘
                   │ every 10min OR 100K atoms
┌──────────────────▼──────────────────────────────┐
│            Background Fold Task                  │
│  1. Normalize: lowercase, NFKC, whitespace      │
│  2. Scan for frequent pairs (within modality)    │
│  3. Check hash collision via SHA-256             │
│  4. Apply merge (if novel) or reuse (if exists) │
│  5. Depth cap at 8 (prevent cache thrashing)    │
│  6. Update compression metrics                   │
└──────────────────┬──────────────────────────────┘
                   │ folded atoms + merge rules
┌──────────────────▼──────────────────────────────┐
│          mmap-backed fold structure              │
│  - Base vocab (frozen, ~100KB)                   │
│  - Domain extensions (per-topic, ~1MB each)     │
│  - Fold rules (atom_id → (child1_id, child2_id))│
│  - LRU cache of unfolded hot atoms              │
└──────────────────┬──────────────────────────────┘
                   │
              walk / query / unfold
```

### Modality-specific folding
- **Text (Hebrew):** morpheme-level BPE (root+prefix+suffix patterns are gold)
- **Text (English):** word-level BPE
- **Audio:** triphone folding (/k-a-t/ common in many words)
- **Graph edges:** subgraph pattern mining (e.g., "X is_a Y, Y is_a Z" ← common path)
- **Images:** 8x8 patch quantization + VQ-VAE-style codebook
- **Numbers:** no folding (already compact)

### Hash safety
- **SHA-256 for Merkle DAG IDs** (32 bytes overhead OK, collision-free)
- **FNV-1a stays for leaf content-hash** (speed)
- **Collision verify:** if FNV hit, compare raw bytes before deduplication

### Realistic compression targets
| Content | Raw | Folded | Ratio |
|---------|-----|--------|-------|
| English Wikipedia text | 100 GB | 18 GB | 5.5× |
| Hebrew Wikipedia text | 1.2 GB | 400 MB | 3.0× |
| Code corpus | 50 GB | 8 GB | 6.3× |
| Conversation logs | 10 GB | 1.5 GB | 6.7× |
| Audio phonemes | 500 MB | 200 MB | 2.5× |
| Image atoms (metadata) | 100 MB | 50 MB | 2.0× |
| **Total ZETS (14.5M articles)** | **~290 GB** | **~50 GB** | **5.8×** |

**לא 100× ולא 1000×.** 5-6× realistic.

### "Folding = encryption" clarification
- **Structural:** YES — stolen partial graph is unreadable
- **Cryptographic:** NO — if someone has full graph, they read it
- **Add AES-GCM layer on top** for real encryption (crypto.rs already in ZETS)

---

## Python prototype — BPE + Hash Consing + depth analysis

עומד להריץ `py_testers/test_folding_v1.py` עם:
1. **BPE על corpus אמיתי** — 1MB Hebrew text
2. **Hash consing 64-bit vs 256-bit** — birthday collision measured
3. **Depth vs compression tradeoff** — graph at depth 4, 8, 16
4. **Walk speed** — ms to unfold atom at various depths
5. **Per-modality BPE works/fails** (text yes, edges no)
6. **Comparison to raw** — actual ratio on real data

**לא הולך לבנות את כל ה-Rust.** רק להוכיח שזה מתאים לפני.

---

## למה אני רוצה להתייעץ שוב

האסימטריה:
- הצעה הראשונה (BPE + Hash Consing) נשמעת חכמה ומרשימה
- אבל היא **generic** — לא תפורה ל-ZETS
- אין בה התייחסות ל:
  - Modality differences (לא הכל טקסט)
  - Depth tradeoff (cache misses)
  - Birthday collision at scale
  - Normalization חובה לפני hash
  - Mutable vs frozen vocabularies
  - Edge folding (different problem)

**המודלים לא בהכרח יודעים את הניואנסים האלה** — הם ממליצים BPE כי זה "המותג." צריך לאתגר אותם.

## מה אני שולח עכשיו — פרומפט חדש

**Key framing:** "ZETS already has hash consing at leaf level via FNV-1a. What are the TRUE advantages of ADDING BPE pair-merging on top? Consider cache misses, modality mismatch, and vocabulary mutability. What are the MIT/Google compression labs doing NOW (2025/2026) that isn't BPE from 2015?"
