# Sprint H — Sessions + Context Disambiguation

**Estimated time:** 1 week
**Branch:** `sprint-h-sessions`
**Risk level:** Medium
**Dependencies:** Sprint B complete
**Blocks:** Sprint D (Cognitive Tree)

**Note:** This sprint was moved up from its original position (H after G)
because the Cognitive Tree (Sprint D) needs session context for proper
disambiguation. Without sessions, "crown" always resolves the same way;
with sessions, context drives meaning.

---

## Why this sprint exists

Idan's requirement:
> "אם אני מדבר עם רופא שיניים... הם מתכוונים כנראה לכתר לשיניים"

Disambiguation of polysemous terms ("crown" = dental / royal / brand) requires
**session context**: what has been discussed, who is speaking, about what topic.

This sprint adds session primitives and context-aware disambiguation.

---

## Tasks

### Task H1 — Session synset type
```rust
pub struct SessionId(pub u64);

// Sessions stored as synsets in graph (IDs 10_000_000..)
impl Graph {
    pub fn create_session(
        &mut self,
        user_id: SynsetId,
        agent_id: SynsetId,
        channel: ChannelKind,
    ) -> SessionId;

    pub fn end_session(&mut self, s: SessionId);
}
```

### Task H2 — Turn tracking
```rust
pub struct Turn {
    pub id: TurnId,
    pub session: SessionId,
    pub input: String,
    pub output: String,
    pub mentioned_synsets: Vec<SynsetId>,
    pub timestamp: u64,
}

impl Graph {
    pub fn record_turn(&mut self, s: SessionId, turn: Turn);
    pub fn session_turns(&self, s: SessionId) -> Vec<&Turn>;
}
```

### Task H3 — Session context as synset bag
```rust
pub fn session_context_synsets(
    graph: &Graph,
    s: SessionId,
    recency_weight: f32,
) -> HashMap<SynsetId, f32>
```

Returns a weighted bag of synsets that have appeared in this session.
More recent turns = higher weight.

### Task H4 — Context-aware disambiguation
Extend Sprint B's ambiguity resolver:
```rust
pub fn resolve_entity_in_context(
    candidates: &[SynsetId],
    context: &HashMap<SynsetId, f32>,
    graph: &Graph,
) -> SynsetId {
    // For each candidate, sum graph-distance-weighted overlap with context
    // Pick candidate with highest score
}
```

### Task H5 — Temporal interpretation
Idan's second example:
> "מתי תתקשר / מאוחר יותר → דקות vs שנים"

Add `TimeContext` type:
```rust
pub enum TimeContext {
    Seconds,    // "right now", "in a moment"
    Minutes,    // "later today", "soon"
    Hours,      // "this evening"
    Days,       // "this week"
    Months,     // "next year"
    Years,      // life events
    Unknown,
}

pub fn infer_time_scale(
    event_synset: SynsetId,  // e.g., "phone call" vs "wedding"
    graph: &Graph,
) -> TimeContext
```

Event synsets carry a `TypicalTimeScale` edge set at ingest time.

### Task H6 — Session persistence across restarts
Sessions serialize to graph file alongside everything else. After restart,
sessions are queryable. Tests cover this.

### Task H7 — Tests
- Create session, record turns, retrieve turns
- Disambiguate "crown" differently based on session context
- Time scale inference: "I'll call later" vs "I'll marry later"
- Session persists across graph reload

---

## Acceptance criteria

- [ ] 85 existing + at least 10 new tests pass (95 total)
- [ ] Benchmark: session creation + disambig <1ms
- [ ] Real-world disambig test: feed 5-turn dental session, query "crown",
  must resolve to dental synset not royal

---

## Commit format

```
Sprint H: Sessions + Context-Aware Disambiguation

- SessionId, Turn, session management (all graph-native)
- session_context_synsets: weighted bag of session synsets
- Context-aware entity resolution (crown = dental vs royal)
- TimeContext: temporal interpretation based on event type
- Session persistence across graph reloads

Tests: 85 -> 95 passing.
Addresses Idan's disambiguation requirement for Cognitive Tree (Sprint D).
```
