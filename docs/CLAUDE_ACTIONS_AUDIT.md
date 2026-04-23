# CLAUDE_ACTIONS_AUDIT.md

> **The core question:** everything Claude does on behalf of ZETS — what does it take
> for ZETS to do it by itself?
>
> The answer lives here. Every row is a handoff plan.

Updated: 23.04.2026.

---

## Why this document exists

Idan's line (23.04.2026):
> *"אני רוצה שזטס יהיה אוטונומי. צריך ללמד אותו לעשות לבד את מה שעד עכשיו קלוד עשה עבורו."*

And earlier:
> *"Learning is in code. What to learn and how — is in the graph. Like a human: knows how to learn, but what to learn is in the graph."*

And later (same day):
> *"שום קוד לא צריך לחזור על עצמו כי זה למעשה קשתות למושגים שצריך שיהיו בגרף, ואם זה לא ככה צריך לתקן את זה ובהמשך שככה יעבוד ZETS"*

This file translates those principles into a handoff queue. Each action Claude
currently performs is either:
- **Rust-primitive work** — a small number of cases where ZETS genuinely needs a new low-level primitive.
- **Graph-procedure work** — the large majority: procedures expressible as TOML atoms that the Learning Loop Executor (`mission P-D`) walks.
- **Graph-knowledge work** — facts / heuristics added as knowledge atoms.
- **Graph-motivation work** — goals / values added as motivation atoms.
- **Research** — open problems we will not solve this quarter.

**DRY corollary (Idan, 23.04.2026):** If Rust code duplicates, the duplication is
a *signal* that a concept is missing from the graph. Fix is to lift the concept
to a graph atom and have both sites walk to it — not extract a shared helper
function. This audit surfaces such duplications.

---

## How to read the rows

Each table row:

| Column | Meaning |
|---|---|
| **Action** | What Claude does today |
| **Freq** | How often a ZETS session needs this (🔴 every session · 🟡 often · 🟢 occasional) |
| **Layer** | `Rust` · `Graph-Proc` · `Graph-Know` · `Graph-Mot` · `Research` |
| **Primitive** | Which of the 7 primitives (if any) it needs |
| **Procedure** | Name of the procedure atom to build (TOML file under `data/procedures/`) |
| **Unblocks** | Which missions from `docs/ZETSON_AGENT_MISSIONS.md` this serves |
| **Status** | ❌ not started · 📝 specced · 🟡 partial · ✅ done |

---

## 1. Fetching content from the open web

Everything Claude does in "go read that page" bucket.

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| HTTP GET a URL | 🔴 | Rust | `fetch` | — (primitive itself) | P-A | 📝 |
| Honour `robots.txt` | 🔴 | Rust+Graph-Know | `fetch` | `procedures/web/check_robots.toml` | P-A | ❌ |
| Set custom User-Agent | 🔴 | Rust | `fetch` | — (fetch config) | P-A | 📝 |
| Backoff on 429 / 5xx | 🔴 | Rust | `fetch` | `procedures/web/rate_limit_backoff.toml` | P-A | ❌ |
| Cache by URL + ETag | 🟡 | Rust | `fetch`+`store` | `procedures/web/cache_with_etag.toml` | P-A | ❌ |
| Pick the right URL for a topic | 🔴 | Graph-Proc | — | `procedures/web/pick_source_for_topic.toml` | P-D | ❌ |
| Decide when to re-fetch (TTL) | 🟡 | Graph-Know | — | `knowledge/source_ttl_defaults.toml` | P-D | ❌ |

### The decision-for-source procedure (sketch)

```toml
# data/procedures/web/pick_source_for_topic.toml
id = "pick_source_for_topic"
inputs = ["topic_atom", "language"]
outputs = ["url_list", "expected_trust"]
steps = [
  { lookup = "knowledge/source_registry" },             # curated list
  { filter = "source.topic_match(topic_atom) > 0.4" },
  { rank  = "source.trust_score * source.freshness" },
  { take  = 3 },
]
```

---

