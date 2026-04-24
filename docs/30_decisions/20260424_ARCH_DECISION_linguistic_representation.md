# ARCHITECTURE DECISION: Linguistic Representation (Words → Senses → Concepts)

**Status:** ACCEPTED (Idan, 2026-04-24)
**Supersedes:** Any implicit "word = concept" assumption in prior docs
**Builds on:** ADR "Atom as Sigil, Executor as Doer" (same day)
**Type:** Architecture Decision Record — binding

---

## The Decision (in one sentence)

> **Thought happens at the Concept layer (language-agnostic). Words are surface
> forms attached via Sense atoms. Grammar (gender, number, tense, definiteness,
> stacking prefixes) lives on edges, not in concepts. Agreement is computed at
> realization time, not stored as data.**

---

## The Four Layers of Language

```
┌──────────────────────────────────────────────────────────────────────┐
│  LAYER 4 — CONCEPT (what the thought IS)                              │
│                                                                        │
│  Pure meaning. No language. No grammar. Universal.                    │
│                                                                        │
│  Examples:                                                             │
│    #VEHICLE_POWERED_BY_ENGINE                                          │
│    #COLOR_RED                                                          │
│    #FRUIT_MALUS_DOMESTICA (apple)                                      │
│    #GREETING_OPEN                                                      │
│    #NUMBER_11                                                          │
│    #ACTION_WALK                                                        │
│                                                                        │
│  This is where REASONING happens.                                     │
│  This is what AGI "thinks in".                                        │
└────────────────────┬─────────────────────────────────────────────────┘
                     │ expressed_by
                     ▼
┌──────────────────────────────────────────────────────────────────────┐
│  LAYER 3 — SENSE (abstract meaning in a context)                      │
│                                                                        │
│  Still language-agnostic, but finer granularity than Concept.         │
│  Captures polysemy. One concept may have several senses.              │
│                                                                        │
│  Examples:                                                             │
│    sense:car.automobile        → #VEHICLE_POWERED_BY_ENGINE           │
│    sense:car.railway_car       → #VEHICLE_RAIL                        │
│    sense:red.color             → #COLOR_RED                            │
│    sense:red.political         → #IDEOLOGY_COMMUNIST                  │
│    sense:greeting.open         → #GREETING_OPEN                       │
│    sense:greeting.close        → #FAREWELL                             │
│    sense:peace.state           → #PEACE                                │
│                                                                        │
│  Why we need this middle layer:                                        │
│    שלום expresses THREE senses (open, close, peace)                   │
│    hello expresses ONE sense (open)                                    │
│    word:שלום --SAME_AS--> word:hello would be WRONG                   │
│    Instead: both express sense:greeting.open (shared)                 │
└────────────────────┬─────────────────────────────────────────────────┘
                     │ expressed_by
                     ▼
┌──────────────────────────────────────────────────────────────────────┐
│  LAYER 2 — LEMMA (dictionary form, per language)                      │
│                                                                        │
│  The "citation form" of a word. Language-specific.                    │
│  This is what appears in dictionaries.                                │
│                                                                        │
│  Examples:                                                             │
│    lemma:תפוח     (he) → sense:apple.fruit                            │
│    lemma:apple    (en) → sense:apple.fruit                            │
│    lemma:manzana  (es) → sense:apple.fruit                            │
│                                                                        │
│    lemma:מכונית   (he) → sense:car.automobile                         │
│    lemma:אוטו     (he) → sense:car.automobile  (informal register)    │
│    lemma:רכב      (he) → sense:vehicle.general (broader)              │
│    lemma:car      (en) → sense:car.automobile                         │
│                                                                        │
│  Each lemma has grammatical properties:                               │
│    lemma:מכונית  → gender=Feminine, type=Noun                         │
│    lemma:אדום    → gender=agrees_with_noun, type=Adjective            │
└────────────────────┬─────────────────────────────────────────────────┘
                     │ inflects_to
                     ▼
┌──────────────────────────────────────────────────────────────────────┐
│  LAYER 1 — WORDFORM (surface form, as written/spoken)                 │
│                                                                        │
│  Every inflected/derived variant that appears in actual text.         │
│                                                                        │
│  Examples for lemma:מכונית:                                            │
│    wordform:מכונית       (fem, sg, indef)                             │
│    wordform:המכונית      (fem, sg, def)                               │
│    wordform:מכוניות      (fem, pl, indef)                             │
│    wordform:המכוניות     (fem, pl, def)                               │
│    wordform:במכונית      (locative + fem sg indef)                    │
│    wordform:ומהמכוניות   (conj + from + def + fem pl)                │
│                                                                        │
│  Each wordform knows:                                                  │
│    - its lemma                                                         │
│    - the morphological features that produced it                      │
│    - the prefixes/suffixes stripped                                   │
│                                                                        │
│  This is what the tokenizer sees and what the generator emits.        │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Worked Example 1 — "תפוח"

### The graph representation

```
Concept:       #FRUIT_MALUS_DOMESTICA
                  ↑
                  │ expresses
                  │
