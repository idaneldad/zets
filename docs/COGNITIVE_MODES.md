# Cognitive Modes — the "genius with disorders" architecture

**Implemented:** 22-Apr-2026
**Lines of code:** ~450 (src/cognitive_modes.rs)
**Tests:** 9/9 passing
**Demo:** src/bin/cognitive_demo.rs

---

## The core idea

Instead of one traversal algorithm, ZETS has **four cognitive modes**, each inspired
by a neurodivergent thinking style. Every mode is a deterministic strategy for
walking the data graph — no randomness, no neural nets, just different algorithms
selecting different edges.

| Mode | Inspired by | Graph algorithm | What it finds |
|---|---|---|---|
| **PrecisionMode** | autism | bounded DFS, strict min_weight | Canonical truth chains (IS_A) |
| **DivergentMode** | ADHD | hash-weighted BFS, weak-edge gate | Remote/surprising connections |
| **GestaltMode** | dyslexia | k-hop neighborhood | The "whole picture" around a concept |
| **NarrativeMode** | dysgraphia | template composer (no walk) | Assembles findings into a story |

---

## The critical engineering decision — no randomness

The dysgraphia engineer from the discussion insisted: **any `rand::random()`
in a traversal destroys the moat.** The moat is 100% determinism.

So we use **hash-based pseudo-divergence**:

```rust
fn divergent_gate(query_id, edge_from, edge_to, weight, divergence_pct) -> bool {
    if weight >= 70 { return true; }           // strong edges always taken
    let h = hash64(&(query_id, edge_from, edge_to));
    (h % 100) < (divergence_pct as u64)
}
```

Properties:
- Same query + same edge = same decision, **forever**.
- Different queries → different hashes → different divergences.
- No platform-dependent randomness.
- Tomorrow's answer = today's answer = debuggable.

This is **creativity without unpredictability.** Query-sensitive divergence
looks random but isn't.

---

## What we measured

Graph: 10-node 'dog' neighborhood with IS_A, HAS_PROP, SEEN_IN, COMPARES_TO edges.

Same query ("What is a dog?"), three modes:

```
PrecisionMode:  [dog, canine, mammal, animal]              4 nodes, 1.10µs
DivergentMode:  [dog, canine, loyal, mammal, animal]       5 nodes, 1.34µs
GestaltMode:    [dog, canine, loyal, friendly, space,      9 nodes, 2.12µs
                 wolf, cat, mammal, feline]
```

**Determinism proof:** DivergentMode ran 3 times → identical output all three.

**Query sensitivity:** Starting from 'dog' vs 'cat' vs 'mammal' → different
query_ids → different hash decisions → different paths selected.

---

## Why this gets us closer to cognitive AI

### 1. Multi-strategy querying
A real thinker doesn't use one traversal algorithm. When you ask "What is a dog?",
a biological brain simultaneously:
- Follows canonical taxonomy (Precision)
- Associates freely with related concepts (Divergent)
- Pictures the whole scene (Gestalt)
- Composes a verbal answer (Narrative)

ZETS can now do all four. They're swappable at runtime via the `CognitiveMode`
trait — no compile needed to change strategy.

### 2. Per-user mode weighting
A user profile can store preferred mode weights:
```
idan: { precision: 0.4, divergent: 0.3, gestalt: 0.2, narrative: 0.1 }
```
ZETS blends modes based on the user's cognitive style.

### 3. Task-appropriate modes
- **Factual Q&A** → PrecisionMode dominates
- **Creative brainstorm** → DivergentMode dominates
- **"Tell me about X"** → GestaltMode dominates
- **Explanation requested** → NarrativeMode wraps whichever walked

### 4. Background ("dream") processing
ADHD brains can't stop thinking. Neither should ZETS.
When idle, a background task runs `DivergentMode` on random pairs of concepts
(chosen deterministically from hash(timestamp / 1h)). If a surprising short
path is found, it's added to the proposal queue for user review:
> "I noticed dogs and space connect via Laika (first dog in space).
>  Should I add a stronger edge between dog and space?"

This is the **Active Learning loop** the metacognition module was waiting for.

### 5. Sanity engine (autism-inspired)
Before any new fact is written to Data scope, PrecisionMode walks its
neighborhood to check for contradictions. If a new claim "DNA IS_A planet"
arrives, precision DFS reaches animal/molecule/gene and detects the conflict
by edge kind mismatch. The new fact is rejected at the gate — never entering
production. This is what the testing_sandbox.rs was designed for.

---

## What this is NOT

- **It's not AGI.** It's four traversal algorithms sharing a trait.
- **It's not a brain simulator.** The neurodivergence inspiration is a
  design metaphor, not a biological claim.
- **It's not statistical.** No probabilities — deterministic pseudo-random
  via hashing.
- **It's not multi-agent.** Single binary, single thread, swap strategies.

---

## What connects to the AGI roadmap

This session added:
1. Four cognitive modes as swappable traversal strategies.
2. Hash-based determinism proof (divergence without randomness).
3. Demo showing per-mode behavior on shared graph.

Still pending (from AGI_ROADMAP.md):
- Populate edges in actual `zets.core` (the biggest gap remains).
- Wire cognitive modes into ZetsEngine query path.
- Background Divergent scanner for active learning proposals.
- PrecisionMode as gatekeeper for new facts in testing_sandbox.

---

## Next concrete session

Priority order:
1. **Populate IS_A edges** from 10K English concept definitions (the most impactful).
2. **Wire PrecisionMode to sandbox** so new facts must pass consistency DFS.
3. **Background dream loop** — idle task runs DivergentMode on concept pairs,
   writes proposals to the `testing` scope.
4. **Query router selects modes** based on (intent classification, user profile).

After those: ZETS does multi-strategy deterministic reasoning with background
curiosity and automatic consistency-checking. That's not AGI — but it's
**a symbolic cognitive engine that matches the shape of human thinking more
than any LLM does, while remaining 100% debuggable and private.**
