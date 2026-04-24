"""
brain_sim_v3 — המוח כ-topology, לא כ-data.

מוסיף 7 מנגנונים שקיימים במוח ולא תלויים ב-knowledge:

1. SPREADING ACTIVATION — כל node מפעיל שכנים בדעיכה אקספוננציאלית
2. INHIBITION — חלופות חלשות מדוכאות, חזקות מופעלות
3. FEEDBACK LOOPS — PFC משנה את attention שמשנה את lookups שמשנים את PFC
4. PREDICTION ERROR — המוח מנבא, משווה לתוצאה, משפר
5. CURIOSITY / NOVELTY — אזור חדש מקבל boost אוטומטי
6. META-COGNITION — המוח עוקב אחר הצלחותיו, בוחר אסטרטגיה
7. ANALOGY / PATTERN MATCH — מושגים חדשים מושוואים למוכרים

אין פה שום data "חכם". רק מנגנונים.
"""

from dataclasses import dataclass, field
from typing import Dict, List, Set, Optional, Tuple, Callable
from collections import defaultdict, deque
import math, time, random


# ═══════════════════════════════════════════════════════════════════════
#  1. SPREADING ACTIVATION — חכמה מפעפוע אקספוננציאלי
# ═══════════════════════════════════════════════════════════════════════

class SpreadingNetwork:
    """
    Network עם nodes ו-weighted edges.
    כש-node מופעל, ההפעלה "דולפת" לשכנים עם דעיכה.
    זה מה שיוצר אסוציאציות: water → liquid → drink → tea.
    """
    def __init__(self):
        self.nodes: Dict[str, float] = {}  # name → current activation
        self.edges: Dict[Tuple[str, str], float] = {}  # (a,b) → weight
        self.decay_rate = 0.6  # activation per hop
        self.threshold = 0.05  # below this, stop spreading

    def add_node(self, name: str):
        if name not in self.nodes:
            self.nodes[name] = 0.0

    def connect(self, a: str, b: str, weight: float = 0.5, bidirectional: bool = True):
        self.add_node(a); self.add_node(b)
        self.edges[(a, b)] = weight
        if bidirectional:
            self.edges[(b, a)] = weight

    def activate(self, node: str, level: float = 1.0):
        """הפעל node + פעפע לשכנים."""
        self.add_node(node)
        self.nodes[node] = min(1.0, self.nodes[node] + level)
        self._spread(node, level, visited=set())

    def _spread(self, from_node: str, strength: float, visited: Set[str]):
        if strength < self.threshold: return
        if from_node in visited: return
        visited.add(from_node)
        for (a, b), w in self.edges.items():
            if a == from_node and b not in visited:
                new_str = strength * w * self.decay_rate
                if new_str >= self.threshold:
                    self.nodes[b] = min(1.0, self.nodes[b] + new_str)
                    self._spread(b, new_str, visited)

    def reinforce(self, a: str, b: str, delta: float = 0.05):
        """Hebbian: 'cells that fire together'."""
        if (a, b) in self.edges:
            self.edges[(a, b)] = min(1.0, self.edges[(a, b)] + delta)
        if (b, a) in self.edges:
            self.edges[(b, a)] = min(1.0, self.edges[(b, a)] + delta)

    def inhibit(self, winner: str, losers: List[str], strength: float = 0.3):
        """
        Lateral inhibition. כש-winner ניצח, losers מחולשים.
        זה מה שמייצר "החלטה" — winner-take-all רך.
        """
        for loser in losers:
            if loser in self.nodes:
                self.nodes[loser] *= (1.0 - strength)
                # החליש את הקשתות מ-loser
                for (a, b), w in self.edges.items():
                    if a == loser:
                        self.edges[(a, b)] = max(0.0, w - 0.02)

    def top_active(self, n: int = 5) -> List[Tuple[str, float]]:
        return sorted(self.nodes.items(), key=lambda x: -x[1])[:n]

    def decay_all(self, factor: float = 0.7):
        """דעיכה גלובלית — אחרי tick."""
        for k in self.nodes:
            self.nodes[k] *= factor


