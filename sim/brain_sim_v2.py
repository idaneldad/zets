"""
brain_sim_v2 — הרחבה של הסימולציה עם ניהול מידע מלא.

מוסיף:
1. KB עם 5 stores נפרדים (lexicon, encyclopedia, procedures, episodes, skills)
2. Lifecycle שלם: READ / WRITE / UPDATE / FORGET / CONSOLIDATE
3. Hebbian learning — חיזוק קשרים בין regions
4. Confidence + provenance (מאיפה כל פיסת מידע)
5. הפרדה בין short-term (working memory) ל-long-term (KB)
"""

from dataclasses import dataclass, field
from typing import Dict, List, Set, Optional, Tuple, Any
from collections import defaultdict, deque
import time, json, hashlib


# ═══════════════════════════════════════════════════════════════════════════
#  KB — 5 STORES נפרדים, כל אחד עם API משלו וסמנטיקה שונה
# ═══════════════════════════════════════════════════════════════════════════

@dataclass
class LexEntry:
    """ערך מילוני — immutable (מילון יבש)."""
    word: str                          # 'water'
    lemma: str                         # 'water' (base form)
    lang: str                          # 'en'
    translations: Dict[str, str]       # {'he': 'מים', 'ar': 'ماء'}
    synonyms: List[str]                # ['liquid', 'aqua']
    pos: str                           # 'noun'
    provenance: str                    # 'wiktionary_2026-04-23'


@dataclass
class Article:
    """מאמר — מבוסס על הZETS DAG המדויק שכבר בנינו."""
    title: str
    lang: str
    sections: List[Dict]               # [{name, sentences:[...]}]
    concepts: Set[str]                 # which words/phrases this article is about
    provenance: str
    version: int = 1                   # updates tracked


@dataclass
class Procedure:
    """תהליך — how-to (רצף פעולות)."""
    name: str
    steps: List[str]
    prerequisites: List[str]           # מושגים שצריך להכיר
    success_criteria: str
    provenance: str


@dataclass
class Episode:
    """זיכרון אפיזודי — מה קרה, מתי, תוצאה."""
    id: str
    timestamp: float
    input: str
    queries_made: List[str]            # אילו KB lookups נעשו
    regions_activated: List[str]       # מי הופעל (לניתוח למידה)
    response: str
    outcome: str                       # 'success', 'failure', 'unknown'


@dataclass
class Skill:
    """מיומנות — pattern נלמד (דרך בהצלחה אבוצעה קודם)."""
    name: str
    trigger_pattern: str               # מתי להפעיל
    action_sequence: List[str]         # רצף regions/procedures
    confidence: float                  # כמה פעמים זה עבד / ניסיונות
    success_count: int = 0
    attempt_count: int = 0


