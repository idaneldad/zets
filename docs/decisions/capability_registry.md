# 0004 — Capability Registry: capabilities as graph atoms, not code blobs

**Date:** 2026-04-23  
**Status:** Accepted  
**Related:** `docs/20_research/openclaw/03_practical_lessons_v2.md`

## Context

Procedures (skills, workflows, recipes) can be stored two ways:

- **Code-blob model:** each procedure is a self-contained bytecode/script. Storage = O(procedures × avg_length).
- **Graph model:** each procedure is a DAG of references to capability atoms. Shared capabilities exist once. Storage = O(unique capabilities + composition edges).

OpenClaw uses the code-blob model (each skill is a self-contained Markdown+bytecode bundle). This works for them because they have ~53 skills. ZETS targets thousands.

## Decision

Capabilities are stored as **graph atoms** with the following constraints:

1. **Primitive capabilities** (bytecode leaves): ~50 base operations. Each exists once, indexed in a CapabilityRegistry. Examples: `http_post`, `json_parse`, `regex_match`, `sha256_hash`.

2. **Composition capabilities** (DAG of CallProcedure steps): unlimited, but each is just a list of references + arg mapping. Storage per composition: ~200 bytes typical.

3. **Domain capabilities** (wrappers): bridge between user intent and primitive composition. Examples: `send_whatsapp_via_greenapi` references `http_post` + `build_request` + `parse_json`.

4. **CapabilityRegistry** is the single point of truth:
   - Indexed by sense_key (for intent → capability lookup)
   - Indexed by I/O signature (for type-compatible composition)
   - Indexed by canonical hash (for exact dedup)
   - Indexed by semantic embedding (for near-dup detection)

5. **Deduplication cascade** runs on every `add()`:
   - Layer 1: Semantic embedding (cosine ≥ 0.92 → candidate match)
   - Layer 2: Behavioral test (run test inputs through both, compare outputs)
   - Layer 3: Sense-key match (WordNet synset + bitflag axes)
   - Layer 4: Hash exact (last-resort integrity check)

6. **Variants** (e.g. `http_post_v1` vs `http_post_v2_with_retry`) are linked via edges (`VariantOf`, `ImprovesOn`) — NOT separate top-level atoms.

## Consequences

**Positive:**
- Storage 10× more efficient than code-blob (measured prediction: 450KB vs 5MB for 1000 procedures)
- New procedures usually require only ~200B (composition) not new bytecode
- Discovery via sense_key works across all capabilities
- Easy to upgrade primitives — composition automatically benefits

**Negative:**
- Lookup latency higher than direct bytecode call (hash → atom → bytecode)
- Compositional debugging is harder (need to trace through multiple atoms)
- Initial registry seeding (~50 primitives) is significant up-front work

## Mitigation

- mtreemap stores hot capabilities adjacent (98% cache hit)
- VM caches resolved capability bytecode for the duration of a procedure execution
- Trace tool that flattens execution path for debugging

## Related work

- OpenClaw's skill model (code-blob) — what we are NOT doing
- Cyc microtheories — similar idea of capability composition, predates by 30 years
- Smalltalk method dispatch — same recursive-call pattern
