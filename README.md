# ZETS

**Deterministic knowledge graph engine for edge devices.**

Symbolic Q&A system that runs on $80 hardware, answers in <100ms, never hallucinates.

## Status

**Week 1-2 of 12.** Implemented:
- UNP (Unified Normalization Pipeline) with Hebrew support
- Hebrew canonicalizer (niqud + final forms) + aggressive stemmer (Wk3 fallback)
- EdgeStore: columnar SOA, 10 bytes/edge, 27 relation types
- BloomFilter: custom FNV-1a + Kirsch-Mitzenmacher double-hashing
- Binary serialization (ZEDG format)
- **Meta-graph**: system synsets in reserved ID range 0..999 (homoiconic)
- **LangCode**: ISO 639-1 codes (he, en, ar, cn, ru, ja, es, fr, de, pt)

## Philosophy

- **Data separated from code.** TSV files in `data/`, compiled in via `build.rs`.
- **Zero runtime dependencies.** Single binary, stdlib only.
- **Single Rust file** (`src/main.rs`). Solo-dev friendly.
- **Deterministic.** Same input → same output. No seed, no sampling.
- **Homoiconic.** System rules are graph nodes; the graph describes itself.
- **Offline-first.** No network at runtime. Packs downloaded on user activation.

## Project layout

```
zets/
├── Cargo.toml
├── build.rs                   # TSV → Rust const tables at compile time
├── .cargo/config.toml         # hardware flags (AES, Neon, AVX2)
├── src/
│   └── main.rs                # single file, all code
└── data/
    ├── core/
    │   ├── relations.tsv      # 27 relation types
    │   ├── languages.tsv      # ISO 639-1 codes
    │   └── system_synsets.tsv # meta-graph reserved nodes
    ├── hebrew/
    │   ├── prefixes.tsv
    │   ├── suffixes.tsv
    │   └── meta.json
    └── test_vectors/
        └── unp_v1.tsv
```

## Build

```bash
cargo build --release
# Binary at target/release/zets
```

## Run

```bash
# Tests
cargo test

# CLI
./target/release/zets normalize he "הַבַּיִת הַגָּדוֹל"
./target/release/zets stem-he "הכלבים"
./target/release/zets languages
./target/release/zets relations
./target/release/zets meta-graph
./target/release/zets test-vectors

# Benchmarks
./target/release/zets bench-edges 1000000
./target/release/zets bench-bloom 500000
./target/release/zets bench-serialize 100000
```

## Meta-graph (11th sphere)

Synset ID ranges:
- `0..5` — root registries (SYSTEM, LANGUAGE_REGISTRY, etc.)
- `10..29` — language nodes (`he`=10, `en`=11, `ar`=12, ...)
- `30..56` — relation type nodes (`IS_A`=30, `HAS_PART`=31, ...)
- `60..69` — status nodes (ACTIVE, INACTIVE, PENDING, ...)
- `70..79` — permission nodes (MASTER, ADMIN, USER, ...)
- `10000+` — user-facing content

The system describes itself: relations are nodes, languages are nodes,
permissions are nodes. Query `outgoing(LANGUAGE_REGISTRY)` to list
active languages. No special cases.

## Roadmap

- **Week 3**: surface→lemma dictionary (Dicta), mmap, Roaring adjacency, BLAKE3
- **Week 4-5**: walk engine Forward+Backward, composition, multi-pass (stop at 95%, cap 7)
- **Week 6-8**: LSM WAL, server-side build pipeline, 1K Hebrew Wikipedia subset
- **Week 9-12**: voice I/O, overlay encryption, Android APK, demo

## License

MIT.

## Author

Idan Eldad (CHOOZ).
