//! graph_v4::retrieve — IDF-weighted path-based retrieval.
//!
//! Flow:
//!   1. Tokenize query, remove stopwords
//!   2. Greedy phrase match (prefer long phrases)
//!   3. For each seed: compute IDF
//!   4. For each sentence: accumulate score by matching seeds (IDF-weighted)
//!   5. Proximity boost: if 2+ seeds close together in sentence, multiply score
//!   6. Aggregate to articles
//!   7. Return top articles + top sentences

use super::phrase::{match_phrases_in_sentence, PhraseMap};
use super::tokenize::{is_stopword, tokenize};
use super::types::{AtomId, AtomKind, Graph, Relation};
use std::collections::HashMap;

/// IDF table: atom_id → log(n_articles / df)
pub type IdfTable = HashMap<AtomId, f32>;

pub fn compute_idf(g: &Graph) -> IdfTable {
    let mut idf = IdfTable::new();
    let n_articles = g.atoms.iter().filter(|a| a.kind == AtomKind::Article).count() as f32;
    let n_articles = n_articles.max(1.0);

    let out = g.out_by_rel.as_ref().expect("call build_indexes() first");

    for (aid, atom) in g.atoms.iter().enumerate() {
        if !matches!(atom.kind, AtomKind::Word | AtomKind::Phrase) {
            continue;
        }
        let aid = aid as u32;
        // seed → fills_slot → sentences
        let mut sentence_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for (sid, _w, _pos) in &out[aid as usize][Relation::FillsSlot as usize] {
            sentence_ids.insert(*sid);
        }
        // sentences → contained_in → articles
        let mut article_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for sid in &sentence_ids {
            for (art_id, _w, _pos) in &out[*sid as usize][Relation::ContainedIn as usize] {
                article_ids.insert(*art_id);
            }
        }
        let df = article_ids.len().max(1) as f32;
        idf.insert(aid, (n_articles / df).ln());
    }
    idf
}

/// Reconstruct phrases map from graph (מחזיר את ה-phrases atoms כ-PhraseMap).
pub fn phrases_from_graph(g: &Graph) -> PhraseMap {
    let mut m = PhraseMap::new();
    for atom in &g.atoms {
        if atom.kind == AtomKind::Phrase {
            let words: Vec<String> = atom.key.split(' ').map(|s| s.to_string()).collect();
            m.insert(words, atom.count);
        }
    }
    m
}

/// תוצאת retrieval: top articles + top sentences + פרטים.
#[derive(Debug)]
pub struct Answer {
    pub tokens: Vec<String>,
    pub seeds: Vec<(AtomKind, String)>, // (kind, key)
    pub top_articles: Vec<(String, f32)>,
    pub top_sentences: Vec<(String, f32, String)>, // (text, score, sentence_key)
}