class KnowledgeBase:
    """
    5 stores, separated by semantics.
    כל store יש לו lifecycle משלו.
    """
    def __init__(self):
        # --- 5 stores ---
        self.lexicon: Dict[str, LexEntry] = {}
        self.encyclopedia: Dict[str, Article] = {}
        self.procedures: Dict[str, Procedure] = {}
        self.episodes: List[Episode] = []
        self.skills: Dict[str, Skill] = {}
        
        # --- indexes (לא "ידע", אלא נתיבים מהירים) ---
        self.word_to_articles: Dict[str, Set[str]] = defaultdict(set)
        self.concept_to_procedures: Dict[str, Set[str]] = defaultdict(set)
        self.lang_index: Dict[str, Set[str]] = defaultdict(set)
        
        # --- metadata (ניהול טוב דורש audit trail) ---
        self.last_modified: Dict[str, float] = {}
        self.access_count: Dict[str, int] = defaultdict(int)

    # ══════════════════════════════════════════════════════════════
    #   LIFECYCLE 1: WRITE — איך מידע נכנס
    # ══════════════════════════════════════════════════════════════
    def add_lexicon(self, entry: LexEntry):
        """הוספת ערך מילוני. אם קיים — bumping version, keeping history."""
        key = f"{entry.word}:{entry.lang}"
        if key in self.lexicon:
            # UPDATE — מילון יבש אמור להיות stable, אבל translation יכולה להתרחב
            existing = self.lexicon[key]
            existing.translations.update(entry.translations)
            existing.synonyms = list(set(existing.synonyms + entry.synonyms))
        else:
            self.lexicon[key] = entry
        self.last_modified[key] = time.time()
        self.lang_index[entry.lang].add(key)

    def add_article(self, article: Article):
        """Ingest article. ה-DAG שלנו היום = מי שבונה את זה."""
        key = f"{article.lang}:{article.title}"
        if key in self.encyclopedia:
            article.version = self.encyclopedia[key].version + 1
        self.encyclopedia[key] = article
        self.last_modified[key] = time.time()
        for concept in article.concepts:
            self.word_to_articles[concept].add(key)

    def add_procedure(self, proc: Procedure):
        self.procedures[proc.name] = proc
        for concept in proc.prerequisites:
            self.concept_to_procedures[concept].add(proc.name)
        self.last_modified[proc.name] = time.time()

    def log_episode(self, ep: Episode):
        """Append-only. אפיזודים לא נמחקים — נצברים."""
        self.episodes.append(ep)

    def learn_skill(self, skill: Skill):
        """Hebbian — חיזוק אם הצליח, החלשה אם לא."""
        if skill.name in self.skills:
            existing = self.skills[skill.name]
            existing.attempt_count += skill.attempt_count
            existing.success_count += skill.success_count
            existing.confidence = existing.success_count / max(1, existing.attempt_count)
        else:
            self.skills[skill.name] = skill

    # ══════════════════════════════════════════════════════════════
    #   LIFECYCLE 2: READ — איך המוח מבקש
    # ══════════════════════════════════════════════════════════════
    def lookup(self, word: str, lang: str = "en") -> Optional[LexEntry]:
        key = f"{word}:{lang}"
        self.access_count[key] += 1
        return self.lexicon.get(key)

    def get_article(self, title: str, lang: str) -> Optional[Article]:
        key = f"{lang}:{title}"
        self.access_count[key] += 1
        return self.encyclopedia.get(key)

    def find_articles_about(self, concept: str) -> List[Article]:
        """חוזר: כל articles שעוסקים ב-concept."""
        keys = self.word_to_articles.get(concept, set())
        return [self.encyclopedia[k] for k in keys if k in self.encyclopedia]

    def find_procedures_for(self, concept: str) -> List[Procedure]:
        keys = self.concept_to_procedures.get(concept, set())
        return [self.procedures[k] for k in keys]

    def recent_episodes(self, n: int = 10) -> List[Episode]:
        return self.episodes[-n:]

    def find_skill_for(self, input: str) -> Optional[Skill]:
        """חוזר skill שהטריגר שלו תואם."""
        best = None
        best_conf = 0
        for s in self.skills.values():
            if s.trigger_pattern.lower() in input.lower() and s.confidence > best_conf:
                best = s
                best_conf = s.confidence
        return best

    # ══════════════════════════════════════════════════════════════
    #   LIFECYCLE 3: UPDATE — תיקונים, contradictions
    # ══════════════════════════════════════════════════════════════
    def update_article(self, title: str, lang: str, new_content: List[Dict], reason: str):
        """Version-tracked update. השקוף."""
        key = f"{lang}:{title}"
        if key in self.encyclopedia:
            article = self.encyclopedia[key]
            article.version += 1
            article.sections = new_content
            self.last_modified[key] = time.time()
            # Log the update reason as episode
            self.log_episode(Episode(
                id=hashlib.md5(f"update:{key}:{time.time()}".encode()).hexdigest()[:8],
                timestamp=time.time(),
                input=f"UPDATE {key}",
                queries_made=[],
                regions_activated=["pfc"],  # deliberative update
                response=reason,
                outcome="update_applied",
            ))

    # ══════════════════════════════════════════════════════════════
    #   LIFECYCLE 4: FORGET — ניהול שכחה (חשוב!)
    # ══════════════════════════════════════════════════════════════
    def consolidate(self):
        """
        Sleep-like consolidation. רץ מעת לעת.
        - מעביר episodes ישנים משמעותיים ל-skills (pattern extraction)
        - מנקה low-access entries (שכחה טבעית)
        - מעדכן confidence של skills
        """
        # 1. Skill extraction: episodes שחוזרים → skill
        episode_patterns = defaultdict(list)
        for ep in self.episodes[-100:]:  # sliding window
            if ep.outcome == "success" and ep.queries_made:
                pattern = tuple(ep.queries_made[:3])
                episode_patterns[pattern].append(ep)
        
        new_skills = 0
        for pattern, eps in episode_patterns.items():
            if len(eps) >= 3:  # 3+ successes = skill!
                name = "skill_" + "_".join(pattern)[:40]
                skill = Skill(
                    name=name,
                    trigger_pattern=eps[0].input[:30],
                    action_sequence=list(pattern),
                    confidence=1.0,
                    success_count=len(eps),
                    attempt_count=len(eps),
                )
                self.learn_skill(skill)
                new_skills += 1
        
        # 2. Forget old low-access lexicon entries (שכחה ביולוגית)
        cutoff = time.time() - 30 * 86400  # 30 days
        forgotten = 0
        for key in list(self.lexicon.keys()):
            if (self.access_count[key] == 0 and 
                self.last_modified.get(key, 0) < cutoff):
                # לא נגיש ב-30 יום + לא שומש → forget
                del self.lexicon[key]
                forgotten += 1
        
        # 3. Compress old episodes (keep summary, drop details)
        if len(self.episodes) > 1000:
            # keep last 100 + summary of older
            old = self.episodes[:-100]
            summary = Episode(
                id="summary_" + str(time.time()),
                timestamp=time.time(),
                input=f"SUMMARY of {len(old)} old episodes",
                queries_made=[],
                regions_activated=[],
                response=f"{len(old)} episodes compacted",
                outcome="consolidated",
            )
            self.episodes = [summary] + self.episodes[-100:]
        
        return {
            "new_skills": new_skills,
            "forgotten_lexicon": forgotten,
            "episodes_after": len(self.episodes),
        }

    # ══════════════════════════════════════════════════════════════
    #   PROVENANCE & AUDIT — חובה לבריאות המערכת
    # ══════════════════════════════════════════════════════════════
    def stats(self) -> Dict:
        return {
            "lexicon_entries": len(self.lexicon),
            "articles": len(self.encyclopedia),
            "procedures": len(self.procedures),
            "episodes": len(self.episodes),
            "skills": len(self.skills),
            "languages": list(self.lang_index.keys()),
            "top_accessed": sorted(self.access_count.items(), key=lambda x: -x[1])[:5],
        }


