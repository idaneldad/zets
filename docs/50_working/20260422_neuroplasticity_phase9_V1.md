# Phase 9 Blueprint — Neuroplasticity Layer
## The Compensation Engine: 49/49 + Shevirah Kfula in code

**Date:** 22.04.2026 (late session)  
**Status:** V1 blueprint — NOT implemented yet, reserved for post-training phase  
**Origin:** Idan shared 3 deep documents on neurodivergent brain compensation mechanisms. Direct match to Idan's "49 recursions + double shevirah" principle that appears throughout Cortex/Lev codebase.

---

## 🔴 The core insight

Neurodivergent brains don't have "broken" wiring — they have **forced compensation paths** that produce capabilities normal brains can't reach:
- **ADHD** → Hyperfocus under dopamine scarcity = explosive throughput at crisis points
- **Autism** → Local clustering at the cost of global edges = pattern recognition at micro-level that nobody else sees
- **Dyslexia** → Switch from sequential → spatial processing = understanding whole systems at once
- **SPD/APD** → Cross-modal routing when one sense is overwhelmed = uncanny sensitivity on other modalities

**Technical translation:** Given a graph with *forced failure* in some pathway + temperature-controlled random walk + 49 destruction iterations + 49 cold-annealing build iterations = the graph emerges with a compensation route that is:
1. Not available to a "healthy" graph (never forced to look there)
2. Objectively more capable on specific problem classes
3. Deterministic given the same seed

---

## 🏗️ Architecture

### Primitive: `Lesion`

```rust
pub enum Lesion {
    /// ADHD simulation: suppress long-term planning edges,
    /// high baseline T with sudden drops when signal is found
    DopamineScarcity { suppress_relations: Vec<u8>, baseline_t: f32 },
    
    /// Autism simulation: cut all edges with span > N hops,
    /// force deep local clustering
    GlobalPruning { max_span_hops: u32 },
    
    /// Dyslexia simulation: disable sequential token-matching,
    /// force vector-space traversal only
    NoSequentialAccess,
    
    /// SPD/APD simulation: when a given atom kind exceeds 
    /// noise threshold, route around it via other kinds
    CrossModalReroute { blocked_kind: AtomKind, fallback_kinds: Vec<AtomKind> },
    
    /// Custom lesion for research
    EdgeMask { mask_fn: fn(&AtomEdge) -> bool },
}
```

### Primitive: `Temperature`

```rust
pub struct Temperature {
    current: f32,
    /// Simulated annealing: P_accept = exp(-delta_E / T)
    cooling_rate: f32,
    min_t: f32,
    max_t: f32,
}

impl Temperature {
    pub fn accept(&self, delta_energy: f32, hash_seed: u64) -> bool {
        if delta_energy < 0.0 { return true; }  // always accept improvement
        let p_accept = (-delta_energy / self.current.max(1e-6)).exp();
        // deterministic via hash, not rand
        let r = hash_to_unit_float(hash_seed);
        r < p_accept
    }
    pub fn cool(&mut self) { self.current *= self.cooling_rate; }
    pub fn reset_hot(&mut self) { self.current = self.max_t; }
}
```

### The 49/49 cycle: `compensated_walk()`

```rust
pub fn compensated_walk(
    store: &mut AtomStore,
    prov_log: &mut ProvenanceLog,
    session: &SessionContext,
    lesion: Lesion,
    query: &str,
) -> CompensationResult {
    let mut walker = HashSeededWalker::new(query_hash(query));
    let mut temp = Temperature::hot();
    let mut best_path: Option<Path> = None;
    let mut best_energy = f32::INFINITY;
    
    // ─── Or Yashar (Going): 49 iterations of controlled destruction ───
    for iteration in 0..49 {
        temp.reset_hot();
        let masked_view = apply_lesion(store, &lesion);
        let candidate_path = walker.stochastic_walk(
            &masked_view, session.seeds(), &temp,
        );
        let energy = evaluate_path_energy(&candidate_path);
        
        if temp.accept(energy - best_energy, walker.next_seed()) {
            if energy < best_energy {
                best_path = Some(candidate_path.clone());
                best_energy = energy;
            }
        }
        temp.cool();  // gradual cooling during exploration
    }
    
    // ─── Shevirah Kfula: forced break of current best path ───
    let reshatter_seed = walker.next_seed().wrapping_mul(0x9E3779B97F4A7C15);
    best_path = reshatter_best(best_path, reshatter_seed);
    
    // ─── Or Chozer (Returning): 49 iterations of cold annealing ───
    for iteration in 0..49 {
        if let Some(ref path) = best_path {
            // Reinforce edges along the compensation path as Hypothesis 
            // (not Learned yet — needs verification)
            for edge in path.edges() {
                prov_log.tag(
                    EdgeKey::from(edge),
                    ProvenanceRecord::hypothesis(),
                );
            }
        }
        temp.cool_to_near_zero();
    }
    
    CompensationResult {
        compensation_path: best_path,
        final_energy: best_energy,
        lesion_used: lesion,
        iterations_total: 98,
    }
}
```

