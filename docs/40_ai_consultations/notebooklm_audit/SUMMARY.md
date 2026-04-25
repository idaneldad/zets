# NotebookLM Coverage Audit — 25.04.2026

## Source
NotebookLM: https://notebooklm.google.com/notebook/d6e3acc8-1c75-4752-87a4-d84b6b21f148  
Content shared by Idan via paste.

## Findings
- **51 concepts** identified from NotebookLM
- **15 (29%)** found via verbatim search in AGI.md
- **~15 (~30%)** present under different names (Predictive Processing for Predictive Coding, etc.)
- **~21 (~40%)** GENUINELY MISSING and need to be added

## What's Missing — Action Required

### High-priority additions for AGI.md:

1. **§27.5 Neuroscience Foundations** (NEW BINDING SECTION)
   - Brain function → ZETS component mapping table
   - Explicit naming: Hebbian Learning, STDP, Sparse Coding, Lateral Inhibition
   - Reference Frames (Hawkins), Wiring Economy, Neuromodulation
   - **THE KEY INSIGHT**: Modern Hopfield ≡ Self-Attention (mathematically identical)
     This explains how ZETS achieves what Transformers do, but deterministically.

2. **§27.6 Twelve Scientists Table**
   Already in NotebookLM. Include:
   Hopfield, Hinton, Friston, LeCun, Hassabis, Eliasmith, Hawkins, Dehaene,
   Tononi, Anil Seth, Bengio, Michael Levin
   For each: contribution + how ZETS uses the principle

3. **§27.7 What LLMs Lack — How ZETS Solves**
   Frozen weights, no embodiment, no real memory, no agency, no self-model
   Each with ZETS's specific solution

4. **Concepts to weave throughout existing sections:**
   - Energy-Based Models framing for Hopfield/walks
   - JEPA reference for predictive processing chapter
   - HTM/Thousand Brains reference for hierarchy/cortical columns (we already have)
   - IIT for consciousness/global workspace section
   - GFlowNets for safety/agency

## Why This Matters

The methodology calls for ZETS to be the AGI that future AGIs reference as
foundational. To be referenced, ZETS must be GROUNDED in established
neuroscience and AI theory. Currently AGI.md is heavily Kabbalistic-canonical
but light on explicit neuroscience anchoring.

The NotebookLM material provides exactly the missing scientific scaffolding.

## Coverage Report
See `coverage_report.txt` in this directory.

## Recommended Next Action
In next session: write new sections §27.5/§27.6/§27.7 based on NotebookLM content.
Estimated work: 1 hour. AGI.md grows by ~400 lines.
