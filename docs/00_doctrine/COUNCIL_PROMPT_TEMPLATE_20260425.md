# 🎯 תבנית ה-Master לפנייה למועצת ה-AI

**תאריך:** 25.04.2026  
**מטרה:** תבנית מאומתת (validated) לכל שאלה למועצת ה-AI — מבטיחה תשובות פנומנאליות.  
**איך נבנתה:** Meta-research על שאלה אמיתית (TMS Deep) דרך GPT-4o-mini + Gemini 2.5 Flash + Llama 3.3 70B. סינתזה של 3 perspectives.

**Claude משתמש בתבנית הזו תמיד כשפונה למועצה.**

---

# 🧠 העקרון המרכזי — "Shvirat Kelim Built Into The Prompt"

במקום לשלוח שאלה רגילה ולקבל תשובה רגילה, **התבנית מכריחה את המודל לעשות שבירת כלים פנימית:**

1. **לייצר 3 גרסאות תשובה במחשבה** — כאילו ב-temperatures שונים (conservative, balanced, exploratory)
2. **לשבור** — לזהות את החזק והחלש בכל גרסה
3. **לבנות מחדש** — סינתזה של החלקים הטובים = תיקון
4. **לדרג את עצמו** 1-10 + מה חסר ל-10
5. **להציע שאלת המשך** שתגיע ל-10

זה לא משנה את ה-temperature בפועל — אבל מאלץ את ה-LLM לחשב את **מרחב התשובה** ולא רק לשלוף את התשובה הראשונה.

---

# 📋 התבנית המלאה

```
=== ROLE & CONTEXT ===
You are a senior systems architect consulting on ZETS — a deterministic 
graph-native AGI engine. Treat this with the rigor of a peer review at 
a top systems conference.

=== ZETS INVARIANTS (non-negotiable) ===
- Deterministic (same input → same output, forever)
- Graph-native: 8-byte atoms in CSR + Article Path Graph
- Walks for reasoning (no continuous embeddings as primary mechanism)
- Hebrew-first canonical (other languages translate)
- Laptop-scale: 6GB RAM, CPU-only, no GPU
- Quantum-inspired (classical implementation of superposition/walks/interference)
- Zero hallucination on facts (graph is ground truth)
- User-in-control self-extension

=== THE GAP / QUESTION ===
GAP: <gap_name>

CONTEXT: <full_context_3_5_sentences>

CONSTRAINTS:
- Memory budget: <X MB / GB>
- Latency budget: <Y ms>
- Other: <integration with other gaps if any>

QUESTION: <one focused question, NOT 5 sub-questions>

=== EVALUATION CRITERIA — what makes this 10/10 ===
A 10/10 answer:
1. Names specific algorithms / data structures (no hand-waving)
2. Provides concrete numbers (memory MB, latency ms, accuracy %)
3. Identifies 2-3 failure modes with mitigations
4. Includes a "+1 subtle thing" — non-obvious insight that elevates good→phenomenal
5. Respects ALL ZETS invariants (or explicitly justifies any compromise)
6. ZETS-native (not just standard best practice applied)

=== INTERNAL THINKING PROTOCOL (mandatory) ===
Before you write your final answer, internally generate THREE versions:

  Version A — CONSERVATIVE (mental temperature ≈ 0.3)
    "What's the safest, most proven approach? Standard textbook solutions."
  
  Version B — BALANCED (mental temperature ≈ 0.7)
    "What's the pragmatic best-fit for ZETS specifically? Tradeoffs articulated."
  
  Version C — EXPLORATORY (mental temperature ≈ 1.0)
    "What's the unconventional angle? What would a kabbalistic systems thinker propose?"

Then perform shvirat kelim (breaking + tikkun):
  - Identify the strongest element in each version
  - Identify the weakest assumption in each version
  - Synthesize a FINAL answer that takes the strongest elements
    and discards the weakest assumptions

Your final answer is the synthesis. Do NOT show the three versions —
show only the final synthesized answer, but ensure it carries the
robustness of all three.

=== REQUIRED OUTPUT STRUCTURE ===

## Architecture
<Specific algorithms/data structures with names. Include:
 - Why these specifically (not alternatives)
 - How they fit ZETS invariants
 - Concrete numbers throughout>

## Concrete Numbers
| Metric | Value | Justification |
| Memory | X MB | ... |
| Latency | Y ms | ... |
| Accuracy | Z% | ... |
| <other> | ... | ... |

## Failure Modes (2-3, with mitigations)
1. <Failure mode 1>: <when it happens> → <mitigation>
2. <Failure mode 2>: <when it happens> → <mitigation>
3. <Failure mode 3>: <when it happens> → <mitigation>

## Trade-offs Made
<What was sacrificed and why. If nothing was sacrificed, you're hand-waving.>

## Anti-patterns (what NOT to do)
<2-3 things that look right but are wrong for ZETS>

## The "+1 Subtle Thing"
<The non-obvious insight that elevates this from good (7/10) to phenomenal (10/10).
 Must be specific, not platitude.>

## Self-Rating
**My answer scores: X/10**

Why X and not 10:
- <Specific gap 1>
- <Specific gap 2>

To reach 10/10, I would need:
- <Specific information / context I don't have>
- <Specific test/validation I cannot run>

## Follow-Up Question
**The single best question to ask next** (to me or to another expert):
"<exact question that would unlock the missing 10/10>"

=== LANGUAGE & LENGTH ===
- English (technical precision matters)
- 600-900 words
- Be specific. Hand-waving = wrong. Numbers > adjectives.
```

