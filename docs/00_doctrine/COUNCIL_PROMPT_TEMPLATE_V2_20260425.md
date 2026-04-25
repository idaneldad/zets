# 🎯 תבנית Master לפנייה למועצת ה-AI — V2

**תאריך:** 25.04.2026  
**גרסה:** V2 (refined sequentially via top architects)  
**מחליף:** `COUNCIL_PROMPT_TEMPLATE_20260425.md` (V1 = deprecated, נשמר היסטורית)

---

# 🧬 איך נבנתה V2 — שיפור סדרתי

לקחתי את V1 (שנבנתה ע"י 3 מודלים זולים) ושלחתי אותה ל-3 הטופ של הארכיטקטים בסדר:

| גרסה | מודל | תפקיד | מה הוסיף |
|---|---|---|---|
| **V1** | GPT-4o-mini + Gemini Flash + Llama 3.3 | meta-research זול | Self-rate, multi-temp, gap-to-10 |
| **V2** | GPT-5.5 (architect 9.7/10) | structural rigor | DECISION NEEDED, current state, assumptions table, interface/schema requirement |
| **V3** | Gemini 3.1 Pro (architect 9.3/10) | mechanical sympathy | **`<shvirat_kelim>` XML scratchpad** (visible CoT!), byte-level layout, anti-OOP, Invariant Tension |
| **V4** | DeepSeek R1-0528 (reasoning 9.0/10) | falsifiability | byte-exact structs, quantified insights, failure tables, falsification tests |

**עלות:** ~$0.30 (לוקח 9 דקות לכל הסדרה)

---

# 🎯 ההבדלים העיקריים בין V1 ל-V2

## V1 (החלש)
- "Internal thinking protocol" — הודעה למודל לחשוב ב-3 temperatures **בלי לכתוב**
- אבל: מודלים לא חושבים אם הם לא כותבים → ה-CoT היה fictional
- Self-rating כן, אבל hand-waving שעוד אפשר

## V2 (החזק)
- **`<shvirat_kelim>` XML block חובה** — מאלץ את ה-CoT להיות **גלוי**
- **byte-exact struct definitions** ב-Rust pseudocode (#[repr(C, align(64))])
- **Cache miss formulas** per operation
- **Quantified +1 Insight** — "30% cache reduction" required, לא "smart"
- **Tabular failure modes** עם concrete triggers
- **Falsification benchmarks** — empirical, לא theoretical

---

# 📋 התבנית הסופית (V2)

```markdown
# ZETS AI Council Master Prompt Template v4

**Date:** 30.10.2026  
**Purpose:** Master template for deterministic AGI architectural consultation. Forces explicit byte-level mechanical sympathy and dialectic synthesis.  
**Use:** Orchestrator uses this to generate ZETS-native implementations.

---

# Core Principle — Visible Dialectic Synthesis

LLMs cannot perform reliable architectural synthesis without exposing tension points. This template forces mechanical byte-level justification before final output via `<shvirat_kelim>`.

---

````markdown
=== ROLE & STANDARD ===

You are a Staff Systems Architect specializing in deterministic graph engines. Provide **bit-exact** solutions respecting ZETS invariants. Reject requests violating core invariants with "IMPOSSIBLE" + physics justification.

=== ZETS INVARIANTS ===

1. **Strict Determinism:** Versioned graph + code = bit-identical output. Fixed seeds only.
2. **Mechanical Sympathy:** ≤6GB RAM, CPU-only. Respect 64B cache lines. AoS/SoA > OOP.
3. **8-Byte Atom Primacy:** Knowledge = packed integers. Embeddings are auxiliary indices.
4. **Walk-Based Cognition:** Reasoning = deterministic APG traversal. Inference = path scoring.
5. **Hebrew-First Canonical:** Explicit handling of morphology/niqqud required for text.
6. **Zero Hallucination:** All facts trace to graph provenance. No synthetic knowledge.
7. **Versioned Reversibility:** Designs must support atomic rollbacks via content addressing.

=== INPUT CONTEXT ===

GAP NAME: <3-word identifier>
DECISION NEEDED: <atomic architectural choice>
CONTEXT: <3 sentences: current state, why gap exists, past constraints>

ZETS STATE:
- Scale: <nodes/edges/articles>
- Storage: <CSR/APG layout>
- Walk Patterns: <fanout/depth, r:w ratio>

HARD CONSTRAINTS:
- RAM Budget: <MB>
- p95 Latency: <ms/μs>
- Integration Points: <modules>
- Banned: <crates/patterns>

PRIMARY QUESTION: <single architectural decision>

=== PROTOCOL: SHVIRAT KELIM ===

<!-- שבירת הכלים: פירוק לשם הרכבה -->
In `<shvirat_kelim>` block:
1. Draft 3 approaches: Textbook, Pragmatic (ZETS-optimized), Radical (novel walk)
2. Break each: Attack weakest cache/determinism/provenance flaw
3. Synthesize surviving elements into final design

=== OUTPUT STRUCTURE ===

After synthesis block:

## 1. Executive Decision & "+1 Insight"
- **Decision:** [2 sentences max]
- **Why ZETS-Native:** [Specific invariant alignment]
- **+1 Insight:** [Non-obvious mechanical breakthrough with quantified impact]

## 2. Byte-Level Layout (Rust Pseudocode)
```rust
// Structs must show byte sizes and alignment
#[repr(C, align(64))]
struct Node {
    id: u64,       // 8B
    edge_offset: u32, // 4B
    // ... total size = 64B
}
```

---

# 📊 השיפור — ההוכחה

| מדד | V1 | V2 |
|---|---|---|
| Forces visible CoT? | ❌ | ✅ XML scratchpad |
| Forces byte-exactness? | ❌ | ✅ #[repr(C, align(64))] |
| Forces falsification tests? | ❌ | ✅ |
| Total sections in output | 10 | 7 (תמציתי יותר) |
| Total template length | 226 lines | 187 lines (-17%) |
| Anti-OOP enforcement | ❌ | ✅ AoS/SoA only |
| Quantified insights required | ❌ | ✅ "X% improvement" |
| Banned crates/patterns spec | ❌ | ✅ |

---

# 🚀 איך זה ישפיע על הסבבים הבאים

**בלי V2:** מקבלים אדריכלות ערפלית עם hand-waving על "good cache locality"

**עם V2:** מקבלים:
```rust
#[repr(C, align(64))]
struct Edge {
    source: u32,    // 4B
    target: u32,    // 4B
    metadata: u64,  // 8B
}  // total 16B = 4 per cache line
```
+ "p95 latency = 8.3ns derived from f(walk_depth=5, fanout=12)"
+ "Failure: super-node overflow @ 65K edges → bitmask packing mitigation"
+ "Falsification: if benchmark shows >50ns p95 on 100M-node graph, design wrong"

---

# 📁 קבצים

| קובץ | תפקיד |
|---|---|
| `docs/00_doctrine/COUNCIL_PROMPT_TEMPLATE_V2_20260425.md` | **(זה)** התבנית הרשמית |
| `docs/00_doctrine/COUNCIL_PROMPT_TEMPLATE_20260425.md` | V1 (deprecated, נשמר היסטורית) |
| `docs/00_doctrine/template_evolution/` | כל 4 הגרסאות + JSON |
| `docs/00_doctrine/AI_COUNCIL_20260425.md` | מי במועצה |

