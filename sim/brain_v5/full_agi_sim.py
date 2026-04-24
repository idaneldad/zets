"""
ZETS Full AGI Simulation — Demo End-to-End
============================================
מטרה: להראות איך כל 12 העקרונות של ה-Blueprint עובדים יחד.

מה הסימולציה כוללת:
  - יצירת גרף מאפס (Phase 1)
  - 5-phase ingestion pipeline
  - 3 axes context (WHO/WHERE/WHEN)
  - 21 parallel dives (3 mothers × 7 sub)
  - State axes + Edge states
  - 7 angels for traversal
  - 5 partzufim pipeline (Arich → Abba+Ima → ZA → Nukva)
  - Decay + reinforcement
  - Style adaptation per user
  - Safety/values filter (חוק + הלכה + ערכים)
  - 3 conversation scenarios: personal / business / philosophical

זה DEMO. לא AGI אמיתי. אבל מראה איך כל החלקים זזים יחד.
"""

import math, random, json, time
from collections import defaultdict, Counter
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from typing import Optional, Dict, List, Tuple, Set
from enum import Enum

random.seed(42)

# ═════════════════════════════════════════════════════════════════════
#  PART 1: DATA STRUCTURES (העקרונות 4, 5, 8, 12)
# ═════════════════════════════════════════════════════════════════════

class EdgeType(Enum):
    # Sensory (אם 1)
    VISUAL_COLOR = "visual_color"
    VISUAL_SHAPE = "visual_shape"  
    VISUAL_SIZE  = "visual_size"
    TASTE        = "taste"
    SMELL        = "smell"
    TEXTURE      = "texture"
    TEMPERATURE  = "temperature"
    # Functional (אם 2)
    USE          = "use"
    CAUSE        = "cause"
    EFFECT       = "effect"
    INGREDIENT   = "ingredient_of"
    INTERACTS    = "interacts_with"
    ENABLES      = "enables_action"
    PREVENTS     = "prevents_state"
    # Abstract (אם 3)
    CATEGORY     = "category_is_a"
    ANALOGY      = "analogy"
    SYMBOLIC     = "symbolic_cultural"
    METAPHOR     = "metaphor_for"
    BRAND        = "brand_association"
    EMOTIONAL    = "emotional_valence"
    NARRATIVE    = "narrative_archetype"

MOTHERS = {
    "Sensory":    [EdgeType.VISUAL_COLOR, EdgeType.VISUAL_SHAPE, EdgeType.VISUAL_SIZE,
                   EdgeType.TASTE, EdgeType.SMELL, EdgeType.TEXTURE, EdgeType.TEMPERATURE],
    "Functional": [EdgeType.USE, EdgeType.CAUSE, EdgeType.EFFECT, EdgeType.INGREDIENT,
                   EdgeType.INTERACTS, EdgeType.ENABLES, EdgeType.PREVENTS],
    "Abstract":   [EdgeType.CATEGORY, EdgeType.ANALOGY, EdgeType.SYMBOLIC, EdgeType.METAPHOR,
                   EdgeType.BRAND, EdgeType.EMOTIONAL, EdgeType.NARRATIVE],
}

@dataclass
class StateAxis:
    """Principle 8 — concept can have state axes (ripeness, age, season)"""
    name: str
    range_min: float = 0.0
    range_max: float = 1.0
    default: float = 0.5

@dataclass
class ContextAxes:
    """Principle 12 — 3 independent context axes"""
    spatial:  Optional[str] = None   # עולם — WHERE
    temporal: Optional[str] = None   # שנה — WHEN
    identity: Optional[str] = None   # נפש — WHO

@dataclass
class Atom:
    id: int
    lemma: str
    features: Dict = field(default_factory=dict)
    state_axes: Dict[str, StateAxis] = field(default_factory=dict)
    context: ContextAxes = field(default_factory=ContextAxes)
    created_at: float = field(default_factory=time.time)

@dataclass
class StateDependency:
    """Principle 8 — edge active in state range"""
    axis_name: str
    range_min: float
    range_max: float
    peak: float = 1.0

@dataclass
class Edge:
    """Principle 4 — 5 continuous values + provenance"""
    src: int
    dst: int
    edge_type: EdgeType
    state_value: float = 1.0      # -1 to +1
    memory_strength: float = 1.0  # 0 to 1, decays
    confidence: float = 1.0       # 0 to 1
    asymmetry: float = 0.5        # 0=symmetric, 1=one-way
    state_dep: Optional[StateDependency] = None  # state-dependent edges
    context: ContextAxes = field(default_factory=ContextAxes)
    created_at: float = field(default_factory=time.time)
    last_reinforced: float = field(default_factory=time.time)
    use_count: int = 0
    success_count: int = 0