Sense:         sense:apple.fruit
                  ↑
                  │ expressed_by
                  │
Lemma:         lemma:תפוח (he) [gender=Masculine, type=Noun]
                  ↑
                  │ inflects_to
                  │
WordForms:     wordform:תפוח       (masc, sg, indef)
               wordform:התפוח      (masc, sg, def)
               wordform:תפוחים     (masc, pl, indef)
               wordform:התפוחים    (masc, pl, def)
               wordform:בתפוח      (locative + masc sg indef)
               wordform:מהתפוחים   (from + def + masc pl)
```

### Same concept across languages

```
Concept #FRUIT_MALUS_DOMESTICA ←expressed_by← sense:apple.fruit
                                                      │
                         ┌────────────────────────────┼─────────────────────┐
                         ↓                            ↓                     ↓
                  lemma:תפוח (he)              lemma:apple (en)    lemma:manzana (es)
                  gender=Masculine             no gender           gender=Feminine
                         │                            │                     │
                  15+ wordforms                 6 wordforms         8+ wordforms
```

**One concept. One sense. Three lemmas (one per language). Many wordforms each.**

---

## Worked Example 2 — "מכונית אדומה"

This is a **compositional phrase**. Not one atom — a structure in working memory.

### Step 1: Parse each word

```
"מכונית" → wordform:מכונית → lemma:מכונית [Feminine, Singular, Noun]
                              → sense:car.automobile → #VEHICLE_POWERED_BY_ENGINE

"אדומה" → wordform:אדומה → lemma:אדום [gender_agrees, Adjective]
                           → sense:red.color → #COLOR_RED
                           morphological features: [Feminine, Singular]
```

### Step 2: Detect agreement

The morphology analyzer on "אדומה" recognizes the **ה suffix** with features
`[Feminine, Singular]`. Since מכונית is Feminine Singular → **agreement confirmed**.

### Step 3: Build phrase atom (in working memory)

```
PhraseAtom:phrase_47  (ephemeral, in working memory)
├── head         → lemma:מכונית  (the noun — center of phrase)
├── modifier     → lemma:אדום    (the adjective)
├── relation     → HAS_PROPERTY_COLOR
├── agreement    → [gender=Feminine, number=Singular]
├── definiteness → Indefinite (no ה prefix observed)
└── concept_structure:
    #VEHICLE_POWERED_BY_ENGINE ⊗ HAS_PROPERTY_COLOR ⊗ #COLOR_RED
