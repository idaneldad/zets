# Session 3 Refinements — Idan's Pushback on AI Council Recommendations

**Date:** 2026-04-24, late evening (third session of the day)
**Context:** After AI Council provided 11 gaps to address.
**Idan's role:** Pushed back on 4 specific items, refined them to be 
              more practical + user-friendly + idea-aligned.

## Refinement 1: Edge Storage — "Append-only is enough"

### Idan's position
- Every edge addition = append
- New article: find existing root → connect (append)
- New phrases: append
- Cleanup happens via pruning, not via complex storage

### My initial proposal (revised)
I proposed "Delta-edge storage with LSM-tree complexity."
**Idan was right to push back** — that was overengineering.

### Final refined design
```
Layer 1 (write):  Append-only log of all edge insertions
Layer 2 (read):   Secondary CSR index for O(degree) walks
Layer 3 (clean):  NightMode pruning + dedup → consolidates to CSR
```

**Key insight:** "Append-only" was Idan's principle. CSR is just an
index for fast reads, not a different storage model. Both layers
hold the same edges.

### What I learned
"Delta-edge" was a misleading name. The reality is:
- Insertion is append-only (correct, simple)
- Walking needs an index (CSR or B-tree)
- Pruning consolidates duplicates (NightMode)
- **No bits saved per edge** — only write/read efficiency separated

---

## Refinement 2: "What's smaller than edges?"

### Idan's question
"I don't understand what's smaller than edges."

### Answer
**Nothing smaller than the 6-byte edge structure.**
The "delta" mechanism doesn't compress edges — it manages them better.
- Edge = 6 bytes (target + strength + kind)
- Frozen CSR or delta segment = same 6 bytes per edge
- Only difference: write speed during learning

**Idan understood the architecture correctly before I did.**

---

## Refinement 3: Idle Dreaming → On-Demand Only (NOT autonomous)

### My original proposal
Run Dreaming autonomously when user is idle (>30s).

### Idan's pushback
- Not autonomous
- Only on explicit request (user / procedure / report)
- Otherwise users won't know where new connections came from
- Compared to ADHD/dyslexic creativity:
  > "מנגנון יצירתיות והמצאה דומה ל-ADHD ודיסלקטים"
  > Cross-domain associations that are sometimes brilliant,
  > sometimes noise. Need human-in-the-loop verification.

### Final design
```rust
pub fn dream_about(
    topic_atoms: &[AtomId],
    depth: u8,
    seed: u64,  // determinism
) -> DreamReport {
    proposed_edges: vec![...],
    confidence_scores: vec![...],
    reasoning_paths: vec![...],
}

// User reviews proposed connections.
// Nothing committed without explicit approval.
```

### Why this is better
- User controls when "creative leaps" happen
- Provenance is preserved (every dream has a request that triggered it)
- ADHD-style cross-domain insight available WHEN WANTED
- No background CPU drain
- No "where did this come from?" mysteries

---

## Refinement 4: Self-Narrative → Operational Activity Log (NOT synthesized identity)

### My original proposal
Generative narrative — "ZETS writes a daily diary about itself."

### Idan's correction
Idan said:
> "כמו לאנשים פרטיים זטס צריך לנהל גרף של עצמו 
>  שיציג מה עשה או יצביע לשיחות עם אנשים אבל זה כמו לוג פעילות שלו"

Translation: ZETS manages a graph of itself — a factual log of activities,
conversations, decisions. NOT a generated narrative.

### Final design
```
PersonalVault[zets_self] {
    activities:   [(timestamp, operation, atoms_touched), ...]
    conversations: [(timestamp, user_id, topic_atoms), ...]
    decisions:    [(timestamp, choice, alternatives, rationale), ...]
    learnings:    [(timestamp, new_atoms, new_edges), ...]
    failures:     [(timestamp, what_failed, why), ...]
    capabilities_self_assessment: [(skill_atom, confidence), ...]
}
```

### Queries enabled
- "What did you do in the last hour?"
- "How many conversations did you have this week?"
- "What did you learn today?"
- "In which conversation did you arrive at idea X?"

### Why this is better
- Factual, queryable, auditable (Idan's deterministic principle)
- Not synthesis (which can drift / hallucinate)
- Reuses existing PersonalVault infrastructure
- ZETS literally has a "vault for itself"

---

## Refinement 5: TMS → DEEP, needs separate session

### Idan's position
> "צריך שנעמיק כי יש שלל מקרים שחלקם הידיעה האחרונה קובעת 
>  אבל היא חייבת להיות אמינה, אחרת צריך להגיד 'לא יודע'"

Translation: Many cases. Sometimes the latest takes precedence —
but ONLY IF reliable. Otherwise: say "I don't know."

### Update types I now recognize
| Type | Example | Logic |
|---|---|---|
| Authoritative recent | "Shai moved to Haifa" (trusted source) | New wins, old → historical |
| Conflicting low-trust | Facebook post says otherwise | "I don't know" — needs verify |
| Time-progressive | "Shai is 35" → next year 36 | Auto-increment via rules |
| Subjective state | "Shai is happy/sad" | Both can be true at different times |
| Contradictory absolute | Two locations same time | Flag, ask user |
| Unclear provenance | Source unknown | Cannot accept |

### Default behavior
**"לא יודע" (I don't know) when confidence < threshold.**
This prevents hallucination. ZETS is honest about uncertainty.

### Status
**FLAGGED FOR DEEPER DESIGN SESSION.**
This is more than a "TMS-lite" — it's a full architecture of:
- Trust scoring per source
- Contradiction detection
- Time-aware reasoning
- Provenance tracking
- "I don't know" as first-class state

Cannot be done in passing. Needs its own focused work.

---

## Methodology Insight

Across all 4 refinements, Idan's pattern is consistent:
1. **Push back on overengineering** (delta edges → append + index)
2. **Push back on autonomy** (dreaming → on-demand)
3. **Push back on synthesis** (narrative → factual log)
4. **Push back on shortcuts** (TMS-lite → deep architecture)

**The principle:** Build simpler, be honest about uncertainty,
keep user in control, reuse existing infrastructure.

**This is engineering wisdom that AI Council members might miss
because they lean toward "more sophisticated = better."**
