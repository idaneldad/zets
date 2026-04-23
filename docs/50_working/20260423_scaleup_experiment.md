# Scaleup Experiment — 10K → 40K (23.04.2026)

## Result: Counter-intuitive

| Corpus | atoms | edges | disk | RSS | Top-1 | Top-3 |
|---|---|---|---|---|---|---|
| 10K   | 2.5M  | 20.7M  | 424MB | 2.77GB | **53%** | 66% |
| 40K   | 8.0M  | 71.8M  | 1.5GB | 9.3GB  | 38%     | 59% |

**Top-1 dropped, Top-3 stayed similar** → correct answer is still found, just not ranked first.

## Root cause

Retrieval architecture doesn't scale linearly:
- Score accumulation favors long articles with many occurrences
- IDF doesn't normalize for article length
- Phrase extraction produces more noise on larger corpora (min_count=3 hits more "died ndash" garbage)
- No confidence threshold — always picks something

## Lesson

**Scale without retrieval improvements hurts.** The architecture needs:
1. TF-IDF length normalization (per article)
2. Phrase PMI filtering (kill "died ndash" tier)
3. Confidence threshold + "I don't know"
4. Named entity boost (capitalize/title-case detection)

These are **mandatory before further scaling**.

## Status

- Production: 10K minimal on port 3149 (unchanged)
- v4_40k.zv4 archived for future re-use after retrieval v2
- 40K snapshot NOT replacing production

## Next

Phase B (retrieval v2) before next scaleup:
  1. length-normalized TF-IDF
  2. phrase quality filter (PMI threshold)
  3. confidence scoring
  4. rerun 40K evaluation — target 70%+ Top-1
  5. only then scale to 100K+
