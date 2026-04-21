//! Deterministic natural-language composition.
//! No LLM. Pure templates over lexicon. Same input = same output, always.

use crate::lexicon::{LexEntry, Lexicon};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Brief,
    Standard,
    Rich,
}

#[derive(Debug, Clone)]
pub struct Composition {
    pub text: String,
    pub citations: Vec<String>,
    pub lang: String,
    pub surface: String,
    pub found: bool,
}

/// Filter out empty/whitespace/junk entries from a list of strings.
fn clean_list(items: &[String], take: usize) -> Vec<String> {
    items
        .iter()
        .map(|s| {
            // Strip leftover template markers, commas, whitespace
            let t = s.trim()
                .trim_matches(|c: char| c == ',' || c == ';' || c == '.' || c.is_whitespace() || c == '|' || c == ':');
            // Remove inner multiple commas (from stripped templates like ",,,")
            let cleaned: String = t.chars().collect::<String>()
                .split(',')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join(", ");
            cleaned
        })
        .filter(|s| {
            !s.is_empty()
                && s.len() >= 2
                && s.len() <= 80
                && !s.starts_with("See ")
                && !s.starts_with("see ")
                && !s.starts_with("Thesaurus:")
                && s.chars().any(|c| c.is_alphanumeric())
        })
        .take(take)
        .collect()
}

/// Translate technical POS codes to a human-readable term in each language.
fn pos_label(lang: &str, raw_pos: &str) -> String {
    let canonical = match raw_pos {
        "noun" | "n" | "Noun" => "noun",
        "verb" | "v" | "Verb" => "verb",
        "adj" | "adjective" | "Adjective" => "adj",
        "adv" | "adverb" | "Adverb" => "adv",
        "interj" | "interjection" => "interj",
        "phrase" => "phrase",
        _ => "",
    };

    match (lang, canonical) {
        ("he", "noun") => "שם־עצם".to_string(),
        ("he", "verb") => "פועל".to_string(),
        ("he", "adj") => "שם־תואר".to_string(),
        ("he", "adv") => "תואר־הפועל".to_string(),
        ("he", "interj") => "מילת־קריאה".to_string(),
        ("he", "phrase") => "צירוף".to_string(),
        ("he", _) => raw_pos.to_string(),

        ("en", "noun") => "noun".to_string(),
        ("en", "verb") => "verb".to_string(),
        ("en", "adj") => "adjective".to_string(),
        ("en", "adv") => "adverb".to_string(),
        ("en", _) => raw_pos.to_string(),

        ("de", "noun") => "Substantiv".to_string(),
        ("de", "verb") => "Verb".to_string(),
        ("de", "adj") => "Adjektiv".to_string(),
        ("de", _) => raw_pos.to_string(),

        ("fr", "noun") => "nom".to_string(),
        ("fr", "verb") => "verbe".to_string(),
        ("fr", "adj") => "adjectif".to_string(),
        ("fr", _) => raw_pos.to_string(),

        ("es", "noun") => "sustantivo".to_string(),
        ("es", "verb") => "verbo".to_string(),
        ("es", "adj") => "adjetivo".to_string(),
        ("es", _) => raw_pos.to_string(),

        ("it", "noun") => "sostantivo".to_string(),
        ("it", "verb") => "verbo".to_string(),
        ("it", "adj") => "aggettivo".to_string(),
        ("it", _) => raw_pos.to_string(),

        ("ar", "noun") => "اسم".to_string(),
        ("ar", "verb") => "فعل".to_string(),
        ("ar", "adj") => "صفة".to_string(),
        ("ar", _) => raw_pos.to_string(),

        ("ru", "noun") => "существительное".to_string(),
        ("ru", "verb") => "глагол".to_string(),
        ("ru", "adj") => "прилагательное".to_string(),
        ("ru", _) => raw_pos.to_string(),

        ("nl", "noun") => "zelfstandig naamwoord".to_string(),
        ("nl", "verb") => "werkwoord".to_string(),
        ("nl", "adj") => "bijvoeglijk naamwoord".to_string(),
        ("nl", _) => raw_pos.to_string(),

        ("pt", "noun") => "substantivo".to_string(),
        ("pt", "verb") => "verbo".to_string(),
        ("pt", "adj") => "adjetivo".to_string(),
        ("pt", _) => raw_pos.to_string(),

        _ => raw_pos.to_string(),
    }
}

pub fn describe(lex: &Lexicon, lang: &str, surface: &str, style: Style) -> Composition {
    let entry = match lex.get(lang, surface) {
        Some(e) if !e.is_empty() => e,
        _ => {
            return Composition {
                text: format!("No information found for '{surface}' in language '{lang}'."),
                citations: vec![],
                lang: lang.to_string(),
                surface: surface.to_string(),
                found: false,
            }
        }
    };
    Composition {
        text: render_entry(lang, surface, &entry, style),
        citations: build_citations(&entry),
        lang: lang.to_string(),
        surface: surface.to_string(),
        found: true,
    }
}

pub fn cross_language(lex: &Lexicon, surface: &str) -> Vec<Composition> {
    lex.find_all_languages(surface)
        .into_iter()
        .filter(|(_, e)| !e.is_empty())
        .map(|(lang, e)| Composition {
            text: render_entry(&lang, surface, &e, Style::Standard),
            citations: build_citations(&e),
            lang,
            surface: surface.to_string(),
            found: true,
        })
        .collect()
}

