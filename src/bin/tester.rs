//! ZETS tester — benchmarks, evaluation suite, and learning demos.
//!
//! This binary is for **diagnostics and performance measurement**, kept
//! separate from the main `zets` CLI so production builds don't ship
//! benchmark code.
//!
//! # Commands
//!
//! - `test-vectors`       run UNP ground-truth vectors
//! - `bench-edges N`      EdgeStore push/read/iterate benchmark
//! - `bench-bloom N`      BloomFilter insert/check benchmark
//! - `bench-serialize N`  EdgeStore write/read disk roundtrip
//! - `bench-parallel N`   parallel UNP normalize (std::thread::scope)
//! - `demo-document`      small document with word-sequence + citations
//! - `demo-homograph`     verify "gift"[en] vs "gift"[de] produce different hashes
//! - `evaluate`           run full Week-2 evaluation suite and print summary

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process;
use std::time::Instant;

use zets::edge_store::{Edge, EdgeStore, Relation};
use zets::bloom::BloomFilter;
use zets::{document, meta_graph, unp, LangCode, SynsetId};
use zets::{TEST_VECTORS, TEST_VECTOR_COUNT};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }
    let cmd = args[1].as_str();
    let rest = &args[2..];

    match cmd {
        "test-vectors"    => run_test_vectors(),
        "bench-edges"     => bench_edges(parse_count(rest, 100_000)),
        "bench-bloom"     => bench_bloom(parse_count(rest, 100_000)),
        "bench-serialize" => bench_serialize(parse_count(rest, 100_000)),
        "bench-parallel"  => bench_parallel(parse_count(rest, 100_000)),
        "demo-document"   => demo_document(),
        "demo-homograph"  => demo_homograph(),
        "evaluate"        => evaluate_all(),
        _ => {
            eprintln!("Unknown command: {cmd}");
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("ZETS tester — benchmarks, evaluation, demos");
    eprintln!();
    eprintln!("Usage: {program} <command> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  test-vectors              UNP ground-truth vectors");
    eprintln!("  bench-edges <N>           EdgeStore benchmark");
    eprintln!("  bench-bloom <N>           BloomFilter benchmark");
    eprintln!("  bench-serialize <N>       Serialization benchmark");
    eprintln!("  bench-parallel <N>        Parallel UNP normalize");
    eprintln!("  demo-document             Word-sequence + citation demo");
    eprintln!("  demo-homograph            Verify homograph hash fix");
    eprintln!("  evaluate                  Full Week-2 evaluation suite");
}

#[inline]
fn parse_count(rest: &[String], default: usize) -> usize {
    rest.first().and_then(|s| s.parse().ok()).unwrap_or(default)
}

// ============================================================================
// TEST VECTORS
// ============================================================================

fn run_test_vectors() {
    let mut passed = 0;
    let mut failed = 0;
    let mut dict_skipped = 0;
    let mut failures: Vec<String> = Vec::new();

    for (i, v) in TEST_VECTORS.iter().enumerate() {
        if v.category == "needs_dict" {
            dict_skipped += 1;
            continue;
        }
        let lang = match LangCode::parse(v.lang) {
            Some(l) => l,
            None => {
                failures.push(format!("[{i}] bad lang: {}", v.lang));
                failed += 1;
                continue;
            }
        };
        let result = unp::normalize(v.input, lang);
        if result.as_str() == v.expected {
            passed += 1;
        } else {
            failed += 1;
            failures.push(format!("[{i}] {} input={:?} expected={:?} got={:?} ({})",
                v.lang, v.input, v.expected, result.as_str(), v.note));
        }
    }

    println!("ZETS UNP Test Vectors");
    println!("  Total vectors:    {TEST_VECTOR_COUNT}");
    println!("  Week 1-2 applied: {}", passed + failed);
    println!("  Passed:           {passed}");
    println!("  Failed:           {failed}");
    println!("  Dict-deferred:    {dict_skipped} (Week 3+)");

    if !failures.is_empty() {
        println!("\nFailures:");
        for f in failures.iter().take(30) {
            println!("  {f}");
        }
    }
    if failed > 0 {
        process::exit(1);
    }
}

// ============================================================================
// BENCHMARKS
// ============================================================================

fn bench_edges(count: usize) {
    println!("EdgeStore benchmark — {count} edges");

    let t0 = Instant::now();
    let mut store = EdgeStore::with_capacity(count);
    for i in 0..count {
        store.push(Edge {
            source: SynsetId(i as u32 % 500_000),
            target: SynsetId((i as u32 * 31) % 500_000),
            relation: Relation::IsA,
            weight: (i % 8) as u8,
            provenance: (i % 100_000) as u32,
        });
    }
    let push_time = t0.elapsed();

    let t1 = Instant::now();
    let mut sum: u64 = 0;
    for i in 0..count {
        if let Some(e) = store.get(i) {
            sum += u64::from(e.weight);
        }
    }
    let read_time = t1.elapsed();

    let t2 = Instant::now();
    let iter_n = store.iter().count();
    let iter_time = t2.elapsed();

    let bytes_per = store.bytes_used() as f64 / count as f64;
    println!("  Push:        {push_time:>10?}  ({:>12.0} edges/sec)",
        count as f64 / push_time.as_secs_f64());
    println!("  Random read: {read_time:>10?}  ({:>12.0} edges/sec)",
        count as f64 / read_time.as_secs_f64());
    println!("  Iterate:     {iter_time:>10?}  ({:>12.0} edges/sec)",
        count as f64 / iter_time.as_secs_f64());
    println!();
    println!("  Bytes used:  {} ({bytes_per:.2} per edge)", store.bytes_used());
    println!("  Iter count:  {iter_n}");
    println!("  Sum weight:  {sum}");
}

fn bench_bloom(count: usize) {
    println!("BloomFilter benchmark — {count} items");
    let mut filter = BloomFilter::default_size();

    let t0 = Instant::now();
    for i in 0..count {
        filter.insert(format!("synset_{i}").as_bytes());
    }
    let insert_time = t0.elapsed();

    let t1 = Instant::now();
    let mut hits = 0;
    for i in 0..count {
        if filter.might_contain(format!("synset_{i}").as_bytes()) { hits += 1; }
    }
    let pos_time = t1.elapsed();

    let t2 = Instant::now();
    let mut false_pos = 0;
    for i in 0..count {
        if filter.might_contain(format!("notpresent_{i}").as_bytes()) { false_pos += 1; }
    }
    let neg_time = t2.elapsed();

    let fp_rate = false_pos as f64 / count as f64;
    println!("  Insert:    {insert_time:>10?}  ({:>12.0} ops/sec)",
        count as f64 / insert_time.as_secs_f64());
    println!("  Pos check: {pos_time:>10?}  ({:>12.0} ops/sec)",
        count as f64 / pos_time.as_secs_f64());
    println!("  Neg check: {neg_time:>10?}  ({:>12.0} ops/sec)",
        count as f64 / neg_time.as_secs_f64());
    println!();
    println!("  Bloom size:        {} bytes", filter.bytes_used());
    println!("  Bits set:          {} / {}", filter.bits_set(), zets::bloom::DEFAULT_BITS);
    println!("  Saturation:        {:.4}%",
        filter.bits_set() as f64 / zets::bloom::DEFAULT_BITS as f64 * 100.0);
    println!("  Positive hits:     {hits} / {count} (must be {count})");
    println!("  False positives:   {false_pos} / {count} ({:.3}%)", fp_rate * 100.0);
    println!("  Estimated FP rate: {:.4}%", filter.estimated_fp_rate() * 100.0);
}

fn bench_serialize(count: usize) {
    println!("Serialization benchmark — {count} edges");

    let mut store = EdgeStore::with_capacity(count);
    for i in 0..count {
        store.push(Edge {
            source: SynsetId(i as u32 % 500_000),
            target: SynsetId((i as u32 * 31) % 500_000),
            relation: Relation::from_u8((i % 30) as u8).unwrap_or(Relation::IsA),
            weight: (i % 8) as u8,
            provenance: (i % 100_000) as u32,
        });
    }

    let path = "/tmp/zets_bench_edges.bin";
    let t0 = Instant::now();
    {
        let f = File::create(path).expect("create");
        let mut w = BufWriter::new(f);
        store.write_to(&mut w).expect("write");
    }
    let write_time = t0.elapsed();
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let t1 = Instant::now();
    let loaded = {
        let f = File::open(path).expect("open");
        let mut r = BufReader::new(f);
        EdgeStore::read_from(&mut r).expect("read")
    };
    let read_time = t1.elapsed();

    let mut mismatches = 0;
    let step = (count / 100).max(1);
    for i in (0..count).step_by(step) {
        if store.get(i) != loaded.get(i) { mismatches += 1; }
    }

    let wmb = (file_size as f64 / 1_048_576.0) / write_time.as_secs_f64();
    let rmb = (file_size as f64 / 1_048_576.0) / read_time.as_secs_f64();
    println!("  Write to disk:  {write_time:>10?}  ({wmb:>6.1} MB/s)");
    println!("  Read from disk: {read_time:>10?}  ({rmb:>6.1} MB/s)");
    println!();
    println!("  File size:      {file_size} bytes ({:.2} per edge)",
        file_size as f64 / count as f64);
    println!("  Loaded count:   {} (matches: {})", loaded.len(), loaded.len() == count);
    println!("  Sampled mismatches: {mismatches} / 100 (must be 0)");
    let _ = std::fs::remove_file(path);
}

fn bench_parallel(count: usize) {
    use std::thread;

    println!("Parallel UNP normalize benchmark — {count} inputs");

    // Build input vector with cycled strings
    let inputs: Vec<String> = (0..count)
        .map(|i| match i % 4 {
            0 => format!("הבית הגדול {i}"),
            1 => format!("Hello World {i}"),
            2 => format!("כלב וחתול {i}"),
            _ => format!("A sentence with text {i}"),
        })
        .collect();

    // Serial baseline
    let t0 = Instant::now();
    let serial: Vec<_> = inputs.iter().map(|s| {
        let lang = if s.chars().any(|c| ('\u{05D0}'..='\u{05EA}').contains(&c)) {
            LangCode::HEBREW
        } else {
            LangCode::ENGLISH
        };
        unp::normalize(s, lang)
    }).collect();
    let serial_time = t0.elapsed();

    // Parallel with std::thread::scope
    let n_threads = thread::available_parallelism()
        .map(|n| n.get().min(16))
        .unwrap_or(1);
    let chunk_size = inputs.len().div_ceil(n_threads);

    let t1 = Instant::now();
    let mut results: Vec<Option<zets::Canonical>> = (0..inputs.len()).map(|_| None).collect();
    thread::scope(|s| {
        let mut handles = Vec::with_capacity(n_threads);
        for (tid, chunk) in inputs.chunks(chunk_size).enumerate() {
            let handle = s.spawn(move || {
                let start = tid * chunk_size;
                let mut out = Vec::with_capacity(chunk.len());
                for (i, s) in chunk.iter().enumerate() {
                    let lang = if s.chars().any(|c| ('\u{05D0}'..='\u{05EA}').contains(&c)) {
                        LangCode::HEBREW
                    } else {
                        LangCode::ENGLISH
                    };
                    out.push((start + i, unp::normalize(s, lang)));
                }
                out
            });
            handles.push(handle);
        }
        for h in handles {
            for (idx, canon) in h.join().expect("thread panicked") {
                results[idx] = Some(canon);
            }
        }
    });
    let parallel_time = t1.elapsed();

    // Verify same output
    let parallel: Vec<_> = results.into_iter().map(Option::unwrap).collect();
    let mismatches = serial.iter().zip(parallel.iter())
        .filter(|(a, b)| a.as_bytes() != b.as_bytes())
        .count();

    let speedup = serial_time.as_secs_f64() / parallel_time.as_secs_f64();
    let serial_rate = count as f64 / serial_time.as_secs_f64();
    let parallel_rate = count as f64 / parallel_time.as_secs_f64();

    println!("  Serial   ({} thread):   {serial_time:>10?}  ({serial_rate:>12.0} ops/sec)", 1);
    println!("  Parallel ({n_threads} threads):  {parallel_time:>10?}  ({parallel_rate:>12.0} ops/sec)");
    println!();
    println!("  Speedup:    {speedup:.2}x");
    println!("  Mismatches: {mismatches} (must be 0 — parallel result must equal serial)");
}

// ============================================================================
// DEMOS
// ============================================================================

fn demo_document() {
    println!("Document + word-sequence demo");
    println!("=============================\n");

    let mut store = EdgeStore::new_with_meta();
    let mut doc_alloc = document::DocumentIdAllocator::new();

    // Doc 1: "הבית הגדול בגינה"
    let doc1 = doc_alloc.allocate();
    let tokens1 = [
        SynsetId(10_001), // הבית
        SynsetId(10_002), // הגדול
        SynsetId(10_003), // בגינה
    ];
    let e1 = document::add_sequence(&mut store, doc1, &tokens1);
    println!("Doc {}: 3 tokens → {} edges (2 TEXT_NEXT + 3 APPEARS_IN)",
        doc1.0, e1);

    // Doc 2: "הבית ריק"
    let doc2 = doc_alloc.allocate();
    let tokens2 = [
        SynsetId(10_001), // הבית (same as doc1!)
        SynsetId(10_004), // ריק
    ];
    let e2 = document::add_sequence(&mut store, doc2, &tokens2);
    println!("Doc {}: 2 tokens → {} edges", doc2.0, e2);

    // Doc 2 cites Doc 1
    document::add_citation(&mut store, doc2, doc1, 7);
    println!("Doc {} CITES Doc {}", doc2.0, doc1.0);

    // Queries
    println!("\n── Queries ──");
    let token_appearances = store.outgoing(SynsetId(10_001)).iter()
        .filter(|e| e.relation == Relation::AppearsIn).count();
    println!("'הבית' (10_001) appears in {} documents", token_appearances);

    let next_after = document::next_tokens(&store, SynsetId(10_001));
    println!("What follows 'הבית'? {} occurrences:", next_after.len());
    for (pos, tok) in &next_after {
        println!("  at position {pos}: token {}", tok.0);
    }

    let cits = document::citations(&store, doc2);
    println!("Doc {} cites: {:?}", doc2.0, cits.iter().map(|s| s.0).collect::<Vec<_>>());

    let cited = document::cited_by(&store, doc1);
    println!("Doc {} is cited by: {:?}", doc1.0, cited.iter().map(|s| s.0).collect::<Vec<_>>());

    let seq1 = document::reconstruct_sequence(&store, doc1);
    println!("Reconstructed Doc {}: {:?}", doc1.0,
        seq1.iter().map(|(p, s)| (*p, s.0)).collect::<Vec<_>>());

    println!("\nTotal edges in graph: {}", store.len());
}

fn demo_homograph() {
    use zets::ContentKind;

    println!("Homograph hash demo — proving cross-language disambiguation");
    println!("===========================================================\n");

    // "gift" in English = present; in German = poison. Same spelling.
    let en = unp::identify("gift", LangCode::ENGLISH, ContentKind::Sentence);
    let de = unp::identify("gift", LangCode::GERMAN, ContentKind::Sentence);

    let en_hash = match en {
        zets::Identity::Hash128(h) => h,
        _ => unreachable!(),
    };
    let de_hash = match de {
        zets::Identity::Hash128(h) => h,
        _ => unreachable!(),
    };

    println!("'gift' in English (meaning: present):");
    println!("  hash: {}", hex(&en_hash));
    println!();
    println!("'gift' in German (meaning: poison):");
    println!("  hash: {}", hex(&de_hash));
    println!();

    if en_hash == de_hash {
        println!("❌ FAIL: identical hashes — homograph bug not fixed!");
        process::exit(1);
    } else {
        println!("✓ Different hashes — cross-language disambiguation works.");
    }

    // Also check bytes (should be same) vs Canonical equality (should be different)
    let en_canon = unp::normalize("gift", LangCode::ENGLISH);
    let de_canon = unp::normalize("gift", LangCode::GERMAN);
    println!("\nCanonical bytes:");
    println!("  EN: {:?}", en_canon.as_bytes());
    println!("  DE: {:?}", de_canon.as_bytes());
    println!("  Bytes equal:      {}", en_canon.as_bytes() == de_canon.as_bytes());
    println!("  Canonical equal:  {} (should be false — language differs)",
        en_canon == de_canon);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ============================================================================
// EVALUATION SUITE
// ============================================================================

fn evaluate_all() {
    println!("ZETS Week-2 Evaluation Suite");
    println!("============================\n");

    let mut failures: Vec<String> = Vec::new();

    // 1. Test vectors
    print!("[1/6] UNP test vectors ...");
    let mut passed = 0;
    let mut failed = 0;
    for v in TEST_VECTORS {
        if v.category == "needs_dict" { continue; }
        if let Some(lang) = LangCode::parse(v.lang) {
            if unp::normalize(v.input, lang).as_str() == v.expected {
                passed += 1;
            } else {
                failed += 1;
            }
        }
    }
    if failed == 0 {
        println!(" ✓  {passed}/{passed} passed");
    } else {
        println!(" ✗  {passed} passed, {failed} failed");
        failures.push(format!("test vectors: {failed} failed"));
    }

    // 2. Meta-graph
    print!("[2/6] Meta-graph lookup ......");
    let he = meta_graph::by_key("he");
    let is_a = meta_graph::by_key("IS_A");
    if he == Some(SynsetId(10)) && is_a == Some(SynsetId(30)) {
        println!(" ✓  he=10, IS_A=30");
    } else {
        println!(" ✗  he={he:?}, IS_A={is_a:?}");
        failures.push("meta-graph lookup".into());
    }

    // 3. Homograph
    print!("[3/6] Homograph disambiguation");
    let en = unp::identify("gift", LangCode::ENGLISH, zets::ContentKind::Sentence);
    let de = unp::identify("gift", LangCode::GERMAN, zets::ContentKind::Sentence);
    let en_h = if let zets::Identity::Hash128(h) = en { h } else { [0u8;16] };
    let de_h = if let zets::Identity::Hash128(h) = de { h } else { [0u8;16] };
    if en_h != de_h {
        println!(" ✓  hashes differ");
    } else {
        println!(" ✗  hashes identical");
        failures.push("homograph disambiguation".into());
    }

    // 4. Document sequences
    print!("[4/6] Document sequences .....");
    let mut store = EdgeStore::new();
    let mut alloc = document::DocumentIdAllocator::new();
    let doc = alloc.allocate();
    let toks = [SynsetId(10_001), SynsetId(10_002), SynsetId(10_003)];
    document::add_sequence(&mut store, doc, &toks);
    let recon = document::reconstruct_sequence(&store, doc);
    if recon.len() == 3 && recon[0].1 == SynsetId(10_001) && recon[2].1 == SynsetId(10_003) {
        println!(" ✓  3 tokens, order preserved");
    } else {
        println!(" ✗  got {:?}", recon);
        failures.push("document sequences".into());
    }

    // 5. EdgeStore performance (quick 10K)
    print!("[5/6] EdgeStore 10K push .....");
    let mut bench_store = EdgeStore::with_capacity(10_000);
    let t = Instant::now();
    for i in 0..10_000u32 {
        bench_store.push(Edge {
            source: SynsetId(i),
            target: SynsetId(i + 1),
            relation: Relation::IsA,
            weight: 0,
            provenance: 0,
        });
    }
    let elapsed = t.elapsed();
    if elapsed.as_millis() < 100 && bench_store.bytes_used() == 100_000 {
        println!(" ✓  {}µs, 10.00 bytes/edge", elapsed.as_micros());
    } else {
        println!(" ✗  {}ms, {} bytes used",
            elapsed.as_millis(), bench_store.bytes_used());
        failures.push("edge store perf".into());
    }

    // 6. Bloom correctness
    print!("[6/6] Bloom filter ...........");
    let mut bloom = BloomFilter::default_size();
    for i in 0..1000 {
        bloom.insert(format!("item_{i}").as_bytes());
    }
    let mut ok = true;
    for i in 0..1000 {
        if !bloom.might_contain(format!("item_{i}").as_bytes()) {
            ok = false;
            break;
        }
    }
    if ok {
        println!(" ✓  1000/1000 insertions retrievable");
    } else {
        println!(" ✗  false negative (impossible for Bloom)");
        failures.push("bloom correctness".into());
    }

    // Summary
    println!("\n============================");
    if failures.is_empty() {
        println!("All 6/6 checks passed. ✓");
    } else {
        println!("{}/6 failures:", failures.len());
        for f in &failures { println!("  - {f}"); }
        process::exit(1);
    }
}
