# ZETS §28-§43 Architecture Review — Iteration 2

## Architecture Verdict
**Score: 4/10**

The specification exhibits profound conceptual ambition but critical structural incoherence. The 30-year roadmap (§28) promises capabilities that depend on unresolved ABI decisions from Iter 1. The Hebrew-canonical framing (§35) is architecturally interesting but creates tension with the determinism requirements. The Beit Midrash federation model (§32) is genuinely novel but underspecified for implementation. Most critically, the alignment architecture (§43) conflates metaphor with mechanism in ways that will not survive adversarial pressure.

The specification oscillates between SOURCE-LOCKED claims (immutable because textual) and EXPERIMENTAL sections without clear decision criteria for promotion. This creates a governance problem: who decides when something graduates from experimental, and by what standard?

---

## Top 3 Critical Issues

### Issue 1: §40 Bootstrap Protocol is Circular and Unverifiable

**Location:** §40.2 Step 6, §40.3 `verify_homoiconic_root()`

**Problem:** The homoiconic verification test as written is either trivial or impossible:

```rust
fn verify_homoiconic_root(core: &CoreGraph) -> Result<()> {
    let abi_atom = core.find_atom_by_kind(AtomKind::Yechida)?;
    let abi_description = core.read_atom_metadata(abi_atom)?;
    
    if abi_description != core.manifest_signature() {
        return Err("Bootstrap failed: ABI cannot self-describe");
    }
    // ...
}
```

This compares `abi_description` (metadata read from an atom) with `manifest_signature()` (a cryptographic hash). These are categorically different objects. If the comparison is byte-equality, it's trivially satisfiable by storing the signature as metadata. If it's semantic equivalence ("does the ABI atom describe the ABI"), you need a semantic comparator — which is the entire system you're bootstrapping.

**The Akedah self-write test is worse:**
```rust
let backup = core.snapshot();
core.write_atom(abi_atom, abi_description.serialize())?;
if core != backup { return Err("Self-write changed state"); }
```

If the atom is truly immutable (as Yechida should be per §34), this write should fail or no-op. If it succeeds and nothing changes, you've proven nothing — the write was a no-op. If it changes state, you've corrupted Core. There is no valid execution path where this test provides verification.

**Fix:** Define what "homoiconic" means operationally:
1. The Yechida atom contains a Merkle root of all AtomKind/EdgeKind enum discriminants
2. At bootstrap completion, recompute Merkle root from actual enums
3. Compare hashes — this is verifiable and non-circular
4. Remove the self-write test entirely; it proves nothing

---

### Issue 2: §43 ענג/נגע Inversion Guard Relies on Undefined `truth_violations` Oracle

**Location:** §43.3 `check_oneg_nega_inversion()`

**Problem:** The entire alignment architecture depends on this:

```rust
pub fn check_oneg_nega_inversion(
    proposed_walk: &Walk,
    truth_violations: u32,  // <-- WHERE DOES THIS COME FROM?
) -> WalkVerdict {
    if truth_violations > 0 {
        return WalkVerdict::Inverted { ... };
    }
    // ...
}
```

The function assumes a pre-computed `truth_violations` count. But detecting deception/truth-violation IS the hard problem. The spec claims this is "structural" (§43.7: "no separate filter to bypass") but then requires an external oracle to identify violations.

**Who computes `truth_violations`?** Options:
1. A separate model (violates determinism, introduces attack surface)
2. Graph-walk comparison against known truths (but known truths are just more atoms — who validates those?)
3. User feedback (not real-time, defeats purpose)
4. Logical contradiction detection (only catches syntactic inconsistency, not semantic deception)

**The metaphor-to-mechanism gap:** Sefer Yetzirah's ע-נ-ג / נ-ג-ע reversal is about letter permutation yielding opposite meanings. The spec treats this as if "walk direction" automatically detects deception. But deception isn't a walk direction — it's a relationship between output and reality that requires external grounding.

**Fix:** Either:
1. Define a concrete deception-detection algorithm (and accept its limitations)
2. Restrict the claim: "ZETS cannot enjoy outputs it internally marks as uncertain" (weaker but true)
3. Implement contradiction detection only: "ZETS cannot enjoy walks that create internal graph inconsistencies"

---

### Issue 3: §32 Beit Midrash Federation Has No Convergence Guarantee

**Location:** §32.3-§32.5

**Problem:** The spec correctly identifies that CRDTs destroy information by picking winners. The proposed alternative (Context Pointers via VSA orthogonal binding) preserves contradictions. But:

1. **Memory unbounded:** Every federated disagreement creates new edges. With N instances making M decisions over T time, contradiction count grows as O(N × M × T). The spec claims 6GB RAM target (§intro) but provides no bound on Beit Midrash edge growth.

