#!/usr/bin/env python3
"""Extract multilingual data from English Wiktionary dump (1.5GB bz2).

Focus on CHALLENGE CASES Idan identified:
1. Homographs across languages (Gift = present/poison/wedding)
2. Variants (café/cafe, größe/groesse)
3. Cross-language translations (10 target languages)

Strategy: Single streaming pass over 1.5GB XML.
For each page (entry), extract:
- All language sections (e.g. English, German, Danish for "gift")
- Translations table
- Definitions per language

Output: multiple TSVs ready for ingest.
Target languages: en, he, ar, de, fr, es, ru, zh, ja, hi
"""
import bz2
import re
import xml.etree.ElementTree as ET
from pathlib import Path
import time

SOURCE = "/home/dinio/cortex-v7/enwiktionary.xml.bz2"
OUT_DIR = Path("/home/dinio/zets/data/multilingual")
OUT_DIR.mkdir(parents=True, exist_ok=True)

# 10 target languages — the core of our multilingual test
TARGET_LANGS = {
    'en': 'English',
    'he': 'Hebrew',
    'ar': 'Arabic',
    'de': 'German',
    'fr': 'French',
    'es': 'Spanish',
    'ru': 'Russian',
    'zh': 'Chinese',
    'ja': 'Japanese',
    'hi': 'Hindi',
}

# Build pattern for any language we care about
LANG_HEADER_PATTERNS = {}
for code, name in TARGET_LANGS.items():
    LANG_HEADER_PATTERNS[code] = re.compile(
        rf'^==\s*{re.escape(name)}\s*==\s*$', re.MULTILINE
    )

# Regex to find ANY language header (for splitting)
ANY_LANG_HEADER = re.compile(r'^==\s*([A-Z][a-zA-Z\s-]+)\s*==\s*$', re.MULTILINE)

# Definition lines: "# definition text" (not "#:" quotes, not "#*" examples)
DEF_LINE = re.compile(r'^#\s+([^#\n].*?)(?=\n|$)', re.MULTILINE)

# Translation pattern in "====Translations====" section
# Formats: {{t|code|word}}, {{t+|code|word}}, {{tt|code|word}}, {{tt+|code|word}}
TRANS_PATTERN = re.compile(
    r'\{\{tt?\+?\|(' + '|'.join(TARGET_LANGS.keys()) + r')\|([^\|\}]+)'
)

TRANS_SECTION = re.compile(r'====\s*Translations\s*====(.*?)(?=\n==|\Z)', re.DOTALL)

# Wiki markup cleanup
WIKI_LINK = re.compile(r'\[\[(?:[^|\]]+\|)?([^\]]+)\]\]')
TEMPLATE_INNER = re.compile(r'\{\{[^{}]*?\}\}')
BOLD_ITALIC = re.compile(r"'{2,}")
HTML_COMMENT = re.compile(r'<!--.*?-->', re.DOTALL)
HTML_TAG = re.compile(r'<[^>]+>')

def clean(text):
    text = HTML_COMMENT.sub('', text)
    for _ in range(3):
        new = TEMPLATE_INNER.sub('', text)
        if new == text:
            break
        text = new
    text = WIKI_LINK.sub(r'\1', text)
    text = BOLD_ITALIC.sub('', text)
    text = HTML_TAG.sub('', text)
    text = re.sub(r'\s+', ' ', text).strip()
    return text

def split_by_language(text):
    """Return dict: {lang_code: section_text}"""
    sections = {}
    # Find all language headers
    all_headers = list(ANY_LANG_HEADER.finditer(text))
    for i, match in enumerate(all_headers):
        lang_name = match.group(1).strip()
        # Find matching target language
        target_code = None
        for code, name in TARGET_LANGS.items():
            if lang_name == name:
                target_code = code
                break
        if target_code is None:
            continue
        start = match.end()
        end = all_headers[i + 1].start() if i + 1 < len(all_headers) else len(text)
        sections[target_code] = text[start:end]
    return sections

def extract_definitions(section, limit=5):
    """Extract up to `limit` definition lines from a section."""
    defs = []
    for match in DEF_LINE.finditer(section):
        raw = match.group(1)
        # Skip lines that are mostly markup (etymologies, redirects)
        if raw.startswith(('{{', '[[Category')):
            continue
        cleaned = clean(raw)
        if cleaned and 3 <= len(cleaned) <= 300:
            defs.append(cleaned)
        if len(defs) >= limit:
            break
    return defs

def extract_translations(section):
    """Extract (lang, translation) pairs from Translations section(s)."""
    results = []
    for trans_section_match in TRANS_SECTION.finditer(section):
        trans_text = trans_section_match.group(1)
        for m in TRANS_PATTERN.finditer(trans_text):
            lang = m.group(1)
            word = clean(m.group(2))
            if word and 1 <= len(word) <= 60:
                results.append((lang, word))
    return results