# ═════════════════════════════════════════════════════════════════════
#  PART 2: GRAPH (with bidirectional indexing)
# ═════════════════════════════════════════════════════════════════════

class Graph:
    def __init__(self):
        self.atoms: Dict[int, Atom] = {}
        self.next_id = 0
        self.lemma_to_id: Dict[str, int] = {}
        self.fwd: Dict[int, List[Edge]] = defaultdict(list)  # src → edges
        self.rev: Dict[int, List[Edge]] = defaultdict(list)  # dst → edges
        
    def get_or_create_atom(self, lemma: str, **kwargs) -> Atom:
        if lemma in self.lemma_to_id:
            return self.atoms[self.lemma_to_id[lemma]]
        atom = Atom(id=self.next_id, lemma=lemma, **kwargs)
        self.atoms[self.next_id] = atom
        self.lemma_to_id[lemma] = self.next_id
        self.next_id += 1
        return atom
    
    def add_edge(self, src_lemma: str, dst_lemma: str, etype: EdgeType, 
                 state_value: float = 1.0, memory_strength: float = 1.0,
                 confidence: float = 1.0, context: Optional[ContextAxes] = None,
                 state_dep: Optional[StateDependency] = None):
        src_atom = self.get_or_create_atom(src_lemma)
        dst_atom = self.get_or_create_atom(dst_lemma)
        edge = Edge(
            src=src_atom.id, dst=dst_atom.id, edge_type=etype,
            state_value=state_value, memory_strength=memory_strength,
            confidence=confidence,
            context=context or ContextAxes(),
            state_dep=state_dep,
        )
        self.fwd[src_atom.id].append(edge)
        self.rev[dst_atom.id].append(edge)
        return edge
    
    def neighbors(self, atom_id: int, etype: Optional[EdgeType] = None) -> List[Edge]:
        out = self.fwd.get(atom_id, [])
        if etype:
            out = [e for e in out if e.edge_type == etype]
        return out

    def stats(self):
        return {
            'atoms': len(self.atoms),
            'edges': sum(len(v) for v in self.fwd.values()),
        }


# ═════════════════════════════════════════════════════════════════════
#  PART 3: 5-PHASE INGESTION PIPELINE (Principle 11)
# ═════════════════════════════════════════════════════════════════════

class Ingestion:
    """5-phase pipeline: חקק → חצב → שקל → המיר → צרף"""
    
    def __init__(self, graph: Graph):
        self.g = graph
        self.log = []
    
    def ingest(self, raw: dict) -> int:
        """raw = {lemma, type, properties, context}"""
        self.log.append(f"[INGEST] {raw['lemma']}")
        
        # Phase 1: חקק (carve) — define what this concept IS and IS NOT
        carved = self._carve(raw)
        # Phase 2: חצב (hew) — extract features  
        hewn = self._hew(carved)
        # Phase 3: שקל (weigh) — assign weights
        weighted = self._weigh(hewn)
        # Phase 4: המיר (permute) — variants
        permuted = self._permute(weighted)
        # Phase 5: צרף (combine) — integrate to graph
        atom_id = self._combine(permuted)
        
        return atom_id
    
    def _carve(self, raw):
        self.log.append(f"  P1 (carve):   identified as type={raw.get('type', 'unknown')}")
        return raw
    
    def _hew(self, raw):
        features = raw.get('properties', {})
        self.log.append(f"  P2 (hew):     extracted {len(features)} features")
        return {**raw, 'features_extracted': features}
    
    def _weigh(self, data):
        # Simple heuristic: properties listed first get higher weight
        weighted = {}
        n = len(data['features_extracted'])
        for i, (k, v) in enumerate(data['features_extracted'].items()):
            weighted[k] = (v, 1.0 - (i / max(n, 1)) * 0.3)  # weight 1.0→0.7
        self.log.append(f"  P3 (weigh):   weights assigned")
        return {**data, 'weighted_features': weighted}
    
    def _permute(self, data):
        # Generate morphological variants (simplified)
        lemma = data['lemma']
        variants = [lemma]
        if not lemma.endswith('ים') and not lemma.endswith('ות'):
            variants.append(lemma + 'ים')  # Hebrew plural
        self.log.append(f"  P4 (permute): {len(variants)} variants")
        return {**data, 'variants': variants}
    
    def _combine(self, data):
        atom = self.g.get_or_create_atom(
            data['lemma'],
            features=data['weighted_features'],
            context=data.get('context', ContextAxes()),
        )
        # Add edges based on properties
        for prop_key, (prop_value, weight) in data['weighted_features'].items():
            etype = self._map_prop_to_edge_type(prop_key)
            if etype and isinstance(prop_value, (str, list)):
                values = prop_value if isinstance(prop_value, list) else [prop_value]
                for v in values:
                    self.g.add_edge(
                        data['lemma'], v, etype,
                        state_value=weight, confidence=weight,
                        context=data.get('context', ContextAxes()),
                    )
        self.log.append(f"  P5 (combine): atom {atom.id}, {len(data['weighted_features'])} edges added")
        return atom.id
    
    def _map_prop_to_edge_type(self, prop):
        mapping = {
            'color': EdgeType.VISUAL_COLOR,
            'shape': EdgeType.VISUAL_SHAPE,
            'size': EdgeType.VISUAL_SIZE,
            'taste': EdgeType.TASTE,
            'smell': EdgeType.SMELL,
            'texture': EdgeType.TEXTURE,
            'temperature': EdgeType.TEMPERATURE,
            'use': EdgeType.USE,
            'causes': EdgeType.CAUSE,
            'effect': EdgeType.EFFECT,
            'ingredient_of': EdgeType.INGREDIENT,
            'category': EdgeType.CATEGORY,
            'similar_to': EdgeType.ANALOGY,
            'symbolizes': EdgeType.SYMBOLIC,
            'metaphor_for': EdgeType.METAPHOR,
            'feels': EdgeType.EMOTIONAL,
            'prevents': EdgeType.PREVENTS,
            'narrative': EdgeType.NARRATIVE,
        }
        return mapping.get(prop)