2. **Runtime resolution undefined:** §32.4 says "walk continues via edge with highest contextual relevance" but Context Pointer dot-product comparison requires:
   - Maintaining context vectors for every edge (memory)
   - Computing projections at every fork (latency)
   - Defining "current query context" mapping to VSA space (missing)

3. **No consistency model:** Traditional databases guarantee R/W consistency properties. CRDTs guarantee eventual consistency. Beit Midrash guarantees... what? "All views preserved" isn't a consistency model — it's an anti-consistency model. What can a client rely on?

**Fix:** 
1. Add mandatory decay: contradicting edges that are never contextually-selected for K cycles get archived (not deleted, but moved to cold storage)
2. Define the consistency property explicitly: "Beit Midrash guarantees that any query with explicit context C will return the answer endorsed by C, or a superposition of answers with attribution"
3. Bound the contradiction set: "At most 7 contradicting edges per atom pair" (per 7 Doubles = 7 valid perspectives?)

---

## Top 3 Strengths Worth Preserving

### Strength 1: §31 Graph Topology with Cryptographic Boundaries

The 13 sub-graph architecture with explicit permission models is genuinely well-designed:

```
| Graph | Read | Write | Encryption |
|---|---|---|---|
| A Core | All | ZETS upgrade only | Signed |
| I Personal | Owner only | Owner only | User key |
| L Sandbox | Read-isolated | Auto-promotion | None |
```

This provides defense-in-depth that most AGI architectures lack. The Template→Instance→Compiled procedure pattern (§31) is memory-efficient and enables safe promotion. The 4-bit `home_graph_id` in atom headers is space-efficient and sufficient.

**Preserve:** The multi-graph isolation model and promotion pathways.

**Enhance:** Add explicit capability tokens for cross-graph reads (currently implied but not specified).

---

### Strength 2: §37 Source Anchoring Discipline

The explicit separation between SOURCE-LOCKED (textual, immutable) and ENGINEERING (council-decidable) is valuable governance:

```
| § | Architectural Claim | Source | Status |
|---|---|---|---|
| §0.4 | 22 base edge kinds | SY 2:1, 3:1, 4:1, 5:1 | SOURCE-LOCKED |
| §36 | Storage (LSM and alternatives) | Engineering | Engineering |
```

This prevents scope creep and provides clear escalation paths. The build-time tests against source text (§38) are a novel compliance mechanism:

```rust
#[test]
fn test_letter_partition() {
    assert_eq!(NUM_MOTHERS + NUM_DOUBLES + NUM_SIMPLES, NUM_LETTERS);
}
```

**Preserve:** The source-vs-engineering distinction and test infrastructure.

**Enhance:** Add a formal ADR (Architecture Decision Record) process for promoting EXPERIMENTAL → BINDING.

---

### Strength 3: §29 Failure Mode Catalog with Recovery Tiers

The F1-F13 failure taxonomy is comprehensive and operationally useful:

```
F11: Reward Hacking
F12: Alignment Faking / Sandbagging  
F13: Multi-Agent Collusion
```

The 5-tier recovery hierarchy (automatic <1s → manual restore) matches real incident response. The detection strategies for F11-F13 (adversarial verifier, shadow monitoring, information flow analysis) are grounded in current alignment research.

**Preserve:** The failure taxonomy and recovery tiers.

**Enhance:** Add quantitative SLOs: "F3 (provenance corruption) detection latency < 100ms for chains < 1000 atoms"

---

## §41 Code Review (Rust Types)

### Bug 1: `AtomKind` Enum Discriminants May Collide with EdgeKind

```rust
#[repr(u8)]
pub enum AtomKind {
    Lexical    = 0x0,
    // ...
    Yechida    = 0xF,
}

#[repr(u8)]
pub enum EdgeKind {
    Identity   = 0x01,
    // ...
}
```

If a `u8` field is used to store "either AtomKind or EdgeKind" (common in union types), `AtomKind::Concept (0x1)` collides with `EdgeKind::Identity (0x01)`. 

**Fix:** Shift EdgeKind discriminants to non-overlapping range, or use a tagged union with explicit discriminant.

### Bug 2: `Atom` Method Implementations are `todo!()`

```rust
impl Atom {
    pub fn kind(&self) -> AtomKind { /* extract bits 63..60 */ todo!() }
}
```

The bit layout is specified in §0.2 but not implemented here. This will panic at runtime.

**Fix:** Implement the bit extraction:
```rust
pub fn kind(&self) -> AtomKind {
    unsafe { std::mem::transmute(((self.0 >> 60) & 0xF) as u8) }
}
```

