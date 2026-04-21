#!/usr/bin/env python3
"""Convert all 69 wiki batches into a single consolidated TSV for ZETS ingestion."""
import os
import glob

INPUT_DIR = "/home/dinio/lev-knowledge/sources/wikipedia_he"
OUTPUT = "/home/dinio/zets/data/hebrew/wiki_full.tsv"
SYNSET_START = 5_000_000

files = sorted(glob.glob(f"{INPUT_DIR}/batch_*.tsv"))
print(f"Processing {len(files)} batches...")

written = 0
total_read = 0
unique_subjects = {}  # dedup by subject
synset_counter = SYNSET_START

with open(OUTPUT, 'w', encoding='utf-8') as out:
    out.write("# Full Hebrew Wikipedia ingested into ZETS\n")
    out.write("# Format: surface<TAB>synset_id<TAB>pos<TAB>english<TAB>definition<TAB>synonyms\n\n")

    for fpath in files:
        with open(fpath, 'r', encoding='utf-8', errors='replace') as f:
            for line in f:
                total_read += 1
                parts = line.rstrip('\n').split('\t')
                if len(parts) < 3:
                    continue
                subj = parts[0].strip()
                defn = parts[2].strip() if len(parts) > 2 else ""

                if not subj or any(ord(c) < 32 and c not in '\t\n' for c in subj):
                    continue

                # Dedup by subject — first occurrence wins
                if subj in unique_subjects:
                    continue
                unique_subjects[subj] = synset_counter

                # Sanitize
                defn = defn[:200].replace('\t', ' ').replace('\n', ' ')
                subj = subj.replace('\t', ' ').replace('\n', ' ')

                out.write(f"{subj}\t{synset_counter}\tnoun\t\t{defn}\t\n")
                synset_counter += 1
                written += 1

print(f"Read {total_read:,} rows")
print(f"Wrote {written:,} unique entries")
print(f"Output: {OUTPUT}")
print(f"Size: {os.path.getsize(OUTPUT):,} bytes = {os.path.getsize(OUTPUT)/1024/1024:.1f} MB")
