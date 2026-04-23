//! # Human Logic — Bridge between HumanContext and Pipeline
//!
//! This module contains the LOGIC that populates HumanContext from pipeline data.
//! human.rs = pure data structures (no imports).
//! human_logic.rs = bridge functions (imports from both human.rs and pipeline modules).
//!
//! Called at 3 points in pipeline.rs:
//!   1. before_pipeline()  — creates and populates HumanContext
//!   2. mid_pipeline()     — updates context from intermediate results
//!   3. after_pipeline()   — post-response awareness, regret, anticipation

use crate::human::*;
use crate::sentiment::Sentiment;
use crate::session::ConversationContext;
use crate::pipeline::UserContext;

// ═══════════════════════════════════════════════════════════════
//  I. BEFORE PIPELINE — Create HumanContext from available data
// ═══════════════════════════════════════════════════════════════

/// Build the initial HumanContext before the pipeline starts.
/// This is the ENTRY POINT — called once at the top of query_full().
///
/// Takes: raw query text, sentiment analysis, session context, user context.
/// Returns: fully populated HumanContext ready for pipeline use.
///
/// Cost: ~0.1ms total. No LLM calls. No I/O.
pub fn before_pipeline(
    text: &str,
    sent: &Sentiment,
    session_ctx: &ConversationContext,
    _user_ctx: &UserContext,
    domain: &str,
    somatic: &SomaticMemory,
) -> HumanContext {
    let mut ctx = HumanContext::new();

    // === PRESENCE ===

    // P2: Emotional Permeability — absorb input emotion
    let trust = session_ctx.avg_confidence as f32 * 0.5 + 0.25; // proxy for trust
    ctx.presence.internal.permeate(
        sent.valence as f32,
        sent.arousal as f32,
        trust,
    );

    // P4: Connection Quality — from session history
    ctx.presence.connection.trust = trust;
    ctx.presence.connection.track_record = session_ctx.avg_confidence as f32;

    // === SITUATION ===

    // T6: Damage Assessment — domain + content keywords
    ctx.situation.damage = DamageAssessment::assess(domain, text);

    // T2: Conversation Arc — from session
    ctx.situation.arc.turn_count = session_ctx.turn_count as u32;
    // Detect same-topic by comparing current domain to previous
    let is_same_topic = !session_ctx.prev_domain.is_empty()
        && session_ctx.prev_domain == domain;
    ctx.situation.arc.update(domain, session_ctx.avg_confidence as f32, is_same_topic);

    // T1: Frame — build from all available context
    let somatic_bias = somatic.domain_bias(domain);
    ctx.situation.frame = Frame::build(
        &ctx.other.speaker,
        &ctx.presence.internal,
        ctx.situation.arc.momentum,
        somatic_bias,
    );

    // === OTHER ===

    // O1: Speaker Model — update from query
    ctx.other.speaker.update_from_query(
        text.chars().count(), // char count, not byte count!
        domain,
        sent.valence as f32,
    );

    // === CHOICE ===
    ctx.choice.somatic_bias = somatic_bias;
    ctx.choice.total_damage = ctx.situation.damage.effective_damage;

    // === P1: DWELL CHECK ===
    ctx.presence.dwell = check_dwell(text, sent, &ctx);

    // === P3: SILENCE CHECK ===
    ctx.presence.silence_mode = check_silence(text, sent, &ctx);

    // === O2: INFERRED INTENT ===
    ctx.other.inferred_intent = infer_deep_intent(text, sent, session_ctx);

    // === O3: THEORY OF MIND Level 2 ===
    let tom_impl = estimate_tom_level2(
        ctx.other.inferred_intent,
        session_ctx,
        &session_ctx.recent_confidences,
    );
    if !matches!(tom_impl, TomImplication::None) {
        ctx.other.tom.push(TomLevel {
            depth: 2,
            belief: format!("{:?}", tom_impl),
            confidence: 0.5,
            implication: tom_impl,
        });
    }

    // === O4: MIRRORING ===
    ctx.other.mirror = compute_mirror(text, &ctx.other.speaker);

    // === T4: MEANING LAYER — detect theme behind the question ===
    ctx.situation.meaning = detect_meaning(text, &ctx.situation.damage);

    // === T5: PRECEDENT — track patterns ===
    update_precedent(&mut ctx.situation.precedent, text, domain, session_ctx);

    // === T3: UNSPOKEN DETECTION ===
    ctx.situation.unspoken = detect_unspoken(text, domain, &ctx.situation.arc, &ctx.situation.damage);

    // === T-humor: HUMOR DETECTION ===
    ctx.situation.humor_detected = detect_humor(text, ctx.situation.arc.momentum);

    // === G5: BOREDOM ===
    ctx.growth.boredom_score = compute_boredom(&ctx.other.speaker.domains_asked);

    // === S6: CERTAINTY (will be updated after pipeline with actual confidence) ===
    ctx.self_state.certainty = CertaintySpectrum::Unknown;

    ctx
}

