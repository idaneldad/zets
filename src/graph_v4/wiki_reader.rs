//! graph_v4::wiki_reader — קריאה streaming מ-wikipedia_dumps/<lang>_parsed.jsonl.gz.
//!
//! כדי לשמור על zero-dependency policy (אין crate ל-gz ב-Cargo.toml),
//! אני מפעיל `gunzip` בתור subprocess. stdin של gunzip ← file, stdout ← pipe
//! שנקרא לתוך BufReader. זה פשוט, מהיר, ונקי.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct Article {
    pub title: String,
    pub text: String,
}

/// קורא עד `max` articles שעונים על פילטר (length window).
/// מחזיר Vec<(title, text)>.
pub fn read_articles<P: AsRef<Path>>(
    gz_path: P,
    max: usize,
    min_len: usize,
    max_len: usize,
) -> std::io::Result<Vec<Article>> {
    let child = Command::new("gunzip")
        .arg("-c")
        .arg(gz_path.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.expect("gunzip stdout");
    let reader = BufReader::new(stdout);

    let mut out = Vec::with_capacity(max);
    for line_r in reader.lines() {
        if out.len() >= max { break; }
        let line = match line_r { Ok(l) => l, Err(_) => continue };
        // parse JSON ידנית (זול) — אנחנו רק צריכים title + text
        if let Some((title, text)) = extract_title_and_text(&line) {
            let tlen = text.len();
            if tlen >= min_len && tlen <= max_len {
                out.push(Article { title: title.to_string(), text: text.to_string() });
            }
        }
    }
    Ok(out)
}

/// JSON parser מינימלי לשדות "title" ו-"text". נמנע מ-serde_json כדי לא להוסיף dep.
/// Supports basic escapes: \", \\, \n, \t, \r, \u{...}.
fn extract_title_and_text(line: &str) -> Option<(String, String)> {
    let title = extract_string_field(line, "\"title\"")?;
    let text = extract_string_field(line, "\"text\"")?;
    Some((title, text))
}

fn extract_string_field(line: &str, key: &str) -> Option<String> {
    let key_idx = line.find(key)?;
    let after_key = &line[key_idx + key.len()..];
    // skip : and whitespace
    let colon = after_key.find(':')?;
    let mut rest = &after_key[colon + 1..];
    while let Some(c) = rest.chars().next() {
        if c.is_whitespace() { rest = &rest[c.len_utf8()..]; } else { break; }
    }
    if !rest.starts_with('"') { return None; }
    rest = &rest[1..];

    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000C}'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                other => out.push(other),
            }
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_extract() {
        let line = r#"{"title": "Heart", "text": "The heart is a pump.", "other": 1}"#;
        let (t, x) = extract_title_and_text(line).unwrap();
        assert_eq!(t, "Heart");
        assert_eq!(x, "The heart is a pump.");
    }

    #[test]
    fn with_escapes() {
        let line = r#"{"title": "Test", "text": "line1\nline2 \"quote\""}"#;
        let (t, x) = extract_title_and_text(line).unwrap();
        assert_eq!(t, "Test");
        assert_eq!(x, "line1\nline2 \"quote\"");
    }

    #[test]
    fn hebrew() {
        let line = r#"{"title": "לב", "text": "הלב הוא איבר."}"#;
        let (t, x) = extract_title_and_text(line).unwrap();
        assert_eq!(t, "לב");
        assert!(x.contains("הלב"));
    }
}
