#!/usr/bin/env python3
"""
v4_path_graph.py — גרף text-as-graph מלא:
  - Word atoms (unique per token)
  - Phrase atoms (n-grams שחוזרים)
  - Sentence atoms (עם tekst)
  - Article atoms

  Edges:
    fills_slot   — word/phrase ב-position N בתוך sentence (מסלול מדויק)
    next         — word/phrase → הבא בתוך sentence (פשטני)
    part_of      — word/phrase part_of sentence (member-of)
    co_occurs    — word ↔ word באותו sentence (לא מכוון)
    has_part     — phrase → words שמרכיבים אותו
    contained_in — sentence → article

  + IDF-weighted retrieval
  + Phrase extraction (n-grams 2-4 שחוזרים 3+ פעמים)
  + Path-based sentence reconstruction
  + Re-learning (deterministic re-ingest)
"""
import gzip, json, re, hashlib, time
from collections import Counter, defaultdict
from math import log

# ═════════════════════════════════════════════════════════════════════════
#                              ATOM TYPES
# ═════════════════════════════════════════════════════════════════════════
W = "word"       # word atom:    kind=W, key=token
P = "phrase"     # phrase atom:  kind=P, key=("w1","w2",...)
S = "sentence"   # sentence:     kind=S, key=(article, idx), has text
A = "article"    # article:      kind=A, key=title

# ═════════════════════════════════════════════════════════════════════════
#                               INGESTION
# ═════════════════════════════════════════════════════════════════════════
def load_articles(titles, path="data/wikipedia_dumps/en_parsed.jsonl.gz", max_scan=200000):
    got = {}
    with gzip.open(path, "rt", encoding="utf-8") as f:
        for i, line in enumerate(f):
            if len(got) == len(titles): break
            if i > max_scan: break
            try:
                d = json.loads(line)
                if d.get("title") in titles:
                    got[d["title"]] = d.get("text", "")
            except: pass
    return got


def tokenize(text):
    """מחזיר רשימת טוקנים נקיים (lowercase, ללא פיסוק)."""
    return re.findall(r"[a-zA-Z']+", text.lower())


def split_sentences(text):
    """פשוט — על פי . ! ?"""
    sents = re.split(r'(?<=[.!?])\s+', text.strip())
    return [s for s in sents if len(s) > 20]  # סנן רעש


# ═════════════════════════════════════════════════════════════════════════
#                             THE GRAPH
# ═════════════════════════════════════════════════════════════════════════
class Graph:
    def __init__(self):
        # atoms: dict[atom_key → atom_id]
        self.atoms = {}
        self.atom_data = []    # atom_id → (kind, key, payload)
        
        # edges: list[(from_id, rel, to_id, weight, pos)]
        self.edges = []
        
        # indexes (מחושבים אחרי build)
        self.out_by_rel = None  # atom_id → rel → [(to_id, weight, pos)]
        self.in_by_rel = None   # atom_id → rel → [(from_id, weight, pos)]
        self.word_ix = None
        self.phrase_ix = None
        self.article_ix = None
    
    def atom(self, kind, key, payload=None):
        """idempotent — אם קיים, מחזיר את הקיים."""
        k = (kind, key)
        if k in self.atoms:
            return self.atoms[k]
        aid = len(self.atom_data)
        self.atoms[k] = aid
        self.atom_data.append((kind, key, payload))
        return aid
    
    def edge(self, from_id, rel, to_id, weight=1, pos=0):
        self.edges.append((from_id, rel, to_id, weight, pos))
    
    def build_indexes(self):
        """בונה out/in adjacency ב-partition לפי relation."""
        out = defaultdict(lambda: defaultdict(list))
        inn = defaultdict(lambda: defaultdict(list))
        for (f, r, t, w, p) in self.edges:
            out[f][r].append((t, w, p))
            inn[t][r].append((f, w, p))
        self.out_by_rel = out
        self.in_by_rel = inn
        
        self.word_ix = {key: aid for (kind, key), aid in self.atoms.items() if kind == W}
        self.phrase_ix = {key: aid for (kind, key), aid in self.atoms.items() if kind == P}
        self.article_ix = {key: aid for (kind, key), aid in self.atoms.items() if kind == A}
    
    def stats(self):
        kinds = Counter(ad[0] for ad in self.atom_data)
        rels = Counter(e[1] for e in self.edges)
        return {
            "atoms_total": len(self.atom_data),
            "atoms_by_kind": dict(kinds),
            "edges_total": len(self.edges),
            "edges_by_rel": dict(rels),
        }
    
    def fingerprint(self):
        """hash של מבנה הגרף — determinism check."""
        h = hashlib.sha256()
        # sort atoms by (kind, key) — deterministic
        for (kind, key, payload) in sorted(self.atom_data, key=lambda x: (x[0], str(x[1]))):
            h.update(f"{kind}:{key}|".encode())
        # edges sorted
        for e in sorted(self.edges):
            h.update(f"{e}|".encode())
        return h.hexdigest()[:16]


