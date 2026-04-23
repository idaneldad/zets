// mefaresh.rs — "מפרש" — Self-Interpretation Layer
// Cortex explains to ITSELF what's being asked before processing.
//
// Kabbalistic: בינה (Understanding) — not חכמה (raw data), but comprehension.
// "אם אין בינה אין דעת, אם אין דעת אין בינה" — Pirkei Avot 3:17
//
// What Claude does in <thinking>:
//   1. What are they asking? (paraphrase)
//   2. Why are they asking? (Kavana)
//   3. What do I need to answer? (resources)
//   4. What should the answer contain? (output spec)
//   5. What constraints apply? (restrictions)
//   6. What does success look like? (acceptance criteria)
//
// This module does the same for Cortex — BEFORE the pipeline runs.
// Position: after Kavana, before S0 lookup.
//
// DINIO Cortex V8.0 | 02.04.2026

use crate::kavana::{KavanaVector, Goal, KavanaSource};

// ============================================================
// THE INTERPRETATION — what Cortex "tells itself"
// ============================================================

#[derive(Debug, Clone)]
pub struct Interpretation {
    /// 1. מה שואלים — the core question in Cortex's own words
    pub what: String,

    /// 2. למה שואלים — purpose (from Kavana, enriched)
    pub why: WhyFrame,

    /// 3. מה צריך — resources needed to answer
    pub needs: Vec<ResourceNeed>,

    /// 4. מה צריך להיות בתשובה — output specification
    pub output_spec: OutputSpec,

    /// 5. מגבלות — what to avoid
    pub constraints: Vec<Constraint>,

    /// 6. מה ההצלחה — when is the answer "good enough"
    pub success_criteria: SuccessCriteria,

    /// 7. תוכנית — ordered steps to execute
    pub plan: Vec<PlanStep>,

    /// Meta: confidence in interpretation itself
    pub interpretation_confidence: f32,

