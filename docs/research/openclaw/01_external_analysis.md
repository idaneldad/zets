# OpenClaw Research — Source Material (Reference Only)

**Date saved:** 21.04.2026
**Status:** Reference documents from external sources. NOT ZETS architecture decisions.
**Purpose:** Capture analysis from Idan's external advisors (Perplexity + Gemini) for reference.
This material informed the ZETS Nervous System spec (see `OPENCLAW_INTEGRATION.md`)
but is not itself authoritative. Treat as input, not conclusion.

---

## Document 1 — Perplexity-style analysis (passed by Idan)

### What OpenClaw is

OpenClaw is a personal AI assistant orchestration runtime, MIT-licensed, written
in TypeScript/Node. It is NOT a model itself; it is a layer that gives LLMs
a "body" — connections to tools, the user's machine, communication channels,
memory, and background tasks.

Instead of being a chatbot in one window, it is a persistent personal agent
that can receive a Telegram message, work on your computer, read/write files,
run commands, browse the web, manage tasks, and return a result.

### What it can do (from openclaw/openclaw README)

- Runs on macOS, Linux, Windows/WSL2
- Connects to many messaging channels: WhatsApp, Telegram, Slack, Discord,
  Google Chat, Signal, iMessage, Microsoft Teams, Matrix, WebChat, more
- Persistent memory across sessions
- Browser control via CDP
- Full system access OR sandboxed mode
- Extensible via skills + plugins
- Multi-agent routing (multiple agents in one Gateway)
- Live Canvas for visual work
- Voice wake/talk mode
- Companion apps for macOS/iOS/Android
- Central Gateway (control plane on `ws://127.0.0.1:18789`) managing
  sessions, tools, channels, events

### Architecture insight

The key architectural insight: it is built around a local Gateway as control
plane, with agent sessions, tools, channels, and a workspace layered above.

The workspace contains files like:
- `AGENTS.md` — agent definitions
- `SOUL.md` — personality / identity
- `TOOLS.md` — available tools
- `~/.openclaw/workspace/skills/<skill>/SKILL.md` — per-skill instructions

So the agent's behavior is defined not just through a momentary prompt, but
through a persistent, structured workspace.

### Why it feels like "talking to other AIs"

It does not literally talk to other AIs via some magical protocol. It routes
between models, sessions, channels, and external tools — including using
other agents/interfaces via APIs, subscriptions, Codex/Claude-style sessions,
or skills written for it. So the feeling is "talking with other AIs", but in
practice the strength is orchestration of many interfaces under one
persistent assistant.

### What patterns are most relevant for DINIO/ZETS

- **Persistent assistant runtime** instead of stateless request/response
- **Workspace + prompt files + skills** as live files, not just DB/config
- **Multi-channel control plane** — same agent reachable from Telegram, CLI, UI
- **Session isolation / multi-agent routing** — separate agent per context,
  customer, project, or task family
- **Tool permission model + sandbox tiers** — main session with full access,
  non-main with sandbox
- **Background automation** — cron, heartbeats, reminders, watchdogs

### What probably should NOT be taken

OpenClaw is a broad "personal OS" product, while ZETS is a lean modular engine.
So it's not certain it makes sense to adopt the entire scope: voice wake,
consumer messaging sprawl, canvas, mobile nodes, dozens of channel
integrations — these could overload a lean B2B system.

If the goal is one-person dev company with vertical instances, more sensible
to take the runtime, skill system, session orchestration, and tool bus,
not the entire consumer assistant layer.

### OpenClaw vs Open Interpreter

| Dimension | OpenClaw | Open Interpreter |
|-----------|----------|------------------|
| Goal | Personal assistant platform | Computer/code execution agent |
| Primary interface | Messaging channels + gateway + apps | Terminal / Python API |
| Memory | Persistent across sessions | Session/history-driven |
| Proactivity | High (heartbeat/cron/always-on) | Low (reactive only) |
| Tools | Many tool families, routing, canvas, nodes, browser | Code exec, shell, browser, data work |
| System complexity | Higher | Lower, simpler |