fn render_entry(lang: &str, surface: &str, e: &LexEntry, style: Style) -> String {
    match style {
        Style::Brief => render_brief(surface, e),
        Style::Standard => render_standard(lang, surface, e),
        Style::Rich => render_rich(lang, surface, e),
    }
}

fn render_brief(surface: &str, e: &LexEntry) -> String {
    if let Some(def) = e.definitions.first() {
        format!("{surface}: {def}")
    } else if let Some(syn) = e.synonyms.first() {
        format!("{surface} ≈ {syn}")
    } else if let Some(pos) = &e.pos {
        format!("{surface} ({pos})")
    } else {
        surface.to_string()
    }
}

fn render_standard(lang: &str, surface: &str, e: &LexEntry) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(pos) = &e.pos {
        parts.push(opener(lang, surface, &pos_label(lang, pos)));
    }
    if let Some(def) = e.definitions.first() {
        parts.push(def.clone());
    }
    let syns = clean_list(&e.synonyms, 3);
    if !syns.is_empty() {
        parts.push(synonym_phrase(lang, &syns));
    }
    let ants = clean_list(&e.antonyms, 2);
    if !ants.is_empty() {
        parts.push(antonym_phrase(lang, &ants));
    }
    parts.join(" ")
}

fn render_rich(lang: &str, surface: &str, e: &LexEntry) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(pos) = &e.pos {
        parts.push(opener(lang, surface, &pos_label(lang, pos)));
    }
    if !e.definitions.is_empty() {
        if e.definitions.len() == 1 {
            parts.push(e.definitions[0].clone());
        } else {
            let numbered: Vec<String> = e
                .definitions
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, d)| format!("({}) {}", i + 1, d))
                .collect();
            parts.push(multi_sense_phrase(lang, e.definitions.len()) + " " + &numbered.join(" "));
        }
    }
    let syns = clean_list(&e.synonyms, 5);
    if !syns.is_empty() {
        parts.push(synonym_phrase(lang, &syns));
    }
    let ants = clean_list(&e.antonyms, 3);
    if !ants.is_empty() {
        parts.push(antonym_phrase(lang, &ants));
    }
    parts.join(" ")
}

fn opener(lang: &str, surface: &str, pos: &str) -> String {
    match lang {
        "he" => format!("{surface} הוא {pos}."),
        "en" => format!("{surface} is a {pos}."),
        "de" => format!("{surface} ist ein {pos}."),
        "fr" => format!("{surface} est un {pos}."),
        "es" => format!("{surface} es un {pos}."),
        "it" => format!("{surface} è un {pos}."),
        "ar" => format!("{surface}: {pos}."),
        "ru" => format!("{surface} — это {pos}."),
        "nl" => format!("{surface} is een {pos}."),
        "pt" => format!("{surface} é um {pos}."),
        _ => format!("{surface} ({pos})"),
    }
}

fn synonym_phrase(lang: &str, list: &[String]) -> String {
    let joined = list.join(", ");
    match lang {
        "he" => format!("מילים נרדפות: {}.", joined),
        "en" => format!("Synonyms: {}.", joined),
        "de" => format!("Synonyme: {}.", joined),
        "fr" => format!("Synonymes : {}.", joined),
        "es" => format!("Sinónimos: {}.", joined),
        "it" => format!("Sinonimi: {}.", joined),
        "ar" => format!("مرادفات: {}.", joined),
        "ru" => format!("Синонимы: {}.", joined),
        "nl" => format!("Synoniemen: {}.", joined),
        "pt" => format!("Sinônimos: {}.", joined),
        _ => format!("Synonyms: {}.", joined),
    }
}

fn antonym_phrase(lang: &str, list: &[String]) -> String {
    let joined = list.join(", ");
    match lang {
        "he" => format!("ניגודים: {}.", joined),
        "en" => format!("Antonyms: {}.", joined),
        "de" => format!("Antonyme: {}.", joined),
        "fr" => format!("Antonymes : {}.", joined),
        "es" => format!("Antónimos: {}.", joined),
        "it" => format!("Contrari: {}.", joined),
        "ar" => format!("نقائض: {}.", joined),
        "ru" => format!("Антонимы: {}.", joined),
        "nl" => format!("Antoniemen: {}.", joined),
        "pt" => format!("Antônimos: {}.", joined),
        _ => format!("Antonyms: {}.", joined),
    }
}

fn multi_sense_phrase(lang: &str, count: usize) -> String {
    match lang {
        "he" => format!("יש לו {} משמעויות:", count),
        "en" => format!("It has {} senses:", count),
        "de" => format!("Es hat {} Bedeutungen:", count),
        "fr" => format!("Il a {} sens :", count),
        "es" => format!("Tiene {} acepciones:", count),
        "it" => format!("Ha {} significati:", count),
        _ => format!("{} senses:", count),
    }
}

fn build_citations(e: &LexEntry) -> Vec<String> {
    let mut cites = Vec::new();
    if !e.definitions.is_empty() {
        cites.push(format!("Wiktionary (definitions × {})", e.definitions.len()));
    }
    if !e.synonyms.is_empty() {
        cites.push(format!("Wiktionary (synonyms × {})", e.synonyms.len()));
    }
    if !e.antonyms.is_empty() {
        cites.push(format!("Wiktionary (antonyms × {})", e.antonyms.len()));
    }
    if e.pos.is_some() {
        cites.push("Wiktionary (POS)".to_string());
    }
    cites
}
