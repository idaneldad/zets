//! # Motif — a short recurring pattern that can be a building block
//!
//! A motif is a small unit of structure that ZETS has seen repeatedly
//! and extracted. Examples:
//!   - Text: "once upon a time there was a {character}" — story opener
//!   - Melody: I-V-vi-IV chord progression — used in thousands of pop songs
//!   - Dialogue: "{a}: {greeting}\n{b}: {response}" — conversation opener
//!   - Prompt: "{subject} in {style}, {lighting}, {composition}" — image prompt
//!
//! Motifs are extracted by the `path_mining` module (already exists) and
//! stored here organized by KIND and DOMAIN.

use std::collections::HashMap;

/// What kind of content this motif produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotifKind {
    /// Short text template — sentences, lines.
    TextTemplate,
    /// Narrative beat — a plot step (introduction, conflict, reveal).
    NarrativeBeat,
    /// Dialogue exchange — 2+ turns.
    Dialogue,
    /// Chord progression or melodic phrase.
    MusicalPhrase,
    /// Image-generation prompt template.
    ImagePrompt,
    /// Code snippet pattern (loop, handler, etc).
    CodePattern,
    /// Argumentation structure (claim → evidence → conclusion).
    ArgumentPattern,
    /// Generic — any structured pattern.
    Generic,
}

impl MotifKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MotifKind::TextTemplate => "text_template",
            MotifKind::NarrativeBeat => "narrative_beat",
            MotifKind::Dialogue => "dialogue",
            MotifKind::MusicalPhrase => "musical_phrase",
            MotifKind::ImagePrompt => "image_prompt",
            MotifKind::CodePattern => "code_pattern",
            MotifKind::ArgumentPattern => "argument_pattern",
            MotifKind::Generic => "generic",
        }
    }
}

/// A single motif — a reusable pattern.
#[derive(Debug, Clone)]
pub struct Motif {
    pub id: String,
    pub kind: MotifKind,
    /// The template body. May contain {slots} for variable insertion.
    pub template: String,
    /// Slot names that need to be filled.
    pub slots: Vec<String>,
    /// Tags for retrieval (domain, mood, style).
    pub tags: Vec<String>,
    /// How often we've seen this motif in source data.
    pub observed_count: u32,
    /// How often we've USED it (for generation).
    pub used_count: u32,
    /// Domain: where this motif is appropriate ("fantasy", "legal", "casual", etc).
    pub domain: Option<String>,
    /// Mood/style tags ("light", "dark", "formal", "tense").
    pub style: Vec<String>,
}

impl Motif {
    pub fn new(
        id: impl Into<String>,
        kind: MotifKind,
        template: impl Into<String>,
    ) -> Self {
        let template = template.into();
        let slots = extract_slots(&template);
        Motif {
            id: id.into(),
            kind,
            template,
            slots,
            tags: Vec::new(),
            observed_count: 1,
            used_count: 0,
            domain: None,
            style: Vec::new(),
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_domain(mut self, d: impl Into<String>) -> Self {
        self.domain = Some(d.into());
        self
    }

    pub fn with_style(mut self, s: impl Into<String>) -> Self {
        self.style.push(s.into());
        self
    }

    /// Fill the template with given slot values.
    /// Returns None if any required slot is missing.
    pub fn fill(&self, values: &HashMap<String, String>) -> Option<String> {
        let mut out = self.template.clone();
        for slot in &self.slots {
            let placeholder = format!("{{{}}}", slot);
            let v = values.get(slot)?;
            out = out.replace(&placeholder, v);
        }
        Some(out)
    }

    /// Record a successful use.
    pub fn record_use(&mut self) {
        self.used_count += 1;
    }

    /// Does this motif match the requested style?
    pub fn matches_style(&self, style: &str) -> bool {
        self.style.iter().any(|s| s == style) || self.tags.iter().any(|t| t == style)
    }
}

fn extract_slots(template: &str) -> Vec<String> {
    let mut slots = Vec::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut slot = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                slot.push(ch);
            }
            if !slot.is_empty() && !slots.contains(&slot) {
                slots.push(slot);
            }
        }
    }
    slots
}

/// A bank of motifs, organized by kind + tag for fast retrieval.
#[derive(Debug, Default)]
pub struct MotifBank {
    motifs: HashMap<String, Motif>,
    by_kind: HashMap<MotifKind, Vec<String>>,
    by_tag: HashMap<String, Vec<String>>,
    by_domain: HashMap<String, Vec<String>>,
}

impl MotifBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, motif: Motif) {
        let id = motif.id.clone();
        self.by_kind
            .entry(motif.kind)
            .or_insert_with(Vec::new)
            .push(id.clone());
        for tag in &motif.tags {
            self.by_tag
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        if let Some(d) = motif.domain.clone() {
            self.by_domain.entry(d).or_insert_with(Vec::new).push(id.clone());
        }
        self.motifs.insert(id, motif);
    }

