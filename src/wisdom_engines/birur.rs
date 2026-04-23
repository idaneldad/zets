//! # Birur — PrecisionVec + BirurGate (Dumiel=91)
//!
//! PrecisionVec = 10 dimensions of confidence, mapped to Sefirot.
//! Each daemon guardian reads specific dimensions.
//! BirurGate = last checkpoint before execution.
//!
//! דומיאל = 91 = יהוה(26) + אדני(65) = Bridge heaven↔earth

/// 10-dimensional confidence vector. Da'at = mean(all).
#[derive(Debug, Clone)]
pub struct PrecisionVec {
    pub intent_clarity: f64,        // 0 — Keter → Metatron(L6)
    pub source_diversity: f64,      // 1 — Chokhmah → Uriel(L5)
    pub recency: f64,               // 2 — Binah → Uriel(L5)
    pub identity_confidence: f64,   // 3 — Chesed → Metatron(L6)
    pub financial_trust: f64,       // 4 — Gevurah → Dumiel(L3)
    pub relationship_strength: f64, // 5 — Tiferet → Dumiel(L3)
    pub org_role_confidence: f64,   // 6 — Netzach → Gabriel(L4)
    pub session_risk: f64,          // 7 — Hod → Samael(⊥)
    pub rhetorical_mode: f64,       // 8 — Yesod → Sandalphon(L1)
    pub gut_feeling: f64,           // 9 — Emotion → Raphael(⊥)
}

impl Default for PrecisionVec {
    fn default() -> Self {
        PrecisionVec {
            intent_clarity: 0.5, source_diversity: 0.5, recency: 0.5,
            identity_confidence: 0.5, financial_trust: 0.5, relationship_strength: 0.5,
            org_role_confidence: 0.5, session_risk: 0.3, rhetorical_mode: 0.0, gut_feeling: 0.5,
        }
    }
}

impl PrecisionVec {
    /// Da'at — the hidden 10th sefirah. Mean of all dimensions.
    pub fn daat(&self) -> f64 {
        let vals = [
            self.intent_clarity, self.source_diversity, self.recency,
            self.identity_confidence, self.financial_trust, self.relationship_strength,
            self.org_role_confidence, 1.0 - self.session_risk,
            self.rhetorical_mode, self.gut_feeling,
        ];
        vals.iter().sum::<f64>() / vals.len() as f64
    }

    pub fn weakest(&self) -> (&'static str, f64) {
        let dims = [
            ("intent_clarity", self.intent_clarity),
            ("source_diversity", self.source_diversity),
            ("recency", self.recency),
            ("identity_confidence", self.identity_confidence),
            ("financial_trust", self.financial_trust),
            ("relationship_strength", self.relationship_strength),
            ("org_role_confidence", self.org_role_confidence),
            ("session_risk_inv", 1.0 - self.session_risk),
            ("rhetorical_mode", self.rhetorical_mode),
            ("gut_feeling", self.gut_feeling),
        ];
        dims.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap().clone()
    }