```

### Step 4: Reasoning happens on the Concept structure

The graph walks reason about:
```
#VEHICLE_POWERED_BY_ENGINE ⊗ #COLOR_RED
```

**Language-free.** Hebrew grammar is discarded for reasoning.

### Step 5: Realization back to Hebrew (if responding in Hebrew)

```
Take: #VEHICLE_POWERED_BY_ENGINE ⊗ #COLOR_RED
      │
      ├─ Pick lemma in target language: lemma:מכונית (he)
      │    lemma has gender=Feminine
      │
      ├─ Pick lemma for modifier: lemma:אדום (he)
      │    lemma requires agreement with head
      │
      ├─ Apply agreement rules:
      │    Head is Feminine Singular Indefinite
      │    → Modifier: אדום + ה (fem sg suffix) = אדומה
      │    → Head: מכונית (no change, base form)
      │
      └─ Order: Hebrew adjective follows noun → "מכונית אדומה"
```

### Same concept, realization to other languages

```
#VEHICLE_POWERED_BY_ENGINE ⊗ #COLOR_RED

Hebrew:    "מכונית אדומה"   (noun-adj order, fem-fem agreement)
English:   "red car"        (adj-noun order, no agreement)
Spanish:   "coche rojo"     (noun-adj order, masc-masc — coche is masc)
             OR "carro rojo" (regional variant)
French:    "voiture rouge"  (noun-adj order, fem but rouge invariant)
Arabic:    "سيارة حمراء"    (noun-adj order, fem-fem, plus taa marbuta)
German:    "rotes Auto"     (adj-noun order, neuter-neuter agreement)
```

**The concept is one. The realization changes per target language's rules.**

---

## Worked Example 3 — Synonyms with Different Breadth

### "אוטו" / "מכונית" / "רכב"

These are not identical. They have different **breadth** and **register**.

```
Concept hierarchy:
                        #VEHICLE (broadest)
                            ↑
                    #GROUND_VEHICLE
                            ↑
                    #MOTOR_VEHICLE
                            ↑
              ┌─────────────┴─────────────┐
       #PASSENGER_CAR              #COMMERCIAL_VEHICLE
       (automobile for people)    (truck, van)

Sense mapping (Hebrew):
  sense:vehicle.general        → #VEHICLE        ← lemma:רכב (formal, broad)
  sense:car.automobile         → #PASSENGER_CAR  ← lemma:מכונית (neutral)
  sense:car.automobile         → #PASSENGER_CAR  ← lemma:אוטו (informal)

Relations:
  #PASSENGER_CAR --is_narrower_than--> #MOTOR_VEHICLE --is_narrower_than--> #VEHICLE
```

### What this gives us

- "מכונית אדומה" = "אוטו אדום" (both express passenger_car + red)
  - But אוטו is masculine, מכונית is feminine → adjective form changes
- "רכב אדום" is **broader** — could be a truck, a motorcycle, any vehicle
  - Still red, still valid, but less specific
- If user says "אני מחפש רכב" — search should return ALL vehicle types
- If user says "אני מחפש מכונית" — search should prefer passenger cars

**The graph captures this via concept hierarchy, not word lists.**

### Register (formality level)

```
lemma:רכב     → sense:vehicle.general   [register=Formal, Written]
lemma:מכונית  → sense:car.automobile    [register=Neutral]
lemma:אוטו    → sense:car.automobile    [register=Informal, Colloquial]
```

When ZETS responds, it picks the lemma whose register matches the conversation style.

---

## Worked Example 4 — Hebrew-Specific Features

### ה הידיעה (definite article)

Not a word. A **prefix** with a feature.

```
MorphologyRule:
  prefix "ה"
  family=DefiniteArticle
  priority=90 (strong signal)
  features=[DefiniteArticle]
  min_stem_chars=3

Applied to "תפוח":
  surface:התפוח
    → strip prefix ה (DefiniteArticle)
    → lemma:תפוח
    → wordform features: [Definite, Masculine, Singular]
```

**Graph representation:**
```
wordform:התפוח
  ├── lemma     → lemma:תפוח
  ├── features  → [Masculine, Singular, Definite]
  └── source    → observed_in_corpus / generated_via_inflection
