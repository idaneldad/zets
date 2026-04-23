# MISSION: Agent A — CapabilityOrchestrator

**Branch:** `feat/capability-runtime`
**Estimated time:** 3-4 hours
**Priority:** CRITICAL — this unblocks all of Tier A (Cat 6, 7, 9, 10)

---

## Context

You are Claude Code working on ZETS, a graph-based knowledge engine in
Rust. Read `/home/dinio/zets/docs/README.md` first for context.

The benchmark identifies 5 categories where ZETS scores 0.00-0.15 because
it has no way to invoke external APIs (Whisper, Gemini Vision, Midjourney,
Suno, Sora). This module solves that.

**Your task:** Build `src/capability_runtime/` — a module that invokes
external capabilities safely, with budget, retry, rate limits, and audit.

Do NOT implement actual Whisper/Gemini calls yet. Just the framework.

---

## Rules of engagement

1. **Branch:** `feat/capability-runtime`. Check out from main. NEVER push to main directly.
2. **Scope:** You create NEW files only in `src/capability_runtime/`. You may NOT modify:
   - `src/lib.rs` (Idan will do registration)
   - `Cargo.toml` (confirm with Idan before adding deps)
   - ANY other file
3. **Commits:** Use conventional commits. `feat(capability_runtime): ...`
4. **Tests:** 15+ tests minimum, cargo test --lib must pass
5. **No hallucinations:** If something is unclear, STOP and write a question to Idan in a file `QUESTIONS.md`. Do not guess.

---

## Interface contract

This is the public API. Don't deviate without discussing with Idan.

```rust
use std::time::Duration;
use crate::personal_graph::IdentityId;
use crate::secrets::Vault;

pub struct CapabilityOrchestrator {
    // internal fields
}

pub struct CapabilityInvocation {
    pub capability_id: String,         // e.g. "whisper.transcribe"
    pub args: serde_json::Value,       // capability-specific args
    pub caller: IdentityId,            // who's calling (for audit + ACL)
    pub max_timeout_ms: u64,           // per-call timeout
    pub max_budget_cents: u32,         // cost ceiling
}

pub enum CapabilityResult {
    Success { output: serde_json::Value, cost_cents: u32, duration_ms: u64 },
    Timeout,
    BudgetExceeded,
    RateLimited { retry_after_ms: u64 },
    TransientError { retry_count: u32 },
    PermanentError { reason: String },
}

#[derive(Debug)]
pub enum CapabilityError {
    NotRegistered(String),
    AccessDenied,
    InvalidArgs(String),
    Unavailable,
}

impl CapabilityOrchestrator {
    pub fn new(vault: /* shared ref */, budget_tracker: BudgetTracker) -> Self;
    pub async fn invoke(&self, inv: CapabilityInvocation) -> Result<CapabilityResult, CapabilityError>;
    pub fn register(&mut self, definition: CapabilityDefinition);
}

pub struct CapabilityDefinition {
    pub id: String,
    pub description: String,
    pub provider: Provider,            // HttpPost | Local | Custom
    pub endpoint: Option<String>,      // for HttpPost
    pub auth_secret_id: Option<String>, // key into vault
    pub cost_per_call_cents: u32,
    pub rate_limit_per_minute: u32,
    pub typical_latency_ms: u64,
}
```

---

## Files to create

```
src/capability_runtime/
    mod.rs               ← module declaration + re-exports
    orchestrator.rs      ← CapabilityOrchestrator main impl
    definition.rs        ← CapabilityDefinition + Provider enum
    invocation.rs        ← CapabilityInvocation struct
    result.rs            ← CapabilityResult + CapabilityError enums
    registry.rs          ← in-memory registry of definitions
    budget.rs            ← BudgetTracker (per-owner cents spent)
    rate_limit.rs        ← Token bucket rate limiter
    executor.rs          ← actual HTTP call execution (stub OK for v1)
    audit.rs             ← record invocations to structured log
```

---

## Features (in priority order)

### 1. Registry + lookup (1h)
- `ConnectorRegistry::register(definition)`
- `lookup(capability_id) -> Option<&Definition>`
- No HTTP yet — just store + retrieve

### 2. Budget tracking (45min)
- Per-caller (IdentityId) cents spent, monotonic
- Reject if + cost_of_call > max_budget
- Reset command for testing

### 3. Rate limiting (45min)
- Token bucket per capability_id
- Returns RateLimited{retry_after_ms} if exhausted
- Refill rate = rate_limit_per_minute / 60 tokens per second

### 4. Orchestrator.invoke() (1h)
- ACL check: does caller's identity have permission for this capability?
- Budget check (see 2)
- Rate limit check (see 3)
- Resolve secret from vault (stub OK — just verify it exists)
- Call executor (stub for now — return Success with dummy)
- Record to audit

### 5. Audit log (30min)
- Append-only in-memory log
- Each entry: timestamp, caller, capability, duration, cost, result type
- No message content (PII-free)
- Query by caller, by capability, date range

### 6. Error handling + retry (30min)
- TransientError → retry up to 3 times, exponential backoff
- PermanentError → no retry
- Timeout → no retry, return Timeout
- BudgetExceeded → no retry

---

## Dependencies you can use

Already in Cargo.toml (check first!):
- `tokio` (async runtime)
- `serde`, `serde_json`
- `thiserror` (error types)
- `async-trait`

Do NOT add new deps without asking Idan first. If you need reqwest (HTTP),
ask first — it's a big dep.

---

## Test requirements

Minimum 15 tests. Examples:

```rust
#[tokio::test]
async fn test_register_and_lookup() { ... }

#[tokio::test]
async fn test_invoke_unregistered_fails() { ... }

#[tokio::test]
async fn test_budget_exhaustion_blocks() { ... }

#[tokio::test]
async fn test_rate_limit_triggers() { ... }

#[tokio::test]
async fn test_retry_on_transient() { ... }

#[tokio::test]
async fn test_no_retry_on_permanent() { ... }

#[tokio::test]
async fn test_timeout_honored() { ... }

#[tokio::test]
async fn test_audit_records_invocation() { ... }

#[tokio::test]
async fn test_per_caller_budget_isolated() { ... }

#[tokio::test]
async fn test_acl_blocks_unauthorized() { ... }
```

---

## Done criteria

1. ✅ All 10 files created
2. ✅ 15+ tests, all passing
3. ✅ `cargo build --lib` clean (0 errors)
4. ✅ `cargo clippy --lib` no new warnings
5. ✅ No modifications to `src/lib.rs` (leave a note `// TODO: register module` nowhere — Idan will add)
6. ✅ README.md in `src/capability_runtime/` explaining the module
7. ✅ All your commits pushed to `feat/capability-runtime`
8. ✅ PR created to main with clear description

---

## When you're blocked

Write to `QUESTIONS.md` at repo root:
- What you're blocked on
- What you tried
- What you think the answer might be

Then STOP and wait. Do not guess.

---

## Final instruction

After you finish:
1. `cargo test --lib capability_runtime` — must show X passed, 0 failed
2. `git push origin feat/capability-runtime`
3. Create PR: "Add CapabilityOrchestrator module"
4. Tag Idan for review

Go.
