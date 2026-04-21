# ZETS Nervous System — Tool Execution Layer Built On Graph

**Date:** 21.04.2026
**Author:** Idan Eldad + Claude (Opus 4.7)
**Status:** Architecture spec — awaiting Idan sign-off
**Influences:** OpenClaw architecture (rejected wholesale, patterns extracted),
  external analyses by Perplexity and Gemini (in `research/openclaw/01_external_analysis.md`)
**Engineering target:** Make ZETS able to take ACTIONS, not just answer questions.

---

## 0. The framing Idan proposed

> "OpenClaw is the nervous system that knows how to activate tools.
> Tools can be internal or external.
> If we keep rules and memories in the graph, it's far more reliable
> and stable than OpenClaw which has many problems."

This is correct. The model is:

```
┌────────────────────────────────────────────────┐
│              THE BRAIN (ZETS graph)            │
│   - Knowledge: facts, definitions, relations    │
│   - Memory:    sessions, conversations          │
│   - Rules:     governance, permissions          │
│   - All deterministic, all in graph nodes/edges │
└────────────────────────────────────────────────┘
                         │
                         │ activates
                         ▼
┌────────────────────────────────────────────────┐
│        THE NERVOUS SYSTEM (Tool Bus)           │
│   - Compiled Rust tool functions                │
│   - Permission gates (graph-defined)            │
│   - Audit trail (back into graph)               │
│   - Sandbox tiers per session type              │
└────────────────────────────────────────────────┘
                         │
                         │ executes
                         ▼
┌────────────────────────────────────────────────┐
│            THE LIMBS (Tools)                    │
│   - Internal: graph queries, math, formatting   │
│   - External: HTTP, file I/O, shell (sandboxed) │
│   - Each tool is a Rust function, not a script  │
└────────────────────────────────────────────────┘
```