```

The Definite feature becomes **part of the phrase atom** when composed:
```
"התפוח האדום" → PhraseAtom with definiteness=Definite (on BOTH head and modifier)
```

Hebrew has **definiteness spreading**: in "התפוח האדום" the adjective
also takes ה. In "תפוח אדום" neither takes ה. The agreement rule is:
> Adjective matches noun in gender, number, AND definiteness.

This is an **agreement rule atom** in the graph:
```
rule:hebrew_adj_noun_agreement
  applies_to: [Noun + Adjective] phrases
  must_match: [gender, number, definiteness]
  realization: suffix on adjective based on noun's features
```

### ת בסוף לנקבה (feminine ending)

Morphology handles this:
```
SuffixRule:
  suffix "ת" / "ה" / "ית"
  features=[Feminine, Singular]

Applied to lemma:טוב + feature=Feminine:
  → wordform:טובה
  
Applied to lemma:ילד + feature=Feminine:
  → wordform:ילדה

Applied to lemma:מלך + feature=Feminine:
  → wordform:מלכה
```

The **rule lives in morphology** (code + tables). The **feature lives in the graph**.

### אחד עשר / אחת עשרה (gendered numbers 11-19)

```
Concept:       #NUMBER_11
                  │
                  ├── expressed_by (if head is masculine):
                  │     phrase-lemma:אחד_עשר (Hebrew, masc)
                  │       → wordform:"אחד עשר"
                  │
                  ├── expressed_by (if head is feminine):
                  │     phrase-lemma:אחת_עשרה (Hebrew, fem)
                  │       → wordform:"אחת עשרה"
                  │
                  ├── expressed_by (no gender, English):
                  │     lemma:eleven (en)
                  │
                  └── expressed_by (gendered, Spanish):
                        lemma:once (es)  [invariant — Spanish loses this in teens]
```

**The number CONCEPT is one atom.** The gendered forms are two phrase-lemmas
in Hebrew. Agreement rule selects the right one based on the counted noun.

Example:
- "11 boys" → #NUMBER_11 + #CHILD[masc,pl] → "אחד עשר ילדים"
- "11 girls" → #NUMBER_11 + #CHILD[fem,pl] → "אחת עשרה ילדות"

### זמנים (past / present / future)

Hebrew verbs change more than English ones. The graph handles this via Lemma + Features:

```
Concept:       #ACTION_WALK
                  │
                  └── expressed_by lemma:ללכת (he) [verb]
                           │
                           ├── inflects_to wordform:הלכתי
                           │     features=[Past, 1sg, common_gender]
                           │
                           ├── inflects_to wordform:הולכת
                           │     features=[Present, sg, Feminine]
                           │
                           ├── inflects_to wordform:אלך
                           │     features=[Future, 1sg, common_gender]
                           │
                           └── ... (dozens more)
```

The morphology module generates/parses these using:
- **Binyan templates** (root + pattern → stem)
- **Suffix tables** (stem + person/number/gender → wordform)

For Hebrew the root is 3-letter (ה.ל.כ). For Arabic the root is 3 or 4.
The "binyan" concept applies to both — it's the **Semitic typology pattern**
already recognized in `families.rs::semitic()`.

---

## Worked Example 5 — Cross-Language Cases Where Words Don't Align

Not every concept has a single word in every language.

### German compounds

```
Concept: #HIGHWAY_SPEED_LIMIT

German:     "Autobahngeschwindigkeitsbegrenzung" (ONE word, compound)
Hebrew:     "הגבלת מהירות בכביש מהיר"           (phrase, 4 words)
English:    "highway speed limit"                (phrase, 3 words)
Hungarian:  "autópálya sebességkorlát"           (phrase, 2 words)
```

**Graph:**
- The concept is one atom.
- Each language has its own surface representation — may be ONE wordform (German compound), TWO-word phrase-lemma, or a three-word phrase-lemma.
- The `expressed_by` edge can point to either a single lemma OR a phrase-lemma atom.

### English terms with no single Hebrew equivalent

```
Concept: #SERENDIPITY (pleasant unexpected discovery)

