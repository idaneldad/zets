//! Epistemic classifier — pattern-based detection of truth-claim type.
//!
//! No LLM needed. Uses keyword patterns + provenance signals to classify
//! content into epistemic categories. This is deliberately conservative:
//! when in doubt, returns Unknown rather than guessing.

use super::provenance::Provenance;
use super::work::EpistemicStatus;

/// Pattern sets for each epistemic category.
/// Each set contains (keywords, minimum_hits) — if a text contains at least
/// `minimum_hits` keywords from the set, it's a candidate for that status.
struct PatternRule {
    status: EpistemicStatus,
    keywords: &'static [&'static str],
    min_hits: usize,
}

const RULES: &[PatternRule] = &[
    PatternRule {
        status: EpistemicStatus::EmpiricalFact,
        keywords: &[
            "measured", "experiment", "data shows", "data show",
            "observed", "replicated", "statistically significant",
            "p-value", "hypothesis", "control group", "results indicate",
            "findings suggest", "evidence demonstrates",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::Theoretical,
        keywords: &[
            "theory", "model predicts", "theoretically",
            "framework suggests", "postulates", "implies that",
            "in principle", "axiom", "conjecture",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::HistoricalRecord,
        keywords: &[
            "in the year", "on the date", "historically",
            "records show", "documented in", "chronicled",
            "archaeological", "according to records", "dated to",
            "century", "dynasty",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::Tradition,
        keywords: &[
            "tradition holds", "traditionally", "passed down",
            "oral tradition", "customary", "ancestral",
            "according to tradition", "folk wisdom", "elders say",
            "it is said", "they say",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::ReligiousNarrative,
        keywords: &[
            "god said", "god created", "the lord", "and god",
            "scripture", "revelation", "divine", "prophet",
            "commandment", "blessed", "holy", "sacred text",
            "psalm", "verse", "surah", "chapter and verse",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::Opinion,
        keywords: &[
            "i think", "i believe", "in my opinion", "should",
            "ought to", "it seems", "arguably", "personally",
            "from my perspective", "i feel that",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::Fiction,
        keywords: &[
            "once upon a time", "in a land far", "the end",
            "chapter one", "novel", "story", "fictional",
            "imaginary", "fairy tale",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::Mythology,
        keywords: &[
            "myth", "legend", "mythological", "legendary",
            "the gods", "demigod", "pantheon", "saga",
            "in the beginning of time",
        ],
        min_hits: 1,
    },
    PatternRule {
        status: EpistemicStatus::Speculation,
        keywords: &[
            "might be", "could be", "possibly", "speculate",
            "unverified", "rumor", "allegedly", "it is claimed",
            "some say", "unconfirmed",
        ],
        min_hits: 1,
    },
];

/// Religious source keywords for provenance-based override.
const RELIGIOUS_SOURCES: &[&str] = &[
    "bible", "torah", "quran", "koran", "talmud", "midrash",
    "gospel", "vedas", "upanishad", "bhagavad", "sutra",
    "tanakh", "mishnah", "zohar", "genesis", "exodus",
    "leviticus", "numbers", "deuteronomy", "psalms",
];

/// Classify text into an epistemic status using pattern matching + provenance.
///
/// Priority:
/// 1. Provenance-based overrides (religious sources → ReligiousNarrative)
/// 2. Keyword pattern matching (highest-scoring category wins)
/// 3. Default: Unknown
pub fn classify_epistemic(text: &str, provenance: &Provenance) -> EpistemicStatus {
    // Provenance override: religious source → ReligiousNarrative
    if provenance.source_contains_any(RELIGIOUS_SOURCES) {
        return EpistemicStatus::ReligiousNarrative;
    }

    // License signal: CC-BY usually implies factual/academic
    let has_cc_license = provenance.license.as_ref()
        .map(|l| l.to_lowercase().contains("cc-by") || l.to_lowercase().contains("cc0"))
        .unwrap_or(false);

    let lower = text.to_lowercase();

    // Score each rule
    let mut best: Option<(EpistemicStatus, usize)> = None;

    for rule in RULES {
        let hits = rule.keywords.iter()
            .filter(|kw| lower.contains(*kw))
            .count();

        if hits >= rule.min_hits {
            match &best {
                Some((_, best_hits)) if hits <= *best_hits => {}
                _ => best = Some((rule.status, hits)),
            }
        }
    }

    if let Some((status, _)) = best {
        return status;
    }

    // If CC license and no other signal, lean toward empirical/theoretical
    if has_cc_license {
        return EpistemicStatus::EmpiricalFact;
    }

    EpistemicStatus::Unknown
}