# ============ Main streaming parse ============
NS = "{http://www.mediawiki.org/xml/export-0.11/}"

# Open output files
f_homographs = open(OUT_DIR / "homographs.tsv", 'w', encoding='utf-8')
f_definitions = open(OUT_DIR / "definitions_en.tsv", 'w', encoding='utf-8')
f_translations = open(OUT_DIR / "translations.tsv", 'w', encoding='utf-8')
f_entries = {code: open(OUT_DIR / f"entry_{code}.tsv", 'w', encoding='utf-8')
             for code in TARGET_LANGS}

f_homographs.write("# surface_form\tnum_languages\tlanguages\n")
f_definitions.write("# word\tdefinition\n")
f_translations.write("# source_word\tsource_lang\ttarget_lang\ttarget_word\n")
for f in f_entries.values():
    f.write("# surface\tlang\tfirst_definition\n")

stats = {
    'pages': 0,
    'multilingual_surface_forms': 0,
    'definitions_written': 0,
    'translations_written': 0,
    'entries_per_lang': {code: 0 for code in TARGET_LANGS},
}

start_time = time.time()

print(f"Streaming {SOURCE}...")
print(f"Target languages: {list(TARGET_LANGS.keys())}")

with bz2.open(SOURCE, 'rb') as f:
    for event, elem in ET.iterparse(f, events=('end',)):
        if elem.tag != NS + 'page':
            continue
        stats['pages'] += 1

        if stats['pages'] % 50000 == 0:
            elapsed = time.time() - start_time
            rate = stats['pages'] / elapsed
            homographs = stats['multilingual_surface_forms']
            trans = stats['translations_written']
            print(f"  {stats['pages']:>8,} pages in {elapsed:>6.1f}s "
                  f"({rate:>5.0f}/s) | homographs={homographs:,} trans={trans:,}")

        ns_el = elem.find(NS + 'ns')
        if ns_el is None or ns_el.text != '0':
            elem.clear(); continue

        title_el = elem.find(NS + 'title')
        if title_el is None or not title_el.text:
            elem.clear(); continue
        title = title_el.text.strip()
        if ':' in title or '/' in title or '#' in title:
            elem.clear(); continue

        rev = elem.find(NS + 'revision')
        if rev is None:
            elem.clear(); continue
        text_el = rev.find(NS + 'text')
        if text_el is None or not text_el.text:
            elem.clear(); continue

        wikitext = text_el.text
        sections = split_by_language(wikitext)
        if not sections:
            elem.clear(); continue

        # Track homographs
        if len(sections) >= 2:
            stats['multilingual_surface_forms'] += 1
            langs = ','.join(sorted(sections.keys()))
            f_homographs.write(f"{title}\t{len(sections)}\t{langs}\n")

        # Per-language entries
        for code, section_text in sections.items():
            stats['entries_per_lang'][code] += 1
            defs = extract_definitions(section_text, limit=3)
            if defs:
                f_entries[code].write(f"{title}\t{code}\t{defs[0]}\n")
                if code == 'en':
                    for d in defs:
                        f_definitions.write(f"{title}\t{d}\n")
                        stats['definitions_written'] += 1

            # Extract translations (usually only in English section)
            if code == 'en':
                trans = extract_translations(section_text)
                for (target_lang, target_word) in trans:
                    if target_lang != 'en':
                        f_translations.write(
                            f"{title}\ten\t{target_lang}\t{target_word}\n"
                        )
                        stats['translations_written'] += 1

        elem.clear()

# Close files
f_homographs.close()
f_definitions.close()
f_translations.close()
for f in f_entries.values():
    f.close()

elapsed = time.time() - start_time
print()
print("=" * 60)
print(f"DONE in {elapsed:.1f}s ({stats['pages']/elapsed:.0f} pages/sec)")
print(f"Total pages: {stats['pages']:,}")
print(f"Multilingual surface forms (homographs): {stats['multilingual_surface_forms']:,}")
print(f"English definitions written: {stats['definitions_written']:,}")
print(f"Translations written: {stats['translations_written']:,}")
print()
print("Entries per language:")
for code, count in stats['entries_per_lang'].items():
    name = TARGET_LANGS[code]
    print(f"  {code} ({name:8s}): {count:>8,}")

print()
print("Output files:")
import os
for p in sorted(OUT_DIR.glob("*.tsv")):
    size = os.path.getsize(p)
    print(f"  {p.name:30s} {size:>12,} bytes ({size/1024/1024:.1f} MB)")
