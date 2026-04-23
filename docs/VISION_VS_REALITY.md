# VISION_VS_REALITY.md

> **The vision is big. This file keeps it honest.**
> Every row: what we dream, what we actually have, where the work lives.

Updated: 23.04.2026.

---

## 📜 The vision in one paragraph

> One deterministic cognitive kernel. One binary. From a $3 microcontroller to a
> data-center Xeon. Offline-first. No LLM inside. Hallucination-free. Every answer
> is traceable to a source. Multilingual (starting with HE + EN, then AR, then the
> Romance/Latin/CJK ladders). Learns autonomously from the open web — dictionaries,
> encyclopedias, Wikipedia — without an LLM teacher. Understands images, speech,
> and video by decomposition, not by outsourcing to an LLM. Multiple instances
> talk to each other as a family. Each instance carries its own DNA (a seed)
> and its own mission. Intelligence as a commodity you own, not a subscription
> you rent. **Doubles as a dense, fast, provenance-preserving graph database —
> so a customer's data can always be reconstructed.**

Full source: `docs/PRODUCT.md`, `docs/INVESTOR_BRIEF.md`, `AGI_ROADMAP.md`.

---

## 🗺️ Vision → Status map

**Legend:**
- `✅ built` — code + tests exist, reproducible today
- `🟡 partial` — exists but thin, incomplete, or needs work
- `📝 spec-only` — design written, no code
- `🔬 research` — open problem, will take weeks+ of focused work
- `❌ missing` — not even a plan

Column "Layer": where the work belongs per the core principle.
- `Rust` — one of the 7 primitives
- `Graph` — procedure / knowledge / motivation atoms
- `Mixed` — needs both
- `Infra` — tooling (tests, observability, deploy)

---

### Cognition & reasoning

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Deterministic lookup (concept → gloss) | ✅ built | Rust+Graph | `src/fold/`, 144K concepts loaded |
| Morphology (HE, EN, AR, +12 langs) | ✅ built | Rust+Graph | `src/morphology/` |
| `is_ancestor(a, b, depth)` multi-hop | ✅ built | Graph (bytecode) | `src/system_graph/reasoning.rs` |
| Metacognition — gap detection | ✅ built | Graph | `src/metacognition.rs`, 7 tests |
| Capability runtime | ✅ built | Rust | `src/capability_runtime/` |
| Calibration harness | ✅ built | Infra | `src/benchmark/calibration/` |
| Canonization (variants → canonical) | ✅ built | Graph | `src/canonization/` |
| Preference store | ✅ built | Graph | `src/preferences/` |
| `is_descendant`, `common_ancestor` (LCA) | ❌ missing | Graph (bytecode) | **needs ~100 bytes each — easy win** |
| `causal_chain` (traverse CAUSES) | ❌ missing | Graph (bytecode) | AGI_ROADMAP §phase-1 |
| `part_of_path` (traverse PART_OF) | ❌ missing | Graph (bytecode) | AGI_ROADMAP §phase-1 |
| `contradiction_detect` | ❌ missing | Graph (bytecode) | AGI_ROADMAP §phase-1 |
| Temporal reasoning (before/after/duration) | 🔬 research | Graph | open problem — model choice matters |
| Theory of mind | 🔬 research | Graph | open problem — needs user-model atoms |
| Active learning loop (MetaCog → Sandbox) | 🟡 partial | Graph | detection works, actuation doesn't |

---

### Knowledge & data

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Dictionary breadth (multi-lang) | ✅ built | Data | 144K concepts, 16 langs |
| Wiktionary bootstrap | 🟡 partial | Data | HE batch loaded, others pending |
| Wikipedia autonomous ingest | 📝 spec-only | Mixed | `docs/AUTONOMOUS_WIKIPEDIA_SPEC.md` |
| ConceptNet import → causal edges | ❌ missing | Data | AGI_ROADMAP §phase-2 |
| Wikidata import → instance_of, subclass_of | ❌ missing | Data | AGI_ROADMAP §phase-2 |
| Average edges per concept ≥3 | ❌ missing | Data | today: <1 semantic edge per concept |
| Cross-lingual sense links (synset alignment) | 🟡 partial | Data | some linking, not systematic |
| Source provenance on every atom | ✅ built | Graph | `src/learning_layer.rs` has provenance |
| Trust weighting per source | ✅ built | Graph | recalibrates automatically |
| **Client-data recovery (dense DB role)** | 📝 spec-only | Mixed | **new as of 23.04.2026 — mission P-P** |

