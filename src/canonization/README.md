# Canonization Module

Variant detection + epistemic classification for ZETS.

## Problem

Given two pieces of content, decide:
1. Is this the SAME content as something already in the graph?
2. At what level? (verbatim / semantic / conceptual / family)
3. What's the canonical reference?
4. What epistemic status does it have? (fact / tradition / opinion / fiction)
5. What's the quotation policy? (freely quotable / paraphrase only / concept only)

## Architecture

```
text + language + provenance
       │
       ▼
  fingerprint    ← structural + semantic hashing via sense_graph
       │
       ▼
  variant        ← compare against existing works
  detection        >0.95 = translation, >0.70 = derivative, >0.50 = parallel
       │
       ▼
  epistemic      ← pattern-based classification (no LLM)
  classifier       fact / tradition / opinion / fiction / mythology / ...
       │
       ▼
  quote policy   ← derived from epistemic + trust + license
                   freely quotable / paraphrase / concept only / private
       │
       ▼
  Work record
```

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module entry, re-exports, tests |
| `work.rs` | Work, WorkId, WorkKind, Fidelity, EpistemicStatus, QuotePolicy |
| `provenance.rs` | Provenance, TrustTier |
| `fingerprint.rs` | Structural + semantic fingerprinting via sense graph |
| `variant.rs` | Variant detection, VariantMatch, similarity thresholds |
| `epistemic.rs` | Pattern-based epistemic classifier |
| `policy.rs` | QuotePolicy derivation rules |
| `engine.rs` | CanonizationEngine — main orchestrator |
| `error.rs` | CanonizationError |

## Key design decisions

1. **Fingerprint skips unknown words in semantic hash** — only resolved senses
   contribute to semantic fingerprint, enabling true cross-lingual comparison.

2. **Epistemic classifier is pattern-based** — no LLM dependency. Provenance
   overrides (e.g., source="torah" → ReligiousNarrative) take priority.

3. **Idan's rule**: ReligiousNarrative → QuotePolicy::ConceptOnly.

4. **Retranslation detection**: if the source work is itself a translation,
   the new work is marked with Fidelity::Retranslation (drift risk).

## Usage

```rust
use std::sync::Arc;
use zets::sense_graph::SenseStore;
use zets::canonization::*;

let store = Arc::new(SenseStore::new());
let mut engine = CanonizationEngine::new(store);

let work = engine.canonize(
    "In the beginning God created the heavens and the earth.",
    1, // EN
    Provenance { source: Some("bible".into()), ..Provenance::unknown() },
).unwrap();

assert_eq!(work.epistemic, EpistemicStatus::ReligiousNarrative);
assert_eq!(work.quote_policy, QuotePolicy::ConceptOnly);
```
