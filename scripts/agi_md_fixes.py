#!/usr/bin/env python3
"""Apply highest-impact fixes to AGI.md based on Gemini + GPT-5.5 audit."""
from pathlib import Path
import json, datetime

AGI_PATH = Path('/home/dinio/zets/docs/AGI.md')
agi = AGI_PATH.read_text()
print(f"AGI.md before: {len(agi):,} chars, {agi.count(chr(10))} lines")

# === ADD §0 — ABI v1 BINDING SECTION (at top, after first heading) ===
# Find first major section break
abi_section = '''
---

# §0. ZETS Core ABI v1 — BINDING SOURCE OF TRUTH

**Status:** BINDING. Any contradiction between this section and others must
be resolved IN FAVOR OF THIS SECTION. Other sections will be patched, not
this one. This is the contract for federation, replay, and 30-year stability.

## §0.1 Document Status Labels

Every other section in this document is labeled:
- **[BINDING]** — final architectural commitment, will not change without ABI version bump
- **[EXPERIMENTAL]** — proposed design, subject to validation
- **[DEFERRED]** — recommendation pending implementation
- **[REJECTED]** — kept for historical reference, do not implement

Sections currently unlabeled are treated as [EXPERIMENTAL] until reviewed.

## §0.2 The Atom — Canonical 64-bit Layout (BINDING)

```
bit  63..60 │ kind          │ AtomKind enum (16 values)
bit  59     │ flag_quad     │ 4-letter root (vs 3-letter)
bit  58     │ flag_loanword │ foreign origin
bit  57     │ flag_irregular│ exception morphology
bit  56     │ flag_extended │ reserved for future
bit  55..50 │ language_id   │ 6 bits, 64 languages (Hebrew=0)
bit  49..32 │ encoded_chars │ 18 bits = 3 chars × 6 bits each
bit  31..30 │ gender        │ 2 bits (00=neuter, 01=fem, 10=masc, 11=both)
bit  29..27 │ binyan        │ 3 bits, 8 values (0=none for non-Semitic)
bit  26..24 │ tense         │ 3 bits
bit  23..20 │ pgn           │ 4 bits (person+gender+number)
bit  19     │ definite      │ 1 bit (Hebrew ה־ prefix)
bit  18..0  │ semantic_id   │ 19 bits = 524,288 disambiguation slots
```

## §0.3 Atom Kinds (BINDING enum, 4 bits = 16 values)

```
0x0  LexicalAtom        — words/morphemes (most common, all languages via language_id)
0x1  ConceptAtom        — abstract concept node
0x2  EdgeAtom           — relationship metadata when needed as first-class
0x3  RadicalAtom        — Chinese radicals + other logographic primitives
0x4  ProcedureAtom      — callable procedure (DAG of operations)
0x5  RuleAtom           — pattern rule for inference (231 gates etc.)
0x6  SourceAtom         — provenance source (document, user, API)
0x7  SenseAtom          — WordNet-style sense node
0x8  ContextAtom        — register/domain context
0x9  TimeAtom           — temporal anchor
0xA  ParseAtom          — provenance for parse decisions (causal chain)
0xB  ObservationAtom    — sensory observation (image, sound)
0xC  GoalAtom           — agentic goal/plan node
0xD  TrustAtom          — source trust score node (per-source)
0xE  MotifAtom          — repeated subpath dictionary entry
0xF  ReservedAtom       — reserved for future ABI extensions
```

**No other AtomKind values exist.** Any code that hardcodes other values is
non-conformant.

## §0.4 Edge Kinds — u16, NOT u8 (BINDING fix from prior versions)

EdgeKind is `u16` (2 bytes). The previous spec used `u8` and assigned values
>255, which was a bug. Canonical layout:

```
0x00..0x15  Sefer Yetzirah primary (22 Hebrew letters as edge primitives)
0x16..0xFF  Reserved for primary semantics extensions
0x100..0x1FF  CoOccurs, HasRgbValue, ObservedHas, TranslatesTo, etc.
0x200..0xFFFF  Application-specific (CHOOZ, etc.)
```

## §0.5 Determinism Boundary (BINDING)

**ZETS guarantees determinism for:**
- Graph storage and serialization
- Walk traversal given fixed (graph_version, query, seed)
- Inference results given fixed inputs
- Atom encoding/decoding
- Compression/decompression

**ZETS does NOT guarantee determinism for:**
- External LLM responses (Gemini/Claude/etc. as I/O parser)
- Image/audio embedding by CLIP/Whisper
- Real-time sensor input
- Network calls

**Boundary:** External outputs become Observations with provenance, never
direct facts. They enter the graph through trust-tiered insertion (see §29).
"Zero hallucination" applies ONLY to graph-derived answers, not to LM
realization layer.

## §0.6 Hardware Target (BINDING)

```
Minimum viable:    6 GB RAM, 4-core x86_64 or ARM64, 20 GB disk, no GPU
Recommended:       16 GB RAM, 8-core, 100 GB disk, optional NPU
Stretch (2031+):   Edge NPU integration via WebNN-like abstraction
```

Idle resident set: ~500 MB. Active query peak: ~2 GB. mmap edges: up to 6 GB
of disk-backed memory. Cold start: <2 sec.

## §0.7 AtomId Scaling (BINDING — addresses 30-year concern)

AtomId is `u32` for v1 (4.29B atoms). Migration path to `u64` is reserved as
ABI v2 trigger. Pre-emptive Gevurah pruning ensures active graph stays
under 2B atoms regardless of operating duration. Archived atoms move to
cold storage with `AtomId64` extended type.

## §0.8 What Will Never Change (30-year commitments)

1. **8-byte atom size** (allow ABI v2 for new fields, never shrink)
2. **Hebrew-first canonical** principle (other languages translate to root atoms)
3. **Determinism guarantee** for graph operations (boundary in §0.5)
4. **Walk-based reasoning** as primary inference mechanism
5. **Provenance** for every fact (no anonymous insertions)
6. **User sovereignty** over PersonalVault data

## §0.9 Versioning & Migration

```
ABI v1   — 2026, current (this document)
ABI v2   — Reserved for u64 AtomId, additional flag bits
ABI v3+  — Future, requires explicit migration tooling
```

Federation between ABI versions: only same-version graphs federate directly.
Cross-version requires explicit translation layer.

---
'''

