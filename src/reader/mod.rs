//! # Reader — the entity reader
//!
//! Reads a `ReadInput` (which wraps a message, its `Source`, and context)
//! and produces a `Reading` (emotion + intent + style + gate + directive).
//!
//! ## What is a Source?
//!
//! A source can be:
//!   - A **human User** (Owner, Collaborator, Admin) — the person who
//!     configured the ZETS instance, or someone working with them.
//!   - A **Client** (Direct, Related, Prospect, Referrer) — someone the
//!     Owner serves. Most B2B traffic comes through this path.
//!   - A **Guest** — anonymous or first-contact.
//!   - An **ExternalApi** (Automation, Webhook, Batch, Integration) —
//!     machines calling ZETS from outside (Zapier, webhooks, etc).
//!   - A **PeerZets** or **ZetsMaster** — another ZETS instance or the
//!     master orchestrator.
//!   - A **SelfInitiated** — ZETS itself (curiosity, background learning,
//!     reflection, maintenance).
//!
//! The same reading logic applies to all, but the *tuning* changes —
//! human sources get emotion + style detection and personality adaptation;
//! API sources skip mirroring and mostly use the gate for access control.
//!
//! ## Design principles
//!
//! 1. **Knowledge lives in the graph** — emotion signals, intent kinds,
//!    style dimensions, gate thresholds are all graph atoms + edges.
//!    This file is orchestration, not content.
//!
//! 2. **Thin structs, heavy graph walks** — `Reading` is mostly AtomIds
//!    and confidences. The *meaning* of each AtomId is resolved via
//!    graph traversal at runtime, which means tuning the system is
//!    editing the graph, not recompiling.
//!
//! 3. **Source determines what modules apply** — not every reader module
//!    runs for every source. `ExternalApi` skips style + directive;
//!    `SelfInitiated` skips gate (trust is pre-established).
//!
//! 4. **Two directions of quality checking**:
//!    - `Reader::read()` reads *input* — what the source said.
//!    - `Reader::check_output()` reads *output* — whether ZETS's draft
//!      response is worthy to send back. This is the Birur-style gate
//!      applied to the other direction.

pub mod emotion;
pub mod input;
pub mod intent;
pub mod style;
pub mod reading;
pub mod source;

pub use input::{Author, HistoryEntry, ReadInput, SessionContext, SessionSignals};
pub use reading::{
    BigFive, EmotionRead, EmotionalState, GateAction, GateRead, Hint, IntentRead,
    PragmaticIntent, Reading, ResponseDirective, StyleRead, UpliftMethod,
};
pub use source::{ApiKind, ClientRole, SelfOrigin, Source, TrustTier, UserRole};

/// The Reader — turns a ReadInput into a Reading.
///
/// Stateless orchestrator. State lives in the graph (profiles, session
/// context) or in the caller's ownership (history, session).
pub struct Reader;

impl Reader {
    /// Read the input — the main entry point.
    ///
    /// Pipeline:
    ///   1. Emotion signals (what do I feel in the text?)
    ///   2. Intent (what do they actually mean?)
    ///   3. Style (how do they communicate?)
    ///   4. Gate (should I respond? how carefully?)
    ///   5. Directive (if yes, what kind of response?)
    ///
    /// Each stage may short-circuit if a prior stage hit a hard block
    /// (e.g. gate says Hold → don't compute directive).
    pub fn read(input: &ReadInput) -> Reading {
        let mut reading = Reading::default();

        // Skip human-only layers for non-human sources.
        let is_human = input.source.is_human();

        // Stage 1: Emotion (human only)
        if is_human {
            reading.emotion = Self::read_emotion(input);
        }

        // Stage 2: Intent (always)
        reading.intent = Self::read_intent(input);

        // Stage 3: Style (human only)
        if is_human {
            reading.style = Self::read_style(input);
        }

        // Stage 4: Gate (always — even APIs need permission checks)
        reading.gate = Self::read_gate(input, &reading);

        // Stage 5: Directive (only if gate allows a response)
        if reading.gate.action != GateAction::Hold {
            reading.directive = Self::compose_directive(input, &reading);
        }

        // Overall confidence — combines per-stage confidence.
        reading.confidence = Self::compute_confidence(&reading, input);

        reading
    }

    /// Check whether a draft output is good enough to send.
    ///
    /// This is the **second** gate — after ZETS has drafted a response,
    /// before sending it back. Same 42-gate logic as input gate, but
    /// scored on the output relative to the input.
    ///
    /// Returns `GateAction` — if `Hold`, the response should be
    /// regenerated or held back.
    pub fn check_output(
        input: &ReadInput,
        reading: &Reading,
        draft: &str,
    ) -> GateRead {
        Self::read_output_quality(input, reading, draft)
    }

    // ─── Stage implementations (stubs for now, filled in Phase 2) ───

