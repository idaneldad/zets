#!/usr/bin/env python3
"""V2: extract definitions (lines starting with #) + POS + translations properly."""
import bz2
import re
import xml.etree.ElementTree as ET
from pathlib import Path

SOURCE = "/home/dinio/cortex-v7/data/wiki/hewiktionary.xml.bz2"
OUT_DIR = Path("/home/dinio/zets/data/hebrew/wiktionary")
OUT_DIR.mkdir(parents=True, exist_ok=True)

# POS extraction from {{ניתוח דקדוקי|...|חלק דיבר=שם־עצם|...}}
POS_PATTERN = re.compile(r'חלק\s*דיבר\s*=\s*([^\|\n}]+)')
# Translation: {{ת/he|en|dog}} or {{ת|en|dog}}
TRANS_PATTERN = re.compile(r'\{\{ת(?:/he)?\|([a-z]{2,3})\|([^\|\}]+)')
# Definition lines start with # (but not #: which are citations)
DEF_LINE = re.compile(r'^#\s+([^\n#].*?)(?:\n|$)', re.MULTILINE)

# Wiki markup cleanup
WIKI_LINK = re.compile(r'\[\[(?:[^|\]]+\|)?([^\]]+)\]\]')
TEMPLATE_SIMPLE = re.compile(r'\{\{[^{}]*?\}\}')
BOLD_ITALIC = re.compile(r"'{2,}")
HTML_COMMENT = re.compile(r'<!--.*?-->', re.DOTALL)
HTML_TAG = re.compile(r'<[^>]+>')

def clean(text):
    text = HTML_COMMENT.sub('', text)
    # Remove templates iteratively (nested)
    for _ in range(5):
        new = TEMPLATE_SIMPLE.sub('', text)
        if new == text: break
        text = new
    text = WIKI_LINK.sub(r'\1', text)
    text = BOLD_ITALIC.sub('', text)
    text = HTML_TAG.sub('', text)
    text = re.sub(r'\s+', ' ', text).strip()
    return text

NS = "{http://www.mediawiki.org/xml/export-0.11/}"

definitions = open(OUT_DIR / "definitions.tsv", 'w', encoding='utf-8')
pos_table = open(OUT_DIR / "pos.tsv", 'w', encoding='utf-8')
translations = open(OUT_DIR / "translations.tsv", 'w', encoding='utf-8')

definitions.write("# word\tdefinition\n")
pos_table.write("# word\tpos\n")
translations.write("# hebrew\tlang\tforeign\n")

stats = {'pages': 0, 'relevant': 0, 'defs': 0, 'pos': 0, 'trans': 0}

with bz2.open(SOURCE, 'rb') as f:
    for event, elem in ET.iterparse(f, events=('end',)):
        if elem.tag != NS + 'page':
            continue
        stats['pages'] += 1
        if stats['pages'] % 10000 == 0:
            print(f"  {stats['pages']:,} pages scanned...")

        ns_el = elem.find(NS + 'ns')
        title_el = elem.find(NS + 'title')
        rev = elem.find(NS + 'revision')

        if ns_el is None or ns_el.text != '0' or title_el is None or rev is None:
            elem.clear(); continue

        title = title_el.text.strip() if title_el.text else ''
        if ':' in title or '/' in title or not title or not ('\u05d0' <= title[0] <= '\u05ea'):
            elem.clear(); continue

        text_el = rev.find(NS + 'text')
        if text_el is None or not text_el.text:
            elem.clear(); continue

        wikitext = text_el.text
        stats['relevant'] += 1

        # Extract POS (only first occurrence per page)
        pos_match = POS_PATTERN.search(wikitext)
        if pos_match:
            pos = clean(pos_match.group(1))
            if pos and len(pos) <= 40:
                pos_table.write(f"{title}\t{pos}\n")
                stats['pos'] += 1

        # Extract definition lines (first definition only, to keep data lean)
        def_matches = DEF_LINE.findall(wikitext)
        for raw_def in def_matches[:3]:  # up to 3 senses
            d = clean(raw_def)
            if d and 5 <= len(d) <= 400:
                definitions.write(f"{title}\t{d}\n")
                stats['defs'] += 1

        # Extract translations
        for m in TRANS_PATTERN.finditer(wikitext):
            lang, word = m.group(1), clean(m.group(2))
            if word and 1 <= len(word) <= 60:
                translations.write(f"{title}\t{lang}\t{word}\n")
                stats['trans'] += 1

        elem.clear()

definitions.close()
pos_table.close()
translations.close()

print()
print("== Results ==")
for k, v in stats.items():
    print(f"  {k}: {v:,}")
import os
for name in ['definitions', 'pos', 'translations']:
    print(f"  {name}.tsv: {os.path.getsize(OUT_DIR / f'{name}.tsv'):,} bytes")
