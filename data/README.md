# ZETS Data Directory

This directory holds **versioned graph snapshots**, corpora, benchmark
question sets, and encrypted installers. It exists because ZETS's
capability depends on its CONTENT as much as its code — you can't
reproduce results with source alone.

## Structure

```
data/
├── baseline/                        ← AtomStore snapshots (tracked)
│   ├── v1_bootstrap.atoms           ← fresh brain, 119 atoms
│   ├── v1_bootstrap.manifest.json
│   ├── v1_world_facts.atoms         ← bootstrap + 30 world facts
│   ├── v1_world_facts.manifest.json
│   ├── v1_pet_facts.atoms           ← bootstrap + pet domain
│   └── v1_pet_facts.manifest.json
│
├── benchmarks/                      ← JSONL question sets (tracked)
│   └── zets_baseline_20q_v1.jsonl   ← current 20-question baseline
│
├── corpora/                         ← raw text inputs (tracked)
│   ├── world_facts_v1.txt
│   └── pet_facts_v1.txt
│
├── installer/                       ← encrypted shippable installers (tracked)
│   └── v1_bootstrap.zets_enc        ← AES-256-GCM wrapped bootstrap
│
├── core/                            ← legacy ZETS tsv (language tables)
├── hebrew/, multilang/, packs/      ← legacy pack data (NOT in git, >100MB)
└── seeds/                           ← seed vocabularies (tracked, small)
```

## Tools

The `snapshot` binary manages `baseline/`, `corpora/`, and `installer/`:

```bash
# Build the default starter set (bootstrap, world facts, pet facts)
./target/release/snapshot bootstrap-default

# List all snapshots
./target/release/snapshot list

# Inspect a specific snapshot's manifest
./target/release/snapshot info v1_world_facts

# Verify a snapshot loads cleanly
./target/release/snapshot verify v1_world_facts

# Create an encrypted shippable installer
./target/release/snapshot package v1_bootstrap "my-passphrase"

# Build a new snapshot from a corpus
./target/release/snapshot create v2_tech_facts --corpus path/to/tech.txt
```

## Versioning Rules

1. **Never mutate a published snapshot.** If you change ingestion logic or
   bootstrap content, create a new version (`v2_*`). Old snapshots remain
   reproducible against their era of code.

2. **Every `.atoms` file has a `.manifest.json`** describing what went in
   (atoms, edges, corpus, ingestion stats, `bootstrapped` flag).

3. **Atom binary format is tagged with `format_version`** inside the header.
   When we move FNV-1a → BLAKE3, bump to `format_version: 2` and provide a
   migration tool. Old snapshots stay readable by old code.

4. **Corpora are the source of truth.** If you can regenerate an `.atoms`
   file from a corpus and bootstrap, you can trust it. The atoms are a
   cache; the corpus is canon.

5. **Installers are BYTE-DETERMINISTIC.** Same passphrase + same snapshot
   → same encrypted bytes. This means a shipped installer can be verified
   by hash against the build server.

## What Belongs Here vs `/home/claude` or `/mnt/`

- **Tracked in git:** small (<100KB), reproducible, canonical.  
  The v1 snapshots (~30KB total) qualify.
- **Ignored:** huge packs (Hebrew corpus 102MB, multilang 194MB).
  These live on the server, regenerable from sources listed in
  `docs/working/...ingestion_sources.md`.

## Upgrade / Restore Workflows

### Restore a lost working graph

```bash
cp data/baseline/v1_world_facts.atoms /tmp/brain.atoms
# Now load it in your code:
#   let store = atom_persist::load_from_file("/tmp/brain.atoms")?;
```

### Ship a brain to a fresh device

```bash
./target/release/snapshot package v1_world_facts "device-key-123"
# Transfer data/installer/v1_world_facts.zets_enc to the device.
# On the device:
#   let store = encrypted_installer::install_from_file("path/to/blob", "device-key-123")?;
```

### Upgrade from v1 to v2

```
1. Keep v1 snapshots committed.
2. Write src/migrations/v1_to_v2.rs that reads v1, applies changes, writes v2.
3. Regenerate v2_* snapshots.
4. Code continues reading both format versions until v1 is EOL'd.
```

## External Ingestion Sources (for future Phase 4)

When we scale to billions of atoms, these are the fire hoses to tap:

| Source | What it gives us | Volume | Access |
|--------|------------------|--------|--------|
| **Common Crawl** | Raw web text, HTML-clean | Petabytes on S3 | Public, free |
| **GDELT** | News events, global, 100+ langs | ~15min refresh | Public API |
| **Wikimedia Dumps** | Wikipedia + version history, all langs | ~100GB compressed | Public download |
| **Archive.org CDX API** | Versioned snapshots of URLs | Billions of pages | Rate-limited JSON |
| **Semantic Scholar** | Academic papers, citations | ~200M abstracts | Public API |

Each would require its own ingester (`src/ingestion/common_crawl.rs`, etc.)
and its own corpus versioning folder under `data/corpora/common_crawl_YYYYMM/`.
