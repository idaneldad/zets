"""
brain_sim.py — סימולציה של הארכיטקטורה שעידן מציע.

שתי שכבות נפרדות:
  1. KNOWLEDGE BASE — מילון/אנציקלופדיה יבש, היררכי, מסודר לפי שפות
     (מקביל ל-"ספרייה" במוח — מה שאתה יודע אבל לא חושב עליו כרגע)
  
  2. COGNITIVE BRAIN — מחווט מראש עם אזורים פונקציונליים
     (entry points, מחובר הכל להכל, עם לולאות)

המוח לא מכיל ידע — הוא *ניגש* לידע. כמו אדם שקורא מילון זר.
"""

from dataclasses import dataclass, field
from typing import Dict, List, Set, Optional, Tuple
from collections import defaultdict
import time


# ═══════════════════════════════════════════════════════════════════════
#  1. KNOWLEDGE BASE — השכבה היבשה
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class KnowledgeBase:
    """
    בנק ידע יבש, היררכי.
    זה *לא* מה שמבין — זה מה שניגשים אליו.
    """
    # מילון: lemma → meaning in each language
    # { 'water': { 'en': 'H2O', 'he': 'מים', 'ar': 'ماء' } }
    lexicon: Dict[str, Dict[str, str]] = field(default_factory=dict)
    
    # סינונימים: lemma → [synonyms]
    synonyms: Dict[str, List[str]] = field(default_factory=dict)
    
    # Encyclopedia — article per concept, NOT parsed into graph
    articles: Dict[str, str] = field(default_factory=dict)
    
    # Procedures — recipes, how-tos
    procedures: Dict[str, List[str]] = field(default_factory=dict)
    
    # Sounds, images, etc (paths to binary files)
    media: Dict[str, str] = field(default_factory=dict)
    
    # ─── API יבש — כמו לפתוח מילון ───
    def lookup_word(self, word: str, target_lang: str = "he") -> Optional[str]:
        entry = self.lexicon.get(word.lower())
        return entry.get(target_lang) if entry else None
    
    def get_synonyms(self, word: str) -> List[str]:
        return self.synonyms.get(word.lower(), [])
    
    def get_article(self, title: str) -> Optional[str]:
        return self.articles.get(title)
    
    def get_procedure(self, name: str) -> Optional[List[str]]:
        return self.procedures.get(name)
    
    def search(self, query: str) -> List[str]:
        """חיפוש טקסטואלי פשוט — כמו full-text index."""
        q = query.lower()
        results = []
        for title, text in self.articles.items():
            if q in text.lower() or q in title.lower():
                results.append(title)
        return results


# ═══════════════════════════════════════════════════════════════════════
#  2. COGNITIVE BRAIN — המחווט
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class BrainRegion:
    """אזור במוח — entry point + specialized processing."""
    name: str
    function: str
    activation: float = 0.0
    connections: Set[str] = field(default_factory=set)  # names של אזורים אחרים
    
    def activate(self, level: float = 1.0):
        self.activation = min(1.0, self.activation + level)
    
    def decay(self, factor: float = 0.9):
        self.activation *= factor