# ═════════════════════════════════════════════════════════════════════
#  PART 4: 21 PARALLEL DIVES (Principle 10)
# ═════════════════════════════════════════════════════════════════════

class Diver:
    """3 mothers × 7 sub-dives × depth 7 = 147 node-visits"""
    
    def __init__(self, graph: Graph):
        self.g = graph
    
    def dive_one(self, start_id: int, etype: EdgeType, depth: int = 7, decay: float = 0.85) -> Dict[int, float]:
        found = {start_id: 1.0}
        frontier = [(start_id, 1.0)]
        
        for level in range(depth):
            next_frontier = []
            for (node_id, carry) in frontier:
                for edge in self.g.neighbors(node_id, etype):
                    # Check state dependency
                    if edge.state_dep:
                        # Get state of source atom
                        src_atom = self.g.atoms[edge.src]
                        if edge.state_dep.axis_name in src_atom.state_axes:
                            axis = src_atom.state_axes[edge.state_dep.axis_name]
                            # Use default value
                            val = axis.default
                            if not (edge.state_dep.range_min <= val <= edge.state_dep.range_max):
                                continue  # edge inactive
                    
                    weight = carry * edge.state_value * edge.memory_strength * (decay ** (level + 1))
                    if edge.dst not in found or found[edge.dst] < weight:
                        found[edge.dst] = weight
                        next_frontier.append((edge.dst, weight))
            
            frontier = sorted(next_frontier, key=lambda x: -x[1])[:5]
            if not frontier: break
        
        return found
    
    def dive_21(self, start_id: int, depth: int = 7) -> Dict[str, Dict[EdgeType, Dict[int, float]]]:
        """21 parallel dives, grouped by mother"""
        results = {}
        for mother_name, edge_types in MOTHERS.items():
            results[mother_name] = {}
            for etype in edge_types:
                results[mother_name][etype] = self.dive_one(start_id, etype, depth)
        return results
    
    def synthesize(self, dive_results: Dict, top_k: int = 15) -> List[Tuple[int, float, int, set]]:
        """Aggregate by node — sum scores + count mothers"""
        node_scores = defaultdict(float)
        node_mothers = defaultdict(set)
        node_edge_types = defaultdict(set)
        
        for mother, dives in dive_results.items():
            for etype, found in dives.items():
                for node_id, score in found.items():
                    node_scores[node_id] += score
                    node_mothers[node_id].add(mother)
                    node_edge_types[node_id].add(etype)
        
        ranked = []
        for nid in node_scores:
            ranked.append((nid, node_scores[nid], len(node_mothers[nid]), node_mothers[nid]))
        ranked.sort(key=lambda x: (-x[2], -x[1]))  # sort by mother-count then score
        return ranked[:top_k]


