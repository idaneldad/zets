# Phase B + C Planning — ZETS Capability Measurement + Tool Ingest

**Date:** 2026-04-23
**Status:** Planning — execution in future sessions
**Inputs:** Idan's Q4 2026 request for benchmark + OpenClaw absorption

---

## The 2 big open questions

Idan asked (translated):

> "How do we know what level of ZETS is? Information coverage, memory,
> clients — but more importantly: simulated conversations + QA + many
> tests to understand how capable it is, and how human it feels.
>
> Then: take tools from OpenClaw GitHub and install them efficiently,
> preserving what's written so nothing is written twice. Also 'cheap'
> tools of appropriate size from open GitHub.
>
> Need a smart pruning process that connects things and cleans excess.
> Major part: find repetition and consolidate into sentences and
> procedures, marking the source additionally on the new, marking
> redundant for deletion — then in a subsequent round, if it's still
> not relevant, delete it."

## Why they are planned separately

Both are 800+ line efforts. Doing them in one session = shallow quality.
Phase A (PersonalGraph) is prerequisite for both: we need identities +
profiles before we can simulate conversations meaningfully (B), and we
need scoped procedure visibility before we can ingest external tools (C).

---

# Phase B — ZETS Capability Benchmark

## The right metrics (not just test count)

Raw test count (829 passing) is a HEALTH metric, not a CAPABILITY metric.
What we actually need:

### B1. Knowledge coverage
> "How much does ZETS know, compared to a baseline?"

- **Breadth**: concept atoms across 48 languages, 17.5M articles
- **Depth**: average edges-per-concept (richer = smarter)
- **Accuracy**: spot-check via QA set

**Measurement plan:**
```rust
pub struct CoverageReport {
    total_concepts: usize,
    concepts_per_language: HashMap<Lang, usize>,
    avg_edges_per_concept: f32,
    qa_accuracy: f32,           // from held-out QA set
    top_topic_gaps: Vec<String>,  // areas where ZETS is weak
}
```

### B2. Memory quality
> "Does ZETS remember what matters, and forget what doesn't?"

- **Retention accuracy**: after 100 interactions, can ZETS recall
  facts from interaction #3?
- **Consolidation rate**: how many raw interactions → stable memory atoms?
- **Drift detection**: does ZETS's understanding of a client evolve or stay static?

### B3. Conversation quality (the humanness test)
> "Does ZETS feel like talking to a thoughtful person?"

This is the hard one. Three proposed tests:

**Test 3a: Simulated conversations**

Build a test harness where an LLM plays "user" with a defined persona:
```rust
pub struct SimulatedUser {
    pub persona: PersonaSpec,       // age, profession, mood, Big Five
    pub goals: Vec<String>,         // what they want from conversation
    pub communication_style: Style, // formal/casual, length, directness
}

pub struct ConversationRun {
    pub user: SimulatedUser,
    pub turns: Vec<Turn>,
    pub duration_ms: i64,
    pub user_satisfaction: f32,     // rated by a separate LLM judge
    pub naturalness: f32,           // rated by a separate LLM judge
}
```

Run 100 conversations. Judge each on 4 axes:
- Understood the user's intent
- Responded in matching tone
- Provided useful information
- Felt human (not robotic)

**Test 3b: Blind comparison**

Show judges responses from (ZETS, generic LLM, human expert) without
labels. Measure how often ZETS is identified as "the most human" or
"the best fit for the question."

**Test 3c: Longitudinal memory test**

- Conversation 1: tell ZETS 5 facts about yourself
- Come back 1 day later. Ask 3 questions that require those facts.
- Score correctness + naturalness of reference.

### B4. Response appropriateness (Reader::check_output gate)

The gate we designed for Phase 1 is the metric:
- Rate of `Pass` (confident direct answer)
- Rate of `Assisted` (answer with caveats)
- Rate of `Escalate` (deliberated longer)
- Rate of `Hold` (refused/redirected)