class CognitiveBrain:
    """
    המוח — pre-wired, full of loops, מחובר להכל.
    הוא לא מכיל ידע — הוא לומד *איך* לגשת ל-KB.
    """
    def __init__(self, kb: KnowledgeBase):
        self.kb = kb
        self.regions: Dict[str, BrainRegion] = {}
        self.episodic: List[Dict] = []    # זיכרון פעולות (hippocampus)
        self.working_memory: List = []    # max 7±2 items (PFC)
        
        self._wire_brain()
    
    # ─── חיווט ראשוני — מ-לידה ───
    def _wire_brain(self):
        """
        החיווט הקבוע — אזורים ו-connections ביניהם.
        לא תלוי בידע. זה מה ש"נולד איתו".
        """
        regions_spec = [
            # SENSORY — נקודות כניסה
            ("sensory_visual",    "עיבוד input חזותי"),
            ("sensory_auditory",  "עיבוד input שמיעתי/טקסטואלי"),
            ("sensory_lexical",   "זיהוי מילים, morphology"),
            
            # LANGUAGE
            ("wernicke",          "הבנת שפה"),
            ("broca",             "הפקת שפה"),
            
            # MEMORY — שערים ל-KB
            ("hippocampus",       "מצביע לזיכרון אפיזודי וצוין KB lookups"),
            ("semantic_gateway",  "שער ל-knowledge base (מילון, articles)"),
            
            # EXECUTIVE
            ("pfc",               "תכנון, working memory, החלטות"),
            ("attention",         "מיקוד על subset מידע"),
            
            # INTEGRATION
            ("thalamus",          "router — כל מידע עובר דרכו"),
            ("default_mode",      "associations בחופש, analogies"),
            
            # OUTPUT
            ("motor",             "output — response generation"),
        ]
        
        for name, func in regions_spec:
            self.regions[name] = BrainRegion(name, func)
        
        # החיבורים — מי מחובר למי (recurrent, לא DAG!)
        connections = [
            # input → processing
            ("sensory_visual",    ["sensory_lexical", "thalamus"]),
            ("sensory_auditory",  ["sensory_lexical", "thalamus"]),
            ("sensory_lexical",   ["wernicke", "hippocampus"]),
            
            # language loop
            ("wernicke",          ["broca", "semantic_gateway", "pfc"]),
            ("broca",             ["wernicke", "motor"]),      # loop!
            
            # memory access
            ("hippocampus",       ["semantic_gateway", "pfc", "default_mode"]),
            ("semantic_gateway",  ["hippocampus", "wernicke", "default_mode"]),
            
            # executive + integration
            ("pfc",               ["wernicke", "hippocampus", "motor", "attention", "default_mode"]),
            ("attention",         ["pfc", "thalamus", "hippocampus"]),
            ("thalamus",          ["pfc", "sensory_visual", "sensory_auditory", "motor"]),
            ("default_mode",      ["hippocampus", "pfc", "semantic_gateway"]),
            
            # output
            ("motor",             ["pfc", "broca"]),
        ]
        for src, targets in connections:
            for t in targets:
                self.regions[src].connections.add(t)
                # recurrent — גם הפוך
                self.regions[t].connections.add(src)
    
    # ─── פעולה קוגניטיבית ───
    def process(self, input_text: str, verbose: bool = True):
        """
        זה ה-flow האמיתי: input → activations → KB lookups → response.
        Not a DAG. Loops everywhere.
        """
        trace = []
        
        # 1. Sensory input
        self.regions["sensory_auditory"].activate(1.0)
        self.regions["sensory_lexical"].activate(1.0)
        trace.append(f"🎧 Sensory input: '{input_text}'")
        
        # 2. Morphology: break into lemmas
        words = input_text.lower().replace("?", "").replace(".", "").split()
        lemmas = []
        for w in words:
            # query KB for lemma (יבש — כמו לפתוח מילון)
            lemma = self.kb.lookup_word(w, "base") or w
            lemmas.append(lemma)
        trace.append(f"🔤 Lexical → lemmas: {lemmas}")
        self.regions["sensory_lexical"].activate(0.5)
        
        # 3. Wernicke: understand
        self.regions["wernicke"].activate(1.0)
        
        # 4. For each meaningful lemma, consult KB
        findings = {}
        for lemma in lemmas:
            # meaning lookup (KB — יבש)
            meaning = self.kb.lookup_word(lemma, "he")
            syns = self.kb.get_synonyms(lemma)
            article = self.kb.get_article(lemma.capitalize())
            
            if meaning or syns or article:
                findings[lemma] = {
                    "meaning": meaning,
                    "synonyms": syns,
                    "article_excerpt": article[:100] if article else None,
                }
                self.regions["semantic_gateway"].activate(0.8)
                self.regions["hippocampus"].activate(0.3)
        
        trace.append(f"📚 KB lookups returned {len(findings)} concepts")
        
        # 5. Default Mode: association — שלב יצירתי
        associations = []
        self.regions["default_mode"].activate(0.7)
        for lemma, data in findings.items():
            for syn in data.get("synonyms", [])[:2]:
                associations.append(syn)
        if associations:
            trace.append(f"💭 Associations: {associations}")
        
        # 6. PFC: integrate + plan response
        self.regions["pfc"].activate(1.0)
        self.working_memory = list(findings.keys())[:7]  # 7±2
        trace.append(f"🧠 Working memory: {self.working_memory}")
        
        # 7. RECURRENT FEEDBACK — זה ההבדל מ-DAG
        # PFC משפיע חזרה על attention ו-wernicke
        self.regions["attention"].activate(self.regions["pfc"].activation * 0.5)
        self.regions["wernicke"].activate(self.regions["pfc"].activation * 0.3)
        trace.append("🔁 Recurrent feedback: PFC → attention, wernicke")
        
        # 8. Broca: generate response
        self.regions["broca"].activate(1.0)
        response_parts = []
        for lemma, data in findings.items():
            if data["meaning"]:
                response_parts.append(f"{lemma} → {data['meaning']}")
            if data["article_excerpt"]:
                response_parts.append(f"  ({data['article_excerpt']}...)")
        response = "\n".join(response_parts) if response_parts else "I don't know this."
        
        # 9. Log episodic memory
        self.episodic.append({
            "input": input_text,
            "lemmas": lemmas,
            "findings": list(findings.keys()),
            "response": response,
            "t": time.time(),
        })
        
        # 10. Decay activations
        for r in self.regions.values():
            r.decay()
        
        if verbose:
            print("\n".join(trace))
            print(f"\n🗣 Response:\n{response}")
        
        return {
            "trace": trace,
            "response": response,
            "activations": {n: r.activation for n, r in self.regions.items() if r.activation > 0.1},
        }
    
    # ─── Learning — Hebbian-like ───
    def learn(self, lemma: str, meaning_en: str, meaning_he: str, synonyms: List[str] = None):
        """
        הוסף entry ל-KB. זה "טעינת מילון", לא שינוי של המוח.
        המוח לא משתנה בין entry ל-entry. רק ה-lookups ישתנו.
        """
        self.kb.lexicon[lemma.lower()] = {
            "en": meaning_en,
            "he": meaning_he,
            "base": lemma.lower(),
        }
        if synonyms:
            self.kb.synonyms[lemma.lower()] = synonyms
        
    def dump_brain(self):
        print("\n━━━ BRAIN STATE ━━━")
        for n, r in self.regions.items():
            bar = "█" * int(r.activation * 20)
            print(f"  {n:18} {r.activation:.2f} {bar}")
    
    def show_wiring(self):
        print("\n━━━ BRAIN WIRING (pre-wired at birth) ━━━")
        for n, r in self.regions.items():
            print(f"  {n} → {sorted(r.connections)}")


