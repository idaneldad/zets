//! graph_v4::build — בונה את ה-Graph מאוסף של (title, text).
//!
//! Flow (זהה ל-v4_path_graph.py):
//!   1. Split sentences, tokenize
//!   2. Extract phrases (n-grams ≥ min_count)
//!   3. Create atoms: article + word + phrase + sentence
//!   4. For each sentence:
//!        - greedy phrase match → units sequence
//!        - edges: fills_slot(pos), next (unit→unit), part_of_backref
//!        - phrase has_part → words; word part_of → phrase
//!        - sentence contained_in → article; article has_sentence → sentence
//!   5. co_occurs edges בין מילים שכנות (window 5)

use super::cleaner::{is_junk_sentence, strip_inline_markup};
use super::morphology::resolve_lemma;
use super::phrase::{extract_phrases, match_phrases_in_sentence};
use super::tokenize::{split_sentences, tokenize};
use super::types::{AtomKind, Graph, Relation};

pub struct BuildConfig {
    pub phrase_min_count: u32,
    pub co_occurs_window: usize,
    pub enable_cleaner: bool,
    pub enable_lemmas: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        BuildConfig {
            phrase_min_count: 3,
            co_occurs_window: 5,
            enable_cleaner: true,
            enable_lemmas: false,
        }
    }
}

