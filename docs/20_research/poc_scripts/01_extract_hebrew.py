import gzip, json, re
from collections import Counter

WIKI = "/home/dinio/zets/data/wikipedia_dumps/he_parsed.jsonl.gz"
N = 3000  # smaller batch for speed

WORD_RE = re.compile(r'[\u05D0-\u05EA]+')
finals = str.maketrans('ךםןףץ', 'כמנפצ')

freq = Counter()
done = 0
with gzip.open(WIKI, 'rt', encoding='utf-8') as f:
    for line in f:
        if done >= N: break
        try:
            d = json.loads(line)
            t = d.get('text', '')
            for w in WORD_RE.findall(t):
                if 2 <= len(w) <= 12:
                    freq[w.translate(finals)] += 1
            done += 1
        except: pass

print(f"Articles: {done}, unique words: {len(freq)}, total tokens: {sum(freq.values())}")
with open('/home/dinio/poc/freq.tsv', 'w', encoding='utf-8') as f:
    for r, (w, c) in enumerate(freq.most_common(20000), 1):
        f.write(f"{r}\t{w}\t{c}\n")
print("wrote freq.tsv")
