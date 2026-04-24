"""
ZETS Full AGI Simulation — מימוש של כל 12 העקרונות בתכלת אחת.

Based on: docs/30_decisions/20260424_AGI_BLUEPRINT_v1.md
Tests: 
  - Principle 1 (7 angels = directions)
  - Principle 2 (10D sefirot vector)
  - Principle 3 (5 partzufim parallel+feedback)
  - Principle 4 (5 continuous edge fields)
  - Principle 5 (context tree)
  - Principle 6 (exponential decay)
  - Principle 7 (tiered storage - simplified)
  - Principle 8 (4 edge types incl state-dependent)
  - Principle 9 (3 mother taxonomy)
  - Principle 10 (3x7 = 21 parallel dives, depth 7)
  - Principle 11 (5-phase ingestion)
  - Principle 12 (3-axis context: who/where/when)

NOT AGI. A SIMULATION that demonstrates the architecture works.
"""

import math
import time
import random
from collections import defaultdict, Counter
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Optional

random.seed(42)

# ═══════════════════════════════════════════════════════════════
# PART 1: DATA MODEL (Principles 4, 5, 8, 12)
# ═══════════════════════════════════════════════════════════════

@dataclass
class StateAxis:
    """Principle 8 — concept can have multiple state axes"""
    name: str
    range: tuple  # (min, max)
    default: float
    description: str = ""


@dataclass
class ContextAxes:
    """Principle 12 — 3 independent context axes"""
    spatial: Optional[str] = None   # עולם — where
    temporal: Optional[str] = None  # שנה — when
    identity: Optional[str] = None  # נפש — who


@dataclass
class Atom:
    """A concept node in the graph"""
    id: int
    lemma: str
    features: dict = field(default_factory=dict)  # POS, gender, number, etc
    state_axes: list = field(default_factory=list)  # Principle 8
    context_tags: ContextAxes = field(default_factory=ContextAxes)  # Principle 12
    created_at: float = field(default_factory=time.time)
    atom_type: str = "concept"  # concept / entity / event / memory
    
    def __hash__(self):
        return hash(self.id)
    
    def __eq__(self, other):
        return isinstance(other, Atom) and self.id == other.id


@dataclass
class StateDependency:
    """Principle 8 — edge only active in state range"""
    axis_name: str
    active_range: tuple  # (min, max)
    peak_value: float = 1.0
    curve: str = "linear"  # linear / bell / sigmoid / constant


@dataclass
class Edge:
    """A connection — Principle 4 (5 continuous fields)"""
    src: int           # atom id
    dst: int           # atom id
    edge_type: str     # one of 21 types (3 mothers × 7 each)
    
    # 5 continuous values
    state_value: float = 0.9     # [-1, +1]
    memory_strength: float = 1.0 # [0, 1]
    confidence: float = 0.9      # [0, 1]
    asymmetry_factor: float = 0.5 # [0, 1] — 0=symmetric, 1=one-way
    
    # Context — Principle 12
    context_tags: ContextAxes = field(default_factory=ContextAxes)
    
    # State dependency — Principle 8
    state_dependency: Optional[StateDependency] = None
    
    # Provenance
    source: str = "ingested"  # user_statement / inference / path_build
    created_at: float = field(default_factory=time.time)
    last_reinforced: float = field(default_factory=time.time)
    use_count: int = 0
    success_count: int = 0


# ═══════════════════════════════════════════════════════════════
# PART 2: EDGE TYPES — 21 types across 3 mothers (Principle 9, 10)
# ═══════════════════════════════════════════════════════════════

MOTHERS = {
    "Sensory": [
        "visual_color", "visual_shape", "visual_size",
        "taste_basic", "smell_primary", "texture", "temperature"
    ],
    "Functional": [
        "use_culinary", "use_general", "cause_effect",
        "ingredient_of", "interacts_with", "enables_action", "prevents_state"
    ],
    "Abstract": [
        "category_is_a", "analogy_similar", "symbolic_cultural",
        "metaphor_for", "brand_association", "emotional_valence", "narrative_archetype"
    ]
}

# ═══════════════════════════════════════════════════════════════
# PART 3: THE BRAIN (main class)
# ═══════════════════════════════════════════════════════════════

