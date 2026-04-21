# ZETS Empathic Response Layer — User Model + Intent + Delivery

**Date:** 21.04.2026
**Status:** Design SPEC — awaiting Idan decisions
**Influences:** Idan's 14-step sales consultant methodology
**Engineering target:** Make ZETS understand WHO asks, not just WHAT they ask.
**Position in roadmap:** New Sprints J, K, L — added after Cognitive Tree (D)

---

## 0. The core insight (quoted from Idan)

> "אם הוא ADHD הוא מחפש סיפוקים מהירים, אם דיסלקט עושה הסקת מסקנות מהירה,
>  אם אוטיסט יבדוק שהכל יסודי"
>
> "40% מענה לכאב, 40% פרקטי, 20% מפתיע מחוץ לקופסה"
>
> "יש מקרים שיש תשובה מיידית אבל נראה כאילו המערכת חשבה, וזה מייצר תחושה
>  מקצועית יותר"

The engineering insight: **A good answer without understanding the asker
is a mediocre product. A mediocre answer with deep understanding of the
asker is a great product.**

This is the moat. Every LLM gives the same answer to the same question.
ZETS will give the right answer for THIS PERSON asking THIS QUESTION.

---

## 1. Critical honesty — what I can and cannot do

Before designing: three honest admissions.

### What I CAN engineer
- Detect **behavioral patterns from interaction**: asks short, scrolls fast,
  rephrases questions, uses slang. These are observable and reliable.
- Build a **User Profile synset** in the graph: stable, queryable, privacy-respecting.
- **Adjust response style** based on observed patterns and explicit preferences.
- **Track relationship history**: past interactions, resolved/unresolved issues.

### What I CANNOT engineer
- Diagnose cognitive conditions (ADHD, dyslexia, autism) from text.
  That's clinical, unethical, and technically unreliable.
- Guess "is this person a decision-maker" from name alone.
  Can infer from explicit statements or external data, not from text patterns.
- Predict "deal breakers" without explicit signals.
  Can track stated red lines; cannot read minds.

### What I WILL do instead
Replace diagnostic labels with **observed preference tags**:
- Not: "User has ADHD"
- Yes: "User prefers bullet-point responses under 200 words"

- Not: "User is autistic"
- Yes: "User asks for complete explanations with all edge cases"

This keeps the system ethical, accurate, and useful.

---

## 2. The four-layer architecture

```
┌─────────────────────────────────────────────────────────┐
│  LAYER 1: USER MODEL                                    │
│    Who is asking — profile, patterns, history           │
│    Implemented in Sprint J                              │
└─────────────────────────────────────────────────────────┘
                         ↓ informs
┌─────────────────────────────────────────────────────────┐
│  LAYER 2: INTENT DECODER                                │
│    What they really want — pain, wants, signals         │
│    Implemented in Sprint K                              │
└─────────────────────────────────────────────────────────┘
                         ↓ shapes
┌─────────────────────────────────────────────────────────┐
│  LAYER 3: BEAM TREE (Cognitive Tree from Sprint D)      │
│    Finds the answer — 7×7 branches + bridges           │
│    Extended to weight candidates by user fit           │
└─────────────────────────────────────────────────────────┘
                         ↓ produces
┌─────────────────────────────────────────────────────────┐
│  LAYER 4: RESPONSE CRAFTER                              │
│    How to deliver — style, length, timing, reasoning    │
│    Implemented in Sprint L                              │
└─────────────────────────────────────────────────────────┘
```

Each layer is a self-contained sprint. Each adds value even if later
layers aren't built yet.

---

## 3. LAYER 1 — USER MODEL

### 3.1 The User Profile synset

