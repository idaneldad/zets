// ═══════════════════════════════════════════════════════════════
// gematria.rs — מנוע גימטריה | DINIO Cortex V7.11.0
// ═══════════════════════════════════════════════════════════════
// חישוב ערכי גימטריה, reverse lookup, וזיהוי קשרים מתמטיים.
// ZERO DEPS — pure Rust, Hebrew-safe (char_indices always).
//
// שיטות: standard, ordinal, reduced, atbash, milui, kolel
// ═══════════════════════════════════════════════════════════════

use std::collections::HashMap;

// ─── Hebrew letter values ────────────────────────────────────

const STANDARD: [(char, u32); 27] = [
    ('א', 1), ('ב', 2), ('ג', 3), ('ד', 4), ('ה', 5),
    ('ו', 6), ('ז', 7), ('ח', 8), ('ט', 9), ('י', 10),
    ('כ', 20), ('ל', 30), ('מ', 40), ('נ', 50), ('ס', 60),
    ('ע', 70), ('פ', 80), ('צ', 90), ('ק', 100), ('ר', 200),
    ('ש', 300), ('ת', 400),
    // Sofit forms
    ('ך', 500), ('ם', 600), ('ן', 700), ('ף', 800), ('ץ', 900),
];

const STANDARD_NO_SOFIT: [(char, u32); 27] = [
    ('א', 1), ('ב', 2), ('ג', 3), ('ד', 4), ('ה', 5),
    ('ו', 6), ('ז', 7), ('ח', 8), ('ט', 9), ('י', 10),
    ('כ', 20), ('ל', 30), ('מ', 40), ('נ', 50), ('ס', 60),
    ('ע', 70), ('פ', 80), ('צ', 90), ('ק', 100), ('ר', 200),
    ('ש', 300), ('ת', 400),
    // Sofit = same as regular
    ('ך', 20), ('ם', 40), ('ן', 50), ('ף', 80), ('ץ', 90),
];

const ORDINAL: [(char, u32); 27] = [
    ('א', 1), ('ב', 2), ('ג', 3), ('ד', 4), ('ה', 5),
    ('ו', 6), ('ז', 7), ('ח', 8), ('ט', 9), ('י', 10),
    ('כ', 11), ('ל', 12), ('מ', 13), ('נ', 14), ('ס', 15),
    ('ע', 16), ('פ', 17), ('צ', 18), ('ק', 19), ('ר', 20),
    ('ש', 21), ('ת', 22),
    ('ך', 11), ('ם', 13), ('ן', 14), ('ף', 17), ('ץ', 18),
];

const ATBASH_PAIRS: [(char, char); 22] = [
    ('א', 'ת'), ('ב', 'ש'), ('ג', 'ר'), ('ד', 'ק'), ('ה', 'צ'),
    ('ו', 'פ'), ('ז', 'ע'), ('ח', 'ס'), ('ט', 'נ'), ('י', 'מ'),
    ('כ', 'ל'), ('ל', 'כ'), ('מ', 'י'), ('נ', 'ט'), ('ס', 'ח'),
    ('ע', 'ז'), ('פ', 'ו'), ('צ', 'ה'), ('ק', 'ד'), ('ר', 'ג'),
    ('ש', 'ב'), ('ת', 'א'),
];

// Milui (spelled-out names of letters)
const MILUI: [(char, &str, u32); 22] = [
    ('א', "אלף", 111), ('ב', "בית", 412), ('ג', "גימל", 83), ('ד', "דלת", 434),
    ('ה', "הא", 6),     ('ו', "ואו", 13),  ('ז', "זין", 67),  ('ח', "חית", 418),
    ('ט', "טית", 419),  ('י', "יוד", 20),  ('כ', "כף", 100),  ('ל', "למד", 74),
    ('מ', "מם", 80),    ('נ', "נון", 106),  ('ס', "סמך", 120), ('ע', "עין", 130),
    ('פ', "פא", 81),    ('צ', "צדי", 104),  ('ק', "קוף", 186), ('ר', "ריש", 510),
    ('ש', "שין", 360),  ('ת', "תו", 406),
];