## 2. Parsing content

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| HTML → DOM tree | 🔴 | Rust | `parse` | — (primitive) | P-B | 📝 |
| DOM → text-with-structure | 🔴 | Rust | `parse` | `procedures/parse/flatten_dom.toml` | P-B | ❌ |
| Extract article body (vs. nav / ads) | 🔴 | Graph-Proc | `parse` | `procedures/parse/extract_article.toml` | P-B | ❌ |
| Parse JSON | 🔴 | Rust | `parse` | — (primitive) | P-B | 📝 |
| Parse Wikitext | 🟡 | Graph-Proc | `parse` | `procedures/parse/wikitext_to_plain.toml` | P-B, ingest | ❌ |
| Identify language of a block | 🟡 | Graph-Proc | `tokenize` | `procedures/text/detect_language.toml` | P-D | ❌ |
| Split into paragraphs / sentences | 🔴 | Rust+Graph | `tokenize` | `procedures/text/segment.toml` | P-D | 🟡 |
| Tokenize + POS (per language) | 🔴 | Rust+Graph | `tokenize` | — (morphology module + POS TSVs) | P-D | ✅ (for 16 langs) |

---

## 3. Deciding what to learn next

This is where "intelligence" feels most like a thing Claude does implicitly. It is
not magic. It's a small set of procedures over motivation atoms.

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Identify a gap (word unknown, edge missing) | 🔴 | Graph-Proc | `retrieve` | `procedures/meta/find_gap.toml` | P-D | 🟡 |
| Pick one gap to work on now | 🔴 | Graph-Proc | — | `procedures/meta/prioritize_gap.toml` | P-D | ❌ |
| Choose a learning strategy for the gap | 🔴 | Graph-Proc | — | `procedures/meta/select_strategy.toml` | P-D | ❌ |
| Bound the work (max pages, max minutes) | 🔴 | Graph-Proc+Graph-Mot | — | `procedures/meta/bound_work.toml` | P-D | ❌ |
| Decide when to stop / come back later | 🔴 | Graph-Proc | — | `procedures/meta/should_yield.toml` | P-D | ❌ |

---

## 4. Judging a source

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Know that Wikipedia > random blog | 🔴 | Graph-Know | — | `knowledge/source_registry` | P-A, P-L | 🟡 |
| Compute a trust score for a new source | 🟡 | Graph-Proc | — | `procedures/meta/score_new_source.toml` | P-L | ❌ |
| Down-weight a source after contradiction | 🟡 | Graph-Proc | `store` | `procedures/meta/penalize_source.toml` | P-L | 🟡 |
| Corroboration: 2 sources agree → trust ↑ | 🔴 | Graph-Proc | — | `procedures/meta/corroborate.toml` | P-D, P-L | ❌ |

---

## 5. Understanding a query (what the user / upstream asked)

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Classify intent (lookup / relate / define / compute) | 🔴 | Graph-Proc | — | `procedures/query/classify_intent.toml` | P-I | ❌ |
| Extract target concept(s) from sentence | 🔴 | Graph-Proc | `tokenize` | `procedures/query/extract_targets.toml` | P-I | ❌ |
| Map "is X a Y?" → `is_ancestor(X, Y, 5)` | 🔴 | Graph-Proc | `retrieve` | `procedures/query/bind_relational.toml` | P-I | ❌ |
| Detect Hebrew vs English vs mixed | 🔴 | Graph-Proc | `tokenize` | `procedures/text/detect_language.toml` | P-I | ❌ |

---

## 6. Generating an answer (natural language out)

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Turn a concept + its relations into a sentence | 🔴 | Graph-Proc | `communicate` | `procedures/speak/render_definition.toml` | P-I | ❌ |
| Apply Hebrew morphology (gender, number, def.) | 🔴 | Rust+Graph | `communicate` | — (morphology module) | P-I | ✅ |
| Apply English morphology (a/an, plurals) | 🔴 | Rust+Graph | `communicate` | — (morphology module) | P-I | ✅ |
| Choose register (formal / casual / technical) | 🟡 | Graph-Proc | — | `procedures/speak/pick_register.toml` | P-I | 🟡 |
| Insert uncertainty hedges when confidence low | 🔴 | Graph-Proc | — | `procedures/speak/hedge_when_uncertain.toml` | P-I | ❌ |

---

