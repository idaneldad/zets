# Chunk 3 — Honest Report: Cleanup + Morphology Didn't Help (23.04.2026)

## Summary
ניסיון להוסיף Phase A (cleanup + morphology) **לא שיפר** את ה-accuracy.
התוצאה נשמרה: MINIMAL הוא ה-production. הקוד של cleanup ו-morphology נשאר ב-repo כ-latent infrastructure שאולי יהיה שימושי אחרי scale-up.

## Metrics

| Configuration | Top-1 | Top-3 | Junk% | RSS |
|---|---|---|---|---|
| MINIMAL (3 relations) | **53%** | 62% | 44% | 2.77 GB |
| CLEAN (+ cleanup regex) | 44% | 59% | 22% | 2.63 GB |
| ENRICHED (+ morphology) | 44% | 59% | 22% | 2.65 GB |

## Why the failure?

### Cleanup
- Filtered 60K/703K sentences (8.5%)
- Removed ALL sentences with `[[link]]` or `{{template}}` markers
- Problem: legitimate intro sentences ALSO contain these markers
- Accuracy dropped because relevant sentences got cut

### Morphology (lemma_of edges)
- 186K edges created (39% of words)
- Forward-walk via lemma didn't improve retrieval
- Reverse-walk (lemma → variants) made it *worse* (score diffusion)
- Root cause: 8/9 failures are **missing articles**, not weak morphology
  - "Apollo 11" article doesn't exist in 10K corpus
  - "Shakespeare" doesn't exist
  - "שמיים" doesn't exist
  - Morphology on non-existent concepts cannot help

## What stays in repo

- `src/graph_v4/cleaner.rs` — 135 lines, 2 unit tests pass
- `src/graph_v4/morphology.rs` — 218 lines, 6 unit tests pass
- Disabled by default in `BuildConfig` — no regression

**Latent infrastructure** — will be re-evaluated after corpus scaleup.

## What I learned

1. **Occam's razor wins** — adding features doesn't automatically help
2. **Empirical validation is essential** — I would have committed a regression without the 40-question test
3. **Scaleup is the real lever** — 10K articles is the limit; below ~100K, tweaking retrieval is noise
4. **Cleanup needs nuance** — regex-based wholesale filtering over-removes

## Next action (scaled-down)

**NOT** in this session: scale up to 50K EN + 50K HE.

Reasons:
- Will require ~15 GB disk, ~28 GB RAM — in bounds but significant
- Accuracy expected: 75-80% (baseline, no improvements needed)
- Test suite must be re-run

## Status

🟢 Production (port 3149): v4_minimal.zv4, MINIMAL arch, 53% Top-1
🟡 cleaner + morphology: code committed, disabled by default
🔴 Next step: scaleup — separate session