// ─── Known sacred values (preloaded) ─────────────────────────

#[derive(Clone, Debug)]
pub struct KnownValue {
    pub word: String,
    pub value: u32,
    pub domain: String,
}

fn preload_known() -> Vec<KnownValue> {
    let entries = [
        // ספירות
        ("כתר", 620, "sefirot"), ("חכמה", 73, "sefirot"), ("בינה", 67, "sefirot"),
        ("דעת", 474, "sefirot"), ("חסד", 72, "sefirot"), ("גבורה", 216, "sefirot"),
        ("תפארת", 1081, "sefirot"), ("נצח", 148, "sefirot"), ("הוד", 15, "sefirot"),
        ("יסוד", 80, "sefirot"), ("מלכות", 496, "sefirot"),
        // מלאכים
        ("מטטרון", 314, "angel"), ("אוריאל", 248, "angel"), ("מיכאל", 101, "angel"),
        ("גבריאל", 246, "angel"), ("רפאל", 311, "angel"), ("סנדלפון", 280, "angel"),
        ("סמאל", 131, "angel"), ("דומיאל", 91, "angel"), ("רזיאל", 248, "angel"),
        // שמות הקודש
        ("יהוה", 26, "name"), ("אלהים", 86, "name"), ("אדני", 65, "name"),
        ("שדי", 314, "name"), ("אהיה", 21, "name"), ("צבאות", 499, "name"),
        // מושגים
        ("אמת", 441, "concept"), ("שכינה", 385, "concept"), ("שלום", 376, "concept"),
        ("אור", 207, "concept"), ("תורה", 611, "concept"), ("ישראל", 541, "concept"),
        ("משיח", 358, "concept"), ("נשמה", 395, "concept"),
        // 3 אמות
        ("אש", 301, "element"), ("מים", 90, "element"), ("אוויר", 227, "element"),
    ];
    entries.iter().map(|(w, v, d)| KnownValue {
        word: w.to_string(), value: *v, domain: d.to_string(),
    }).collect()
}

// ─── Math relation detection ─────────────────────────────────

#[derive(Clone, Debug)]
pub struct MathRelation {
    pub name: String,
    pub formula: String,
    pub error_pct: f64,
    pub tier: u8, // 1=proven, 2=notable, 3=inspiration
}

fn find_math_relations(value: u32) -> Vec<MathRelation> {
    let mut rels = Vec::new();
    let v = value as f64;

    // π relationships
    let pi = std::f64::consts::PI;
    for (mult, name) in [(100.0, "π×100"), (1000.0, "π×1000"), (10.0, "π×10")] {
        let err = ((v - pi * mult) / (pi * mult) * 100.0).abs();
        if err < 2.0 {
            rels.push(MathRelation { name: name.into(), formula: format!("{:.4}×{}", pi, mult),
                error_pct: err, tier: if err < 0.1 { 1 } else { 2 } });
        }
    }
    // φ relationships
    let phi = 1.618033988749895;
    for (mult, name) in [(1000.0, "φ×1000"), (100.0, "φ×100")] {
        let err = ((v - phi * mult) / (phi * mult) * 100.0).abs();
        if err < 2.0 {
            rels.push(MathRelation { name: name.into(), formula: format!("{:.6}×{}", phi, mult),
                error_pct: err, tier: if err < 0.1 { 1 } else { 2 } });
        }
    }
    let inv_phi = 0.618033988749895;
    for (mult, name) in [(1000.0, "1/φ×1000"), (100.0, "1/φ×100")] {
        let err = ((v - inv_phi * mult) / (inv_phi * mult) * 100.0).abs();
        if err < 2.0 {
            rels.push(MathRelation { name: name.into(), formula: format!("{:.6}×{}", inv_phi, mult),
                error_pct: err, tier: 2 });
        }
    }
    // e relationship
    let e = std::f64::consts::E;
    let err_e = ((v - e * 100.0) / (e * 100.0) * 100.0).abs();
    if err_e < 2.0 {
        rels.push(MathRelation { name: "e×100".into(), formula: format!("{:.4}×100", e),
            error_pct: err_e, tier: 2 });
    }
    // Perfect number
    if is_perfect(value) {
        rels.push(MathRelation { name: "perfect_number".into(), formula: format!("{value}=Σdivisors"),
            error_pct: 0.0, tier: 1 });
    }
    // Prime
    if is_prime(value) {
        rels.push(MathRelation { name: "prime".into(), formula: format!("{value} is prime"),
            error_pct: 0.0, tier: 2 });
    }
    // Triangular
    let n_tri = ((((8 * value + 1) as f64).sqrt() - 1.0) / 2.0).round() as u32;
    if n_tri * (n_tri + 1) / 2 == value {
        rels.push(MathRelation { name: "triangular".into(), formula: format!("T({n_tri})"),
            error_pct: 0.0, tier: 2 });
    }

    rels
}

