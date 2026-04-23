# capability_runtime

Invoke external capabilities (Whisper, Gemini Vision, Midjourney, Suno, Sora)
safely, with budget tracking, rate limiting, ACL, and audit.

## Architecture

```
CapabilityOrchestrator.invoke(invocation)
  ├── 1. Registry lookup     → ConnectorRegistry
  ├── 2. ACL check           → HashSet<(caller, capability_id)>
  ├── 3. Budget check        → BudgetTracker (per-caller cents)
  ├── 4. Rate limit check    → RateLimiter (token bucket per capability)
  ├── 5. Secret resolution   → Stub (→ Vault integration later)
  ├── 6. Execute             → Executor trait (StubExecutor for v1)
  ├── 7. Retry on transient  → up to 3× with backoff
  └── 8. Audit log           → AuditLog (PII-free)
```

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations + re-exports |
| `result.rs` | `Value` enum, `CapabilityResult`, `CapabilityError` |
| `definition.rs` | `CapabilityDefinition`, `Provider` enum |
| `invocation.rs` | `CapabilityInvocation` request struct |
| `registry.rs` | `ConnectorRegistry` — in-memory capability store |
| `budget.rs` | `BudgetTracker` — per-caller cost accounting |
| `rate_limit.rs` | `RateLimiter` — token-bucket per capability |
| `executor.rs` | `Executor` trait, `StubExecutor`, `MockExecutor` |
| `audit.rs` | `AuditLog` — append-only invocation records |
| `orchestrator.rs` | `CapabilityOrchestrator` — ties everything together |

## Usage

```rust
use zets::capability_runtime::*;
use zets::personal_graph::{IdentityId, IdentityKind};

let budget = BudgetTracker::with_default_limit(1000); // $10 max
let mut orch = CapabilityOrchestrator::new(budget);

// Register capability
orch.register(
    CapabilityDefinition::new("whisper.transcribe", "Transcribe audio", Provider::HttpPost)
        .with_cost(3)
        .with_rate_limit(60)
        .with_auth_secret("person:idan/api_key/openai"),
);

// Register secret (stub — will use Vault in production)
orch.register_secret("person:idan/api_key/openai", "sk-...");

// Grant access
let caller = IdentityId::new(IdentityKind::Person, "idan");
orch.grant_access(&caller, "whisper.transcribe");

// Invoke
let inv = CapabilityInvocation::new("whisper.transcribe", Value::Null, caller)
    .with_timeout(10_000)
    .with_budget(50);
let result = orch.invoke(&inv);
```

## TODOs for integration

- [ ] Wire `Vault` for real secret resolution (currently uses in-memory stub)
- [ ] Add `tokio` for async execution when it's added to Cargo.toml
- [ ] Replace `Value` with `serde_json::Value` when serde_json is available
- [ ] Add `HttpExecutor` using `reqwest` for real HTTP calls
- [ ] Add exponential backoff sleep in retry loop (needs tokio)

## Tests

44 tests covering: registry CRUD, budget enforcement, rate limiting,
ACL checks, retry logic, audit logging, secret resolution, and full
orchestrator integration.

```sh
cargo test --lib capability_runtime
```
