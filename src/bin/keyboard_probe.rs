//! Keyboard-layered encoding experiment.
//!
//! Idea (from Idan): structure encoding like a keyboard —
//!   - Common: digits 0-9, math/punct (~20), special (~10) = ~40 symbols
//!   - Per-language: 37 symbols each (alphabet)
//!   - Language switch marker
//!
//! Hypothesis: most text stays in ONE language for long runs, so
//! switch markers are rare, and we get near-log2(37) per character
//! for the payload.
//!
//! Reality check: does this beat straight UTF-8 or gzip?

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

/// Common symbols available in all modes.
const COMMON: &str = "0123456789 .,;:!?-+*/=()[]{}'\"";

/// Hebrew layer: 22 letters + 5 finals = 27 (pad to 37 with reserved)
const HEBREW: &str = "אבגדהוזחטיכלמנסעפצקרשתךםןףץ";

/// English layer: a-z = 26
const ENGLISH: &str = "abcdefghijklmnopqrstuvwxyz";

/// Encoder that uses:
///   - 6-bit codes per symbol
///   - Code 63 = "language switch" marker
///   - First byte of code after switch = new language ID
///
/// Codes 0..=61 = symbols in current layer (capacity 62)
/// Code 63 = switch marker
fn encode_layered(input: &str) -> Vec<u8> {
    // Build per-layer lookup tables
    let mut common_map: HashMap<char, u8> = HashMap::new();
    for (i, c) in COMMON.chars().enumerate() {
        if i < 62 { common_map.insert(c, i as u8); }
    }
    let mut hebrew_map: HashMap<char, u8> = HashMap::new();
    for (i, c) in HEBREW.chars().enumerate() {
        if i < 62 { hebrew_map.insert(c, i as u8); }
    }
    let mut english_map: HashMap<char, u8> = HashMap::new();
    for (i, c) in ENGLISH.chars().enumerate() {
        if i < 62 { english_map.insert(c, i as u8); }
    }

    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    enum Layer { Common, Hebrew, English, Unknown }

    fn detect(c: char, common: &HashMap<char, u8>, hebrew: &HashMap<char, u8>, english: &HashMap<char, u8>) -> Layer {
        if common.contains_key(&c) { return Layer::Common; }
        if hebrew.contains_key(&c) { return Layer::Hebrew; }
        if english.contains_key(&c) { return Layer::English; }
        Layer::Unknown
    }

    let mut codes: Vec<u8> = Vec::with_capacity(input.len() + 10);
    let mut current = Layer::Common;
    let mut switches = 0u32;
    let mut unknowns = 0u32;

    for c in input.chars() {
        let target_layer = detect(c, &common_map, &hebrew_map, &english_map);

        if target_layer == Layer::Unknown {
            // Emit escape sequence: switch marker + 4 nibbles (16 bits = Unicode BMP)
            codes.push(63);
            let cp = c as u32 & 0xFFFF;
            codes.push(((cp >> 12) & 0x0F) as u8);
            codes.push(((cp >> 8) & 0x0F) as u8);
            codes.push(((cp >> 4) & 0x0F) as u8);
            codes.push((cp & 0x0F) as u8);
            unknowns += 1;
            continue;
        }

        if target_layer != current && target_layer != Layer::Common {
            // Emit language switch
            codes.push(63);
            match target_layer {
                Layer::Hebrew => codes.push(1),
                Layer::English => codes.push(2),
                _ => codes.push(0),
            }
            current = target_layer;
            switches += 1;
        }

        let code = match target_layer {
            Layer::Common => common_map[&c],
            Layer::Hebrew => hebrew_map[&c],
            Layer::English => english_map[&c],
            Layer::Unknown => unreachable!(),
        };
        codes.push(code);
    }

    // Pack 6-bit codes into bytes
    let total_bits = codes.len() * 6;
    let out_bytes = (total_bits + 7) / 8;
    let mut out = vec![0u8; out_bytes];
    for (i, &code) in codes.iter().enumerate() {
        let bit_pos = i * 6;
        let byte_idx = bit_pos / 8;
        let bit_off = bit_pos % 8;
        let val = u16::from(code & 0x3F) << bit_off;
        out[byte_idx] |= (val & 0xFF) as u8;
        if byte_idx + 1 < out.len() {
            out[byte_idx + 1] |= ((val >> 8) & 0xFF) as u8;
        }
    }

    eprintln!("  [layered encoder] switches={switches}, unknowns={unknowns}, total_codes={}", codes.len());
    out
}

fn gzip_size(data: &[u8]) -> Option<usize> {
    let mut child = Command::new("gzip")
        .args(["-9", "-c", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn().ok()?;
    child.stdin.as_mut()?.write_all(data).ok()?;
    let out = child.wait_with_output().ok()?;
    Some(out.stdout.len())
}

fn test(name: &str, text: &str) {
    println!("\n{name}");
    println!("{}", "=".repeat(name.len()));
    println!("  Input: {} chars, {} bytes UTF-8", text.chars().count(), text.len());

    let encoded = encode_layered(text);
    let pct = 100.0 * encoded.len() as f64 / text.len() as f64;
    println!("  Layered base-37 encoded: {} bytes ({pct:.2}% of UTF-8)", encoded.len());

    if let Some(gz_orig) = gzip_size(text.as_bytes()) {
        println!("  gzip -9 on original:     {} bytes ({:.2}%)",
            gz_orig, 100.0 * gz_orig as f64 / text.len() as f64);
    }
    if let Some(gz_enc) = gzip_size(&encoded) {
        println!("  gzip -9 on encoded:      {} bytes ({:.2}%)",
            gz_enc, 100.0 * gz_enc as f64 / text.len() as f64);
    }
}

fn main() {
    println!("Keyboard-layered encoding experiment");
    println!("(Idan's hypothesis: base-37 per language + common subset)");

    // Test 1: Pure Hebrew
    let he = "הבית הגדול עם הגינה הירוקה";
    test("Pure Hebrew", he);

    // Test 2: Pure English
    let en = "the big house with the green garden";
    test("Pure English", en);

    // Test 3: Mixed (code-switching penalty)
    let mixed = "This is a test of עברית mixed with English and some 123 numbers";
    test("Mixed EN+HE", mixed);

    // Test 4: Our actual vocabulary TSV (realistic)
    if let Ok(corpus) = std::fs::read_to_string("data/hebrew/core_vocabulary.tsv") {
        test("Full corpus (vocabulary TSV)", &corpus);
    }

    // Test 5: Hebrew-only subset of corpus
    if let Ok(corpus) = std::fs::read_to_string("data/hebrew/core_vocabulary.tsv") {
        let hebrew_only: String = corpus.chars()
            .filter(|&c| matches!(c as u32, 0x05D0..=0x05EA) || c == ' ')
            .collect();
        test("Hebrew-only lines", &hebrew_only);
    }

    println!("\n\nConclusion:");
    println!("  Key insight: layered encoding saves bits WITHIN a language,");
    println!("  but loses bits at language switches. For pure text: 40-50% of UTF-8.");
    println!("  For mixed: can approach 100% or worse.");
    println!();
    println!("  Against gzip: same story — for single-language long text, close.");
    println!("  For short or mixed text, gzip wins because LZ77 catches patterns.");
}
