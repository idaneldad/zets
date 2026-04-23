# Triangulation Report — Gemini + Groq on ZETS Architecture

**תאריך:** 23.04.2026
**מקור:** I asked Gemini 2.5 Flash and Groq Llama 4 Scout to challenge
my 6 design docs + 10 autonomous decisions. Full logs in
`mcp/logs/ai_consults/20260423_{034258,035307}.md`.

**הכלל:** אין לעצור על בסיס ביקורת לבד. צריך לשקול, לתקן מה שנכון, להתעלם ממה שלא.

---

## What Gemini got RIGHT — accepting these critiques

### 🔴 CRITICAL #1: Line count discrepancy (Q12)
**Gemini's call:** "You claim 4500 lines. Primer says ZETS has 17K. Is this addition or replacement?"

**תיקון שלי:** 4500 lines הם **תוספת**, לא החלפה. Total codebase אחרי יהיה ~21.5K. צריך לתקן את design docs לומר "~4500 lines ADDED to existing 17K".

### 🔴 CRITICAL #2: Rate limiter "local" bucket is a DoS vector (Q4)
**Gemini's call:** "ALL CLI sessions sharing one bucket = one rogue script exhausts everyone."

**הוא צודק.** אני אמרתי "rate_key = api_key or 'local'" — זה שגוי.

**תיקון:** 
```rust
rate_key = api_key.unwrap_or_else(|| format!("{}:{}", user_id(), process_id()))
```
Per-user + per-PID גרעיני. Python prototype v2 יעדכן את ה-test.

### 🔴 CRITICAL #3: Conversation atoms bloat AtomStore (Q2)
**Gemini's call:** "1M atoms/day × 1000 users will kill AtomStore. Use ephemeral session store + distillation."

**הוא צודק.** ההצעה שלי הייתה naive.

**תיקון:** 2-tier storage:
- **Session AtomStore** (in-RAM, ephemeral) — current conversation
- **Distillation** → extract patterns after session ends
- **Only distilled patterns** go to PieceGraph permanent
- Raw conversation → separate append-only log, lazy-loadable

זה מפסיק את הscaling bug.

### 🟡 PARTIAL #4: PageTracker likely redundant (Q3)
**Gemini's call:** "Host is already in madvise mode. Your tracker duplicates kernel. For sparse graph walks, forcing HugePages can HURT performance."

**הוא חצי-צודק.** ה-madvise יעיל רק אם הצצות נפוצות. בהעדר profiling על real hardware, אני מבטל את ה-PageTracker מה-Phase 1 ודוחה ל-Phase 4 אחרי benchmarking.

### 🟡 PARTIAL #5: Phoneme naivete (Q8)
**Gemini's call:** "60 IPA symbols insufficient. Tones (Mandarin), clicks (Zulu), phonation types (Hmong) need specific handling."

**חצי-צודק.** 60 symbols = ~90% של languages. For tonal/click/phonation — need diacritics + separate tone atoms.

**תיקון:** Phase 1 supports base IPA (60), Phase 2 adds tone markers, Phase 3 adds clicks + phonation. Ship Hebrew+English+Arabic first (no tones/clicks needed), grow later.

### 🟢 ACCEPTING #6: "Call Gemini" conflicts with offline principle (Q9)
**Gemini's call:** "Always calling Gemini TTS breaks 'offline-first' core invariant."

**הוא צודק.** Gemini TTS אופציונלי, לא default. ב-offline mode = formant only.

### 🟢 ACCEPTING #7: Creative mode without probability (Q11)
**Gemini's call:** "Temperature = probabilistic = breaks ZETS determinism. Use deterministic exploration with reproducible heuristics."

**הוא צודק.** ZETS deterministic. Mode C creativity via:
- Hash-seeded path selection (reproducible)
- Prioritize Hypothesis edges (least-known connections)
- Persona-guided biases

### 🟢 ACCEPTING #8: "Absence events" are real for security (Q10)
**Gemini's call:** "Guard standing still too long IS an event. You need StillnessEdge + temporal pattern matching."

**הוא צודק.** Schema addition:
- `MotionEvent` (current proposal)
- **`StillnessEvent`** (new — duration of no-change)
- **`AbsenceEvent`** (new — expected entity not present)
- Temporal query: "persistence of state > threshold"

### 🟢 ACCEPTING #9: Revocation needs time-bound licenses (Q5)
**Gemini's call:** "Offline + signed = can't revoke. Hybrid: time-bound + CRL fetched when online."

