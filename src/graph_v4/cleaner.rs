//! graph_v4::cleaner — מסנן junk ב-wiki markup לפני tokenization.
//!
//! הוכח אמפירית: 40% מ-top sentences ברecall הנוכחי מכילים junk כמו
//! "thumb|200px|", "| children = 3", "*ראו גם", infobox rows.
//!
//! ה-cleaner פועל ברמת המשפט (לפני split), ומחזיר None ל-junk או String נקיה.

/// אם משפט הוא junk — לא יוכנס לגרף.
pub fn is_junk_sentence(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() { return true; }

    // Wiki markup: thumb|, 200px|, |right|, etc.
    if t.starts_with("thumb|") || t.starts_with("|") { return true; }
    if t.contains("thumb|") && t.contains("px|") { return true; }

    // Infobox row: "| children = 3", "| death_cause = ..."
    if t.starts_with("| ") && t.contains(" = ") { return true; }

    // HTML comments
    if t.starts_with("<!--") || t.starts_with("-->") { return true; }

    // Bullet points (בדרך כלל "See also" / "ראו גם" lists)
    if t.starts_with("* ") || t.starts_with("*") { return true; }
    if t.starts_with("# ") { return true; }

    // Section headers (== Heading ==)
    if t.starts_with("==") { return true; }

    // Templates-only ({{...}} wrapping)
    if (t.starts_with("{{") && t.ends_with("}}")) ||
       (t.starts_with("[[") && t.ends_with("]]")) { return true; }

    // URLs ונמל
    if t.starts_with("http://") || t.starts_with("https://") { return true; }

    // Very short (אחרי strip) — פחות מ-4 מילים
    let word_count = t.split_whitespace().count();
    if word_count < 4 { return true; }

    // סקציית "ראו גם" / "See also" — מתחיל בטקסט זה
    if t.starts_with("ראו גם") || t.starts_with("See also") ||
       t.starts_with("קישורים חיצוניים") || t.starts_with("External links") ||
       t.starts_with("לקריאה נוספת") || t.starts_with("References") ||
       t.starts_with("Notes") || t.starts_with("הערות שוליים") { return true; }

    false
}

/// מסיר wiki markup inline מתוך משפט תקין (keeps the sentence, cleans its contents).
pub fn strip_inline_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // [[link|text]] → text  (או [[link]] → link)
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i+1] == b'[' {
            if let Some(end) = find_close(s, i + 2, "]]") {
                let inner = &s[i+2..end];
                let text = inner.rsplit_once('|').map(|(_, t)| t).unwrap_or(inner);
                out.push_str(text);
                i = end + 2;
                continue;
            }
        }
        // {{template}} → drop entirely
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i+1] == b'{' {
            if let Some(end) = find_close(s, i + 2, "}}") {
                i = end + 2;
                continue;
            }
        }
        // <ref>...</ref> → drop
        if s[i..].starts_with("<ref") {
            if let Some(end) = s[i..].find("</ref>") {
                i += end + 6;
                continue;
            }
            if let Some(end) = s[i..].find("/>") {
                i += end + 2;
                continue;
            }
        }
        // <!--...--> → drop
        if s[i..].starts_with("<!--") {
            if let Some(end) = s[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }

        // החסר — פשוט push
        let c = s[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

fn find_close(s: &str, start: usize, delim: &str) -> Option<usize> {
    s[start..].find(delim).map(|rel| start + rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn junk_detection() {
        assert!(is_junk_sentence("thumb|200px|Albert Einstein"));
        assert!(is_junk_sentence("| children = 3, including Hans Albert"));
        assert!(is_junk_sentence("*ראו גם *דמוקרטיה *אנרכיזם"));
        assert!(is_junk_sentence("== History =="));
        assert!(is_junk_sentence("ראו גם *אלברט איינשטיין"));
        assert!(is_junk_sentence("See also"));
        assert!(is_junk_sentence("http://example.com"));
        assert!(is_junk_sentence("short"));

        assert!(!is_junk_sentence("Anarchism is a political philosophy that opposes authority."));
        assert!(!is_junk_sentence("הלב הוא איבר השריר המרכזי שבגוף האדם."));
    }

    #[test]
    fn inline_markup_strip() {
        assert_eq!(strip_inline_markup("Einstein [[physicist|was a physicist]]."),
                   "Einstein was a physicist.");
        assert_eq!(strip_inline_markup("Text {{template|arg}} continues."),
                   "Text  continues.");
        assert_eq!(strip_inline_markup("As noted<ref>Smith 1985</ref> previously."),
                   "As noted previously.");
        assert_eq!(strip_inline_markup("[[Anarchism]] is a movement."),
                   "Anarchism is a movement.");
    }
}