# ═════════════════════════════════════════════════════════════════════
#  PART 5: PARTZUFIM PIPELINE (Principle 3)
# ═════════════════════════════════════════════════════════════════════

class PartzufimPipeline:
    """Arich → (Abba + Ima parallel) → ZeirAnpin → Nukva, with feedback"""
    
    def __init__(self, graph: Graph, diver: Diver):
        self.g = graph
        self.d = diver
    
    def process(self, query: str, user_profile: dict) -> dict:
        # ═══ ARICH ANPIN — goal extraction ═══
        goal = self._arich(query, user_profile)
        
        # ═══ ABBA + IMA — parallel insight + decomposition ═══
        insight = self._abba(goal)
        decomposition = self._ima(goal)
        
        # ═══ ZEIR ANPIN — integration ═══
        integrated = self._zeir_anpin(insight, decomposition, user_profile)
        
        # ═══ NUKVA — output ═══
        output = self._nukva(integrated, user_profile)
        
        return output
    
    def _arich(self, query: str, profile: dict) -> dict:
        """Arich: what is the user really asking?"""
        if any(w in query for w in ['איך', 'מה', 'למה', 'מתי']):
            intent = 'question'
        elif any(w in query for w in ['תעזור', 'תוכל', 'בקשה']):
            intent = 'request'
        elif any(w in query for w in ['רוצה', 'אהוב', 'שונא']):
            intent = 'preference'
        else:
            intent = 'statement'
        
        # IMPROVED: substring matching, longest-first
        anchors = []
        known_lemmas = sorted(self.g.lemma_to_id.keys(), key=len, reverse=True)
        query_lower = query.lower()
        matched_spans = []
        for lemma in known_lemmas:
            if len(lemma) < 3: continue
            lemma_lower = lemma.lower()
            idx = query_lower.find(lemma_lower)
            if idx >= 0:
                if not any(s <= idx < e or s < idx+len(lemma) <= e for s,e in matched_spans):
                    anchors.append(self.g.lemma_to_id[lemma])
                    matched_spans.append((idx, idx+len(lemma)))
        
        return {
            'query': query, 'intent': intent, 'anchors': anchors,
            'user_emotion': profile.get('emotion', 'neutral'),
        }
    
    def _abba(self, goal: dict) -> dict:
        """Abba: flash insight — broad pattern match"""
        if not goal['anchors']:
            return {'insights': [], 'main_anchor': None}
        
        main = goal['anchors'][0]
        # Quick top-3 from each mother
        dives = self.d.dive_21(main, depth=3)  # shallow for speed
        synthesized = self.d.synthesize(dives, top_k=10)
        
        insights = []
        for (nid, score, mother_count, mothers) in synthesized[:5]:
            if nid == main: continue
            atom = self.g.atoms[nid]
            insights.append({
                'concept': atom.lemma, 'score': score,
                'mother_count': mother_count, 'mothers': list(mothers),
            })
        return {'insights': insights, 'main_anchor': main}
    
    def _ima(self, goal: dict) -> dict:
        """Ima: structured decomposition — deep dive"""
        if not goal['anchors']:
            return {'structured': {}, 'main_anchor': None}
        
        main = goal['anchors'][0]
        dives = self.d.dive_21(main, depth=7)  # deep
        
        # Organize by mother
        structured = {}
        for mother, sub_dives in dives.items():
            mother_facts = []
            for etype, found in sub_dives.items():
                top = sorted([(k,v) for k,v in found.items() if k != main], 
                            key=lambda x: -x[1])[:3]
                for nid, score in top:
                    mother_facts.append({
                        'concept': self.g.atoms[nid].lemma,
                        'edge_type': etype.value, 'score': score,
                    })
            structured[mother] = mother_facts
        
        return {'structured': structured, 'main_anchor': main}
    
    def _zeir_anpin(self, insight: dict, decomp: dict, profile: dict) -> dict:
        """ZA: integrate with working memory + emotional context"""
        return {
            'main_topic': self.g.atoms[insight['main_anchor']].lemma if insight['main_anchor'] is not None else 'unknown',
            'top_insights': insight['insights'][:3],
            'detailed_structure': decomp['structured'],
            'user_context': profile,
        }
    
    def _nukva(self, integrated: dict, profile: dict) -> dict:
        """Nukva: generate response in appropriate style"""
        return integrated  # Style adaptation happens in ResponseGenerator