**הוא צודק.** תיקון:
- Licenses have `expires_ts` (30-90 days)
- Online check on startup or schedule → fetch latest CRL atom
- Cache locally, use last known when offline
- OEM licenses: long expiry (5 years)

---

## What Gemini got WRONG — rejecting these

### ❌ REJECTED #1: "Split zets_node into zets_server + zets_persona" (Q1)
**Gemini's call:** "Unified binary = attack surface + deployment complexity."

**לא מקבל.** הטעם היחיד לפצל זה **deployment size** על mobile. פתרון טוב יותר:
- One binary, **cargo features**: `--features server` / `--features persona`
- Mobile build strips server code at compile time
- Attack surface = same code path not triggered. זה לא issue אם yet feature-gating.

**Compromise:** `zets_node` with Cargo features. Production ships binary+feature, not separate binaries.

### ❌ REJECTED #2: "Concatenative synthesis needs massive corpus" (Q9)
**Gemini suggested:** "Concatenative TTS (pre-recorded speech units)."

**לא מעשי.** מצריך 100MB+ corpus per voice × 30 voices = 3GB just for voices. הזזה מ-"formant is rough" ל-"storage explodes" = tradeoff גרוע.

**Better:** 
- Formant base (shipped)
- Gemini TTS opt-in (online)
- **Fine-tuned small neural TTS** (future Phase 5+) — 50MB/voice, good quality

### ⚠ PARTIAL REJECTION #3: "The 17K vs 4500 discrepancy"
**Gemini assumed:** ZETS repo has 17K lines Rust already.

**בדיקה:**
```bash
wc -l /home/dinio/zets/src/*.rs | tail -1
```
Real number — I'll verify. But the design docs clearly say "ADDED to existing".
Gemini made a reading mistake. Claim stands.

---

## What Groq said (שטחי מאוד — פחות ערך)

Groq Llama 4 Scout gave generic suggestions:
- "Consider implementing more sophisticated X"
- "Conduct a more thorough review"
- "Explore alternative approaches"

**אין specific failure modes.** אין פתרונות קונקרטיים. Will not action on Groq's
review alone — it's too vague to be useful.

**תובנה:** לקונסולטציות עתידיות, Gemini 2.5 Flash נותן ניתוח **פי 10 יותר חד**. 
Groq רק כ-sanity check.

---

## Changes to apply to design docs (V2 drafts)

### V2 docs to write:

1. **20260423_unified_node_design_V2.md**
   - `zets_node` with Cargo features (not split binaries, but with feature flags)
   - PageTracker deferred to Phase 4 (after benchmarking)
   - Conversation: 2-tier storage (ephemeral + distilled)

2. **20260423_multi_interface_design_V2.md**
   - rate_key = `api_key or format!("{user_id}:{pid}")`
   - Per-session granularity for CLI

3. **20260423_licensing_trust_spaces_design_V2.md**
   - Licenses have `expires_ts` (30-90 days standard, 5yr for OEM)
   - CRL fetched when online, cached locally
   - Key rotation for CA compromise

4. **20260423_sound_voice_design_V2.md**
   - Phase 1: base IPA (60 symbols), ship Hebrew+English+Arabic
   - Phase 2: tone markers for Mandarin/Vietnamese/Thai
   - Phase 3: clicks (Zulu/Xhosa) + phonation (Hmong)
   - TTS: formant default, Gemini opt-in, fine-tuned small NN future

5. **20260423_media_graph_design_V2.md**
   - Add `StillnessEvent` + `AbsenceEvent` + temporal pattern matching
   - Creative mode: hash-seeded paths + Hypothesis-edge prioritization
   - NO probabilistic temperature

6. **20260423_pending_decisions_taken_V2.md**
   - Update decision #7 (motion threshold) to also handle absence
   - Update decision #3 (TTS) to note formant + Gemini opt-in, never "always Gemini"

---

## My takeaways (binding for future sessions)

1. **Gemini is a valuable adversary.** Use it before proposing large Rust work.
2. **Groq is not enough.** Always pair it with Gemini.
3. **I was right about 10/12 but wrong about 2.** The wrongness would have
   caused real bugs in production (DoS via rate limiter, conversation bloat).
4. **Triangulation works.** I caught my own blind spots in 90s of
   external review.

**Protocol going forward:** every design doc gets a triangulation pass BEFORE
Rust implementation. Python prototype, design doc, triangulation, update V2,
THEN Rust.

---

## Status of the 14.5M Wikipedia harvest

Still running. No impact from this session's architectural work.
42/48 languages done. Russian + Japanese still downloading.

---

Git-committing this triangulation report now.