fn is_prime(n: u32) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

fn is_perfect(n: u32) -> bool {
    if n < 2 { return false; }
    let mut sum = 1u32;
    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            sum += i;
            if i != n / i { sum += n / i; }
        }
        i += 1;
    }
    sum == n
}

// ─── GematriaEngine ──────────────────────────────────────────

pub struct GematriaEngine {
    standard_map: HashMap<char, u32>,
    no_sofit_map: HashMap<char, u32>,
    ordinal_map: HashMap<char, u32>,
    atbash_map: HashMap<char, char>,
    milui_map: HashMap<char, u32>,
    // Reverse lookup: value → [words]
    reverse: HashMap<u32, Vec<KnownValue>>,
    known: Vec<KnownValue>,
}

impl GematriaEngine {
    pub fn new() -> Self {
        let standard_map: HashMap<char, u32> = STANDARD.iter().cloned().collect();
        let no_sofit_map: HashMap<char, u32> = STANDARD_NO_SOFIT.iter().cloned().collect();
        let ordinal_map: HashMap<char, u32> = ORDINAL.iter().cloned().collect();
        let atbash_map: HashMap<char, char> = ATBASH_PAIRS.iter().cloned().collect();
        let milui_map: HashMap<char, u32> = MILUI.iter().map(|(c, _, v)| (*c, *v)).collect();

        let known = preload_known();
        let mut reverse: HashMap<u32, Vec<KnownValue>> = HashMap::new();
        for kv in &known {
            reverse.entry(kv.value).or_default().push(kv.clone());
        }

        Self { standard_map, no_sofit_map, ordinal_map, atbash_map, milui_map, reverse, known }
    }

    // ── Calculation methods ──

    /// Standard gematria — traditional (א=1..ת=400, sofit treated as regular: ך=20, ם=40 etc.)
    /// This is the most common Kabbalistic calculation used in all sources.
    /// Use standard_with_sofit() for the extended sofit values (ך=500..ץ=900).
    pub fn standard(&self, text: &str) -> u32 {
        self.compute(text, &self.no_sofit_map)
    }

    /// Standard with sofit distinction (ך=500, ם=600, ן=700, ף=800, ץ=900)
    pub fn standard_with_sofit(&self, text: &str) -> u32 {
        self.compute(text, &self.standard_map)
    }

    /// Alias — kept for compatibility
    pub fn standard_no_sofit(&self, text: &str) -> u32 {
        self.compute(text, &self.no_sofit_map)
    }

    /// Ordinal (א=1, ב=2, ... ת=22)
    pub fn ordinal(&self, text: &str) -> u32 {
        self.compute(text, &self.ordinal_map)
    }

    /// Reduced / Katan (cyclical single digit: א=1..ט=9, י=1..צ=9, ק=1..ץ=9)
    pub fn reduced(&self, text: &str) -> u32 {
        let std_val = self.standard(text);  // standard already uses no-sofit
        digital_root(std_val)
    }

    /// Atbash transform (א↔ת, ב↔ש, ...)
    pub fn atbash(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        // SAFE: iterating by char, not byte index
        for ch in text.chars() {
            // normalize sofit first
            let base = sofit_to_regular(ch);
            if let Some(&replacement) = self.atbash_map.get(&base) {
                result.push(replacement);
            } else {
                result.push(ch);
            }
        }
        result
    }

