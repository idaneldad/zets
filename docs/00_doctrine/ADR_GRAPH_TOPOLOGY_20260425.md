# ADR — ZETS Graph Topology (13 Sub-Graphs)

**תאריך:** 25.04.2026  
**Status:** [PROPOSED — דורש אישור עידן + Iter 1 council validation]  
**Author:** Idan Eldad + Claude Opus 4.7

---

## ההחלטה

ZETS תורכב מ-**13 sub-graphs** דטרמיניסטיים, נפרדים פיזית בstorage,
מחוברים על ידי cross-graph edges עם cryptographic permissions.

## 13 ה-Sub-Graphs

### Layer 1 — Core (ZETS Kernel)
**A. CORE** — Logic of ZETS itself (homoiconic). Read-only, signed.

### Layer 2 — Knowledge (Public Substrate)
**B. SENSE** — Linguistic synsets (already exists, src/sense_graph.rs)  
**C. SEMANTIC** — Abstract concepts and reasoning  
**D. ARTICLE** — Content paths (Wikipedia-scale)

### Layer 3 — Verification & History
**E. PROVENANCE** — Source attribution per fact (enables Deterministic Attestation)  
**F. TRUST** — Beta-Binomial scores per source (TMS #11b)  
**G. TEMPORAL** — Time-anchored events (uses Q16 Dynamic Temporal Tag)

### Layer 4 — Action
**H. PROCEDURE** — Agents/tools/workflows. Templates + Instances + Compiled binaries.

### Layer 5 — Identity (Sovereign)
**I. PERSONAL[user]** — Per-user, encrypted with user's key  
**J. ZETS-SELF** — ZETS own monologue, encrypted with ZETS own key  
**K. GROUP** — Scoped visibility (NOT OO inheritance) — refs to shared Personal atoms

### Layer 6 — Safety & Federation
**L. SANDBOX** — L0-L3 quarantine for untrusted code, failed parses, speculative analogies  
**M. FEDERATION** — Cross-instance public substrate (2036+ AGI ecosystem)

## עקרונות

### 1. Physical Separation
- Each graph = separate mmap file: data/graphs/{a..m}.dat
- Per-graph schema version, ABI, manifest
- Per-graph encryption key (where applicable)
- Independent scaling — Personal can grow without bloating Core

### 2. Cross-Graph References
- Atoms reference other graphs via (GraphId, AtomId) tuple
- Stable IDs preserved across restarts, exports, federation
- 4 bits in atom header reserved for home_graph_id

### 3. Permission Model (cryptographic)

| Graph | Read | Write | Encryption |
|---|---|---|---|
| A Core | All | ZETS upgrade only | Signed |
| B-D Knowledge | All | Append-only via TMS | None (public) |
| E-G Verification | All | Internal only | Signed |
| H Procedure | All | L0-L3 promotion | Signed at L2+ |
| I Personal | Owner only | Owner only | User key |
| J ZETS-Self | ZETS only | ZETS only | ZETS master key |
| K Group | Members per scope | Per scope rules | Group key |
| L Sandbox | All (read isolated) | Auto-promotion | None |
| M Federation | All (auth required) | Consensus protocol | Multi-sig |

### 4. NightMode Consolidation
- Per-graph independent timing
- Personal graph consolidates first (cheapest, user-private)
- Article last (most expensive, can be queued)

### 5. Procedure: Template + Instance Pattern

```
ProcedureAtom (Template) -- INSTANTIATES -- InstanceAtom (Instance)
       |                                         |
       |-- kind: TemplateAtom                    |-- kind: InstanceAtom
       |-- pure pattern, no state                |-- bound params
       |-- stored once                           |-- runtime state
       |-- promoted L0..L3                       |-- event log

After L1 (100 successful runs): compile to WASM bytecode
After L2 (verified): compile to native binary
After L3 (core): mmap as executable, native call

Memory: 1 template + N instances << N copies of same code
```

### 6. Group Hierarchy = Scoped Visibility (NOT OO)

```
Personal[Idan] -- owns/canonical -- PersonalAtom
                           |
                           +- shares-to[CHOOZ, ReadOnly] ----+
                                                             |
Group[CHOOZ] -- ref-only ----------------------------------+
                           |
                           +- shares-to[Industry, Anonymous]+
                                                             |
Group[Industry] -- ref-only ------------------------------+
```

GrantLevel:
- ReadOnly — group reads atom as-is
- ReadWrite — group can update (rare)
- Anonymous — group sees fact, not source identity
- Aggregated — group sees only aggregate (k-anonymity)

## Cost Estimate

| Graph | Size estimate | Storage |
|---|---|---|
| A Core | <50 MB | RAM resident |
| B Sense | 200 MB | mmap |
| C Semantic | 500 MB - 2 GB | mmap |
| D Article | 1-3 GB | mmap |
| E Provenance | 300 MB | mmap |
| F Trust | 50 MB | RAM resident |
| G Temporal | 200 MB | mmap |
| H Procedure | 100 MB | mmap |
| I Personal | per user | mmap, encrypted |
| J ZETS-Self | 200 MB | mmap, encrypted |
| K Group | per group | mmap, encrypted |
| L Sandbox | <100 MB | mmap, isolated |
| M Federation | 1 GB | mmap, signed |

**Total core (A-H):** ~3-7 GB. Fits 6 GB target with mmap demand-paging.

## Open Questions for Iter 1 Council

1. GraphId 4 bits enough (16 graph types)? Or 5 bits for expansion?
2. Default cross-graph walk behavior — auto-traverse or explicit grant?
3. Backup strategy — per-graph independent or atomic across all?
4. Federation atom format — same 8-byte or extended with signatures?

## למה זה חשוב

בלי הפרדה פיזית-קריפטוגרפית:
- Personal graph leak -> User loses sovereignty (catastrophic)
- ZETS-Self leak -> User reads ZETS internal thoughts -> broken trust
- Group leak -> Cross-organization data spillage

**Sovereignty is a structural property of the storage layout, not a policy.**