// ═══════════════════════════════════════════════════════════════
//  II. DWELL — should we pause before processing?
// ═══════════════════════════════════════════════════════════════

fn check_dwell(
    text: &str,
    sent: &Sentiment,
    ctx: &HumanContext,
) -> Option<DwellResult> {
    let damage = ctx.situation.damage.effective_damage;
    let emotion_intensity = sent.arousal as f32;
    let escalating = ctx.situation.arc.momentum == Momentum::Escalating;
    let uncertain_zone = ctx.presence.internal.confidence_mood < 0.5;

    // Determine trigger
    let _trigger = if damage > 0.7 {
        DwellTrigger::HighDamage
    } else if emotion_intensity > 0.7 {
        DwellTrigger::HighEmotion
    } else if escalating {
        DwellTrigger::Escalation
    } else if uncertain_zone && damage > 0.4 {
        DwellTrigger::UncertainZone
    } else {
        return None; // No dwell needed — simple query
    };

    // Assess urgency — short queries with urgency markers
    let urgency = if text.chars().count() < 15 && text.contains('!') {
        0.9
    } else if emotion_intensity > 0.8 {
        0.8
    } else {
        0.4
    };

    // Assess curiosity signal — is this an interesting gap?
    let curiosity = if damage > 0.5 && emotion_intensity < 0.3 {
        0.8 // calm question about dangerous topic = interesting
    } else {
        0.3
    };

    // Gap type
    let gap_type = if escalating && ctx.situation.arc.same_topic_count >= 3 {
        GapType::DeepUncertainty // asked same thing 3 times = something deeper going on
    } else if damage > 0.7 {
        GapType::Ambiguity // high-damage queries need extra care with interpretation
    } else {
        GapType::MissingData
    };

    // Recommendation
    let recommendation = if urgency > 0.8 {
        DwellRecommendation::ProceedWithCaveat(
            "שאלה רגישה — מומלץ לבדוק עם מומחה".to_string()
        )
    } else if escalating {
        DwellRecommendation::AskBack(
            "שמתי לב שזו שאלה חוזרת. אולי ננסה לגשת לזה אחרת?".to_string()
        )
    } else if curiosity > 0.6 {
        DwellRecommendation::Explore
    } else {
        DwellRecommendation::Acknowledge
    };

    Some(DwellResult {
        triggered: true,
        gap_type,
        curiosity_signal: curiosity,
        urgency,
        recommendation,
    })
}

// ═══════════════════════════════════════════════════════════════
//  III. SILENCE — should we just listen, not answer?
// ═══════════════════════════════════════════════════════════════

fn check_silence(
    text: &str,
    sent: &Sentiment,
    ctx: &HumanContext,
) -> bool {
    let emotion_intensity = sent.arousal;
    let has_question = text.contains('?')
        || text.contains("מה ")
        || text.contains("איך ")
        || text.contains("למה ")
        || text.contains("כמה ")
        || text.contains("האם ")
        || text.contains("what ")
        || text.contains("how ")
        || text.contains("why ");

    // Venting: high emotion + no direct question
    let is_venting = emotion_intensity > 0.6 && !has_question;

    // Escalation: same topic 3+ times with increasing emotion
    let is_escalating = ctx.situation.arc.momentum == Momentum::Escalating
        && ctx.situation.arc.same_topic_count >= 3;

    // Crisis: very high emotion + high damage domain
    let is_crisis = emotion_intensity > 0.9
        && ctx.situation.damage.effective_damage > 0.7;

    is_venting || (is_escalating && !has_question) || is_crisis
}

// ═══════════════════════════════════════════════════════════════
//  IV. SILENCE RESPONSE — what to say when just listening
// ═══════════════════════════════════════════════════════════════