    /// Milui (spelled-out letter values: א→אלף=111, ב→בית=412, ...)
    pub fn milui(&self, text: &str) -> u32 {
        let mut total = 0u32;
        for ch in text.chars() {
            let base = sofit_to_regular(ch);
            if let Some(&val) = self.milui_map.get(&base) {
                total += val;
            }
        }
        total
    }

    /// Kolel (standard + number of letters)
    pub fn kolel(&self, text: &str) -> u32 {
        let std = self.standard(text);
        let letter_count = text.chars().filter(|c| self.standard_map.contains_key(c)
            || self.standard_map.contains_key(&sofit_to_regular(*c))).count() as u32;
        std + letter_count
    }

    // ── Lookup ──

    /// Reverse lookup: find all known words with this gematria value
    pub fn reverse_lookup(&self, value: u32) -> Vec<&KnownValue> {
        self.reverse.get(&value).map(|v| v.iter().collect()).unwrap_or_default()
    }

    /// Find all math relations for a value
    pub fn find_relations(&self, value: u32) -> Vec<MathRelation> {
        find_math_relations(value)
    }

    /// Compute gematria for each word in text, return (word, value) pairs
    pub fn annotate_words(&self, text: &str) -> Vec<(String, u32)> {
        let mut results = Vec::new();
        for word in text.split_whitespace() {
            let clean: String = word.chars().filter(|c| is_hebrew_letter(*c)).collect();
            if !clean.is_empty() {
                let val = self.standard(&clean);
                if val > 0 {
                    results.push((clean, val));
                }
            }
        }
        results
    }

    /// Full analysis: compute value + find matches + find math relations
    pub fn analyze(&self, text: &str) -> GematriaAnalysis {
        let value = self.standard(text);
        let ordinal = self.ordinal(text);
        let reduced = self.reduced(text);
        let matches = self.reverse_lookup(value).iter().map(|kv| kv.word.clone()).collect();
        let relations = self.find_relations(value);
        GematriaAnalysis { text: text.to_string(), value, ordinal, reduced, matches, relations }
    }

    // ── Internal ──

    fn compute(&self, text: &str, map: &HashMap<char, u32>) -> u32 {
        let mut total = 0u32;
        for ch in text.chars() {
            if let Some(&val) = map.get(&ch) {
                total += val;
            } else {
                let base = sofit_to_regular(ch);
                if let Some(&val) = map.get(&base) {
                    total += val;
                }
            }
        }
        total
    }
}

#[derive(Debug, Clone)]
pub struct GematriaAnalysis {
    pub text: String,
    pub value: u32,
    pub ordinal: u32,
    pub reduced: u32,
    pub matches: Vec<String>,     // words with same standard value
    pub relations: Vec<MathRelation>,
}

impl GematriaAnalysis {
    /// JSON output
    pub fn to_json(&self) -> String {
        let matches_json: Vec<String> = self.matches.iter()
            .filter(|w| **w != self.text)
            .map(|w| format!("\"{}\"", w))
            .collect();
        let rels_json: Vec<String> = self.relations.iter()
            .filter(|r| r.tier <= 2)
            .map(|r| format!("{{\"name\":\"{}\",\"error_pct\":{:.2},\"tier\":{}}}", r.name, r.error_pct, r.tier))
            .collect();
        format!(
            "{{\"text\":\"{}\",\"value\":{},\"ordinal\":{},\"reduced\":{},\"matches\":[{}],\"relations\":[{}]}}",
            self.text, self.value, self.ordinal, self.reduced,
            matches_json.join(","),
            rels_json.join(",")
        )
    }

    /// Format as Hebrew response string
    pub fn format_he(&self) -> String {
        let mut parts = vec![
            format!("הגימטריה של \"{}\" = {}", self.text, self.value),
            format!("סדורי: {}, קטן: {}", self.ordinal, self.reduced),
        ];
        if !self.matches.is_empty() {
            let m: Vec<_> = self.matches.iter().filter(|w| **w != self.text).cloned().collect();
            if !m.is_empty() {
                parts.push(format!("שווה ערך ל: {}", m.join(", ")));
            }
        }
        for rel in &self.relations {
            if rel.tier <= 2 {
                parts.push(format!("{} (שגיאה: {:.2}%)", rel.name, rel.error_pct));
            }
        }
        parts.join(". ")
    }
}