## 7. Cross-lingual alignment

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Align same-meaning words across languages | 🟡 | Graph-Proc | `retrieve`+`store` | `procedures/align/translation_to_edge.toml` | P-J | ❌ |
| Align to a shared sense / synset | 🟡 | Graph-Proc | — | `procedures/align/find_shared_sense.toml` | P-J | 🟡 |
| Detect false-friend (looks same, means different) | 🟢 | Graph-Proc | — | `procedures/align/detect_false_friend.toml` | P-J | ❌ |
| Pivot through English for low-resource pairs | 🟢 | Graph-Proc | — | `procedures/align/pivot_via_english.toml` | P-J | ❌ |

---

## 8. Math & reasoning

Per the core principle: math is **not** a Rust module. Math is procedures that use
the `reason` primitive + knowledge atoms for axioms.

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Add two numbers | 🔴 | Graph-Know+Graph-Proc | `reason` | `procedures/math/add_naturals.toml` + axioms | P-H | ❌ |
| Multiply, subtract, divide | 🔴 | Graph-Proc | `reason` | `procedures/math/multiply_naturals.toml`, etc. | P-H | ❌ |
| Apply arithmetic to word problems | 🟡 | Graph-Proc | `reason` | `procedures/math/word_problem_parse.toml` | P-H | ❌ |
| Algebra substitution | 🟢 | Graph-Proc | `reason` | `procedures/math/algebra_substitute.toml` | P-H | ❌ |
| Symbolic proof search (tiny) | 🔬 | Research | — | — | far future | 🔬 |

**Open question — bignum:** should arithmetic on very large numbers use a Rust
`bignum` primitive, or stay in graph bytecode? **Council answered:** stay in
bytecode for the teachable/auditable win; measure throughput first. Add a
`bignum_ops` sub-primitive inside `reason` *only if* bytecode can't hit target ops/sec
after benchmarking. Re-examine after P-C + P-H land.

---

## 9. Image understanding (new — Idan's ask 23.04.2026)

The rule: **no LLM calls.** All features come from Python CV libraries (OpenCV,
scikit-image, PIL) and atoms on the graph encode what a given feature *means*.

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Decode image bytes to pixel grid | 🔴 | Rust (or Python helper) | `parse` | — (primitive extension) | P-N | ❌ |
| Resize / normalize | 🔴 | Rust (or Python) | `parse` | `procedures/image/normalize.toml` | P-N | ❌ |
| Color histogram (RGB / HSV) | 🔴 | Python helper | `parse` | `procedures/image/color_histogram.toml` | P-N | ❌ |
| Edge detection (Canny / Sobel) | 🔴 | Python helper | `parse` | `procedures/image/edges.toml` | P-N | ❌ |
| Shape / contour detection | 🟡 | Python helper | `parse` | `procedures/image/contours.toml` | P-N | ❌ |
| LBP / HOG feature descriptors | 🟡 | Python helper | `parse` | `procedures/image/lbp_hog.toml` | P-N | ❌ |
| Feature → concept atom (learned edge) | 🔴 | Graph-Proc | `store`+`retrieve` | `procedures/image/feature_to_concept.toml` | P-N | ❌ |
| Face detection (Haar cascades — classical) | 🟢 | Python helper | `parse` | `procedures/image/faces_haar.toml` | P-N | ❌ |
| OCR of printed text (Tesseract — classical) | 🟡 | Python helper | `parse`+`tokenize` | `procedures/image/ocr_tesseract.toml` | P-N | ❌ |

**Key design point (council):** treat image analysis the same way we treat text.
The Python layer (already present as `mcp/`) extracts features; the Rust `store`
primitive stores them; the graph learns feature → concept edges the same way it
learns "this Hebrew string → this synset." No special-casing in Rust.

---