    fn read_emotion(input: &ReadInput) -> EmotionRead {
        emotion::EmotionDetector::detect(input)
    }

    fn read_intent(input: &ReadInput) -> IntentRead {
        intent::IntentDetector::detect(input)
    }

    fn read_style(input: &ReadInput) -> StyleRead {
        style::StyleDetector::detect(input)
    }

    fn read_gate(input: &ReadInput, _reading: &Reading) -> GateRead {
        // Phase 3: 7-dim × 6-threshold precision vector → 42 gates.
        // Stub: grant Pass for Owner, Hold for unknown API.
        let default_action = match input.source.default_trust() {
            TrustTier::Full | TrustTier::High => GateAction::Pass,
            TrustTier::Known | TrustTier::Mid => GateAction::Pass,
            TrustTier::Limited => GateAction::Assisted,
            TrustTier::Cautious => GateAction::Assisted,
        };
        GateRead {
            action: default_action,
            reason: format!("stub:trust={:?}", input.source.default_trust()),
            weak_dim: None,
            gates_passed: 21, // majority by default
        }
    }

    fn compose_directive(_input: &ReadInput, _reading: &Reading) -> ResponseDirective {
        // Phase 4: energy computation, mirror selection, uplift strategy.
        ResponseDirective::default()
    }

    fn read_output_quality(
        _input: &ReadInput,
        _reading: &Reading,
        _draft: &str,
    ) -> GateRead {
        // Phase 3b: same gate logic applied to output.
        // Pass for now.
        GateRead {
            action: GateAction::Pass,
            reason: "stub:output_check".into(),
            weak_dim: None,
            gates_passed: 30,
        }
    }

    fn compute_confidence(_reading: &Reading, input: &ReadInput) -> f32 {
        // Phase 2: aggregate per-stage confidences.
        // Stub: higher for longer messages (more signal).
        let base = match input.word_count() {
            0..=2 => 0.3,
            3..=10 => 0.6,
            _ => 0.8,
        };
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_session() -> SessionContext {
        SessionContext::new("test_session", 1745400000000)
    }

    #[test]
    fn test_read_owner_pass() {
        let src = Source::User {
            id: "idan".into(),
            role: UserRole::Owner,
        };
        let sess = mk_session();
        let input = ReadInput::new("שלום איך הולך", &src, &[], &sess);

        let reading = Reader::read(&input);

        assert_eq!(reading.gate.action, GateAction::Pass);
        assert!(reading.confidence > 0.0);
    }

    #[test]
    fn test_read_guest_assisted() {
        let src = Source::Guest {
            session: "anon".into(),
        };
        let sess = mk_session();
        let input = ReadInput::new("what is this?", &src, &[], &sess);

        let reading = Reader::read(&input);

        // Guest has Cautious trust → Assisted gate by default
        assert_eq!(reading.gate.action, GateAction::Assisted);
    }

    #[test]
    fn test_read_api_skips_style() {
        let src = Source::ExternalApi {
            id: "zapier".into(),
            kind: ApiKind::Automation,
        };
        let sess = mk_session();
        let input = ReadInput::new(r#"{"event":"new_lead"}"#, &src, &[], &sess);

        let reading = Reader::read(&input);

        // API sources don't get emotion read → primary stays Neutral
        assert_eq!(reading.emotion.primary, EmotionalState::Neutral);
        // Big Five stays at default (0.0 = unknown)
        assert_eq!(reading.style.big_five.openness, 0.0);
    }

    #[test]
    fn test_confidence_higher_for_longer() {
        let src = Source::User {
            id: "x".into(),
            role: UserRole::Owner,
        };
        let sess = mk_session();

        let short = ReadInput::new("hi", &src, &[], &sess);
        let long = ReadInput::new(
            "hi there I have a question about something important today",
            &src,
            &[],
            &sess,
        );

        let r_short = Reader::read(&short);
        let r_long = Reader::read(&long);

        assert!(r_long.confidence > r_short.confidence);
    }

    #[test]
    fn test_output_check_returns_gate() {
        let src = Source::User {
            id: "idan".into(),
            role: UserRole::Owner,
        };
        let sess = mk_session();
        let input = ReadInput::new("how's the weather?", &src, &[], &sess);
        let reading = Reader::read(&input);

        let output_gate = Reader::check_output(&input, &reading, "It's sunny.");
        assert_eq!(output_gate.action, GateAction::Pass);
    }

    #[test]
    fn test_self_initiated_input_runs() {
        let src = Source::SelfInitiated {
            origin: SelfOrigin::Curiosity,
        };
        let sess = mk_session();
        let input = ReadInput::new("explore entropy concepts", &src, &[], &sess);

        let reading = Reader::read(&input);

        // Self has Full trust, but also not_human → no emotion/style
        assert_eq!(reading.gate.action, GateAction::Pass);
        assert_eq!(reading.emotion.primary, EmotionalState::Neutral);
    }
}