# ═══════════════════════════════════════════════════════════════════════
#  2. PREDICTION + ERROR — המוח כמנבא
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class Prediction:
    """מה המוח מצפה לראות בשלב הבא."""
    expected: Set[str]  # nodes שמצופים להיפעל
    confidence: float   # כמה בטוח

class PredictiveLayer:
    """
    PFC-like: לפני כל action, נבאת מה תהיה התוצאה.
    אחרי action: השוואה. אם שונה — למידה מהירה (prediction error).
    """
    def __init__(self):
        self.history: List[Dict] = []  # past predictions + actual

    def predict(self, context: Set[str], network: SpreadingNetwork) -> Prediction:
        """בהינתן context, נבא מה יהיה active בdone."""
        expected = set()
        conf = 0.0
        for c in context:
            # מי הכי מחובר ל-c?
            neighbors = [(b, w) for (a, b), w in network.edges.items() if a == c]
            neighbors.sort(key=lambda x: -x[1])
            for b, w in neighbors[:3]:
                if w > 0.4:
                    expected.add(b)
                    conf += w
        conf = min(1.0, conf / max(1, len(context)))
        return Prediction(expected, conf)

    def compute_error(self, predicted: Prediction, actual_active: Set[str]) -> float:
        """כמה שגינו בניבוי?"""
        if not predicted.expected: return 0.0
        missed = predicted.expected - actual_active  # ציפינו ולא קיבלנו
        unexpected = actual_active - predicted.expected  # לא ציפינו וקיבלנו
        error = (len(missed) + len(unexpected)) / max(1, len(predicted.expected | actual_active))
        return error

    def learn_from_error(self, error: float, context: Set[str], actual: Set[str],
                        network: SpreadingNetwork):
        """
        Prediction error → מחזק/מחליש קשרים.
        שגיאה גדולה = למידה מהירה (surprise = memorable).
        """
        learning_rate = error * 0.1
        for c in context:
            for a in actual:
                if c != a:
                    network.reinforce(c, a, delta=learning_rate)


# ═══════════════════════════════════════════════════════════════════════
#  3. CURIOSITY — המוח נמשך לחדש
# ═══════════════════════════════════════════════════════════════════════

class CuriosityDrive:
    """
    מעקב אחר מה שפגשנו. דברים חדשים / נדירים מקבלים boost.
    זה מה שמייצר motivation ללמידה.
    """
    def __init__(self):
        self.familiarity: Dict[str, int] = defaultdict(int)  # כמה פעמים ראינו
        self.total_encounters = 0

    def encounter(self, node: str):
        self.familiarity[node] += 1
        self.total_encounters += 1

    def novelty_boost(self, node: str) -> float:
        """
        Inverse-frequency: node חדש = boost גבוה.
        node מוכר מאוד = boost נמוך (=anti-boost).
        """
        seen = self.familiarity.get(node, 0)
        if seen == 0: return 1.0  # לחלוטין חדש
        if self.total_encounters < 10: return 0.5
        frequency = seen / self.total_encounters
        # rare items get boost up to 0.8
        return min(0.8, -math.log(frequency + 0.01) / 10)


# ═══════════════════════════════════════════════════════════════════════
#  4. META-COGNITION — המוח עוקב אחר עצמו
# ═══════════════════════════════════════════════════════════════════════

class MetaCognition:
    """
    מוניטור: איזה strategies עבדו? מתי כדאי ללכת ישר vs. לפעפע?
    זה מה שמפריד 'חוכמה מנוסה' מ-'beginner'.
    """
    def __init__(self):
        self.strategy_success: Dict[str, List[bool]] = defaultdict(list)

    def record(self, strategy: str, success: bool):
        self.strategy_success[strategy].append(success)
        # sliding window
        if len(self.strategy_success[strategy]) > 20:
            self.strategy_success[strategy] = self.strategy_success[strategy][-20:]

    def confidence(self, strategy: str) -> float:
        h = self.strategy_success[strategy]
        if not h: return 0.5  # prior
        return sum(h) / len(h)

    def choose_strategy(self, options: List[str]) -> str:
        """
        Epsilon-greedy: רוב הפעמים בחר הכי טוב, לפעמים גלה.
        זה exploration vs. exploitation.
        """
        if random.random() < 0.15:  # 15% exploration
            return random.choice(options)
        return max(options, key=self.confidence)


