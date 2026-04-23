# ZETS — Content Download Scripts

**Purpose:** Ready-to-run commands for fetching every high-priority content source.
**Use:** Copy each block separately, paste to server. Each step is standalone and resumable.

---

## Prerequisites

```bash
mkdir -p /home/dinio/zets/data/downloads
cd /home/dinio/zets/data/downloads
```

---

## Step 1 — ConceptNet 5.7 (BIGGEST IMPACT)

**What:** 34M typed assertions across 80+ languages.
**Size:** 1 GB compressed, 4 GB uncompressed.
**Time:** ~5 min download + 10 min unzip.

```bash
cd /home/dinio/zets/data/downloads
wget -c https://conceptnet.s3.amazonaws.com/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz
ls -lh conceptnet-assertions-5.7.0.csv.gz
# Expected: ~1 GB
```

Sanity check first 10 rows:

```bash
zcat /home/dinio/zets/data/downloads/conceptnet-assertions-5.7.0.csv.gz | head -10
# Expected output format:
# /a/[/r/IsA/,/c/en/dog/,/c/en/mammal/]  /r/IsA  /c/en/dog  /c/en/mammal  {"weight":2.0}
```

Filter to our 10 target languages (saves 70% space):

```bash
cd /home/dinio/zets/data/downloads
zcat conceptnet-assertions-5.7.0.csv.gz | \
  awk -F'\t' 'BEGIN { OFS="\t" }
    {
      split($3, src_parts, "/")
      split($4, tgt_parts, "/")
      src_lang = src_parts[3]
      tgt_lang = tgt_parts[3]
      if ((src_lang == "en" || src_lang == "de" || src_lang == "fr" || src_lang == "es" ||
           src_lang == "it" || src_lang == "he" || src_lang == "ar" || src_lang == "ru" ||
           src_lang == "nl" || src_lang == "pt") &&
          (tgt_lang == "en" || tgt_lang == "de" || tgt_lang == "fr" || tgt_lang == "es" ||
           tgt_lang == "it" || tgt_lang == "he" || tgt_lang == "ar" || tgt_lang == "ru" ||
           tgt_lang == "nl" || tgt_lang == "pt"))
        print $2, $3, $4, $5
    }' > conceptnet_10langs.tsv
wc -l conceptnet_10langs.tsv
# Expected: ~10M rows
du -sh conceptnet_10langs.tsv
# Expected: ~800 MB
```

---

## Step 2 — Simple English Wikipedia infoboxes (STRUCTURED FACTS)

**What:** 200K articles with structured data fields.
**Size:** 329 MB compressed (already on server).
**Time:** 20-30 min to process.

Source already present: `/home/dinio/cortex-v7/data/wiki/simplewiki.xml.bz2`

Python extractor to write:

```bash
cat > /tmp/extract_infoboxes.py << 'PYEOF'
#!/usr/bin/env python3
"""Extract infoboxes from Simple English Wikipedia → typed graph edges."""
import bz2
import re
import xml.etree.ElementTree as ET
from pathlib import Path

SOURCE = "/home/dinio/cortex-v7/data/wiki/simplewiki.xml.bz2"
OUT = Path("/home/dinio/zets/data/encyclopedic/simplewiki_infoboxes.tsv")
OUT.parent.mkdir(parents=True, exist_ok=True)

# Infobox template pattern
INFOBOX_RE = re.compile(r'\{\{\s*[Ii]nfobox\s+([^\n|}]+)(.*?)\n\}\}', re.DOTALL)
FIELD_RE = re.compile(r'^\s*\|\s*([a-zA-Z_][a-zA-Z0-9_\-]*)\s*=\s*([^\n|]+?)\s*$', re.MULTILINE)

# Wiki markup cleanup
WIKI_LINK = re.compile(r'\[\[(?:[^|\]]+\|)?([^\]]+)\]\]')
TEMPLATE = re.compile(r'\{\{[^{}]*?\}\}')
HTML = re.compile(r'<[^>]+>')

def clean(t):
    t = WIKI_LINK.sub(r'\1', t)
    t = TEMPLATE.sub('', t)
    t = HTML.sub('', t)
    t = re.sub(r"'{2,}", '', t)
    return re.sub(r'\s+', ' ', t).strip()

NS = "{http://www.mediawiki.org/xml/export-0.11/}"

with bz2.open(SOURCE, 'rb') as f, open(OUT, 'w', encoding='utf-8') as out:
    out.write("# article\tinfobox_type\tfield\tvalue\n")
    pages, infoboxes, fields = 0, 0, 0

    for event, elem in ET.iterparse(f, events=('end',)):
        if elem.tag != NS + 'page':
            continue
        pages += 1
        if pages % 20000 == 0:
            print(f"  {pages:,} pages, {infoboxes:,} infoboxes, {fields:,} fields", flush=True)

        ns_el = elem.find(NS + 'ns')
        if ns_el is None or ns_el.text != '0':
            elem.clear(); continue
        title_el = elem.find(NS + 'title')
        rev = elem.find(NS + 'revision')
        if title_el is None or rev is None:
            elem.clear(); continue
        title = (title_el.text or '').strip()
        if ':' in title or '/' in title:
            elem.clear(); continue
        text_el = rev.find(NS + 'text')
        if text_el is None or not text_el.text:
            elem.clear(); continue

        for m in INFOBOX_RE.finditer(text_el.text):
            infoboxes += 1
            box_type = clean(m.group(1))[:40]
            body = m.group(2)
            for fm in FIELD_RE.finditer(body):
                field = fm.group(1).strip()[:40]
                value = clean(fm.group(2))[:200]
                if value and len(value) >= 2:
                    out.write(f"{title}\t{box_type}\t{field}\t{value}\n")
                    fields += 1
        elem.clear()

print(f"\nDone. {pages:,} pages, {infoboxes:,} infoboxes, {fields:,} field rows")
print(f"Output: {OUT} ({OUT.stat().st_size:,} bytes)")
PYEOF
python3 /tmp/extract_infoboxes.py
```

