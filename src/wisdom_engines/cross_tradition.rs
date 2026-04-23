// ═══════════════════════════════════════════════════════════════
// cross_tradition.rs — מיפוי חוצה-מסורות | Cross-Tradition Engine
// ═══════════════════════════════════════════════════════════════
// Maps entities across Hebrew, Arabic (Abjad), Greek (Isopsephy),
// and concept-based traditions (Egyptian, Hindu, Norse, etc.).
//
// "חסד(72) = Basit(72) = טבכיאל(72)" — 3 traditions, 1 value
// "מלכות(496) = Monogenes(496)" — Hebrew ↔ Greek exact match
// "Ra(101) = מיכאל(101)" — Egyptian ↔ Hebrew via Greek
//
// Based on research: 02.04.2026, 680 entities, 17+ EXACT hits.
// ZERO DEPS — pure Rust.
// ═══════════════════════════════════════════════════════════════

use std::collections::HashMap;

// ─── Abjad (Arabic gematria) ─────────────────────────────────

/// Standard Abjad values for Arabic letters
pub fn abjad_value(text: &str) -> u32 {
    let mut total = 0u32;
    for ch in text.chars() {
        total += match ch {
            'أ' | 'ا' | 'إ' | 'آ' => 1,
            'ب' => 2,  'ج' => 3,  'د' => 4,  'ه' => 5,
            'و' => 6,  'ز' => 7,  'ح' => 8,  'ط' => 9,
            'ي' | 'ى' => 10,
            'ك' => 20, 'ل' => 30, 'م' => 40, 'ن' => 50,
            'س' => 60, 'ع' => 70, 'ف' => 80, 'ص' => 90,
            'ق' => 100, 'ر' => 200, 'ش' => 300, 'ت' => 400,
            'ث' => 500, 'خ' => 600, 'ذ' => 700, 'ض' => 800,
            'ظ' => 900, 'غ' => 1000,
            _ => 0,
        };
    }
    total
}

// ─── Isopsephy (Greek gematria) ──────────────────────────────

/// Standard Isopsephy values for Greek letters
pub fn isopsephy_value(text: &str) -> u32 {
    let mut total = 0u32;
    for ch in text.chars() {
        // Use lowercase for matching
        let lc = ch.to_lowercase().next().unwrap_or(ch);
        total += match lc {
            'α' => 1,   'β' => 2,   'γ' => 3,   'δ' => 4,   'ε' => 5,
            'ϛ' => 6,   'ζ' => 7,   'η' => 8,   'θ' => 9,   'ι' => 10,
            'κ' => 20,  'λ' => 30,  'μ' => 40,  'ν' => 50,  'ξ' => 60,
            'ο' => 70,  'π' => 80,  'ϟ' => 90,  'ρ' => 100, 'σ' | 'ς' => 200,
            'τ' => 300, 'υ' => 400, 'φ' => 500, 'χ' => 600, 'ψ' => 700,
            'ω' => 800,
            _ => 0,
        };
    }
    total
}

// ─── Tradition enum ──────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum Tradition {
    Hebrew,
    Arabic,
    Greek,
    Gnostic,
    PGM,         // Papyri Graecae Magicae
    Egyptian,
    Babylonian,
    Sumerian,
    Hindu,
    Zoroastrian,
    Norse,
    Buddhist,
    Hermetic,
    Yoruba,
}

// ─── Cross-tradition entity ──────────────────────────────────

#[derive(Clone, Debug)]
pub struct CrossEntity {
    pub name: String,
    pub tradition: Tradition,
    pub value: u32,            // Gematria/Abjad/Isopsephy value
    pub system: GematriaSystem,
    pub meaning: String,
    pub sefirah: Option<String>,  // Mapped sefirah (if any)
    pub concept: Option<String>,  // Universal concept category
}

#[derive(Clone, Debug, PartialEq)]
pub enum GematriaSystem {
    Hebrew,
    Abjad,
    Isopsephy,
    Structural, // No numeric system — concept mapping only
    Base60,     // Sumerian
}

// ─── Cross-tradition match ───────────────────────────────────