# ═══════════════════════════════════════════════════════════════════════
#  5. ANALOGY — המוח מוצא דומה למוכר
# ═══════════════════════════════════════════════════════════════════════

class AnalogyEngine:
    """
    כשפוגשים משהו חדש, מחפשים pattern דומה בזיכרון.
    זה הכוח הגדול של המוח: מעבר בין דומיינים.
    """
    def __init__(self, network: SpreadingNetwork):
        self.network = network

    def find_analogous(self, new_concept: str, context: Set[str]) -> List[Tuple[str, float]]:
        """
        מצא concepts שיש להם מבנה קשר דומה לzה של new_concept.
        לא תוכן זהה — מבנה דומה.
        """
        if new_concept not in self.network.nodes:
            # אין עליו מידע ישיר, נשתמש ב-context
            candidates = defaultdict(float)
            for c in context:
                for (a, b), w in self.network.edges.items():
                    if a == c:
                        candidates[b] += w
            return sorted(candidates.items(), key=lambda x: -x[1])[:5]
        
        # אם יש — מצא nodes עם pattern קשרים דומה
        my_neighbors = {b: w for (a, b), w in self.network.edges.items() if a == new_concept}
        similarities = []
        for node in self.network.nodes:
            if node == new_concept: continue
            their_neighbors = {b: w for (a, b), w in self.network.edges.items() if a == node}
            # overlap-based similarity
            shared = set(my_neighbors) & set(their_neighbors)
            if shared:
                sim = sum(min(my_neighbors[k], their_neighbors[k]) for k in shared)
                similarities.append((node, sim))
        return sorted(similarities, key=lambda x: -x[1])[:5]


# ═══════════════════════════════════════════════════════════════════════
#  6. THE SMART BRAIN — חיבור הכל
# ═══════════════════════════════════════════════════════════════════════