Healthy system: distribution should match input difficulty — easy
queries should be mostly `Pass`, hard/ambiguous should be `Assisted` or
`Escalate`. All-`Pass` = overconfident. All-`Hold` = paralyzed.

## Implementation plan

```
src/benchmark/
    mod.rs              ← orchestrator
    coverage.rs         ← B1 — corpus coverage tests
    memory.rs           ← B2 — retention + consolidation metrics
    conversation.rs     ← B3a — simulated conversation harness
    comparison.rs       ← B3b — blind A/B/C scoring
    longitudinal.rs     ← B3c — multi-session memory tests
    gate_distribution.rs ← B4 — gate decision histogram
    report.rs           ← aggregate HumannessScore (0..1)
```

**HumannessScore** = weighted combination:
- Knowledge coverage: 20%
- Memory quality: 20%
- Conversation naturalness (avg over 100 simulated): 40%
- Gate distribution health: 10%
- Blind comparison win rate: 10%

Target after full build-out: **0.75+**. Current (stub Reader): probably 0.3.

---

# Phase C — OpenClaw + GitHub Tool Ingest

## The 5-stage pipeline (from practical_lessons_v2.md, now actionable)

```
External GitHub repo
       ↓
┌─────────────────┐
│ STAGE 1: FETCH  │  clone repo, shallow, read-only
└────────┬────────┘
         ↓
┌─────────────────┐
│ STAGE 2: PARSE  │  AST walk via tree-sitter
│                 │  extract function signatures, docstrings, tests
│                 │  DO NOT keep source code
└────────┬────────┘
         ↓
┌─────────────────┐
│ STAGE 3: DEDUP  │  4-layer cascade:
│                 │    1. Semantic embedding (cosine > 0.92)
│                 │    2. Behavioral test (same I/O)
│                 │    3. Sense-key match
│                 │    4. Hash (identity check)
│                 │  If match → skip. If near-match → variant edge.
└────────┬────────┘
         ↓
┌─────────────────┐
│ STAGE 4: RESYNTH│  LLM writes FRESH code from canonical spec
│                 │  Input: behavior description + existing ZETS atoms
│                 │  Output: procedure_atom in ZETS format
│                 │  NEVER sees external source code
└────────┬────────┘
         ↓
┌─────────────────┐
│ STAGE 5: VERIFY │  Run in sandbox with extracted test cases
│                 │  If pass → register as TrustLevel::Learned
│                 │  If fail → log, retry with different prompt
└─────────────────┘
```

## The pruning cycle (Idan's specific request)

> "Smart process that does pruning, connects things, cleans excess.
> Major: find repetition and consolidate. Mark source additionally.
> Mark redundant for deletion — subsequent round, if still irrelevant,
> delete."

**4-round consolidation cycle:**

```
ROUND 1: DETECT
  Scan all procedures + concepts
  For each pair with similarity > 0.85:
    Create 'similar_to' edge
    Score: which is more general / more tested / more used?

ROUND 2: CONSOLIDATE
  For each similar_to cluster:
    Pick the 'winner' (highest score)
    Add 'supersedes' edge from loser to winner
    Transfer the loser's 'source' edges to winner
    Mark loser with status='pending_deletion', tombstone_round=current

ROUND 3: GRACE PERIOD (N days later)
  Check each pending_deletion:
    Was it referenced by new procedures in the interim?
      YES → restore, remove pending_deletion flag
      NO  → advance tombstone_round

ROUND 4: PRUNE (M days after initial mark)
  For each pending_deletion with tombstone_round + M < current:
    Move from active graph to archive
    Keep in archive for forensic queries
    Never truly delete
```

## Pseudocode structure