    pub fn get(&self, id: &str) -> Option<&Motif> {
        self.motifs.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Motif> {
        self.motifs.get_mut(id)
    }

    pub fn count(&self) -> usize {
        self.motifs.len()
    }

    /// Find motifs of a given kind.
    pub fn by_kind(&self, kind: MotifKind) -> Vec<&Motif> {
        self.by_kind
            .get(&kind)
            .map(|ids| ids.iter().filter_map(|id| self.motifs.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find motifs with a given tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&Motif> {
        self.by_tag
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.motifs.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find motifs for a specific domain.
    pub fn by_domain(&self, domain: &str) -> Vec<&Motif> {
        self.by_domain
            .get(domain)
            .map(|ids| ids.iter().filter_map(|id| self.motifs.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find the best matching motif for kind + style hints.
    /// Returns the most-used matching motif (proxy for "proven good").
    pub fn best_match(&self, kind: MotifKind, style_hint: &str) -> Option<&Motif> {
        let candidates = self.by_kind(kind);
        let styled: Vec<_> = candidates
            .iter()
            .filter(|m| m.matches_style(style_hint))
            .copied()
            .collect();
        let pool = if !styled.is_empty() { styled } else { candidates };

        pool.into_iter().max_by_key(|m| m.used_count.max(m.observed_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motif_slot_extraction() {
        let m = Motif::new("opener", MotifKind::TextTemplate, "Once upon a time, {hero} lived in {place}.");
        assert_eq!(m.slots, vec!["hero", "place"]);
    }

    #[test]
    fn test_motif_fill() {
        let m = Motif::new(
            "opener",
            MotifKind::TextTemplate,
            "Once upon a time, {hero} lived in {place}.",
        );

        let mut vals = HashMap::new();
        vals.insert("hero".into(), "a dragon".into());
        vals.insert("place".into(), "a cave".into());

        let out = m.fill(&vals).unwrap();
        assert_eq!(out, "Once upon a time, a dragon lived in a cave.");
    }

    #[test]
    fn test_motif_fill_missing_slot_returns_none() {
        let m = Motif::new("m", MotifKind::TextTemplate, "{a} and {b}");
        let mut vals = HashMap::new();
        vals.insert("a".into(), "x".into());
        assert!(m.fill(&vals).is_none());
    }

    #[test]
    fn test_bank_insert_and_lookup() {
        let mut bank = MotifBank::new();
        bank.insert(
            Motif::new("o1", MotifKind::TextTemplate, "Hello {name}")
                .with_tag("greeting")
                .with_domain("casual"),
        );
        bank.insert(Motif::new("c1", MotifKind::CodePattern, "for (let {i} = 0; {i} < {n}; {i}++)"));

        assert_eq!(bank.count(), 2);
        assert_eq!(bank.by_kind(MotifKind::TextTemplate).len(), 1);
        assert_eq!(bank.by_kind(MotifKind::CodePattern).len(), 1);
        assert_eq!(bank.by_tag("greeting").len(), 1);
        assert_eq!(bank.by_domain("casual").len(), 1);
    }

    #[test]
    fn test_best_match_prefers_style() {
        let mut bank = MotifBank::new();
        let mut formal = Motif::new("formal1", MotifKind::TextTemplate, "Dear {name},");
        formal.observed_count = 100;
        formal.style.push("formal".into());

        let mut casual = Motif::new("casual1", MotifKind::TextTemplate, "Hey {name},");
        casual.observed_count = 200;
        casual.style.push("casual".into());

        bank.insert(formal);
        bank.insert(casual);

        // Asking for "formal" should get formal even though casual is more popular
        let m = bank.best_match(MotifKind::TextTemplate, "formal").unwrap();
        assert_eq!(m.id, "formal1");

        let m2 = bank.best_match(MotifKind::TextTemplate, "casual").unwrap();
        assert_eq!(m2.id, "casual1");
    }

    #[test]
    fn test_narrative_beat_motifs() {
        let mut bank = MotifBank::new();
        bank.insert(
            Motif::new(
                "conflict_intro",
                MotifKind::NarrativeBeat,
                "{character} faced {obstacle} for the first time.",
            )
            .with_tag("conflict")
            .with_style("dramatic"),
        );
        bank.insert(
            Motif::new(
                "reveal",
                MotifKind::NarrativeBeat,
                "Then {character} realized that {truth}.",
            )
            .with_tag("reveal")
            .with_style("surprising"),
        );

        assert_eq!(bank.by_kind(MotifKind::NarrativeBeat).len(), 2);
        assert_eq!(bank.by_tag("conflict").len(), 1);
        assert_eq!(bank.by_tag("reveal").len(), 1);
    }

    #[test]
    fn test_musical_phrase_motif() {
        let chord = Motif::new(
            "pop_progression",
            MotifKind::MusicalPhrase,
            "I-V-vi-IV in {key}",
        )
        .with_domain("pop")
        .with_style("uplifting");

        assert_eq!(chord.slots, vec!["key"]);
        assert!(chord.matches_style("uplifting"));
    }

    #[test]
    fn test_image_prompt_motif() {
        let m = Motif::new(
            "portrait_spec",
            MotifKind::ImagePrompt,
            "{subject}, {style} portrait, {lighting} lighting, {composition}",
        );

        assert_eq!(m.slots, vec!["subject", "style", "lighting", "composition"]);

        let mut vals = HashMap::new();
        vals.insert("subject".into(), "a brown bulldog with white face".into());
        vals.insert("style".into(), "photorealistic".into());
        vals.insert("lighting".into(), "soft natural".into());
        vals.insert("composition".into(), "centered, eye-level".into());

        let prompt = m.fill(&vals).unwrap();
        assert!(prompt.contains("brown bulldog"));
        assert!(prompt.contains("soft natural"));
    }

    #[test]
    fn test_record_use_updates_count() {
        let mut m = Motif::new("m", MotifKind::TextTemplate, "hi");
        assert_eq!(m.used_count, 0);
        m.record_use();
        assert_eq!(m.used_count, 1);
    }
}