```rust
// User synset IDs: 20_000_000..
pub struct UserProfile {
    pub id: SynsetId,

    // Identity (from explicit enrollment)
    pub display_name: String,
    pub email_hash: [u8; 16],   // not raw email - privacy
    pub primary_language: LangCode,

    // Organization context (from CRM or explicit)
    pub company_synset: Option<SynsetId>,
    pub role_synset: Option<SynsetId>,      // "CEO", "engineer", "pediatrician"
    pub seniority: Option<Seniority>,        // Junior / Mid / Senior / Exec
    pub decision_power: Option<DecisionRole>,// Decider / Influencer / Gatekeeper

    // Demographics (optional, explicit only)
    pub gender: Option<Gender>,
    pub age_range: Option<AgeRange>,         // never exact age

    // Observed communication style (from interaction history)
    pub style_tags: Vec<StyleTag>,

    // Observed cognitive preferences (OBSERVATIONS, not labels)
    pub cognitive_tags: Vec<CognitiveTag>,

    // Red lines (explicit only — never inferred)
    pub red_lines: Vec<RedLine>,

    // Relationship metadata
    pub first_contact_at: u64,
    pub total_interactions: u32,
    pub satisfaction_signal: i16,   // -100..+100, from feedback
    pub last_unresolved_issue: Option<IssueRef>,
}
```

### 3.2 Style and cognitive tags (observation-based)

```rust
pub enum StyleTag {
    // Communication register
    Formal,           // "Dear Sir/Madam"
    Professional,     // business tone, no jargon
    Casual,           // "Hey", "תודה מטורף"
    Slang,            // heavy informal markers

    // Language markers
    UsesEmojis,
    UsesHebrew,
    UsesEnglish,
    MixesLanguages,

    // Courtesy markers
    ThankfulFrequently,
    DirectNoPleasantries,
}

pub enum CognitiveTag {
    // Length preferences (OBSERVATIONS from behavior)
    PrefersBrevity,              // replies stop reading past 200 words
    PrefersDepth,                // asks follow-ups for more detail
    PrefersBullets,              // engages more with lists

    // Response speed preferences
    PrefersFastAnswers,          // doesn't read long chain-of-thought
    TolerantOfReasoning,         // explicitly thanks for explanation

    // Decision style
    AnalyticalDecisionMaker,     // asks for numbers, comparisons
    IntuitiveDecisionMaker,      // goes with gut, resists data dumps

    // Detail orientation
    WantsAllEdgeCases,           // asks "what if X happens"
    WantsMainPathOnly,           // doesn't engage with edge cases

    // Pace
    SeeksQuickSatisfaction,      // rewards instant answers with +feedback
    SeeksThoroughness,           // rewards careful exploration
}
```

**Critical:** these are all **observations from behavior**, never self-labels
or inferred medical conditions. They accumulate over interactions with
weighted decay (old observations matter less).

### 3.3 Red lines

```rust
pub enum RedLineKind {
    Ethical(String),     // "cannot work with firearms industry"
    Legal(String),       // "cannot share patient data"
    Financial { max: u64, currency: String },
    Temporal { deadline: u64 },
    Relational(String),  // "don't involve X person"
}

pub struct RedLine {
    pub kind: RedLineKind,
    pub source: RedLineSource,   // Stated / Inferred-from-context / Default
    pub confidence: u8,          // 0..100
}
```

Red lines stored as synsets with `RedLineOf` edges to the user. Checked
before every response.

### 3.4 Cross-user pattern detection

Idan's example: "if same company name, or celebrity name, cross-reference."

```rust
pub fn find_related_users(
    graph: &Graph,
    user: &UserProfile,
) -> Vec<RelatedUser> {
    // Same company → shared context
    // Same industry → shared domain
    // Similar name → worth noting but low confidence
    // Explicit referral ("John sent me") → high confidence
}
```

Important: **never make assumptions from name alone**. "Steven Spielberg
from Acme Widgets" is probably not the director. Use it as soft hint,
not as source of truth.

---

## 4. LAYER 2 — INTENT DECODER

Idan's steps 11-13 translated to code.

### 4.1 The decoder produces an IntentVector