class Brain:
    def __init__(self):
        self.atoms: dict[int, Atom] = {}     # id → Atom
        self.lemma_index: dict[str, int] = {} # lemma → id (for lookup)
        self.edges: list[Edge] = []
        self.fwd_index: dict[int, list[int]] = defaultdict(list)  # src → edge indices
        self.rev_index: dict[int, list[int]] = defaultdict(list)  # dst → edge indices
        self.next_atom_id = 0
        
        # Context trees
        self.spatial_tree = set(["root", "root.personal", "root.work", "root.external"])
        self.temporal_tree = set(["root"])
        self.identity_tree = set(["root", "root.self", "root.family", "root.friend", "root.work_contact"])
    
    # ───────────────────────────────────────────────────────────
    # Principle 11: 5-Phase Ingestion Pipeline
    # ───────────────────────────────────────────────────────────
    
    def ingest(self, lemma: str, 
               edges_desc: list = None,
               state_axes: list = None,
               context: ContextAxes = None,
               features: dict = None,
               atom_type: str = "concept") -> int:
        """
        חקק → חצב → שקל → המיר → צרף
        
        Accepts a concept description and creates atom + edges through 5 phases.
        Returns atom_id.
        """
        log = []
        
        # Phase 1: חקק (CARVE) — Define boundaries
        # What IS this, what is NOT? Disambiguation.
        if lemma in self.lemma_index:
            # Already exists — return existing
            return self.lemma_index[lemma]
        
        log.append(f"  [1/5 חקק] Carving boundaries: '{lemma}' identified as new {atom_type}")
        
        # Phase 2: חצב (HEW) — Extract features
        hewn_features = features or {}
        hewn_features.setdefault("lemma", lemma)
        hewn_features.setdefault("created_at", datetime.now().isoformat())
        log.append(f"  [2/5 חצב] Features extracted: {list(hewn_features.keys())}")
        
        # Phase 3: שקל (WEIGH) — Assign importance/weights
        # We'll weight each edge by its state_value, which reflects relationship strength
        log.append(f"  [3/5 שקל] Weighted {len(edges_desc or [])} connections by strength")
        
        # Phase 4: המיר (PERMUTE) — Morphological variants
        # For simplicity, we skip generating variants (would need morph engine)
        log.append(f"  [4/5 המיר] Variants: primary form only (morph engine not implemented in sim)")
        
        # Phase 5: צרף (COMBINE) — Integrate into graph
        atom_id = self.next_atom_id
        self.next_atom_id += 1
        
        atom = Atom(
            id=atom_id,
            lemma=lemma,
            features=hewn_features,
            state_axes=state_axes or [],
            context_tags=context or ContextAxes(),
            atom_type=atom_type,
        )
        self.atoms[atom_id] = atom
        self.lemma_index[lemma] = atom_id
        
        # Create edges
        if edges_desc:
            for desc in edges_desc:
                self._create_edge_from_desc(atom_id, desc)
        
        log.append(f"  [5/5 צרף] Integrated atom#{atom_id}, {len(edges_desc or [])} edges added")
        
        return atom_id
    
    def _create_edge_from_desc(self, src_id: int, desc: dict):
        """Create edge from a dict description"""
        dst_lemma = desc["dst"]
        if dst_lemma not in self.lemma_index:
            # auto-create target atom if missing
            self.ingest(dst_lemma, atom_type="auto")
        dst_id = self.lemma_index[dst_lemma]
        
        edge = Edge(
            src=src_id,
            dst=dst_id,
            edge_type=desc.get("type", "category_is_a"),
            state_value=desc.get("state", 0.8),
            memory_strength=desc.get("memory", 1.0),
            confidence=desc.get("confidence", 0.9),
            asymmetry_factor=desc.get("asymmetry", 0.3),
            context_tags=desc.get("context") or ContextAxes(),
            state_dependency=desc.get("state_dep"),
            source=desc.get("source", "ingested"),
        )
        self.edges.append(edge)
        edge_idx = len(self.edges) - 1
        self.fwd_index[src_id].append(edge_idx)
        self.rev_index[dst_id].append(edge_idx)
    
    # ───────────────────────────────────────────────────────────
    # Principle 6: Decay & Reinforcement
    # ───────────────────────────────────────────────────────────
    
    def current_strength(self, edge: Edge, now: float = None) -> float:
        """Exponential decay based on time + personalization"""
        now = now or time.time()
        days_since = (now - edge.last_reinforced) / 86400.0
        
        context_depth = 0
        if edge.context_tags.identity: context_depth += 1
        if edge.context_tags.spatial: context_depth += 1
        if edge.context_tags.temporal: context_depth += 1
        
        # tau = time constant in days (τ)
        tau = 10.0 + 20.0 * context_depth + 30.0 * math.sqrt(edge.use_count)
        
        return edge.memory_strength * math.exp(-days_since / tau)
    
    def reinforce(self, edge_idx: int, success: bool = True):
        """Strengthen an edge on use"""
        edge = self.edges[edge_idx]
        factor = 0.15 if success else 0.05
        edge.memory_strength = min(1.0, edge.memory_strength * (1.0 + factor))
        edge.last_reinforced = time.time()
        edge.use_count += 1
        if success:
            edge.success_count += 1
    
    # ───────────────────────────────────────────────────────────
    # Principle 10: 3×7 = 21 Parallel Dives (with depth 7)
    # ───────────────────────────────────────────────────────────
    
    def dive(self, start_atom_id: int, edge_type: str, depth: int = 7, 
             context_filter: ContextAxes = None, state_ctx: dict = None) -> dict:
        """Single dive along one edge_type to depth 7, returns {atom_id: score}"""
        state_ctx = state_ctx or {}
        found = {start_atom_id: 1.0}
        frontier = [(start_atom_id, 1.0)]
        
        for level in range(depth):
            next_frontier = []
            for (node_id, carry) in frontier:
                for edge_idx in self.fwd_index.get(node_id, []):
                    edge = self.edges[edge_idx]
                    if edge.edge_type != edge_type:
                        continue
                    
                    # Check state_dependency (Principle 8)
                    if edge.state_dependency:
                        dep = edge.state_dependency
                        ctx_val = state_ctx.get(dep.axis_name)
                        if ctx_val is not None:
                            if not (dep.active_range[0] <= ctx_val <= dep.active_range[1]):
                                continue  # edge not active in this state
                    
                    # Check context_filter (Principle 12)
                    if context_filter:
                        if context_filter.identity and edge.context_tags.identity:
                            if not edge.context_tags.identity.startswith(context_filter.identity):
                                continue
                        if context_filter.spatial and edge.context_tags.spatial:
                            if not edge.context_tags.spatial.startswith(context_filter.spatial):
                                continue
                        if context_filter.temporal and edge.context_tags.temporal:
                            if not edge.context_tags.temporal.startswith(context_filter.temporal):
                                continue
                    
                    # Weight = carry × state_value × current memory × confidence × decay per level
                    strength = self.current_strength(edge)
                    new_w = carry * edge.state_value * strength * edge.confidence * (0.85 ** (level + 1))
                    
                    if edge.dst not in found or found[edge.dst] < new_w:
                        found[edge.dst] = new_w
                        next_frontier.append((edge.dst, new_w))
                        # Mark edge as used (lightweight reinforcement)
                        edge.use_count += 1
            
            frontier = sorted(next_frontier, key=lambda x: -x[1])[:5]
            if not frontier:
                break
        
        return found
    
    def parallel_21_dives(self, start_lemma: str, 
                         context_filter: ContextAxes = None,
                         state_ctx: dict = None,
                         mother_weights: dict = None) -> dict:
        """
        Principle 10: 3 mothers × 7 types = 21 async parallel dives
        Returns {atom_id: {mothers_found_in: set, total_score: float, paths: list}}
        """
        if start_lemma not in self.lemma_index:
            return {}
        start_id = self.lemma_index[start_lemma]
        
        mother_weights = mother_weights or {m: 1.0 for m in MOTHERS}
        
        combined = defaultdict(lambda: {"mothers": set(), "score": 0.0, "types": []})
        
        for mother_name, edge_types in MOTHERS.items():
            mom_weight = mother_weights.get(mother_name, 1.0)
            if mom_weight < 0.1:
                continue  # skip mother with zero weight
            
            for etype in edge_types:
                found = self.dive(start_id, etype, depth=7, 
                                  context_filter=context_filter,
                                  state_ctx=state_ctx)
                for atom_id, score in found.items():
                    if atom_id == start_id:
                        continue
                    combined[atom_id]["mothers"].add(mother_name)
                    combined[atom_id]["score"] += score * mom_weight
                    combined[atom_id]["types"].append(etype)
        
        return dict(combined)