# ═════════════════════════════════════════════════════════════════════════
#                              BUILD GRAPH
# ═════════════════════════════════════════════════════════════════════════
def extract_phrases(all_sentences_tokens, min_count=3, ngram_sizes=(2, 3, 4)):
    """מוצא n-grams שחוזרים לפחות min_count פעמים על-פני כל הקורפוס."""
    counts = Counter()
    for tokens in all_sentences_tokens:
        for n in ngram_sizes:
            for i in range(len(tokens) - n + 1):
                ng = tuple(tokens[i:i+n])
                counts[ng] += 1
    # סינון: רק n-grams שחוזרים מספיק
    phrases = {ng: c for ng, c in counts.items() if c >= min_count}
    # סינון נוסף: המנע מ-n-grams שמורכבים *רק* מ-stopwords
    STOPS = {"the","a","an","is","are","was","were","of","in","on","at","to","from",
             "by","with","for","and","or","but","not","as","it","its","this","that"}
    phrases = {ng: c for ng, c in phrases.items() 
               if not all(t in STOPS for t in ng)}
    return phrases


def build(articles):
    """בונה את הגרף המלא מ-dict של articles."""
    t0 = time.time()
    g = Graph()
    
    # שלב 1: parse sentences + tokens
    all_sentences_tokens = []
    sent_meta = []  # (article, sent_idx, tokens, text)
    for title, text in articles.items():
        sents = split_sentences(text)
        for sidx, stext in enumerate(sents):
            tokens = tokenize(stext)
            if len(tokens) < 3: continue
            all_sentences_tokens.append(tokens)
            sent_meta.append((title, sidx, tokens, stext))
    
    print(f"  sentences parsed: {len(sent_meta):,}")
    
    # שלב 2: extract phrases
    phrases = extract_phrases(all_sentences_tokens, min_count=3)
    print(f"  phrases extracted (n-gram, ≥3 occurrences): {len(phrases):,}")
    print(f"    top 5: {[' '.join(ng) for ng, _ in sorted(phrases.items(), key=lambda x:-x[1])[:5]]}")
    
    # שלב 3: יצירת atoms
    for title in articles:
        g.atom(A, title)
    for tokens in all_sentences_tokens:
        for tok in tokens:
            g.atom(W, tok)
    for ng in phrases:
        g.atom(P, ng, payload={"count": phrases[ng]})
    
    # שלב 4: sentence atoms + edges
    for (title, sidx, tokens, stext) in sent_meta:
        sid = g.atom(S, (title, sidx), payload={"text": stext, "tokens": tokens})
        aid = g.atoms[(A, title)]
        g.edge(sid, "contained_in", aid, weight=1, pos=sidx)
        g.edge(aid, "has_sentence", sid, weight=1, pos=sidx)
        
        # ─── fills_slot + next per word ───
        # מזהה phrases בתוך המשפט (matching greedy longest-first)
        matched_spans = []  # list of (start, end, phrase_key)
        i = 0
        while i < len(tokens):
            found = None
            for n in (4, 3, 2):  # greedy longest
                if i + n <= len(tokens):
                    ng = tuple(tokens[i:i+n])
                    if ng in phrases:
                        found = (i, i+n, ng)
                        break
            if found:
                matched_spans.append(found)
                i = found[1]
            else:
                i += 1
        
        # ה-units הם phrases + words שלא נבלעו
        units = []  # (start, end, kind, key)
        covered = set()
        for (s, e, ng) in matched_spans:
            units.append((s, e, P, ng))
            for j in range(s, e): covered.add(j)
        for j, tok in enumerate(tokens):
            if j not in covered:
                units.append((j, j+1, W, tok))
        units.sort(key=lambda x: x[0])  # by position
        
        # edges: fills_slot + next
        prev_uid = None
        for pos, (s, e, kind, key) in enumerate(units):
            uid = g.atoms[(kind, key)]
            g.edge(uid, "fills_slot", sid, weight=1, pos=pos)
            g.edge(sid, "part_of_backref", uid, weight=1, pos=pos)  # reverse
            if prev_uid is not None:
                g.edge(prev_uid, "next", uid, weight=1, pos=pos-1)
            prev_uid = uid
            # אם זה phrase — הוסף has_part ל-words שלו
            if kind == P:
                for wtok in key:
                    wid = g.atoms[(W, wtok)]
                    g.edge(uid, "has_part", wid)
                    g.edge(wid, "part_of", uid)
        
        # ─── co_occurs between words (skip phrases) ───
        word_ids_in_sent = [g.atoms[(W, t)] for t in tokens]
        # edges בין כל זוג מילים באותו משפט (שונות)
        for a in range(len(word_ids_in_sent)):
            for b in range(a+1, min(a+6, len(word_ids_in_sent))):  # רק שכנות 5 מילים
                if word_ids_in_sent[a] != word_ids_in_sent[b]:
                    g.edge(word_ids_in_sent[a], "co_occurs", word_ids_in_sent[b], weight=1)
    
    g.build_indexes()
    dt = time.time() - t0
    print(f"  graph built in {dt:.1f}s")
    return g