```rust
pub struct IntentVector {
    // The surface ask (what they typed)
    pub surface_request: String,
    pub surface_entities: Vec<SynsetId>,

    // The underlying pain (step 12)
    pub pain_signals: Vec<PainSignal>,

    // Hidden wants (what they didn't say — step 13)
    pub inferred_wants: Vec<InferredWant>,

    // Communication signals (step 13 deep)
    pub specificity: Specificity,    // Concrete / Vague / Metaphorical
    pub emotional_register: Emotion,  // Neutral / Frustrated / Excited
    pub exaggeration_score: u8,       // 0..100; "עולם ומלואו" raises this

    // Deal-breaker candidates (step 13)
    pub likely_deal_breakers: Vec<SynsetId>,
}

pub enum Specificity {
    Concrete,       // "I need 100 units by Tuesday"
    Vague,          // "something good"
    Metaphorical,   // "עולם ומלואו", "piece of cake"
    Mixed,
}
```

### 4.2 Pain signal detection

Pain signals are phrases that indicate underlying distress:
- "I've been trying for weeks"
- "אף אחד לא עוזר לי"
- "I'm desperate"
- Repeated question phrasings within one session
- Negative emotional words accumulating

Implementation: a small curated list of phrase patterns (~200 entries
per language) with pain-category synsets.

### 4.3 Specificity classification

```rust
pub fn classify_specificity(text: &str, lang: LangCode) -> Specificity {
    let numeric_ratio = count_numbers(text) as f32 / text.len() as f32;
    let metaphor_count = count_metaphor_markers(text, lang);
    let vague_words = count_vague_words(text, lang);

    if numeric_ratio > 0.05 || has_specific_entities(text) {
        Specificity::Concrete
    } else if metaphor_count > 2 {
        Specificity::Metaphorical
    } else if vague_words > 3 {
        Specificity::Vague
    } else {
        Specificity::Mixed
    }
}
```

Idan's insight: **"דימויים גדולים על דברים לא גדולים = לא ממוקד פתרון"**.
If a user asks for "a simple spreadsheet" but uses metaphors like
"עולם ומלואו", flag specificity as Metaphorical and ask clarifying
questions before committing.

### 4.4 Exaggeration detection

```rust
pub fn detect_exaggeration(intent: &IntentVector, user: &UserProfile) -> u8 {
    // Look for:
    // - Superlatives far from context ("best ever", "never", "everything")
    // - Quantities far from reasonable ("a million", "infinity")
    // - Historical pattern: does this user typically exaggerate?
    //   (from user profile)
}
```

Don't treat exaggeration as dishonesty. Treat it as a signal that the
factual claim should be weighted lower in the beam tree.

### 4.5 Clarifying question generator

Step 11 — "what do you mean" questions.

When IntentVector has:
- specificity = Vague, OR
- ambiguous entities with multiple synsets, OR
- contradictory signals

→ System generates 1-3 clarifying questions **before** running full beam tree.

```rust
pub fn generate_clarifications(
    intent: &IntentVector,
    user: &UserProfile,
    graph: &Graph,
) -> Vec<ClarifyingQuestion> {
    // Adapt tone to user style_tags
    // Prefer 1 question for PrefersBrevity users
    // Up to 3 for SeeksThoroughness users
}
```

---

## 5. LAYER 3 — BEAM TREE WITH USER FIT

### 5.1 Extension to Sprint D

Sprint D beam tree scored candidates by:
- Edge weight (B1)
- Path confidence (B2)
- Source tier (B3)

**Sprint K adds a fourth dimension: User Fit (B4).**

```rust
pub fn score_candidate_with_user(
    candidate: &Candidate,
    user: &UserProfile,
    intent: &IntentVector,
) -> Score {
    let base = b1_edge_weight(candidate).min(
               b2_path_confidence(candidate)).min(
               b3_source_tier(candidate));

    let user_fit = compute_user_fit(candidate, user, intent);

    // Combine: min of quality dimensions × user fit multiplier
    base * user_fit
}

pub fn compute_user_fit(
    candidate: &Candidate,
    user: &UserProfile,
    intent: &IntentVector,
) -> f32 {
    let mut fit: f32 = 1.0;

    // Does candidate match communication style?
    if candidate.style_requires_formal() && user.has_tag(StyleTag::Casual) {
        fit *= 0.7;
    }

    // Does candidate length match preference?
    if candidate.answer_length() > 500 && user.has_tag(CognitiveTag::PrefersBrevity) {
        fit *= 0.6;
    }

    // Does candidate respect red lines?
    if candidate.violates_any(&user.red_lines) {
        fit = 0.0;  // hard block
    }

    // Does candidate include pain acknowledgment?
    if !intent.pain_signals.is_empty() && !candidate.acknowledges_pain() {
        fit *= 0.8;
    }

    fit.clamp(0.0, 1.0)
}
```