English:    lemma:serendipity                    (one word)
Hebrew:     phrase-lemma:"גילוי מקרי נעים"       (three words, descriptive)
Arabic:     phrase-lemma:"الصدفة السعيدة"         (happy coincidence)
```

### Idioms — meaning NOT compositional

```
Concept: #EVERYTHING_WENT_WRONG

English:   idiom:"the wheels fell off"
Hebrew:    idiom:"יצא מהכלים" / "נפל האסימון"
Arabic:    idiom:"خاب الظن"

→ Cannot decompose word-by-word.
→ Store as single idiom-lemma pointing directly to the concept.
→ expressed_by edge has flag: is_idiomatic=true
```

The graph doesn't try to reason about "why the wheels fell off" in English —
it treats the phrase as an atomic idiom-lemma bound to the concept.

---

## The Deep Question — "Do We Think In One Language Or All?"

Cognitive science is divided. Sapir-Whorf says language shapes thought.
Universalists say thought is prior to language.

**ZETS's position (architectural, not philosophical):**

> **Thought happens at the Concept layer — language-agnostic.
> Language is I/O: encoding input, decoding output.**

### Evidence from how ZETS reasons

When ZETS processes "מכונית אדומה":

1. **Parse** → WordForm → Lemma → Sense (language-specific cleanup)
2. **Lift to Concept** → `#VEHICLE ⊗ HAS_COLOR ⊗ #RED`
3. **Reason** → graph walks on Concept layer only
4. **Realize** → pick target language, generate WordForms

The reasoning step has NO Hebrew in it. No English. No grammar.
Pure concept relationships.

### Why this matters

- **Translation is a special case of realization.** Same reasoning, different
  target language. No "translation" step — just re-realize the same concept.
- **Code-switching** (Hebrew-English mixing, as Idan often writes) is just
  realizing some concepts in Hebrew and others in English, all from the same
  concept graph.
- **Bilinguals who "think without language"** match this model: they experience
  concept activation before realization chooses a language.
- **Aphasia patients** who lose language but retain reasoning match this model:
  their Layer 4 is intact, Layer 1-3 damaged.

### But — language DOES influence thought

The architecture allows for Sapir-Whorf-like effects:

- If a language has finer distinctions (Russian has two words for blue,
  светлый/тёмный), the language's LEMMA inventory drives SENSE granularity,
  which drives CONCEPT granularity.
- A speaker of Russian may have `#COLOR_LIGHT_BLUE` and `#COLOR_DARK_BLUE` as
  separate concepts, while English speakers have only `#COLOR_BLUE`.
- This is captured in the graph as different concept structures per speaker/culture.
- It's a SOFT effect — not "impossible to think about" but "less readily activated".

### Architectural consequence

**Thought is universal. Expression is local. Graph captures both.**

---

## How Agreement Is Implemented

### Storage of grammatical features

Lemmas carry **intrinsic** features:
```
lemma:מכונית  [gender=Feminine (intrinsic — the noun IS feminine)]
lemma:תפוח    [gender=Masculine (intrinsic)]
lemma:שולחן   [gender=Masculine (intrinsic)]
```

Adjectives carry **agreement-template** features:
```
lemma:אדום    [gender=AgreesWithHead, type=Adjective]
                (must inflect based on the noun it modifies)
```

### Agreement rule as first-class atom

Hebrew agreement is an atom in the graph:
```
rule:hebrew_adj_noun_agreement
  type:           AgreementRule
  language:       he
  pattern:        [Noun] + [Adjective]
  constraints:    adjective.gender = noun.gender
                  adjective.number = noun.number
                  adjective.definiteness = noun.definiteness
  applied_by:     TextExecutor during realization
```

At realization time the TextExecutor:
1. Takes the PhraseAtom with head + modifier
2. Looks up the language's agreement rules
3. Applies inflection to the modifier to match the head
4. Emits the surface form

