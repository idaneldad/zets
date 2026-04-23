// ═══════════════════════════════════════════════════════════════
// torah.rs — מנהל ידע תורני | DINIO Cortex V7.11.0
// ═══════════════════════════════════════════════════════════════
// זיהוי שאלות תורניות, routing, ומתן metadata על מקורות.
// לא מחליף את ה-pipeline — משלים אותו. כל הידע בפועל = expressions.
//
// תפקידים:
// 1. detect_torah_query — האם השאלה תורנית?
// 2. classify_torah_domain — מהו התחום הספציפי?
// 3. enrich_answer — הוספת metadata (מקור, פרשן, גימטריה)
// 4. torah_keywords — מילות מפתח לחיפוש
// ═══════════════════════════════════════════════════════════════

use crate::wisdom_engines::gematria::GematriaEngine;

// ─── Torah sub-domains ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TorahSubDomain {
    Mikra,       // תנ"ך — פסוקים
    Parshanut,   // פרשנות — רש"י, רמב"ן, אב"ע
    Gematria,    // חישובים מספריים
    Kabbalah,    // ספירות, זוהר, עץ חיים
    Hekhalot,    // היכלות, מרכבה, סודות
    Halakha,     // הלכה (עתידי)
    General,     // כללי — לא ספציפי
}

impl TorahSubDomain {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mikra => "mikra",
            Self::Parshanut => "parshanut",
            Self::Gematria => "gematria",
            Self::Kabbalah => "kabbalah",
            Self::Hekhalot => "hekhalot",
            Self::Halakha => "halakha",
            Self::General => "torah",
        }
    }
}

// ─── Book references ─────────────────────────────────────────

const TANAKH_BOOKS_HE: [&str; 38] = [
    "בראשית", "שמות", "ויקרא", "במדבר", "דברים",
    "יהושע", "שופטים", "שמואל", "מלכים",
    "ישעיהו", "ירמיהו", "יחזקאל",
    "הושע", "יואל", "עמוס", "עובדיה", "יונה", "מיכה",
    "נחום", "חבקוק", "צפניה", "חגי", "זכריה", "מלאכי",
    "תהלים", "משלי", "איוב", "שיר השירים", "רות",
    "איכה", "קהלת", "אסתר", "דניאל", "עזרא", "נחמיה",
    "דברי הימים", "תהילים", "קוהלת",
];

const PARSHANIM_NAMES: [&str; 12] = [
    "רש\"י", "רשי", "רמב\"ן", "רמבן", "אבן עזרא", "אב\"ע",
    "ספורנו", "אור החיים", "בעל הטורים", "רשב\"ם", "רד\"ק", "רמב\"ם",
];

const PARSHIOT: [&str; 10] = [
    "בראשית", "נח", "לך לך", "וירא", "חיי שרה",
    "תולדות", "ויצא", "וישלח", "וישב", "מקץ",
    // ... truncated for size, full 54 in production
];

// Torah/Kabbalah detection keywords
const TORAH_KEYWORDS: [&str; 70] = [
    // תנ"ך
    "תורה", "תנך", "תנ\"ך", "פסוק", "פרק", "בראשית", "שמות", "ויקרא", "במדבר", "דברים",
    "נביאים", "כתובים", "תהלים", "משלי", "איוב",
    // פרשנות
    "רש\"י", "רשי", "רמב\"ן", "רמבן", "אבן עזרא", "פירוש", "פרשנות", "מדרש",
    // קבלה
    "קבלה", "ספירות", "ספירה", "זוהר", "תניא", "עץ חיים", "ספר יצירה",
    "כתר", "חכמה", "בינה", "חסד", "גבורה", "תפארת", "נצח", "הוד", "יסוד", "מלכות",
    // גימטריה
    "גימטריה", "גימטריא", "ערך מספרי",
    // היכלות
    "היכלות", "מרכבה", "מטטרון", "סנדלפון",
    // כללי
    "שבת", "תפילה", "מצווה", "הלכה", "תלמוד", "משנה", "גמרא",
    "פרשה", "פרשת", "הפטרה", "אלוהים", "השם", "הקדוש ברוך הוא",
    // English
    "torah", "bible", "talmud", "kabbalah", "gematria", "sefirot",
    "zohar", "tanya", "midrash", "rashi",
];

