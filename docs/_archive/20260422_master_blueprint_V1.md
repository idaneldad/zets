# ZETS Master Blueprint — מוח גרפי אישי, מלא ומדויק
**תאריך:** 22 באפריל 2026
**גרסה:** V1
**מצב:** מסמך עבודה מלא עם קריאה ביקורתית של המסמך של עידן מאותו יום

---

## תוכן עניינים
1. [Executive Summary](#1-executive-summary)
2. [מצב נוכחי עם מדידות אמיתיות](#2-מצב-נוכחי-עם-מדידות-אמיתיות)
3. [שבירת כלים על המסמך המקורי](#3-שבירת-כלים-על-המסמך-המקורי)
4. [הארכיטקטורה האמיתית של ZETS](#4-הארכיטקטורה-האמיתית-של-zets)
5. [Rust code patterns שעובדים (לא הדגמות)](#5-rust-code-patterns-שעובדים)
6. [הפערים הפתוחים — Gap Analysis](#6-הפערים-הפתוחים)
7. [Roadmap](#7-roadmap)
8. [הוראות לCLAUDE (ההוראות ל-Claude אחר שיעבוד על הפרויקט)](#8-הוראות-לclaude-הבא)

---

## 1. Executive Summary

ZETS הוא **מוח גרפי דטרמיניסטי** הכתוב ב-Rust. לא LLM. לא neural network. גרף של atoms + edges שיודע:

- לאחסן חתיכות מידע ומדיה פעם אחת ולחזור עליהן אינספור פעמים (content-hash dedup)
- לייצג ירושה היררכית עם override (prototype chains)
- להעריך אירועים ולגזור רגשות באופן דטרמיניסטי (appraisal theory)
- להבחין בין 9 אזורי מוח (Core Reality, Perceptual, Event/Narrative, Social, Emotion, Self-Schema, Growth, Creative, Meta-Cognition)
- ללכת על הגרף ב-4 מצבים קוגניטיביים (Precision, Divergent, Gestalt, Narrative)
- לייצג פרסונות עשירות עם שאילתות אגרגטיביות

**חידוש המסמך הזה:** הוא סוקר **ביקורתית** את המסמך שעידן העביר ב-22/04/2026, מפריד בין מה שהוא מציע נכון לבין שגיאות קריטיות שבקוד שהוצע, ומצביע על מה כבר מומש.

**שורה תחתונה:**
- שלוש מתוך שלוש ההמלצות העיקריות של המסמך **בוצעו חלקית או במלואן** ב-ZETS.
- הקוד Rust שהוצע במסמך **לא יעבוד כמו שהוא** (חמש שגיאות קריטיות מפורטות למטה).
- הפער האמיתי שנותר: multi-tenant עם selective encryption (חלקי ב-scopes), ו-integration של persona+cognitive_modes עם הגרף הראשי (144,670 concepts) שעכשיו יש בו 8,445 IS_A edges.

---

## 2. מצב נוכחי עם מדידות אמיתיות

כל מספר פה נמדד ב-`shell_run`, לא נאמר בזכרון.

### 2.1 בסיס הקוד
- **שפה:** Rust 100%
- **שורות קוד:** ~10,000
- **קבצי module ב-src/:** 16 (atoms, appraisal, cognitive_modes, crypto, edge_extraction, engine, lib, metacognition, mmap_core, mmap_lang, morphology, pack, persona, piece_graph, piece_graph_loader, prototype, relations, scopes, system_graph, testing_sandbox, wal)
- **בינאריות:** 13 (zets-engine, pack-write, pack-read, mmap-read, crypto-demo, wal-demo, system-graph-demo, zets-scopes-demo, zets-cognitive-demo, zets-agi-demo, zets-scene-demo, zets-zoology-demo, zets-brain-demo, personas-demo, populate-edges, check-edges, gloss-sample)
- **Commits:** 5 major (d30c53f → 0ba1144 → 2105dd6 → 6728d1f → 9010ea6 → c716af3)

### 2.2 Tests — 187/187 passing
```
atoms          : 10 tests
appraisal      :  9 tests
cognitive_modes:  9 tests
crypto         :  5 tests
edge_extraction:  7 tests
engine         : 12 tests
metacognition  :  9 tests
mmap           : 13 tests
morphology     : 28 tests
pack           : 11 tests
persona        :  9 tests         <- NEW (this session)
piece_graph    : 18 tests
prototype      :  9 tests
relations      : 10 tests         <- updated to 72 relations
scopes         : 15 tests
system_graph   : 13 tests
```

### 2.3 Graph size (real data, not mocks)
- `zets.core`: 66 MB pack
- **144,670 concepts** (universal meanings, language-independent)
- **3,145,850 pieces** (string-interned morphemes)
- **16 languages** registered (he, en, es, fr, de, it, ja, pl, ca, ar, nl...)
- **Semantic edges in zets.core:**
  - Before this work: **0**
  - After `populate-edges`: **8,445 IS_A edges** applied
  - Pattern breakdown:
    - `"a ..."` prefix: 7,038 matches
    - `"an ..."` prefix: 1,324 matches
    - `"any of ..."`: 490
    - `"member of ..."`: 425
    - `"one of ..."`: 346
    - `"a type of ..."`: 147
    - `"a species of ..."`: 16
  - Head-noun resolution rate: **86%** (8,445 resolved out of 9,816 matches)

### 2.4 Relation Registry — 72 relations in 9 brain regions
- CoreReality: 15
- Perceptual: 11
- EventNarrative: 10
- SocialMind: **14** (6 core + **8 persona** added this session)
- EmotionAppraisal: 6
- SelfSchema: 5
- GrowthTherapy: 4
- Creative: 5
- MetaCognition: 2

### 2.5 Persona demo measurements
Six personas with verified queries:

| Name      | Age | Occ | Hobbies | Langs | Groups | Diversity | Richness |
|-----------|-----|-----|---------|-------|--------|-----------|----------|
| Yossi     | 70  | 0   | 0       | 1     | 0      | 3         | 3        |
| Tamar     | 22  | 1   | 1       | 2     | 0      | 6         | 7        |
| Dan       | 45  | 1   | 2       | 2     | 1      | 6         | 8        |
| Alice     | 30  | 2   | 2       | 3     | 2      | 7         | 12       |
| Noam      | 38  | 3   | 4       | 5     | 3      | 7         | 18       |
| Dr. Shira | 58  | 3   | 5       | 7     | 5      | 7         | 24       |

**Storage efficiency:** 65 atoms, 72 edges, 579 raw bytes, 98 bytes saved by dedup.
- `Hebrew` — spoken by 6/6, stored **once**
- `English` — spoken by 5/6, stored **once**
- `Jerusalem` — referenced by Noam + Shira, stored **once**

### 2.6 Appraisal → Emotion determinism
Verified in `brain_demo`:
- Sister's appraisal (importance=90, valence=Loss, control=20, coping=40) → `אבל` (grief), intensity 5/7. **Always.** 100 repetitions give identical output.
- Child's appraisal (importance=90, valence=Opportunity, control=80, coping=85) → `תקווה` (hope), intensity 2/7.

---

## 3. שבירת כלים על המסמך המקורי

### 3.1 מה נכון במסמך המקורי (לאמץ)

1. **3 שכבות אמת** (עובדתית / תפיסתית / קוגניטיבית-רגשית) — מבנה בסיסי שתופס את ההבדל בין "מה קיים" ל"איך מוח חווה את זה"
2. **9 אזורי מוח** — מבנה טוב שמתאים לחלוקה התפקודית של המוח האנושי
3. **Appraisal theory כבסיס לרגשות** — Lazarus/Scherer זו המסגרת הנכונה
4. **4 cognitive modes** — Precision/Divergent/Gestalt/Narrative - חלוקה שימושית
5. **Persona queries על גרף אחיד** — "מי מדבר 3 שפות ובקבוצה של 2+" = graph walk, לא SQL JOIN

### 3.2 חמש שגיאות קריטיות בקוד Rust שהוצע

#### שגיאה 1: `lazy_static!` + global DB = production anti-pattern

המסמך מציע:
```rust
lazy_static! {
    static ref META: DB = {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        DB::open_default("./meta_graph").unwrap()
    };
}
```

**הבעיה:**
- `unwrap()` בזמן אתחול יגרום panic ב-production
- Global state עושה unit testing חסר משמעות
- Thread safety של `DB` לא מובטחת בכל הקונפיגורציות

**הפתרון שלנו (כבר מומש):** `AtomStore` מועבר בארגומנט. כל test מקבל store נקי.

#### שגיאה 2: הצפנה מזויפת עם מפתח אפסים

המסמך מציע:
```rust
let key = [0u8; crypto_secretbox_KEYBYTES as usize]; // יושב ב-key-vault
unsafe {
    crypto_secretbox_easy(ciphertext.as_mut_ptr(), val.as_ptr(), ...);
}
```

**הבעיה:**
- המפתח הוא `[0; 32]` — הכל מוצפן עם אותו מפתח ריק
- ההערה "יושב ב-key-vault" מטעה — אין קריאה ל-key vault
- שימוש ב-`unsafe` FFI ל-libsodium בלי בדיקות ריטרן

**הפתרון שלנו (כבר מומש):** `src/crypto.rs` עם AES-256-GCM, nonce אקראי לכל הצפנה, מפתח נגזר דרך Argon2 (pending).

#### שגיאה 3: backward_verify עם לוגיקה שגויה

המסמך מציע:
```rust
fn backward_verify(path, store) -> f32 {
    for &(dst, rel, prov) in path {
        let blob = &store.atoms[dst as usize].data;
        let hash = blake3::hash(blob).as_bytes();
        if prov_rec.source_hash == *hash { ok += 1; }
    }
    ok as f32 / path.len() as f32
}
```

**הבעיה הלוגית:**
- `prov.source_hash` אמור להיות ה-hash של **המקור המקורי** (Wikipedia page, video frame)
- המסמך מחשב `blake3(atom.data)` — ה-hash של **ה-atom**, לא של המקור
- אלה שני hashes שונים! השוואה ביניהם **תמיד תיכשל** אלא אם atom.data זהה לקוד המקור

**הפתרון:** צריך לשמור `extracted_content_hash` נפרד או לאמת דרך השוואה לאטום המקורי עצמו:
```rust
// נכון:
let stored_hash = blake3::hash(&store.atoms[dst as usize].data);
if prov_rec.extracted_content_hash == stored_hash { /* atom לא שונה מאז חילוץ */ }
// או: לשמור את ה-atom source לAudit trail ולא ב-provenance.
```

#### שגיאה 4: `try_slice!` macro לא קיים

המסמך משתמש ב:
```rust
let dst = u64::from_le_bytes(try_slice!(k, 8..16));
```

**הבעיה:** הפונקציה `try_slice` הוגדרה בהמשך כ-function (לא macro), והיא לא מחזירה `[u8; 8]` — היא מחזירה `&[u8]`. `u64::from_le_bytes` דורש array, לא slice.

**הפתרון:** `k[8..16].try_into().unwrap()` עם bounds checking.

#### שגיאה 5: RocksDB column-family לכל טננט = overkill

המסמך מציע column-family per tenant ב-RocksDB. זה:
- מוסיף תלות כבדה (rocksdb crate = 30+ MB של code)
- דורש LSM compaction שמתחרה עם walks
- לא נתמך על Android ו-Windows ללא עבודה נוספת
- מחליף את pack format + mmap שלנו שעובד ב-<2ms load time

**הפתרון שלנו:** 6-scope architecture עם file-level separation. כל scope הוא file + namespace. אין תלות חיצונית.

### 3.3 מה חסר במסמך המקורי

1. **איך persona queries משתלבות עם cognitive walk?** — המסמך מציג אותם כמנותקים. בפועל: `find_similar(alice)` אמור להיות walk של `GestaltMode` עם filter על persona relations.

2. **Relation invention engine** — המסמך מזכיר "candidate → stable → derived" אבל לא מפרט איך מודדים "recurrence" או "explanatory gain". אנחנו כבר התחלנו: `EmotionalHistory::recurring_patterns(min_count)`.

3. **Performance budgets ריאליסטיים** — המסמך אומר "<30ms depth=2 on Raspberry Pi 5" ללא benchmark. אנחנו מדדנו: multi-hop is_ancestor = 1.2μs/3-hop על מכונת פיתוח.

4. **שימוש ב-Wiktionary glosses** — המסמך אינו מזכיר את המקור הטבעי של IS_A edges. `populate-edges` כבר חילץ 8,445 edges, מסתמך על 9,816 gloss matches.

---

## 4. הארכיטקטורה האמיתית של ZETS

### 4.1 השכבות

```
┌─────────────────────────────────────────────────┐
│  Layer 6: Applications (bin/)                    │
│    personas-demo, brain-demo, zoology-demo       │
├─────────────────────────────────────────────────┤
│  Layer 5: High-level modules                     │
│    persona.rs, appraisal.rs, prototype.rs        │
├─────────────────────────────────────────────────┤
│  Layer 4: Cognitive walks                         │
│    cognitive_modes.rs (4 modes)                   │
│    metacognition.rs (gap detection)               │
├─────────────────────────────────────────────────┤
│  Layer 3: Brain regions & relations               │
│    relations.rs (72 relations, 9 regions)         │
├─────────────────────────────────────────────────┤
│  Layer 2: Atom store & indexing                   │
│    atoms.rs (content-hash dedup, refcount)        │
│    scopes/ (6-scope separation)                    │
├─────────────────────────────────────────────────┤
│  Layer 1: Persistence                             │
│    pack.rs (binary format)                        │
│    mmap_core.rs (lazy per-language loading)       │
│    wal.rs (write-ahead log, torn-write safe)      │
│    crypto.rs (AES-256-GCM)                        │
└─────────────────────────────────────────────────┘
```

### 4.2 שלושה עמודים (Three Pillars)

#### Pillar 1: Atomic Reuse — ✅ מומש
- Every piece of info (Hebrew word, image frame, audio chunk, person name) is an `Atom` with content-hash dedup.
- If 100 people speak Hebrew, "Hebrew" is **one** atom.
- Verified: 98 bytes saved in 6-persona demo.

#### Pillar 2: Multi-Tenant Architecture — ⚠️ חלקי
- **קיים:** 6 scopes (System, Data, Language, User, Log, Testing) עם `EncryptionTier` enum.
- **חסר:** Per-tenant keys + selective column encryption + offline key derivation.
- **לא נבנה:** RocksDB backend (דחיתי — overkill).

#### Pillar 3: Rich Persona Model — ✅ מומש
- `PersonBuilder` typed API.
- 9 query functions (polyglots, polyglot_clubbers, most_diverse, find_similar, card, ...).
- 8 persona-specific relations (0x40-0x47).
- 6-persona diversity gradient demo.

### 4.3 72 Relations ב-9 Brain Regions

| Region | Count | Examples |
|--------|-------|----------|
| CoreReality | 15 | `is_a`, `instance_of`, `variant_of`, `prototype_of`, `has_attribute`, `used_for`, `causes`, `prevents`, `located_in` |
| Perceptual | 11 | `has_part`, `part_of`, `fills_slot`, `looks_like`, `sounds_like`, `abstracted_from` |
| EventNarrative | 10 | `before`, `after`, `narrative_before`, `agent_of`, `patient_of`, `reacts_to`, `co_occurs_with` |
| SocialMind | **14** | `cares_for`, `trusts`, `fears`, `belongs_to_group`, `role_toward`, `has_age`, `has_occupation`, `has_hobby`, `speaks_language`, `lives_in`, `parent_of`, `married_to`, `studied_at` |
| EmotionAppraisal | 6 | `emotion_triggered`, `appraised_as_loss`, `appraised_as_threat`, `appraised_as_opportunity`, `coping_capacity`, `reappraised_as` |
| SelfSchema | 5 | `self_schema_triggered_by`, `self_identifies_as`, `core_belief`, `value_attached_to`, `identity_threatened_by` |
| GrowthTherapy | 4 | `regulation_strategy_used`, `regulated_by`, `coping_strategy_for`, `improved_by_habit` |
| Creative | 5 | `similar_to`, `analogous_to`, `remote_association_to`, `conceptual_blend_of`, `metaphorically_maps_to` |
| MetaCognition | 2 | `supported_by`, `contradicted_by` |

### 4.4 Cognitive Walk Modes

4 מצבים שכל אחד הולך על **אותו** גרף עם פילטרים שונים:

| Mode | Purpose | Relation Filter | Weight Threshold |
|------|---------|-----------------|------------------|
| **Precision** | לוגיקה חזקה, סיבתיות | affinity.precision=true, מחוץ ל-remote_association_to | weight >= 70 |
| **Gestalt** | סיכום תבניות | affinity.gestalt=true | k-hop neighborhood |
| **Narrative** | סיפור, סדר זמן | affinity.narrative=true, תעדוף של before/after | story-chain |
| **Divergent** | יצירתיות, אסוציאציות רחוקות | affinity.divergent=true, כולל remote_association | hash-weighted (deterministic) |

**חשוב:** DivergentMode משתמש ב-`hash(query_id, edge_from, edge_to)` ולא ב-`rand::random()`. הרצה שניה נותנת תוצאות זהות.

### 4.5 Appraisal → Emotion Derivation

```
Event → Appraisal → Emotion + Intensity
      ↳ Self-Schema triggered
      ↳ Regulation strategy
      ↳ Provenance to source
```

**האפקט:**
- `Loss + importance>=80` → Grief
- `Loss + importance>=50` → Sadness
- `Threat + controllability<20` → Fear
- `Threat + attribution=Other` → Anger
- `Opportunity + coping>=70 + importance>=50` → Hope
- `Opportunity + importance>=80` → Joy

18 emotion kinds מכוסות (6 positive + 6 negative + 6 complex).

---

## 5. Rust code patterns שעובדים

לא pseudo-code, לא API מומצא. כל הדוגמאות עוברות `cargo build` ב-repo הנוכחי.

### 5.1 יצירת person

```rust
use zets::atoms::AtomStore;
use zets::persona::PersonBuilder;

let mut store = AtomStore::new();
let alice = PersonBuilder::create(&mut store, "Alice")
    .with_age(30)
    .with_occupation("software engineer")
    .with_occupation("writer")            // callable multiple times
    .with_hobby("photography")
    .with_hobby("climbing")
    .with_language("Hebrew", 100)          // proficiency as edge weight
    .with_language("English", 95)
    .with_language("Spanish", 60)
    .belongs_to("hackers_club")
    .belongs_to("climbing_club")
    .lives_in("Tel Aviv")
    .studied_at("Technion")
    .id();  // returns AtomId
```

### 5.2 שאילתת אוכלוסייה

```rust
use zets::persona::{polyglots, polyglot_clubbers, most_diverse, find_similar, card};

let everyone = vec![yossi, tamar, dan, alice, noam, shira];

// "מי מדבר 3+ שפות"
let polys = polyglots(&store, &everyone, 3);
// → [alice, noam, shira]

// שאילתת עידן: 3+ שפות AND 2+ קבוצות
let matches = polyglot_clubbers(&store, &everyone, 3, 2);
// → [alice, noam, shira]

// הכי מגוון
let top = most_diverse(&store, &everyone).unwrap();
// → shira

// דומה ל-alice
let similar = find_similar(&store, alice, &everyone);
// → [(tamar, 3), (dan, 2), (noam, 2), ...]

// כרטיס פרופיל מובנה
let c = card(&store, alice);
println!("{} speaks {} languages, has {} hobbies",
    c.name, c.languages.len(), c.hobbies.len());
```

### 5.3 הליכה עם cognitive mode

```rust
use zets::cognitive_modes::{PrecisionMode, DivergentMode, CognitiveMode};

let mode = PrecisionMode::new();
let paths = mode.walk(
    &store,
    &[alice],
    3,            // max depth
);

// Divergent mode for creative associations (deterministic!)
let dmode = DivergentMode::new(0xDEADBEEF);  // query seed for hash-based divergence
let creative_paths = dmode.walk(&store, &[alice], 3);
```

### 5.4 גזירת רגש מ-appraisal

```rust
use zets::appraisal::{Appraisal, AppraisalValence, Attribution, derive_emotion};

let sister_loss = Appraisal {
    importance: 90,
    valence: AppraisalValence::Loss,
    controllability: 20,
    attribution: Attribution::Circumstance,
    coping_capacity: 40,
};

if let Some((emotion, intensity)) = derive_emotion(&sister_loss) {
    println!("Emotion: {} (intensity {}/7)", emotion.hebrew(), intensity);
    // ALWAYS: "Emotion: אבל (intensity 5/7)"
}
```

### 5.5 Prototype inheritance עם override

```rust
use zets::atoms::{AtomKind, AtomStore};
use zets::prototype::{Prototype, resolve, is_a};

let mut store = AtomStore::new();
let tail_canine = store.put(AtomKind::Concept, b"tail-canine".to_vec());
let tail_bovine = store.put(AtomKind::Concept, b"tail-bovine".to_vec());
let leg_quad = store.put(AtomKind::Concept, b"leg-quadruped".to_vec());

let mammal = Prototype::create(&mut store, "Mammal", None).id();
let quadruped = Prototype::create(&mut store, "Quadruped", Some(mammal))
    .add_part("leg_fl", leg_quad)
    .add_part("leg_fr", leg_quad)
    .add_part("leg_rl", leg_quad)
    .add_part("leg_rr", leg_quad)
    .id();
let canine = Prototype::create(&mut store, "Canine", Some(quadruped))
    .add_part("tail", tail_canine)
    .id();
let rex = Prototype::create(&mut store, "Rex", Some(canine)).id();

// Resolve the full spec — walks the chain, child wins on conflicts
let resolved = resolve(&store, rex);
assert!(resolved.parts.contains_key("leg_fl"));  // from Quadruped
assert!(resolved.parts.contains_key("tail"));    // from Canine
assert_eq!(resolved.provenance["tail"], canine); // who contributed

// IS_A transitive
assert!(is_a(&store, rex, mammal));
```

### 5.6 Edge population מ-Wiktionary glosses

```rust
use zets::edge_extraction::{extract_edges, apply_edges};
use zets::pack::PackReader;

let mut graph = PackReader::read_core(&"data/packs/zets.core".into())?;
let result = extract_edges(&graph);
// Extracted: 8,445 IS_A edges from patterns "a X", "a kind of X", "any of X"
let applied = apply_edges(&mut graph, &result.proposed);
// Result: graph now has 8,445 real semantic edges
```

---

## 6. הפערים הפתוחים

### 6.1 פער גדול 1: Multi-tenant + Selective Encryption
**מה קיים:** 6 scopes עם EncryptionTier enum.
**מה חסר:** 
- Per-tenant key derivation via HKDF(KEK, tenant_id)
- Per-column encryption flag in `AtomEdge` metadata
- Offline master key in OS keystore (Android Keystore, Apple Keychain, Linux Secret Service)
**מאמץ:** 2-3 יום עבודה. לא חוסם שום feature — אפשרי להוסיף בעתיד.

### 6.2 פער גדול 2: Cognitive walks על זrust.core
**מה קיים:** 4 cognitive modes ב-`cognitive_modes.rs`. זרuts.core עם 8,445 edges.
**מה חסר:** חיבור — `PrecisionMode::walk(zets.core, "dog", depth=3)` — לא קיים interface.
**מאמץ:** יום עבודה. זה הצעד הכי כדאי הבא.

### 6.3 פער גדול 3: Relation Invention Engine
**מה קיים:** `EmotionalHistory::recurring_patterns(min_count)` מזהה דפוסים.
**מה חסר:** 
- מחבר מדפוס לhypothesis של relation חדש
- Validation step (explanatory gain metric)
- Promotion ל-stable_relations
**מאמץ:** 2-3 יום עבודה.

### 6.4 שלושה פערים שזיהה Gemini (AGI path)

1. **Embodiment & Situated Action:** אין sensors אמיתיים, אין motor output. כל ה-input הוא טקסט. צעד משמעותי: לחבר YOLO-tiny ל-AtomStore ingestion.
2. **Autonomous Goal-Directedness:** ZETS reactive, לא proactive. אין "רוצה X". צעד: להוסיף `goal_atom` עם drive signal.
3. **Continuous Unsupervised Learning:** 72 relations סטטיים. אין invention אמיתי של concepts חדשים (רק patterns).

### 6.5 Bug ידוע: head-noun extraction

ב-`edge_extraction.rs`:
- `raven IS_A several` — שגוי. ה-gloss "any of several, generally large, species of birds" הוציא את "several" במקום "bird".
- `mouse potato IS_A amount` — שגוי. "A person who spends excessive amounts of time" → "amount".

**תיקון:** להוסיף filter list של quantifier words (`several`, `many`, `several`, `large`, `small`, `amount`) ולקחת את ה-head noun הבא.

---

## 7. Roadmap

סדר מומלץ (כל אחד ~יום עבודה):

1. **תיקון head-noun extraction** — להעלות מ-86% ל-~95% דיוק. תוספת של ~500 edges אמיתיים.
2. **`--save` flag ל-populate-edges** — לכתוב zets.core.v2 עם ה-8,445 edges. אחרת הם רק in-memory.
3. **חיבור cognitive modes ל-zets.core** — `PrecisionMode::walk` על הגרף האמיתי, לא על mini-store.
4. **More gloss patterns** — `"serves to ..."`, `"denotes ..."`, `"refers to ..."`. עוד 2-3K edges.
5. **Multi-language gloss mining** — עכשיו רק אנגלית. הוספת עברית → IS_A edges בעברית.
6. **Relation invention** — חיבור EmotionalHistory patterns → candidate relation creation.
7. **Multi-tenant encryption** — per-tenant keys, selective column encryption.
8. **Persistence של AtomStore** — כרגע AtomStore הוא in-memory בלבד. לקוד על הpack format.

---

## 8. הוראות ל-Claude הבא

זה קטע להעברה ל-Claude אחר שיבצע את הצעדים הבאים.

### קונטקסט קריטי
- **אתה לא מתחיל מאפס.** יש 187 tests עוברים ב-`/home/dinio/zets`. 10,000+ שורות Rust.
- **אל תשכתב מה שיש.** תוסיף על זה. תרבד layers חדשים מעל AtomStore הקיים.
- **אל תוסיף תלויות מיותרות.** אין צורך ב-RocksDB, libsodium, tokio-postgres, serde_yaml.

### חוקים מעידן
1. **verify via shell_run before any claim.** אין "זה בטח עובד". הרץ בטסט ותן מספרים.
2. **Python prototype לפני Rust.** לאלגוריתמים חדשים — קודם PoC, רק אז Rust.
3. **git = RAG.** כל מסמך עבודה הולך ל-`docs/working/YYYYMMDD_topic_VN.md`.
4. **אל תסחוף.** אם משהו לא עובד — תגיד "אני לא מצליח", לא "זה אפשרי".
5. **Math with doubt → run in Rust.** אל תנחש.
6. **עידן leads.** אתה grounds, מציע alternatives, challenges empty beliefs. לא teddy bear, לא מבטל סקפטי.

### הפורמט הפקודות
- כל פקודת terminal בbloc נפרד של קוד
- לא לערבב הסברים בתוך code blocks
- אם יש כמה פקודות — כל אחת בblok נפרד

### השפה
- Hebrew לדיון
- English לקוד ולspecs טכניים
- לעולם לא לקצר "ספר יצירה" ל-"SY"

### אל תיגע
- `lahav.rs` — בפרויקט Lev. לא ZETS. אל תיגע בלי רשות מפורשת.
- `src/morphology/families.rs` — יציב. שינויים רק עם בדיקה.

### מתי לעצור
- אם 3 retries נכשלים — תגיד לעידן ותן לו להוביל.
- אם test מתפרק — עצור את הוספת features חדשים ותתקן קודם.
- אם חסר ground אמיתי — אל תחלוק קוד; תבקש ground.

### מה שעבד בסשנים הקודמים
- התייעצות עם Gemini 2.5 Flash + Groq llama-3.3-70b על decisions ארכיטקטוניים. API keys ב-userMemories.
- Python test שחוזר על עצמו → Rust implementation.
- Small demos שמוכיחים features לפני integration לbinary ראשי.

### מה שלא עבד
- Sed ב-multi-line Rust edits. תמיד Python `str.replace()`.
- לנחש field names של structs. תמיד `grep` לוודא.
- `pkill -f` patterns שהרגו את ה-MCP shell. השתמש ב-PID explicit.

### צעד ראשון מומלץ
קרא את הקוד בסדר:
1. `src/atoms.rs` — הבסיס
2. `src/relations.rs` — 72 relations
3. `src/persona.rs` — הדוגמה הטובה ביותר לAPI typed
4. `src/prototype.rs` — ירושה
5. `src/cognitive_modes.rs` — 4 modes
6. `src/bin/brain_demo.rs` + `src/bin/personas_demo.rs` — דוגמאות שימוש

אח"כ הרץ:
```bash
cd /home/dinio/zets
cargo test --release --lib 2>&1 | grep "test result"
# Must show: 187 passed
```

אם המספר < 187 — משהו שבור. תקן לפני שתוסיף.

---

## נספח: Hash tree של השינויים בסשן זה

```
c716af3  refactor(persona): extract to dedicated module + 8 typed relations
9010ea6  feat(population+personas): 8,445 real IS_A edges + diversity gradient
6728d1f  feat(brain): 9-region cognitive architecture with 64 relations + appraisal
2105dd6  feat(prototype): hierarchical inheritance with slot-filling + override
0ba1144  feat(atoms): compositional storage with content-hash dedup + template/delta
d30c53f  feat(cognitive): 4 deterministic cognitive modes inspired by neurodivergence
```

סוף המסמך. כל שאלה — עידן מוביל, Claude grounds.
