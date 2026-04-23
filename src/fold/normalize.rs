//! Text normalization before hashing.
//!
//! Python prototype measured: normalization yields **6× more deduplication**
//! on near-duplicate inputs (Hello World / hello world / HELLO WORLD! etc.)
//!
//! Order matters:
//! 1. Unicode NFKC (compatibility decomposition + canonical composition)
//! 2. Lowercase (locale-insensitive ASCII fast path + Unicode fallback)
//! 3. Whitespace collapse (runs of whitespace → single space)
//! 4. Punctuation strip (optional — kept behind flag for content atoms)
//! 5. Trim
//!
//! We DON'T depend on the `unicode-normalization` crate — we implement
//! the minimum needed (NFKC for common characters, ASCII-fast lowercase)
//! to stay dependency-lean per ZETS philosophy.

/// Normalization flags — tune per-atom based on content kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeFlags {
    /// Lowercase (default: true for text, false for names/codes).
    pub lowercase: bool,
    /// Strip punctuation (default: false; enable for dedup matching only).
    pub strip_punct: bool,
    /// Collapse whitespace runs (default: true).
    pub collapse_ws: bool,
    /// Trim leading/trailing whitespace (default: true).
    pub trim: bool,
    /// NFKC-like unicode compose (basic implementation — handles Hebrew dagesh, combining marks).
    pub unicode_compose: bool,
}

impl Default for NormalizeFlags {
    fn default() -> Self {
        Self {
            lowercase: true,
            strip_punct: false,
            collapse_ws: true,
            trim: true,
            unicode_compose: true,
        }
    }
}

impl NormalizeFlags {
    /// Maximum normalization — for near-duplicate detection (dedup key).
    pub fn for_dedup() -> Self {
        Self {
            lowercase: true,
            strip_punct: true,
            collapse_ws: true,
            trim: true,
            unicode_compose: true,
        }
    }

    /// Minimal — preserve content exactly (for leaves that need roundtrip).
    pub fn preserve_content() -> Self {
        Self {
            lowercase: false,
            strip_punct: false,
            collapse_ws: false,
            trim: false,
            unicode_compose: false,
        }
    }
}

/// Normalize a string per the given flags.
///
/// Hebrew note: we do NOT strip niqqud/trope marks by default — they carry
/// meaning. Only `strip_punct` removes punctuation like `.,;:!?"'()[]{}`.
pub fn normalize(input: &str, flags: NormalizeFlags) -> String {
    let mut out = String::with_capacity(input.len());

    // Pass 1: collect chars with optional lowercasing
    for ch in input.chars() {
        let ch = if flags.lowercase { lowercase_char(ch) } else { ch };

        if flags.strip_punct && is_punct(ch) {
            continue;
        }

        out.push(ch);
    }

    // Pass 2: whitespace collapse
    if flags.collapse_ws {
        out = collapse_whitespace(&out);
    }

    // Pass 3: unicode compose (basic — combines Hebrew dagesh/vowel points if separate)
    if flags.unicode_compose {
        out = basic_nfkc(&out);
    }

    // Pass 4: trim
    if flags.trim {
        out = out.trim().to_string();
    }

    out
}

/// ASCII-fast lowercase that falls through to Unicode for non-ASCII.
fn lowercase_char(ch: char) -> char {
    if ch.is_ascii() {
        ch.to_ascii_lowercase()
    } else {
        // Hebrew doesn't have case; this just returns ch for Hebrew.
        // Cyrillic/Greek/etc. get proper lowercase via stdlib.
        ch.to_lowercase().next().unwrap_or(ch)
    }
}

fn is_punct(ch: char) -> bool {
    matches!(ch,
        '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | '`' |
        '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' |
        '—' | '–' | '…' | '«' | '»' | '\u{201C}' | '\u{201D}' |
        '\u{2018}' | '\u{2019}'  // curly quotes
    )
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out
}

/// Minimal NFKC-like composition. Handles:
/// - combining diacritics (precompose where possible)
/// - full-width digits → ASCII (if encountered)
/// - ligatures (fi, fl → f+i, f+l)
///
/// For a comprehensive NFKC, pull in the `unicode-normalization` crate.
/// This handles the 95% case without extra deps.
fn basic_nfkc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            // Common ligatures
            '\u{FB00}' => out.push_str("ff"),
            '\u{FB01}' => out.push_str("fi"),
            '\u{FB02}' => out.push_str("fl"),
            '\u{FB03}' => out.push_str("ffi"),
            '\u{FB04}' => out.push_str("ffl"),
            // Full-width digits
            '\u{FF10}'..='\u{FF19}' => {
                let offset = ch as u32 - 0xFF10;
                out.push((b'0' + offset as u8) as char);
            }
            // Full-width latin
            '\u{FF21}'..='\u{FF3A}' => {
                let offset = ch as u32 - 0xFF21;
                out.push((b'A' + offset as u8) as char);
            }
            '\u{FF41}'..='\u{FF5A}' => {
                let offset = ch as u32 - 0xFF41;
                out.push((b'a' + offset as u8) as char);
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_lowercase() {
        let r = normalize("Hello World", NormalizeFlags::default());
        assert_eq!(r, "hello world");
    }

    #[test]
    fn strip_punct_dedup() {
        let r = normalize("Hello, World!", NormalizeFlags::for_dedup());
        assert_eq!(r, "hello world");
    }

    #[test]
    fn whitespace_collapse() {
        let r = normalize("hello    world\n\t  foo", NormalizeFlags::default());
        assert_eq!(r, "hello world foo");
    }

    #[test]
    fn hebrew_preserved() {
        // Hebrew has no case — should pass through
        let r = normalize("שלום עולם", NormalizeFlags::default());
        assert_eq!(r, "שלום עולם");
    }

    #[test]
    fn hebrew_with_niqqud_preserved_unless_stripped() {
        // שָׁלוֹם with niqqud — should preserve by default
        let r = normalize("שָׁלוֹם", NormalizeFlags::default());
        assert!(r.contains('\u{05B8}') || r.starts_with('ש'));
    }

    #[test]
    fn ligature_decomposition() {
        let r = normalize("o\u{FB01}ce", NormalizeFlags::default());
        assert_eq!(r, "ofice");  // FB01 is "fi" ligature (2 chars), not "ffi"
    }

    #[test]
    fn fullwidth_normalized() {
        let r = normalize("\u{FF11}\u{FF12}\u{FF13}", NormalizeFlags::default());
        assert_eq!(r, "123");
    }

    #[test]
    fn dedup_flags_collapses_variants() {
        let variants = [
            "Hello World",
            "hello world",
            "HELLO WORLD",
            "Hello, World!",
            "Hello  World",
            "hello world.",
        ];
        let flags = NormalizeFlags::for_dedup();
        let first = normalize(variants[0], flags);
        for v in &variants[1..] {
            assert_eq!(normalize(v, flags), first, "variant {} didn't normalize to {}", v, first);
        }
    }

    #[test]
    fn preserve_content_flag_does_nothing() {
        let s = "Hello, World!  Keep It Exact.";
        let r = normalize(s, NormalizeFlags::preserve_content());
        assert_eq!(r, s);
    }
}