# ═════════════════════════════════════════════════════════════════════════
#                              RETRIEVAL
# ═════════════════════════════════════════════════════════════════════════
STOPS = {"the","a","an","is","are","was","were","what","who","where","when","how",
         "why","of","in","on","at","to","from","by","with","for","and","or","but",
         "not","do","does","did","have","has","had","can","could","would","should",
         "this","that","these","those","it","its","my","your","tell","me","about","us"}

def find_seeds(q, g):
    """טוקניזציה + חיפוש seeds (phrases קודמים!)."""
    tokens = [t for t in tokenize(q) if t not in STOPS and len(t) > 1]
    seeds = []  # (kind, key, atom_id, matched_tokens)
    
    # greedy phrase matching
    i = 0
    while i < len(tokens):
        found = None
        for n in (4, 3, 2):
            if i + n <= len(tokens):
                ng = tuple(tokens[i:i+n])
                if ng in g.phrase_ix:
                    found = ng
                    seeds.append((P, ng, g.phrase_ix[ng], list(ng)))
                    i += n
                    break
        if not found:
            tok = tokens[i]
            if tok in g.word_ix:
                seeds.append((W, tok, g.word_ix[tok], [tok]))
            i += 1
    return tokens, seeds


def compute_idf(g):
    """IDF לכל word/phrase לפי מספר ה-articles שהוא מופיע בהם."""
    idf = {}
    n_articles = len(g.article_ix)
    # לכל word/phrase — find articles via part_of/fills_slot → sentence → article
    for (kind, key), aid in g.atoms.items():
        if kind not in (W, P): continue
        # fills_slot → sentence ids
        sent_ids = set()
        for (sid, w, pos) in g.out_by_rel[aid].get("fills_slot", []):
            sent_ids.add(sid)
        # sentence → article
        article_ids = set()
        for sid in sent_ids:
            for (art_id, w, pos) in g.out_by_rel[sid].get("contained_in", []):
                article_ids.add(art_id)
        df = max(len(article_ids), 1)
        idf[aid] = log(n_articles / df)
    return idf