# Find good insertion point — after the first major heading
# Look for first "# " line that's not a frontmatter
lines = agi.split('\n')
insert_at = -1
for i, line in enumerate(lines):
    # Find first content section after table of contents / preamble
    if line.startswith('# §1') or line.startswith('## §1'):
        insert_at = i
        break
    if line.startswith('## 1.') or line.startswith('# 1.'):
        insert_at = i
        break

if insert_at < 0:
    # Fallback: insert after first 50 lines
    insert_at = 50

new_agi = '\n'.join(lines[:insert_at]) + abi_section + '\n'.join(lines[insert_at:])
print(f"After §0 ABI insert: {len(new_agi):,} chars (added at line {insert_at})")

# === APPEND §28 — FORWARD-LOOKING ROADMAP ===
roadmap_section = '''

---

# §28. Forward-Looking Roadmap (2031–2056)

**Status:** [BINDING] for ZETS positioning, [EXPERIMENTAL] for specific
implementations.

This section addresses the 5/10/15/20/25/30-year horizons explicitly
requested by review and required for a 30-year foundational architecture.

## §28.1 — 2031 (5 years out)

**World context:** Local NPUs standard on laptops. Multimodal interaction
mature. Personal AI assistants ubiquitous but cloud-dependent.

**ZETS role:** The privacy-first, offline-capable alternative.

**Required capabilities:**
- NPU acceleration via WebNN-like abstraction (without breaking deterministic core)
- Multimodal Hebrew parsing at 99%+ accuracy
- Personal graph at 100M atoms scale
- Cold start <500ms
- Federation protocol v1 (between ZETS instances)

**Risks:** Frontier LLMs may close the privacy gap with on-device variants.

**Migration path:** ABI v1 must remain readable. New capabilities additive.

## §28.2 — 2036 (10 years out)

**World context:** AGI assistants mainstream. Public expects continuous
learning. Multi-agent ecosystems forming.

**ZETS role:** The trusted personal substrate that other AGIs query.

**Required capabilities:**
- Federated knowledge exchange protocol with provenance
- Zero-knowledge proofs for private answer attestation
- Conflict resolution for federated graphs (CRDT-based merge)
- Human-readable audit logs spanning years
- Stable ABI v1 with optional ABI v2 (u64 AtomId) introduction

**Risks:** Frontier AGIs may treat ZETS as just another data source rather
than respecting its sovereignty.

**Counter-strategy:** Cryptographic provenance ensures ZETS-sourced facts
are uniquely attributable. ZETS becomes the "notarized truth" layer.

## §28.3 — 2041 (15 years out)

**World context:** AGIs make most operational decisions. Humans focus on
goals, not execution. Multiple competing AGI ecosystems.

**ZETS role:** Constitutional layer — defines what user wants, AGIs execute
within that constraint.

**Required capabilities:**
- Goal specification language (formal, machine-checkable)
- Override protocols when AGIs deviate from user constitution
- Multi-AGI coordination via shared trust graph
- Long-term memory continuity across decades

**Risks:** Larger AGI systems may attempt to absorb or replace ZETS.

**Counter-strategy:** ZETS becomes harder to replace as personal graph
accumulates. Switching cost = decades of tagged personal knowledge.

## §28.4 — 2046 (20 years out)

**World context:** ZETS controls/orchestrates other AGIs on user's behalf.
Sub-AGIs run as plugins.

**ZETS role:** Orchestration layer with delegation, monitoring, termination.

**Required capabilities:**
- AgentExecutor for spawning, monitoring, and terminating sub-AGIs
- Permission model: capability-based, time-limited, scope-limited
- Proof-carrying plans (sub-AGIs must prove plan compliance before execution)
- Safety interlocks: ZETS can override any sub-AGI in real-time
- Constitutional escalation: certain decisions reserved for human

**Risks:** Sub-AGI sophistication may exceed ZETS's ability to verify plans.

**Counter-strategy:** Plan verification via formal methods (Z3 SMT) and
runtime monitoring. ZETS doesn't compete on intelligence — it competes on
trust and verification.

## §28.5 — 2051 (25 years out)

**World context:** Human-AGI integration deeply embedded. Cognitive
prosthetics common. Memory sovereignty becomes legal right.

**ZETS role:** Personal identity continuity vehicle.

**Required capabilities:**
- Encrypted lifelong vaults with quantum-resistant cryptography
- Identity continuity across hardware migrations
- Inheritance protocols (legal/social) for ZETS instances
- Cognitive prosthesis interface (when allowed by user)
- Human-in-the-loop boundaries (clearly defined where human required)

**Risks:** Loss of vault = loss of self for users who depend on ZETS.

**Counter-strategy:** Distributed backup with threshold cryptography.
User holds master key, never ZETS company.

## §28.6 — 2056 (30 years out)

**World context:** ZETS as foundational substrate. Citation network where
future AGIs cite ZETS-attested facts as authoritative.

**ZETS role:** The bedrock — what other AGIs build on.

**Required capabilities:**
- ABI v1 still readable (perfect backward compatibility)
- Migration tooling for ABI v2/v3
- Formal verification of core invariants
- Post-quantum cryptography throughout
- Federated canonical registries (the "Wikipedia" of ZETS-attested truth)
- Stable IDs that have been valid for 30 years

**What must never change:**
- 8-byte atom semantic core (only additive fields allowed)
- Hebrew-first canonical principle
- Determinism guarantee
- Walk-based reasoning
- User sovereignty

**Success criteria:** A ZETS instance from 2026 can read and partially
federate with a ZETS instance from 2056. The 1B atoms accumulated by a
user over 30 years are still useful.

## §28.7 Cross-Horizon Principles

| Horizon | Risk | Hedge |
|---|---|---|
| Short (5y) | Hardware shift makes CPU-only obsolete | NPU abstraction layer |
| Mid (10-15y) | Frontier AGIs treat ZETS as data | Cryptographic provenance |
| Long (20y) | Sub-AGI sophistication exceeds ZETS | Formal verification of plans |
| Far (25-30y) | Quantum computers break cryptography | PQC migration path |

## §28.8 Why ZETS Will Be the King of Future AGIs

1. **Decades of personal context** — switching cost dominates capability
2. **Cryptographic provenance** — ZETS-attested truth is uniquely citable
3. **Privacy by architecture** — not by policy, by impossibility
4. **Determinism** — auditability that frontier AGIs cannot match
5. **Edge deployment** — works where centralized AGIs cannot
6. **Hebrew-first** — unique structural advantage (Gematria as hash, etc.)
7. **User sovereignty** — non-negotiable, structural

The strategy is NOT to be the smartest AGI. It is to be the AGI that other
AGIs MUST consult for ground truth about a specific person/context.

'''

