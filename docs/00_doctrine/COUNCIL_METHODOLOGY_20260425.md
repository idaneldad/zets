# 🏛️ Council Methodology — 7-Iteration Refinement Protocol

**תאריך:** 25.04.2026  
**מטרה:** השיטה הרשמית לעיבוד מסמכים ארכיטקטוניים גדולים דרך מועצת ה-AI.  
**איך נבנתה:** הצעה ראשונית שלי → שבירת כלים של 5 ארכיטקטים (Claude Opus 4.7, GPT-5.5, Gemini 3.1 Pro, DeepSeek R1, Cogito 671B) → סינתזה.

---

# 🎯 הקונצנזוס מ-5 הארכיטקטים

## כולם מסכימים על:
1. **Synthesizer bias אמיתי** — Claude לא יכול להיות הסינתזיסט היחיד
2. **לא להכניס raw responses ל-prompt** — context degradation, "Lost in the Middle"
3. **לשמור attribution במחלוקות** — מי אמר מה, לא קונצנזוס אנונימי
4. **Adversarial pairing רק על מחלוקות מפתח** — לא על הכל
5. **לא לדלג על cheap models** — הם תופסים failure modes שונים
6. **דרושה traceability** — לא רק comments, אלא decision ledger

## הציונים שניתנו לhצעה המקורית
- **GPT-5.5: 8/10** — "lacks issue/decision ledger"
- **Claude Opus 4.7: 7.5/10** — "lacks adversarial stress-testing"
- **DeepSeek R1: 7/10** — "underprotects minority views"
- **Cogito 671B: 7/10** — "flattens nuance"
- **Gemini 3.1 Pro: 7/10** — "context-churn from angle changes"

---

# ⭐ 5 התרומות הייחודיות (כל ארכיטקט הוסיף משהו אחר)

| ארכיטקט | התוספת המבריקה |
|---|---|
| **Claude Opus 4.7** | **Red Team iteration** — 3-4 מודלים מקבלים: "ZETS נכשל קטסטרופלית ב-2045. מה היה הסימן בspec שאויש?" |
| **GPT-5.5** | **Issue/Decision Ledger** — כל critique הופך ל-issue עם section ref, models, claim, severity, patch, accept/reject, rationale |
| **Gemini 3.1 Pro** | **Mutation Protocol** — כל model חייב להוציא `diff` blocks או line-numbers, לא הצעות פילוסופיות |
| **DeepSeek R1** | **Dissent Scorecard** — top 3 unresolved disputes + "minority report" quote (rotating לפי iteration) |
| **Cogito 671B** | **Confidence scoring** — כל claim מקבל 0-100 confidence ע"י המודל. Forensic patterns emerge. |

---

# 📐 השיטה הסופית — שילוב של כל החמישה

## שלב 0 — Pre-flight Setup

לפני שמתחילים את 7 האיטרציות:

```
✅ AGI.md is in clean state in git (current: line 4530)
✅ Issue Ledger initialized (empty CSV/JSON)
✅ Output directory: docs/40_ai_consultations/master_council/iter_<N>/
✅ Model pool defined (14 models, see AI_COUNCIL_20260425.md)
```

## שלב 1 — Per-Iteration Output Format (חובה לכל מודל)

כל מודל מחזיר structured output:

```json
{
  "iteration": N,
  "model": "claude-opus-4-7",
  "critical_issues": [
    {
      "id": "ISS-NNN",
      "agi_section": "§5.2",
      "agi_lines": [234, 250],
      "claim": "...",
      "severity": "critical|important|nice-to-have",
      "confidence": 85,           // ← Cogito's contribution
      "proposed_patch": "...",    // ← Gemini's mutation protocol
      "hidden_assumption": "...",
      "strongest_self_objection": "...",
      "validation_test": "..."    // ← falsifier
    }
  ],
  "minority_report": "...",       // ← DeepSeek's contribution
  "iteration_focus_response": "..." // direct answer to this iter's angle
}
```

## שלב 2 — Synthesis Format (חובה אחרי כל איטרציה)

```markdown
# Iteration N Synthesis

## Consensus Block (1K tokens)
- [Claim A] (agreed by: 12/14 models, avg confidence 82)
- [Claim B] (agreed by: 9/14, avg confidence 71)
...

## Disagreements Block (1.5K tokens, ATTRIBUTED!)
### Disagreement 1: <topic>
- **Position A** (Claude, GPT-5.5, Cogito): "..." conf=85
- **Position B** (DeepSeek, Qwen, GLM): "..." conf=78
- **Resolution path:** ...

## Minority Report (rotating, 200 tokens)
"Quote from least-consensus model": _Why this matters_

## Dissent Scorecard
| Dispute | Pro | Con | Iter resolved? |

## Issue Ledger Updates
- ISS-NNN: status changed to ACCEPTED, patch applied to §X
- ISS-MMM: NEW, deferred for iter X+1
- ISS-LLL: REJECTED (rationale: ...)

## Pointer to Raw
docs/40_ai_consultations/master_council/iter_N/
```

## שלב 3 — איטרציה Sequence (7 שלבים + Red Team + Final)