pub fn answer(
    query: &str,
    g: &Graph,
    idf: &IdfTable,
    phrases: &PhraseMap,
    top_k_sents: usize,
    top_k_arts: usize,
) -> Answer {
    // tokenize + filter stopwords
    let all_tokens = tokenize(query);
    let kept: Vec<String> = all_tokens.into_iter()
        .filter(|t| !is_stopword(t) && t.len() > 1)
        .collect();

    // greedy: phrase matches first, then standalone words
    let matched = match_phrases_in_sentence(&kept, phrases);
    let mut covered = vec![false; kept.len()];
    let mut seeds: Vec<(AtomKind, String, AtomId)> = Vec::new();
    for (s, e, ng) in &matched {
        let key = ng.join(" ");
        if let Some(&id) = g.by_key.get(&(AtomKind::Phrase, key.clone())) {
            seeds.push((AtomKind::Phrase, key, id));
            for j in *s..*e { covered[j] = true; }
        }
    }
    for (j, tok) in kept.iter().enumerate() {
        if covered[j] {
            continue;
        }
        if let Some(&id) = g.by_key.get(&(AtomKind::Word, tok.clone())) {
            seeds.push((AtomKind::Word, tok.clone(), id));
        }
    }

    // score sentences: for each seed, walk fills_slot → sentence, add IDF
    let out = g.out_by_rel.as_ref().expect("call build_indexes()");
    let mut sent_scores: HashMap<AtomId, f32> = HashMap::new();
    let mut sent_positions: HashMap<AtomId, Vec<u16>> = HashMap::new();

    for (_, _, aid) in &seeds {
        let w = *idf.get(aid).unwrap_or(&0.5);
        // Direct hop: seed → fills_slot → sentence
        for (sid, _weight, pos) in &out[*aid as usize][Relation::FillsSlot as usize] {
            *sent_scores.entry(*sid).or_insert(0.0) += w;
            sent_positions.entry(*sid).or_insert_with(Vec::new).push(*pos);
        }
        // 2-hop via phrase: seed (word) → part_of → phrase → fills_slot → sentence
        for (phrase_id, _w1, _p1) in &out[*aid as usize][Relation::PartOf as usize] {
            let w2 = w * 0.8;
            for (sid, _w2, pos) in &out[*phrase_id as usize][Relation::FillsSlot as usize] {
                *sent_scores.entry(*sid).or_insert(0.0) += w2;
                sent_positions.entry(*sid).or_insert_with(Vec::new).push(*pos);
            }
        }
        // (lemma_of walk disabled — didn't improve on 10K corpus; re-evaluate on 50K+)
    }

    // proximity boost
    for (sid, positions) in &mut sent_positions {
        if positions.len() >= 2 {
            positions.sort();
            let gaps: Vec<u16> = positions.windows(2).map(|w| w[1] - w[0]).collect();
            let avg_gap: f32 = gaps.iter().map(|&g| g as f32).sum::<f32>() / gaps.len() as f32;
            let boost = 1.0 + 1.0 / (1.0 + avg_gap);
            if let Some(s) = sent_scores.get_mut(sid) {
                *s *= boost;
            }
        }
    }

    // aggregate → articles
    let mut art_scores: HashMap<AtomId, f32> = HashMap::new();
    for (sid, score) in &sent_scores {
        for (art_id, _w, _pos) in &out[*sid as usize][Relation::ContainedIn as usize] {
            *art_scores.entry(*art_id).or_insert(0.0) += score;
        }
    }

    // ── DISAMBIGUATION BOOST ──
    // If a seed's key matches an article title (case-insensitive), multiply that article's score.
    // Helps distinguish e.g. "Apollo 11" query → Apollo 11 article vs Apollo 14.
    for (_, seed_key, _) in &seeds {
        let seed_lower = seed_key.to_lowercase();
        for (art_id, score) in art_scores.iter_mut() {
            let title = &g.atoms[*art_id as usize].key;
            let title_lower = title.to_lowercase();
            // exact match: score ×5
            if title_lower == seed_lower {
                *score *= 5.0;
            }
            // title starts with seed (e.g. "Apollo 11" → "Apollo 11" article): ×3
            else if title_lower.starts_with(&seed_lower) && seed_lower.len() >= 4 {
                *score *= 3.0;
            }
            // seed is full article title minus parenthetical (e.g. "C" → "C (programming language)"): ×2
            else if title_lower.split(" (").next().unwrap_or("") == seed_lower {
                *score *= 4.0;
            }
        }
    }

    // sort
    let mut sent_vec: Vec<(AtomId, f32)> = sent_scores.into_iter().collect();
    sent_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut art_vec: Vec<(AtomId, f32)> = art_scores.into_iter().collect();
    art_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let top_articles: Vec<(String, f32)> = art_vec.into_iter()
        .take(top_k_arts)
        .map(|(id, s)| (g.atoms[id as usize].key.clone(), s))
        .collect();
    let top_sentences: Vec<(String, f32, String)> = sent_vec.into_iter()
        .take(top_k_sents)
        .map(|(id, s)| {
            let atom = &g.atoms[id as usize];
            (
                atom.text.clone().unwrap_or_else(|| atom.key.clone()),
                s,
                atom.key.clone(),
            )
        })
        .collect();

    Answer {
        tokens: kept,
        seeds: seeds.into_iter().map(|(k, key, _)| (k, key)).collect(),
        top_articles,
        top_sentences,
    }
}
