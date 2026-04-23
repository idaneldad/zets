//! graph_v4::phrase — n-gram extraction.
//!
//! Given all tokenized sentences of the corpus, find n-grams (2-4 words) that
//! occur ≥ min_count times. Filter out all-stopword n-grams.
//!
//! זה ה-"גילוי ביטויים חוזרים" — "albert einstein", "the heart", "quantum mechanics".

use crate::graph_v4::tokenize::is_stopword;
use std::collections::HashMap;

/// n-gram → count
pub type PhraseMap = HashMap<Vec<String>, u32>;

pub fn extract_phrases(sentences: &[Vec<String>], min_count: u32) -> PhraseMap {
    let mut counts: HashMap<Vec<String>, u32> = HashMap::new();

    for tokens in sentences {
        // מכסה 2-grams, 3-grams, 4-grams
        for n in 2..=4usize {
            if tokens.len() < n { continue; }
            for i in 0..=(tokens.len() - n) {
                let ng: Vec<String> = tokens[i..i + n].to_vec();
                *counts.entry(ng).or_insert(0) += 1;
            }
        }
    }

    // פילטר: רק אלה שמעל min_count, ולא כל-stopwords
    counts.retain(|ng, &mut cnt| {
        cnt >= min_count && !ng.iter().all(|t| is_stopword(t))
    });

    counts
}

/// Greedy longest-match: בתוך tokens של משפט, מצא phrase matches בעדיפות ל-n הגדול ביותר.
/// מחזיר רשימת (start, end, phrase).
pub fn match_phrases_in_sentence(
    tokens: &[String],
    phrases: &PhraseMap,
) -> Vec<(usize, usize, Vec<String>)> {
    let mut matches = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let mut matched = false;
        for n in (2..=4usize).rev() {
            if i + n <= tokens.len() {
                let ng: Vec<String> = tokens[i..i + n].to_vec();
                if phrases.contains_key(&ng) {
                    matches.push((i, i + n, ng));
                    i += n;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            i += 1;
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_repeated() {
        let s = vec![
            vec!["the".into(), "heart".into(), "is".into()],
            vec!["the".into(), "heart".into(), "works".into()],
            vec!["the".into(), "heart".into(), "beats".into()],
        ];
        let phrases = extract_phrases(&s, 3);
        // "the heart" מופיע 3 פעמים
        let key: Vec<String> = vec!["the".into(), "heart".into()];
        // NOT all-stopwords כי "heart" לא stopword
        assert_eq!(phrases.get(&key), Some(&3));
        // "the" "the" בודדת היא word, לא phrase — אין כזה
    }

    #[test]
    fn filter_all_stopwords() {
        let s = vec![
            vec!["the".into(), "of".into(), "in".into()],
            vec!["the".into(), "of".into(), "in".into()],
            vec!["the".into(), "of".into(), "in".into()],
        ];
        let phrases = extract_phrases(&s, 2);
        // "the of" - כל stopwords → מסונן
        let key: Vec<String> = vec!["the".into(), "of".into()];
        assert!(!phrases.contains_key(&key));
    }

    #[test]
    fn greedy_matching() {
        let mut phrases = PhraseMap::new();
        phrases.insert(vec!["albert".into(), "einstein".into()], 5);
        phrases.insert(vec!["the".into(), "heart".into()], 3);

        let tokens = vec![
            "who".into(), "was".into(), "albert".into(), "einstein".into(),
            "and".into(), "the".into(), "heart".into(), "specialist".into(),
        ];
        let matches = match_phrases_in_sentence(&tokens, &phrases);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].0, 2);
        assert_eq!(matches[0].1, 4);
        assert_eq!(matches[1].0, 5);
    }
}
