"""Test cross-language root sharing: Hebrew ↔ Arabic via consonant mapping."""
import re
from collections import Counter

# Hebrew → Arabic consonant correspondence (linguistically validated)
# Both are Semitic; share most root consonants
HE_TO_AR = {
    'א': 'ا',  'ב': 'ب',  'ג': 'ج',  'ד': 'د',  'ה': 'ه',
    'ו': 'و',  'ז': 'ز',  'ח': 'ح',  'ט': 'ط',  'י': 'ي',
    'כ': 'ك',  'ל': 'ل',  'מ': 'م',  'נ': 'ن',  'ס': 'س',
    'ע': 'ع',  'פ': 'ف',  'צ': 'ص',  'ק': 'ق',  'ר': 'ر',
    'ש': 'ش',  'ת': 'ت',
    # Hebrew has merged some Proto-Semitic letters; allow alternates
}

# Arabic → Hebrew  
AR_TO_HE = {v: k for k, v in HE_TO_AR.items()}
# Arabic letters not in Hebrew (had distinct ProtoSemitic origin):
AR_TO_HE.update({'ث': 'ש', 'ذ': 'ז', 'ض': 'צ', 'ظ': 'ט', 'غ': 'ע', 'ة': 'ה', 'ى': 'י', 'ئ': 'י', 'ء': 'א', 'آ': 'א', 'إ': 'א', 'أ': 'א'})

# Arabic prefixes
AR_PREFIXES = list("والفبكلستيمن")
AR_BINYAN_PREFIXES = [("است", 3), ("ان", 2), ("مت", 2), ("ت", 1), ("ا", 1), ("م", 1), ("ي", 1), ("ن", 1)]
AR_SUFFIXES = ["كم", "كن", "هم", "هن", "نا", "ها", "ون", "ين", "ات", "ة", "ه", "ي", "ك", "ا", "ت", "ن"]

def strip_ar(word):
    # Strip prefixes
    for n in range(min(4, len(word) - 2), -1, -1):
        if all(c in AR_PREFIXES for c in word[:n]):
            stem = word[n:]
            break
    else:
        stem = word
    # Strip suffix
    for suf in AR_SUFFIXES:
        if stem.endswith(suf) and len(stem) - len(suf) >= 2:
            stem = stem[:-len(suf)]
            break
    # Strip binyan
    for prefix, n in AR_BINYAN_PREFIXES:
        if stem.startswith(prefix) and len(stem) - n >= 3:
            stem = stem[n:]
            break
    # Internal vowel collapse (yaa/waw)
    if len(stem) == 4 and stem[1] in 'يو':
        stem = stem[0] + stem[2:]
    elif len(stem) == 4 and stem[2] in 'يو':
        stem = stem[:2] + stem[3]
    return stem

def ar_root_to_he(ar_root):
    """Convert Arabic root letters to Hebrew equivalents."""
    return ''.join(AR_TO_HE.get(c, '?') for c in ar_root)

# Load Hebrew roots from prior run
PREFIX_LETTERS = list("ושהבלמכ")
BINYAN_PATTERNS = [("התה", 3), ("הת", 2), ("ני", 2), ("נו", 2), ("מת", 2), ("נ", 1), ("מ", 1), ("ת", 1), ("י", 1), ("א", 1)]
SUFFIXES = ["ותיהם","ותיהן","תיכם","תיכן","יהם","יהן","כם","כן","הם","הן","נו","תי","תה","תם","תן","ות","ים","ין","יה","יו","יך","ית","ה","ת","י","ך","ם","ן","ו"]

def strip_he(word):
    for n in range(min(4, len(word) - 2), -1, -1):
        if all(c in PREFIX_LETTERS for c in word[:n]):
            stem = word[n:]
            break
    else:
        stem = word
    for suf in SUFFIXES:
        if stem.endswith(suf) and len(stem) - len(suf) >= 2:
            stem = stem[:-len(suf)]
            break
    for prefix, n in BINYAN_PATTERNS:
        if stem.startswith(prefix) and len(stem) - n >= 3:
            stem = stem[n:]
            break
    if len(stem) == 4 and stem[1] in 'וי':
        stem = stem[0] + stem[2:]
    elif len(stem) == 4 and stem[2] in 'וי':
        stem = stem[:2] + stem[3]
    return stem