    pub fn apply_emotion(&mut self, valence: f64) {
        self.gut_feeling = (self.gut_feeling + valence * 0.3).clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BirurAction {
    Pass,
    Assisted,
    EscalateS2,
    FallThrough,
}

#[derive(Debug)]
pub struct BirurDecision {
    pub action: BirurAction,
    pub reason: String,
    pub weak_dim: String,
}

/// BirurGate — שם מ"ב = 42 gates (7 sefirot × 6 categories).
/// Each gate is a threshold check. FallThrough only if majority (>21) gates fail.
pub fn evaluate(pv: &PrecisionVec, involves_financial: bool) -> BirurDecision {
    let _daat = pv.daat();

    // ═══ 42 GATES: 7 sefirot × 6 categories ═══
    // Categories: lexical(intent), semantic(source), structural(recency),
    //             contextual(identity), emotional(gut), ethical(risk)
    // Each dimension checked against 6 thresholds = 6 gates per dimension
    let thresholds: [f64; 6] = [0.20, 0.30, 0.40, 0.50, 0.60, 0.70];
    let dimensions = [
        pv.intent_clarity,       // chesed gates (0-5)
        pv.source_diversity,     // gevurah gates (6-11)
        pv.recency,              // tiferet gates (12-17)
        pv.identity_confidence,  // netzach gates (18-23)
        pv.financial_trust,      // hod gates (24-29)
        pv.relationship_strength,// yesod gates (30-35)
        pv.gut_feeling,          // malkhut gates (36-41)
    ];

    let mut gates_passed: usize = 0;
    for dim_val in &dimensions {
        for thresh in &thresholds {
            if dim_val >= thresh {
                gates_passed += 1;
            }
        }
    }
    // Total possible = 42 (7 dims × 6 thresholds)
    // gates_passed = how many of the 42 checks this query passes

    // Hard blocks (override gates — safety)
    if pv.session_risk > 0.8 {
        return BirurDecision { action: BirurAction::FallThrough,
            reason: format!("session_risk_high gates={}/42", gates_passed), weak_dim: "session_risk".into() };
    }
    if involves_financial && pv.financial_trust < 0.5 {
        return BirurDecision { action: BirurAction::FallThrough,
            reason: format!("financial_trust_low gates={}/42", gates_passed), weak_dim: "financial_trust".into() };
    }

    // Gate-based routing (שם מ"ב)
    if gates_passed < 12 {
        // < 12/42 = very weak → FallThrough
        return BirurDecision { action: BirurAction::FallThrough,
            reason: format!("gates={}/42 < 12", gates_passed), weak_dim: pv.weakest().0.to_string() };
    }
    if gates_passed < 21 {
        // < 21/42 = below majority → needs S2 deliberation
        return BirurDecision { action: BirurAction::EscalateS2,
            reason: format!("gates={}/42 < majority", gates_passed), weak_dim: pv.weakest().0.to_string() };
    }
    if gates_passed < 30 {
        // 21-29/42 = marginal → assisted
        return BirurDecision { action: BirurAction::Assisted,
            reason: format!("gates={}/42 marginal", gates_passed), weak_dim: pv.weakest().0.to_string() };
    }
    // ≥ 30/42 = strong → pass
    BirurDecision { action: BirurAction::Pass,
        reason: format!("gates={}/42 strong", gates_passed), weak_dim: String::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_daat() {
        let pv = PrecisionVec::default();
        let d = pv.daat();
        assert!(d > 0.3 && d < 0.7);
    }

    #[test]
    fn test_birur_pass() {
        let pv = PrecisionVec {
            intent_clarity: 0.9, source_diversity: 0.8, recency: 0.7,
            identity_confidence: 0.8, financial_trust: 0.7, relationship_strength: 0.6,
            org_role_confidence: 0.7, session_risk: 0.1, rhetorical_mode: 0.5, gut_feeling: 0.7,
        };
        let d = evaluate(&pv, false);
        assert_eq!(d.action, BirurAction::Pass);
    }

    #[test]
    fn test_birur_fallthrough_low() {
        let pv = PrecisionVec {
            intent_clarity: 0.1, source_diversity: 0.1, recency: 0.1,
            identity_confidence: 0.1, financial_trust: 0.1, relationship_strength: 0.1,
            org_role_confidence: 0.1, session_risk: 0.8, rhetorical_mode: 0.0, gut_feeling: 0.1,
        };
        let d = evaluate(&pv, false);
        assert_eq!(d.action, BirurAction::FallThrough);
    }

    #[test]
    fn test_birur_financial() {
        let pv = PrecisionVec { financial_trust: 0.2, ..PrecisionVec::default() };
        let d = evaluate(&pv, true);
        assert_eq!(d.action, BirurAction::FallThrough);
    }
}
