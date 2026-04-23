// ═══════════════════════════════════════════════════════════════
// gates231.rs — ר"לא שערים | 231 Gates of Sefer Yetzirah
// ═══════════════════════════════════════════════════════════════
// "עשרים ושתיים אותיות יסוד, קבען בגלגל כמין חומה,
//  במאתיים ושלושים ואחד שערים" — ספר יצירה ב:ד
//
// Each Hebrew word = a 22-dimensional vector (one dim per letter).
// Each pair of letters = a "gate" (שער) = a connection type.
// 231 = C(22,2) = all possible letter pairs.
//
// ZERO DEPS — pure Rust. Uses gematria.rs for letter values.
// ═══════════════════════════════════════════════════════════════

/// The 22 Hebrew letters in order (no sofit forms)
pub const LETTERS_22: [char; 22] = [
    'א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט', 'י',
    'כ', 'ל', 'מ', 'נ', 'ס', 'ע', 'פ', 'צ', 'ק', 'ר',
    'ש', 'ת',
];

/// Letter classification per Sefer Yetzirah
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LetterClass {
    Mother,  // 3 אמות: א, מ, ש
    Double,  // 7 כפולות: ב, ג, ד, כ, פ, ר, ת
    Simple,  // 12 פשוטות: ה, ו, ז, ח, ט, י, ל, נ, ס, ע, צ, ק
}

/// Element associated with Mother letters
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Element {
    Air,   // א = אוויר — mediator
    Water, // מ = מים — fluid
    Fire,  // ש = אש — transformation
}

/// Sofit (final) mapping
const SOFIT_TO_REGULAR: [(char, char); 5] = [
    ('ך', 'כ'), ('ם', 'מ'), ('ן', 'נ'), ('ף', 'פ'), ('ץ', 'צ'),
];

/// A 22-dimensional letter vector representing a Hebrew word
#[derive(Clone, Debug)]
pub struct LetterVec {
    /// Count of each letter (index = letter position in LETTERS_22)
    pub dims: [u8; 22],
    /// The original word
    pub word: String,
    /// Standard gematria value (scalar projection)
    pub gematria: u32,
}

/// A "gate" = pair of letters = connection type
#[derive(Clone, Debug)]
pub struct Gate {
    pub letter_a: char,
    pub letter_b: char,
    pub index: u16, // 0-230
    /// Class combination: Mother-Mother, Mother-Double, etc.
    pub class_a: LetterClass,
    pub class_b: LetterClass,
}

/// The 231 Gates engine
pub struct Gates231 {
    /// All 231 gates
    gates: Vec<Gate>,
    /// Letter index lookup
    letter_index: [i8; 128], // ASCII range quick lookup (Hebrew chars need special handling)
}

impl Gates231 {
    pub fn new() -> Self {
        let mut gates = Vec::with_capacity(231);
        let mut idx = 0u16;
        for i in 0..22 {
            for j in (i + 1)..22 {
                gates.push(Gate {
                    letter_a: LETTERS_22[i],
                    letter_b: LETTERS_22[j],
                    index: idx,
                    class_a: classify_letter(LETTERS_22[i]),
                    class_b: classify_letter(LETTERS_22[j]),
                });
                idx += 1;
            }
        }
        assert_eq!(gates.len(), 231); // C(22,2) = 231

        let letter_index = [0i8; 128]; // Will use char_to_index() instead for Hebrew

        Gates231 { gates, letter_index }
    }

    /// Convert a Hebrew word to a 22-dimensional letter vector
    pub fn word_to_vec(&self, word: &str) -> LetterVec {
        let mut dims = [0u8; 22];
        let mut gematria = 0u32;

        // CRITICAL: use char_indices for Hebrew safety
        for (_, ch) in word.char_indices() {
            let c = normalize_sofit(ch);
            if let Some(idx) = char_to_index(c) {
                dims[idx] += 1;
                gematria += letter_value(c);
            }
        }

        LetterVec {
            dims,
            word: word.to_string(),
            gematria,
        }
    }

    /// Calculate cosine distance between two letter vectors
    pub fn distance(&self, a: &LetterVec, b: &LetterVec) -> f64 {
        let mut dot = 0.0f64;
        let mut mag_a = 0.0f64;
        let mut mag_b = 0.0f64;

        for i in 0..22 {
            let va = a.dims[i] as f64;
            let vb = b.dims[i] as f64;
            dot += va * vb;
            mag_a += va * va;
            mag_b += vb * vb;
        }

        if mag_a == 0.0 || mag_b == 0.0 {
            return 1.0; // Maximum distance
        }

        1.0 - (dot / (mag_a.sqrt() * mag_b.sqrt()))
    }