```
┌─────────────────────────────────────────────────────────┐
│ Iter 1: BROAD SURVEY                                     │
│   Models: ALL 14                                         │
│   Angle: "Loving parent — what's right, wrong, missing"  │
│   Output: Issue Ledger v1                                │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 2: FEASIBILITY (moved early per Claude)             │
│   Models: TOP 7                                          │
│   Angle: "Kill unimplementable ideas — what literally    │
│         cannot work on 6GB laptop?"                      │
│   Output: Issue Ledger v2 with feasibility scores        │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 3: WEAKEST GAPS DEEP DIVE                           │
│   Models: ALL 14 + adversarial pairs on TOP 3 disputes   │
│   Angle: "How do we close gaps with score < 7/10"        │
│   Output: Issue Ledger v3 + adversarial resolutions      │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 4: FORWARD-LOOKING (5-30 years)                     │
│   Models: TOP 7 + 1 contrarian (prompted to attack)      │
│   Synthesizer: GPT-5.5 (NOT Claude — bias rotation!)     │
│   Angle: "Refine §28 roadmap. What 30-year risk         │
│         did §28 miss?"                                   │
│   Output: Issue Ledger v4                                │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 5: CONTRADICTIONS / CONSISTENCY                     │
│   Models: ALL 14                                         │
│   Angle: "Find internal contradictions. Output diffs."   │
│   Mutation Protocol enforced — every issue = diff       │
│   Output: Issue Ledger v5 + applied patches             │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 6: COMPETITOR DIFFERENTIATION                       │
│   Models: TOP 7                                          │
│   Angle: "Why will ZETS still matter when GPT-7,        │
│         Claude 6, Gemini 5 exist? Be specific."          │
│   Output: Issue Ledger v6                                │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 6.5: RED TEAM (Claude's contribution!)              │
│   Models: 3-4 models assigned ADVERSARIAL role           │
│   Angle: "ZETS failed catastrophically in 2045.          │
│         Reverse-engineer: what was the visible warning   │
│         in this spec that everyone ignored?"             │
│   Output: Catastrophic failure scenarios + mitigations  │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 7: INTEGRATION PUSH TO 10/10                        │
│   Models: STRONGEST 5 (Opus, GPT-5.5, Gemini, R1, K2.6) │
│   Angle: "Final pass. Score 1-10. What single change    │
│         pushes to 10/10?"                                │
│   Synthesizer: GPT-5.5 audit + Claude finalization      │
│   Output: Final Issue Ledger                             │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Iter 8: GENIUS SYNTHESIS (Claude using ALL context)     │
│   - Read all 7 iterations + all raw responses            │
│   - Apply Mutation Protocol patches that survived 7      │
│   - Resolve remaining disputes by best-evidence rule     │
│   - Final AGI.md v2.0                                    │
│   - GPT-5.5 audit pass before publication                │
└─────────────────────────────────────────────────────────┘
```

## שלב 4 — Bias Mitigation (קריטי)

1. **Claude לא יחיד** — Iter 4 synthesizer = GPT-5.5
2. **Final audit by non-Claude** — Iter 8 result audited by GPT-5.5 לפני commit
3. **Attribution preserved** — שמות מודלים נשמרים בdisagreements (לא אנונימי)
4. **Confidence scores logged** — patterns יכולים לחשוף הטיות

---

# 💰 הערכת עלות וזמן

## Per Iteration
- 14 models × avg 6K tokens (input AGI.md+synthesis, output 4K) 
- Mix של pricing: ~$1.50-3.00 per iteration
- Time parallel: 3-5 דקות per iteration

## Total
- 7 איטרציות + Red Team + Final
- **עלות: $15-25**
- **זמן: 30-50 דקות compute time**
- **גודל data: ~5MB raw responses + 50KB synthesis**

---

# 🤖 על השאלה "להפעיל Claude Code ברקע?"

**כן, מומלץ.** הסיבות:
1. Claude Code יכול לרוץ אוטונומית 30-50 דקות בלי אינטראקציה
2. הוא יכול לחזור ל-git, להריץ scripts, לבדוק אם איטרציה הושלמה
3. הוא יכול **לקרוא את הsynthesis של iter N לפני שמריץ iter N+1**
4. בזמן הזה אתה ואני יכולים לעבוד על דברים אחרים

**ההצעה המעשית:** 
- **Session הזה (אנחנו):** השיטה מתועדת, scripts מוכנים, Iter 1 מתוכנן
- **Claude Code ברקע:** מריץ את 7 האיטרציות, עוקב, מסנתז
- **Session הבא (אנחנו):** מקבלים את התוצר, אני עושה את ה-Iter 8 הגאוני

---

# 📁 Scripts הנדרשים

```
scripts/
├── council_iteration.py        # מריץ איטרציה N
├── council_synthesis.py        # מסנתז תשובות, מעדכן Issue Ledger
├── council_orchestrator.py     # מריץ את כל ה-7 ב-sequence
└── issue_ledger_manager.py     # CRUD ל-Issue Ledger
```

יוצרים אותם ב-session הבא.

---

# 🎯 ההמלצה הסופית

הצעדים:

1. **NOW (סוף הsession):** השיטה ב-git ✅
2. **Session 2:** כתיבת 4 הscripts + Iter 1 demo run
3. **Claude Code ברקע (אם תרצה):** הרצת Iter 1-7 לפי השיטה
4. **Session 3:** הסינתזה הגאונית הסופית (Iter 8) ע"י Claude

עלות סופית מוערכת: **$20-30**.
זמן compute: **45-60 דקות**.
תוצר: **AGI.md v2.0 — work of art**.