// ─── Utility ─────────────────────────────────────────────────

fn sofit_to_regular(ch: char) -> char {
    match ch {
        'ך' => 'כ', 'ם' => 'מ', 'ן' => 'נ', 'ף' => 'פ', 'ץ' => 'צ',
        _ => ch,
    }
}

fn is_hebrew_letter(ch: char) -> bool {
    ('\u{05D0}'..='\u{05EA}').contains(&ch) || // א-ת
    matches!(ch, 'ך' | 'ם' | 'ן' | 'ף' | 'ץ')
}

fn digital_root(mut n: u32) -> u32 {
    while n > 9 {
        let mut sum = 0u32;
        while n > 0 {
            sum += n % 10;
            n /= 10;
        }
        n = sum;
    }
    n
}

// ─── Tests ───────────────────────────────────────────────────


/// Standalone gematria computation - no engine needed.
/// Use this from other modules instead of duplicating the table.
pub fn compute_standard(text: &str) -> u32 {
    const VALS: &[(char, u32)] = &[
        ('א',1),('ב',2),('ג',3),('ד',4),('ה',5),('ו',6),('ז',7),('ח',8),('ט',9),('י',10),
        ('כ',20),('ל',30),('מ',40),('נ',50),('ס',60),('ע',70),('פ',80),('צ',90),('ק',100),
        ('ר',200),('ש',300),('ת',400),
        ('ך',20),('ם',40),('ן',50),('ף',80),('ץ',90),
    ];
    text.chars().map(|c| VALS.iter().find(|&&(ch,_)| ch==c).map(|&(_,v)| v).unwrap_or(0)).sum()
}


// ═══════════════════════════════════════════════════════
// Standalone lightweight gematria — no Engine needed
// Used by dream.rs and other hot-path code
// ═══════════════════════════════════════════════════════