This is fundamentally different from OpenClaw's model, which assumes:
- The LLM decides what to do (we don't have an LLM in core)
- Skills are markdown files interpreted at runtime (we use compiled functions)
- Memory is external storage (we use the graph itself)
- Tools are scripts (we use sandboxed Rust)

**Why our way wins on Pi:**
- No always-on Gateway eating RAM
- No scripts written by LLM at runtime (security)
- No external memory layer (graph IS the memory)
- No third-party channel adapters in core (deferred to optional cloud relay)

---

## 1. The five patterns we ARE taking from OpenClaw

After reviewing the repo and external analyses, these five patterns are
worth adopting. Each maps cleanly to graph-native primitives.

### Pattern 1: Workspace as data
OpenClaw's `AGENTS.md`, `SOUL.md`, `TOOLS.md`, `SKILL.md` files define agent
behavior as files, not code.

**Our equivalent:** All of these become graph nodes:
- Agent definition → `Agent` synset with edges to Tool nodes
- Personality/identity → `Persona` synset with edges to language style rules
- Available tools → `Tool` synsets with `AvailableTo` edges
- Skill content → `Skill` synsets with edges to procedure steps

**Win:** Same flexibility (data-driven configuration), but stored
in the graph alongside everything else. No file watchers, no config reload.

### Pattern 2: Sessions are first-class
OpenClaw maintains long-running sessions per (channel × user × agent).

**Our equivalent:** `Session` synset with edges:
- `StartedBy` → User synset
- `BelongsToAgent` → Agent synset
- `OnChannel` → Channel synset
- `ContainsTurn` → ordered Turn synsets (each holds query + response)
- `RememberedFact` → Fact synsets surfaced during this session

**Win:** Sessions queryable like any other graph data. "What did we discuss
yesterday?" is a graph walk, not a database query.

### Pattern 3: Tool permissions
OpenClaw has main-session vs sandboxed-session distinction.

**Our equivalent:** `PermissionTier` synsets with `AllowsTool` edges.
- Tier 0 (read-only): query graph, list known tools
- Tier 1 (safe writes): add memory, log feedback
- Tier 2 (file I/O): read user files within whitelisted dirs
- Tier 3 (network): fetch URLs (with rate limit + domain allowlist)
- Tier 4 (shell): execute pre-defined commands
- Tier 5 (full): only for explicitly-blessed admin sessions

Every Tool node has a `RequiresTier` edge. Every Session has a `GrantedTier` edge.
Pre-execution check: Session.Tier >= Tool.RequiredTier? If not, refuse.

**Win:** Permission system is data, audit-able, modifiable without code change.

### Pattern 4: Background loops
OpenClaw has cron, heartbeats, reminders, watchdogs.

**Our equivalent:** `Schedule` synset with edges to procedures.
- A daemon thread polls scheduled procedures every 1 second
- Procedures run as if they were query handlers (same engine, same graph)
- Results write back to graph as new facts

**Win:** No external scheduler. State is graph. Restart resumes from where
graph says we are.

### Pattern 5: Multi-channel ingress (optional cloud relay)
OpenClaw connects to WhatsApp/Telegram/Slack directly from the assistant.

**Our equivalent:** Pi exposes ONLY a local HTTP/WebSocket API.
A separate optional service (cloud or LAN) translates external channels
to that API. Pi never holds tokens for WhatsApp/Telegram/etc.

**Win:** Pi stays simple, secure, and offline-capable. Channels are
swappable at the relay without touching core.

---

## 2. What we are explicitly NOT taking

| OpenClaw feature | Why we skip it |
|------------------|----------------|
| Voice wake word | Wakeword detection on Pi 5 = 100MB+ RAM. V3+. |
| Live Canvas (visual A2UI) | Out of scope for backend engine. |
| Mobile companion apps | Pi is the brain; phones consume API. |
| Browser control via CDP | Defer to V2. Tool can be added later. |
| LLM-written scripts | Security hole. We use compiled tools only. |
| Workspace file watching | We use graph queries; no file watcher needed. |
| Direct WhatsApp/Telegram connections | Use cloud relay pattern instead. |
| Always-on TS/Node Gateway | Pi runs single Rust binary. |

---

## 3. Detailed component design — The "Nervous System"

### 3.1 Tool registry as graph

```
[ToolRegistry root synset]
        │
        │ ContainsTool
        ▼
[Tool: graph_query]
   ├── RequiresTier → [Tier 0]
   ├── HasParameter → [Param: "synset_id" of type SynsetId]
   ├── HasParameter → [Param: "depth" of type u8]
   ├── ReturnsType  → [Type: "Vec<Edge>"]
   └── ImplementedBy → [RustFn id 1]

[Tool: read_file]
   ├── RequiresTier → [Tier 2]
   ├── HasParameter → [Param: "path" of type PathBuf]
   ├── HasConstraint → [Constraint: path under /home/user/]
   └── ImplementedBy → [RustFn id 12]

[Tool: http_fetch]
   ├── RequiresTier → [Tier 3]
   ├── HasParameter → [Param: "url" of type Url]
   ├── HasConstraint → [Constraint: domain in allowlist]
   ├── HasConstraint → [Constraint: rate <= 10/min]
   └── ImplementedBy → [RustFn id 27]
```

The tool registry is just synsets and edges. New tools added by:
1. Implementing the Rust function (compiled in)
2. Adding tool synset + edges to a TSV file at build time

### 3.2 Tool dispatch flow

```rust
pub fn dispatch_tool(
    graph: &mut Graph,
    session: SessionId,
    tool_name: &str,
    params: &[Value],
) -> Result<ToolResult, ToolError> {
    // 1. Find tool synset
    let tool = graph.lookup_tool(tool_name)
        .ok_or(ToolError::UnknownTool)?;

    // 2. Check permission
    let session_tier = graph.session_tier(session);
    let required_tier = graph.tool_required_tier(tool);
    if session_tier < required_tier {
        graph.log_audit(session, AuditEvent::PermissionDenied(tool));
        return Err(ToolError::Forbidden);
    }

    // 3. Check constraints
    for constraint in graph.tool_constraints(tool) {
        constraint.check(params)?;
    }

    // 4. Resolve and call Rust function
    let fn_id = graph.tool_implementation(tool);
    let result = TOOL_FUNCTIONS[fn_id](params)?;

    // 5. Log to audit graph
    graph.log_audit(session, AuditEvent::ToolCalled {
        tool, params: params.to_vec(), result_summary: result.summary()
    });

    Ok(result)
}
```

This is ~50 lines of code. No LLM involved in the dispatch itself.
The LLM (or template engine) decides WHICH tool to call. The dispatcher
enforces the rules.

### 3.3 Session model

```rust
pub struct SessionId(u64);

// All session state stored as graph synsets/edges:
// - SessionId is just the synset ID
// - StartedAt, EndedAt are edges to time synsets
// - Each turn is a child synset
// - Memory references are edges to fact synsets

pub fn create_session(
    graph: &mut Graph,
    user: UserId,
    agent: AgentId,
    channel: ChannelId,
    granted_tier: PermissionTier,
) -> SessionId {
    let id = graph.allocate_session_synset();
    graph.add_edge(id, Relation::StartedBy, user);
    graph.add_edge(id, Relation::BelongsToAgent, agent);
    graph.add_edge(id, Relation::OnChannel, channel);
    graph.add_edge(id, Relation::GrantedTier, granted_tier.into());
    graph.add_edge(id, Relation::StartedAt, current_time_synset());
    SessionId(id.0)
}

pub fn session_recall(
    graph: &Graph,
    session: SessionId,
    topic_synset: SynsetId,
) -> Vec<Turn> {
    // Walk: session → ContainsTurn → Turn → DiscussedTopic → topic
    // Return turns where topic was discussed
    graph.walk_session_for_topic(session, topic_synset)
}
```

**Memory persistence is automatic.** When the system restarts and reloads
the graph, all sessions are right where they were.

### 3.4 Background scheduler

```rust
pub struct Scheduler {
    // Polls graph for due Schedule synsets every tick
}

impl Scheduler {
    pub fn tick(&mut self, graph: &mut Graph) {
        let now = current_time_synset();
        let due = graph.schedules_due_at(now);

        for schedule in due {
            let procedure = graph.schedule_procedure(schedule);
            let session = graph.system_session_for(schedule);

            // Run procedure as if it were a query
            let result = self.execute_procedure(graph, session, procedure);

            // Log result back to graph
            graph.add_edge(schedule, Relation::ProducedResult, result);

            // Reschedule if recurring
            if let Some(next) = graph.schedule_next_run(schedule) {
                graph.update_edge(schedule, Relation::NextRunAt, next);
            }
        }
    }
}
```

**~40 lines of code.** Deterministic. State persists in graph.

### 3.5 Cloud relay pattern (the "ingress" plane)

```
┌─────────────────────────────┐
│  External user on WhatsApp  │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│   Tiny Cloud Relay (~100MB) │
│   - Holds Telegram tokens   │
│   - Translates message in   │
│   - Forwards to Pi via HTTPS│
│   - Returns response        │
└──────────────┬──────────────┘
               │
               ▼  HTTPS over Tailscale or signed JWT
┌─────────────────────────────┐
│   Pi 5 ZETS Instance         │
│   - Receives JSON request    │
│   - Looks up user session    │
│   - Runs cognitive loop      │
│   - Returns JSON response    │
└─────────────────────────────┘
```

The relay is **stateless** (or near-stateless). All real state is on Pi.
Multiple relays can serve same Pi (redundancy). Relay can be on Cloudflare
Workers, Oracle Cloud Free Tier, or even on user's home router.

**~200 lines of Node/Bun code per channel adapter.** Optional.

---

## 4. Distinction from OpenClaw, technically

| Concern | OpenClaw approach | ZETS approach |
|---------|-------------------|---------------|
| Where does config live? | TS code + .md files | Graph synsets |
| Who interprets skills? | LLM at runtime | Compiled Rust functions |
| Memory storage | External DB or files | Graph nodes/edges |
| Tool security | Sandbox + approval | Tier check (graph) + constraints |
| Background tasks | Node.js scheduler | Graph-driven daemon (Rust) |
| Channel adapters | First-class in Gateway | Optional cloud relay |
| RAM footprint | 200-500MB+ on dev box | ~30MB target on Pi |
| LLM dependency | Required for routing | Required only for composition |
| Recovery from crash | Re-init from disk + replay | Reload graph file (atomic) |

---

## 5. Implementation plan (3 sprints, after AGI-Lite Sprint A-F complete)

This is **post-AGI-lite** work. The Nervous System is built ON TOP of
the deliberation engine, not before it.

### Sprint G (1 week): Tool Registry + Permission Gates
- Define `Tool` and `PermissionTier` synset types
- Build `dispatch_tool` function
- Implement first 5 tools: graph_query, format_response, log_event,
  add_memory, current_time
- Tests: permission denied for low tier, correct dispatch for high tier

### Sprint H (1 week): Session Management + Background Scheduler
- Define `Session` synset structure
- Implement create_session, session_recall, end_session
- Implement Scheduler with 1-second tick
- First scheduled task: midnight memory consolidation
- Tests: session persists across restart, scheduled task runs

### Sprint I (1 week): Cloud Relay Reference Implementation
- Tiny Bun/Node script for Telegram relay
- Pi-side HTTP endpoint
- End-to-end test: Telegram message → Pi → response back
- Documentation for adding more channels

### Optional Sprint J: Additional tool families
- file_read (Tier 2)
- http_fetch (Tier 3)
- shell_run (Tier 4, very restricted whitelist)
- All implemented as graph entries + Rust functions

---

## 6. Open questions for Idan

1. **Sprint order:** Build Nervous System BEFORE Composition Engine (Sprint E),
   or AFTER? My recommendation: AFTER. Composition needs to exist first
   so we have something to dispatch tool calls FOR.

2. **Cloud relay tech:** Bun (fast TypeScript)? Cloudflare Workers (cheapest)?
   Plain Node (most familiar)? My pick: **Cloudflare Workers** — free tier
   covers personal use, no server to maintain.

3. **Tool security default:** Default tier for new sessions = Tier 0 (read-only)?
   Or Tier 1 (safe writes)? My pick: **Tier 0**, requires explicit grant.

4. **Audit retention:** All tool calls logged to graph forever?
   Or rotated after N days? My pick: **forever for high tiers,
   rotate Tier 0-1 after 30 days**.

5. **Multi-agent within Pi:** Allow 2+ Agent synsets? Or one agent per Pi?
   My pick: **multiple allowed**, but each user-channel pair maps to one.

---

## 7. The honest assessment

**What this Nervous System gives us:** ZETS can take actions, not just answer
questions. With the same determinism, same Pi-friendliness, same graph-native
storage philosophy.

**What it does NOT give us:** OpenClaw's polished UX, multi-channel adapters
out of the box, or "magic" of LLM-written skills. Those are products. We're
building infrastructure.

**Is this AGI-like?** It is closer. AGI-like means: stores knowledge,
reasons over it, takes actions, learns from feedback, runs anywhere. The
graph + deliberation gives us the first three. The Nervous System gives us
action. Feedback Learner (Sprint F) gives us learning.

After all of these ship: yes, this system exhibits AGI-like behavior on
Pi 5 hardware. But it is not AGI. It is a deterministic symbolic engine
that approaches AGI behaviors within a bounded domain.

---

## 8. Final ask

I want to ship the AGI-Lite (Sprints A–F) first, then this Nervous System
on top (Sprints G–I). Approve this order, and I can prepare task-cards in
git for Claude Code to pick up sequentially.

---

**End of Nervous System spec.**