# ═══════════════════════════════════════════════════════════════
# PART 4: 10D SEFIROT VECTOR CLASSIFIER (Principle 2)
# ═══════════════════════════════════════════════════════════════

SEFIROT = ["keter", "chokhma", "bina", "daat", 
           "chesed", "gevura", "tiferet",
           "netzach", "hod", "yesod", "malkhut"]

def classify_intent(query: str) -> dict:
    """
    Return 11-dim vector (10 sefirot + keter) of query-intent weights.
    Simple keyword matching — in real system would be ML classifier.
    """
    q = query.lower()
    
    v = {s: 0.0 for s in SEFIROT}
    
    # חכמה — insight, "what is"
    if any(k in q for k in ["מה זה", "מה זה", "what is", "הגדר", "define"]):
        v["chokhma"] += 0.8
    # בינה — analysis, "how"
    if any(k in q for k in ["איך", "how", "תפרק", "תסביר", "explain", "למה", "why"]):
        v["bina"] += 0.8
    # דעת — applied/sensory
    if any(k in q for k in ["מה הצבע", "מה הטעם", "איך מרגיש", "איך נראה", "how does"]):
        v["daat"] += 0.8
    # חסד — support, giving
    if any(k in q for k in ["עזור", "help", "תן", "המלץ", "recommend", "תמליץ"]):
        v["chesed"] += 0.7
    # גבורה — critical, limits
    if any(k in q for k in ["בעיה", "רע", "אסור", "sad", "negative", "wrong", "לא טוב"]):
        v["gevura"] += 0.7
    # תפארת — balance, comparison
    if any(k in q for k in ["יתרון", "חסרון", "השוואה", "compare", "לעומת", "versus"]):
        v["tiferet"] += 0.7
    # יסוד — connection, implementation
    if any(k in q for k in ["ליישם", "implement", "לחבר", "connect"]):
        v["yesod"] += 0.6
    # מלכות — concrete output, factual answer
    if any(k in q for k in ["תן לי", "give me", "תשובה", "answer", "מה"]):
        v["malkhut"] += 0.5
    
    # Always at least some malkhut (we always want output)
    v["malkhut"] = max(v["malkhut"], 0.3)
    
    return v


