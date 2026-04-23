//! graph_v4::tokenize — tokenization בסיסי לאנגלית ועברית.
//!
//! עקרונות:
//!   - tokens: רצפים של תווים אלפא-בתיים (כולל עברית)
//!   - lowercase לטוקנים אנגליים (עברית היא case-insensitive בטבע)
//!   - הסרת פיסוק, ספרות, סמלים
//!   - sentence split על . ! ? (וגם '! ו-? בעברית זהים)

/// מחזיר tokens נקיים מתוך text.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        let is_word_char = ch.is_alphabetic() || ch == '\'';
        if is_word_char {
            buf.extend(ch.to_lowercase());
        } else {
            if !buf.is_empty() {
                tokens.push(std::mem::take(&mut buf));
            }
        }
    }
    if !buf.is_empty() {
        tokens.push(buf);
    }
    tokens
}

/// פירוק ל-sentences. חותך על "." "!" "?" כשאחריהם whitespace.
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        current.push(chars[i]);
        if matches!(chars[i], '.' | '!' | '?') {
            // בדוק אם הבא whitespace
            if i + 1 >= chars.len() || chars[i + 1].is_whitespace() {
                let s = current.trim().to_string();
                if s.len() > 10 {
                    sentences.push(s);
                }
                current.clear();
            }
        }
        i += 1;
    }
    // tail
    let s = current.trim().to_string();
    if s.len() > 10 {
        sentences.push(s);
    }
    sentences
}

/// stopwords לאנגלית + עברית — לפילטור ב-retrieval, לא ב-ingestion.
pub fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        // English
        "the" | "a" | "an" | "is" | "are" | "was" | "were" | "what" | "who"
        | "where" | "when" | "how" | "why" | "of" | "in" | "on" | "at" | "to"
        | "from" | "by" | "with" | "for" | "and" | "or" | "but" | "not" | "do"
        | "does" | "did" | "have" | "has" | "had" | "can" | "could" | "would"
        | "should" | "will" | "this" | "that" | "these" | "those" | "it" | "its"
        | "my" | "your" | "tell" | "me" | "about" | "us" | "as" | "be" | "been"
        // Hebrew
        | "מה" | "מי" | "איפה" | "מתי" | "איך" | "למה" | "של" | "את"
        | "זה" | "זאת" | "זו" | "יש" | "אין" | "אני" | "אתה" | "הוא"
        | "היא" | "על" | "אל" | "עם"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_basic() {
        let toks = tokenize("The quick brown fox jumps.");
        assert_eq!(toks, vec!["the", "quick", "brown", "fox", "jumps"]);
    }

    #[test]
    fn hebrew_basic() {
        let toks = tokenize("שלום עולם, איך הולך?");
        // Hebrew chars are alphabetic, lowercase is no-op
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0], "שלום");
    }

    #[test]
    fn punctuation_stripped() {
        let toks = tokenize("Hello, world! (test)");
        assert_eq!(toks, vec!["hello", "world", "test"]);
    }

    #[test]
    fn apostrophe_preserved() {
        let toks = tokenize("Newton's laws don't apply.");
        assert_eq!(toks, vec!["newton's", "laws", "don't", "apply"]);
    }

    #[test]
    fn sentence_split() {
        let sents = split_sentences("First sentence is here. Second sentence is longer. Third!");
        assert_eq!(sents.len(), 2); // "Third!" too short
        assert!(sents[0].starts_with("First"));
    }

    #[test]
    fn stopwords_filtered() {
        assert!(is_stopword("the"));
        assert!(is_stopword("of"));
        assert!(is_stopword("מה"));
        assert!(!is_stopword("gravity"));
    }
}