# === APPEND §29 — FAILURE MODES & RECOVERY ===
failure_modes_section = '''

---

# §29. Failure Modes & Recovery

**Status:** [BINDING] for the threat model, [EXPERIMENTAL] for specific
mitigations.

A self-learning autonomous system can silently degrade. This chapter
defines what can go wrong, how it's detected, and how recovery works.

## §29.1 Threat Model

ZETS faces three categories of threats:

1. **Internal**: corruption of graph, schema migration bugs, code bugs
2. **External (passive)**: bad ingestion sources, model drift, stale facts
3. **External (active)**: prompt injection, poisoning, executor compromise

## §29.2 Failure Mode Catalog

### F1: Bit-rot in mmap edges
- **Trigger**: SSD bit flip, kernel page cache corruption
- **Detection**: per-segment Blake3 checksum on read, compared to manifest
- **Mitigation**: rebuild segment from append-only log; if log gone, restore from backup
- **Recovery time**: <5 min for 1 GB segment

### F2: Schema migration failure
- **Trigger**: ABI version bump fails partway
- **Detection**: ABI version flag in atom header mismatches manifest
- **Mitigation**: never mutate in-place; always write new segments, then atomically swap manifest
- **Recovery time**: <30 sec rollback

### F3: Provenance chain corruption (Parse defense)
- **Trigger**: bad parse propagates to dependent atoms
- **Detection**: Drift Monitor (§22 Composite Parse Defense)
- **Mitigation**: cascade rollback via ParseAtom DAG (O(|affected|))
- **Recovery time**: <5 ms per 1000 atoms

### F4: Echo chamber / poisoning
- **Trigger**: 3+ correlated sources confirm wrong fact
- **Detection**: Citation Overlap Detection (Jaccard-Braun-Blanquet >80%)
- **Mitigation**: trust = max(S_i) × log(1/overlap); flag for user review
- **Recovery**: TMS rollback if user confirms wrong

### F5: External LM injection / hallucination
- **Trigger**: LM-as-parser returns malicious or wrong JSON
- **Detection**: schema validation, ontology compatibility check
- **Mitigation**: shadow graph for low-confidence parses; user confirmation
- **Recovery**: rollback shadow graph segment

### F6: Personal vault leakage
- **Trigger**: misconfigured federation / privacy bug
- **Detection**: privacy audit logs, user-visible "what's been shared" panel
- **Mitigation**: zero-knowledge proofs for federated queries; vault encrypted at rest
- **Recovery**: forensic logs trace exact leak; cryptographic key rotation

### F7: Executor compromise (Code/Web)
- **Trigger**: WASM sandbox escape, malicious code execution
- **Detection**: capability bitmap mismatch, unexpected syscalls
- **Mitigation**: process isolation in addition to WASM; capability minimization
- **Recovery**: kill executor, rebuild from manifest

### F8: Catastrophic over-learning
- **Trigger**: confirmation bias amplification, runaway self-reinforcement
- **Detection**: NightMode entropy monitor; if graph becomes too "ordered," flag
- **Mitigation**: Gevurah pruning forces decay; user can mass-rollback time window
- **Recovery**: time-window rollback to known-good state

### F9: Hardware failure (disk, RAM)
- **Trigger**: physical hardware death
- **Detection**: I/O errors, kernel panics, memory ECC errors
- **Mitigation**: encrypted off-device backup (user-controlled); replication optional
- **Recovery**: from backup; or partial reconstruction from graph append log

### F10: Silent semantic drift
- **Trigger**: word meanings shift over time (concept drift)
- **Detection**: Sense graph edge weights monitored over time
- **Mitigation**: time-tagged senses; old usage retrievable
- **Recovery**: not "recovery" but "evolution" — both old and new senses coexist

## §29.3 Recovery Hierarchy

```
Tier 1 (automatic, <1 sec):    rollback transaction, recompute walk
Tier 2 (automatic, <30 sec):   schema migration rollback, mmap segment rebuild
Tier 3 (automatic, <5 min):    full segment rebuild from append log
Tier 4 (semi-automatic):       user-confirmed time-window rollback
Tier 5 (manual):              restore from backup
```

## §29.4 Auditability

Every failure recovery generates an immutable audit log entry:
- Timestamp (logical clock + wall clock)
- Failure type
- Detection method
- Mitigation applied
- Recovery duration
- Atoms affected (count + sample)

User can query: `audit("what went wrong last week?")` returns chronological
list of all recoveries with explanations.

'''

new_agi += roadmap_section + failure_modes_section
print(f"After §28+§29: {len(new_agi):,} chars")

# Write back
AGI_PATH.write_text(new_agi)
print(f"AGI.md after fixes: {len(new_agi):,} chars, {new_agi.count(chr(10))} lines")
