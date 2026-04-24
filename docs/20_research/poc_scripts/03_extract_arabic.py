"""Extract top Arabic words from Wikipedia."""
import gzip, json, re
from collections import Counter

WIKI = "/home/dinio/zets/data/wikipedia_dumps/ar_parsed.jsonl.gz"
N = 3000

# Arabic letters: U+0627-U+064A (ا-ي)
WORD_RE = re.compile(r'[\u0621-\u064A]+')
# Strip diacritics (tashkeel)
DIACRITICS = re.compile(r'[\u064B-\u0652\u0670]')

freq = Counter()
done = 0
with gzip.open(WIKI, 'rt', encoding='utf-8') as f:
    for line in f:
        if done >= N: break
        try:
            d = json.loads(line)
            t = DIACRITICS.sub('', d.get('text', ''))
            for w in WORD_RE.findall(t):
                if 2 <= len(w) <= 12:
                    freq[w] += 1
            done += 1
        except: pass

print(f"Articles: {done}, unique: {len(freq)}, tokens: {sum(freq.values())}")
with open('/home/dinio/poc/freq_ar.tsv', 'w', encoding='utf-8') as f:
    for r, (w, c) in enumerate(freq.most_common(20000), 1):
        f.write(f"{r}\t{w}\t{c}\n")
print("done")
