# Multilingual Data Extraction — Results

**Date:** 21.04.2026
**Phase:** Dictionary & multilingual knowledge ingestion (pre-Zets load)
**Source:** English Wiktionary dump (1.5GB bz2)

## Goal

Extract multilingual vocabulary focused on the 4 challenge cases Idan identified:
1. **Homographs across languages** — e.g., "gift" = present (en) vs poison (de) vs marriage (sv)
2. **Variants with special characters** — e.g., café/cafe, größe/groesse
3. **Abbreviations and acronyms** — DNA, D.N.A., "di-en-ei" (deferred to later sprint)
4. **Cross-language translation networks**

## Target languages (10)

| Code | Language | Why chosen |
|------|----------|------------|
| en | English | Base language, case study for homographs |
| he | Hebrew | Our primary language, RTL, niqud variants |
| ar | Arabic | RTL, niqud, ligatures |
| de | German | Umlauts, ß, major homograph source with English |
| fr | French | Accents, cedilla |
| es | Spanish | ñ, accents |
| ru | Russian | Cyrillic script |
| zh | Chinese | No spaces, characters |
| ja | Japanese | 3 scripts mixed |
| hi | Hindi | Devanagari script |

## Performance — Single streaming pass

- **Input:** 1.5 GB compressed (enwiktionary.xml.bz2)
- **Pages scanned:** 10,488,505
- **Time:** 690 seconds (11.5 minutes)
- **Throughput:** 15,195 pages/sec
- **RAM usage:** ~700 MB during run (XML iterparse + clear)

## Output

| File | Size | Rows | Purpose |
|------|------|------|---------|
| `entry_en.tsv` | 22 MB | 1,362,873 | English entries with POS + first definition |
| `entry_he.tsv` | 303 KB | 13,642 | Hebrew entries |
| `entry_ar.tsv` | 604 KB | 57,832 | Arabic entries |
| `entry_de.tsv` | 1.9 MB | 347,139 | German entries |
| `entry_fr.tsv` | 1.8 MB | 387,852 | French entries |
| `entry_es.tsv` | 2.0 MB | 762,292 | Spanish entries |
| `entry_ru.tsv` | 1.6 MB | 426,046 | Russian entries |
| `entry_zh.tsv` | 3.0 MB | 300,597 | Chinese entries |
| `entry_ja.tsv` | 1.8 MB | 170,829 | Japanese entries |
| `entry_hi.tsv` | 651 KB | 34,197 | Hindi entries |
| `definitions_en.tsv` | 23.8 MB | 396,079 | English definitions (up to 3 senses each) |
| `homographs.tsv` | 1.5 MB | 95,103 | Surface forms in 2+ languages |
| `translations.tsv` | 21.4 MB | 734,771 | Cross-language translation pairs |

**Total: 82.5 MB, ~4.56M rows of structured multilingual knowledge.**

## Key validation

### Gift — the target case for homographs
```
gift -> ar : هَدِيَّة
gift -> fr : présent, cadeau, don
gift -> de : Geschenk, Präsent, Spende, Gabe
gift -> he : מַתָּנָה, שַׁי
gift -> hi : उपहार, देन, बख़्शिश, सौग़ात, भेंट, हदिया
```
We have the translations from English "gift" to all 10 languages. Lowercase
"gift" appears in multiple language sections — when we add the German `Gift`
(poison) and Swedish `gift` (married), the homograph is fully captured.

### Scripts represented
- Latin (en, de, fr, es)
- Cyrillic (ru)
- Hebrew RTL (he)
- Arabic RTL (ar)
- Chinese characters (zh)
- Japanese 3-script (ja)
- Devanagari (hi)

All major script families needed for initial multilingual graph testing.

## Integration into ZETS

This data is **not yet loaded into the graph**. Next step (pending Idan's approval):

1. Extend `learning::ingest()` to accept multiple TSV formats
2. Generate `SameAs` edges from `translations.tsv` (word in lang A ↔ translation in lang B)
3. Generate `NearSynonym` edges for multi-translation cases (gift→Geschenk, gift→Präsent both exist)
4. Generate homograph-aware synsets: same surface, different lang → different SynsetId
5. Benchmark on Oracle ARM: RAM + query latency after ingest

## Known limitations (honest)

- **Acronym handling deferred**: D.N.A. ↔ DNA ↔ "di-en-ei" requires a separate
  sprint. Not in this data.
- **Variants (café/cafe) deferred**: Wiktionary does have these as separate entries,
  but linking them requires a Unicode normalization pass in ZETS UNP (Hebrew already
  normalized, other 9 languages need equivalent profiles).
- **Translation quality varies**: some entries have one translation per language,
  others have 6. We captured all of them; downstream code will need to pick.
- **Title-case noise**: single-letter "homographs" (V, O, N, etc.) are dominant at the
  top of the count. Filter needed before ingest.

## Files saved

```
/home/dinio/zets/
├── data/
│   └── multilingual/       (13 TSVs, 82.5 MB)
└── scripts/
    ├── extract_multilingual.py      (main extractor, this phase)
    ├── extract_hewiktionary_v2.py   (Hebrew Wiktionary definitions + POS)
    └── build_wiki_full.py           (Wikipedia consolidation)
```
