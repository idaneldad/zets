# שבירת כלים — Forensic Audit of ZETS
## Evidence-based, not claim-based

**Date:** 2026-04-24 (evening session)
**Triggered by:** Idan's directive: "אסור להסתמך על זה כי אולי עשינו טעויות.
צריך רק ללמוד ממה שעבד ומה שלא עבד."

**Method:** Every finding below verified by running commands, not reading documents.

---

## What I Verified Actually Works

These are confirmed by running code, not by reading claims:

| Finding | Evidence |
|---|---|
| `cargo build --release` succeeds | 23.11s, 18 warnings, 0 errors |
| `cargo test --lib` passes 1,301 tests | 0 failed, 0 ignored |
| `autonomous-demo` binary runs | Full pipeline: install→ingest→walk→dream→persist |
| 119 bootstrap atoms load from encrypted installer | Observed in demo output |
| Autonomous text ingestion functional | 40 new atoms, 137 new edges from 308 chars of input |
| smart_walk with mode selection works | Chose "narrative" mode, returned ranked candidates |
| Dreaming proposes+accepts edges | 5 proposed, 3 accepted, 2 rejected |
| Persistence integrity perfect | Save→reload cycle verified byte-for-byte |
| 144K concepts loaded in `data/` | Disk usage: 22 GB |
| Morphology for HE/EN/AR | Tests in `src/morphology/` pass |
| Metacognition (gap detection) | 7 tests in `src/metacognition.rs` pass |

**These are the real achievements. They are solid foundations.**

---

## What I Verified Does NOT Work (or is Broken)

| Finding | Evidence |
|---|---|
| Doctest in `src/hash_registry.rs` FAILS | Contains Unicode `→` breaking Rust lexer |
| **Test count claim is inflated** | Claimed "1,354", actual 1,301 (gap: 53 tests) |
| 637 `.unwrap()` calls in non-test code | Brittle — panics on any edge case |
| 17 TODO/FIXME markers in code | Incomplete implementations scattered |
| 18 compiler warnings | Dead code, unused imports/variables |
| Performance claims unverified | "2.6 MB RAM", "80.8 µs latency" — no reproducible methodology |
| "HumannessScore 0.48" | Metric spec unclear, no verification pipeline |

---

## The Most Serious Finding — Parallel Atom Systems

**Four independent implementations of the same concept coexist:**

```
System 1: src/atoms.rs                (547 lines, used by 29 binaries)
          pub type AtomId = u32;
          pub struct Atom { ... }
          pub struct AtomStore { ... }

System 2: src/graph_v4/types.rs       (1,603 lines in graph_v4/, used by 6 bins)
          pub type AtomId = u32;       // DUPLICATE TYPE NAME
          pub struct Atom { ... }       // DUPLICATE STRUCT NAME

System 3: src/piece_graph.rs          (used by ZetsEngine facade)
          pub type ConceptId = u32;
          pub struct PiecePool;
          pub struct ConceptNode;
          
System 4: src/mmap_core.rs            (used by ZetsEngine facade)
          pub struct MmapCore;
          pub struct MmapConcept<'a>;
```

**This violates Idan's own stated rule** (DECISIONS_LOG 2026-04-23):
> "No code duplication; duplication is a graph gap"

The `ZetsEngine` facade uses systems 3+4 (mmap_core + piece_graph).  
The `autonomous-demo` uses system 1 (atoms.rs).  
The `graph_v4` system exists in parallel, used by 6 newer bins.

**No single "source of truth" for what an atom IS in this codebase.**

---

## Abandoned Architecture History

**6 abandoned plans in `docs/_archive/`:**
- `20260422_agi_roadmap_V1.md`
- `20260422_architecture_plan_V1.md`
- `20260422_architecture_plan_V2.md`
- `20260422_bottleneck_master_V1.md`
- `10_agi_roadmap.md` (undated, older)
- `11_agi_lite_spec.md`

**Cognitive thrashing pattern:** 3 AGI roadmap versions in 3 days (April 22→24).
Every version was partially acted on, creating archaeological layers in `src/`.

---

## What I Wrote Today That Was Also Wrong

**My own contributions today need honest critique:**

### 1. `00_ZETS_MASTER_BLUEPRINT.md` (1,559 lines, written this morning)

**Problems:**
- Assumed greenfield build — ignored 67,635 lines of existing Rust
- "Phase 1: Build AtomHeader" — AtomHeader effectively already exists (in 4 places!)
- 20 principles listed theoretically, none mapped to real code verification
- 5 capabilities from AI council never checked against `sense_graph.rs`, `metacognition.rs`, etc.
- Roadmap "24 weeks starting from zero" — false premise

**Honest reclassification:** This document is **aspirational reference architecture**,
not a buildable plan. It should be labeled as such, not presented as "the path forward."

### 2. `20260424_blueprint_reality_reconciliation.md` (written this afternoon)

**Problems:**
- Better than Blueprint (acknowledged existing code) but still superficial
- "Recommendation: Option A + Option C in parallel" — but I never measured
  whether Option A (Zetson missions) is feasible given the parallel-atoms mess
- Implied Zetson can be built on top of `atoms.rs` without addressing which
  atom system it should use

### 3. The AI Masters' Council consultation

