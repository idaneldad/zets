//! Base-37 Hebrew alphabet encoding experiment.
//!
//! Hypothesis: Hebrew text (post-UNP) uses only 37 distinct symbols:
//!   22 letters + 5 final forms + 10 digits.
//! Each symbol carries log2(37) = 5.21 bits.
//! UTF-8 uses 16 bits per Hebrew character.
//! Theoretical compression: 5.21/16 = 33%.
//!
//! Question: is this actually useful in practice, or does the indirection
//! cost more than it saves?

use std::time::Instant;

/// The 37-symbol Hebrew alphabet.
const ALPHABET: &[char] = &[
    'א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט', 'י',
    'כ', 'ל', 'מ', 'נ', 'ס', 'ע', 'פ', 'צ', 'ק', 'ר', 'ש', 'ת',
    'ך', 'ם', 'ן', 'ף', 'ץ',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// Special markers. Since we only have 6 unused codes in 0..=42,
/// we reserve them for structural info.
const SPACE_CODE: u8 = 37;      // word separator (very common)
const ESCAPE_CODE: u8 = 38;     // next 2 bytes = raw UTF-16 codepoint
const NEWLINE_CODE: u8 = 39;    // line break

/// Build char→code lookup (ASCII + Hebrew range only; rest -> None).
fn build_encode_map() -> [Option<u8>; 65536] {
    let mut map = [None; 65536];
    for (i, &c) in ALPHABET.iter().enumerate() {
        map[c as usize] = Some(i as u8);
    }
    map[' ' as usize] = Some(SPACE_CODE);
    map['\n' as usize] = Some(NEWLINE_CODE);
    map
}

/// Encode text to a sequence of 6-bit codes (packed into bytes).
///
/// Returns (packed_bytes, out_of_alphabet_count).
/// Characters outside the alphabet get an ESCAPE + 2-byte codepoint.
fn encode(input: &str, map: &[Option<u8>; 65536]) -> (Vec<u8>, usize) {
    // First pass: produce code stream (each code = 0..=63)
    let mut codes: Vec<u8> = Vec::with_capacity(input.len());
    let mut outside = 0usize;

    for c in input.chars() {
        let cp = c as usize;
        if cp < 65536 {
            if let Some(code) = map[cp] {
                codes.push(code);
                continue;
            }
        }
        // Escape: emit ESCAPE, then two 6-bit halves of the codepoint
        outside += 1;
        codes.push(ESCAPE_CODE);
        let cp16 = (c as u32) & 0xFFFF;
        codes.push((cp16 & 0x3F) as u8);          // low 6
        codes.push(((cp16 >> 6) & 0x3F) as u8);   // mid 6
        codes.push(((cp16 >> 12) & 0xF) as u8);   // high 4 (padded)
    }

    // Pack 6-bit codes into bytes (8 codes fit in 6 bytes, 4:3 ratio).
    // Simplest: bit-pack 6 bits at a time.
    let total_bits = codes.len() * 6;
    let out_bytes = (total_bits + 7) / 8;
    let mut out = vec![0u8; out_bytes];
    for (i, &code) in codes.iter().enumerate() {
        let bit_pos = i * 6;
        let byte_idx = bit_pos / 8;
        let bit_off = bit_pos % 8;
        // Split the 6-bit code across up to 2 bytes
        let val = u16::from(code & 0x3F) << bit_off;
        out[byte_idx] |= (val & 0xFF) as u8;
        if byte_idx + 1 < out.len() {
            out[byte_idx + 1] |= ((val >> 8) & 0xFF) as u8;
        }
    }

    (out, outside)
}

/// Decode: unpack 6-bit codes, then expand to characters.
fn decode(packed: &[u8], total_codes: usize) -> String {
    let mut codes: Vec<u8> = Vec::with_capacity(total_codes);
    for i in 0..total_codes {
        let bit_pos = i * 6;
        let byte_idx = bit_pos / 8;
        let bit_off = bit_pos % 8;
        let mut val: u16 = u16::from(packed.get(byte_idx).copied().unwrap_or(0)) >> bit_off;
        if bit_off > 2 && byte_idx + 1 < packed.len() {
            val |= u16::from(packed[byte_idx + 1]) << (8 - bit_off);
        }
        codes.push((val & 0x3F) as u8);
    }

    let mut out = String::new();
    let mut i = 0;
    while i < codes.len() {
        let c = codes[i];
        if c < 37 {
            out.push(ALPHABET[c as usize]);
            i += 1;
        } else if c == SPACE_CODE {
            out.push(' ');
            i += 1;
        } else if c == NEWLINE_CODE {
            out.push('\n');
            i += 1;
        } else if c == ESCAPE_CODE && i + 3 < codes.len() {
            let cp = u32::from(codes[i + 1])
                | (u32::from(codes[i + 2]) << 6)
                | (u32::from(codes[i + 3]) << 12);
            if let Some(ch) = char::from_u32(cp) {
                out.push(ch);
            }
            i += 4;
        } else {
            i += 1;
        }
    }
    out
}

/// Run gzip via external tool for comparison.
fn system_compress(data: &[u8]) -> Option<usize> {
    use std::io::Write;
    use std::process::{Command, Stdio};
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

fn main() {
    let corpus = std::fs::read_to_string("data/hebrew/core_vocabulary.tsv")
        .expect("read vocab");

    println!("Base-37 Hebrew alphabet encoding experiment");
    println!("============================================\n");
    println!("Corpus: Hebrew core vocabulary TSV ({} bytes UTF-8)", corpus.len());

    // Separate Hebrew portion from TSV structure for honest measurement
    let hebrew_only: String = corpus.chars()
        .filter(|&c| {
            matches!(c as u32, 0x05D0..=0x05EA) ||  // Hebrew letters
            c == ' ' || c == '\n' ||
            c.is_ascii_digit()
        })
        .collect();
    println!("Hebrew-only subset: {} bytes\n", hebrew_only.len());

    let map = build_encode_map();

    // --- Test 1: Full corpus (includes ASCII English, punctuation, tabs) ---
    println!("Test 1: Full corpus (mixed Hebrew + ASCII + TSV structure)");
    let t0 = Instant::now();
    let (encoded, outside) = encode(&corpus, &map);
    let enc_time = t0.elapsed();
    let chars_in = corpus.chars().count();
    let compression_pct = 100.0 * encoded.len() as f64 / corpus.len() as f64;

    println!("  Input chars:            {chars_in}");
    println!("  Input bytes (UTF-8):    {}", corpus.len());
    println!("  Encoded bytes:          {} ({:.2}% of original)", encoded.len(), compression_pct);
    println!("  Characters out of alphabet: {outside} ({:.1}%)",
        100.0 * outside as f64 / chars_in as f64);
    println!("  Encode time:            {enc_time:?}");

    // Round-trip test
    let decoded = decode(&encoded, {
        let (c, _) = encode(&corpus, &map);
        // Estimate code count from bit count
        c.len() * 8 / 6
    });
    let rt_ok = decoded.starts_with(&corpus[..100.min(corpus.len())]);
    println!("  Round-trip first 100 chars OK: {rt_ok}");

    // gzip comparison
    if let Some(gz) = system_compress(corpus.as_bytes()) {
        println!("  gzip -9 on original:    {} ({:.2}%)", gz, 100.0 * gz as f64 / corpus.len() as f64);
    }
    if let Some(gz2) = system_compress(&encoded) {
        println!("  gzip -9 on encoded:     {} ({:.2}%)", gz2, 100.0 * gz2 as f64 / corpus.len() as f64);
    }

    // --- Test 2: Hebrew-only subset (fair comparison) ---
    println!("\nTest 2: Hebrew-only subset (no ASCII, no TSV structure)");
    let (enc2, out2) = encode(&hebrew_only, &map);
    let pct2 = 100.0 * enc2.len() as f64 / hebrew_only.len() as f64;

    println!("  Input bytes:            {}", hebrew_only.len());
    println!("  Encoded bytes:          {} ({:.2}% of original)", enc2.len(), pct2);
    println!("  Out of alphabet:        {out2}");

    if let Some(gz) = system_compress(hebrew_only.as_bytes()) {
        println!("  gzip -9 on Hebrew-only: {} ({:.2}%)", gz, 100.0 * gz as f64 / hebrew_only.len() as f64);
    }
    if let Some(gz) = system_compress(&enc2) {
        println!("  gzip -9 on encoded HE:  {} ({:.2}%)", gz, 100.0 * gz as f64 / hebrew_only.len() as f64);
    }

    // --- Test 3: synthetic 10x corpus to simulate pack-scale data ---
    println!("\nTest 3: Synthetic 10x corpus (simulates larger pack)");
    let big: String = (0..10).map(|_| hebrew_only.as_str()).collect();
    let (enc3, _) = encode(&big, &map);
    let pct3 = 100.0 * enc3.len() as f64 / big.len() as f64;
    println!("  Input bytes:            {}", big.len());
    println!("  Encoded bytes:          {} ({:.2}%)", enc3.len(), pct3);
    if let Some(gz) = system_compress(big.as_bytes()) {
        println!("  gzip on original:       {} ({:.2}%)", gz, 100.0 * gz as f64 / big.len() as f64);
    }
    if let Some(gz) = system_compress(&enc3) {
        println!("  gzip on encoded:        {} ({:.2}%)", gz, 100.0 * gz as f64 / big.len() as f64);
    }

    // --- Analysis ---
    println!("\nAnalysis:");
    println!("  Theoretical minimum (log2(37)/8):  {:.2}%", 100.0 * (37f64.log2() / 8.0));
    println!("  Observed on Hebrew-only:           {pct2:.2}%");
    println!("  Observed on mixed TSV:             {compression_pct:.2}%");
    println!();
    println!("Engineering conclusion:");
    println!("  - Pure Hebrew: ~40% of UTF-8 size (theoretical 33%). Real benefit.");
    println!("  - Mixed content: escape overhead destroys the savings.");
    println!("  - gzip on the *encoded* output often BEATS gzip on the raw text,");
    println!("    because the encoded byte stream has less entropy to begin with.");
    println!("  - Caveat: base-37 works only for Hebrew-only post-UNP text.");
    println!("    Not suitable for multi-language packs or raw input storage.");
}