```
src/ingestion/
    mod.rs                    ← orchestrator
    fetch.rs                  ← Stage 1: git clone + file scan
    parse.rs                  ← Stage 2: tree-sitter AST → BehaviorSpec
    behavior_spec.rs          ← canonical representation
    dedup/
        mod.rs                ← 4-layer cascade coordinator
        embedding.rs          ← Layer 1
        behavioral.rs         ← Layer 2
        sense_key.rs          ← Layer 3
        hash.rs               ← Layer 4
    resynthesize.rs           ← Stage 4: LLM call with anti-copy constraint
    verify.rs                 ← Stage 5: sandbox run + test match

src/pruning/
    mod.rs                    ← cycle orchestrator
    detect.rs                 ← Round 1: similarity detection
    consolidate.rs            ← Round 2: winner selection + edge transfer
    grace.rs                  ← Round 3: grace period check
    prune.rs                  ← Round 4: archive
    archive.rs                ← forensic archive store
```

## Anti-copy enforcement

Critical rule: **the LLM never sees source code**. It sees only:
- Behavior description (what the function does)
- Input/output types
- Example test cases
- List of existing ZETS atoms it can compose

The resulting code is fresh. If it accidentally resembles the source,
our semantic similarity check catches it (cosine > 0.95 between input
AST embedding and output code embedding → reject, retry with different
prompt).

## Concrete first target: GreenAPI WhatsApp wrapper

Walkthrough as first ingest:

1. **Fetch** `greenapi-go/whatsapp-api-client-golang`
2. **Parse** the `SendMessage` function:
   ```go
   func (c *Client) SendMessage(chatId, message string) (*Response, error)
   ```
3. **Dedup** check:
   - Embedding: similar to existing `http_post` + `build_json_payload`?
     → yes, ZETS has those primitives. Don't create new.
   - Conclude: this is a COMPOSITION, not a new primitive.
4. **Resynth** prompt:
   ```
   Write a ZETS procedure `send_whatsapp_via_greenapi` that:
   Inputs: chat_id (string), message (string), api_token (secret)
   Outputs: message_id (string) or error
   Behavior: HTTP POST to https://api.green-api.com/waInstance{id}/sendMessage/{token}
   Body: JSON with chatId + message
   Compose from these existing ZETS atoms: http_post, build_json_payload, parse_json_field
   ```
5. **Verify**: run against a mock server, confirm 200 response + message_id returned.
6. **Register**: `TrustLevel::Learned`, composition_of: [http_post, build_json_payload, parse_json_field].

Storage cost: ~200 bytes (composition edges), not 5KB of new bytecode.

## Rate limits for healthy ingest

- No more than 10 procedures ingested per day (avoid flooding)
- 48-hour grace period for new ingests before marking others redundant
- Weekly consolidation round, monthly prune round
- Manual owner approval required for `TrustLevel::OwnerVerified` promotion

---

# Sequencing

**Next 2 weeks (when ready):**
- Week 1: Phase B skeleton — simulated conversation harness
- Week 2: Phase C stage 1-2 — fetch + parse pipeline

**Weeks 3-4:**
- Phase B full run — HumannessScore computation
- Phase C stages 3-5 + first ingest (GreenAPI)

**Week 5+:**
- Pruning cycle implementation
- Reader Phase 2 — 8 emotion signals, pragmatics, Big Five

---

# Why this plan is realistic

1. **Incremental**: each stage produces working code + tests, not just docs
2. **Graph-first**: procedures, profiles, capabilities all live as atoms
3. **Never-delete principle** applied consistently: archive, don't destroy
4. **Anti-copy architecturally enforced**: LLM doesn't see source
5. **Measurable**: HumannessScore gives us a number to drive

---

# What we still don't know (flag for Idan)

- **Who will run the benchmark judges?** (LLM-as-judge has known biases)
- **What's the initial corpus for Phase C?** (OpenClaw only? Add curated list?)
- **Rate of ingest per day** — 10 sounds right, need real data
- **Pruning thresholds** — 0.85 similarity is a guess, needs calibration
