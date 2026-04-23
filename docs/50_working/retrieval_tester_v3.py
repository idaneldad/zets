#!/usr/bin/env python3
"""
retrieval_tester_v3.py — גרסה שלישית, פונה לשכבת ה-part_of.

ממצאי v2:
  * 87% מהedges = co_occurs_with (רעש לרוב)
  * 12% = part_of (word→sentence)  — זה הבסיס ל-retrieval אמיתי
  * 0.14% = is_a (18K edges)        — ידע מבני קטן אבל אמיתי

אסטרטגיה חדשה:
  1. טוקניזציה של השאלה → seed words (word:paris, word:france)
  2. לכל seed word: למצוא את כל ה-sentences שהיא part_of (word --part_of--> sentence)
  3. לצבור — sentence שמופיעה אצל הרבה seeds = relevant sentence
  4. עבור ל-source (source:wikipedia:X) שה-sentence שייכת לו
  5. תשובה = שם ה-source המדורג + רשימת sentence labels

זה BM25-lite מעל הגרף. לא עונה במשפט טבעי — אבל מצביע על הidan-article הנכון.
"""
import struct, time, re
from collections import defaultdict, Counter

def load_all(path):
    t0 = time.time()
    with open(path, "rb") as f:
        data = f.read()
    off = 4 + 4
    (atom_count,) = struct.unpack_from("<Q", data, off); off += 8
    (edge_count,) = struct.unpack_from("<Q", data, off); off += 8

    atoms = []
    for _ in range(atom_count):
        kind = data[off]; off += 1
        (dlen,) = struct.unpack_from("<I", data, off); off += 4
        s = data[off:off+dlen].decode("utf-8", errors="replace"); off += dlen
        off += 8
        atoms.append((kind, s))

    # edges → 2 indexes: out (all) + part_of reverse (sentence → [words])
    out_adj = [[] for _ in range(atom_count)]
    # word --part_of--> sentence   ⇒  in_adj_partof[sentence] = [words]
    in_partof = defaultdict(list)
    fmt = "<IIBBH"; sz = struct.calcsize(fmt)
    for _ in range(edge_count):
        f, t, r, w, s = struct.unpack_from(fmt, data, off); off += sz
        out_adj[f].append((t, r, w))
        if r == 0x05:  # part_of
            in_partof[t].append((f, w))

    print(f"  loaded: atoms={len(atoms):,}, edges={edge_count:,}, part_of→{sum(len(v) for v in in_partof.values()):,}")
    return atoms, out_adj, in_partof


def build_word_index(atoms):
    word_ix = {}
    source_ix = {}
    sent_to_source = {}  # sentence_id → source name (e.g., "Heart")
    for i, (k, s) in enumerate(atoms):
        if k == 0:
            if s.startswith("word:"):
                word_ix[s[5:].lower()] = i
            elif s.startswith("source:wikipedia:"):
                source_ix[s[len("source:wikipedia:"):]] = i
        elif k == 1 and s.startswith("sent:wikipedia:"):
            # "sent:wikipedia:Heart:3" → article="Heart"
            parts = s.split(":")
            if len(parts) >= 3:
                sent_to_source[i] = parts[2]
    return word_ix, source_ix, sent_to_source


STOP_EN = {"the","a","an","is","are","was","were","what","who","where","when","how","why",
           "of","in","on","at","to","from","by","with","for","and","or","but","not","do",
           "does","did","have","has","had","can","could","would","should","will","this",
           "that","these","those","it","its","my","your","tell","me","about"}
STOP_HE = {"מה","מי","איפה","מתי","איך","למה","של","את","זה","זאת","זו","יש","אין"}

def tokenize(text):
    t = re.sub(r"[^\w\s]", " ", text.lower(), flags=re.UNICODE)
    return [w for w in t.split() if w and w not in STOP_EN and w not in STOP_HE and len(w) > 1]


def retrieve(q, atoms, word_ix, source_ix, in_partof, sent_to_source, top_k_sents=5, top_k_articles=3):
    tokens = tokenize(q)
    seed_ids = [word_ix[t] for t in tokens if t in word_ix]

    if not seed_ids:
        return {"tokens": tokens, "seeds": [], "sentences": [], "articles": []}

    # לכל sentence — כמה מהseeds מקושרות אליה כ-part_of?
    # צריך reverse: מה-word למקום sentences שהיא part_of.
    # יש לנו in_partof[sent_id] = [(word_id, weight)]
    # אז: ל-sentence s, seeds שנמצאות בה = {wid for (wid,w) in in_partof[s] if wid in seed_ids_set}
    seed_set = set(seed_ids)
    sent_scores = Counter()
    sent_hits = defaultdict(list)  # sent_id → [word_ids]
    for sent_id, parts in in_partof.items():
        matched = [wid for (wid, w) in parts if wid in seed_set]
        if matched:
            sent_scores[sent_id] = len(matched)  # מספר seeds במשפט
            sent_hits[sent_id] = matched

    # הקבץ לפי article
    article_scores = Counter()
    for sid, score in sent_scores.items():
        art = sent_to_source.get(sid)
        if art:
            article_scores[art] += score

    top_sents = sent_scores.most_common(top_k_sents)
    top_articles = article_scores.most_common(top_k_articles)

    return {
        "tokens": tokens,
        "seeds": [(atoms[sid][1], sid) for sid in seed_ids],
        "sentences": [(atoms[sid][1], score, [atoms[w][1] for w in sent_hits[sid]]) for sid, score in top_sents],
        "articles": top_articles,
    }


if __name__ == "__main__":
    path = "/home/dinio/zets/data/baseline/wiki_all_domains_v1.atoms"
    print("📖 Loading ...")
    atoms, out_adj, in_partof = load_all(path)
    word_ix, source_ix, sent_to_source = build_word_index(atoms)
    print(f"  indexed: {len(word_ix):,} words, {len(source_ix):,} articles, {len(sent_to_source):,} sentences")

    questions = [
        "What is the capital of France?",
        "Who wrote Don Quixote?",
        "Is a dog an animal?",
        "What is gravity?",
        "What is the heart?",
        "Tell me about the Earth.",
        "Who was Albert Einstein?",
        "What is photosynthesis?",
        "Explain quantum mechanics.",
        "מה הבירה של צרפת?",
        "מי כתב את ספר יצירה?",
    ]

    for q in questions:
        t0 = time.time()
        r = retrieve(q, atoms, word_ix, source_ix, in_partof, sent_to_source)
        dt = (time.time() - t0) * 1000

        print(f"\n{'═'*72}")
        print(f"❓ {q}")
        print(f"   tokens: {r['tokens']}")
        print(f"   seeds:  {[s[0] for s in r['seeds']]}")
        if r['articles']:
            print(f"   📚 Top articles:")
            for art, score in r['articles']:
                print(f"      • {art:30}  score={score}")
        if r['sentences']:
            print(f"   📝 Top sentence-pointers (matched seeds):")
            for slabel, score, matched_words in r['sentences']:
                print(f"      • {slabel:45}  hits={score}  words={matched_words}")
        else:
            print(f"   (no sentence hits)")
        print(f"   ⏱  {dt:.0f}ms")