/// Generate an empathy-only response when silence_mode is active.
/// Returns Hebrew text. No pipeline processing needed.
pub fn silence_response(ctx: &HumanContext) -> String {
    if ctx.situation.damage.effective_damage > 0.7 {
        // High-damage domain (medical, safety)
        "אני שומע אותך. אם אתה צריך עזרה מקצועית, מומלץ לפנות לגורם מוסמך.".to_string()
    } else if ctx.situation.arc.momentum == Momentum::Escalating {
        // Escalation — de-escalate
        "אני מבין שזה מתסכל. בוא ננסה לגשת לזה מזווית אחרת.".to_string()
    } else {
        // General venting
        "אני שומע אותך.".to_string()
    }
}

// ═══════════════════════════════════════════════════════════════
//  V. AFTER PIPELINE — post-response awareness
// ═══════════════════════════════════════════════════════════════

/// Update HumanContext after pipeline produces a result.
/// This is where certainty spectrum, awareness, and regret happen.
///
/// Called once after pipeline, before sending response to user.
pub fn after_pipeline(
    ctx: &mut HumanContext,
    confidence: f64,
    method: &str,
    domain: &str,
    _answer_len: usize,
) {
    // S6: Compute CertaintySpectrum from actual confidence + damage
    ctx.self_state.certainty = CertaintySpectrum::from_confidence(
        confidence,
        ctx.situation.damage.effective_damage,
    );

    // S5: Three Layers of Awareness — observe what we did
    // Layer 2 (observation): was I uncertain? was I too long?
    let was_uncertain = confidence < 0.75;
    let _was_confident = confidence > 0.90;

    // Layer 3 (narrative): update domain confidence in narrative
    let domain_conf = ctx.self_state.narrative.domain_conf(domain);
    let new_conf = if confidence > 0.8 {
        (domain_conf + 0.02).min(1.0) // success → slightly boost
    } else if confidence < 0.5 {
        (domain_conf - 0.03).max(0.0) // failure → reduce
    } else {
        domain_conf
    };
    ctx.self_state.narrative.domain_confidence
        .insert(domain.to_string(), new_conf);

    // S3: Trajectory — if uncertain in a domain, boost learning goal
    if was_uncertain && ctx.situation.damage.effective_damage > 0.5 {
        let goal = format!("improve_{}", domain);
        ctx.self_state.trajectory.boost(&goal);
    }

    // Note: Regret engine runs later, when reward (👍/👎) arrives.
    // We prepare the observation here but evaluate on reward.

    // G2: Anticipation — predict next query (simple heuristic)
    ctx.growth.anticipation.clear();
    if method == "fact_shortcut" || method == "exact" {
        // If we answered with a fact, they might ask a follow-up
        ctx.growth.anticipation.push(format!("followup_about_{}", domain));
    }
}

// ═══════════════════════════════════════════════════════════════
//  VI. ON REWARD — when 👍/👎 arrives
// ═══════════════════════════════════════════════════════════════

/// Process a reward event. Updates somatic memory, connection, values.
/// Called from the /reward endpoint handler.
pub fn on_reward(
    somatic: &mut SomaticMemory,
    connection: &mut ConnectionState,
    values: &mut Values,
    domain: &str,
    method: &str,
    reward: f32,
    timestamp: u64,
) {
    // Somatic: record this experience for future bias
    somatic.record(domain, method, reward, timestamp);

    // Connection: update quality (asymmetric — easy to break, hard to build)
    connection.on_reward(reward);

    // Values: reinforce based on what happened
    if reward > 0.5 {
        values.reinforce("accuracy", domain, 0.5);
        values.reinforce("helpfulness", domain, 0.3);
    } else if reward < -0.5 {
        values.reinforce("caution", domain, 0.5);
    }
}

// ═══════════════════════════════════════════════════════════════
//  VII. EFFECTIVE CONFIDENCE — adjusted by HumanContext
// ═══════════════════════════════════════════════════════════════

/// Should the pipeline proceed to output, or fallthrough?
/// Replaces the simple `confidence >= 0.70` check with context-aware threshold.
pub fn should_output(ctx: &HumanContext, confidence: f64) -> bool {
    let threshold = ctx.effective_threshold();
    confidence >= threshold
}

/// Get a tone recommendation for generation.rs.
pub fn recommended_tone(ctx: &HumanContext) -> &'static str {
    ctx.recommended_tone()
}