### 5.2 The 40/40/20 weighting (Idan's formula)

After beam produces candidates, partition into three pools:

```rust
pub fn partition_candidates(
    candidates: &[ScoredCandidate],
) -> ThreePools {
    ThreePools {
        // 40%: directly address pain/need/question
        pain_solvers: candidates.iter()
            .filter(|c| c.addresses_pain())
            .take_top(0.40),

        // 40%: practical and executable
        practical: candidates.iter()
            .filter(|c| c.is_feasible())
            .take_top(0.40),

        // 20%: surprising value-add (upsell/creative)
        surprising: candidates.iter()
            .filter(|c| c.is_outside_expected())
            .take_top(0.20),
    }
}
```

Then **render** all three pools in the final response, labeled clearly:
- Primary answer (from pain_solvers)
- Practical alternatives (from practical)
- "You might also consider" (from surprising)

This is the Idan differentiation: most chatbots give ONE answer. ZETS
gives the answer they asked for PLUS what they should have asked for.

---

## 6. LAYER 4 — RESPONSE CRAFTER

### 6.1 Style selection per user

```rust
pub struct RenderSpec {
    pub register: Register,         // Formal / Professional / Casual
    pub max_length_words: usize,    // 100 for PrefersBrevity, 500 for PrefersDepth
    pub use_bullets: bool,          // true if PrefersBullets
    pub include_reasoning: bool,    // visible chain-of-thought?
    pub reasoning_location: ReasoningLoc,
    pub include_sources: bool,
    pub language: LangCode,
}

pub enum ReasoningLoc {
    Inline,       // reasoning in main response
    Collapsible,  // "click to see reasoning"
    Hidden,       // only final answer visible
    AttachedDoc,  // separate link/doc
}
```

### 6.2 The "collapsible reasoning pane" Idan mentioned

Idan's insight: **"יש יוזרים שיראו בעומק טרחנות"**.

Solution: every response has two tracks:
1. **Headline answer** — concise, matches user preference
2. **Deep reasoning pane** — full beam tree output, closed by default

For PrefersBrevity: headline only, reasoning link at bottom
For SeeksThoroughness: headline + summary + "see full reasoning"
For AnalyticalDecisionMaker: headline + numerical breakdown inline

### 6.3 The "appear to think" pattern (Idan step 8)

> "יש מקרים שיש תשובה מיידית אבל נראה כאילו המערכת חשבה, וזה מייצר
>  תחושה מקצועית יותר"

Two honest patterns:

**Pattern A (always-honest):** Return time = actual compute time.
Fast answers arrive fast. This is fine for analytical users who respect speed.

**Pattern B (perception layer):** For cached/instant answers to complex
questions, introduce a deliberate delay (200-800ms) + a visible "thinking..."
indicator. This **matches user expectation** for the question's complexity.

**My engineering position:** Pattern B is acceptable IF:
- The system has genuinely considered the answer (not just cached blindly)
- The delay is proportional to question complexity
- User can opt out with a "fast mode" setting

Deceiving users about compute is bad. Matching pacing expectations is fine.

### 6.4 Proactive outreach (Idan steps 7, 9, 10)

When the system initiates contact:

```rust
pub fn craft_proactive_message(
    target: &UserProfile,
    reason: OutreachReason,
    relationship_bridge: Option<Bridge>,
) -> ProactiveMessage {
    let best_time = infer_best_contact_time(target);
    let channel = pick_channel(target);  // email/slack/whatsapp
    let opening = craft_opening(target, relationship_bridge);
    // "I saw you at ClawCon" or "You're a customer of X, and we..."

    ProactiveMessage {
        channel,
        scheduled_for: best_time,
        opening,
        body: compose_outreach_body(target, reason),
    }
}
```

Relationship bridge: a **graph edge connecting the reason for contact
to something the recipient cares about**. Without a bridge, don't send.
(Idan step 10.)

---

## 7. Alternative flow when we CAN'T help (Idan step 14)

```rust
pub fn handle_cannot_fulfill(
    request: &IntentVector,
    user: &UserProfile,
    blocking_criteria: &[BlockingCriterion],
) -> AlternativeResponse {
    // 1. State clearly what blocks the original request
    // 2. Offer three kinds of alternatives:
    //    a. Same goal, relaxed criteria ("if you could extend deadline...")
    //    b. Similar goal, adjacent solution ("we can't do X but we can do X'")
    //    c. Different goal that gives partial value

    // NEVER just say "I can't help with this."
    // ALWAYS leave user with something useful or a clear next step.
}
```

This is **customer retention logic**, not just politeness. A user who
leaves empty-handed is gone. A user who leaves with "here's what I CAN
do" remembers.

---

## 8. Data model additions

### New synset types
- `UserProfile` (IDs 20_000_000..)
- `Company` (IDs 21_000_000..)
- `StyleTag`, `CognitiveTag` (reserved system synsets)
- `RedLine`
- `PainCategory`
- `Bridge` (relationship connector)

### New relations
- `UserIs` / `WorksAt` / `ReportsTo`
- `ObservedPattern` — attaches style/cognitive tag to user with confidence
- `HasRedLine` — connects user to red line
- `InteractedIn` — user to session
- `ReferredBy` — user to user (trust path)
- `BridgedTo` — for proactive outreach

### Privacy constraints
- User data encrypted at rest (user overlay layer)
- GDPR: user can request `forget_user(SynsetId)` → overlay deleted
- Base graph untouched (no user data in base pack)
- No user-to-user data leakage across agents

---

## 9. The three sprints

### Sprint J: User Model (1 week)
- `UserProfile` struct + graph integration
- Explicit enrollment flow (API: create_user, update_profile)
- Behavioral observation engine (collect tags from interactions)
- Red lines management
- Cross-user pattern detection (company/industry/referrals)
- Privacy: encryption + forget()

### Sprint K: Intent Decoder (1 week)
- `IntentVector` struct
- Pain signal detector (200-phrase curated lists per lang)
- Specificity classifier
- Exaggeration detector
- Clarifying question generator
- Integration with beam tree (produces user-weighted scoring)

### Sprint L: Response Crafter (1-2 weeks)
- `RenderSpec` struct
- Style-adapted templates (every response-type × every register)
- Collapsible reasoning pane layout
- Pacing/delay control (ethical implementation)
- Proactive outreach crafter
- Cannot-fulfill alternative generator
- 40/40/20 partition renderer

**Total: 3-4 weeks of focused work.**

---

## 10. Critical decisions for Idan

### Decision X: How deep does User Model go?

**(X1) Minimal:** name, language, company. Observations only from current session.
**(X2) Medium:** as X1 + persistent behavioral tags across sessions + red lines.
**(X3) Maximum:** as X2 + cross-user patterns + proactive outreach model.

My recommendation: **X2** for V1, **X3** later when we have enough data.

### Decision Y: Cognitive tag source policy

**(Y1) Pure observation:** tags only from behavior, user cannot self-label.
**(Y2) Hybrid:** user can self-declare in settings, observations refine.
**(Y3) Self-only:** user fills questionnaire, no observation.

My recommendation: **Y1**. Self-declarations are unreliable, observation is
factual. Users who want override can do so via explicit red lines.

### Decision Z: Default behavior when user profile is empty

**(Z1) Default to formal/thorough/safe:** treat unknown user as senior decision-maker.
**(Z2) Default to neutral:** medium register, medium depth, no assumptions.
**(Z3) Default to casual/brief:** assume modern user, offer more on request.