# ═══════════════════════════════════════════════════════════════════════
#  DEMO — הסימולציה
# ═══════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print("═" * 75)
    print("  SIM: Dual Architecture — Cognitive Brain + Knowledge Base")
    print("═" * 75)
    
    # 1. Create empty KB (like a newborn — brain is wired, but KB is empty)
    kb = KnowledgeBase()
    brain = CognitiveBrain(kb)
    
    print("\n[1] Newborn state: brain wired, KB empty")
    brain.show_wiring()
    
    # 2. Try to process something — should fail gracefully
    print("\n[2] Query with empty KB:")
    brain.process("what is water?")
    
    # 3. Load some knowledge (like teaching a child)
    print("\n[3] Teaching the brain: loading KB entries...")
    brain.learn("water", "H2O, liquid essential for life", "מים",
                synonyms=["liquid", "fluid", "aqua"])
    brain.learn("fire",  "oxidation producing heat and light", "אש",
                synonyms=["flame", "blaze"])
    brain.learn("love",  "strong affection", "אהבה",
                synonyms=["care", "affection"])
    kb.articles["Water"] = (
        "Water is a transparent, tasteless liquid that is the main constituent of "
        "Earth's streams, lakes, and oceans. Its chemical formula is H2O."
    )
    kb.procedures["make_tea"] = [
        "Boil water",
        "Add tea leaves to a cup",
        "Pour hot water",
        "Wait 3 minutes",
    ]
    
    print(f"    KB now has {len(kb.lexicon)} words, {len(kb.articles)} articles")
    
    # 4. Process same query — should succeed
    print("\n[4] Same query, now with KB loaded:")
    brain.process("what is water?")
    brain.dump_brain()
    
    # 5. New concept the brain doesn't know yet — comes in a foreign lang
    print("\n[5] Foreign word 'ma' (imagine that's water in another language):")
    brain.kb.lexicon["ma"] = {"en": "water", "he": "מים", "base": "water"}
    brain.kb.synonyms["ma"] = ["water"]
    brain.process("ma is good")
    
    # 6. Episodic memory
    print("\n[6] Episodic memory (last 3 queries):")
    for ep in brain.episodic[-3:]:
        print(f"    Q: '{ep['input']}' → {ep['findings']}")