def answer(q, g, idf, top_k_sents=5, top_k_arts=5):
    """Path-aware + phrase-aware + IDF-weighted retrieval."""
    tokens, seeds = find_seeds(q, g)
    seed_ids = [aid for (_, _, aid, _) in seeds]
    
    # לכל sentence — ספור seeds שמופיעים בה (משוקלל IDF)
    sent_scores = Counter()
    sent_hits = defaultdict(list)
    for aid in seed_ids:
        w_idf = idf.get(aid, 0.5)
        # seed → fills_slot → sentence
        for (sid, w, pos) in g.out_by_rel[aid].get("fills_slot", []):
            sent_scores[sid] += w_idf
            sent_hits[sid].append(aid)
    
    # proximity boost: משפט שמכיל 2+ seeds קרובים אחד לשני
    for sid in list(sent_scores.keys()):
        positions = []
        for aid in sent_hits[sid]:
            for (s, w, p) in g.out_by_rel[aid].get("fills_slot", []):
                if s == sid:
                    positions.append(p)
                    break
        positions.sort()
        if len(positions) >= 2:
            # ככל שהמרחק הממוצע קטן, הבונוס גדול
            gaps = [positions[i+1] - positions[i] for i in range(len(positions)-1)]
            avg_gap = sum(gaps) / len(gaps)
            boost = 1.0 + 1.0 / (1.0 + avg_gap)
            sent_scores[sid] *= boost
    
    # aggregate → articles
    art_scores = Counter()
    art_best_sent = {}
    for sid, score in sent_scores.items():
        for (art_id, w, pos) in g.out_by_rel[sid].get("contained_in", []):
            art_scores[art_id] += score
            if art_id not in art_best_sent or sent_scores[art_best_sent[art_id]] < score:
                art_best_sent[art_id] = sid
    
    top_sents = sent_scores.most_common(top_k_sents)
    top_arts = art_scores.most_common(top_k_arts)
    
    return {
        "tokens": tokens,
        "seeds": [(kind, key) for (kind, key, _, _) in seeds],
        "top_articles": [(g.atom_data[aid][1], float(score)) for (aid, score) in top_arts],
        "top_sentences": [
            (g.atom_data[sid][2]["text"], float(score), g.atom_data[sid][1])
            for (sid, score) in top_sents
        ],
    }


# ═════════════════════════════════════════════════════════════════════════
#                           PATH RECONSTRUCTION
# ═════════════════════════════════════════════════════════════════════════
def reconstruct_sentence(sid, g):
    """שחזור טקסט מהגרף דרך 'next' walk."""
    # מצא first unit (אין inbound next → הוא הראשון)
    units_in_sent = set()
    for (uid, rel, sid2, w, p) in g.edges:
        if rel == "fills_slot" and sid2 == sid:
            units_in_sent.add((uid, p))
    if not units_in_sent:
        return None
    
    # מיין לפי pos
    ordered = sorted(units_in_sent, key=lambda x: x[1])
    
    # שחזור
    parts = []
    for (uid, p) in ordered:
        kind, key, payload = g.atom_data[uid]
        if kind == W:
            parts.append(key)
        elif kind == P:
            parts.append(" ".join(key))
    return " ".join(parts)