/// Fast standard gematria without Engine allocation.
/// Use this in hot paths (dream loops, spread activation).
/// For full features (reverse lookup, milui, atbash) use GematriaEngine.
pub fn quick_standard(text: &str) -> u32 {
    text.chars().map(|c| match c {
        'א' => 1, 'ב' => 2, 'ג' => 3, 'ד' => 4, 'ה' => 5,
        'ו' => 6, 'ז' => 7, 'ח' => 8, 'ט' => 9, 'י' => 10,
        'כ' | 'ך' => 20, 'ל' => 30, 'מ' | 'ם' => 40,
        'נ' | 'ן' => 50, 'ס' => 60, 'ע' => 70,
        'פ' | 'ף' => 80, 'צ' | 'ץ' => 90,
        'ק' => 100, 'ר' => 200, 'ש' => 300, 'ת' => 400,
        _ => 0,
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> GematriaEngine { GematriaEngine::new() }

    #[test]
    fn test_standard_basic() {
        let g = engine();
        assert_eq!(g.standard("אמת"), 441);
        assert_eq!(g.standard("יהוה"), 26);
        assert_eq!(g.standard("אלהים"), 86);
        assert_eq!(g.standard("חכמה"), 73);
        assert_eq!(g.standard("חסד"), 72);
        assert_eq!(g.standard("גבורה"), 216);
        assert_eq!(g.standard("מלכות"), 496);
        assert_eq!(g.standard("מטטרון"), 314);
        assert_eq!(g.standard("שדי"), 314);
    }

    #[test]
    fn test_standard_sofit() {
        let g = engine();
        // with sofit: ך=500, ם=600, ן=700, ף=800, ץ=900
        assert_eq!(g.standard_with_sofit("ך"), 500);
        assert_eq!(g.standard_no_sofit("ך"), 20);
        assert_eq!(g.standard("ך"), 20); // standard = no_sofit by default
    }

    #[test]
    fn test_ordinal() {
        let g = engine();
        assert_eq!(g.ordinal("אמת"), 1 + 13 + 22); // 36
        assert_eq!(g.ordinal("אבגד"), 1 + 2 + 3 + 4); // 10
    }

    #[test]
    fn test_reduced() {
        let g = engine();
        // אמת standard_no_sofit = 1+40+400 = 441 → 4+4+1 = 9
        assert_eq!(g.reduced("אמת"), 9);
    }

    #[test]
    fn test_atbash() {
        let g = engine();
        assert_eq!(g.atbash("א"), "ת");
        assert_eq!(g.atbash("אמת"), "תיא");
    }

    #[test]
    fn test_milui() {
        let g = engine();
        // א=111(אלף) + מ=80(מם) + ת=406(תו) = 597
        assert_eq!(g.milui("אמת"), 597);
    }

    #[test]
    fn test_kolel() {
        let g = engine();
        // אמת = 441 + 3 letters = 444
        assert_eq!(g.kolel("אמת"), 444);
    }

    #[test]
    fn test_reverse_lookup() {
        let g = engine();
        let matches = g.reverse_lookup(314);
        let words: Vec<_> = matches.iter().map(|m| m.word.as_str()).collect();
        assert!(words.contains(&"מטטרון"));
        assert!(words.contains(&"שדי"));
    }

    #[test]
    fn test_math_relations_314() {
        let rels = find_math_relations(314);
        let names: Vec<_> = rels.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"π×100"), "314 should match π×100 (err<0.1%)");
        // 314 = 2×157 — NOT prime. 157 is prime but 314 itself is not.
        assert!(!names.contains(&"prime"), "314 is composite, should not be marked prime");
    }

    #[test]
    fn test_perfect_496() {
        let rels = find_math_relations(496);
        let names: Vec<_> = rels.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"perfect_number"), "496 is a perfect number");
    }

    #[test]
    fn test_prime_73() {
        let rels = find_math_relations(73);
        assert!(rels.iter().any(|r| r.name == "prime"));
    }

    #[test]
    fn test_annotate_words() {
        let g = engine();
        let words = g.annotate_words("בראשית ברא אלהים");
        assert_eq!(words.len(), 3);
        assert_eq!(words[2].0, "אלהים");
        assert_eq!(words[2].1, 86);
    }

    #[test]
    fn test_analyze() {
        let g = engine();
        let a = g.analyze("חכמה");
        assert_eq!(a.value, 73);
        assert!(a.relations.iter().any(|r| r.name == "prime"));
    }

    #[test]
    fn test_format_he() {
        let g = engine();
        let a = g.analyze("מטטרון");
        let formatted = a.format_he();
        assert!(formatted.contains("314"));
        assert!(formatted.contains("π"));
    }

    #[test]
    fn test_empty_string() {
        let g = engine();
        assert_eq!(g.standard(""), 0);
        assert_eq!(g.ordinal(""), 0);
    }

    #[test]
    fn test_mixed_hebrew_other() {
        let g = engine();
        // Only Hebrew letters counted
        assert_eq!(g.standard("אמת 123 emet"), 441);
    }

    #[test]
    fn test_sefirot_values() {
        let g = engine();
        // All 10 sefirot values verified
        assert_eq!(g.standard("כתר"), 620);
        assert_eq!(g.standard("בינה"), 67);
        assert_eq!(g.standard("דעת"), 474);
        assert_eq!(g.standard("תפארת"), 1081);
        assert_eq!(g.standard("נצח"), 148);
        assert_eq!(g.standard("הוד"), 15);
        assert_eq!(g.standard("יסוד"), 80);
    }

    #[test]
    fn test_angel_values() {
        let g = engine();
        assert_eq!(g.standard("גבריאל"), 246);
        assert_eq!(g.standard("מיכאל"), 101);
        assert_eq!(g.standard("רפאל"), 311);
        assert_eq!(g.standard("סנדלפון"), 280);
        assert_eq!(g.standard("סמאל"), 131);
        assert_eq!(g.standard("דומיאל"), 91);
    }
}