---

# 🔧 איך משתמשים בתבנית

## 1. במחקר ארכיטקטוני (7 מודלים במקביל)

```python
# Fill template variables
prompt = TEMPLATE.format(
    gap_name="Truth Maintenance System Deep Implementation",
    full_context="ZETS needs full TMS: provenance per edge, trust scoring per source...",
    constraints_memory="< 200 MB overhead",
    constraints_latency="< 5 ms per walk verification",
    question="What's the phenomenal TMS implementation for ZETS deterministic graph?"
)

# Send to 7 council members in parallel
council = pick_7_members(topic_type='architecture')
results = parallel_consult(council, prompt)

# Each returns structured output:
# - Architecture
# - Concrete numbers
# - Failure modes
# - Trade-offs
# - Anti-patterns
# - +1 subtle
# - Self-rating
# - Follow-up question

# Synthesize
final = shvirat_kelim_synthesis(results)
```

## 2. בקודינג (5 מודלים)

זהה לארכיטקטורה אבל:
- החליף "systems architect" ב-"senior Rust engineer"
- הוסיף "Code MUST compile, follow Rust 2024 edition idioms, use only stable features"
- בקש implementation בנוסף לארכיטקטורה

---

# 📊 למה זה עובד

| בלי תבנית | עם תבנית |
|---|---|
| מודל נותן תשובה אחת מ-mode התשובה הראשון | מודל מחפש 3 תשובות, בוחר הטובות |
| תשובות generic | תשובות ZETS-native (invariants enforced) |
| Hand-waving acceptable | Numbers required |
| חוסר מודעות לחולשות | Self-rating + gap analysis explicit |
| End of conversation | Follow-up question מאפשרת המשך |

---

# 🎯 דוגמת שימוש — TMS Deep (מהסבב הקודם)

**שאלה ישנה (בלי תבנית):** קיבלנו 2-3 רעיונות כלליים, שום self-rating.

**שאלה עם תבנית:** היינו מקבלים:
- Architecture עם schema מפורט
- 3 failure modes עם mitigations
- Trade-offs מודעים (latency vs accuracy)
- Anti-patterns (כמו "אל תשמור full provenance בקריאה — bloat")
- +1: "Lazy Bayesian update" המבריק שאני הצעתי
- Self-rating 8/10 עם follow-up: "what's the actual query pattern distribution?"

**תוצאה:** במקום סבב נוסף לסינתזה, מקבלים תשובה כמעט מלאה כבר בסבב הראשון.

---

# 🚦 כללי שימוש קריטיים

1. **תמיד למלא את כל המשתנים** — אל תשאיר `<context>` ריק
2. **השאלה אחת ויחידה** — אם יש 5 שאלות, תפצל ל-5 קריאות
3. **Concrete numbers בconstraints** — "low memory" = bad, "<200MB" = good
4. **Internal Thinking Protocol הוא חובה** — בלעדיו אין shvirat kelim
5. **Output Structure הוא חובה** — בלי זה אין השוואה בין מודלים

---

# 📁 איפה להשתמש בתבנית

| מסמך | תפקיד |
|---|---|
| `docs/00_doctrine/AI_COUNCIL_20260425.md` | מי במועצה |
| `docs/00_doctrine/COUNCIL_PROMPT_TEMPLATE_20260425.md` | **(זה)** איך לשאול |
| `docs/40_ai_consultations/<topic>/<gap>.json` | תוצאות raw |
| `docs/40_ai_consultations/<date>_SYNTHESIS.md` | סינתזה אחרי |