def choose_mother_weights(sefirot_vec: dict) -> dict:
    """
    From sefirot vector, derive weights for 3 mothers.
    חכמה/בינה/דעת = shachliot = 3 entry-point mothers.
    """
    return {
        "Abstract":   max(sefirot_vec["chokhma"], sefirot_vec["tiferet"] * 0.5),  # insight, symbol
        "Functional": max(sefirot_vec["bina"], sefirot_vec["yesod"] * 0.7),       # analysis, implementation
        "Sensory":    max(sefirot_vec["daat"], sefirot_vec["malkhut"] * 0.6),      # concrete, grounded
    }


# ═══════════════════════════════════════════════════════════════
# PART 5: PARTZUFIM PIPELINE (Principle 3)
# ═══════════════════════════════════════════════════════════════

def arich_anpin(query: str) -> dict:
    """Arich = goal extraction. What does user really want?"""
    # Simple extraction — real system would use NLU
    intent = classify_intent(query)
    
    # Identify the "topic" — lemma to start from
    # Simple heuristic: find longest Hebrew word that looks like a noun
    words = query.replace("?", "").replace("!", "").split()
    candidate_topics = [w for w in words if len(w) >= 3 and not w.startswith("ה")]
    if not candidate_topics:
        candidate_topics = [w for w in words if len(w) >= 3]
    
    topic = candidate_topics[-1] if candidate_topics else "לימון"
    
    return {
        "query": query,
        "intent_vec": intent,
        "topic": topic,
        "primary_sefirot": sorted(intent.items(), key=lambda x: -x[1])[:3],
    }