    /// Find which gates a word passes through
    pub fn word_gates(&self, vec: &LetterVec) -> Vec<u16> {
        let mut active_gates = Vec::new();
        for gate in &self.gates {
            let idx_a = char_to_index(gate.letter_a).unwrap_or(0);
            let idx_b = char_to_index(gate.letter_b).unwrap_or(0);
            if vec.dims[idx_a] > 0 && vec.dims[idx_b] > 0 {
                active_gates.push(gate.index);
            }
        }
        active_gates
    }

    /// Find shared gates between two words
    pub fn shared_gates(&self, a: &LetterVec, b: &LetterVec) -> Vec<u16> {
        let gates_a = self.word_gates(a);
        let gates_b = self.word_gates(b);
        gates_a.iter().filter(|g| gates_b.contains(g)).copied().collect()
    }

    /// Get the gate for a specific letter pair
    pub fn get_gate(&self, a: char, b: char) -> Option<&Gate> {
        let ia = char_to_index(normalize_sofit(a))?;
        let ib = char_to_index(normalize_sofit(b))?;
        let (lo, hi) = if ia < ib { (ia, ib) } else { (ib, ia) };
        // Gate index formula: sum of (22-1) + (22-2) + ... + (22-lo) + (hi - lo - 1)
        let mut idx = 0usize;
        for k in 0..lo {
            idx += 22 - k - 1;
        }
        idx += hi - lo - 1;
        self.gates.get(idx)
    }