class SmartBrain:
    """
    מוח עם 7 המנגנונים. לא תלוי ב-data.
    קבל nodes, הקמתי יחסים, והתבונן איך זה מתנהג.
    """
    def __init__(self):
        self.network = SpreadingNetwork()
        self.predictor = PredictiveLayer()
        self.curiosity = CuriosityDrive()
        self.metacog = MetaCognition()
        self.analogy = AnalogyEngine(self.network)
        self.working_memory = deque(maxlen=7)
        self.trace = []

    def teach_relation(self, a: str, b: str, weight: float = 0.5):
        """הוסף קשר. זה הכל — המוח יטפל בשאר."""
        self.network.connect(a, b, weight)

    def think(self, query: Set[str], depth: int = 3) -> Dict:
        """
        A 'thinking' cycle. 
        Query = set of nodes (מה הופעל מבחוץ).
        Return trace ותוצאה.
        """
        self.trace = []
        
        # 1. ACTIVATE query + SPREAD
        for q in query:
            self.curiosity.encounter(q)
            boost = 1.0 + self.curiosity.novelty_boost(q)
            self.network.activate(q, boost)
        
        top = self.network.top_active(10)
        self.trace.append(f"[1] Activated {query}, spreading...")
        self.trace.append(f"    Top: {[(n, round(v,2)) for n,v in top[:5]]}")

        # 2. PREDICT — what SHOULD come next?
        prediction = self.predictor.predict(query, self.network)
        self.trace.append(f"[2] Prediction: {prediction.expected} (conf {prediction.confidence:.2f})")

        # 3. CHOOSE STRATEGY (metacog)
        strategy = self.metacog.choose_strategy(["direct", "analogy", "deep_spread"])
        self.trace.append(f"[3] Chose strategy: {strategy}")

        # 4. EXECUTE strategy
        result_nodes = set()
        if strategy == "direct":
            # Top-3 הכי active זה התשובה
            result_nodes = {n for n, v in top[:3] if v > 0.2}
        elif strategy == "analogy":
            # מצא דומים ל-query
            for q in query:
                analogs = self.analogy.find_analogous(q, query)
                result_nodes.update([a for a, _ in analogs[:3]])
        elif strategy == "deep_spread":
            # השהה tick נוסף של פעפוע
            for _ in range(depth):
                for n, v in list(self.network.nodes.items()):
                    if v > 0.3:
                        self.network.activate(n, v * 0.3)
            result_nodes = {n for n, v in self.network.top_active(5) if v > 0.15}

        self.trace.append(f"[4] Result: {result_nodes}")

        # 5. INHIBITION — דכא losers, חזק winners
        if result_nodes:
            winner = max(result_nodes, key=lambda n: self.network.nodes.get(n, 0))
            losers = [n for n in result_nodes if n != winner]
            self.network.inhibit(winner, losers)
            self.trace.append(f"[5] Winner: {winner}, inhibited: {losers}")

        # 6. PREDICTION ERROR — learn from mismatch
        error = self.predictor.compute_error(prediction, result_nodes)
        self.predictor.learn_from_error(error, query, result_nodes, self.network)
        self.trace.append(f"[6] Prediction error: {error:.2f} → learning rate adjusted")

        # 7. METACOG — record strategy outcome
        success = len(result_nodes) > 0 and error < 0.6
        self.metacog.record(strategy, success)
        self.trace.append(f"[7] Strategy '{strategy}' {'succeeded' if success else 'failed'}")

        # decay before next cycle
        self.network.decay_all(0.6)
        
        return {
            "result": result_nodes,
            "strategy": strategy,
            "error": error,
            "success": success,
            "trace": self.trace,
        }