**Problems (on my end of the interaction):**
- Fed the masters the theoretical Blueprint, not the actual codebase
- Their advice (C1-C5 capabilities) was architecturally clean but disconnected
  from reality — they didn't know about the 4 parallel atom systems
- Their "teaching" was generic wisdom, not grounded in this project's debt

---

## The Mistakes to NOT Repeat

Crystalized from everything above:

### Mistake 1: Planning without grounding in code reality
- I wrote 1,559 lines of theory before I ran `autonomous-demo` once
- Future versions must start with audit, then plan

### Mistake 2: Allowing parallel systems to coexist
- Every time "v4" or "v5" is added without retiring older systems, the debt compounds
- Rule: Before creating new version, archive old — or merge, don't leave orphans

### Mistake 3: Trusting self-reported numbers
- "1,354 tests" was claim, 1,301 is reality (4% drift, small, but shows the pattern)
- Performance claims never verified
- Rule: Every number in docs must have a reproducible measurement command

### Mistake 4: Over-consultation with theorists (AIs or humans) before code-grounding
- Masters' Council added 94KB of good-sounding advice
- None of it touched the real bottleneck (parallel atom systems)
- Rule: Consult experts on concrete, code-grounded questions — not on architecture-in-the-abstract

### Mistake 5: `.unwrap()` as default
- 637 unwrap() calls = 637 potential panics in production
- For an AGI meant to run on edge devices, this is unacceptable
- Rule: Every `unwrap()` must be either (a) provably-never-panic or (b) replaced with proper error handling

### Mistake 6: Spec-only missions that never get built
- P-A through P-Q exist as documents
- 0 of them have been fully completed
- Rule: Don't spec mission N+1 until mission N has shipped and been verified

### Mistake 7: My "Phase 1-12 over 24 weeks" plan
- Arbitrary timeline, no grounding in team capacity, no verification milestones
- Rule: Don't plan beyond "the next verifiable increment"

---

## What Should Happen Now

### Step 1: Decide on the One Atom System

This is the biggest unresolved question in the codebase. Choices:
1. **Pick `atoms.rs`** (most binaries use it, but simpler model). Archive graph_v4 and piece_graph+mmap_core would need to be integrated or replaced.
2. **Pick `graph_v4`** (more recent, larger, 1,603 lines). Others retire.
3. **Pick `mmap_core` + `piece_graph` stack** (what ZetsEngine uses). Others retire.
4. **Design a 5th system that unifies them**. Worst option — adds to the debt.

**This decision must be made by Idan before any more code is written.**

### Step 2: Clean the Existing Code

Based on chosen atom system:
- Archive the others (move to `_legacy/` in Rust module tree)
- Fix the 1 failing doctest
- Audit 637 unwraps — make a priority list of which to fix

### Step 3: Reduce Architecture Documents to ONE

Currently exist as "source of truth":
- `docs/00_ZETS_MASTER_BLUEPRINT.md` (my theoretical one)
- `docs/ZETS_LEARNING_HOW_IT_LEARNS.md` (Idan's "7 primitives + graph")
- `AGI_ROADMAP.md` (third roadmap)
- `config/zetson-infant.yaml` (seed spec)
- Various VISION_VS_REALITY / STATUS / CLAUDE_ACTIONS_AUDIT

**These overlap and partially contradict.** Should be merged into exactly one
authoritative document, ideally with `AGI_ROADMAP.md` being the operational plan
and everything else archived or clearly sub-ordinate.

### Step 4: Then — and Only Then — Build the Next Thing

Whatever "next thing" is (P-A, P-C, or something else), it should:
- Build on ONE atom system
- Have a clear verification command  
- Not require creating parallel modules
- Ship in one session

---

## The New AGI Document Should Contain

Once the above cleanup happens, a new AGI doc can be written with these properties:

1. **Evidence-first.** Every claim cites a file, line number, or reproducible command.
2. **One atom system named.** Decision made, others retired.
3. **Zetson path as operational plan.** Not parallel theoretical Blueprint.
4. **5 capabilities (C1-C5) mapped precisely** to existing code with gaps identified.
5. **No 24-week timeline.** Only "next verifiable increment" + "vision beyond."
6. **Explicit non-goals.** What ZETS will NOT try to be (LLM fluency twin, etc.).
7. **Performance budget with measurement methodology.** Not just "80.8 µs" but
   "run `cargo bench --bench X` on commit Y, should show Z."

---

## Closing

Idan said: "צריך רק ללמוד ממה שעבד ומה שלא עבד."

### What worked:
- 7-primitive principle (when applied)
- autonomous-demo as a forcing function
- Unit tests discipline (1,301 passing)
- Core modules (morphology, sense_graph, metacognition)

### What didn't work:
- Multiple parallel architectures never retired
- Theoretical blueprints written ahead of code reality
- Spec-only missions accumulating without completion
- Claim-without-verification (test counts, performance numbers)
- Consulting AI masters on architecture-in-abstract

### The question for Idan:

Do you want to:
- **(A)** Accept this audit as the current truth and proceed to cleanup decisions?
- **(B)** Deepen the audit further (specific modules, tests I haven't run)?
- **(C)** Archive everything through today and start one new clean plan?

I will not write the next AGI document until you decide, because writing it
without your decision would repeat mistake #1: planning ahead of ground-truth.