# ═══════════════════════════════════════════════════════════════════════════
#  BRAIN — עם spreading activation + Hebbian learning
# ═══════════════════════════════════════════════════════════════════════════

class Brain:
    def __init__(self, kb: KnowledgeBase):
        self.kb = kb
        self.regions: Dict[str, float] = {}
        self.connections: Dict[Tuple[str, str], float] = {}  # edge → weight
        self.working_memory = deque(maxlen=7)
        self._wire()

    def _wire(self):
        """Pre-wired at birth. לא משתנה."""
        names = ["sensory", "lexical", "wernicke", "broca",
                 "hippocampus", "semantic_gw", "pfc", "attention",
                 "default_mode", "motor", "thalamus"]
        for n in names: self.regions[n] = 0.0

        wires = [
            ("sensory", "lexical"), ("lexical", "wernicke"),
            ("wernicke", "semantic_gw"), ("wernicke", "pfc"),
            ("semantic_gw", "hippocampus"), ("hippocampus", "pfc"),
            ("pfc", "attention"), ("pfc", "broca"),
            ("attention", "wernicke"),    # recurrent!
            ("default_mode", "hippocampus"),
            ("default_mode", "semantic_gw"),
            ("broca", "motor"), ("motor", "thalamus"),
            ("thalamus", "sensory"),       # recurrent!
            ("pfc", "default_mode"),
        ]
        for a, b in wires:
            self.connections[(a, b)] = 0.5   # baseline weight

    def activate(self, region: str, level: float):
        self.regions[region] = min(1.0, self.regions[region] + level)
        # spreading activation דרך הקשרים
        for (a, b), weight in self.connections.items():
            if a == region:
                self.regions[b] = min(1.0, self.regions[b] + level * weight * 0.5)

    def process(self, input_text: str) -> Dict:
        # Reset activations
        for n in self.regions: self.regions[n] = 0.0
        queries_made = []
        
        # 1. Sensory → Lexical
        self.activate("sensory", 1.0)
        words = input_text.lower().replace("?", "").split()
        
        # 2. Check for skill match (experience speeds recognition)
        skill = self.kb.find_skill_for(input_text)
        if skill:
            # Skill pathway — hippocampus → PFC directly
            self.activate("hippocampus", 0.8)
            self.activate("pfc", 0.9)
            queries_made.append(f"skill:{skill.name}")
            # strengthen skill-related connections (Hebbian)
            self._hebbian_update("hippocampus", "pfc", reward=0.1)
        
        # 3. Lexical lookup for each word
        self.activate("lexical", 0.7)
        self.activate("wernicke", 0.5)
        findings = {}
        for w in words:
            if len(w) < 2: continue
            entry = self.kb.lookup(w, "en")
            if entry:
                findings[w] = entry
                queries_made.append(f"lex:{w}")
                self.activate("semantic_gw", 0.3)
        
        # 4. Context lookup — find articles for working-memory concepts
        for w, entry in findings.items():
            articles = self.kb.find_articles_about(entry.lemma)
            if articles:
                self.working_memory.append((w, articles[0].title))
                self.activate("hippocampus", 0.2)
                queries_made.append(f"art:{articles[0].title}")
        
        # 5. PFC integration
        self.activate("pfc", 0.7)
        
        # 6. Response generation (Broca)
        self.activate("broca", 0.6)
        response = self._compose_response(findings)
        self.activate("motor", 0.5)
        
        # 7. Episode log
        ep = Episode(
            id=hashlib.md5(f"{input_text}:{time.time()}".encode()).hexdigest()[:8],
            timestamp=time.time(),
            input=input_text,
            queries_made=queries_made,
            regions_activated=[n for n, a in self.regions.items() if a > 0.3],
            response=response,
            outcome="success" if findings else "no_knowledge",
        )
        self.kb.log_episode(ep)
        
        return {
            "response": response,
            "queries_made": queries_made,
            "regions_active": {n: a for n, a in self.regions.items() if a > 0.1},
            "working_memory": list(self.working_memory),
        }

    def _compose_response(self, findings: Dict) -> str:
        if not findings:
            return "I don't have this knowledge."
        parts = []
        for w, entry in findings.items():
            he = entry.translations.get("he", "?")
            parts.append(f"{w} ({he})")
        return ", ".join(parts)

    def _hebbian_update(self, a: str, b: str, reward: float):
        """Cells that fire together, wire together."""
        key = (a, b)
        if key in self.connections:
            self.connections[key] = min(1.0, self.connections[key] + reward)