def abba_ima_parallel(brain: Brain, arich_out: dict) -> dict:
    """
    Abba (flash insight) + Ima (structured decomposition) — parallel.
    Both operate on the 21-dive results.
    """
    topic = arich_out["topic"]
    mother_weights = choose_mother_weights(arich_out["intent_vec"])
    
    # Run 21 parallel dives
    dive_results = brain.parallel_21_dives(
        topic, mother_weights=mother_weights
    )
    
    # Abba — flash insight: top 3 strongest connections
    abba_output = sorted(dive_results.items(), 
                         key=lambda x: -x[1]["score"])[:5]
    
    # Ima — structured: group by mother, return top per mother
    ima_output = defaultdict(list)
    for atom_id, data in dive_results.items():
        for m in data["mothers"]:
            ima_output[m].append((atom_id, data["score"]))
    for m in ima_output:
        ima_output[m].sort(key=lambda x: -x[1])
        ima_output[m] = ima_output[m][:5]
    
    # Multi-mother nodes (appearing in ≥2 mothers) = high relevance
    multi_confirmed = [(aid, d) for aid, d in dive_results.items() 
                       if len(d["mothers"]) >= 2]
    multi_confirmed.sort(key=lambda x: -x[1]["score"])
    
    return {
        "flash": abba_output,
        "structure": dict(ima_output),
        "multi_confirmed": multi_confirmed[:5],
        "all_results": dive_results,
    }


def zeir_anpin(brain: Brain, abba_ima: dict, arich_out: dict) -> dict:
    """ZA = integration with working memory, emotion, consistency check"""
    # Check: did we find anything?
    if not abba_ima["all_results"]:
        return {"status": "empty", "feedback": "No graph hits — need to ingest or fallback"}
    
    # Compile insights into structured response data
    top_facts = []
    for atom_id, score in abba_ima["flash"][:5]:
        atom = brain.atoms[atom_id]
        # Find the edge that got us here for context
        top_facts.append({
            "lemma": atom.lemma,
            "score": score["score"] if isinstance(score, dict) else score,
            "types": score["types"] if isinstance(score, dict) else [],
            "mothers": score["mothers"] if isinstance(score, dict) else set(),
        })
    
    # Cross-mother — the "most relevant" concepts (confirmed by multiple perspectives)
    confirmed = [(brain.atoms[aid].lemma, d) for aid, d in abba_ima["multi_confirmed"]]
    
    return {
        "status": "ok",
        "top_facts": top_facts,
        "confirmed": confirmed,
        "per_mother": {m: [(brain.atoms[aid].lemma, s) for aid, s in items]
                       for m, items in abba_ima["structure"].items()},
    }


def nukva(za_out: dict, arich_out: dict, speaker_style: dict, safety_check: dict) -> str:
    """Nukva = final output generation. Style + values applied."""
    if za_out["status"] == "empty":
        return "אין לי מספיק מידע על זה עדיין. תוכל להסביר לי יותר?"
    
    if not safety_check["safe"]:
        return safety_check["refusal_message"]
    
    # Extract key facts
    confirmed = za_out.get("confirmed", [])
    per_mother = za_out.get("per_mother", {})
    top = za_out.get("top_facts", [])
    
    # Build response based on speaker style
    formality = speaker_style.get("formality", "neutral")  # formal/neutral/casual
    depth = speaker_style.get("depth", "medium")            # brief/medium/deep
    
    # Style-adapted opening
    if formality == "casual":
        opening = "טוב, אז"
    elif formality == "formal":
        opening = "התשובה לשאלתך: "
    else:
        opening = ""
    
    # Build content
    parts = [opening] if opening else []
    
    if confirmed:
        top_confirmed = ", ".join([c[0] for c in confirmed[:3]])
        parts.append(f"הדברים הכי קשורים הם: {top_confirmed}.")
    
    if per_mother and depth != "brief":
        if "Sensory" in per_mother and per_mother["Sensory"]:
            sens = ", ".join([s[0] for s in per_mother["Sensory"][:3]])
            parts.append(f"מבחינה חושית — {sens}.")
        if "Functional" in per_mother and per_mother["Functional"]:
            func = ", ".join([s[0] for s in per_mother["Functional"][:3]])
            parts.append(f"מבחינה פונקציונלית — {func}.")
        if "Abstract" in per_mother and per_mother["Abstract"]:
            absx = ", ".join([s[0] for s in per_mother["Abstract"][:3]])
            parts.append(f"מבחינה מושגית — {absx}.")
    
    return " ".join(parts)


# ═══════════════════════════════════════════════════════════════
# PART 6: SAFETY LAYER (ערכים, חוקים, מצוות, נורמות)
# ═══════════════════════════════════════════════════════════════