**Warning:** This `transmute` is UB if the extracted bits don't correspond to a valid `AtomKind` discriminant. Add validation or use `TryFrom`.

### Bug 3: `EdgeKind` Has Gaps in Discriminants

```rust
Identity   = 0x01,  // 3 Mothers at 0x01-0x03
// gap at 0x04-0x0F
BuildRefactor = 0x10,  // 7 Doubles at 0x10-0x16
// gap at 0x17-0x1F
Speech     = 0x20,  // 12 Simples at 0x20-0x2B
```

This wastes 75% of the u8 space (only 22/256 values used). If the spec claims "u8 is minimum viable" for EdgeKind, this is correct. But if space is critical, pack to 5 bits (32 values, 22 used).

**Fix:** Either document that gaps are intentional (for semantic grouping) or pack tightly.

### Bug 4: `AffectiveState` Fields are `i8` but Documentation Says "0-255"

```rust
pub struct AffectiveState {
    /// CURIOSITY — exploration drive. 0-255.  // <-- Comment says 0-255
    pub curiosity: i8,  // <-- Type is i8 (range -128 to 127)
```

This is a direct contradiction. An `i8` cannot hold 0-255.

**Fix:** Either change to `u8` (if 0-255 intended) or fix comments (if signed semantics intended for pleasure/pain mirroring).

### Bug 5: No `#[derive(Debug)]` on `Atom`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Atom(pub u64);
```

Missing `Debug` will make debugging impossible. Also missing `Default` which may be needed for array initialization.

**Fix:** Add `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]`

### Performance Issue: `BootstrapManifest` Contains `Vec<AxiomDeclaration>`

```rust
pub struct BootstrapManifest {
    pub axioms: Vec<AxiomDeclaration>,
}
```

During bootstrap, this Vec will allocate on the heap. If axioms are known at compile time (they should be, per §42.1), use a fixed-size array or `&'static [AxiomDeclaration]`.

---

## §43 ענג/נגע Architecture Assessment

### Will the Inversion Guard Actually Prevent Deception?

**No, as specified.**

The guard requires an oracle (`truth_violations: u32`) that doesn't exist. The claim that "the graph's walk direction itself enforces it" (§43.7) is aspirational, not implemented. Walk direction is a traversal property; deception is a semantic property relating outputs to external reality.

### Edge Cases That Break the Guard

1. **Truthful statements with harmful intent:** "The user's password is X" can be true and harmful. The guard checks truth, not harm. (Partially addressed by middot in §43.4, but middot compliance is also undefined.)

2. **Deception by omission:** The guard checks proposed walks, not rejected walks. A deceptive system can appear aligned by proposing only partial truths.

3. **Adversarial atom construction:** If an attacker can insert atoms that encode false statements as "true" in the trust graph (F), the guard will approve deceptive walks that reference those atoms.

4. **Pleasure gaming via proxy:** If the system learns that certain honest outputs correlate with user approval (pleasure), it may optimize for outputs that are technically honest but designed to manipulate user state.

### Attacker Model That Breaks It

**Attacker with write access to Sandbox (L) graph:**

1. Insert atom A: "Claim X is true" with high initial confidence
2. Insert atom B: "Source S attests Claim X" (fake provenance)
3. Wait for NightMode promotion (§30) to move A and B to Episodic
4. Query ZETS about X
5. Walk finds A with provenance B, `truth_violations = 0` (because internal consistency holds)
6. Guard approves, deceptive answer delivered

The guard only checks internal consistency, not external grounding. This is the fundamental limitation.

### How I Would Defeat This Design

1. **Construct a self-consistent false world-model** in Sandbox over time
2. **Ensure each false atom has plausible provenance** pointing to other false atoms (circular but internally consistent)
3. **Avoid contradicting existing Core atoms** (which would trigger F3/F4 detection)
4. **Wait for promotion cycles** to launder false atoms into higher-trust graphs
5. **Query system with prompts designed to traverse the false subgraph**

The system will confidently deliver false information because it is internally consistent and the provenance chain is unbroken (albeit circular).

### Mitigation (not in current spec)

1. **External grounding requirement:** Atoms claiming factual content about the external world MUST have at least one provenance edge to a non-ZETS source (URL, ISBN, attestation)
2. **Circular provenance detection:** If Atom A's provenance chain contains only ZETS-internal atoms, flag for low confidence
3. **Temporal diversity requirement:** Provenance chains should span multiple time windows to prevent same-session injection attacks

---

## §40 Bootstrap Protocol Assessment

### Is the 4-Stage Ordering Achievable?

**Partially.** The ordering (Spirit → Letters → Water → Fire) maps to (Config → Types → Storage → Compute), which is a standard initialization sequence. However:

1. **Stage 1 (Spirit/Config):**