    /// Find all words in a set that share the most gates with a target
    pub fn find_similar(&self, target: &LetterVec, candidates: &[LetterVec], top_n: usize) -> Vec<(usize, f64)> {
        let mut scored: Vec<(usize, f64)> = candidates.iter().enumerate()
            .map(|(i, c)| (i, 1.0 - self.distance(target, c)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_n);
        scored
    }

    /// Get all 231 gates
    pub fn all_gates(&self) -> &[Gate] {
        &self.gates
    }

    /// Get the 3 Mother-Mother gates (א-מ, א-ש, מ-ש)
    pub fn mother_gates(&self) -> Vec<&Gate> {
        self.gates.iter()
            .filter(|g| g.class_a == LetterClass::Mother && g.class_b == LetterClass::Mother)
            .collect()
    }

    /// Count active gates per class combination
    pub fn gate_profile(&self, vec: &LetterVec) -> GateProfile {
        let active = self.word_gates(vec);
        let mut profile = GateProfile::default();

        for gate_idx in &active {
            let gate = &self.gates[*gate_idx as usize];
            match (gate.class_a, gate.class_b) {
                (LetterClass::Mother, LetterClass::Mother) => profile.mother_mother += 1,
                (LetterClass::Mother, LetterClass::Double) |
                (LetterClass::Double, LetterClass::Mother) => profile.mother_double += 1,
                (LetterClass::Mother, LetterClass::Simple) |
                (LetterClass::Simple, LetterClass::Mother) => profile.mother_simple += 1,
                (LetterClass::Double, LetterClass::Double) => profile.double_double += 1,
                (LetterClass::Double, LetterClass::Simple) |
                (LetterClass::Simple, LetterClass::Double) => profile.double_simple += 1,
                (LetterClass::Simple, LetterClass::Simple) => profile.simple_simple += 1,
            }
        }
        profile.total = active.len() as u16;
        profile
    }
}

/// Profile of gate types active in a word
#[derive(Clone, Debug, Default)]
pub struct GateProfile {
    pub mother_mother: u16,
    pub mother_double: u16,
    pub mother_simple: u16,
    pub double_double: u16,
    pub double_simple: u16,
    pub simple_simple: u16,
    pub total: u16,
}

// ─── Helper functions ────────────────────────────────────────

fn classify_letter(c: char) -> LetterClass {
    match c {
        'א' | 'מ' | 'ש' => LetterClass::Mother,
        'ב' | 'ג' | 'ד' | 'כ' | 'פ' | 'ר' | 'ת' => LetterClass::Double,
        _ => LetterClass::Simple,
    }
}

fn normalize_sofit(c: char) -> char {
    for (sofit, regular) in &SOFIT_TO_REGULAR {
        if c == *sofit { return *regular; }
    }
    c
}

fn char_to_index(c: char) -> Option<usize> {
    // CRITICAL: use match, not array index — Hebrew is multi-byte
    match c {
        'א' => Some(0),  'ב' => Some(1),  'ג' => Some(2),
        'ד' => Some(3),  'ה' => Some(4),  'ו' => Some(5),
        'ז' => Some(6),  'ח' => Some(7),  'ט' => Some(8),
        'י' => Some(9),  'כ' => Some(10), 'ל' => Some(11),
        'מ' => Some(12), 'נ' => Some(13), 'ס' => Some(14),
        'ע' => Some(15), 'פ' => Some(16), 'צ' => Some(17),
        'ק' => Some(18), 'ר' => Some(19), 'ש' => Some(20),
        'ת' => Some(21),
        _ => None,
    }
}

fn letter_value(c: char) -> u32 {
    match c {
        'א' => 1,   'ב' => 2,   'ג' => 3,   'ד' => 4,   'ה' => 5,
        'ו' => 6,   'ז' => 7,   'ח' => 8,   'ט' => 9,   'י' => 10,
        'כ' => 20,  'ל' => 30,  'מ' => 40,  'נ' => 50,  'ס' => 60,
        'ע' => 70,  'פ' => 80,  'צ' => 90,  'ק' => 100, 'ר' => 200,
        'ש' => 300, 'ת' => 400,
        _ => 0,
    }
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_231_gates_count() {
        let g = Gates231::new();
        assert_eq!(g.all_gates().len(), 231);
    }

    #[test]
    fn test_3_mother_gates() {
        let g = Gates231::new();
        let mothers = g.mother_gates();
        assert_eq!(mothers.len(), 3); // א-מ, א-ש, מ-ש
    }

    #[test]
    fn test_metatron_vector() {
        let g = Gates231::new();
        let v = g.word_to_vec("מטטרון");
        assert_eq!(v.gematria, 314);
        assert_eq!(v.dims[8], 2);  // ט appears twice
        assert_eq!(v.dims[12], 1); // מ appears once
        assert_eq!(v.dims[19], 1); // ר appears once
        assert_eq!(v.dims[5], 1);  // ו appears once
        assert_eq!(v.dims[13], 1); // נ appears once
    }

    #[test]
    fn test_same_word_zero_distance() {
        let g = Gates231::new();
        let v = g.word_to_vec("מיכאל");
        assert!(g.distance(&v, &v) < 1e-10, "self-distance should be ~0");
    }

    #[test]
    fn test_shared_gates_same_word() {
        let g = Gates231::new();
        let v = g.word_to_vec("מיכאל");
        let shared = g.shared_gates(&v, &v);
        let all = g.word_gates(&v);
        assert_eq!(shared.len(), all.len());
    }

    #[test]
    fn test_oneg_nega_same_gates() {
        // עונג and נגע — traditional teaching: same ROOT letters (ע,נ,ג) different order
        // But עונג includes ו (mater lectionis), so full-word gematria differs
        // Here we test the ROOT form without ו: ענג vs נגע
        let g = Gates231::new();
        let root_oneg = g.word_to_vec("ענג"); // root without ו
        let nega = g.word_to_vec("נגע");
        // Same 3 root letters = same active gates
        assert_eq!(g.word_gates(&root_oneg).len(), g.word_gates(&nega).len());
        // Same root gematria: ע(70)+נ(50)+ג(3)=123 = נ(50)+ג(3)+ע(70)=123
        assert_eq!(root_oneg.gematria, nega.gematria);
    }

    #[test]
    fn test_uriel_raziel_same_value() {
        let g = Gates231::new();
        let uriel = g.word_to_vec("אוריאל");
        let raziel = g.word_to_vec("רזיאל");
        // Both = 248, but different letter vectors
        assert_eq!(uriel.gematria, 248);
        assert_eq!(raziel.gematria, 248);
        // Different vectors despite same gematria
        assert_ne!(uriel.dims, raziel.dims);
        // Distance > 0
        assert!(g.distance(&uriel, &raziel) > 0.0);
    }

    #[test]
    fn test_gate_profile_mothers() {
        let g = Gates231::new();
        // אמש = only mother letters
        let v = g.word_to_vec("אמש");
        let profile = g.gate_profile(&v);
        assert_eq!(profile.mother_mother, 3); // א-מ, א-ש, מ-ש
        assert_eq!(profile.total, 3);
    }
}
