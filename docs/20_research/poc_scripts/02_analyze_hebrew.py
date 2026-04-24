"""POC v2: smarter algorithm that recognizes binyanim and weak roots."""
import re
from collections import Counter

PREFIX_LETTERS = list("ושהבלמכ")

# Binyan prefixes (added to root in specific binyanim)
# נפעל: ני- (passive of pa'al)
# הפעיל: ה- + י vowel
# התפעל: הת- (reflexive)
BINYAN_PATTERNS = [
    ("התה", 3),  # התהפך, התהלך
    ("הת", 2),   # התלבש, התרגש
    ("ני", 2),   # נילחם, נישמע
    ("נו", 2),   # נולד, נוסע
    ("מת", 2),   # מתלבש, מתרגש
    ("נ", 1),    # נכתב, נפטר (נפעל past 3sg masc)
    ("מ", 1),    # מכתב, מקום (mem nominal/participle)
    ("ת", 1),    # תכתוב (future 2sg)
    ("י", 1),    # יכתוב (future 3sg)
    ("א", 1),    # אכתוב (future 1sg)
]

SUFFIXES = [
    "ותיהם", "ותיהן", "תיכם", "תיכן",
    "יהם", "יהן",
    "כם", "כן", "הם", "הן", "נו", "תי", "תה", "תם", "תן",
    "ות", "ים", "ין",
    "יה", "יו", "יך", "ית",
    "ה", "ת", "י", "ך", "ם", "ן", "ו",
]

FUNCTION_WORDS = set("""
של את אל על מן עד אם או כי גם רק לא לו לי לה לך לכם
זה זאת אלה אלו הוא היא הם הן אני אתה את אנחנו אתם אתן
מה מי איך למה איפה מתי כן יש אין הנה הנו זהו
""".split())

words = []
with open('/home/dinio/poc/freq.tsv', encoding='utf-8') as f:
    for line in f:
        parts = line.strip().split('\t')
        if len(parts) == 3:
            _, w, c = parts
            words.append((w, int(c)))

def strip_prefix(word):
    """Strip up to 3 stackable prefix letters."""
    for n in range(min(4, len(word) - 2), -1, -1):
        if all(c in PREFIX_LETTERS for c in word[:n]):
            return word[:n], word[n:]
    return '', word

def strip_binyan_prefix(stem):
    """Strip a binyan prefix (more aggressive)."""
    for prefix, n in BINYAN_PATTERNS:
        if stem.startswith(prefix) and len(stem) - n >= 3:
            return prefix, stem[n:]
    return '', stem

def strip_suffix(stem):
    for suf in SUFFIXES:
        if stem.endswith(suf) and len(stem) - len(suf) >= 2:
            return stem[:-len(suf)], suf
    return stem, ''

def strip_internal_vowels(stem):
    """Try removing 'leftover' vowel letters in middle (yod, vav)."""
    # Hollow roots: ק.ו.ם → קם, קום, קים, קמים
    # If stem is 4 chars with vav/yod in position 2 or 3, try collapsing
    if len(stem) == 4 and stem[1] in 'וי':
        return stem[0] + stem[2:]  # remove the vav/yod (II-weak)
    if len(stem) == 4 and stem[2] in 'וי':
        return stem[:2] + stem[3]  # remove from middle-end
    return stem

def classify(word):
    if word in FUNCTION_WORDS:
        return 'function', None, None, None, None
    if len(word) < 2:
        return 'too_short', None, None, None, None
    
    prefix, after = strip_prefix(word)
    after, suffix = strip_suffix(after)
    binyan, root_candidate = strip_binyan_prefix(after)
    
    # If after binyan strip we're 3 chars → strong root
    if len(root_candidate) == 3:
        return 'root_3_clean', prefix, binyan, root_candidate, suffix
    
    # 4 chars after binyan strip → maybe weak root (collapsed vowel)
    if len(root_candidate) == 4:
        weak = strip_internal_vowels(root_candidate)
        if len(weak) == 3:
            return 'root_3_weak', prefix, binyan, weak, suffix
        return 'root_4', prefix, binyan, root_candidate, suffix
    
    # Try without binyan strip
    if len(after) == 3:
        return 'root_3_no_binyan', prefix, '', after, suffix
    if len(after) == 4:
        weak = strip_internal_vowels(after)
        if len(weak) == 3:
            return 'root_3_weak_no_binyan', prefix, '', weak, suffix
        return 'root_4_no_binyan', prefix, '', after, suffix
    
    if len(root_candidate) == 2:
        return 'root_2', prefix, binyan, root_candidate, suffix
    if len(root_candidate) == 5:
        return 'root_5', prefix, binyan, root_candidate, suffix
    return 'other', prefix, binyan, root_candidate, suffix