### Why this is in the graph (not hardcoded in Rust)

Per core principle ("Learning is in code, what/how is in graph"):
- The MECHANISM of applying a suffix is in Rust (morphology module)
- The RULE ("Hebrew adjectives agree in gender+number+definiteness") is a graph atom
- ZETS can LEARN rules by observing corpus — noticing "אדום always follows masculine nouns"
- Rules can be edited, versioned, have confidence scores
- Different dialects can have different rules (archaic vs modern agreement)

---

## Relations Needed (new or existing)

### Existing in relations.rs / sense_graph.rs
- `expresses_sense` (word → sense)
- `in_language` (word → language)
- `broader_than` (sense → sense)

### New relations required by this ADR

```
realizes_concept       (sense → concept)
has_lemma              (wordform → lemma)
inflects_to            (lemma → wordform, with features edge-attribute)
has_gender_intrinsic   (lemma → [Masculine/Feminine/Neuter])
agrees_with            (modifier-lemma → head via pattern)
is_narrower_than       (concept → concept, for hierarchy)
part_of_phrase         (lemma → phrase-lemma)
is_idiomatic           (phrase-lemma flag)
has_register           (lemma → [Formal/Neutral/Informal/Literary])
```

---

## Storage Efficiency

### Per the Sigil-Executor ADR

- **Concept atoms**: 8 bytes (pure semantics, no data)
- **Sense atoms**: 8 bytes + small inline data (gloss stored via DocExecutor)
- **Lemma atoms**: 8 bytes + features (inline) + language ID
- **WordForm atoms**: 8 bytes + surface-form pointer + features inline

For a Hebrew lemma with 20 wordforms:
- 1 lemma atom: 8 bytes
- 20 wordform atoms: 20 × 8 = 160 bytes
- 20 inflects_to edges: 20 × ~6 bytes = 120 bytes
- Surface strings in blob store: ~200 bytes total (8-10 chars each)

**~500 bytes per lemma including all inflections.** 20K Hebrew lemmas → 10 MB.

### Why this is efficient

- Grammar rules are stored ONCE per language (~50-200 rule atoms)
- Not once per lemma
- Not once per wordform
- Not once per phrase
- Inflection GENERATES wordforms on demand — don't even need to persist all of them

---

## Open Questions (for later ADRs)

1. **When to materialize wordforms vs generate on-demand?**
   Frequent forms persisted, rare forms generated lazily?

2. **How to handle ambiguous parses?**
   "הוא ראה את האיש הקטן" — "small" modifies "man" or is it "the small" (NP)?
   Need disambiguation via sense + context.

3. **Phrase-lemma boundary — when does a phrase become a lemma?**
   "מכונית אדומה" = compositional (two lemmas).
   "רוח רפאים" (ghost) = idiomatic (one phrase-lemma).
   Threshold? Co-occurrence frequency + unpredictable meaning?

4. **Historical/archaic forms** — how to flag?
   Biblical Hebrew "ויאמר" has archaic agreement. Tag on wordform? lemma?

5. **Creole/mixed forms** — "עברית היברידית"
   "התוכנית פעלה great" — mixing inside phrase.
   How to parse+realize code-switched text?

6. **Sign languages** — visual-gestural, not sequential.
   Can the same framework handle ISL/ASL signing as "wordforms"?

---

## What This ADR Supersedes

- Any implicit "word = concept" assumption
- Any approach storing grammar as separate per-lemma data
- Any cross-lingual SAME_AS direct word-to-word edges
- Prior sense_graph.rs design is COMPATIBLE and EXTENDED by this ADR

---

## Signed

**Architect:** Idan Eldad (עידן אלדד)
**Scribe:** Claude 4.7
**Date:** 2026-04-24

Builds on: ADR "Atom as Sigil, Executor as Doer" (same day)
Implemented partially in: src/sense_graph.rs, src/morphology/
Needs implementation of: Concept layer atoms, phrase-lemma support, agreement rule atoms