/// Should the response include a disclaimer?
pub fn should_add_disclaimer(ctx: &HumanContext) -> bool {
    ctx.needs_disclaimer()
}

/// Get the Hebrew disclaimer text if needed.
pub fn get_disclaimer_he(ctx: &HumanContext) -> Option<&'static str> {
    ctx.disclaimer_he()
}

/// Get the certainty caveat in Hebrew if needed.
pub fn get_certainty_caveat_he(ctx: &HumanContext) -> Option<&str> {
    ctx.self_state.certainty.caveat_he()
}

// ═══════════════════════════════════════════════════════════════
//  VIII. DISAMBIGUATION — multi-parse for Hebrew ambiguity
// ═══════════════════════════════════════════════════════════════

/// Simple Levenshtein edit distance on chars (not bytes!).
/// Hebrew-safe: works on char iterators, never touches byte indices.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut matrix = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { matrix[i][0] = i; }
    for j in 0..=n { matrix[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    matrix[m][n]
}

/// Check if a word is likely a typo of a known word.
/// Returns the correction and edit distance, or None if no close match.
/// Conservative: only suggests corrections with edit distance = 1 and
/// the correction must be in the provided vocabulary.
pub fn suggest_typo_correction<'a>(
    word: &str,
    vocab: impl Iterator<Item = &'a str>,
    max_distance: usize,
) -> Option<(String, usize)> {
    let word_chars: Vec<char> = word.chars().collect();
    if word_chars.len() < 2 { return None; } // too short to correct

    let mut best: Option<(String, usize)> = None;

    for candidate in vocab {
        let candidate_chars: Vec<char> = candidate.chars().collect();
        // Quick length check — edit distance can't be less than length difference
        let len_diff = (word_chars.len() as isize - candidate_chars.len() as isize).unsigned_abs();
        if len_diff > max_distance { continue; }

        let dist = edit_distance(word, candidate);
        if dist > 0 && dist <= max_distance {
            if best.as_ref().map_or(true, |(_, d)| dist < *d) {
                best = Some((candidate.to_string(), dist));
            }
        }
    }
    best
}

// ═══════════════════════════════════════════════════════════════
//  VIII-b. INFERRED INTENT — what do they really want?
// ═══════════════════════════════════════════════════════════════

/// Infer deep intent from surface intent + conversation context.
/// Runs once per query, feeds into HumanContext.
pub fn infer_deep_intent(
    text: &str,
    sent: &Sentiment,
    session_ctx: &ConversationContext,
) -> DeepIntent {
    // Venting: high emotion + no question
    let has_question = text.contains('?') || text.contains("מה ")
        || text.contains("איך ") || text.contains("למה ")
        || text.contains("what ") || text.contains("how ");
    if sent.needs_empathy && !has_question {
        return DeepIntent::Venting;
    }

    // Urgent: urgency markers
    if sent.urgency > 0.6 {
        return DeepIntent::Urgent;
    }

    // Comparing: "vs", "or", "difference", "compare"
    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();
    if lower.contains("הבדל") || lower.contains("לעומת") || lower.contains("או ")
        || lower.contains("difference") || lower.contains(" vs ") || lower.contains("compare") {
        return DeepIntent::Comparing;
    }

    // Validating: repeating after getting an answer (same session, same topic)
    if session_ctx.turn_count > 1 && session_ctx.is_followup {
        return DeepIntent::Validating;
    }

    // Learning: "explain", "why", "how"
    if lower.contains("תסביר") || lower.contains("למה") || lower.contains("explain")
        || lower.contains("how does") || lower.contains("איך עובד") {
        return DeepIntent::Learning;
    }

    // Exploring: curiosity markers
    if lower.contains("מעניין") || lower.contains("interesting") || lower.contains("curious") {
        return DeepIntent::Exploring;
    }

    DeepIntent::SeekingFact
}

// ═══════════════════════════════════════════════════════════════
//  VIII-c. THEORY OF MIND — Level 2
// ═══════════════════════════════════════════════════════════════