---

## 🎯 Use cases this unlocks

### 1. Hard queries that Precision mode can't answer
When `smart_walk_with_provenance(Precision)` returns empty, escalate to
`compensated_walk(Lesion::DopamineScarcity)` which forces exploration of
unexpected pathways. The compensation route might hit a Learned pattern
cluster that the normal walk missed because its seed atoms were too obvious.

### 2. Finding non-obvious analogies (research mode)
`Lesion::NoSequentialAccess` forces the graph to treat all atoms as a vector
space and find connections by structural similarity, not by proximity. This
is how dyslexic architects see system-level flaws that others miss.

### 3. Cross-domain synthesis (creative mode)
`Lesion::GlobalPruning { max_span_hops: 2 }` restricts thinking to tight
local clusters. Run it 49 times in parallel on different subgraphs → get
49 micro-expert outputs → synthesize.

### 4. Sensory override (robotics/multimodal)
`Lesion::CrossModalReroute { blocked_kind: ImageFrame, fallback_kinds: [AudioChunk, Text] }`
when vision is occluded/broken, the graph reroutes reasoning through available
modalities — without manual retraining.

---

## 🧪 Required before implementation

1. **Corpus at scale** — compensation is only interesting when graph is big enough
   to have non-obvious paths. Target: 100K+ atoms. Currently ~200.
2. **Energy function** — need a well-defined `evaluate_path_energy()`. Candidates:
   - Path length (shorter = better)
   - Edge confidence product (higher = better)
   - Query-seed alignment (better overlap = lower energy)
3. **Reshatter deterministic spec** — current proposal uses hash-based 
   edge selection. Needs formal spec so results are reproducible.

---

## 🔗 Connection to existing ZETS

This is NOT a replacement for `smart_walk`. It's an **escalation layer**:

```
User query
    │
    ▼
smart_walk_with_provenance(Precision) ─── has answer? → return
    │
    │ no answer
    ▼
smart_walk_with_provenance(Narrative) ── has answer? → return
    │
    │ still no
    ▼
smart_walk_with_provenance(Divergent) ─── has answer? → return
    │
    │ STILL no
    ▼
compensated_walk(lesion = auto-select) ← the neuroplastic escape
    │
    ▼
return compensation_path as Hypothesis edges
    │
    ▼
USER feedback → promote to Learned via distillation
```

This turns "graph can't answer" from a dead-end into a research opportunity.

---

## ⚠️ Why NOT now

1. **Scale prerequisite unmet** — at ~200 atoms, compensation produces noise,
   not insight. Need corpus first.
2. **Effort budget** — implementing properly (with 49/49 + deterministic seeds +
   energy function calibration) = 3-4 sessions.
3. **Product priority** — enterprise customers want corpus scale + verify API
   polish first. Neuroplasticity is a 2027 feature.

---

## 📝 Research references (Idan's shared docs)

Three documents from 22.04.2026 form the theoretical basis:
1. Neurodiversity overview (ADHD/ASD/Dyslexia/Dysgraphia/Dyscalculia/Dyspraxia/SPD/APD)
   with brain compensation mechanisms
2. GNN architecture mapping — each condition → specific graph-level fault pattern
3. Concrete algorithmic spec — 49 forward + 49 backward cycles with simulated
   annealing, temperature decay, edge masking

Saved to: `docs/research/neuroplasticity_sources.md` (TODO: create)

---

## ✅ Blueprint status

Written, filed, not implemented. Reserved for Phase 9 (post-training).
Current priority: get corpus ingestion to production scale first.