const GEMATRIA_KEYWORDS: [&str; 8] = [
    "גימטריה", "גימטריא", "ערך מספרי", "שווה ל", "בגימטריא",
    "gematria", "numerical value", "הגימטריה של",
];

const PARSHANUT_KEYWORDS: [&str; 12] = [
    "רש\"י", "רשי", "רמב\"ן", "רמבן", "אבן עזרא", "ספורנו",
    "אור החיים", "בעל הטורים", "פירוש", "מפרש", "פרשנות", "מה אומר",
];

const KABBALAH_KEYWORDS: [&str; 16] = [
    "קבלה", "ספירות", "ספירה", "זוהר", "תניא", "עץ חיים",
    "ספר יצירה", "אור ישר", "אור חוזר", "צמצום", "שבירת הכלים",
    "פרצוף", "עולם האצילות", "אין סוף", "klipot", "sefira",
];

const HEKHALOT_KEYWORDS: [&str; 10] = [
    "היכלות", "מרכבה", "מטטרון", "שר הפנים", "יורדי מרכבה",
    "כסא הכבוד", "חיות הקודש", "שרפים", "אופנים", "רקיע",
];

// ─── Detection functions ─────────────────────────────────────

/// Is this query about Torah / Kabbalah?
/// Returns confidence 0.0-1.0
pub fn detect_torah_query(text: &str) -> f32 {
    let lower = text.to_lowercase();
    let mut score = 0.0f32;

    // Direct keyword hits
    for kw in &TORAH_KEYWORDS {
        if lower.contains(kw) {
            score += 0.4;
            break; // One hit is enough for base
        }
    }

    // Book name reference
    for book in &TANAKH_BOOKS_HE {
        if text.contains(book) {
            score += 0.5;
            break;
        }
    }

    // Parshanim reference
    for name in &PARSHANIM_NAMES {
        if text.contains(name) {
            score += 0.5;
            break;
        }
    }

    // Chapter:verse pattern (e.g. "1:1", "פרק 1 פסוק 2")
    if text.contains(':') && text.chars().any(|c| c.is_ascii_digit()) {
        if text.contains("פרק") || text.contains("פסוק") {
            score += 0.3;
        }
    }

    // "מה כתוב ב" pattern
    if text.contains("מה כתוב") || text.contains("מה אומר") {
        score += 0.2;
    }

    score.min(1.0)
}

/// Classify into specific sub-domain
pub fn classify_subdomain(text: &str) -> TorahSubDomain {
    let lower = text.to_lowercase();

    // Gematria first (most specific)
    for kw in &GEMATRIA_KEYWORDS {
        if lower.contains(kw) { return TorahSubDomain::Gematria; }
    }

    // Hekhalot
    for kw in &HEKHALOT_KEYWORDS {
        if lower.contains(kw) { return TorahSubDomain::Hekhalot; }
    }

    // Kabbalah
    for kw in &KABBALAH_KEYWORDS {
        if lower.contains(kw) { return TorahSubDomain::Kabbalah; }
    }

    // Parshanut (check before Mikra — "רש"י על בראשית" = parshanut, not mikra)
    for kw in &PARSHANUT_KEYWORDS {
        if text.contains(kw) { return TorahSubDomain::Parshanut; }
    }

    // Mikra (direct book reference)
    for book in &TANAKH_BOOKS_HE {
        if text.contains(book) { return TorahSubDomain::Mikra; }
    }

    TorahSubDomain::General
}