My recommendation: **Z2**. Adapt quickly based on first 2-3 turns.

### Decision AA: Pacing/delay ethics

**(AA1) Never delay beyond actual compute time.** Honest speed.
**(AA2) Match user expectation with visible "thinking":** 200-800ms for complex.
**(AA3) Per-user setting:** fast-mode vs thoughtful-mode.

My recommendation: **AA3**. Let user decide. Default to AA1.

### Decision BB: 40/40/20 strict vs adaptive

**(BB1) Strict:** always 40/40/20 partition, never deviate.
**(BB2) Adaptive:** 40/40/20 default, but collapse to 100% practical for
urgent pain signals, or 60% surprising for exploratory queries.

My recommendation: **BB2**. Rules with escape hatches beat rigid formulas.

---

## 11. Integration with existing work

### With Sprint D (Cognitive Tree)
Sprint K's `IntentVector` feeds into beam-tree seed selection.
Sprint K's user-fit score becomes the fourth probability dimension.
No changes needed to Sprint D's core algorithm.

### With Sprint H (Sessions)
User profile edges live in session-adjacent synsets.
Session context directly uses profile tags for disambiguation
(already planned in Sprint H, now formalized).

### With Sprint E (Composition)
Sprint E provides templates. Sprint L adds style/register selection layer.
E + L together produce the final text.

### With Sprint F (Feedback)
Feedback updates both:
- Edge weights (Sprint F original scope)
- User profile tags (newly added via Sprint K)

Thumbs-up on "brief answer" → strengthens PrefersBrevity tag for user.

---

## 12. Sequencing (updated roadmap)

Revised full path:

| Sprint | Name | Duration |
|--------|------|----------|
| A | CLI + iterator | 1 week |
| B | Query Planner + Multi-seed | 1 week |
| C | PreGraphBuilder + data files | 1 week |
| H | Sessions + Disambig | 1 week |
| **J** | **User Model (new)** | 1 week |
| **K** | **Intent Decoder (new)** | 1 week |
| D | Cognitive Tree (beam 7×7) | 4-6 weeks |
| E | Composition | 1 week |
| **L** | **Response Crafter (new)** | 1-2 weeks |
| F | Feedback Learner | 1 week |
| G | Tool Registry | 1 week |
| I | Cloud Relay | 1 week |

**J + K move BEFORE D because** the user model and intent vector are
inputs to the beam tree's scoring function. Building D without them
means re-architecting D later.

**L moves after E because** L is style overlay on top of composition templates.

**Total: 16-20 weeks** to full AGI-like ZETS with empathic response.

---

## 13. Why this is the moat

Every LLM can answer "what's the weather." Few understand:
- That this user asks at 7am every day (context = commute planning)
- That they're PrefersBrevity (don't give 10-day forecast)
- That they're in Tel Aviv (don't answer in Fahrenheit)
- That they have red line "no push notifications before 6am" (don't offer auto-alerts)

The combination of persistent graph + user model + intent decoder +
beam tree + style crafter = **an assistant that actually knows them**.

This is what Idan called "an agent that's not a tool but a teammate."
The architecture above is how to engineer it.

---

## 14. Honest risk statement

**This is 3-4 weeks of additional work** on top of the already-long roadmap.

**But it changes what ZETS is.** Without User Model + Intent + Crafter,
ZETS is a good knowledge retrieval engine. With them, it's a *personalized
consultant* that happens to run on a Pi.

My strong recommendation: **build it**. If the extra time is a concern,
we can cut Sprint L's proactive outreach to V2 and keep Sprints J+K as
the must-haves.

---

## 15. Final ask

Answer these five decisions (X, Y, Z, AA, BB in section 10), and I'll
write the detailed Sprint J + K + L briefs to add to the tracker.

Without decisions → briefs will be speculative, code will drift.
With decisions → 3 more ready-to-execute sprints in the backlog.

---

**End of Empathic Response Layer spec.**