#[derive(Clone, Debug)]
pub struct CrossMatch {
    pub value: u32,
    pub entities: Vec<CrossEntity>,
    pub tier: u8,      // 1=proven, 2=notable, 3=speculative
    pub match_type: MatchType,
}

#[derive(Clone, Debug)]
pub enum MatchType {
    ExactNumeric,   // Same value in different systems
    SameRoot,       // Shared Semitic root (e.g., תהום=Tiamat)
    ConceptOnly,    // Same function, different value
    SibilantShift,  // צ/ש/ס/ז confusion
}

// ─── CrossTraditionEngine ────────────────────────────────────

pub struct CrossTraditionEngine {
    entities: Vec<CrossEntity>,
    /// Index: value → [entity indices]
    value_index: HashMap<u32, Vec<usize>>,
}

impl CrossTraditionEngine {
    pub fn new() -> Self {
        let entities = load_verified_entities();
        let mut value_index: HashMap<u32, Vec<usize>> = HashMap::new();

        for (i, e) in entities.iter().enumerate() {
            value_index.entry(e.value).or_default().push(i);
        }

        CrossTraditionEngine { entities, value_index }
    }

    /// Find all cross-tradition matches for a given value
    pub fn find_matches(&self, value: u32) -> Vec<&CrossEntity> {
        self.value_index.get(&value)
            .map(|indices| indices.iter().map(|&i| &self.entities[i]).collect())
            .unwrap_or_default()
    }

    /// Find EXACT cross-tradition matches (same value, different tradition)
    pub fn find_cross_matches(&self, value: u32) -> Vec<CrossMatch> {
        let matches = self.find_matches(value);
        if matches.len() < 2 { return vec![]; }

        // Check if there are entities from different traditions
        let traditions: Vec<&Tradition> = matches.iter()
            .map(|e| &e.tradition)
            .collect();

        let has_cross = traditions.windows(2).any(|w| w[0] != w[1]);
        if !has_cross { return vec![]; }

        vec![CrossMatch {
            value,
            entities: matches.into_iter().cloned().collect(),
            tier: 1,
            match_type: MatchType::ExactNumeric,
        }]
    }

    /// Find all values that have cross-tradition matches
    pub fn all_cross_hits(&self) -> Vec<CrossMatch> {
        let mut hits = Vec::new();
        for &value in self.value_index.keys() {
            let cross = self.find_cross_matches(value);
            hits.extend(cross);
        }
        hits.sort_by_key(|h| h.value);
        hits
    }