## 10. Speech and audio (new — Idan's ask 23.04.2026)

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Decode audio file (wav / mp3 / ogg) | 🔴 | Rust or Python | `parse` | — (primitive extension) | P-O | ❌ |
| Waveform → MFCC features | 🔴 | Python (librosa) | `parse` | `procedures/audio/mfcc.toml` | P-O | ❌ |
| Spectral features (centroid, rolloff) | 🟡 | Python | `parse` | `procedures/audio/spectral.toml` | P-O | ❌ |
| Voice activity detection (VAD) | 🔴 | Python (webrtcvad) | `parse` | `procedures/audio/vad.toml` | P-O | ❌ |
| Speech → phoneme stream (Vosk / Sphinx) | 🔴 | Python (offline) | `parse`+`tokenize` | `procedures/audio/stt_vosk.toml` | P-O | ❌ |
| Phoneme stream → Hebrew / English words | 🔴 | Graph-Proc | `tokenize` | `procedures/audio/phonemes_to_words.toml` | P-O | ❌ |
| Classify (speech / music / silence / env) | 🟡 | Graph-Proc | `reason` | `procedures/audio/classify_kind.toml` | P-O | ❌ |
| TTS (ZETS speaking back) | 🟢 | Python (espeak-ng / piper) | `communicate` | `procedures/audio/tts_piper.toml` | P-O | ❌ |