Expected yield: ~2-3M rows of structured facts.

---

## Step 3 — WordNet (English semantic depth)

**What:** 117K English words, 82K synsets, typed relations.
**Size:** 12 MB compressed.
**Time:** 5 min.

```bash
cd /home/dinio/zets/data/downloads
wget -c https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz
tar xzf wn3.1.dict.tar.gz
ls dict/
# Should contain: data.noun, data.verb, data.adj, data.adv, index.*
```

---

## Step 4 — Process existing rav_mekubal data (HEBREW DEPTH)

**What:** 98 MB of Jewish religious texts, commentaries, Kabbalah.
**Already on server:** `/home/dinio/rav_mekubal/data/`
**Time:** 2-3 hours extraction.

Sample inspection:

```bash
head -3 /home/dinio/rav_mekubal/data/cortex_import.jsonl | python3 -c "
import json, sys
for line in sys.stdin:
    d = json.loads(line)
    print('Keys:', list(d.keys())[:10])
    print('Sample:', {k: str(v)[:80] for k, v in list(d.items())[:5]})
    print('---')
"
```

Extraction script (to write next session):

```bash
# Task for next session:
# /tmp/extract_rav_mekubal.py — parse JSONL → graph edges
# Expected yield: 50K-200K Hebrew semantic edges
```

---

## Step 5 — Full Hebrew Wikipedia infoboxes

**What:** 400K Hebrew articles with infoboxes.
**Source on server:** `/home/dinio/cortex-v7/data/wiki/hewiki-latest.xml.bz2` (1.1 GB)
**Time:** 45-60 min.

Reuse Step 2 script but change SOURCE and OUT path:

```python
SOURCE = "/home/dinio/cortex-v7/data/wiki/hewiki-latest.xml.bz2"
OUT = Path("/home/dinio/zets/data/encyclopedic/hewiki_infoboxes.tsv")
```

Expected yield: ~1-2M rows of Hebrew structured facts.

---

## Step 6 — Tanakh as sequential Flows

**What:** 39 books as ordered sequences of verses.
**Source on server:** `/home/dinio/cortex-v7/data/tanakh/*.txt`
**Time:** 30 min.

```bash
ls /home/dinio/cortex-v7/data/tanakh/*.txt | wc -l
# Should be 53 files (includes commentaries)

# Count total Hebrew characters across all books
cat /home/dinio/cortex-v7/data/tanakh/*.txt | wc -c
# ~6M characters
```

Extraction: treat each verse as a node, chain them with `NextInFlow` edges. Cross-reference by gematria.

---

## Step 7 (OPTIONAL) — Tatoeba multilingual sentence pairs

**What:** Community-translated sentence pairs for 400+ languages.
**Size:** ~200 MB.
**Time:** 30 min.

```bash
cd /home/dinio/zets/data/downloads
wget -c https://downloads.tatoeba.org/exports/sentences.tar.bz2
wget -c https://downloads.tatoeba.org/exports/links.tar.bz2
tar xjf sentences.tar.bz2
tar xjf links.tar.bz2
head sentences.csv
```

Value: provides **real usage examples** of every word, not just dictionary definitions.

---

## Step 8 (DEFER) — Wikidata on-demand

Full Wikidata dump is 90 GB. **Don't download it.**

Instead, query SPARQL endpoint when specific entities needed:

```bash
# Example — get all humans born in Israel with profession:
curl -s -G 'https://query.wikidata.org/sparql' \
  --data-urlencode 'format=json' \
  --data-urlencode 'query=
    SELECT ?person ?personLabel ?occupationLabel WHERE {
      ?person wdt:P31 wd:Q5.
      ?person wdt:P19/wdt:P131* wd:Q801.
      ?person wdt:P106 ?occupation.
      SERVICE wikibase:label { bd:serviceParam wikibase:language "he,en". }
    }
    LIMIT 100'
```

This returns targeted facts on demand, without 90GB storage burden.

---

## Storage budget after all downloads

| Source | Compressed | Processed to TSV |
|--------|-----------:|-----------------:|
| Wiktionary 10 langs | already processed | 44 MB ✅ |
| ConceptNet (filtered) | 1 GB | 800 MB |
| Simple Wikipedia infoboxes | (from 329 MB) | 300 MB |
| WordNet | 12 MB | 50 MB |
| rav_mekubal parsed | (from 98 MB) | 80 MB |
| Hebrew Wikipedia infoboxes | (from 1.1 GB) | 400 MB |
| Tanakh Flows | (from 6 MB) | 15 MB |
| Tatoeba | 200 MB | 300 MB |
| **TOTAL** | **~2.7 GB** | **~2 GB processed** |

Server has 42 GB RAM + plenty of disk. Budget is fine.

---

## The one-shot run (when ready)

Copy-paste this single block to execute Steps 1-3 in sequence:

```bash
cd /home/dinio/zets/data/downloads

echo "=== Step 1: ConceptNet ===" 
wget -c https://conceptnet.s3.amazonaws.com/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz

echo "=== Step 2: Simple Wikipedia already on server ===" 
python3 /tmp/extract_infoboxes.py

echo "=== Step 3: WordNet ===" 
wget -c https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz
tar xzf wn3.1.dict.tar.gz

echo "=== All base downloads done ===" 
du -sh /home/dinio/zets/data/
```

Expected total runtime: 1-2 hours.

---

**End of download scripts.**