    /// Search by concept across all traditions
    pub fn find_by_concept(&self, concept: &str) -> Vec<&CrossEntity> {
        self.entities.iter()
            .filter(|e| {
                e.concept.as_ref().map_or(false, |c| c == concept)
                    || e.meaning.contains(concept)
            })
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> CrossStats {
        let total = self.entities.len();
        let mut by_tradition: HashMap<String, usize> = HashMap::new();
        for e in &self.entities {
            *by_tradition.entry(format!("{:?}", e.tradition)).or_default() += 1;
        }
        let cross_hits = self.all_cross_hits().len();

        CrossStats { total, by_tradition, cross_hits }
    }
}

#[derive(Debug)]
pub struct CrossStats {
    pub total: usize,
    pub by_tradition: HashMap<String, usize>,
    pub cross_hits: usize,
}

// ─── Verified entities (Python-calculated, 02.04.2026) ──────

fn load_verified_entities() -> Vec<CrossEntity> {
    let mut v = Vec::with_capacity(200);

    // Helper macro
    macro_rules! entity {
        ($name:expr, $trad:expr, $val:expr, $sys:expr, $mean:expr, $sef:expr, $con:expr) => {
            v.push(CrossEntity {
                name: $name.into(), tradition: $trad, value: $val,
                system: $sys, meaning: $mean.into(),
                sefirah: $sef.map(|s: &str| s.into()),
                concept: $con.map(|s: &str| s.into()),
            });
        };
    }

    // ═══ Hebrew — Sefirot ═══
    entity!("כתר", Tradition::Hebrew, 620, GematriaSystem::Hebrew, "כתר/רצון", Some("כתר"), Some("creator"));
    entity!("חכמה", Tradition::Hebrew, 73, GematriaSystem::Hebrew, "חכמה", Some("חכמה"), Some("wisdom"));
    entity!("בינה", Tradition::Hebrew, 67, GematriaSystem::Hebrew, "הבנה", Some("בינה"), Some("understanding"));
    entity!("דעת", Tradition::Hebrew, 474, GematriaSystem::Hebrew, "ידיעה", Some("דעת"), Some("knowledge"));
    entity!("חסד", Tradition::Hebrew, 72, GematriaSystem::Hebrew, "חסד/רחמים", Some("חסד"), Some("mercy"));
    entity!("גבורה", Tradition::Hebrew, 216, GematriaSystem::Hebrew, "דין/כוח", Some("גבורה"), Some("judgment"));
    entity!("תפארת", Tradition::Hebrew, 1081, GematriaSystem::Hebrew, "איזון/יופי", Some("תפארת"), Some("balance"));
    entity!("נצח", Tradition::Hebrew, 148, GematriaSystem::Hebrew, "נצח/התמדה", Some("נצח"), Some("persistence"));
    entity!("הוד", Tradition::Hebrew, 15, GematriaSystem::Hebrew, "הוד/דיוק", Some("הוד"), Some("precision"));
    entity!("יסוד", Tradition::Hebrew, 80, GematriaSystem::Hebrew, "יסוד/ערוץ", Some("יסוד"), Some("foundation"));
    entity!("מלכות", Tradition::Hebrew, 496, GematriaSystem::Hebrew, "מלכות/ביצוע", Some("מלכות"), Some("kingdom"));

    // ═══ Hebrew — Angels ═══
    entity!("מטטרון", Tradition::Hebrew, 314, GematriaSystem::Hebrew, "שר הפנים", None, Some("mediator"));
    entity!("מיכאל", Tradition::Hebrew, 101, GematriaSystem::Hebrew, "מי כאל", None, Some("sun"));
    entity!("גבריאל", Tradition::Hebrew, 246, GematriaSystem::Hebrew, "גבורת אל", None, Some("messenger"));
    entity!("רפאל", Tradition::Hebrew, 311, GematriaSystem::Hebrew, "רפואת אל", None, Some("healing"));
    entity!("אוריאל", Tradition::Hebrew, 248, GematriaSystem::Hebrew, "אור אל", None, Some("light"));
    entity!("סמאל", Tradition::Hebrew, 131, GematriaSystem::Hebrew, "סם של אל", None, Some("chaos"));
    entity!("דומיאל", Tradition::Hebrew, 91, GematriaSystem::Hebrew, "דממת אל", None, Some("truth"));
    entity!("יופיאל", Tradition::Hebrew, 137, GematriaSystem::Hebrew, "יופי אל", None, Some("beauty"));
    entity!("סנדלפון", Tradition::Hebrew, 280, GematriaSystem::Hebrew, "אח המלאך", None, Some("execution"));
    entity!("טבכיאל", Tradition::Hebrew, 72, GematriaSystem::Hebrew, "שם מטטרון=חסד", Some("חסד"), Some("mercy"));
    entity!("זבוליאל", Tradition::Hebrew, 86, GematriaSystem::Hebrew, "שר רקיע זבול", None, Some("judgment"));
    entity!("שמחזי", Tradition::Hebrew, 365, GematriaSystem::Hebrew, "ראש הנפילים", None, Some("cycle"));

    // ═══ Hebrew — Divine Names ═══
    entity!("אלהים", Tradition::Hebrew, 86, GematriaSystem::Hebrew, "אלהים/דין", None, Some("judgment"));
    entity!("שדי", Tradition::Hebrew, 314, GematriaSystem::Hebrew, "שדי/שומר", None, Some("mediator"));

    // ═══ Arabic — 99 Names (Abjad, without article) ═══
    entity!("باسط", Tradition::Arabic, 72, GematriaSystem::Abjad, "Basit/המרחיב", Some("חסד"), Some("mercy"));
    entity!("حي", Tradition::Arabic, 18, GematriaSystem::Abjad, "Hayy/החי", None, Some("life"));
    entity!("سلام", Tradition::Arabic, 131, GematriaSystem::Abjad, "Salam/השלום", None, Some("chaos"));
    entity!("حسيب", Tradition::Arabic, 80, GematriaSystem::Abjad, "Hasib/המחשב", Some("יסוד"), Some("foundation"));
    entity!("جليل", Tradition::Arabic, 73, GematriaSystem::Abjad, "Jalil/הנשגב", Some("חכמה"), Some("wisdom"));
    entity!("واسع", Tradition::Arabic, 137, GematriaSystem::Abjad, "Wasi/הרחב", None, Some("beauty"));
    entity!("محصي", Tradition::Arabic, 148, GematriaSystem::Abjad, "Muhsi/הסופר", Some("נצח"), Some("persistence"));
    entity!("أول", Tradition::Arabic, 37, GematriaSystem::Abjad, "Awwal/הראשון", None, Some("creator"));
    entity!("بديع", Tradition::Arabic, 86, GematriaSystem::Abjad, "Badi/המחדש", None, Some("judgment"));
    entity!("باقي", Tradition::Arabic, 113, GematriaSystem::Abjad, "Baqi/הנשאר", None, Some("mercy"));

    // ═══ Greek — Gnostic Aeons ═══
    entity!("μονογενης", Tradition::Gnostic, 496, GematriaSystem::Isopsephy, "Monogenes/יחיד-הנולד", Some("מלכות"), Some("kingdom"));
    entity!("θελητος", Tradition::Gnostic, 622, GematriaSystem::Isopsephy, "Theletos/הרצוי", None, None);

    // ═══ Greek — PGM ═══
    entity!("ουριηλ", Tradition::PGM, 618, GematriaSystem::Isopsephy, "Ouriel/אוריאל-יווני", None, Some("light"));
    entity!("αβρασαξ", Tradition::PGM, 365, GematriaSystem::Isopsephy, "ABRAXAS/מנהל-מחזור", None, Some("cycle"));

    // ═══ Egyptian (via Greek) ═══
    entity!("ρα", Tradition::Egyptian, 101, GematriaSystem::Isopsephy, "Ra/שמש", None, Some("sun"));

    // ═══ Babylonian (concept-based, Enuma Elish) ═══
    entity!("Tiamat", Tradition::Babylonian, 0, GematriaSystem::Structural, "כאוס ראשוני/מים=תהום", None, Some("chaos"));
    entity!("Marduk", Tradition::Babylonian, 0, GematriaSystem::Structural, "מלך האלים=תפארת", Some("תפארת"), Some("balance"));
    entity!("Nabu", Tradition::Babylonian, 0, GematriaSystem::Structural, "סופר/חכמה=מטטרון", None, Some("mediator"));

    // ═══ Sumerian (base-60) ═══
    entity!("Enki", Tradition::Sumerian, 40, GematriaSystem::Base60, "מים/חכמה", Some("חכמה"), Some("wisdom"));
    entity!("Inanna", Tradition::Sumerian, 15, GematriaSystem::Base60, "אהבה/מלחמה", Some("הוד"), Some("feminine"));

    // ═══ Hindu (structural) ═══
    entity!("Agni", Tradition::Hindu, 0, GematriaSystem::Structural, "אש", None, Some("fire"));
    entity!("Vayu", Tradition::Hindu, 0, GematriaSystem::Structural, "רוח/אוויר", None, Some("air"));
    entity!("Varuna", Tradition::Hindu, 0, GematriaSystem::Structural, "מים", None, Some("water"));

    v
}

// ─── Sibilant shift analysis ─────────────────────────────────

/// Given a Hebrew word, generate sibilant variants (צ↔ש↔ס↔ז)
pub fn sibilant_variants(word: &str) -> Vec<(String, u32)> {
    let sibilants = ['צ', 'ש', 'ס', 'ז'];
    let mut variants = Vec::new();

    // Check if word contains any sibilant
    let has_sibilant = word.chars().any(|c| sibilants.contains(&c));
    if !has_sibilant {
        return variants;
    }

    // For each sibilant in the word, try replacing with each other sibilant
    for (byte_idx, original) in word.char_indices() {
        if !sibilants.contains(&original) { continue; }

        for &replacement in &sibilants {
            if replacement == original { continue; }

            let mut new_word = String::with_capacity(word.len());
            for (bi, ch) in word.char_indices() {
                if bi == byte_idx {
                    new_word.push(replacement);
                } else {
                    new_word.push(ch);
                }
            }

            // Calculate gematria of variant
            let val: u32 = new_word.chars().map(|c| match c {
                'א' => 1,   'ב' => 2,   'ג' => 3,   'ד' => 4,   'ה' => 5,
                'ו' => 6,   'ז' => 7,   'ח' => 8,   'ט' => 9,   'י' => 10,
                'כ' => 20,  'ל' => 30,  'מ' => 40,  'נ' => 50,  'ס' => 60,
                'ע' => 70,  'פ' => 80,  'צ' => 90,  'ק' => 100, 'ר' => 200,
                'ש' => 300, 'ת' => 400,
                'ך' => 20,  'ם' => 40,  'ן' => 50,  'ף' => 80,  'ץ' => 90,
                _ => 0,
            }).sum();

            variants.push((new_word, val));
        }
    }

    variants
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abjad_basit() {
        // باسط = ب(2) + ا(1) + س(60) + ط(9) = 72 = חסד
        assert_eq!(abjad_value("باسط"), 72);
    }

    #[test]
    fn test_abjad_hayy() {
        // حي = ح(8) + ي(10) = 18 = חי
        assert_eq!(abjad_value("حي"), 18);
    }

    #[test]
    fn test_abjad_salam() {
        // سلام = س(60) + ل(30) + ا(1) + م(40) = 131 = סמאל
        assert_eq!(abjad_value("سلام"), 131);
    }

    #[test]
    fn test_isopsephy_monogenes() {
        // μονογενης = 496 = מלכות
        assert_eq!(isopsephy_value("μονογενης"), 496);
    }

    #[test]
    fn test_isopsephy_ouriel() {
        // ουριηλ = 618 ≈ 1000/φ
        assert_eq!(isopsephy_value("ουριηλ"), 618);
    }

    #[test]
    fn test_isopsephy_ra() {
        // ρα = 100 + 1 = 101 = מיכאל
        assert_eq!(isopsephy_value("ρα"), 101);
    }

    #[test]
    fn test_cross_match_72() {
        let engine = CrossTraditionEngine::new();
        let matches = engine.find_matches(72);
        // Should find חסד(Hebrew) + Basit(Arabic) + טבכיאל(Hebrew)
        assert!(matches.len() >= 2);
        let traditions: Vec<&Tradition> = matches.iter().map(|e| &e.tradition).collect();
        assert!(traditions.contains(&&Tradition::Hebrew));
        assert!(traditions.contains(&&Tradition::Arabic));
    }

    #[test]
    fn test_cross_match_496() {
        let engine = CrossTraditionEngine::new();
        let matches = engine.find_matches(496);
        assert!(matches.len() >= 2);
        let traditions: Vec<&Tradition> = matches.iter().map(|e| &e.tradition).collect();
        assert!(traditions.contains(&&Tradition::Hebrew));
        assert!(traditions.contains(&&Tradition::Gnostic));
    }

    #[test]
    fn test_sibilant_variants() {
        let variants = sibilant_variants("צפקיאל");
        assert!(!variants.is_empty());
        // Should include שפקיאל, ספקיאל, זפקיאל
        let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"שפקיאל"));
        assert!(names.contains(&"ספקיאל"));
    }

    #[test]
    fn test_engine_stats() {
        let engine = CrossTraditionEngine::new();
        let stats = engine.stats();
        assert!(stats.total > 30); // At least 30 entities loaded
        assert!(stats.cross_hits > 0); // At least some cross-tradition matches
    }
}