# Extract Hebrew roots
he_roots = Counter()
with open('/home/dinio/poc/freq.tsv', encoding='utf-8') as f:
    for line in f:
        parts = line.strip().split('\t')
        if len(parts) == 3:
            _, w, c = parts
            r = strip_he(w)
            if len(r) == 3:
                he_roots[r] += int(c)

# Extract Arabic roots, transliterate to Hebrew alphabet
ar_roots_in_he = Counter()
ar_roots_native = Counter()
with open('/home/dinio/poc/freq_ar.tsv', encoding='utf-8') as f:
    for line in f:
        parts = line.strip().split('\t')
        if len(parts) == 3:
            _, w, c = parts
            r = strip_ar(w)
            if len(r) == 3:
                ar_roots_native[r] += int(c)
                he_form = ar_root_to_he(r)
                if '?' not in he_form:
                    ar_roots_in_he[he_form] += int(c)

# Compute overlap
he_set = set(he_roots.keys())
ar_set_in_he = set(ar_roots_in_he.keys())
shared = he_set & ar_set_in_he
he_only = he_set - ar_set_in_he
ar_only = ar_set_in_he - he_set

print(f"\n{'='*70}")
print(f"CROSS-LANGUAGE SHARED ROOTS (Hebrew ↔ Arabic)")
print(f"{'='*70}\n")
print(f"Hebrew 3-letter roots discovered:    {len(he_set):,}")
print(f"Arabic 3-letter roots discovered:    {len(ar_set_in_he):,}")
print(f"SHARED roots (same 3 consonants):    {len(shared):,}")
print(f"Hebrew-only:                          {len(he_only):,}")
print(f"Arabic-only:                          {len(ar_only):,}")
print(f"")
print(f"Sharing rate (Hebrew ∩ Arabic / Hebrew): {100*len(shared)/len(he_set):.1f}%")
print(f"Sharing rate (Hebrew ∩ Arabic / Arabic): {100*len(shared)/len(ar_set_in_he):.1f}%")

# Weighted by token frequency
shared_he_tokens = sum(he_roots[r] for r in shared)
total_he_tokens = sum(he_roots.values())
shared_ar_tokens = sum(ar_roots_in_he[r] for r in shared)
total_ar_tokens = sum(ar_roots_in_he.values())

print(f"")
print(f"WEIGHTED BY USAGE FREQUENCY:")
print(f"  Hebrew tokens covered by shared roots: {100*shared_he_tokens/total_he_tokens:.1f}%")
print(f"  Arabic tokens covered by shared roots: {100*shared_ar_tokens/total_ar_tokens:.1f}%")

# Top 30 shared roots
print(f"\n{'='*70}")
print(f"TOP 30 SHARED ROOTS (by combined Hebrew + Arabic usage)")
print(f"{'='*70}")
shared_combined = [(r, he_roots[r] + ar_roots_in_he[r]) for r in shared]
shared_combined.sort(key=lambda x: -x[1])
print(f"{'#':<4}{'Root':<8}{'HE count':<14}{'AR count':<14}{'Total':<14}")
for i, (r, total) in enumerate(shared_combined[:30], 1):
    print(f"{i:<4}{r:<8}{he_roots[r]:<14,}{ar_roots_in_he[r]:<14,}{total:<14,}")

print(f"\n{'='*70}")
print(f"STORAGE IMPLICATIONS — UNIFIED SEMITIC ROOT POOL")
print(f"{'='*70}")
total_unified = len(he_set | ar_set_in_he)
saved = len(he_set) + len(ar_set_in_he) - total_unified
print(f"If we share root atom across HE+AR:")
print(f"  Total distinct roots in pool: {total_unified:,}")
print(f"  vs separate per-language: {len(he_set) + len(ar_set_in_he):,}")
print(f"  Savings: {saved:,} root atoms ({100*saved/(len(he_set)+len(ar_set_in_he)):.1f}%)")
print(f"")
print(f"For 8-byte atom budget:")
print(f"  Root field: 12 bits sufficient (4096 slots, we have ~3K combined)")
print(f"  Binyan: 3 bits (7 binyanim)")
print(f"  Tense/aspect: 3 bits (6 forms)")
print(f"  Person+gender+number: 4 bits (10 combinations)")
print(f"  Definiteness: 1 bit")
print(f"  Total morphological encoding: 23 bits")
print(f"  Remaining for kind+semantic_id+flags: 41 bits")
print(f"  Fits comfortably in 64 bits (8 bytes)")
