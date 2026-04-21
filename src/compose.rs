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
        text: render_entry(lang, surface, entry, style),
        citations: build_citations(entry),
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
            text: render_entry(&lang, surface, e, Style::Standard),
            citations: build_citations(e),
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
        parts.push(opener(lang, surface, pos));
    }
    if let Some(def) = e.definitions.first() {
        parts.push(def.clone());
    }
    if !e.synonyms.is_empty() {
        let list: Vec<_> = e.synonyms.iter().take(3).cloned().collect();
        parts.push(synonym_phrase(lang, &list));
    }
    if !e.antonyms.is_empty() {
        let list: Vec<_> = e.antonyms.iter().take(2).cloned().collect();
        parts.push(antonym_phrase(lang, &list));
    }
    parts.join(" ")
}

fn render_rich(lang: &str, surface: &str, e: &LexEntry) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(pos) = &e.pos {
        parts.push(opener(lang, surface, pos));
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
    if !e.synonyms.is_empty() {
        let list: Vec<_> = e.synonyms.iter().take(5).cloned().collect();
        parts.push(synonym_phrase(lang, &list));
    }
    if !e.antonyms.is_empty() {
        let list: Vec<_> = e.antonyms.iter().take(3).cloned().collect();
        parts.push(antonym_phrase(lang, &list));
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
