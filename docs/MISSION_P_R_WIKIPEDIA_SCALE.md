# Mission P-R — Wikipedia Ingestion at Scale (millions/day, delete-after-ingest)

**Track:** multi-modal prep / scale
**Owner:** Claude Code, Opus (long-running)
**Est wall-clock:** 8–12h implementation, then perpetual runtime
**Depends on:** P-A (HTTP fetch) — but can build mock version meanwhile

Updated: 23.04.2026.

---

## The goal in one sentence

Take ZETS from "a few hundred articles per day via `night_learner.py`" to
**1,000,000+ articles per day**, ingested → atomized → merged to graph → **source deleted**,
all within a bounded disk budget (≤ 20 GB working set at any moment), 16+ languages in parallel,
fully resumable, and LLM-free.

## Why now

Idan, 23.04.2026:
> *"אני רוצה שהוא יילמד מליוני מאמרים ביום ולא מאות כמו שזה עכשיו העניין שכשהוא מסיים הוא צריך למחוק כדי שלא נגמור את הדיסק"*

The current autonomous learner is reading at the wrong order of magnitude.
Disk is a hard constraint. Everything below is shaped by both those facts.

## Throughput math

Target: **1 M articles/day** (per language, optional; let's start with 1M total).

- 1 M / 86,400 s ≈ **11.6 articles/sec** sustained
- Typical Wikipedia article (cirrussearch JSON) ≈ 4–8 KB text → ~6 KB average
- Raw data flow: 11.6 × 6 KB = **~70 KB/s** downstream from dump
- If we extract say 50 atoms/article (generous), that's **580 atoms/sec** into the graph
- At 10 bytes/edge (ZETS BitFlag), graph growth ≈ **500 MB/day of persisted atoms**
- At current fold compression ratios (~2×), post-compression ≈ **250 MB/day** persisted

**Disk working-set budget:** 20 GB max at any moment.
- Hot queue of dumps being processed: ≤ 5 GB.
- Hot queue of parsed JSONL awaiting atomization: ≤ 5 GB.
- WAL + pre-fold atoms: ≤ 10 GB.
Everything gets folded and compressed within 24 hours or deleted.

## The pipeline (6 stages, bounded queues)

```
          ┌────────────┐   ┌────────────┐   ┌────────────┐
Dumps --> │ 1. Fetch   │-->│ 2. Parse   │-->│ 3. Extract │
          │ shard pull │   │ JSON lines │   │ atoms+edges│
          │ q=5GB      │   │ q=5GB      │   │ q=500MB    │
          └────────────┘   └────────────┘   └─────┬──────┘
                                                  v
          ┌─────────────┐   ┌────────────┐   ┌────────────┐
   <---   │ 6. Delete   │<--│ 5. Merge   │<--│ 4. Canon-  │
          │ source      │   │ to graph   │   │ icalize    │
          │ (confirmed) │   │ WAL+atoms  │   │ dedupe     │
          └─────────────┘   └────────────┘   └────────────┘
```

Each stage has a bounded in-memory queue. If stage N+1 falls behind,
stage N blocks (backpressure). This keeps disk predictable.

### Stage 1 — Fetch

- **Source:** Wikipedia **Cirrussearch JSON dumps** — `https://dumps.wikimedia.org/other/cirrussearch/<YYYYMMDD>/`.
  Why cirrussearch, not XML: it's one JSON per line, already includes cleaned plaintext,
  trivially streamable. XML dumps require heavy parsing with `mwparserfromhell`.
- Download per-language files (e.g. `hewiki-<date>-cirrussearch-content.json.gz`).
- Validate sha1sum from dump index.
- Write to `/var/lib/zets/dumps/<lang>/` with filename `<date>-<lang>-content.json.gz`.
- **Max resident:** 5 GB. Evict oldest completed shard when budget exceeded.

### Stage 2 — Parse

- Stream the gzipped JSON line-by-line (never load whole file).
- For each line (= one article):
  - Parse JSON once.
  - Keep only: `title`, `text`, `lang`, `pageid`, `namespace`, `outgoing_link`, `categories`, `timestamp`.
  - Emit to stage 3 queue.
- **Never** write intermediate JSON to disk. Parse in-memory, emit downstream.
- Throughput target: ~20 MB/s per core sustained (trivial for Rust).

### Stage 3 — Extract

- For each article, extract:
  - **1 concept atom** for the article subject (canonical form via morphology module).
  - **N definition atoms** — the opening paragraph, split into sentence atoms with provenance = "wikipedia:<lang>:<pageid>:<revid>".
  - **IS_A edges** from opening pattern "X is a Y" / "X היא Y" / per-language templates.
  - **CATEGORY edges** from the categories field.
  - **OUTLINK edges** to other concept atoms (to be resolved in stage 4).
- Apply budget: **max 50 atoms per article**. If an article would produce more, keep the 50 highest-confidence.
- Emit to stage 4 queue.

**Key procedure atoms this stage calls** (graph-level, not Rust):
- `procedures/parse/wikipedia_article_to_atoms.toml`
- `procedures/parse/extract_is_a_from_opening.toml` (per-language variants)
- `procedures/parse/extract_outlinks.toml`

### Stage 4 — Canonicalize + Dedupe

- Every incoming atom is run through the canonization engine (`src/canonization/`).
- Hash-lookup against the existing `atom_hash_registry` (already partially present as `data/autonomous/hash_registry_sidecar.json`).
- If the atom hash exists → skip (but increment its `corroboration_count` + update provenance list).
- If new → forward to stage 5.

### Stage 5 — Merge to graph

- Append-only to WAL.
- Every 10K atoms: checkpoint → add to the `staging` scope.
- Every 100K atoms: promote staging → data scope (after sandbox verification runs, which are fast bloom-filter checks).
- Every 1M atoms OR every 24h: trigger `fold` to compress the data pack.

### Stage 6 — Delete source

**This is the crucial part Idan emphasized.**

- Once a shard is fully processed (all its articles either merged or skipped-as-duplicate):
  - Mark it in `/var/lib/zets/dumps/manifest.jsonl` with status=`"ingested"`, `completed_at`, `atoms_merged`, `atoms_skipped`.
  - **Delete the shard file.**
  - If we ever need to re-ingest, the manifest tells us to download again.
- A daily cron prunes the manifest: dumps older than 30 days get the full file deleted from `manifest.jsonl` too (just a count remains).

## Safety rails (all enforced by code)

- **Disk watchdog:** systemd timer every 60 s that checks `/var/lib/zets` usage. If > 90% of budget, pauses stage 1 until usage drops below 70%.
- **Checkpoint:** every stage persists its cursor (last processed shard + last processed article pageid within that shard). Kill -9 and restart resumes cleanly.
- **Backpressure:** each stage has a bounded queue; upstream blocks when queue is full. No OOM.
- **Idempotency:** re-processing the same article is a no-op (dedupe via hash in stage 4).
- **Rate limits to Wikipedia:** 2 concurrent connections max, 1 MB/s cap per connection per etiquette guidance. At 70 KB/s working throughput, we're nowhere near this.

## Observability

New CLI: `zets ingest-status`.
Outputs:
```
language    shards_done  shards_pending  atoms_today  atoms_total   disk_MB
he          12           3               48,321       1,203,482     142
en          28           7               186,904      7,921,003     521
ar          4            2               12,014       198,322       62
...
```

Also: a per-night summary appended to `data/autonomous/daily_report.jsonl`
(same file `procedures/self/daily_report.toml` will write).

## Parallelism strategy

- **16 languages concurrently**, each with its own queue set.
  Priority order: HE, EN, AR first (they are the anchor languages for v1).
- Within a language, the 6 stages form a pipeline — single producer at each stage.
- Stage 4 (canonicalize/dedupe) is the hottest and uses all cores (`tokio::spawn` + `rayon`).
- All I/O is async.

## Implementation plan (the deliverable for P-R)

### Sub-missions

**P-R.1 — Cirrussearch downloader** (Rust)
- `src/ingest/cirrussearch_fetcher.rs`
- Polls dump index every 24 h.
- Downloads the N most recent shards for each configured language.
- Validates sha1.
- Writes to `/var/lib/zets/dumps/<lang>/`.

**P-R.2 — Streaming parser** (Rust)
- `src/ingest/cirrussearch_parser.rs`
- gzip-decode + JSON-stream + line iterator.
- Emits `Article` struct to an mpsc channel.

**P-R.3 — Atom extractor pipeline** (Rust + graph procedures)
- `src/ingest/pipeline.rs`
- Ties stages 3–6.
- Calls out to procedure atoms for language-specific extraction.
- **This is where P-R is majority graph, not Rust.** The Rust side is orchestration.

**P-R.4 — Disk watchdog** (Rust)
- `src/ingest/watchdog.rs`
- systemd timer or internal periodic task.
- Pauses stage 1 when budget approached.

**P-R.5 — CLI + observability**
- `src/bin/zets_ingest_cli.rs` — `status`, `start`, `stop`, `pause`.
- `/var/lib/zets/ingest_status.json` updated every 10 s.
- Optional: small HTML dashboard (mission P-F can adopt this).

**P-R.6 — Migration of existing `mcp/autonomous/`**
- The existing Python pipeline in `mcp/autonomous/` (night_learner.py, multi_lang_wiki.py)
  has been useful but runs at the wrong order of magnitude and keeps raw JSONL.
- **Do not delete it.** Instead:
  1. Rename `mcp/autonomous/` → `mcp/autonomous_legacy/`.
  2. Add a deprecation note pointing at P-R.
  3. Keep it runnable for ~30 days as a fallback.
  4. Once P-R hits 1M articles/day in production for a week, remove legacy.

## Acceptance criteria (done when)

- [ ] `zets ingest-start he,en,ar` runs stable for 24 hours.
- [ ] At least 500,000 unique atoms added in a day, cross 3 languages combined.
- [ ] Disk usage stays under 20 GB at all times (peak measured by disk watchdog log).
- [ ] Killing the process and restarting resumes cleanly (verify via checkpoint file).
- [ ] `zets ingest-status` output matches reality (spot-check 3 times over the day).
- [ ] Cross-lingual IS_A edge count grows each day (proof that we're not just duplicating).
- [ ] 0 LLM calls (grep the source code, grep the run logs).

## What this does NOT do (out of scope for P-R itself)

- Image / speech / video ingestion — those are P-N, P-O, P-Q.
- Full NL I/O — that's P-I.
- Answering questions about the ingested content — separate, phase-4.
- Math reasoning — P-H.

P-R's job is to make the GRAPH BIG, FAST, HONEST, AND RECOVERABLE. Nothing else.

---

## How this mission ships (runtime playbook)

Once P-R.1–P-R.5 are merged:

```bash
# One-time setup
sudo mkdir -p /var/lib/zets/dumps
sudo chown -R dinio:dinio /var/lib/zets

# Start the pipeline
zets ingest-start --languages he,en,ar --disk-budget 20G

# Check on it
zets ingest-status

# Tail logs
journalctl -u zets-ingest -f

# Graceful pause (leaves state resumable)
zets ingest-pause

# Resume
zets ingest-resume
```

And it runs. Forever. Within budget. Deleting sources as it consumes them.