stats = Counter()
weighted = Counter()
roots = Counter()
binyanim = Counter()

for w, c in words:
    cls, pre, bin_, root, suf = classify(w)
    stats[cls] += 1
    weighted[cls] += c
    if root:
        roots[root] += c
    if bin_:
        binyanim[bin_] += c

total = sum(stats.values())
total_w = sum(weighted.values())

# Group categories
covered_cls = ('root_3_clean', 'root_3_weak', 'root_3_no_binyan', 'root_3_weak_no_binyan')
weakly_covered = ('root_4', 'root_4_no_binyan', 'root_2')

covered_unique = sum(stats[c] for c in covered_cls)
covered_w = sum(weighted[c] for c in covered_cls)
weakly_unique = sum(stats[c] for c in weakly_covered)
weakly_w = sum(weighted[c] for c in weakly_covered)

print(f"\n{'='*70}")
print(f"HEBREW MORPHOLOGY POC v2 — improved algorithm with binyan detection")
print(f"{'='*70}\n")
print(f"Corpus: 20,000 unique words, {total_w:,} tokens (top from 3K Hebrew Wiki articles)\n")

print(f"{'Class':<28}{'Unique':<10}{'%':<7}{'Tokens':<14}{'%':<7}")
print('-' * 70)
for cls, n in stats.most_common():
    n_w = weighted[cls]
    print(f"{cls:<28}{n:<10,}{100*n/total:<6.1f}%{n_w:<14,}{100*n_w/total_w:<6.1f}%")

print(f"\n{'='*70}")
print(f"BOTTOM LINE")
print(f"{'='*70}")
print(f"Strong 3-letter root coverage: {100*covered_unique/total:.1f}% unique, {100*covered_w/total_w:.1f}% tokens")
print(f"+ Weak (4-letter or 2-letter): {100*weakly_unique/total:.1f}% unique, {100*weakly_w/total_w:.1f}% tokens")
print(f"= TOTAL within Hebrew morph model: {100*(covered_unique+weakly_unique)/total:.1f}% unique, {100*(covered_w+weakly_w)/total_w:.1f}% tokens")
print(f"")
print(f"Function words: {100*weighted['function']/total_w:.1f}% of tokens")
print(f"Other (likely loanwords/names/wiki-markup-residue): {100*(stats['other']+stats['root_5']+stats['long_no_clear_root'])/total:.1f}% unique")

print(f"\n{'='*70}")
print(f"BINYAN DETECTION — what binyan prefixes were found")
print(f"{'='*70}")
for b, c in binyanim.most_common(15):
    print(f"  '{b}'  → {c:,} tokens")

print(f"\n{'='*70}")
print(f"DISTINCT 3-LETTER ROOTS — COMPRESSION POTENTIAL")
print(f"{'='*70}")
roots_3 = sorted([(r, roots[r]) for r in roots if len(r) == 3], key=lambda x: -x[1])
print(f"Distinct 3-letter roots: {len(roots_3):,}")
print(f"Top 30 by token frequency:")
for i, (r, c) in enumerate(roots_3[:30], 1):
    print(f"  {i:3}. {r}  ({c:,})")

print(f"\nBits needed for distinct roots seen: {len(roots_3).bit_length()}")
print(f"Bits for theoretical max (27^3=19,683):     15 bits")
print(f"Hebrew's REAL active roots (linguists say): ~2,500 → 12 bits sufficient")