# ═══════════════════════════════════════════════════════════════════════════
#  DEMO — הדגמת ניהול המידע
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print("═" * 78)
    print("  BRAIN_V2 — Full Information Lifecycle Demo")
    print("═" * 78)
    kb = KnowledgeBase()
    brain = Brain(kb)

    # ─── Scene 1: teach words ───
    print("\n[SCENE 1] Teaching lexicon (WRITE)")
    kb.add_lexicon(LexEntry("water","water","en",{"he":"מים","ar":"ماء"},
                            ["liquid","aqua","fluid"],"noun","wiktionary"))
    kb.add_lexicon(LexEntry("fire","fire","en",{"he":"אש"},
                            ["flame","blaze"],"noun","wiktionary"))
    kb.add_lexicon(LexEntry("tea","tea","en",{"he":"תה"},
                            ["brew"],"noun","wiktionary"))
    print(f"  lexicon now has {len(kb.lexicon)} entries")

    # ─── Scene 2: teach article ───
    print("\n[SCENE 2] Teaching article (WRITE)")
    kb.add_article(Article(
        title="Water",
        lang="en",
        sections=[
            {"name":"Introduction","sentences":["Water is a transparent liquid..."]},
            {"name":"Chemistry","sentences":["Formula H2O, bond angle 104.5°"]},
        ],
        concepts={"water","h2o","liquid","chemistry"},
        provenance="wikipedia_2026-04-23",
    ))
    print(f"  encyclopedia has {len(kb.encyclopedia)} articles")

    # ─── Scene 3: teach procedure ───
    print("\n[SCENE 3] Teaching procedure (WRITE)")
    kb.add_procedure(Procedure(
        name="make_tea",
        steps=["Boil water","Add tea leaves","Steep 3 minutes","Serve"],
        prerequisites=["water","tea","heat_source"],
        success_criteria="liquid is hot and flavored",
        provenance="manual",
    ))

    # ─── Scene 4: READ — brain queries KB ───
    print("\n[SCENE 4] Brain queries (READ)")
    r = brain.process("what is water?")
    print(f"  response: {r['response']}")
    print(f"  queries:  {r['queries_made']}")
    print(f"  regions active: {list(r['regions_active'].keys())}")
    print(f"  working memory: {r['working_memory']}")

    # ─── Scene 5: repeat same query → skill emerges ───
    print("\n[SCENE 5] Repeat same query 3 times (skill formation)")
    for i in range(3):
        brain.process("what is water?")
    print(f"  episodes now: {len(kb.episodes)}")

    # ─── Scene 6: CONSOLIDATE (sleep) ───
    print("\n[SCENE 6] Sleep / consolidation")
    result = kb.consolidate()
    print(f"  new skills: {result['new_skills']}")
    print(f"  forgotten lexicon: {result['forgotten_lexicon']}")
    print(f"  skills in KB: {list(kb.skills.keys())}")

    # ─── Scene 7: query again — should hit skill path ───
    print("\n[SCENE 7] Query again — should use skill (faster path)")
    r = brain.process("what is water?")
    print(f"  queries (includes skill): {r['queries_made']}")
    skill_key = ("hippocampus", "pfc")
    if skill_key in brain.connections:
        print(f"  hebbian weight hippocampus→pfc: {brain.connections[skill_key]:.3f}")

    # ─── Scene 8: UPDATE — existing article corrected ───
    print("\n[SCENE 8] UPDATE article (correction)")
    kb.update_article("Water", "en",
                      [{"name":"Intro","sentences":["Water: H2O, the main solvent of life"]}],
                      reason="scientific clarification")
    print(f"  article version: {kb.encyclopedia['en:Water'].version}")

    # ─── Scene 9: FORGET — mark unused entries ───
    print("\n[SCENE 9] FORGET — simulated old unused entry")
    kb.add_lexicon(LexEntry("obscure_xyz","obscure_xyz","en",{},[],"noun","test"))
    kb.last_modified["obscure_xyz:en"] = time.time() - 40*86400  # 40 days old
    # never accessed → will be forgotten
    before = len(kb.lexicon)
    kb.consolidate()
    after = len(kb.lexicon)
    print(f"  lexicon before: {before}, after consolidate: {after}")

    # ─── Scene 10: stats ───
    print("\n[SCENE 10] KB stats")
    stats = kb.stats()
    for k, v in stats.items():
        print(f"  {k}: {v}")