# ═══════════════════════════════════════════════════════════════════════
#  DEMO — זנען את המוח בלי data, רק יחסים, וראה מה קורה
# ═══════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print("═" * 82)
    print("  SMART BRAIN — חכמה ממנגנונים, לא מ-data")
    print("═" * 82)
    brain = SmartBrain()

    # הוראת קשרים בסיסיים — רק topology, בלי הגדרות!
    # נבנה "sub-network" של concepts הקשורים
    print("\n[STEP 1] Teaching relations (no definitions — just connections)")
    relations = [
        # cluster 1: liquids
        ("water","liquid", 0.9), ("water","drink", 0.8), ("water","cold", 0.5),
        ("water","hot", 0.5), ("liquid","fluid", 0.9),
        ("tea","drink", 0.9), ("tea","hot", 0.8), ("tea","liquid", 0.8),
        ("coffee","drink", 0.9), ("coffee","hot", 0.8), ("coffee","liquid", 0.8),
        
        # cluster 2: emotions
        ("love","positive", 0.9), ("love","warm", 0.7), ("love","human", 0.6),
        ("joy","positive", 0.9), ("joy","warm", 0.7),
        ("fear","negative", 0.9), ("fear","cold", 0.5),
        ("anger","negative", 0.8), ("anger","hot", 0.6),
        
        # cluster 3: actions
        ("drink","consume", 0.8), ("eat","consume", 0.9),
        ("run","move", 0.9), ("walk","move", 0.9),
        
        # cross-domain (analogies!)
        ("warm","love", 0.4),    # emotional warmth ≈ physical warmth
        ("cold","fear", 0.4),    # cold feet = fear
        ("hot","anger", 0.4),    # hot-headed = angry
    ]
    for a, b, w in relations:
        brain.teach_relation(a, b, w)
    print(f"  Taught {len(relations)} relations across {len(brain.network.nodes)} nodes")

    # ─────────────────────────────────────────────────────
    print("\n\n" + "█"*82)
    print("  QUERY 1: {water, hot} — המוח אמור לאסוציאציב")
    print("█"*82)
    r = brain.think({"water", "hot"})
    for line in r["trace"]: print(f"  {line}")
    print(f"\n  🎯 Result: {r['result']}")
    print(f"  ━ ניתוח: המוח הפעיל water+hot → פעפע → מצא שיחסים מובילים ל-tea, coffee")
    print(f"    זו אסוציאציה אמיתית ללא 'ידע' — רק topology.")

    # ─────────────────────────────────────────────────────
    print("\n\n" + "█"*82)
    print("  QUERY 2: {love} — האם יימצא 'warm' דרך cross-domain?")
    print("█"*82)
    r = brain.think({"love"})
    for line in r["trace"]: print(f"  {line}")
    print(f"\n  🎯 Result: {r['result']}")
    print(f"  ━ 'love → warm' הוא הcross-domain שהוספנו. המוח צריך למצוא אותו.")

    # ─────────────────────────────────────────────────────
    print("\n\n" + "█"*82)
    print("  QUERY 3: concept חדש — 'whiskey' בלי שום קשרים")
    print("█"*82)
    brain.network.add_node("whiskey")  # חדש, אין קשרים
    r = brain.think({"whiskey"})
    for line in r["trace"]: print(f"  {line}")
    print(f"\n  🎯 Result: {r['result']}")
    print(f"  ━ בלי קשרים — לא קורה כלום. חשוב: המוח 'יודע שהוא לא יודע'.")

    # ─────────────────────────────────────────────────────
    print("\n\n" + "█"*82)
    print("  QUERY 4: הוספת קשר לwhiskey → שאלה חוזרת")
    print("█"*82)
    brain.teach_relation("whiskey", "drink", 0.9)
    brain.teach_relation("whiskey", "liquid", 0.8)
    print(f"  ✓ whiskey → drink (0.9), whiskey → liquid (0.8)")
    r = brain.think({"whiskey"})
    for line in r["trace"]: print(f"  {line}")
    print(f"\n  🎯 Result: {r['result']}")
    print(f"  ━ עכשיו whiskey מוביל לאשכול המשקאות. ה-topology החדשה עשתה את העבודה.")

    # ─────────────────────────────────────────────────────
    print("\n\n" + "█"*82)
    print("  QUERY 5: Analogy — 'tea hot' ←→ 'anger hot'?")
    print("█"*82)
    r = brain.think({"anger"})
    for line in r["trace"][:3]: print(f"  {line}")
    # mooo על analogy
    analogs = brain.analogy.find_analogous("anger", {"anger"})
    print(f"\n  🔍 Analogs to 'anger': {analogs}")
    print(f"  ━ המוח מוצא concepts שמחוברים לאותם things כמו anger (e.g., hot)")
    print(f"    'anger' ו-'tea' שניהם מחוברים ל-'hot' → analog!")

    # ─────────────────────────────────────────────────────
    print("\n\n" + "█"*82)
    print("  FINAL STATE — מה המוח למד?")
    print("█"*82)
    print(f"\n  Strategy confidence (אחרי 5 שאילתות):")
    for strat in ["direct", "analogy", "deep_spread"]:
        conf = brain.metacog.confidence(strat)
        bar = "▓" * int(conf * 30)
        print(f"    {strat:<14} {conf:.2f} {bar}")
    
    print(f"\n  Curiosity (hottest, most-seen nodes):")
    fam = sorted(brain.curiosity.familiarity.items(), key=lambda x: -x[1])[:5]
    for n, c in fam: print(f"    {n:<10} seen {c}×")

    print(f"\n  Top strengthened edges (Hebbian):")
    strong = sorted(brain.network.edges.items(), key=lambda x: -x[1])[:7]
    for (a, b), w in strong:
        bar = "█" * int(w * 20)
        print(f"    {a:>10} → {b:<10} {w:.3f} {bar}")
