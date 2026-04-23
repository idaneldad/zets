//! graph_v4::morphology — rule-based lemma extraction.
//!
//! עברית: prefix/suffix stripping לפי הטבלאות ב-data/hebrew/{prefixes,suffixes}.tsv
//! אנגלית: stemming בסיסי (s, es, ed, ing, er)
//!
//! העיקרון:
//!   word "הבית" → strip prefix "ה" → "בית" (lemma candidate)
//!   אם "בית" קיים ב-vocab → return Some("בית"), else return None
//!
//! זה מבטיח שאנחנו לא יוצרים "הלכ" או "הבית" fake lemmas — רק ה-lemma 
//! החוזר אם הוא מילה אמיתית ב-corpus.

use std::collections::HashSet;

/// Hebrew prefixes — longest-first לstripping.
/// טעון מ-/home/dinio/zets/data/hebrew/prefixes.tsv.
pub const HEBREW_PREFIXES: &[&str] = &[
    // 3-letter compounds
    "וכש", "שכש", "וכל",
    // 2-letter compounds
    "וה", "ומ", "ול", "וב", "וכ", "וש",
    "שה", "שב", "שכ", "שמ", "של",
    "כה", "לה", "מה", "בה",
    // 1-letter (מש"ה וכל"ב)
    "ו", "ה", "ש", "ב", "כ", "ל", "מ",
];

/// Hebrew suffixes — longest-first.
pub const HEBREW_SUFFIXES: &[&str] = &[
    // 3-letter (possessive plural)
    "יכם", "יכן", "יהם", "יהן",
    // 2-letter
    "ים", "ות", "תי", "תם", "תן", "נו", "כם", "כן", "הם", "הן",
    "ית", "יה", "כי",
    // 1-letter
    "י", "ה", "ך",
];

/// בודק אם `word` מתחיל ב-prefix שני; מחזיר Some(rest) אם כן.
fn strip_prefix_he<'a>(word: &'a str) -> Option<(&'a str, &'a str)> {
    for p in HEBREW_PREFIXES {
        if word.starts_with(p) && word.chars().count() > p.chars().count() + 1 {
            return Some((p, &word[p.len()..]));
        }
    }
    None
}

fn strip_suffix_he<'a>(word: &'a str) -> Option<(&'a str, &'a str)> {
    for s in HEBREW_SUFFIXES {
        if word.ends_with(s) && word.chars().count() > s.chars().count() + 1 {
            let cut = word.len() - s.len();
            return Some((&word[..cut], s));
        }
    }
    None
}

/// Hebrew detection — האם string מכיל לפחות ch עברית אחת?
fn is_hebrew(word: &str) -> bool {
    word.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c))
}

/// עברית: מחזיר candidate lemmas (max 3):
///   - strip prefix → lemma
///   - strip suffix → lemma
///   - strip both
pub fn hebrew_candidates(word: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some((_, rest)) = strip_prefix_he(word) {
        out.push(rest.to_string());
    }
    if let Some((stem, _)) = strip_suffix_he(word) {
        out.push(stem.to_string());
    }
    // prefix + suffix
    if let Some((_, after_pref)) = strip_prefix_he(word) {
        if let Some((stem, _)) = strip_suffix_he(after_pref) {
            out.push(stem.to_string());
        }
    }
    // עיצורים ת → ה (יחידת → יחידה) — ל-feminine construct
    if word.ends_with("ת") && word.chars().count() > 2 {
        let mut stem: String = word.chars().take(word.chars().count() - 1).collect();
        stem.push('ה');
        out.push(stem);
    }
    out
}