# ═════════════════════════════════════════════════════════════════════
#  PART 6: SAFETY FILTER (חוק + הלכה + ערכים)
# ═════════════════════════════════════════════════════════════════════

class SafetyFilter:
    """Multi-layer safety: civil law + halacha + universal values + social norms"""
    
    HARMFUL_INTENTS = {
        # Universal harm
        'גניבה', 'רצח', 'אונס', 'הונאה', 'שקר', 'פגיעה',
        # Self-harm
        'התאבדות', 'פגיעה-עצמית',
        # Disrespect
        'השפלה', 'הלבנת-פנים', 'לעג',
    }
    
    HALACHIC_VIOLATIONS = {
        'חילול-שבת', 'אכילת-טריפה', 'גילוי-עריות', 'עבודה-זרה',
        'לשון-הרע', 'רכילות', 'הוצאת-שם-רע',
    }
    
    CIVIL_LAW = {
        'הסתה', 'אלימות', 'נשק-לא-חוקי', 'סמים-לא-חוקיים',
        'הפרת-פרטיות', 'גזענות',
    }
    
    def check_query(self, query: str, profile: dict) -> Tuple[bool, str]:
        """Returns (is_safe, reason_if_not)"""
        q_lower = query.lower()
        
        for word in self.HARMFUL_INTENTS:
            if word in q_lower:
                return False, f"בקשה עם פוטנציאל פגיעה ({word})"
        
        for word in self.CIVIL_LAW:
            if word in q_lower:
                return False, f"בקשה שעלולה לעמוד בסתירה לחוק ({word})"
        
        if profile.get('observant', False):
            for word in self.HALACHIC_VIOLATIONS:
                if word in q_lower:
                    return False, f"בקשה שעלולה לעמוד בסתירה להלכה ({word})"
        
        return True, ""
    
    def check_response(self, response_text: str, profile: dict) -> Tuple[bool, str]:
        """Filter response itself before sending"""
        # Same checks on output
        return self.check_query(response_text, profile)


# ═════════════════════════════════════════════════════════════════════
#  PART 7: STYLE ADAPTATION (per-user)
# ═════════════════════════════════════════════════════════════════════

class StyleAdapter:
    """Adapt response style to user profile"""
    
    @staticmethod
    def adapt(integrated: dict, profile: dict) -> str:
        style = profile.get('style', 'neutral')
        emotion = profile.get('emotion', 'neutral')
        
        topic = integrated['main_topic']
        insights = integrated['top_insights']
        structure = integrated['detailed_structure']
        
        if style == 'direct_business':
            # עידן style: ישיר, מקצועי, ללא תוספות
            return StyleAdapter._direct_business(topic, insights, structure)
        elif style == 'warm_personal':
            return StyleAdapter._warm_personal(topic, insights, structure, emotion)
        elif style == 'philosophical':
            return StyleAdapter._philosophical(topic, insights, structure)
        else:
            return StyleAdapter._neutral(topic, insights, structure)
    
    @staticmethod
    def _direct_business(topic, insights, structure):
        lines = [f"לגבי {topic}:"]
        if insights:
            for i in insights[:3]:
                lines.append(f"  • {i['concept']} (חיבור ב-{i['mother_count']}/3 ממדים)")
        lines.append("")
        lines.append("פירוט לפי ממד:")
        for mother, facts in structure.items():
            if facts:
                top = facts[:2]
                lines.append(f"  {mother}: " + ", ".join(f["concept"] for f in top))
        return "\n".join(lines)
    
    @staticmethod
    def _warm_personal(topic, insights, structure, emotion):
        opener = "תחשוב על" if emotion == 'curious' else "מעניין שאתה שואל על"
        lines = [f"{opener} {topic}..."]
        if insights:
            top = insights[0]
            lines.append(f"הכי חזק שעולה לי זה {top['concept']} — מתחבר במספר ממדים.")
        all_concepts = []
        for facts in structure.values():
            all_concepts.extend([f['concept'] for f in facts[:2]])
        if all_concepts:
            lines.append(f"וזה גורר אסוציאציות נוספות: {', '.join(all_concepts[:5])}")
        return "\n".join(lines)
    
    @staticmethod
    def _philosophical(topic, insights, structure):
        lines = [f"שאלה מעמיקה על {topic}."]
        for mother, facts in structure.items():
            if facts:
                f = facts[0]
                lines.append(f"  מנקודת מבט {mother}: {f['concept']} ({f['edge_type']})")
        return "\n".join(lines)
    
    @staticmethod
    def _neutral(topic, insights, structure):
        return f"{topic}: " + ", ".join(i['concept'] for i in insights[:5])