**Stack chosen by the council:**
- **STT offline primary:** [Vosk](https://alphacephei.com/vosk/) — multilingual,
  free, Apache 2.0, Hebrew model available. Runs on MCU-class hardware.
- **STT fallback:** Whisper-local (openai/whisper small — still no OpenAI API call; local inference).
  Uses GPU if present. Heavier.
- **TTS:** Piper (Rhasspy) — local, neural-but-tiny, good Hebrew voices.
- **DSP features:** librosa + webrtcvad. Classical; no ML magic.
- **Cloud option rejected.** Google STT mentioned by Idan as an option,
  rejected by council because (a) not offline, (b) sends audio to Google,
  (c) ZETS's thesis is everything-local. If needed as *bootstrap* teacher
  for labeling data during training, OK; not in runtime.

---

## 11. Video understanding (new — Idan's ask 23.04.2026)

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Extract keyframes (scene change) | 🔴 | Python (FFmpeg) | `parse` | `procedures/video/keyframes.toml` | P-Q | ❌ |
| Keyframe → image procedures (reuse P-N) | 🔴 | Graph-Proc | — | (composition of image/*.toml) | P-Q | ❌ |
| Temporal atom: "frame_t comes_after frame_t-1" | 🔴 | Graph-Proc | `store` | `procedures/video/temporal_link.toml` | P-Q | ❌ |
| Audio track → speech procedures (reuse P-O) | 🔴 | Graph-Proc | — | (composition of audio/*.toml) | P-Q | ❌ |
| Scene summary from frames + speech | 🟡 | Graph-Proc | `reason`+`communicate` | `procedures/video/scene_summary.toml` | P-Q | ❌ |

**Design principle:** video = sequence of images + audio track + temporal edges.
No new Rust primitives. Mission P-Q is a *composition* mission — it combines
P-N and P-O outputs with a thin temporal layer.

---

## 12. Self-inspection and self-improvement

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Count atoms / edges / coverage by domain | 🟡 | Graph-Proc | `retrieve` | `procedures/self/inventory.toml` | P-F | ❌ |
| Produce a daily report of what was learned | 🔴 | Graph-Proc | `retrieve` | `procedures/self/daily_report.toml` | P-F | ❌ |
| Propose its own next learning goal | 🟡 | Graph-Proc | — | `procedures/self/propose_goal.toml` | P-D | 🟡 |
| Propose a new procedure when a gap repeats | 🔬 | Research | — | — | far future | ❌ |
| Modify own Rust code | 🚫 | **never** | — | **forbidden** — stays out of graph | — | 🚫 |

**Hard rule (repeat for emphasis):** ZETS does **not** self-modify its Rust
primitives. Only the graph layer is self-modifiable. If the graph detects the
7 primitives are insufficient, that's a ticket for Idan / Claude, not a self-patch.

---

## 13. Talking to Idan / external agents

| Action | Freq | Layer | Primitive | Procedure | Unblocks | Status |
|---|---|---|---|---|---|---|
| Respond over MCP | 🟡 | Rust | `communicate` | — | mcp integration | ✅ |
| Respond in Hebrew with correct register | 🟡 | Graph-Proc | `communicate` | `procedures/speak/hebrew_reply.toml` | P-I | ❌ |
| Ask a clarifying question when ambiguous | 🔴 | Graph-Proc | `communicate` | `procedures/speak/request_clarification.toml` | P-I | ❌ |
| Remember previous turns in a session | 🔴 | Graph | `store`+`retrieve` | — (conversation module) | dialogue | 🟡 |

---

## 📋 The 20 initial procedures for mission P-C (MVP list)

Ordering by **(frequency × unblocks-count)** — here are the 20 first procedures
Claude Code should author as TOML for mission P-C. This is the deliverable target.

1. `procedures/web/check_robots.toml`
2. `procedures/web/rate_limit_backoff.toml`
3. `procedures/web/cache_with_etag.toml`
4. `procedures/web/pick_source_for_topic.toml`
5. `procedures/parse/flatten_dom.toml`
6. `procedures/parse/extract_article.toml`
7. `procedures/parse/wikitext_to_plain.toml`
8. `procedures/text/detect_language.toml`
9. `procedures/text/segment.toml`
10. `procedures/meta/find_gap.toml`
11. `procedures/meta/prioritize_gap.toml`
12. `procedures/meta/select_strategy.toml`
13. `procedures/meta/bound_work.toml`
14. `procedures/meta/should_yield.toml`
15. `procedures/meta/learning_cycle.toml` — the outer loop, in graph form
16. `procedures/meta/corroborate.toml`
17. `procedures/self/daily_report.toml`
18. `procedures/self/inventory.toml`
19. `procedures/speak/render_definition.toml`
20. `procedures/speak/hedge_when_uncertain.toml`

---

## 🧩 Mission roster (as of 23.04.2026 session #2)

| Code | Title | Owner | Est. wall-clock |
|---|---|---|---|
| P-A | HTTP fetch primitive | Claude Code, Sonnet | 3h |
| P-B | HTML parser primitive | Claude Code, Sonnet | 3h |
| P-C | Procedure loader + 20 procedures | Claude Code, Opus | 6h |
| P-D | Learning loop executor | Claude Code, Opus | 8h |
| P-E | Seed loader (YAML → atoms) | Claude Code, Sonnet | 2h |
| P-F | Observability dashboard | Claude Code, Sonnet | 3h |
| P-G | Zetson first-day demo | Claude Code, Opus | 4h |
| **P-H** | **Math procedures pack** | Claude Code, Opus | 8h |
| **P-I** | **NL I/O pack** | Claude Code, Opus | 12h |
| **P-J** | **Cross-lingual alignment pack** | Claude Code, Sonnet | 6h |
| **P-K** | **Benchmarks integration (MMLU/HumanEval/MATH)** | Claude Code, Sonnet | 6h |
| **P-L** | **Source registry & trust** | Claude Code, Sonnet | 4h |
| **P-M** | **Code quality audit + duplication→graph-gap report** | Claude Code, Opus | 4h |
| **P-N** | **Image understanding (no LLM)** | Claude Code, Opus | 10h |
| **P-O** | **Speech recognition + TTS (no LLM)** | Claude Code, Opus | 10h |
| **P-P** | **Provenance DB — client-data recovery API** | Claude Code, Opus | 6h |
| **P-Q** | **Video understanding (composition of P-N + P-O)** | Claude Code, Sonnet | 6h |

Proposed ordering (dependencies first):
**P-M** → **P-A, P-B** (parallel) → **P-C, P-E** (parallel) → **P-D** → **P-F** →
**P-L** → **P-I** → **P-H** → **P-J** → **P-K** → **P-P** → **P-N** → **P-O** → **P-Q** → **P-G** (demo integration).

---

## 🧠 Meta: how this file stays honest

- When a row moves from `❌` to `📝` or from `📝` to `🟡` or `🟡` to `✅`,
  add a one-liner to `docs/DECISIONS_LOG.md` with date + what changed.
- When Claude-in-a-future-session catches itself doing something not on this
  list — **add the row**. That's the mechanism.
- The goal isn't zero Claude actions forever. The goal is: every Claude action
  should be *logged* as a gap, even if it's not filled yet. Silent Claude-assists
  are the real enemy.

---

*If this file is empty one day — ZETS is autonomous.*
*We are very far from that day. But the path is visible.*
