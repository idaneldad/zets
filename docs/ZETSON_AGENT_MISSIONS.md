# Zetson Development — 7 Claude Code Agent Missions

**Spec reference:** [ZETS_LEARNING_HOW_IT_LEARNS.md](ZETS_LEARNING_HOW_IT_LEARNS.md)
**Seed file:** [config/zetson-infant.yaml](../config/zetson-infant.yaml)
**Purpose:** Parallel agents build the Zetson demo. Each agent has strict boundaries.

---

## Rules that apply to ALL agents

1. **Zero new crate dependencies.** ZETS policy: pure std + existing crates only. If a capability requires a new crate, document why and stop — do not add it.
2. **`pub mod` registration is allowed in `src/lib.rs`** (pragmatic — otherwise tests can't run).
3. **Every new module has 10+ tests.** No exceptions.
4. **No self-modifying code.** Agents write code. ZETS never will.
5. **Deterministic.** No rand::random, no SystemTime::now without injection for tests.
6. **`cargo test --lib` must pass** on agent's branch before PR.
7. **Agents write a `docs/working/<date>_<agent>_notes.md`** explaining decisions.
8. **Agents push to `feat/<branch-name>`** and leave a PR URL in their final message.

---

## Track 1 — Complete the Primitives (2 agents, parallel)

### Agent P-A — HTTP Fetch Primitive
- **Branch:** `feat/primitive-http-fetch`
- **Module:** `src/net/http_fetch.rs` + `src/net/mod.rs`
- **Mission:**
  - Zero-dep HTTP/HTTPS client: TcpStream + minimal TLS (investigate if existing crates in Cargo.toml suffice; if not, `rustls` is acceptable IF unavoidable — document).
  - Support GET with:
    - User-Agent header
    - Timeout (configurable, default 10s)
    - Redirect follow (max 3 hops)
    - Proper status-code handling
  - `robots.txt` check helper — `fn robots_allows(url) -> bool`.
  - Rate limit per-host (configurable) — respects `max_requests_per_host_per_minute` from Zetson seed.
- **Interface:**
```rust
pub struct HttpFetcher { /* config: timeout, user_agent, rate_limiter */ }

pub struct FetchResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub final_url: String,
    pub elapsed_ms: u64,
}

impl HttpFetcher {
    pub fn new(user_agent: &str) -> Self;
    pub fn get(&mut self, url: &str) -> Result<FetchResponse, FetchError>;
    pub fn robots_allows(&mut self, url: &str) -> bool;
}
```
- **Tests required (15+):**
  - Basic GET on httpbin or example.com
  - Redirect handling
  - Timeout triggers correctly
  - robots.txt parse + enforce
  - Rate limit enforcement (mock clock)
  - 404/500 error paths
  - Large response handling
  - Custom User-Agent sent
  - Header parsing
  - UTF-8 body decode
- **Done when:** 15+ tests passing, `cargo check` clean, PR pushed.

### Agent P-B — HTML Parser Primitive
- **Branch:** `feat/primitive-html-parser`
- **Module:** `src/parse/html.rs` + `src/parse/mod.rs`
- **Mission:**
  - Zero-dep HTML → structured text extractor.
  - Handle malformed HTML gracefully (real Wikipedia has plenty).
  - Extract:
    - Article body text (prioritizing `<article>`, `<main>`, `<div id="content">`)
    - Links (`href` + anchor text)
    - Metadata: title, description, lang, canonical URL
    - Headings structure (h1-h6) — for outline extraction
- **Interface:**
```rust
pub struct HtmlDocument {
    pub title: Option<String>,
    pub language: Option<String>,
    pub body_text: String,                    // plain text, formatting stripped
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub metadata: HashMap<String, String>,
}

pub fn parse_html(bytes: &[u8]) -> Result<HtmlDocument, ParseError>;
pub fn parse_html_utf8(text: &str) -> Result<HtmlDocument, ParseError>;
```
- **Tests required (15+):** minimal doc, Wikipedia sample, malformed HTML, entities (`&nbsp;`, `&amp;`), UTF-8 + Hebrew, nested tags, self-closing tags, scripts/styles stripped, links extraction.
- **Done when:** PR pushed, tests passing, documented real Wikipedia parse example in `docs/working/`.

---

## Track 2 — Seed the Procedure Graph (2 agents, parallel, AFTER Track 1)

### Agent P-C — Procedure Atom Loader + Initial Library
- **Branch:** `feat/procedure-library`
- **Modules:**
  - `src/procedure_atom/loader.rs` — TOML loader for procedure atoms
  - `data/procedures/*.toml` — 20 initial procedure files
- **Mission:**
  - Read TOML files from `data/procedures/`, instantiate as `ProcedureAtom`s in the graph.
  - Validate: referenced steps must be other known procedures OR primitive IDs.
  - 20 initial procedures (listed in spec). Write them as TOML files.
- **Example procedure file (`data/procedures/fetch-url-politely.toml`):**
```toml
[procedure]
id = "fetch-url-politely"
description = "Fetch a URL respecting robots.txt and rate limits"
trust_level = "System"
allowed_tools = ["http_fetch", "robots_check"]

[[procedure.step]]
id = "check_robots"
primitive = "robots_check"
on_false = "halt"

[[procedure.step]]
id = "rate_limit"
primitive = "rate_limit_gate"
per_host_rpm = 10

[[procedure.step]]
id = "fetch"
primitive = "http_fetch"
timeout_ms = 10000

[when_to_use]
conditions = ["caller wants web content", "url is from allowlisted domain"]

[when_not_to_use]
conditions = ["offline mode", "budget exceeded"]
```
- **Tests required (30+):** each procedure validates, DAG has no cycles, referenced primitives exist, trust-level checks, allowed-tools enforcement.
- **Done when:** 20 procedures loaded into an AtomStore, query `find_procedure("fetch-url-politely")` returns it, PR pushed.

### Agent P-D — Learning Loop Executor
- **Branch:** `feat/learning-loop`
- **Modules:**
  - `src/learner/executor.rs` — walks procedure DAG, calls primitives
  - `src/bin/zetson_tick.rs` — CLI entry: one learning tick
- **Mission:**
  - Given a procedure atom ID, walk its DAG executing each step.
  - On each step, call the right primitive function (from Track 1 outputs + existing ones).
  - Enforce budget from the seed (HTTP req count, storage MB, CPU time).
  - Observability: every step logged to `data/zetson/reports/`.
  - Error handling: if primitive fails, trigger the procedure's `on_error` (or halt + log).
- **Interface:**
```rust
pub struct Learner {
    store: AtomStore,
    primitives: PrimitiveRegistry,
    budget: Budget,
    log_sink: LogSink,
}

impl Learner {
    pub fn execute_procedure(&mut self, proc_id: &str, context: &ExecutionContext) -> ExecResult;
    pub fn tick(&mut self, max_procedures: usize) -> TickReport;
}
```
- **Tests required (20+):** end-to-end "fetch → parse → store → link" loops on mock HTTP responses, budget enforcement, determinism (same input → same graph), failure recovery.
- **Done when:** `./zets_tick --seed=config/zetson-infant.yaml --procedures=5` completes, graph grows, report written, PR pushed.

---

## Track 3 — Zetson Infrastructure (2 agents, parallel, can run with Track 2)

### Agent P-E — Seed Loader + Identity Bootstrap
- **Branch:** `feat/seed-loader`
- **Module:** `src/seed/loader.rs`
- **Mission:**
  - Parse `config/zetson-infant.yaml` (zero-dep YAML subset is OK — we only need the subset we use).
  - Inject atoms into a fresh AtomStore:
    - Identity atoms (name, role, personality)
    - Goal atoms (one per goal with progress=0)
    - Budget atoms (read by Learner)
    - Safety-limit atoms
  - Link allowlisted domains, parent, etc., as first-class graph citizens.
- **Interface:**
```rust
pub struct Seed {
    pub identity: Identity,
    pub goals: Vec<Goal>,
    pub budget: Budget,
    pub safety: SafetyLimits,
    pub packages: Vec<String>,
}

pub fn load_seed(path: &str) -> Result<Seed, SeedError>;
pub fn inject_seed(seed: &Seed, store: &mut AtomStore) -> InjectReport;
```
- **Tests required (15+):** Zetson YAML loads correctly, atoms exist after inject, identity/goal/budget accessible, edge-case YAMLs, malformed YAML errors, re-inject idempotent.
- **Done when:** Fresh ZETS started with Zetson seed has the expected initial atom set, PR pushed.

### Agent P-F — Observability Dashboard
- **Branch:** `feat/observability`
- **Module:** `src/bin/zetson_dashboard.rs`
- **Mission:**
  - Read `data/zetson/reports/*.json` from last 30 days.
  - Aggregate: atoms over time, edges over time, languages grown, goals progress, budget utilization.
  - Output: simple HTML with inline SVG charts + JSON endpoint.
  - Runs as `./zetson_dashboard --port=8080`, binds to localhost.
- **Tests required (10+):** report parsing, aggregate math, HTML generation, missing report days handling, JSON endpoint schema validation.
- **Done when:** Dashboard loads, shows growth curve from ≥ 3 days of mocked reports, PR pushed.

---

## Track 4 — Integration (1 agent, AFTER all above)

### Agent P-G — End-to-End Zetson Demo
- **Branch:** `feat/zetson-demo`
- **Depends on:** P-A through P-F merged to main.
- **Mission:**
  - Compose everything into a working Zetson instance.
  - A demo script (`scripts/zetson_first_day.sh`) that:
    1. Starts fresh ZETS with Zetson seed.
    2. Runs learning loop for a bounded period (e.g., 50 HTTP requests, then stop).
    3. Generates a report.
    4. Verifies: atoms grew, goals progressed, budget respected.
  - Record measurements — investor-facing numbers.
- **Tests required (15+):** full pipeline runs, no crashes across 50 tick cycles, report fields match expected schema.
- **Done when:** A user can run `scripts/zetson_first_day.sh`, see real graph growth, and show the dashboard to an investor.

---

## Milestones

| Milestone | What's done | When |
|-----------|-------------|------|
| **M1** | Track 1 complete (primitives) | 2-3 hours |
| **M2** | Track 2 complete (learning loop + procedures) | +3-4 hours |
| **M3** | Track 3 complete (Zetson infra) | +2-3 hours |
| **M4** | Track 4 integration | +2 hours |
| **M5** | First 24h Zetson run, first growth report | +24h wall-clock |
| **M6** | First week Zetson milestone review | +1 week wall-clock |
| **M7** | 30-day snapshot — investor demo ready | +30 days wall-clock |
| **M8** | 90-day graduation milestone | +90 days wall-clock |

---

## If an Agent Is Stuck

Write to `/home/dinio/agent-logs/agent-<id>-questions.md` with:
- The specific blocker
- What you tried
- What decision you made (if any) to proceed or halt

Idan reviews these daily. Do NOT invent answers to unclear spec questions — ask.

---

## Why This Plan Is Right

1. **Code + Graph split is clean.** Primitives in Rust. Everything else as atoms.
2. **Agents are bounded.** Each has strict module scope. No overlap = no merge conflicts.
3. **Tests gate progress.** Nothing merges without 10+ tests.
4. **Zetson is the demo.** Investors don't want to read specs — they want to see a mind grow. This plan ends in a mind growing.
5. **Reversible.** Zetson lives in its own data directory. If anything goes wrong — delete it, start over with same seed, same result (determinism).
6. **Idan stays in control.** Every atom is tagged. Every HTTP request logged. No surprise behavior.

---

*Spec ready. Waiting for Idan to launch agents.*