**Engineering conclusion:** OpenClaw is the orchestrator. Open Interpreter is
the executor. If building your own system, take the assistant/runtime/session
model from OpenClaw, take direct code execution with clear simple approval
from Open Interpreter.

---

## Document 2 — Gemini-style analysis (passed by Idan)

### The "Frankenstein system" problem

The Perplexity analysis suggests "take orchestration from OpenClaw and
execution from Open Interpreter." This is a recipe for a heavy, bloated,
fragile system.

**Why OpenClaw will hurt you on Pi:**
- Built around always-running Gateway connected to WhatsApp/Telegram/Slack
- Means always-open network connections, polling, webhook handling, complex
  state management
- On Raspberry Pi, becomes a memory and network management nightmare

**Why Open Interpreter is dangerous:**
- Letting an LLM write Python scripts and run them on edge hardware = security
  hole + uncontrolled resource consumption
- Pi will crash when bad script makes infinite loop and eats all RAM

### Architectural recommendation for ZETS (the clean solution)

Your system is different: you already have a fast internal "brain" (the
language-agnostic ZETS graph). We don't need an LLM to write Python code to
know what time it is or to fetch a fact.

**Recommendation:** Don't adopt either tool as-is. Extract their patterns
and implement directly inside the ZETS graph.

#### 1. Runtime: sessions and memory (take from OpenClaw)

OpenClaw is right about one thing: a real assistant needs persistent context.

**How to implement in ZETS:** Not via heavy background server, but via
nodes in the graph. Each conversation with a user is a `Session` node. Each
`Session` has edges to other nodes ("we discussed X", "the goal is Y").
Persistent memory IS the graph itself. It stores history with zero effort.

#### 2. Skills and tools (the tool bus)

Instead of `SKILL.md` files (like OpenClaw) or arbitrary Python execution
(like Open Interpreter), define tools as **pre-compiled Rust functions**
inside our binary.

**How it works:** The LLM analyzes the request. If an external action is
needed (e.g., search a file), it doesn't run code, it returns to ZETS an
action code (e.g., `ExecuteTool(42)`). Rust translates 42 to a safe fast
search function and returns the result to the graph.

**The advantage:** 100% security (built-in sandboxing), 0 unnecessary memory
allocations, enormous speed.

#### 3. Command ingress

OpenClaw connects to Telegram/WhatsApp. Convenient but cumbersome for edge.

**Recommendation:** Your Pi should expose only a simple lean API (e.g., local
HTTP/WebSockets). If you want the bot to answer in Telegram, put a light
server in the cloud (very cheap server) that translates Telegram messages
and forwards them cleanly to the Pi. Don't burden third-party communication
management on the edge device.

### Action plan summary

1. **Don't copy platforms:** OpenClaw and Open Interpreter are systems trying
   to compensate for not having a persistent knowledge graph. You have one.
2. **The graph is the operating system:** All management ideas (sessions,
   goals, permissions) encode directly into your agnostic edge and node tree.
3. **Limited and safe execution:** Tools should be built into Rust code
   (Compiled Tools), not scripts the LLM invents at runtime.

---

## Notes on source reliability

- These analyses are by Perplexity and Gemini, paraphrasing OpenClaw's
  README + websites. Treat all claims as needing verification against
  github.com/openclaw/openclaw directly.
- web_search returned conflicting metadata about OpenClaw: some sources
  cite specific events (ClawCon, OpenAI backing) that I cannot verify
  independently. The repo and architecture description are real; the
  marketing/PR layer around it should be discounted.
- The architectural patterns described (Gateway, workspace files, skills,
  sessions) ARE in the public repo. The recommendation framework above
  (patterns yes, wholesale import no) is sound.

---

**End of reference document.**
