# Capability Runtime — External API Orchestration

**Module:** `src/capability_runtime/`
**Status:** 🟢 0.80 / 1.00
**Landed:** 23.04.2026 (Agent A, Opus 4.7)
**Tests:** 44 passing
**LOC:** ~1,887

## מה המשימה

ZETS לבד לא יודע לעשות Speech-to-Text, Vision understanding, image generation ב-studio quality, music generation, video generation. אבל הוא יכול **לתזמר** APIs חיצוניים שעושים את זה — באופן בטוח, עם budget, rate-limiting, retry, ו-audit.

## הקריטריון להצלחה

- [x] Orchestrator שמקבל `invoke(capability_id, args, caller) → Result`
- [x] Registry של capabilities עם definitions
- [x] Budget tracking per-caller (monotonic cents)
- [x] Rate limiting per-capability (token bucket)
- [x] Retry on transient errors (exponential backoff)
- [x] Timeout enforcement
- [x] Audit log (PII-free, append-only)
- [ ] **ממתין:** wiring של capabilities אמיתיות (Whisper, Gemini Vision)

## איך בוחנים (44 tests)

### QA (איכות ארכיטקטונית)
- Registry lookup נכון
- Orchestrator מזהה unregistered capability
- Budget exhaustion חוסם invocation
- Rate limit מחזיר retry_after_ms
- Retry על transient error — exponential backoff
- No retry על permanent error
- Audit log רושם כל invocation

### TEST (התנהגות עם עומס)
- Per-caller budget isolation
- Concurrent invocations (sync + mutex)
- Timeout honored
- ACL blocks unauthorized caller

## באחריות

**גרף + חיצוני** (hybrid):
- **Graph side** (ZETS's responsibility): decide when to invoke, with what args, based on conversation context + user preferences
- **External side** (capability runtime): execute the call, handle auth, retries, budgets, rate limits

## קוד

```
src/capability_runtime/
├── mod.rs                (84 lines)  — module root + re-exports
├── orchestrator.rs       (529 lines) — CapabilityOrchestrator main impl
├── registry.rs           (130 lines) — ConnectorRegistry
├── definition.rs         (132 lines) — CapabilityDefinition + Provider
├── invocation.rs         (73 lines)  — CapabilityInvocation
├── result.rs             (222 lines) — Value enum + Result + Error
├── budget.rs             (194 lines) — BudgetTracker
├── rate_limit.rs         (202 lines) — Token bucket
├── executor.rs           (144 lines) — Executor trait + StubExecutor
├── audit.rs              (177 lines) — Append-only audit log
└── README.md             (82 lines)  — Module docs
```

## Interface

```rust
pub struct CapabilityOrchestrator { /* ... */ }

pub struct CapabilityInvocation {
    pub capability_id: String,       // "whisper.transcribe"
    pub args: Value,                 // capability-specific
    pub caller: IdentityId,
    pub max_timeout_ms: u64,
    pub max_budget_cents: u32,
}

pub enum CapabilityResult {
    Success { output: Value, cost_cents: u32, duration_ms: u64 },
    Timeout,
    BudgetExceeded,
    RateLimited { retry_after_ms: u64 },
    TransientError { retry_count: u32 },
    PermanentError { reason: String },
}

impl CapabilityOrchestrator {
    pub fn invoke(&self, inv: CapabilityInvocation) -> Result<CapabilityResult, CapabilityError>;
    pub fn register(&mut self, definition: CapabilityDefinition);
}
```

## Notable decisions

**Zero new dependencies.** Mission said tokio/serde_json/thiserror would be available — they weren't. Agent adapted:
- Custom `Value` enum instead of `serde_json::Value`
- Sync functions (TODO: convert to async when tokio is added)
- Manual `Display` + `Error` impls

This matches ZETS's "zero runtime dependencies" philosophy.

## פער (מה חסר להגיע ל-1.00)

1. **Async conversion** — when tokio is added, convert sync → async
2. **Real capability wiring** — Whisper, Gemini Vision, Midjourney, Suno
3. **Cost estimation UI** — "before invoking, tell me estimated cost"
4. **Parallel invocations** — בטוח, אבל עם deadlock prevention
5. **Integration tests** — mock HTTP server + e2e scenarios

## Impact על HumannessScore

Cat 13 (Task Orchestration): 0.34 → 0.52 (+0.18)

אחרי שWhisper+Vision יוחברו: Cat 6 (Speech) 0.00 → 0.75, Cat 7 (Vision) 0.06 → 0.70 — זה הדרך מ-0.48 ל-0.60 (MVP).