---

### Language I/O

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Natural-language query parser | ❌ missing | Graph (procedure atoms) | AGI_ROADMAP §phase-3 — **goes in graph, not Rust** |
| Natural-language answer generator | ❌ missing | Graph (procedure atoms) | must use morphology from Rust layer |
| Dialogue session management | 🟡 partial | Rust | `src/conversation/`, `src/dialogue.rs` — shell only |
| Style/register (formal vs casual) | 🟡 partial | Graph | `src/reader/style.rs` exists |
| Emotion awareness in replies | 🟡 partial | Graph | `src/reader/emotion.rs` exists |

---

### Autonomous learning (the Zetson arc)

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| 7 Rust primitives defined | ✅ built (6/7) | Rust | `fetch` is the missing one (P-A mission) |
| Primitive: HTTP fetch + robots.txt + rate limits | 📝 spec-only | Rust | mission **P-A** |
| Primitive: HTML parser (Wikipedia-grade) | 📝 spec-only | Rust | mission **P-B** |
| Procedure loader (TOML → atoms) | 📝 spec-only | Rust | mission **P-C** |
| 20 initial procedure atoms | 📝 spec-only | Graph | mission **P-C** (content) |
| Learning loop executor (DAG walker) | 📝 spec-only | Rust | mission **P-D** |
| Seed loader (YAML → atoms injection) | 📝 spec-only | Rust | mission **P-E** |
| Observability dashboard | 📝 spec-only | Infra | mission **P-F** |
| Zetson infant binary running | 📝 spec-only | Mixed | mission **P-G** (integration) |
| 500+ procedures learned from scratch | 🔬 research | Graph | **90-day success criterion** |
| 20K+ HE atoms, 20K+ EN atoms from scratch | 🔬 research | Data | **90-day success criterion** |
| 5K+ cross-lingual links learned | 🔬 research | Data | **90-day success criterion** |

---

### Multi-modal learning (new, 23.04.2026)

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Image understanding without LLM | 📝 spec-only | Mixed | **mission P-N** — feature extraction via scikit-image / OpenCV (Python side), atoms on graph side |
| Speech → tokens without LLM | 📝 spec-only | Mixed | **mission P-O** — Whisper-local (offline), Vosk, or CMU Sphinx as primitives; phonetic atoms on graph |
| Video understanding | 📝 spec-only | Mixed | **mission P-Q** — keyframe extraction + image procedures + temporal atoms |
| Audio classification (music vs speech vs env) | 📝 spec-only | Mixed | part of P-O — classic DSP features (MFCC, spectral centroid) |

