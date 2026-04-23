# Sprint A — CLI cleanup + Iterator APIs

**Estimated time:** 4-6 hours of focused work
**Branch:** `sprint-a-cli`
**Risk level:** Low — refactor of working code, no new behavior
**Dependencies:** None (this is the first sprint)
**Blocks:** Sprints B, C, ING1

---

## Why this sprint exists

External reviews (Gemini + Perplexity) identified specific inefficiencies in
the current code that need fixing before adding more features. These are
"engineering hygiene" issues that compound if not addressed early.

Specifically:
1. CLI uses `env::args().collect()` — allocates Vec<String> when iterator works
2. CLI does `rest[1..].join(" ")` — extra allocation
3. CLI calls `println!` in loops — locks stdout every line
4. CLI uses `process::exit(1)` inside helpers — untestable
5. `EdgeStore::outgoing()` returns `Vec<Edge>` — heap allocation per call
6. `Graph::outgoing()` falls back to linear scan if index stale — should always-index

After Sprint A: cleaner APIs, measurable speedup, easier tests.

---

## Tasks (in order)

### Task A1 — CLI: switch to iterator-based args
**File:** `src/bin/zets.rs`

Current code:
```rust
let args: Vec<String> = env::args().collect();
let cmd = &args[1];
```

Replace with:
```rust
let mut args = env::args();
let _program = args.next().unwrap_or_else(|| "zets".into());
let cmd = match args.next() {
    Some(c) => c,
    None => { print_usage(); return ExitCode::FAILURE; }
};
```

### Task A2 — CLI: lock stdout once per command
**File:** `src/bin/zets.rs`

Current code uses `println!` everywhere. Each call locks stdout.

Add at start of `main`:
```rust
let stdout = io::stdout();
let mut out = stdout.lock();
```

Then replace all `println!("...")` inside command handlers with
`writeln!(out, "...")`. Each command function takes `&mut impl Write`
as parameter.

### Task A3 — CLI: build text without join allocation
**File:** `src/bin/zets.rs`, function `cmd_normalize`

Current code:
```rust
let text = rest[1..].join(" ");
```

Replace with:
```rust
let mut text = String::with_capacity(128);
for arg in args {
    if !text.is_empty() { text.push(' '); }
    text.push_str(&arg);
}
```

### Task A4 — CLI: Result-based error flow
**File:** `src/bin/zets.rs`

Current handlers use `process::exit(1)` and `eprintln!` directly.

Refactor:
```rust
fn main() -> ExitCode {
    if let Err(e) = run_cli() {
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run_cli() -> Result<(), String> { /* dispatch here */ }
```

Each `cmd_*` function returns `Result<(), String>`.

### Task A5 — EdgeStore: add iterator-based outgoing
**File:** `src/lib.rs`, module `edge_store`

KEEP existing `outgoing(&self, src) -> Vec<Edge>` for backward compat.

ADD new method:
```rust
pub fn outgoing_iter<'a>(&'a self, src: SynsetId)
    -> impl Iterator<Item = Edge> + 'a
{
    // Returns iterator that streams edges without allocating Vec.
    // Use AdjacencyIndex slice + map.
}
```

Same for `incoming_iter`.

### Task A6 — Graph: route through iterator API
**File:** `src/lib.rs`, module `graph`

Update `Graph::outgoing` and similar to call `outgoing_iter().collect()`
internally so it's clear there's only one code path.

### Task A7 — Graph: index always-on after load
**File:** `src/lib.rs`, module `graph`

Currently: `Graph::outgoing` may fall back to linear scan if index stale.

Change: After every `Graph::insert_edge`, mark index dirty. Before any
read operation, if dirty, rebuild index. Never linear-scan.

For batch loads: add `Graph::insert_batch(edges)` that defers index
rebuild to end.

### Task A8 — Add benchmark for iterator vs Vec
**File:** `src/bin/scale_probe.rs`

Add benchmark:
- Run outgoing() 10K times via Vec API → measure
- Run outgoing_iter() 10K times → measure
- Print ratio

Goal: iterator should be ≥50% faster on hot loops.

---

## Acceptance criteria

Before marking Sprint A complete:

- [ ] All 72 existing tests pass (`cargo test`)
- [ ] At least 5 new tests added covering iterator APIs
- [ ] CLI behavior unchanged from user perspective (manual verification)
- [ ] Benchmark shows iterator API faster than Vec API on loops ≥1000
- [ ] No new dependencies added
- [ ] All `process::exit` calls removed from inside command functions
- [ ] All `println!` in CLI commands replaced with `writeln!(out, ...)`

---

## Files touched

- `src/bin/zets.rs` (refactored)
- `src/lib.rs` (edge_store module + graph module)
- `src/bin/scale_probe.rs` (new benchmark)
- New tests in `src/lib.rs` test module

Estimated changed lines: ~250.

---

## Testing protocol

Run on Oracle server, not sandbox:

```bash
cd /home/dinio/zets
cargo test --release 2>&1 | tail -20
cargo build --release --bin zets --bin scale_probe
./target/release/zets normalize he "שלום עולם"
./target/release/zets languages
./target/release/scale_probe
```

All should succeed. Test count must be ≥77 (72 existing + 5 new).

---

## Commit format

```
Sprint A: CLI iterator refactor + EdgeStore iterator API

- CLI: env::args() iterator, stdout locking, Result-based errors
- EdgeStore: outgoing_iter() returning Iterator<Item=Edge>
- Graph: index always-rebuilt before reads, no linear-scan fallback
- Benchmark added: iterator API X% faster on outgoing() hot loop

Tests: 72 -> 77 passing.
Addresses Gemini + Perplexity feedback on hot-path allocations.
```

---

## If something is unclear

DO NOT GUESS. Write findings to `docs/sprints/SPRINT_A_NOTES.md` describing
the ambiguity, push notes branch, ping Idan.

The whole point of these briefs is to remove guesswork. If a brief is
ambiguous, the brief is wrong, not your interpretation.
