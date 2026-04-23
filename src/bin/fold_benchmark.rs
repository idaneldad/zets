//! fold_benchmark — realistic measurement of SERIALIZED sizes
//! (what we'd actually write to disk, not in-memory hashmap overhead).

use zets::fold::vocab::{Vocab, TokenContent};
use zets::fold::vocab_index::VocabIndex;
use zets::fold::bpe::{tokenize_words, bpe_fold, BpeConfig};
use zets::fold::walk::unfold_stream;
use zets::fold::FoldId;
use zets::fold::MAX_FOLD_DEPTH;
use std::env;
use std::fs;
use std::time::Instant;

fn varint_encode(stream: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(stream.len());
    for &v in stream {
        let mut x = v;
        loop {
            let mut byte = (x & 0x7F) as u8;
            x >>= 7;
            if x != 0 { byte |= 0x80; out.push(byte); }
            else { out.push(byte); break; }
        }
    }
    out
}

/// Compute the SERIALIZED size of the vocab if written to disk.
/// Packed format: [kind_tag, content] for each token indexed by LocalIdx.
/// - Leaf:  [0x00, u16_len, bytes...]
/// - Merge: [0x01, u32_left_idx_varint, u32_right_idx_varint]
/// No hashmap, no FoldId in serialized form — the LocalIdx IS the key (array index).
/// FoldId is recomputed on load from content (for verification) or omitted entirely
/// (trusting file integrity via outer AES-GCM).
fn serialized_vocab_size(vocab: &Vocab, index: &VocabIndex) -> u64 {
    let mut total: u64 = 0;
    for i in 0..index.len() {
        let Some(fid) = index.resolve(i as u32) else { continue; };
        let Some(content) = vocab.lookup(fid) else { continue; };
        total += 1;  // kind tag
        match content {
            TokenContent::Leaf(bytes) => {
                total += 2 + bytes.len() as u64;  // u16 length + content
            }
            TokenContent::Merge(left, right) => {
                // Two LocalIdx as varints (avg 2-3 bytes each)
                let l_idx = index.find(*left).unwrap_or(0);
                let r_idx = index.find(*right).unwrap_or(0);
                total += varint_size(l_idx) + varint_size(r_idx);
            }
        }
    }
    total
}

fn varint_size(v: u32) -> u64 {
    let mut x = v; let mut n = 0u64;
    loop { n += 1; x >>= 7; if x == 0 { return n; } }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: fold_benchmark <path> [max_merges]");
        std::process::exit(1);
    }
    let path = &args[1];
    let max_merges: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10_000);

    let text = fs::read_to_string(path).expect("read file");
    let raw_bytes = text.len() as u64;
    println!("Input: {} ({} bytes, {} chars, {} words)",
             path, raw_bytes, text.chars().count(), text.split_whitespace().count());

    let config = BpeConfig {
        max_merges, min_frequency: 2, max_depth: MAX_FOLD_DEPTH,
    };

    let mut vocab = Vocab::new();
    let mut index = VocabIndex::new();

    println!("Folding (word-level BPE, max_merges={})...", max_merges);
    let t0 = Instant::now();
    let mut tokens = tokenize_words(&text, &mut vocab);
    let merges = bpe_fold(&mut tokens, &mut vocab, &config);
    let fold_elapsed = t0.elapsed();

    // u32 stream + varint-encoded
    let u32_stream: Vec<u32> = tokens.iter().map(|&fid| index.get_or_assign(fid)).collect();
    let varint_stream = varint_encode(&u32_stream);

    // SERIALIZED sizes (what actually goes to disk)
    let stream_bytes = varint_stream.len() as u64;
    let vocab_serialized = serialized_vocab_size(&vocab, &index);
    let total_disk = stream_bytes + vocab_serialized;

    // Amortization: if vocab is shared across many documents, its overhead shrinks per-doc
    let vocab_per_100x_corpus = (vocab_serialized + stream_bytes * 100) / 100;

    println!();
    println!("═══════════════════ SERIALIZED (on-disk) sizes ═══════════════════");
    println!("  Merges:                {}", merges);
    println!("  Unique tokens:         {}", index.len());
    println!("  Stream (varint):       {} bytes ({:.2}× of raw)",
             stream_bytes, stream_bytes as f64 / raw_bytes as f64);
    println!("  Vocab serialized:      {} bytes", vocab_serialized);
    println!("  Total on disk:         {} bytes", total_disk);
    println!("  RATIO (single file):   {:.2}×",
             raw_bytes as f64 / total_disk as f64);
    println!();
    println!("  Amortized over 100×:   {:.2}× (if same vocab reused on 100 files)",
             (raw_bytes * 100) as f64 / ((vocab_serialized + stream_bytes * 100) as f64));
    println!("  Amortized over 1000×:  {:.2}×",
             (raw_bytes * 1000) as f64 / ((vocab_serialized + stream_bytes * 1000) as f64));
    println!();
    println!("  Fold time:             {:?} ({:.1} MB/s)",
             fold_elapsed, raw_bytes as f64 / 1e6 / fold_elapsed.as_secs_f64());

    // Verify
    print!("\nVerifying lossless content... ");
    let recovered = unfold_stream(&vocab, &tokens, MAX_FOLD_DEPTH).expect("unfold");
    if recovered == text.as_bytes() {
        println!("OK — {} bytes match byte-for-byte", recovered.len());
    } else {
        println!("FAIL");
        // Find first diff
        let orig = text.as_bytes();
        let diff_at = recovered.iter().zip(orig.iter()).position(|(a, b)| a != b);
        if let Some(pos) = diff_at {
            let start = pos.saturating_sub(20);
            let end_o = (pos + 20).min(orig.len());
            let end_r = (pos + 20).min(recovered.len());
            eprintln!("  First diff at byte {}", pos);
            eprintln!("  Original [{}..{}]: {:?}", start, end_o,
                      String::from_utf8_lossy(&orig[start..end_o]));
            eprintln!("  Recovered[{}..{}]: {:?}", start, end_r,
                      String::from_utf8_lossy(&recovered[start..end_r]));
        }
        eprintln!("  orig len = {}, recovered len = {}", orig.len(), recovered.len());
        std::process::exit(1);
    }
}