/// Build gematria response if the query is a gematria question
pub fn handle_gematria_query(text: &str, gematria: &GematriaEngine) -> Option<String> {
    let lower = text.to_lowercase();
    let is_gematria = GEMATRIA_KEYWORDS.iter().any(|kw| lower.contains(kw));
    if !is_gematria {
        return None;
    }

    // Extract the word to analyze
    // Patterns: "הגימטריה של X", "מה הגימטריה של X", "X בגימטריא"
    let word = extract_gematria_target(text)?;
    let analysis = gematria.analyze(&word);

    Some(analysis.format_he())
}

/// Extract the target word from a gematria query
fn extract_gematria_target(text: &str) -> Option<String> {
    // "הגימטריה של WORD"
    if let Some(pos) = text.find("של ") {
        let after = &text[pos + "של ".len()..];
        let word: String = after.chars()
            .take_while(|c| !c.is_whitespace() || *c == ' ')
            .take_while(|c| {
                ('\u{05D0}'..='\u{05EA}').contains(c) ||
                matches!(*c, 'ך' | 'ם' | 'ן' | 'ף' | 'ץ') ||
                *c == ' ' || *c == '"'
            })
            .collect();
        let trimmed = word.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    // Fallback: find any Hebrew word cluster in the query that isn't a keyword
    let words: Vec<&str> = text.split_whitespace().collect();
    for w in words.iter().rev() { // prefer last word
        let clean: String = w.chars()
            .filter(|c| ('\u{05D0}'..='\u{05EA}').contains(c) ||
                        matches!(*c, 'ך' | 'ם' | 'ן' | 'ף' | 'ץ'))
            .collect();
        if clean.len() > 2 && !GEMATRIA_KEYWORDS.iter().any(|kw| clean.contains(kw)) {
            return Some(clean);
        }
    }
    None
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_torah_query() {
        assert!(detect_torah_query("מה כתוב בבראשית פרק א") > 0.5);
        assert!(detect_torah_query("מה רש\"י אומר על בראשית א:א") > 0.5);
        assert!(detect_torah_query("מה הגימטריה של חכמה") > 0.3);
        assert!(detect_torah_query("מה הספירות בקבלה") > 0.3);
        assert!(detect_torah_query("מה מזג האוויר היום") < 0.1);
        assert!(detect_torah_query("how to code in python") < 0.1);
    }

    #[test]
    fn test_classify_subdomain() {
        assert_eq!(classify_subdomain("מה הגימטריה של אמת"), TorahSubDomain::Gematria);
        assert_eq!(classify_subdomain("מה רש\"י אומר"), TorahSubDomain::Parshanut);
        assert_eq!(classify_subdomain("מה כתוב בבראשית"), TorahSubDomain::Mikra);
        assert_eq!(classify_subdomain("מה כתוב בזוהר"), TorahSubDomain::Kabbalah);
        assert_eq!(classify_subdomain("היכלות רבתי"), TorahSubDomain::Hekhalot);
    }

    #[test]
    fn test_handle_gematria() {
        let g = GematriaEngine::new();
        let result = handle_gematria_query("מה הגימטריה של חכמה", &g);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("73"));
    }

    #[test]
    fn test_non_torah_query() {
        assert!(detect_torah_query("כמה עולה פיצה") < 0.1);
        assert!(detect_torah_query("מה השעה") < 0.1);
    }

    #[test]
    fn test_gematria_target_extraction() {
        assert_eq!(extract_gematria_target("מה הגימטריה של אמת"), Some("אמת".into()));
        assert_eq!(extract_gematria_target("הגימטריה של מטטרון"), Some("מטטרון".into()));
    }

    #[test]
    fn test_subdomain_priority() {
        // Parshanut should beat Mikra when parshanim are mentioned
        assert_eq!(classify_subdomain("רש\"י על בראשית"), TorahSubDomain::Parshanut);
        // Kabbalah should beat general
        assert_eq!(classify_subdomain("מה זה ספר יצירה"), TorahSubDomain::Kabbalah);
    }
}
