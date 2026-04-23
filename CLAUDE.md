# CLAUDE.md — Entry Point for AI Agents Working on ZETS

> **If you are a Claude (or any AI agent) arriving on this repo — read this file first, then [`STATUS.md`](./STATUS.md), then proceed.**
> This file changes rarely. `STATUS.md` is the live state.

---

## 1. Who owns this repo

**Idan Eldad** (עידן אלדד) — solo founder.
Works primarily in Hebrew. Runs CHOOZ (B2B corporate gifting), DINIO (ops platform),
and ZETS/LEV (the cognitive kernel — this repo).

### How Idan wants you to work

1. **Flow over permission.** Don't ask — decide. If you're wrong, he corrects.
2. **Council of 5 experts.** For every planning decision, mentally recruit 5 top experts
   in the relevant domain and reason through their perspectives before choosing.
3. **Break assumptions out loud.** Before acting, list what you think you know and
   challenge each item. Pattern-matching on familiarity is the #1 failure mode.
4. **Be honest about what you can and cannot do in this session.** No theater.
5. **No scope creep.** If a problem is bigger than one session, say so and ship the
   vertical slice. Cortex already burned us on this — don't repeat.

---

## 2. The ZETS core principle (non-negotiable)

> **"Learning is in the code. *What* to learn and *how* — is in the graph."**
> (Idan's own words, 23.04.2026)

This splits the system cleanly:

| Layer | What lives there | Language |
|---|---|---|
| **Code** | 7 primitives: `fetch`, `parse`, `tokenize`, `store`, `retrieve`, `reason`, `communicate` | Rust |
| **Graph** | Everything else: procedure atoms, knowledge atoms, motivation atoms | TOML / binary atoms |
| **Seed** | Initial identity (DNA): who am I, what do I want, what's safe | YAML |

**Implication:** If you catch yourself adding a capability as Rust code when it could
be a procedure atom — **stop**. That's the wrong layer. Math, language learning,
dialogue style, curiosity — all of these are graph content, not Rust modules.

**Corollary — no code duplication (Idan, 23.04.2026):** If two pieces of code
express the same concept, that concept is a missing atom in the graph. Don't
deduplicate by extracting a shared Rust function — deduplicate by lifting the
concept to the graph and having both call-sites walk to the same atom.
Duplication in `src/*.rs` is a graph-gap indicator. Fix the graph, not the code.

**The north star:** ZETS should do autonomously everything Claude currently does for it.
Every action a Claude takes on behalf of ZETS is a gap — log it, then move it to the graph.
The [`CLAUDE_ACTIONS_AUDIT.md`](./docs/CLAUDE_ACTIONS_AUDIT.md) tracks this explicitly.

---

## 3. Read order for a new session

```yaml
# machine-readable for agents
read_order:
  1: CLAUDE.md           # this file — principles
  2: STATUS.md           # current state (what was done, what's next)
  3: docs/VISION_VS_REALITY.md    # what's dreamed vs what's built
  4: docs/CLAUDE_ACTIONS_AUDIT.md # capabilities still relying on Claude
  5: docs/DECISIONS_LOG.md        # why we made the choices we made
  6: docs/ZETS_LEARNING_HOW_IT_LEARNS.md  # the architecture decision
  7: docs/ZETSON_AGENT_MISSIONS.md        # parallel agent work queue
  8: AGI_ROADMAP.md                       # phased roadmap with current state
  9: README.md                    # public-facing summary
```

Everything else in `docs/` is reference. `docs/_archive/` is historical — do not read
unless specifically investigating a past decision.

---

## 4. Working in parallel / across sessions

If multiple Claude sessions run at once (different windows, Claude Code agents, etc.):

- **Every session appends to `STATUS.md`** under the `## Session log` section
  with: timestamp, scope of work, branch, status (active / done / abandoned).
- **One session = one branch**, unless the work is read-only.
  Naming: `claude/<yyyy-mm-dd>-<short-scope>` (e.g. `claude/2026-04-23-audit-restructure`).
- **Never push directly to `main`** without Idan's explicit approval in chat.
- Before starting work, scan `STATUS.md` for active sessions to avoid collision.

When a session ends (naturally or because the chat hit limits):
1. Update `STATUS.md` — what was done, what's next, what's blocked.
2. Update `docs/DECISIONS_LOG.md` if a decision was made.
3. If work was abandoned / a path was deleted, append to the deletion log (same file).
4. Commit with a descriptive message and push to `main` (Idan has given
   Claude-with-Cortex-MCP push access; Claude-in-browser-chat does not push).

---

## 5. Claude's access model (know which Claude you are)

This repo is worked on by three distinct Claudes, with different privileges:

| Surface | Has shell on ddev? | Has Claude Code CLI? | Can push? |
|---|:-:|:-:|:-:|
| Claude in Anthropic chat (browser) | ✅ via Cortex MCP | ✅ via `claude -p` on server | ✅ |
| Claude Code CLI (run on ddev) | ✅ (it's running there) | ✅ (it IS that) | ✅ |
| Claude with no MCP | ❌ | ❌ | ❌ (patch-only) |

If you're the third kind — you produce patches, not pushes, and you say so.

---

## 6. Things that are NOT allowed, ever

Matching Anthropic's content policies + Idan's specific lines:

- No hallucinated benchmark numbers. If a number isn't verified, label it `[claim]` not `[measured]`.
- No deleting `STATUS.md` history — append only.
- No LLM calls from *inside* ZETS code. ZETS is LLM-free by design.
  (Claude-the-assistant calls itself via API is a *separate* thing — that's for CHOOZ/DINIO tools,
  not for ZETS runtime.)
- No creating new top-level dirs without noting it in `DECISIONS_LOG.md`.
- No adding a capability as Rust code when it could be a procedure atom (see §2).

---

## 7. Quick facts (keep updated but rarely change)

- **Language:** Rust (core). Python (mcp/ tooling + testing helpers only).
- **License / openness:** Not yet decided. Treat as private for now.
- **Primary branch:** `main`.
- **Deployment target:** `ddev.chooz.co.il` — Idan's dev server. Repo at `/home/dinio/zets`.
- **Council invocation:** If a decision is weighty, write the council's reasoning
  into `docs/DECISIONS_LOG.md` before committing. This is the receipts trail.

---

## 8. Memory hooks (for Claude's memory system)

If you (Claude) have memory of past ZETS work, the following pointers take precedence
over your memory — your memory may be stale:

- Current state → `STATUS.md`
- What we've deleted / abandoned → `docs/DECISIONS_LOG.md` (bottom section)
- What's still done-by-Claude-not-ZETS → `docs/CLAUDE_ACTIONS_AUDIT.md`
- What's dreamed vs built → `docs/VISION_VS_REALITY.md`
- Phased technical plan with status → `AGI_ROADMAP.md`

---

*Last updated: 23.04.2026 · Maintained by: whoever touched the repo last · Single source of truth for agent onboarding.*