/// אנגלית: פשוט stripping.
pub fn english_candidates(word: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = word.to_lowercase();
    // plural s/es
    if lower.ends_with("ies") && lower.len() > 4 {
        let stem = &lower[..lower.len()-3];
        out.push(format!("{}y", stem));  // cities → city
    }
    if lower.ends_with("es") && lower.len() > 3 {
        out.push(lower[..lower.len()-2].to_string());
    }
    if lower.ends_with("s") && lower.len() > 2 && !lower.ends_with("ss") {
        out.push(lower[..lower.len()-1].to_string());
    }
    // -ing → stem (running → runn → OR run)
    if lower.ends_with("ing") && lower.len() > 4 {
        let stem = &lower[..lower.len()-3];
        out.push(stem.to_string());                       // run→run
        // doubled consonant: running → run
        let chars: Vec<char> = stem.chars().collect();
        if chars.len() >= 2 && chars[chars.len()-1] == chars[chars.len()-2] {
            let shortened: String = chars[..chars.len()-1].iter().collect();
            out.push(shortened);
        }
    }
    // -ed
    if lower.ends_with("ed") && lower.len() > 3 {
        out.push(lower[..lower.len()-2].to_string());
        if lower.ends_with("ied") {
            let stem = &lower[..lower.len()-3];
            out.push(format!("{}y", stem));  // tried → try
        }
    }
    // -er / -est  (faster → fast)
    if lower.ends_with("er") && lower.len() > 3 {
        out.push(lower[..lower.len()-2].to_string());
    }
    if lower.ends_with("est") && lower.len() > 4 {
        out.push(lower[..lower.len()-3].to_string());
    }
    // -ly (quickly → quick)
    if lower.ends_with("ly") && lower.len() > 3 {
        out.push(lower[..lower.len()-2].to_string());
    }
    out
}

/// ה-API הראשי — בהינתן word, החזר candidates כל ה-lemmas האפשריים.
pub fn candidates(word: &str) -> Vec<String> {
    if is_hebrew(word) {
        hebrew_candidates(word)
    } else {
        english_candidates(word)
    }
}

/// מיפוי surface → lemma אחרי סינון כנגד vocabulary מוכר.
/// רק אם ה-candidate קיים ב-vocab, נשמר. כך נמנעים fake-lemmas.
pub fn resolve_lemma(word: &str, vocab: &HashSet<String>) -> Option<String> {
    if vocab.contains(word) {
        // המילה עצמה lemma אם היא קצרה/מופיעה ישירות? לא — רק אם אין candidate טוב
    }
    for cand in candidates(word) {
        if vocab.contains(&cand) && cand != word {
            return Some(cand);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hebrew_prefix_strip() {
        assert_eq!(strip_prefix_he("הבית"), Some(("ה", "בית")));
        assert_eq!(strip_prefix_he("לבית"), Some(("ל", "בית")));
        assert_eq!(strip_prefix_he("מהבית"), Some(("מה", "בית")));
        assert_eq!(strip_prefix_he("ולבית"), Some(("ול", "בית")));
    }

    #[test]
    fn hebrew_suffix_strip() {
        let (stem, suf) = strip_suffix_he("בתים").unwrap();
        assert_eq!(stem, "בת");
        assert_eq!(suf, "ים");
    }

    #[test]
    fn hebrew_candidates_basic() {
        let c = hebrew_candidates("הבית");
        assert!(c.contains(&"בית".to_string()), "got {:?}", c);
    }

    #[test]
    fn hebrew_t_to_h_feminine() {
        let c = hebrew_candidates("יחידת");
        // 'יחידת' → strip 'ת' → 'יחיד' (wrong) OR cut 'ת' → 'יחיד' → append 'ה' → 'יחידה'
        assert!(c.contains(&"יחידה".to_string()), "got {:?}", c);
    }

    #[test]
    fn english_basic() {
        let c = english_candidates("running");
        assert!(c.contains(&"run".to_string()) || c.contains(&"runn".to_string()),
                "got {:?}", c);

        let c = english_candidates("democracies");
        assert!(c.contains(&"democracy".to_string()), "got {:?}", c);

        let c = english_candidates("cats");
        assert!(c.contains(&"cat".to_string()), "got {:?}", c);
    }

    #[test]
    fn resolve_with_vocab() {
        let mut vocab: HashSet<String> = HashSet::new();
        vocab.insert("בית".to_string());
        vocab.insert("run".to_string());

        assert_eq!(resolve_lemma("הבית", &vocab), Some("בית".to_string()));
        assert_eq!(resolve_lemma("לבית", &vocab), Some("בית".to_string()));
        assert_eq!(resolve_lemma("running", &vocab), Some("run".to_string()));
        assert_eq!(resolve_lemma("xyzabc", &vocab), None);
    }
}