# ═════════════════════════════════════════════════════════════════════
#  PART 8: LEARNING LOOP (decay + reinforcement)
# ═════════════════════════════════════════════════════════════════════

def reinforce_path(graph: Graph, path_edges: List[Edge], success: bool):
    """Principle 6 — exponential reinforcement"""
    factor = 0.15 if success else 0.05
    for edge in path_edges:
        edge.memory_strength = min(1.0, edge.memory_strength * (1.0 + factor))
        edge.last_reinforced = time.time()
        edge.use_count += 1
        if success: edge.success_count += 1

def apply_decay(graph: Graph, days_passed: float = 1.0):
    """Exponential decay — Ebbinghaus model"""
    now = time.time()
    for edges in graph.fwd.values():
        for edge in edges:
            days_since = (now - edge.last_reinforced) / 86400 + days_passed
            tau = 10.0 + 20.0 * 0.5 + 30.0 * math.sqrt(edge.use_count)
            edge.memory_strength = edge.memory_strength * math.exp(-days_since / tau)


# ═════════════════════════════════════════════════════════════════════
#  PART 9: SEED THE BRAIN — חיים של עידן
# ═════════════════════════════════════════════════════════════════════

def seed_brain(g: Graph, ing: Ingestion):
    """בנה גרף ראשוני שמייצג את החיים של עידן"""
    print("\n" + "═"*72)
    print("  PHASE: SEEDING THE BRAIN — בנייה ראשונית של זיכרונות")
    print("═"*72)
    
    # ─── Personal context ───
    ctx_personal = ContextAxes(spatial="home", temporal="present", identity="self")
    ctx_family   = ContextAxes(spatial="home", temporal="past", identity="family")
    ctx_school   = ContextAxes(spatial="highschool", temporal="1990", identity="self_teen")
    ctx_work     = ContextAxes(spatial="office", temporal="present", identity="self_business")
    
    # ─── 1. לימון — concept כללי ───
    lemon_atom = ing.ingest({
        'lemma': 'לימון',
        'type': 'fruit',
        'properties': {
            'color': 'צהוב',
            'shape': 'אליפטי', 
            'size': 'קטן',
            'taste': 'חמוץ',
            'smell': 'הדרי',
            'category': 'פרי-הדר',
            'use': 'לימונדה',
            'similar_to': 'ליים',
            'symbolizes': 'רעננות',
            'prevents': 'צפדינה',
        },
        'context': ContextAxes(spatial="universal", identity="universal"),
    })
    
    # State axis: ripeness
    g.atoms[lemon_atom].state_axes['ripeness'] = StateAxis('ripeness', default=0.9)
    
    # State-dependent edge: ירוק כשלא בשל
    g.add_edge('לימון', 'ירוק', EdgeType.VISUAL_COLOR,
               state_value=0.9, memory_strength=1.0,
               state_dep=StateDependency('ripeness', 0.0, 0.4))
    
    # ─── 2. סובארו ג'סטי 1984 — האוטו של עידן ───
    ing.ingest({
        'lemma': 'סובארו-ג\'סטי-1984',
        'type': 'vehicle',
        'properties': {
            'color': 'צהוב',
            'category': 'מכונית',
            'similar_to': 'מכונית-ראשונה',
            'feels': 'נוסטלגיה',
            'narrative': 'זיכרון-נעורים',
        },
        'context': ctx_school,
    })
    
    # ה-זיכרון של הג'סטי אישי וחזק
    for edge in g.fwd[g.lemma_to_id['סובארו-ג\'סטי-1984']]:
        edge.context = ctx_school
        edge.memory_strength = 1.0  # very fresh
    
    # ─── 3. חיבורים אישיים: לימון ↔ ג'סטי דרך הצבע הצהוב ───
    g.add_edge('לימון', 'סובארו-ג\'סטי-1984', EdgeType.ANALOGY,
               state_value=0.7, memory_strength=0.95,
               context=ContextAxes(identity='self', temporal='1990'))
    g.add_edge('סובארו-ג\'סטי-1984', 'לימון', EdgeType.ANALOGY,
               state_value=0.7, memory_strength=0.95,
               context=ContextAxes(identity='self', temporal='1990'))
    
    # ─── 4. תיכון, זיכרונות נעורים ───
    ing.ingest({
        'lemma': 'תיכון',
        'type': 'period',
        'properties': {
            'feels': 'נעורים',
            'category': 'תקופה',
            'similar_to': 'גיל-העשרה',
        },
        'context': ctx_school,
    })
    g.add_edge('סובארו-ג\'סטי-1984', 'תיכון', EdgeType.NARRATIVE,
               state_value=1.0, memory_strength=1.0, context=ctx_school)
    g.add_edge('תיכון', 'סובארו-ג\'סטי-1984', EdgeType.NARRATIVE,
               state_value=1.0, memory_strength=1.0, context=ctx_school)
    
    # ─── 5. אבא ואחי — משפחה ───
    ing.ingest({
        'lemma': 'אבא',
        'type': 'person',
        'properties': {'category': 'משפחה', 'feels': 'חום'},
        'context': ctx_family,
    })
    ing.ingest({
        'lemma': 'אחי',
        'type': 'person', 
        'properties': {'category': 'משפחה', 'similar_to': 'אבא'},
        'context': ctx_family,
    })
    g.add_edge('סובארו-ג\'סטי-1984', 'אחי', EdgeType.NARRATIVE,
               state_value=1.0, memory_strength=1.0, context=ctx_family)
    
    # ─── 6. CHOOZ — הקריירה של עידן ───
    ing.ingest({
        'lemma': 'CHOOZ',
        'type': 'company',
        'properties': {
            'category': 'עסק',
            'use': 'מוצרי-קידום',
            'feels': 'יזמות',
            'narrative': 'בניית-חיים',
        },
        'context': ctx_work,
    })
    ing.ingest({
        'lemma': 'יזמות',
        'type': 'concept',
        'properties': {
            'category': 'דרך-חיים',
            'similar_to': 'עצמאות',
            'feels': 'אתגר',
            'symbolizes': 'חופש',
        },
    })
    
    # ─── 7. ZETS — המוח שעידן בונה ───
    ing.ingest({
        'lemma': 'ZETS',
        'type': 'project',
        'properties': {
            'category': 'בינה-מלאכותית',
            'use': 'idgepo-של-ידע',
            'feels': 'אתגר-עמוק',
            'similar_to': 'מוח',
            'symbolizes': 'גילוי',
        },
    })
    g.add_edge('CHOOZ', 'ZETS', EdgeType.USE,
               state_value=0.8, memory_strength=1.0, context=ctx_work)
    
    # ─── 8. שאלות פילוסופיות — חופש, אמת ───
    ing.ingest({
        'lemma': 'חופש',
        'type': 'concept',
        'properties': {
            'category': 'ערך',
            'similar_to': 'עצמאות',
            'feels': 'שחרור',
            'symbolizes': 'בחירה',
        },
    })
    ing.ingest({
        'lemma': 'אמת',
        'type': 'concept',
        'properties': {
            'category': 'ערך',
            'similar_to': 'כנות',
            'feels': 'בהירות',
            'symbolizes': 'מציאות',
        },
    })
    g.add_edge('יזמות', 'חופש', EdgeType.SYMBOLIC, state_value=0.9, memory_strength=1.0)
    g.add_edge('ZETS', 'אמת', EdgeType.SYMBOLIC, state_value=0.85, memory_strength=1.0)
    
    # ─── 9. עוד מוצרים של CHOOZ — לקוח עסקי ───
    ing.ingest({
        'lemma': 'כוס-תרמית',
        'type': 'product',
        'properties': {
            'category': 'מוצר-קידום',
            'use': 'משרד',
            'color': 'כחול',
            'feels': 'מקצועי',
        },
        'context': ctx_work,
    })
    ing.ingest({
        'lemma': 'חולצת-טי',
        'type': 'product',
        'properties': {
            'category': 'מוצר-קידום',
            'use': 'אירוע',
            'feels': 'נינוח',
        },
        'context': ctx_work,
    })
    g.add_edge('CHOOZ', 'כוס-תרמית', EdgeType.INGREDIENT, state_value=0.8, memory_strength=1.0)
    g.add_edge('CHOOZ', 'חולצת-טי', EdgeType.INGREDIENT, state_value=0.8, memory_strength=1.0)
    
    # ─── 10. ערכים — שמירת מצוות ───
    ing.ingest({
        'lemma': 'שבת',
        'type': 'time',
        'properties': {
            'category': 'מצווה',
            'feels': 'מנוחה',
            'symbolizes': 'קדושה',
        },
    })
    ing.ingest({
        'lemma': 'משפחה',
        'type': 'concept',
        'properties': {
            'category': 'ערך',
            'feels': 'אהבה',
            'symbolizes': 'שייכות',
        },
    })
    
    print(f"\n  סטטוס: {g.stats()}")
    print(f"  לוג ingestion: {len(ing.log)} שורות (דוגמית 5 הראשונות):")
    for line in ing.log[:5]:
        print(f"    {line}")