# ═════════════════════════════════════════════════════════════════════════
#                                MAIN
# ═════════════════════════════════════════════════════════════════════════
if __name__ == "__main__":
    TITLES = ["Gravity", "Heart", "Earth", "Moon", "Venus", "Sun",
              "Photosynthesis", "Brain", "Insulin", "Relativity",
              "Black hole", "Quantum mechanics", "Big Bang",
              "Newton's laws of motion", "Circulatory system",
              "Oxygen", "Redox", "Parasitism", "Skin"]
    
    print("═" * 72)
    print("  v4 PATH GRAPH — Build & Test")
    print("═" * 72)
    
    print("\n[1] Loading articles ...")
    articles = load_articles(TITLES)
    print(f"  got {len(articles)}/{len(TITLES)} articles")
    
    print("\n[2] Building graph (ingestion #1) ...")
    g1 = build(articles)
    stats1 = g1.stats()
    print(f"\n  stats:")
    for k, v in stats1.items():
        print(f"    {k}: {v}")
    fp1 = g1.fingerprint()
    print(f"  fingerprint: {fp1}")
    
    # ─── Retrieval ───
    print("\n[3] Computing IDF ...")
    idf = compute_idf(g1)
    print(f"  IDF computed for {len(idf):,} atoms")
    
    print("\n[4] Testing retrieval on real questions:")
    QUESTIONS = [
        "What is gravity?",
        "What is gravity on Earth?",
        "Who was Albert Einstein?",
        "What is the heart?",
        "What is photosynthesis?",
        "What is quantum mechanics?",
        "Is the Moon larger than Earth?",
        "What are Newton's laws?",
    ]
    for q in QUESTIONS:
        r = answer(q, g1, idf)
        print(f"\n   ❓ {q}")
        print(f"      seeds: {r['seeds'][:5]}")
        print(f"      📚 top articles:")
        for name, score in r['top_articles'][:3]:
            print(f"         {name:35} {score:.2f}")
        if r['top_sentences']:
            print(f"      📝 best sentence:")
            txt, score, key = r['top_sentences'][0]
            print(f"         \"{txt[:200]}\"")
    
    # ─── Path reconstruction ───
    print("\n[5] Path-based sentence reconstruction:")
    # מצא sentence של Gravity:0
    gravity_art_id = g1.article_ix.get("Gravity")
    if gravity_art_id:
        first_sent_id = None
        for (sid, w, pos) in g1.out_by_rel[gravity_art_id].get("has_sentence", []):
            if pos == 0:
                first_sent_id = sid
                break
        if first_sent_id:
            original = g1.atom_data[first_sent_id][2]["text"]
            reconstructed = reconstruct_sentence(first_sent_id, g1)
            print(f"   original:      \"{original[:150]}\"")
            print(f"   reconstructed: \"{reconstructed[:150] if reconstructed else '(failed)'}\"")
            print(f"   match: {original.lower() == reconstructed.lower() if reconstructed else 'N/A'}")
    
    # ─── Re-learning determinism ───
    print("\n[6] RE-LEARNING TEST — build same graph again, compare fingerprints:")
    g2 = build(articles)
    fp2 = g2.fingerprint()
    print(f"   fingerprint 1: {fp1}")
    print(f"   fingerprint 2: {fp2}")
    print(f"   ✓ DETERMINISTIC" if fp1 == fp2 else f"   ✗ NOT deterministic")
    
    # ─── Incremental learning ───
    print("\n[7] INCREMENTAL LEARNING TEST — add 1 article, check only it changes:")
    # טען מאמר נוסף
    more = load_articles(["Adverb"])
    if more:
        articles2 = dict(articles); articles2.update(more)
        g3 = build(articles2)
        stats3 = g3.stats()
        print(f"   before: atoms={stats1['atoms_total']:,} edges={stats1['edges_total']:,}")
        print(f"   after:  atoms={stats3['atoms_total']:,} edges={stats3['edges_total']:,}")
        print(f"   delta:  +{stats3['atoms_total']-stats1['atoms_total']:,} atoms, +{stats3['edges_total']-stats1['edges_total']:,} edges")
        # בדוק שהfingerprint של gravity_sentence לא השתנה
        # (זה test מקורב — ב-full implementation נוודא שכל sentence אטום שהיה קיים נשמר)
