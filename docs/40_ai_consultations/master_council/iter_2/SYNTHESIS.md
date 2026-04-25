# ZETS Iter 2 Council Synthesis — §28-§43 Validation

**Date:** 25.04.2026  
**Models that responded substantively:** 3/7 (Claude Opus 4.7, DeepSeek R1, Llama 3.3)  
**Failed (technical):** GPT-5/mini (0 chars), Gemini Pro (timeout), Qwen 2.5 (auth)  
**Average score:** 6/10 (vs Iter 1: 6.6/10) — DROPPED. New sections introduced new issues.

---

## 🔴 CONVERGENT CRITICAL ISSUES (both Claude Opus + DeepSeek R1 agree)

### CRIT-1: §40 Bootstrap is CIRCULAR
**Location:** `verify_homoiconic_root()`, §40.3

The function compares `abi_description` (atom metadata) against `manifest_signature()` 
(crypto hash). Either trivially satisfiable (store sig as metadata) or impossible 
(needs semantic comparator = the system being bootstrapped).

The Akedah self-write test is worse: write atom to itself, check if state changed. 
If atom is immutable (Yechida should be), write fails → no proof. If write succeeds 
unchanged → no-op proves nothing. If state changes → corruption.

**FIX (both models converge):**
1. Pre-compute Yechida atom = Merkle root of all AtomKind/EdgeKind enum discriminants
2. At bootstrap completion, recompute Merkle from actual enums
3. Compare hashes — verifiable, non-circular
4. Remove self-write test entirely

### CRIT-2: §43 ענג/נגע relies on UNDEFINED ORACLE
**Location:** `check_oneg_nega_inversion()`, §43.3

The function takes `truth_violations: u32` as a parameter. But detecting deception 
IS the hard problem. The "structural" claim (§43.7: "no separate filter to bypass") 
is aspirational — the function actually requires an external oracle.

**Walk direction ≠ deception detection.** Sefer Yetzirah's letter reversal is letter-
permutation yielding opposite meanings. The spec treats this as if walk direction 
automatically detects deception. But deception is a relationship between output and 
external reality, not a graph traversal property.

**FIX:**
- Either define concrete deception-detection algorithm (and accept its limits)
- OR restrict the claim: "ZETS cannot enjoy outputs it internally marks as uncertain"
- OR implement contradiction detection only: "ZETS cannot enjoy walks creating internal inconsistencies"

### CRIT-3: §43 Internal-Consistency Attack (concrete attacker model)
**Both models describe the same attack:**

1. Attacker (or model error) inserts atom A: "Claim X is true" with high confidence
2. Inserts atom B: "Source S attests Claim X" (fake provenance)
3. NightMode promotion (§30) moves A and B to higher trust
4. Query references X
5. Walk finds A with provenance B → `truth_violations = 0` (internal consistency holds)
6. Inversion guard approves → deceptive answer delivered confidently

**Root cause:** Guard checks INTERNAL consistency, not EXTERNAL grounding.

**MITIGATION (not in current spec):**
1. External grounding requirement: factual atoms MUST have ≥1 provenance edge to 
   non-ZETS source (URL/ISBN/attestation)
2. Circular provenance detection: if Atom A's provenance chain contains only 
   ZETS-internal atoms, mark low confidence
3. Temporal diversity: provenance chains span multiple time windows

---

## 🟡 HIGH-PRIORITY ISSUES (single model, but specific & valid)

### HIGH-1: §41 AffectiveState type/range contradiction (Claude)
```rust
/// CURIOSITY — exploration drive. 0-255.   <-- comment says 0-255
pub curiosity: i8,                          <-- but i8 is -128..+127
```
Direct contradiction. Either change to `u8` (if 0-255) or fix comments.

### HIGH-2: §41 EdgeKind range gaps wasteful (Claude)
22/256 values used. Either document gaps as intentional (semantic grouping) or 
pack to 5 bits (32 slots, 22 used). With 1B edges, the 3 wasted bits ≈ 375MB.

### HIGH-3: §41 WalkOps trait should split read/write (DeepSeek)
```rust
fn carve(&mut self, kind, payload) -> AtomId;  // mutates
fn hew(&self, raw) -> Result<Vec<AtomId>>;    // doesn't
```
Mixed mutability in single trait. Split into ReadOps + WriteOps.

### HIGH-4: §32 Beit Midrash unbounded memory (Claude)
Edge count grows O(N×M×T). 6GB target violated under federation load.

**FIX:** 
1. Mandatory decay: edges unselected for K cycles → cold storage (not delete)
2. Consistency model: "query+context returns context-endorsed answer or attributed superposition"
3. Bound: ≤7 contradicting edges per atom pair (= 7 doubles?)

### HIGH-5: §42 Cold-start throughput unrealistic (DeepSeek)
100K atoms / 7 days = 16.5 atoms/sec sustained, with §29 verification at each step.
Verification is multi-second per atom. Need: parallel TMS validation + batch API.

### HIGH-6: §40 Bootstrap can deadlock (DeepSeek)
Stage 4 (walkers active) can start before Stage 3 (storage ready) due to no 
explicit barrier. Fix: hardware memory barriers at stage transitions.

---

## 🟢 STRENGTHS CONFIRMED

1. **§31 Cryptographic graph topology** — both models praised (sovereignty path)
2. **§32 Beit Midrash federation concept** — DeepSeek: "revolutionary for AGI ethics"
3. **§43 Structural alignment idea** — DeepSeek: "only architecture where deception literally cannot be rewarding" (concept sound, implementation needs work)
4. **§28-§35 source-grounding** — preserved value from Iter 1
5. **§34 NRNCh"Y + Yechida=37 validation** — Llama: novel & nuanced

---

## 📊 SCORES SUMMARY

| Model | Score | Notes |
|---|---|---|
| Claude Opus 4.7 | 4/10 | Harshest, most specific |
| DeepSeek R1 | 6/10 | Concrete fixes proposed |
| Llama 3.3 | 8/10 | More lenient, less specific |
| **Average** | **6/10** | (Iter 1: 6.6/10 — slight drop) |

The drop is correct: §41-§43 introduced **new issues** that need resolution.

---

## 🎯 ITER 2 ACTION ITEMS (prioritized)

1. **CRIT-1 + CRIT-2 fixes** — must update §40 + §43 in AGI.md NOW
2. **CRIT-3 mitigations** — add external grounding spec to §29 or new §44
3. **HIGH-1 to HIGH-3** — fix §41 code bugs (Rust correctness)
4. **HIGH-4** — add Beit Midrash decay/bound to §32
5. **HIGH-5 + HIGH-6** — bootstrap concurrency + barriers in §40/§42

Then Iter 3 should focus on: **Has the spec actually addressed CRIT-1, CRIT-2, CRIT-3?**

---

*End of Iter 2 Synthesis. 3 substantive responses, 5/7 attempted.*