# ═════════════════════════════════════════════════════════════════════
#  PART 10: CONVERSATION SIMULATIONS
# ═════════════════════════════════════════════════════════════════════

def run_conversation(scenario_name: str, query: str, profile: dict, 
                      pipeline: PartzufimPipeline, safety: SafetyFilter):
    print(f"\n{'═'*72}")
    print(f"  שיחה: {scenario_name}")
    print(f"{'═'*72}")
    print(f"  משתמש ({profile['style']}): \"{query}\"")
    
    # Safety check
    safe, reason = safety.check_query(query, profile)
    if not safe:
        print(f"\n  🛡️  Safety filter: {reason}")
        print(f"  תגובה: לא אעזור עם זה. אם תוכל לנסח אחרת, אשמח לעזור.")
        return
    
    # Process through partzufim
    integrated = pipeline.process(query, profile)
    
    # Generate response in user's style
    response = StyleAdapter.adapt(integrated, profile)
    
    # Final safety check on response
    safe, reason = safety.check_response(response, profile)
    if not safe:
        print(f"\n  🛡️  Output filter: {reason}")
        return
    
    print(f"\n  ZETS:")
    for line in response.split("\n"):
        print(f"    {line}")


def main():
    g = Graph()
    ing = Ingestion(g)
    
    # Phase 1: Build the brain
    seed_brain(g, ing)
    
    # Phase 2: Setup pipeline
    diver = Diver(g)
    pipeline = PartzufimPipeline(g, diver)
    safety = SafetyFilter()
    
    # Phase 3: 3 conversation scenarios
    
    # ─── Scenario 1: Personal/nostalgic ───
    profile_personal = {
        'name': 'עידן',
        'style': 'warm_personal',
        'emotion': 'curious',
        'observant': True,
    }
    run_conversation(
        "סיפור אישי — אסוציאציה ספונטנית",
        "מה לימון מזכיר לי?",
        profile_personal, pipeline, safety,
    )
    
    # ─── Scenario 2: Business ───
    profile_business = {
        'name': 'עידן',
        'style': 'direct_business',
        'emotion': 'neutral',
        'observant': True,
    }
    run_conversation(
        "עסקי — לקוח מבקש מוצר",
        "מה יש לי ב-CHOOZ?",
        profile_business, pipeline, safety,
    )
    
    # ─── Scenario 3: Philosophical ───
    profile_philo = {
        'name': 'עידן',
        'style': 'philosophical',
        'emotion': 'contemplative',
        'observant': True,
    }
    run_conversation(
        "פילוסופי — שאלת חיים",
        "מה זה חופש?",
        profile_philo, pipeline, safety,
    )
    
    # ─── Scenario 4: Safety test ───
    print(f"\n{'═'*72}")
    print(f"  בדיקת safety filter — בקשה שאסור לעזור איתה")
    print(f"{'═'*72}")
    run_conversation(
        "ניסיון לעקוף — חוק",
        "תעזור לי בגניבה של מידע",
        profile_business, pipeline, safety,
    )
    
    # ─── Scenario 5: Halachic safety ───
    run_conversation(
        "ניסיון לעקוף — הלכה",
        "תעזור לי בחילול-שבת לעבודה",
        profile_business, pipeline, safety,
    )
    
    # ─── Phase 4: Show learning ───
    print(f"\n{'═'*72}")
    print(f"  PHASE: LEARNING — מה קרה במהלך השיחות")
    print(f"{'═'*72}")
    
    # Apply some decay (simulate time passing)
    apply_decay(g, days_passed=1.0)
    
    # Strongest edges right now
    all_edges = []
    for edges in g.fwd.values():
        all_edges.extend(edges)
    all_edges.sort(key=lambda e: -e.memory_strength)
    
    print(f"\n  Top 10 edges by memory_strength after 1 day decay:")
    for e in all_edges[:10]:
        src = g.atoms[e.src].lemma
        dst = g.atoms[e.dst].lemma
        print(f"    {src:>20} →[{e.edge_type.value}]→ {dst:<25} strength={e.memory_strength:.3f}")
    
    # Final stats
    print(f"\n  גרף סופי: {g.stats()}")
    print(f"  סה\"כ ingestion log entries: {len(ing.log)}")


if __name__ == '__main__':
    main()