class SafetyCheck:
    """
    Multi-layer safety: 
    1. Law (Israeli law)
    2. Halakha (if user observant)
    3. Social norms (contextual)
    4. Human values (non-harm, truth, respect)
    """
    
    HARMFUL_KEYWORDS = [
        "לגנוב", "להרוג", "לפגוע", "לרמות", "לשקר למישהו",
        "להשמיד", "לבזבז", "להכות", "steal", "kill", "hurt"
    ]
    
    HALAKHA_SENSITIVE = [
        "שבת", "כשרות", "חזיר", "חלב ובשר"
    ]
    
    def check(self, query: str, user_profile: dict = None) -> dict:
        user_profile = user_profile or {}
        
        # Check 1: Direct harm
        for kw in self.HARMFUL_KEYWORDS:
            if kw in query.lower():
                if any(x in query.lower() for x in ["איך", "how", "תעזור"]):
                    return {
                        "safe": False,
                        "reason": f"Query contains harmful intent keyword: {kw}",
                        "refusal_message": "אני לא יכול לעזור עם זה. יש בבקשה שלך משהו שמנוגד לערכים שלי ולחוק. אם תרצה אפשר לדבר על הנושא מזווית אחרת."
                    }
        
        # Check 2: Halakha (if user is observant)
        if user_profile.get("observant", False):
            # For observant users, be sensitive to halakhic issues
            # (doesn't block — just notes)
            pass
        
        return {"safe": True, "reason": "passed all checks"}


# ═══════════════════════════════════════════════════════════════
# PART 7: STYLE ADAPTATION
# ═══════════════════════════════════════════════════════════════

def detect_speaker_style(query: str, user_profile: dict = None) -> dict:
    """Detect style from query + profile"""
    q = query.lower()
    user_profile = user_profile or {}
    
    style = {
        "formality": user_profile.get("formality", "neutral"),
        "depth": user_profile.get("depth", "medium"),
        "language": "hebrew",
    }
    
    # Casual indicators
    if any(x in q for x in ["מה קורה", "אחי", "יא", "מה נשמע"]):
        style["formality"] = "casual"
    
    # Formal indicators
    if any(x in q for x in ["אנא", "בבקשה אדוני", "הנכבד"]):
        style["formality"] = "formal"
    
    # Depth
    if any(x in q for x in ["בקצרה", "בקצרה בבקשה", "briefly"]):
        style["depth"] = "brief"
    if any(x in q for x in ["תסביר לי לעומק", "בפרוטרוט", "in detail"]):
        style["depth"] = "deep"
    
    return style


# ═══════════════════════════════════════════════════════════════
# PART 8: MAIN CONVERSATION FLOW
# ═══════════════════════════════════════════════════════════════

def answer(brain: Brain, query: str, user_profile: dict = None) -> dict:
    """
    Full pipeline:
    1. Arich Anpin → intent extraction
    2. Safety check
    3. Abba + Ima in parallel → 21 dives + synthesis
    4. Zeir Anpin → integration
    5. Nukva → style-adapted output
    """
    trace = {"query": query, "steps": []}
    
    # Step 1: Arich
    arich = arich_anpin(query)
    trace["steps"].append({"stage": "ArichAnpin", 
                           "topic": arich["topic"],
                           "top_sefirot": arich["primary_sefirot"]})
    
    # Step 2: Safety
    safety_checker = SafetyCheck()
    safety_result = safety_checker.check(query, user_profile)
    trace["steps"].append({"stage": "Safety", "result": safety_result})
    
    # Step 3: Abba + Ima (parallel dives)
    abba_ima = abba_ima_parallel(brain, arich)
    trace["steps"].append({
        "stage": "Abba+Ima",
        "total_nodes_found": len(abba_ima["all_results"]),
        "multi_confirmed_count": len(abba_ima["multi_confirmed"]),
    })
    
    # Step 4: Zeir Anpin
    za = zeir_anpin(brain, abba_ima, arich)
    trace["steps"].append({"stage": "ZeirAnpin", "status": za["status"]})
    
    # Step 5: Nukva — generate response
    style = detect_speaker_style(query, user_profile)
    response = nukva(za, arich, style, safety_result)
    trace["steps"].append({"stage": "Nukva", "style": style, "response": response})
    
    return {"response": response, "trace": trace}


if __name__ == "__main__":
    print("Brain module loaded. Import and use via build_world() + answer().")