    /// Meta: should we ASK the user to clarify?
    pub needs_clarification: bool,
    pub clarification_question: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WhyFrame {
    pub goal: Goal,
    pub deeper_why: String,       // "wants to order for an event" not just "Act"
    pub what_happens_after: String, // "will place order" / "will decide" / "just curious"
    pub emotional_context: String,  // "frustrated" / "excited" / "neutral"
    pub urgency_reason: String,     // "event in 3 days" / "no rush"
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceNeed {
    InternalKnowledge(String),  // reshimo/expression lookup for "X"
    Calculation(String),        // calc.rs for "500*12"
    Gematria(String),           // gematria.rs for "שלום"
    ExternalLLM(String),        // need Gemini/Claude for complex answer
    ProductCatalog(String),     // CHOOZ catalog for products
    FactLookup(String),         // fact_store for specific facts
    Comparison(String, String), // compare two concepts
    WebSearch(String),          // need current info
    NoResource,                 // just need empathy/acknowledgment
}

#[derive(Debug, Clone)]
pub struct OutputSpec {
    pub format: OutputFormat,
    pub max_length: usize,
    pub include_sources: bool,
    pub include_next_steps: bool,
    pub include_alternatives: bool,
    pub tone: String,
    pub language: String, // "he" / "en" — match query language
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    DirectAnswer,     // "42" — short, factual
    Explanation,      // "X הוא... כי..." — educational
    Comparison,       // "A vs B: ..." — balanced
    Recommendation,   // "אני ממליץ על X כי..." — actionable
    Empathy,          // "אני מבין שזה קשה..." — supportive
    Confirmation,     // "כן, נכון ש..." — validating
    StepByStep,       // "1. ... 2. ... 3. ..." — procedural
    Calculation,      // "17% × 4350 = 739.50" — precise
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub kind: ConstraintKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    DontGuess,         // if not sure, say "don't know"
    DontOverload,      // keep it short (urgency high)
    DontBeSalesly,     // user is venting, not buying
    StayInDomain,      // don't mix CHOOZ products into science answers
    FactsOnly,         // validation question — only verifiable facts
    Sensitive,         // health/finance/legal — add disclaimer
    CharterLimit,      // ethics check needed
}

#[derive(Debug, Clone)]
pub struct SuccessCriteria {
    pub min_confidence: f32,
    pub must_answer_core_question: bool,
    pub must_match_format: bool,
    pub must_be_actionable: bool,  // for Act goals
    pub acceptable_fallthrough: bool, // is "I don't know" OK?
}

#[derive(Debug, Clone)]
pub struct PlanStep {
    pub order: u32,
    pub action: String,
    pub resource: ResourceNeed,
    pub fallback: Option<String>,
}

// ============================================================
// THE INTERPRETER — builds Interpretation from query + context
// ============================================================

/// Main entry point: interpret the query before pipeline processes it
pub fn interpret(
    original_query: &str,
    cleaned_query: &str,
    kavana: &KavanaVector,
    intent: &str,          // from understand.rs
    domain: &str,          // from domain detection
    session_turn_count: u32,
    is_followup: bool,
    user_sentiment: f64,
) -> Interpretation {
    // ── 1. WHAT — paraphrase the core question ──
    let what = build_what(cleaned_query, intent, domain);

    // ── 2. WHY — enrich Kavana into full WhyFrame ──
    let why = build_why(kavana, original_query, user_sentiment);

    // ── 3. NEEDS — what resources do we need? ──
    let needs = identify_needs(cleaned_query, &kavana.goal, domain, intent);

    // ── 4. OUTPUT SPEC — what should the answer look like? ──
    let output_spec = build_output_spec(kavana, domain, &needs);

    // ── 5. CONSTRAINTS — what to avoid ──
    let constraints = identify_constraints(kavana, domain, &needs);

    // ── 6. SUCCESS CRITERIA — when is the answer good? ──
    let success_criteria = build_success_criteria(kavana, &needs);

    // ── 7. PLAN — ordered steps ──
    let plan = build_plan(&needs, kavana);

    // ── META: Do we need to ask the user? ──
    let (needs_clarification, clarification_question) =
        check_if_needs_clarification(kavana, &needs, session_turn_count);

    // ── META: How confident are we in this interpretation? ──
    let interpretation_confidence = compute_interpretation_confidence(
        kavana, &needs, is_followup, session_turn_count,
    );

    Interpretation {
        what,
        why,
        needs,
        output_spec,
        constraints,
        success_criteria,
        plan,
        interpretation_confidence,
        needs_clarification,
        clarification_question,
    }
}

/// Format interpretation as a human-readable trace (for debugging/stages)
pub fn format_trace(interp: &Interpretation) -> String {
    let needs_str: Vec<&str> = interp.needs.iter().map(|n| match n {
        ResourceNeed::InternalKnowledge(_) => "knowledge",
        ResourceNeed::Calculation(_) => "calc",
        ResourceNeed::Gematria(_) => "gematria",
        ResourceNeed::ExternalLLM(_) => "LLM",
        ResourceNeed::ProductCatalog(_) => "catalog",
        ResourceNeed::FactLookup(_) => "facts",
        ResourceNeed::Comparison(_, _) => "compare",
        ResourceNeed::WebSearch(_) => "web",
        ResourceNeed::NoResource => "none",
    }).collect();

    // CRITICAL: use chars() not byte slice for Hebrew text!
    let what_short: String = interp.what.chars().take(40).collect();
    let why_short: String = interp.why.deeper_why.chars().take(25).collect();

    format!(
        "what=[{}] why={:?}/{} needs=[{}] format={:?} conf={:.2}{}",
        what_short,
        interp.why.goal,
        why_short,
        needs_str.join("+"),
        interp.output_spec.format,
        interp.interpretation_confidence,
        if interp.needs_clarification { " CLARIFY?" } else { "" },
    )
}

// ============================================================
// BUILDERS — each aspect of the interpretation
// ============================================================

fn build_what(cleaned: &str, intent: &str, domain: &str) -> String {
    // Paraphrase the query in Cortex's "internal language"
    let intent_he = match intent {
        "Define" | "Explain" => "רוצה הגדרה/הסבר של",
        "Compare" => "רוצה השוואה בין",
        "HowTo" => "רוצה לדעת איך",
        "WhyBecause" => "רוצה לדעת למה",
        "Price" | "Calculate" => "רוצה חישוב/מחיר של",
        "Recommend" => "רוצה המלצה על",
        "Greeting" => "פנייה חברתית",
        _ => "שואל על",
    };
    format!("{} \"{}\" (תחום: {})", intent_he, cleaned, domain)
}

fn build_why(kavana: &KavanaVector, _original: &str, user_sentiment: f64) -> WhyFrame {
    let deeper_why = match kavana.goal {
        Goal::Learn => "רוצה להבין ולהרחיב ידע".to_string(),
        Goal::Decide => "עומד בפני החלטה וצריך מידע להשוואה".to_string(),
        Goal::Calculate => "צריך מספר מדויק, לא הערכה".to_string(),
        Goal::Create => "צריך תוכן חדש שלא קיים".to_string(),
        Goal::Fix => "יש בעיה שצריך לפתור".to_string(),
        Goal::Compare => "רוצה להבין הבדלים".to_string(),
        Goal::Explore => "סקרן, בלי מטרה ספציפית".to_string(),
        Goal::Vent => "צריך הקשבה, לא מידע".to_string(),
        Goal::Validate => "רוצה אישור למשהו שהוא חושב שנכון".to_string(),
        Goal::Act => "רוצה שיבוצע משהו עכשיו".to_string(),
    };

    let what_happens_after = match kavana.goal {
        Goal::Act => kavana.action_after.clone().unwrap_or_else(|| "יבצע פעולה".to_string()),
        Goal::Decide => "יחליט".to_string(),
        Goal::Calculate => "ישתמש במספר".to_string(),
        Goal::Create => "ישתמש בתוכן".to_string(),
        Goal::Learn | Goal::Explore => "ילמד".to_string(),
        Goal::Vent => "ירגיש טוב יותר".to_string(),
        Goal::Validate => "ימשיך בביטחון".to_string(),
        _ => "ישתמש בתשובה".to_string(),
    };

    let emotional_context = if user_sentiment < -0.3 {
        "מתוסכל/עצוב".to_string()
    } else if user_sentiment > 0.3 {
        "חיובי/נלהב".to_string()
    } else if kavana.emotional_load > 0.5 {
        "רגשי".to_string()
    } else {
        "ניטרלי".to_string()
    };

    let urgency_reason = if kavana.urgency > 0.8 {
        "דחוף — צריך עכשיו".to_string()
    } else if kavana.urgency > 0.5 {
        "חשוב אבל לא דחוף".to_string()
    } else {
        "אין לחץ זמן".to_string()
    };

    WhyFrame {
        goal: kavana.goal.clone(),
        deeper_why,
        what_happens_after,
        emotional_context,
        urgency_reason,
    }
}

fn identify_needs(
    cleaned: &str,
    goal: &Goal,
    domain: &str,
    intent: &str,
) -> Vec<ResourceNeed> {
    let mut needs = Vec::new();

    // Calculator
    if matches!(goal, Goal::Calculate) || intent == "Calculate" || intent == "Price" {
        needs.push(ResourceNeed::Calculation(cleaned.to_string()));
    }

    // Gematria
    if cleaned.contains("גימטריה") || cleaned.contains("gematria") {
        let word = cleaned.replace("גימטריה של ", "").replace("הגימטריה של ", "").trim().to_string();
        needs.push(ResourceNeed::Gematria(word));
    }

    // Product catalog
    if domain == "business" || domain == "product" {
        needs.push(ResourceNeed::ProductCatalog(cleaned.to_string()));
    }

    // Comparison
    if matches!(goal, Goal::Compare) || intent == "Compare" {
        // Try to extract the two things being compared
        let parts: Vec<&str> = cleaned.split(|c| c == 'ל' || c == ' ').collect();
        if parts.len() >= 2 {
            needs.push(ResourceNeed::Comparison(
                parts[0].to_string(),
                parts.last().unwrap_or(&"").to_string(),
            ));
        }
    }

    // Empathy — no resource needed
    if matches!(goal, Goal::Vent) {
        needs.push(ResourceNeed::NoResource);
        return needs; // don't add knowledge lookup for vent
    }

    // Default: internal knowledge
    if needs.is_empty() || matches!(goal, Goal::Learn | Goal::Explore | Goal::Validate) {
        needs.push(ResourceNeed::InternalKnowledge(cleaned.to_string()));
    }

    // Fact lookup for specific questions
    if intent == "FactShortcut" || cleaned.contains("כמה") || cleaned.contains("מה הגודל") {
        needs.push(ResourceNeed::FactLookup(cleaned.to_string()));
    }

    needs
}

fn build_output_spec(kavana: &KavanaVector, _domain: &str, _needs: &[ResourceNeed]) -> OutputSpec {
    let format = match kavana.goal {
        Goal::Calculate => OutputFormat::Calculation,
        Goal::Compare | Goal::Decide => OutputFormat::Comparison,
        Goal::Vent => OutputFormat::Empathy,
        Goal::Validate => OutputFormat::Confirmation,
        Goal::Act => OutputFormat::StepByStep,
        Goal::Fix => OutputFormat::StepByStep,
        Goal::Learn | Goal::Explore => OutputFormat::Explanation,
        Goal::Create => OutputFormat::DirectAnswer,
    };

    let max_length = if kavana.urgency > 0.7 {
        80  // urgent = short
    } else if kavana.depth > 0.7 {
        400 // deep = long
    } else {
        200 // default
    };

    let tone = match kavana.goal {
        Goal::Vent => "warm",
        Goal::Calculate => "precise",
        Goal::Act => "direct",
        Goal::Validate => "supportive",
        Goal::Fix => "helpful",
        _ => "balanced",
    };

    OutputSpec {
        format,
        max_length,
        include_sources: matches!(kavana.goal, Goal::Learn | Goal::Validate),
        include_next_steps: matches!(kavana.goal, Goal::Act | Goal::Decide | Goal::Fix),
        include_alternatives: matches!(kavana.goal, Goal::Decide | Goal::Compare),
        tone: tone.to_string(),
        language: "he".to_string(), // will be set by pipeline
    }
}

fn identify_constraints(
    kavana: &KavanaVector,
    domain: &str,
    _needs: &[ResourceNeed],
) -> Vec<Constraint> {
    let mut constraints = Vec::new();

    // Always: don't guess if not sure
    constraints.push(Constraint {
        kind: ConstraintKind::DontGuess,
        description: "אם לא בטוח — אמור 'לא יודע'".to_string(),
    });

    // Vent: don't try to sell
    if matches!(kavana.goal, Goal::Vent) {
        constraints.push(Constraint {
            kind: ConstraintKind::DontBeSalesly,
            description: "המשתמש מפרוק רגשית — לא למכור לו".to_string(),
        });
    }

    // Urgent: keep short
    if kavana.urgency > 0.7 {
        constraints.push(Constraint {
            kind: ConstraintKind::DontOverload,
            description: "דחוף — תשובה קצרה וישירה".to_string(),
        });
    }

    // Non-business domain: don't inject products
    if domain != "business" && domain != "product" {
        constraints.push(Constraint {
            kind: ConstraintKind::StayInDomain,
            description: "לא למכור מוצרים בתשובה על מדע/כללי".to_string(),
        });
    }

    // Validation: facts only
    if matches!(kavana.goal, Goal::Validate) {
        constraints.push(Constraint {
            kind: ConstraintKind::FactsOnly,
            description: "שאלת אימות — רק עובדות מוכחות".to_string(),
        });
    }

    // Sensitive domains
    if domain == "health" || domain == "finance" || domain == "law" {
        constraints.push(Constraint {
            kind: ConstraintKind::Sensitive,
            description: "תחום רגיש — הוסף הבהרה".to_string(),
        });
    }

    constraints
}

fn build_success_criteria(kavana: &KavanaVector, _needs: &[ResourceNeed]) -> SuccessCriteria {
    SuccessCriteria {
        min_confidence: match kavana.goal {
            Goal::Calculate => 0.99, // math must be exact
            Goal::Act => 0.80,       // action needs certainty
            Goal::Validate => 0.85,  // validation needs facts
            Goal::Vent => 0.0,       // empathy doesn't need confidence
            _ => 0.70,               // default threshold
        },
        must_answer_core_question: !matches!(kavana.goal, Goal::Vent | Goal::Explore),
        must_match_format: true,
        must_be_actionable: matches!(kavana.goal, Goal::Act | Goal::Fix),
        acceptable_fallthrough: matches!(kavana.goal, Goal::Explore),
    }
}

fn build_plan(needs: &[ResourceNeed], _kavana: &KavanaVector) -> Vec<PlanStep> {
    let mut plan = Vec::new();
    let mut order = 1;

    for need in needs {
        let (action, fallback) = match need {
            ResourceNeed::InternalKnowledge(q) => (
                format!("חפש '{}' ב-reshimo/expressions", q),
                Some("נסה Gemini אם לא נמצא".to_string()),
            ),
            ResourceNeed::Calculation(q) => (
                format!("חשב: {}", q),
                None, // calc doesn't fail
            ),
            ResourceNeed::Gematria(w) => (
                format!("חשב גימטריה: {}", w),
                None,
            ),
            ResourceNeed::ExternalLLM(q) => (
                format!("שאל LLM: {}", q),
                Some("נסה LLM אחר".to_string()),
            ),
            ResourceNeed::ProductCatalog(q) => (
                format!("חפש בקטלוג CHOOZ: {}", q),
                Some("חפש בידע כללי".to_string()),
            ),
            ResourceNeed::FactLookup(q) => (
                format!("חפש עובדה: {}", q),
                Some("בנה תשובה מ-facts".to_string()),
            ),
            ResourceNeed::Comparison(a, b) => (
                format!("השווה: {} vs {}", a, b),
                None,
            ),
            ResourceNeed::WebSearch(q) => (
                format!("חפש ברשת: {}", q),
                Some("ענה מידע שיש".to_string()),
            ),
            ResourceNeed::NoResource => (
                "הקשב ותמוך".to_string(),
                None,
            ),
        };

        plan.push(PlanStep {
            order,
            action,
            resource: need.clone(),
            fallback,
        });
        order += 1;
    }

    plan
}

fn check_if_needs_clarification(
    kavana: &KavanaVector,
    _needs: &[ResourceNeed],
    session_turns: u32,
) -> (bool, Option<String>) {
    // Don't ask on first message — just try to answer
    if session_turns == 0 {
        return (false, None);
    }

    // Low Kavana confidence + unknown source → ask
    if kavana.confidence < 0.40 && kavana.source == KavanaSource::Unknown {
        return (true, Some(
            "אני רוצה לעזור בצורה הכי טובה — אתה מחפש מידע, רוצה להשוות אפשרויות, או צריך שאבצע משהו?".to_string()
        ));
    }

    // Act without specifics → ask what exactly
    if matches!(kavana.goal, Goal::Act) && kavana.action_after.is_none() {
        return (true, Some(
            "מה בדיוק תרצה שאבצע?".to_string()
        ));
    }

    (false, None)
}

fn compute_interpretation_confidence(
    kavana: &KavanaVector,
    needs: &[ResourceNeed],
    is_followup: bool,
    session_turns: u32,
) -> f32 {
    let mut conf = kavana.confidence;

    // Followup = higher confidence (context helps)
    if is_followup && session_turns > 1 {
        conf = (conf + 0.1).min(1.0);
    }

    // Explicit Kavana = higher confidence
    if kavana.source == KavanaSource::Explicit {
        conf = (conf + 0.1).min(1.0);
    }

    // Multiple resource needs = might be wrong
    if needs.len() > 3 {
        conf *= 0.9;
    }

    conf
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kavana(goal: Goal, conf: f32, urgency: f32) -> KavanaVector {
        KavanaVector {
            goal,
            confidence: conf,
            urgency,
            depth: 0.5,
            action_after: None,
            emotional_load: 0.0,
            source: KavanaSource::PatternMatch,
        }
    }

    #[test]
    fn test_learn_interpretation() {
        let k = make_kavana(Goal::Learn, 0.8, 0.3);
        let i = interpret("מה זה DNA", "DNA", &k, "Define", "general", 0, false, 0.0);
        assert!(i.what.contains("DNA"));
        assert_eq!(i.why.goal, Goal::Learn);
        assert!(i.needs.iter().any(|n| matches!(n, ResourceNeed::InternalKnowledge(_))));
        assert_eq!(i.output_spec.format, OutputFormat::Explanation);
        assert!(!i.needs_clarification);
    }

    #[test]
    fn test_calculate_interpretation() {
        let k = make_kavana(Goal::Calculate, 0.9, 0.5);
        let i = interpret("כמה זה 500 כפול 3", "500 כפול 3", &k, "Calculate", "math", 0, false, 0.0);
        assert!(i.needs.iter().any(|n| matches!(n, ResourceNeed::Calculation(_))));
        assert_eq!(i.output_spec.format, OutputFormat::Calculation);
        assert_eq!(i.success_criteria.min_confidence, 0.99);
    }

    #[test]
    fn test_vent_interpretation() {
        let k = KavanaVector {
            goal: Goal::Vent,
            confidence: 0.9,
            urgency: 0.2,
            depth: 0.3,
            action_after: None,
            emotional_load: 0.8,
            source: KavanaSource::Explicit,
        };
        let i = interpret("נמאס לי מהכל", "נמאס לי מהכל", &k, "Vent", "general", 2, false, -0.5);
        assert_eq!(i.output_spec.format, OutputFormat::Empathy);
        assert!(i.constraints.iter().any(|c| c.kind == ConstraintKind::DontBeSalesly));
        assert_eq!(i.success_criteria.min_confidence, 0.0);
        assert!(i.needs.iter().any(|n| matches!(n, ResourceNeed::NoResource)));
        assert!(i.why.emotional_context.contains("מתוסכל"));
    }

    #[test]
    fn test_act_needs_clarification() {
        let k = KavanaVector {
            goal: Goal::Act,
            confidence: 0.9,
            urgency: 0.8,
            depth: 0.3,
            action_after: None, // no specific action
            emotional_load: 0.0,
            source: KavanaSource::PatternMatch,
        };
        let i = interpret("תעשה", "תעשה", &k, "Act", "general", 3, false, 0.0);
        assert!(i.needs_clarification);
        assert!(i.clarification_question.is_some());
    }

    #[test]
    fn test_business_needs_catalog() {
        let k = make_kavana(Goal::Decide, 0.7, 0.5);
        let i = interpret("מה זה ספל ממותג", "ספל ממותג", &k, "Define", "business", 0, false, 0.0);
        assert!(i.needs.iter().any(|n| matches!(n, ResourceNeed::ProductCatalog(_))));
    }

    #[test]
    fn test_sensitive_domain_constraint() {
        let k = make_kavana(Goal::Learn, 0.8, 0.3);
        let i = interpret("מה זה סוכרת", "סוכרת", &k, "Define", "health", 0, false, 0.0);
        assert!(i.constraints.iter().any(|c| c.kind == ConstraintKind::Sensitive));
    }

    #[test]
    fn test_gematria_need() {
        let k = make_kavana(Goal::Learn, 0.8, 0.3);
        let i = interpret("מה הגימטריה של שלום", "גימטריה של שלום", &k, "Define", "torah", 0, false, 0.0);
        assert!(i.needs.iter().any(|n| matches!(n, ResourceNeed::Gematria(_))));
    }

    #[test]
    fn test_validate_facts_only() {
        let k = make_kavana(Goal::Validate, 0.85, 0.3);
        let i = interpret("נכון שהשמש גדולה מהירח", "השמש גדולה מהירח", &k, "Validate", "general", 0, false, 0.0);
        assert_eq!(i.output_spec.format, OutputFormat::Confirmation);
        assert!(i.constraints.iter().any(|c| c.kind == ConstraintKind::FactsOnly));
    }

    #[test]
    fn test_followup_boosts_confidence() {
        let k = make_kavana(Goal::Learn, 0.7, 0.3);
        let i1 = interpret("מה זה X", "X", &k, "Define", "general", 0, false, 0.0);
        let i2 = interpret("ומה עם Y", "Y", &k, "Define", "general", 3, true, 0.0);
        assert!(i2.interpretation_confidence > i1.interpretation_confidence);
    }

    #[test]
    fn test_format_trace() {
        let k = make_kavana(Goal::Learn, 0.8, 0.3);
        let i = interpret("מה זה DNA", "DNA", &k, "Define", "general", 0, false, 0.0);
        let trace = format_trace(&i);
        assert!(trace.contains("DNA"));
        assert!(trace.contains("Learn"));
        assert!(trace.contains("knowledge"));
    }

    #[test]
    fn test_stay_in_domain_constraint() {
        let k = make_kavana(Goal::Learn, 0.8, 0.3);
        let i = interpret("מה זה DNA", "DNA", &k, "Define", "general", 0, false, 0.0);
        assert!(i.constraints.iter().any(|c| c.kind == ConstraintKind::StayInDomain));
    }

    #[test]
    fn test_urgent_keeps_short() {
        let k = make_kavana(Goal::Act, 0.9, 0.9);
        let i = interpret("תזמין עכשיו", "תזמין", &k, "Act", "business", 0, false, 0.0);
        assert!(i.output_spec.max_length <= 80);
        assert!(i.constraints.iter().any(|c| c.kind == ConstraintKind::DontOverload));
    }
}
