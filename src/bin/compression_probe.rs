//! Compression experiment: recursive encode/decode with base-37 polynomial.
//!
//! The hypothesis (idea from Idan): run N recursion rounds of "forward
//! squeeze / backward expand" over a text, using GF(37) arithmetic and
//! polynomial reductions. Start with N=49, test how many rounds actually
//! help vs hurt size, and verify round-trip correctness.
//!
//! This is EXPERIMENTAL — results determine whether the approach has legs
//! or whether we stick with Huffman/front-coding for real pack compression.

use std::time::Instant;

/// Run `rounds` encode rounds, then `rounds` decode rounds, return
/// (encoded_len, decoded_text, round_trip_ok).
fn experiment_round(input: &str, rounds: usize) -> (usize, String, bool) {
    let mut buf: Vec<u8> = input.as_bytes().to_vec();

    // Forward pass: `rounds` iterations of a reversible transform.
    // Each round XORs with a GF(37)-derived keystream and folds adjacent bytes.
    for r in 0..rounds {
        // Derive a keystream byte from round index via GF(37).
        // 37 is prime; r^3 mod 37 cycles through many values.
        let seed = (((r as u64) * (r as u64) * (r as u64)) % 37) as u8;
        for i in 0..buf.len() {
            // Mix byte with its predecessor (reversible fold)
            let prev = if i == 0 { seed } else { buf[i - 1].wrapping_add(seed) };
            buf[i] = buf[i].wrapping_add(prev);
        }
    }

    let encoded_len = buf.len();

    // Backward pass: invert the folds in reverse round order.
    for r in (0..rounds).rev() {
        let seed = (((r as u64) * (r as u64) * (r as u64)) % 37) as u8;
        for i in (0..buf.len()).rev() {
            let prev = if i == 0 { seed } else { buf[i - 1].wrapping_add(seed) };
            buf[i] = buf[i].wrapping_sub(prev);
        }
    }

    let decoded = String::from_utf8(buf).unwrap_or_default();
    let ok = decoded == input;
    (encoded_len, decoded, ok)
}

/// Count information per byte via Shannon entropy (0..=8 bits/byte).
fn shannon_entropy(data: &[u8]) -> f64 {
    let mut counts = [0u32; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let n = data.len() as f64;
    if n == 0.0 { return 0.0; }
    let mut h = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * p.log2();
        }
    }
    h
}

/// Huffman-like byte-level length estimate: sum(-p * log2(p)) * n / 8 bytes.
fn huffman_estimate(data: &[u8]) -> usize {
    let h = shannon_entropy(data);
    ((h * data.len() as f64) / 8.0).ceil() as usize
}

/// Also probe gzip/zstd via system tools for comparison.
fn system_compress(data: &[u8], cmd: &str) -> Option<usize> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(cmd)
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
    // Read Hebrew vocabulary TSV as our test corpus (realistic Hebrew text)
    let corpus = std::fs::read_to_string("data/hebrew/core_vocabulary.tsv")
        .expect("read vocabulary");
    let n = corpus.len();

    println!("Compression experiment on Hebrew vocabulary ({n} bytes)");
    println!("==============================================================\n");

    // Baselines
    let entropy = shannon_entropy(corpus.as_bytes());
    let huff_est = huffman_estimate(corpus.as_bytes());
    let gzip_size = system_compress(corpus.as_bytes(), "gzip").unwrap_or(0);
    let zstd_size = system_compress(corpus.as_bytes(), "zstd").unwrap_or(0);

    println!("Baselines:");
    println!("  Raw:                 {n} bytes (100.00%)");
    println!("  Shannon entropy:     {entropy:.3} bits/byte (theoretical minimum)");
    println!("  Huffman estimate:    {huff_est} bytes ({:.2}%)", 100.0 * huff_est as f64 / n as f64);
    println!("  gzip -9:             {gzip_size} bytes ({:.2}%)", 100.0 * gzip_size as f64 / n as f64);
    println!("  zstd -9:             {zstd_size} bytes ({:.2}%)", 100.0 * zstd_size as f64 / n as f64);
    println!();

    // Test the idea: recursive rounds via GF(37)
    println!("Recursive GF(37) transform (hypothesis: N rounds squeeze size):");
    println!("  {:>5}  {:>10}  {:>8}  {:>10}  {:>12}", "N", "enc_size", "pct", "entropy", "round_trip");
    for &rounds in &[1usize, 3, 7, 13, 21, 37, 49, 73, 100] {
        let t0 = Instant::now();
        let (enc_len, dec, ok) = experiment_round(&corpus, rounds);
        let dt = t0.elapsed();
        let pct = 100.0 * enc_len as f64 / n as f64;
        // Measure entropy of encoded output (lower = more redundant = compressible)
        let enc_entropy = {
            // Re-run forward only to inspect the intermediate bytes
            let mut buf: Vec<u8> = corpus.as_bytes().to_vec();
            for r in 0..rounds {
                let seed = (((r as u64) * (r as u64) * (r as u64)) % 37) as u8;
                for i in 0..buf.len() {
                    let prev = if i == 0 { seed } else { buf[i-1].wrapping_add(seed) };
                    buf[i] = buf[i].wrapping_add(prev);
                }
            }
            shannon_entropy(&buf)
        };
        println!("  {:>5}  {:>10}  {:>7.2}%  {:>10.3}  {:>12}  [{:?}]",
            rounds, enc_len, pct, enc_entropy,
            if ok { "ok" } else { "BROKEN" },
            dt);
        if !ok {
            println!("    FIRST MISMATCH: decoded[0..60] = {:?}", &dec.chars().take(60).collect::<String>());
        }
    }

    println!();
    println!("Interpretation:");
    println!("  - enc_size doesn't shrink because our transform is byte-length preserving.");
    println!("    A recursive transform alone CANNOT compress — information theory.");
    println!("  - What it CAN do: change entropy. If entropy drops after N rounds, a");
    println!("    downstream Huffman/zstd would compress better.");
    println!("  - Compare 'entropy' column to raw entropy ({entropy:.3}) to see if the transform helps.");
    println!();

    // Final test: chain our transform + gzip, compare to gzip alone
    println!("Chained compression test (N rounds + gzip):");
    for &rounds in &[0usize, 7, 37, 49] {
        let mut buf: Vec<u8> = corpus.as_bytes().to_vec();
        for r in 0..rounds {
            let seed = (((r as u64) * (r as u64) * (r as u64)) % 37) as u8;
            for i in 0..buf.len() {
                let prev = if i == 0 { seed } else { buf[i-1].wrapping_add(seed) };
                buf[i] = buf[i].wrapping_add(prev);
            }
        }
        let gz = system_compress(&buf, "gzip").unwrap_or(0);
        println!("  rounds={:>3}  gzip_after_transform = {gz} bytes ({:.2}%)",
            rounds, 100.0 * gz as f64 / n as f64);
    }

    println!();
    println!("Conclusion (bottom line for Idan):");
    if huff_est < gzip_size {
        println!("  Huffman-ish (entropy-based) theoretical: {huff_est}b");
    }
    println!("  gzip    actual: {gzip_size}b");
    println!("  zstd    actual: {zstd_size}b");
    println!("  The transform is reversible but does not compress by itself.");
    println!("  A learned/trained entropy coder is the right path — not recursion count.");
}