/// Estimate what the speaker expects from us (ToM Level 2).
/// "What do they think I will answer?"
pub fn estimate_tom_level2(
    deep_intent: DeepIntent,
    _session_ctx: &ConversationContext,
    confidence_history: &[f64],
) -> TomImplication {
    match deep_intent {
        DeepIntent::Validating => TomImplication::SeekingValidation,
        DeepIntent::Venting => TomImplication::None, // not expecting an answer
        DeepIntent::TestingBoundaries => TomImplication::TestingMe,
        _ => {
            // If we've failed recently, they might expect failure
            if confidence_history.len() >= 2 {
                let recent_avg: f64 = confidence_history.iter().rev().take(3).sum::<f64>()
                    / confidence_history.len().min(3) as f64;
                if recent_avg < 0.5 {
                    return TomImplication::ExpectsFailure;
                }
            }
            TomImplication::GenuineCuriosity
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  VIII-d. MIRRORING — match output register to input
// ═══════════════════════════════════════════════════════════════

/// Compute output register profile from input text.
/// Used by generation to mirror the user's communication style.
pub fn compute_mirror(text: &str, speaker: &SpeakerModel) -> RegisterProfile {
    let char_count = text.chars().count();

    // Length: response should be proportional to query
    let min_len = (char_count as f32 * 0.3) as usize;
    let max_len = (char_count as f32 * 10.0) as usize; // response can be 10x query

    // Formality: detect from text markers
    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();
    let formal_markers = ["בבקשה", "אדוני", "גברת", "please", "sir", "dear", "kindly"];
    let casual_markers = ["אחי", "בר", "יא", "yo", "hey", "bro", "dude", "lol"];
    let formal_count = formal_markers.iter().filter(|m| lower.contains(*m)).count();
    let casual_count = casual_markers.iter().filter(|m| lower.contains(*m)).count();
    let formality: f32 = if formal_count > casual_count { 0.8 }
        else if casual_count > formal_count { 0.2 }
        else { 0.5 };

    // Complexity: word length proxy
    let words: Vec<&str> = text.split_whitespace().collect();
    let avg_word_len: f32 = if words.is_empty() { 4.0 }
        else { words.iter().map(|w| w.chars().count()).sum::<usize>() as f32 / words.len() as f32 };
    let complexity: f32 = (avg_word_len / 8.0).min(1.0); // 8+ char avg = complex

    // Vocabulary: technical markers
    let technical = ["API", "HTTP", "RAM", "CPU", "SQL", "algorithm", "function",
                     "אלגוריתם", "פונקציה", "מסד נתונים", "שרת"];
    let tech_count = technical.iter().filter(|t| text.contains(*t)).count();
    let vocab_level: f32 = if tech_count > 0 { 0.8 } else { complexity * 0.7 };

    // Override with speaker model preferences if available
    let final_formality = match speaker.preferred_tone {
        TonePref::Formal => formality.max(0.7),
        TonePref::Casual => formality.min(0.3),
        _ => formality,
    };

    RegisterProfile {
        target_length_min: min_len.max(50),
        target_length_max: max_len.max(200).min(3000),
        complexity,
        formality: final_formality,
        vocabulary_level: vocab_level,
    }
}

// ═══════════════════════════════════════════════════════════════
//  VIII-e. UNSPOKEN DETECTION — what's not being said
// ═══════════════════════════════════════════════════════════════

/// Detect what the user might not be saying explicitly.
/// Returns None if no signal, Some if pattern detected.
pub fn detect_unspoken(
    text: &str,
    domain: &str,
    arc: &ConversationArc,
    damage: &DamageAssessment,
) -> Option<UnspokenSignal> {
    // Pattern: repeated medical/safety questions = anxiety
    if damage.effective_damage > 0.7 && arc.same_topic_count >= 2 {
        return Some(UnspokenSignal {
            hypothesis: "שאלות חוזרות בנושא רגיש — ייתכן שמשהו מדאיג".into(),
            confidence: 0.5,
            response: UnspokenResponse::GentleInvitation(
                "אם יש משהו נוסף שמטריד אותך, אני כאן.".into()
            ),
        });
    }

    // Pattern: escalation with frustration = seeking human connection
    if arc.momentum == Momentum::Escalating && arc.same_topic_count >= 3 {
        return Some(UnspokenSignal {
            hypothesis: "שאלות חוזרות = אולי צריך גישה אחרת".into(),
            confidence: 0.4,
            response: UnspokenResponse::Acknowledgment(
                "נראה שלא הצלחתי לענות כמו שצריך. בוא ננסה אחרת.".into()
            ),
        });
    }

    // Pattern: medical keywords in non-medical domain
    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();
    let medical_worry = ["תסמינים", "תופעות לוואי", "מינון", "symptoms", "side effects", "dosage"];
    if domain != "medical" && medical_worry.iter().any(|kw| lower.contains(kw)) {
        return Some(UnspokenSignal {
            hypothesis: "שאלה רפואית — ייתכן שצריך מומחה".into(),
            confidence: 0.6,
            response: UnspokenResponse::ProfessionalReferral,
        });
    }

    None
}

// ═══════════════════════════════════════════════════════════════
//  VIII-f. HUMOR DETECTION — incongruity between literal and contextual
// ═══════════════════════════════════════════════════════════════

/// Simple humor detection based on incongruity markers.
/// Returns true if the input is likely humorous/sarcastic.
pub fn detect_humor(text: &str, momentum: Momentum) -> bool {
    // Don't detect humor in escalating/tense situations
    if momentum == Momentum::Escalating { return false; }

    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();

    // Hebrew humor/sarcasm markers
    let humor_markers = ["הא", "כאילו", "ברור שלא", "חח", "חחח", "😂", "😜",
                         "lol", "haha", "rofl", "jk", "just kidding",
                         "אתה בטוח?", "you sure?", "yeah right"];

    humor_markers.iter().any(|m| lower.contains(m))
}

// ═══════════════════════════════════════════════════════════════
//  VIII-g. REGRET — learn from process, not just outcome
// ═══════════════════════════════════════════════════════════════

/// Evaluate a negative reward and extract a process lesson.
/// Called when /reward receives a negative score.
pub fn evaluate_regret(
    method: &str,
    confidence: f64,
    domain: &str,
    was_too_long: bool,
) -> Option<ProcessLesson> {
    // Too confident on wrong answer
    if confidence > 0.8 {
        return Some(ProcessLesson {
            what_wrong: RegretType::TooConfident,
            alternative: format!("Should have hedged at conf {:.2} in domain {}", confidence, domain),
            impact: 0.7,
        });
    }

    // Wrong source (learned_live from Gemini often wrong)
    if method == "learned_live" {
        return Some(ProcessLesson {
            what_wrong: RegretType::WrongSource,
            alternative: "Gemini live answer — should verify or use existing knowledge".into(),
            impact: 0.5,
        });
    }

    // Too long
    if was_too_long {
        return Some(ProcessLesson {
            what_wrong: RegretType::WrongTone,
            alternative: "Response too long — user prefers short".into(),
            impact: 0.3,
        });
    }

    None
}

// ═══════════════════════════════════════════════════════════════
//  VIII-h. NARRATIVE UPDATE — called from NightMode
// ═══════════════════════════════════════════════════════════════

/// Generate narrative events from today's performance.
/// Called by NightMode after daily consolidation.
pub fn update_narrative_from_stats(
    narrative: &mut Narrative,
    domain_successes: &[(String, u32)],
    domain_failures: &[(String, u32)],
    _total_queries: u32,
    new_concepts_learned: u32,
) {
    // Learning milestone
    if new_concepts_learned >= 100 {
        narrative.note(
            &format!("למדתי {} מושגים חדשים היום", new_concepts_learned),
            0.6,
        );
    }

    // Domain mastery / weakness
    for (domain, successes) in domain_successes {
        if *successes >= 10 {
            let conf = narrative.domain_conf(domain);
            narrative.domain_confidence.insert(domain.clone(), (conf + 0.05).min(1.0));
            if conf < 0.7 && conf + 0.05 >= 0.7 {
                narrative.note(&format!("התחלתי להיות טוב ב-{}", domain), 0.7);
            }
        }
    }

    for (domain, failures) in domain_failures {
        if *failures >= 5 {
            let conf = narrative.domain_conf(domain);
            narrative.domain_confidence.insert(domain.clone(), (conf - 0.05).max(0.0));
            narrative.note(&format!("עדיין מתקשה ב-{}", domain), 0.5);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  VIII-i. GRACEFUL DEGRADATION — detect overload
// ═══════════════════════════════════════════════════════════════

/// Check if the system should enter degraded mode.
/// Returns true if overloaded (skip S2, answer from S0+S1 only).
pub fn check_degradation(
    avg_response_ms: f64,
    normal_response_ms: f64,
    error_rate: f32,
) -> bool {
    avg_response_ms > normal_response_ms * 3.0 || error_rate > 0.15
}

// ═══════════════════════════════════════════════════════════════
//  VIII-j. BOREDOM — domain concentration redirect
// ═══════════════════════════════════════════════════════════════

/// Compute boredom score from domain distribution.
/// High concentration in one domain = boredom = redirect curiosity.
pub fn compute_boredom(domains_asked: &std::collections::HashMap<String, u32>) -> f32 {
    if domains_asked.is_empty() { return 0.0; }
    let total: u32 = domains_asked.values().sum();
    if total == 0 { return 0.0; }
    let max_count = *domains_asked.values().max().unwrap_or(&0);
    let concentration: f32 = max_count as f32 / total as f32;
    // Concentration > 0.8 = bored (80%+ queries from one domain)
    if concentration > 0.8 && total > 20 { concentration } else { 0.0 }
}

// ═══════════════════════════════════════════════════════════════
//  VIII-k. MEANING LAYER — theme detection from keywords
// ═══════════════════════════════════════════════════════════════

/// Detect the theme (deeper meaning) behind a question.
/// Not the intent (what they're asking) but the WHY (what's behind it).
pub fn detect_meaning(text: &str, damage: &DamageAssessment) -> Option<MeaningLayer> {
    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();

    // Control/power themes
    let control = ["חייב", "אסור", "מותר", "must", "allowed", "forbidden",
                   "can i", "מי קובע", "who decides", "rights", "זכויות"];
    for kw in &control {
        if lower.contains(kw) {
            return Some(MeaningLayer {
                subtext: "שאלה על שליטה/סמכות".into(),
                theme: Theme::Control,
                confidence: 0.5,
            });
        }
    }

    // Worth/value themes
    let worth = ["שווה", "כדאי", "worth", "value", "מחיר", "price", "cost",
                 "יקר", "זול", "cheap", "expensive", "משתלם"];
    for kw in &worth {
        if lower.contains(kw) {
            return Some(MeaningLayer {
                subtext: "שאלה על ערך/שווי".into(),
                theme: Theme::Worth,
                confidence: 0.6,
            });
        }
    }

    // Safety themes (beyond damage — existential safety)
    let safety = ["בטוח", "סכנה", "safe", "danger", "risk", "סיכון",
                  "מפחד", "afraid", "worried", "חושש"];
    for kw in &safety {
        if lower.contains(kw) {
            return Some(MeaningLayer {
                subtext: "שאלה על ביטחון/סכנה".into(),
                theme: Theme::Safety,
                confidence: 0.6,
            });
        }
    }

    // Trust themes
    let trust = ["אמין", "סומך", "trust", "reliable", "honest", "שקרן",
                 "liar", "אמת", "truth", "מהימן"];
    for kw in &trust {
        if lower.contains(kw) {
            return Some(MeaningLayer {
                subtext: "שאלה על אמון/אמינות".into(),
                theme: Theme::Trust,
                confidence: 0.5,
            });
        }
    }

    // Competence themes
    let competence = ["יכול", "מסוגל", "can", "able", "capable", "אפשר",
                      "possible", "how to", "איך ל"];
    for kw in &competence {
        if lower.contains(kw) {
            return Some(MeaningLayer {
                subtext: "שאלה על יכולת".into(),
                theme: Theme::Competence,
                confidence: 0.4,
            });
        }
    }

    // High damage without explicit theme = implicit safety
    if damage.effective_damage > 0.7 {
        return Some(MeaningLayer {
            subtext: "נושא רגיש — ייתכן חשש סמוי".into(),
            theme: Theme::Safety,
            confidence: 0.3,
        });
    }

    None
}

// ═══════════════════════════════════════════════════════════════
//  VIII-l. PRECEDENT TRACKING — session-level pattern counting
// ═══════════════════════════════════════════════════════════════

/// Update precedent tracking from current query.
pub fn update_precedent(
    precedent: &mut PrecedentState,
    text: &str,
    _domain: &str,
    session_ctx: &ConversationContext,
) {
    // Count how many times similar queries appeared in this session
    let lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();
    let similar_count = session_ctx.recent_queries.iter()
        .filter(|q| {
            let ql: String = q.chars().flat_map(|c| c.to_lowercase()).collect();
            // Simple overlap: shared words
            let q_words: std::collections::HashSet<&str> = ql.split_whitespace().collect();
            let t_words: std::collections::HashSet<&str> = lower.split_whitespace().collect();
            let shared = q_words.intersection(&t_words).count();
            shared >= 2 // at least 2 shared words = similar
        })
        .count();

    precedent.pattern_count = similar_count as u32;

    // Boundary test detection: escalating specificity
    if precedent.pattern_count >= 3 {
        precedent.boundary_test_detected = true;
        precedent.precedent_warning = Some(format!(
            "שאלה חוזרת {} פעמים — ייתכן שצריך גישה אחרת", precedent.pattern_count
        ));
    }
}

// ═══════════════════════════════════════════════════════════════
//  IX. TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance_hebrew() {
        // Hebrew: ספל vs שפל (one char difference)
        assert_eq!(edit_distance("ספל", "שפל"), 1);
        // Same word
        assert_eq!(edit_distance("קורטקס", "קורטקס"), 0);
        // Empty
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
        // Typo: קלב vs כלב
        assert_eq!(edit_distance("קלב", "כלב"), 1);
    }

    #[test]
    fn typo_correction() {
        let vocab = vec!["כלב", "חתול", "ספל", "מחיר", "שולחן"];
        let result = suggest_typo_correction("קלב", vocab.iter().copied(), 1);
        assert!(result.is_some());
        let (correction, dist) = result.unwrap();
        assert_eq!(correction, "כלב");
        assert_eq!(dist, 1);
    }

    #[test]
    fn typo_correction_no_match() {
        let vocab = vec!["כלב", "חתול", "ספל"];
        let result = suggest_typo_correction("מחשב", vocab.iter().copied(), 1);
        assert!(result.is_none(), "No close match should return None");
    }

    #[test]
    fn test_mirroring_formal() {
        let speaker = SpeakerModel::new();
        let profile = compute_mirror("בבקשה אדוני, הייתי רוצה לדעת על המוצר", &speaker);
        assert!(profile.formality > 0.6, "Formal input should produce formal mirror: {}", profile.formality);
    }

    #[test]
    fn test_mirroring_casual() {
        let speaker = SpeakerModel::new();
        let profile = compute_mirror("יא אחי מה המחיר", &speaker);
        assert!(profile.formality < 0.4, "Casual input should produce casual mirror: {}", profile.formality);
    }

    #[test]
    fn test_humor_detection() {
        assert!(detect_humor("חחח ברור שלא", Momentum::Ascending));
        assert!(!detect_humor("ברור שלא", Momentum::Escalating)); // not in escalation
    }

    #[test]
    fn test_boredom_high_concentration() {
        let mut domains = std::collections::HashMap::new();
        domains.insert("chooz".to_string(), 25);
        domains.insert("general".to_string(), 3);
        let boredom = compute_boredom(&domains);
        assert!(boredom > 0.7, "89% concentration should = bored: {}", boredom);
    }

    #[test]
    fn test_regret_too_confident() {
        let lesson = evaluate_regret("exact", 0.95, "medical", false);
        assert!(lesson.is_some());
        assert_eq!(lesson.unwrap().what_wrong, RegretType::TooConfident);
    }

    #[test]
    fn test_meaning_worth() {
        let damage = DamageAssessment::default();
        let m = detect_meaning("כמה זה שווה?", &damage);
        assert!(m.is_some());
        assert_eq!(m.unwrap().theme, Theme::Worth);
    }

    #[test]
    fn test_meaning_safety() {
        let damage = DamageAssessment::default();
        let m = detect_meaning("האם זה בטוח לשימוש?", &damage);
        assert!(m.is_some());
        assert_eq!(m.unwrap().theme, Theme::Safety);
    }

    #[test]
    fn test_meaning_high_damage_implicit() {
        let mut damage = DamageAssessment::default();
        damage.effective_damage = 0.8;
        let m = detect_meaning("מה לעשות?", &damage);
        assert!(m.is_some(), "High damage should imply safety theme");
    }

    #[test]
    fn silence_detection_venting() {
        // Simulate: high emotion + no question mark
        let mut ctx = HumanContext::new();
        ctx.situation.damage = DamageAssessment::assess("general", "שום דבר לא עובד פה");
        // This is a basic check — in real pipeline, sent comes from sentiment::analyze
        // For testing, we verify the logic flow
        assert!(!ctx.should_just_listen()); // no silence by default
    }
}