**Principle confirmed with the council (23.04.2026, session #2):** All four of these
lean on Rust/Python primitives (existing CV/DSP libraries, no LLMs) for feature
extraction, and the *meaning* of the features lives in graph atoms. Example:
"edge-histogram is a signature" is an atom; "this histogram → concept 'cat'" is a
learned edge, not a hard-coded rule.

---

### Deployment & networking

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Single binary, zero runtime deps | ✅ built | Rust | stdlib-only policy holds |
| Offline-first | ✅ built | Rust | no runtime network by default |
| Runs on microcontroller ($3 hardware) | 📝 spec-only | Rust | no tests on actual MCU yet |
| Encryption (AES-256-GCM) | 🟡 partial | Rust | works; key derivation needs Argon2 |
| Multiple instances talking as a family | 📝 spec-only | Mixed | mentioned in PRODUCT.md — not built |
| Master → Family → Client → Leaf hierarchy | 📝 spec-only | Mixed | conceptual only |
| MCP server (for Claude integration) | ✅ built | Infra | `mcp/zets_mcp_server.py` |

---

### Dense storage / "graph as database" role

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Dense atom encoding | ✅ built | Rust | `src/pack.rs`, `src/mmap_core.rs` |
| BitFlag edge compression | ✅ built | Rust | `src/bitflag_edge.rs` (10 bytes/edge) |
| Fold / BPE vocabulary compression | ✅ built | Rust | `src/fold/` |
| Crash-safe WAL | ✅ built | Rust | `src/wal.rs` |
| Per-client reconstruction (any atom → full history) | 📝 spec-only | Graph | **mission P-P** |
| Per-concept provenance chain | 🟡 partial | Graph | `src/learning_layer.rs` stores it; no reconstruction API |

---

### Benchmarks (gap to LLMs)

| Vision | Status | Layer | Where it lives / goes |
|---|---|---|---|
| Internal benchmark: 32-question calibration | ✅ built | Infra | `data/benchmarks/zets_expanded_32q_v1.jsonl` |
| Internal benchmark: 20-question baseline | ✅ built | Infra | `data/benchmarks/zets_baseline_20q_v1.jsonl` |
| MMLU-style general knowledge eval | ❌ missing | Infra | **priority: wire up after P-series lands** |
| HumanEval-style code completion | ❌ missing | Infra | very far — requires code understanding atoms |
| GPQA-style graduate QA | ❌ missing | Infra | same |
| MATH-style math reasoning | ❌ missing | Graph | math lives as procedure atoms per core principle |
| Honest leaderboard vs GPT/Claude/Gemini | ❌ missing | Infra | **mission P-K** |

---

## 🔍 Where the biggest leverage is (PM hat on)

If we rank by `impact × (1 / effort)`:

1. **Code quality audit (P-M)** — unblocks trust in everything else's measurements.
   **Effort: hours (Claude Code). Impact: high — stops drift.**
2. **Add 4 bytecode routes** (LCA, causal_chain, part_of_path, contradiction_detect)
   — ~400 bytes of bytecode total. Unlocks "Why?", "What's similar?", "What's X made of?"
   **Effort: days. Impact: unlocks phase-2 usage.**
3. **Populate IS_A edges from existing Wiktionary glosses**
   — ~10K concepts, scan glosses for "X is a Y" pattern. Today average edges per
   concept is <1, target is ≥3. **Effort: days. Impact: graph stops being a dictionary.**
4. **Run P-A and P-B** via Claude Code — unblocks the entire Zetson arc.
   **Effort: 2–3h wall-clock. Impact: lets autonomous story start.**
5. **Write the LLM-benchmark comparison table** (MMLU / HumanEval / MATH)
   — not to run them yet, but to commit to honest metrics before self-deceiving.
   **Effort: one session. Impact: keeps us from false claims.**

---

## ⚠️ Claims to stop making until proven

| Claim currently used in public docs | Why it's risky | What to do |
|---|---|---|
| "HumannessScore 0.48" | No published spec for the metric | Either spec the metric or drop the number |
| "2.6 MB RAM" | No reproducible measurement protocol in repo | Add `benches/memory.rs`; document methodology |
| "80.8 µs latency" | Unclear which op, which hardware, which build flags | Add `benches/latency.rs`; report P50/P95/P99 |
| "1,278 tests passing" | Likely true, but should be a CI badge not a markdown number | Add CI; link the badge |
| "AGI readiness" framing | The gap list in AGI_ROADMAP contradicts it | Reframe as "deterministic reasoning kernel" in investor materials |

We keep the ambitious prose. We stop putting [claimed] numbers in investor brief
without the `[claimed]` tag.

---

## 📌 Update rules

- When a row moves status (e.g. `📝 spec-only` → `🟡 partial`), add a one-line
  entry to `docs/DECISIONS_LOG.md` with date + what changed.
- When a new vision item appears (e.g. Idan says "we also want Y"), add a row.
  Don't quietly fold it into existing rows.
- When a claimed number is verified, move it from the "claims to stop making"
  table into a reproducible benchmark file and delete the row here.

---

*Source of truth when vision and reality disagree: **reality**. If a public doc claims something not reflected here, the public doc is behind and should be updated.*