/// בונה graph מלא מ-articles. כל `(title, text)` מוכנס.
pub fn build_graph(articles: &[(String, String)], config: &BuildConfig) -> Graph {
    let mut g = Graph::new();

    // שלב 1: parse sentences + tokens
    // sent_meta[k] = (article_title, sent_idx_in_article, tokens, original_text)
    let mut sent_meta: Vec<(String, usize, Vec<String>, String)> = Vec::new();
    for (title, text) in articles {
        let sents = split_sentences(text);
        let mut kept_sidx = 0;
        for stext in sents.iter() {
            // CLEANUP: skip junk sentences (infobox, markup, bullets, etc.)
            if config.enable_cleaner && is_junk_sentence(stext) { continue; }
            let cleaned = if config.enable_cleaner { strip_inline_markup(stext) } else { stext.clone() };
            let tokens = tokenize(&cleaned);
            if tokens.len() < 2 { continue; }
            sent_meta.push((title.clone(), kept_sidx, tokens, cleaned));
            kept_sidx += 1;
        }
    }

    let all_tokens: Vec<Vec<String>> = sent_meta.iter().map(|(_, _, t, _)| t.clone()).collect();

    // שלב 2: extract phrases
    let phrases = extract_phrases(&all_tokens, config.phrase_min_count);

    // שלב 3: create atoms — article + word + phrase (sentence נוצרים תוך-כדי)
    // כדי לקבל דטרמיניות: sort articles by title
    let mut sorted_titles: Vec<String> = articles.iter().map(|(t, _)| t.clone()).collect();
    sorted_titles.sort();
    for title in &sorted_titles {
        g.atom(AtomKind::Article, title);
    }

    // words: נסרוק את כל ה-tokens, sorted כדי לקבל determinism ב-id
    let mut all_words: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for tokens in &all_tokens {
        for t in tokens {
            all_words.insert(t.clone());
        }
    }
    for w in &all_words {
        g.atom(AtomKind::Word, w);
    }

    // phrases: sorted by key for determinism
    let mut sorted_phrases: Vec<(Vec<String>, u32)> = phrases.iter()
        .map(|(k, v)| (k.clone(), *v)).collect();
    sorted_phrases.sort();
    for (ng, count) in &sorted_phrases {
        let key = ng.join(" ");
        let id = g.atom(AtomKind::Phrase, &key);
        g.atoms[id as usize].count = *count;
    }

    // שלב 4: sentence atoms + edges
    for (title, sidx, tokens, stext) in &sent_meta {
        let sent_key = format!("{}:{}", title, sidx);
        let sid = g.atom(AtomKind::Sentence, &sent_key);
        g.set_text(sid, stext.clone());

        let aid = g.atom(AtomKind::Article, title);
        g.edge(sid, aid, Relation::ContainedIn, 1, *sidx as u16);

        // greedy phrase matching
        let matched = match_phrases_in_sentence(tokens, &phrases);

        // units = phrases + words שלא נבלעו
        let mut covered: Vec<bool> = vec![false; tokens.len()];
        let mut units: Vec<(usize, usize, AtomKind, String)> = Vec::new();
        for (s, e, ng) in &matched {
            units.push((*s, *e, AtomKind::Phrase, ng.join(" ")));
            for j in *s..*e { covered[j] = true; }
        }
        for (j, tok) in tokens.iter().enumerate() {
            if !covered[j] {
                units.push((j, j + 1, AtomKind::Word, tok.clone()));
            }
        }
        units.sort_by_key(|u| u.0);

        // fills_slot + next + part_of_backref
        let mut prev_uid: Option<u32> = None;
        for (pos, (_, _, kind, key)) in units.iter().enumerate() {
            let uid = g.atom(*kind, key);
            g.edge(uid, sid, Relation::FillsSlot, 1, pos as u16);
            let _ = prev_uid; prev_uid = Some(uid);

            // phrase: has_part → words, word part_of → phrase
            if *kind == AtomKind::Phrase {
                for w in key.split(' ') {
                    let wid = g.atom(AtomKind::Word, w);
                    g.edge(wid, uid, Relation::PartOf, 1, 0);
                }
            }
        }

// co_occurs/next/reverse edges — אינם נחוצים, מושגים דרך fills_slot + sort by pos
    }
    // ─── Lemma edges — rule-based morphology (surface → lemma) ───
    if config.enable_lemmas {
        // Build vocab: set of all word atoms' keys
        let mut vocab: std::collections::HashSet<String> = std::collections::HashSet::new();
        for a in &g.atoms {
            if a.kind == AtomKind::Word {
                vocab.insert(a.key.clone());
            }
        }
        // For each word atom, try to resolve lemma; if found, add lemma_of edge
        let word_atoms: Vec<(u32, String)> = g.atoms.iter().enumerate()
            .filter(|(_, a)| a.kind == AtomKind::Word)
            .map(|(i, a)| (i as u32, a.key.clone()))
            .collect();
        for (aid, key) in word_atoms {
            if let Some(lemma) = resolve_lemma(&key, &vocab) {
                if let Some(&lemma_id) = g.by_key.get(&(AtomKind::Word, lemma)) {
                    g.edge(aid, lemma_id, Relation::LemmaOf, 1, 0);
                }
            }
        }
    }

    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tiny_corpus() {
        let articles = vec![
            ("Test".to_string(), "The heart is strong. The heart beats fast. The heart works well. \
             Gravity pulls down. Gravity is a force. Gravity affects all matter.".to_string()),
        ];
        let g = build_graph(&articles, &BuildConfig::default());
        let s = g.stats();

        // Sanity
        assert!(s.atoms_total > 0);
        assert!(s.edges_total > 0);

        // Phrase "the heart" מופיע 3 פעמים — אמור להיות phrase atom
        let key = "the heart";
        assert!(g.by_key.contains_key(&(AtomKind::Phrase, key.to_string())),
                "'the heart' phrase not found");
    }

    #[test]
    fn determinism() {
        let articles = vec![
            ("A".to_string(), "Photosynthesis is the process plants use. \
             Photosynthesis requires light and water. Photosynthesis happens in leaves.".to_string()),
            ("B".to_string(), "The sun provides energy. The sun is a star. \
             The sun drives photosynthesis in plants.".to_string()),
        ];
        let g1 = build_graph(&articles, &BuildConfig::default());
        let g2 = build_graph(&articles, &BuildConfig::default());
        assert_eq!(g1.atom_count(), g2.atom_count());
        assert_eq!(g1.edge_count(), g2.edge_count());
        // verify atom keys in same order (determinism)
        for (a1, a2) in g1.atoms.iter().zip(g2.atoms.iter()) {
            assert_eq!(a1.kind, a2.kind);
            assert_eq!(a1.key, a2.key);
        }
    }
}
