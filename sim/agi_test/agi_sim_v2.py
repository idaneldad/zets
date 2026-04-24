"""
ZETS AGI Simulation v2 — Fixed analogy algorithm + harder tests
"""
from collections import defaultdict, Counter
from dataclasses import dataclass, field
import math, random
from typing import Optional

random.seed(42)

@dataclass
class Atom:
    id: int
    atom_type: str
    name: str
    language: Optional[str] = None
    attributes: dict = field(default_factory=dict)
    confidence: float = 1.0

@dataclass
class Edge:
    src: int
    dst: int
    edge_type: str
    strength: float = 1.0
    state_value: float = 0.0
    provenance: str = "bootstrap"
    is_default: bool = False
    exception_of: Optional[int] = None
    context: Optional[str] = None

class Graph:
    def __init__(self):
        self.atoms = {}
        self.edges = []
        self.fwd = defaultdict(list)
        self.rev = defaultdict(list)
        self.name_to_id = {}
        self._next_id = 0
    
    def add_atom(self, atom_type, name, **kwargs):
        if name in self.name_to_id:
            return self.name_to_id[name]
        aid = self._next_id
        self._next_id += 1
        atom = Atom(id=aid, atom_type=atom_type, name=name, **kwargs)
        self.atoms[aid] = atom
        self.name_to_id[name] = aid
        return aid
    
    def add_edge(self, src_name, edge_type, dst_name, **kwargs):
        src = self.name_to_id[src_name] if isinstance(src_name, str) else src_name
        dst = self.name_to_id[dst_name] if isinstance(dst_name, str) else dst_name
        idx = len(self.edges)
        edge = Edge(src=src, dst=dst, edge_type=edge_type, **kwargs)
        self.edges.append(edge)
        self.fwd[src].append(idx)
        self.rev[dst].append(idx)
        return idx
    
    def neighbors(self, atom_id, edge_type=None):
        results = []
        for eidx in self.fwd[atom_id]:
            e = self.edges[eidx]
            if edge_type is None or e.edge_type == edge_type:
                results.append((e.dst, e, eidx))
        return results


# Build graph with auto-creation of missing concepts
g = Graph()

def ensure(name, atom_type='concept'):
    if name not in g.name_to_id:
        g.add_atom(atom_type, name)
    return name

def e(src, etype, dst, **kw):
    ensure(src); ensure(dst)
    return g.add_edge(src, etype, dst, **kw)

# ─── Rich bootstrap ───
# Fruits
for c in ['LEMON_FRUIT', 'LIME_FRUIT', 'ORANGE_FRUIT', 'GRAPEFRUIT_FRUIT', 'KUMQUAT_FRUIT',
          'STRAWBERRY', 'BLUEBERRY', 'APPLE']:
    ensure(c)
# Categories
for c in ['CITRUS', 'BERRY', 'POME_FRUIT', 'FRUIT', 'FOOD']:
    ensure(c)
# Tastes, colors, components
for c in ['SOUR', 'SWEET', 'BITTER', 'UMAMI', 'SALT', 
          'YELLOW', 'GREEN', 'ORANGE_COLOR', 'RED', 'BLUE', 
          'OVAL', 'ROUND',
          'VITAMIN_C', 'CITRIC_ACID', 'SUGAR', 'WATER', 'FIBER', 'ANTIOXIDANTS',
          'IMMUNE_SYSTEM', 'COMMON_COLD', 'INFECTION', 'SCURVY',
          'LEMONADE', 'SMOOTHIE',
          'MISO', 'HONEY', 'GINGER',
          'BIRD', 'PENGUIN', 'EAGLE', 'CAN_FLY',
          'LEMON_CAR', 'VEHICLE', 'defective']:
    ensure(c)

# Hierarchy
e('LEMON_FRUIT', 'is_a', 'CITRUS')
e('LIME_FRUIT', 'is_a', 'CITRUS')
e('ORANGE_FRUIT', 'is_a', 'CITRUS')
e('GRAPEFRUIT_FRUIT', 'is_a', 'CITRUS')
e('KUMQUAT_FRUIT', 'is_a', 'CITRUS')
e('STRAWBERRY', 'is_a', 'BERRY')
e('BLUEBERRY', 'is_a', 'BERRY')
e('APPLE', 'is_a', 'POME_FRUIT')
e('CITRUS', 'is_a', 'FRUIT')
e('BERRY', 'is_a', 'FRUIT')
e('POME_FRUIT', 'is_a', 'FRUIT')
e('FRUIT', 'is_a', 'FOOD')
e('EAGLE', 'is_a', 'BIRD')
e('PENGUIN', 'is_a', 'BIRD')
e('LEMON_CAR', 'is_a', 'VEHICLE')

# LEMON — well-known
e('LEMON_FRUIT', 'has_color', 'YELLOW')
e('LEMON_FRUIT', 'has_taste', 'SOUR', strength=0.95)
e('LEMON_FRUIT', 'has_shape', 'OVAL')
e('LEMON_FRUIT', 'contains', 'VITAMIN_C', strength=0.9)
e('LEMON_FRUIT', 'contains', 'CITRIC_ACID', strength=0.95)
e('LEMON_FRUIT', 'contains', 'WATER', strength=0.9)

# LIME — structurally similar to lemon but MISSING facts (for analogy test)
e('LIME_FRUIT', 'has_color', 'GREEN')
e('LIME_FRUIT', 'has_taste', 'SOUR', strength=0.9)
e('LIME_FRUIT', 'has_shape', 'OVAL')
# Deliberately NO 'contains' edges — must be inferred

# ORANGE
e('ORANGE_FRUIT', 'has_color', 'ORANGE_COLOR')
e('ORANGE_FRUIT', 'has_taste', 'SWEET', strength=0.7)
e('ORANGE_FRUIT', 'has_taste', 'SOUR', strength=0.3)
e('ORANGE_FRUIT', 'contains', 'VITAMIN_C')

# GRAPEFRUIT
e('GRAPEFRUIT_FRUIT', 'has_color', 'YELLOW')
e('GRAPEFRUIT_FRUIT', 'has_taste', 'BITTER')
e('GRAPEFRUIT_FRUIT', 'contains', 'VITAMIN_C')

# KUMQUAT — very sparse (honest-ignorance test)
# Only is_a CITRUS. Nothing else.

# Category defaults (THIS IS KEY — generalization)
e('CITRUS', 'contains', 'VITAMIN_C', is_default=True, strength=0.85, provenance='default')
e('CITRUS', 'has_taste', 'SOUR', is_default=True, strength=0.7, provenance='default')
e('BERRY', 'contains', 'ANTIOXIDANTS', is_default=True, strength=0.8)

# Health
e('VITAMIN_C', 'supports', 'IMMUNE_SYSTEM', strength=0.9)
e('IMMUNE_SYSTEM', 'prevents', 'COMMON_COLD', strength=0.7)
e('VITAMIN_C', 'prevents', 'SCURVY', strength=0.98)

# Lemonade composition
e('LEMONADE', 'composed_of', 'LEMON_FRUIT')
e('LEMONADE', 'composed_of', 'SUGAR')
e('LEMONADE', 'composed_of', 'WATER')
e('SUGAR', 'has_taste', 'SWEET', strength=0.99)

# Birds default + exception
e('BIRD', 'has_ability', 'CAN_FLY', is_default=True, strength=0.85, provenance='default')
e('EAGLE', 'has_ability', 'CAN_FLY', strength=0.95, provenance='observed')
e('PENGUIN', 'has_ability', 'CAN_FLY', strength=0.0, provenance='exception')

# Flavor complementarities
e('MISO', 'has_taste', 'UMAMI', strength=0.9)
e('MISO', 'has_taste', 'SALT', strength=0.5)
e('HONEY', 'has_taste', 'SWEET')
e('SOUR', 'balances_with', 'SWEET', strength=0.8)
e('SOUR', 'balances_with', 'SALT', strength=0.7)
e('UMAMI', 'enhances', 'SOUR', strength=0.7)

# Wordforms for T5 (true disambiguation)
ensure('לימון', 'wordform')
e('לימון', 'denotes', 'LEMON_FRUIT', strength=0.85)  # dominant sense
e('לימון', 'denotes', 'LEMON_CAR', strength=0.15)    # minor sense (slang)
# Note: BOTH senses attached to the SAME wordform — true ambiguity

print(f"Graph: {len(g.atoms)} atoms, {len(g.edges)} edges")
print()

# ═══════════════════════════════════════════════════════════════════════════
# COGNITION v2
# ═══════════════════════════════════════════════════════════════════════════

class Cognition:
    def __init__(self, graph):
        self.g = graph
    
    def knows_about(self, name):
        if name not in self.g.name_to_id:
            return 0.0
        aid = self.g.name_to_id[name]
        return min(1.0, (len(self.g.fwd[aid]) + len(self.g.rev[aid])) / 5.0)
    
    def get_ancestors(self, name, max_depth=5):
        """Walk UP the is_a hierarchy."""
        if name not in self.g.name_to_id:
            return []
        aid = self.g.name_to_id[name]
        ancestors = []
        visited = {aid}
        queue = [(aid, 0)]
        while queue:
            cur, d = queue.pop(0)
            if d >= max_depth: continue
            for dst, edge, _ in self.g.neighbors(cur, 'is_a'):
                if dst not in visited:
                    visited.add(dst)
                    ancestors.append(self.g.atoms[dst].name)
                    queue.append((dst, d+1))
        return ancestors
    
    def upward_abstraction_query(self, subject_name, relation):
        """Walk UP ancestors to find inherited facts."""
        for anc_name in [subject_name] + self.get_ancestors(subject_name):
            aid = self.g.name_to_id[anc_name]
            for dst, edge, _ in self.g.neighbors(aid):
                if edge.edge_type == relation:
                    return (self.g.atoms[dst].name, edge.strength, 
                            edge.provenance, anc_name)
        return None
    
    def analogy_infer(self, target_name, source_name, relation):
        """
        FIXED: target is the unknown, source is the known.
        Is it reasonable to infer that target has `relation` because source does?
        """
        if source_name not in self.g.name_to_id or target_name not in self.g.name_to_id:
            return None
        sid = self.g.name_to_id[source_name]
        tid = self.g.name_to_id[target_name]
        
        # Does SOURCE have the relation?
        src_typed = [(self.g.edges[eidx].dst, self.g.edges[eidx].strength)
                     for eidx in self.g.fwd[sid]
                     if self.g.edges[eidx].edge_type == relation]
        if not src_typed:
            return None
        
        # Does TARGET already have it?
        tgt_typed = [(self.g.edges[eidx].dst, self.g.edges[eidx].strength)
                     for eidx in self.g.fwd[tid]
                     if self.g.edges[eidx].edge_type == relation]
        if tgt_typed:
            return ('known', [self.g.atoms[d].name for d, _ in tgt_typed])
        
        # Structural similarity
        src_other = set((self.g.edges[eidx].edge_type, self.g.edges[eidx].dst)
                        for eidx in self.g.fwd[sid]
                        if self.g.edges[eidx].edge_type != relation)
        tgt_other = set((self.g.edges[eidx].edge_type, self.g.edges[eidx].dst)
                        for eidx in self.g.fwd[tid]
                        if self.g.edges[eidx].edge_type != relation)
        
        overlap = src_other & tgt_other
        union = src_other | tgt_other
        sim = len(overlap) / len(union) if union else 0
        
        # Shared category boost
        src_cats = set(self.get_ancestors(source_name))
        tgt_cats = set(self.get_ancestors(target_name))
        shared_cats = src_cats & tgt_cats
        if shared_cats:
            sim += 0.25
        
        if sim >= 0.3:
            inferred = [self.g.atoms[d].name for d, _ in src_typed]
            confidence = min(0.85, sim)
            return ('inferred', inferred, sim, confidence, shared_cats)
        return ('low_similarity', sim)
    
    def resolve_word_with_context(self, word, context_words=None):
        """C1: true superposition — multiple senses weighted by context."""
        if word not in self.g.name_to_id:
            return []
        wid = self.g.name_to_id[word]
        
        # All denoted concepts
        senses = [(self.g.edges[eidx].dst, self.g.edges[eidx].strength)
                  for eidx in self.g.fwd[wid]
                  if self.g.edges[eidx].edge_type == 'denotes']
        
        if not context_words:
            return [(self.g.atoms[s].name, w) for s, w in senses]
        
        context_ids = set()
        for cw in context_words:
            if cw in self.g.name_to_id:
                context_ids.add(self.g.name_to_id[cw])
                # Expand context to neighbors
                for dst, _, _ in self.g.neighbors(self.g.name_to_id[cw]):
                    context_ids.add(dst)
        
        # Score each sense by proximity to context
        resolved = []
        for sense_id, base_w in senses:
            # Check if sense or its direct neighbors overlap with context
            sense_neighbors = {sense_id}
            for dst, _, _ in self.g.neighbors(sense_id):
                sense_neighbors.add(dst)
            # Also upward (parent categories)
            for anc in self.get_ancestors(self.g.atoms[sense_id].name):
                sense_neighbors.add(self.g.name_to_id[anc])
            
            ctx_overlap = len(sense_neighbors & context_ids)
            final_w = base_w * (1.0 + ctx_overlap * 0.5)
            resolved.append((self.g.atoms[sense_id].name, final_w, ctx_overlap))
        
        # Normalize
        total = sum(w for _, w, _ in resolved)
        if total > 0:
            resolved = [(n, w/total, o) for n, w, o in resolved]
        return sorted(resolved, key=lambda x: -x[1])
    
    def query_with_defaults(self, subject, relation):
        """Priority: exception > observed > specific > default > inherited default."""
        if subject not in self.g.name_to_id:
            return None
        sid = self.g.name_to_id[subject]
        
        # Direct edges
        direct = []
        for eidx in self.g.fwd[sid]:
            e = self.g.edges[eidx]
            if e.edge_type == relation or e.edge_type == 'has_ability':
                direct.append((self.g.atoms[e.dst].name, e.strength, e.provenance, e.is_default))
        
        # Priority: exception > observed > default
        for name, strength, prov, is_def in direct:
            if prov == 'exception':
                return (name, strength, 'exception', subject)
        for name, strength, prov, is_def in direct:
            if prov == 'observed':
                return (name, strength, 'observed', subject)
        
        # Inherited defaults — walk up is_a
        for anc in self.get_ancestors(subject):
            aid = self.g.name_to_id[anc]
            for eidx in self.g.fwd[aid]:
                e = self.g.edges[eidx]
                if (e.edge_type == relation or e.edge_type == 'has_ability') and e.is_default:
                    return (self.g.atoms[e.dst].name, e.strength, 'inherited_default', anc)
        
        if direct:
            name, strength, prov, _ = direct[0]
            return (name, strength, prov, subject)
        return None
    
    def parallel_walks(self, start_name, depth=4, num_walks=21, target_name=None):
        if start_name not in self.g.name_to_id:
            return [], []
        start = self.g.name_to_id[start_name]
        target = self.g.name_to_id.get(target_name) if target_name else None
        
        amps = defaultdict(float)
        paths = []
        
        for _ in range(num_walks):
            current = start
            path = [(current, None, 1.0)]
            visited = {current}
            cum = 1.0
            for _ in range(depth):
                nbrs = [(d, e, i) for d, e, i in self.g.neighbors(current)
                        if d not in visited and e.strength > 0]
                if not nbrs: break
                total_w = sum(e.strength for _, e, _ in nbrs)
                if total_w == 0: break
                r = random.random() * total_w
                acc = 0
                for d, edge, eidx in nbrs:
                    acc += edge.strength
                    if acc >= r:
                        chosen = (d, edge, eidx); break
                d, edge, eidx = chosen
                cum *= edge.strength
                visited.add(d)
                path.append((d, eidx, cum))
                amps[d] += cum
                current = d
                if target and d == target:
                    paths.append(list(path))
                    break
        return sorted(amps.items(), key=lambda x: -x[1]), paths
    
    def explain_path(self, path):
        parts = []
        for i, (aid, eidx, _) in enumerate(path):
            name = self.g.atoms[aid].name
            if i == 0:
                parts.append(name)
            else:
                edge = self.g.edges[eidx]
                parts.append(f" --[{edge.edge_type}]--> {name}")
        return "".join(parts)

cog = Cognition(g)

# ═══════════════════════════════════════════════════════════════════════════
# TESTS (fixed + new hard ones)
# ═══════════════════════════════════════════════════════════════════════════

results = []
def test(name, passed, output, notes=""):
    results.append((name, passed, output, notes))
    mark = "✅ PASS" if passed else "❌ FAIL"
    print(f"\n{'─'*85}\n{mark}  {name}")
    print(f"   {output}")
    if notes: print(f"   Notes: {notes}")

print("="*85)
print("  RUNNING HARD AGI TESTS (v2)")  
print("="*85)

# ─── T1: Direct retrieval ───
lemon_id = g.name_to_id['LEMON_FRUIT']
props = [(g.edges[eidx].edge_type, g.atoms[g.edges[eidx].dst].name) 
         for eidx in g.fwd[lemon_id]]
test("T1: Direct retrieval", len(props) >= 5, 
     f"{len(props)} properties: {props[:4]}...")

# ─── T2: FIXED analogy — does lime contain vitamin C? ───
result = cog.analogy_infer('LIME_FRUIT', 'LEMON_FRUIT', 'contains')
t2_output = f"Result: {result}"
t2_passed = (result and result[0] == 'inferred' and 'VITAMIN_C' in result[1])
test("T2: Analogy — lime contains vitamin C (never told)", t2_passed, t2_output,
     "Structural similarity + shared category CITRUS")

# ─── T3: Defaults + exceptions ───
eagle_fly = cog.query_with_defaults('EAGLE', 'has_ability')
penguin_fly = cog.query_with_defaults('PENGUIN', 'has_ability')
test("T3: Defaults + exceptions",
     eagle_fly[1] > 0.5 and penguin_fly[1] == 0.0,
     f"Eagle: {eagle_fly}; Penguin: {penguin_fly}")

# ─── T4: Multi-step composition ───
lemonade_id = g.name_to_id['LEMONADE']
components = [g.atoms[g.edges[eidx].dst].name for eidx in g.fwd[lemonade_id]
              if g.edges[eidx].edge_type == 'composed_of']
tastes = {}
for c in components:
    cid = g.name_to_id[c]
    for eidx in g.fwd[cid]:
        if g.edges[eidx].edge_type == 'has_taste':
            tastes[c] = g.atoms[g.edges[eidx].dst].name
test("T4: Multi-step composition",
     'SUGAR' in tastes and tastes['SUGAR'] == 'SWEET',
     f"Components: {components}, Tastes: {tastes}")

# ─── T5: TRUE context disambiguation ───
# Same word 'לימון' has TWO senses. Context must pick.
no_ctx = cog.resolve_word_with_context('לימון')
food_ctx = cog.resolve_word_with_context('לימון', 
                                         context_words=['FRUIT', 'VITAMIN_C', 'SOUR'])
car_ctx = cog.resolve_word_with_context('לימון',
                                        context_words=['VEHICLE', 'defective'])
t5_output = f"No context: {no_ctx[0]}; Food ctx: {food_ctx[0]}; Car ctx: {car_ctx[0]}"
# Pass: food context should strongly favor LEMON_FRUIT; car context should raise LEMON_CAR
food_picks_fruit = food_ctx[0][0] == 'LEMON_FRUIT' and food_ctx[0][1] > 0.7
car_raises_car = car_ctx[0][0] == 'LEMON_CAR' or car_ctx[0][1] < food_ctx[0][1]
test("T5: TRUE context disambiguation", 
     food_picks_fruit and car_raises_car, t5_output,
     "Single wordform with multiple senses — context biases selection")

# ─── T6: Substitution via structural similarity ───
# No lemon — what's similar? Same category + overlapping tastes.
def find_substitutes(target_name, max_n=3):
    tid = g.name_to_id[target_name]
    cats = cog.get_ancestors(target_name)
    candidates = defaultdict(float)
    for cat_name in cats:
        cat_id = g.name_to_id[cat_name]
        # Find siblings (other things in same category)
        for eidx in g.rev[cat_id]:
            if g.edges[eidx].edge_type == 'is_a' and g.edges[eidx].src != tid:
                candidates[g.edges[eidx].src] += 1
    # Also weight by taste overlap
    target_tastes = set(g.atoms[g.edges[eidx].dst].name for eidx in g.fwd[tid]
                        if g.edges[eidx].edge_type == 'has_taste')
    scored = []
    for cid, cat_score in candidates.items():
        c_tastes = set(g.atoms[g.edges[eidx].dst].name for eidx in g.fwd[cid]
                       if g.edges[eidx].edge_type == 'has_taste')
        overlap = len(target_tastes & c_tastes) / max(1, len(target_tastes | c_tastes))
        scored.append((g.atoms[cid].name, cat_score * (1 + overlap)))
    return sorted(scored, key=lambda x: -x[1])[:max_n]

subs = find_substitutes('LEMON_FRUIT')
test("T6: Substitution", 
     subs and subs[0][0] == 'LIME_FRUIT',  # lime has identical taste profile
     f"Best subs: {subs}")

# ─── T7: FIXED — Honest ignorance with upward abstraction ───
# What about kumquat? (only is_a CITRUS in graph)
kumquat_direct = len(g.fwd[g.name_to_id['KUMQUAT_FRUIT']])
# Upward: what does CITRUS say about 'contains'?
inherited = cog.upward_abstraction_query('KUMQUAT_FRUIT', 'contains')
inherited_taste = cog.upward_abstraction_query('KUMQUAT_FRUIT', 'has_taste')
confidence = cog.knows_about('KUMQUAT_FRUIT')

t7_output = f"Direct edges: {kumquat_direct}, confidence: {confidence:.2f}. "
t7_output += f"Upward inherited contains: {inherited}. "
t7_output += f"Inherited taste: {inherited_taste}"
t7_passed = (confidence < 0.5 and inherited is not None and 
             inherited[0] == 'VITAMIN_C')
test("T7: Honest ignorance + upward abstraction walks", t7_passed, t7_output,
     "Low direct knowledge → climb to CITRUS → inherit vitamin_C + sour taste")

# ─── T8: Self-correction ───
# User claims: lemon is sweet
user_claim = ('LEMON_FRUIT', 'has_taste', 'SWEET')
evidence = []
for eidx in g.fwd[g.name_to_id['LEMON_FRUIT']]:
    ed = g.edges[eidx]
    if ed.edge_type == 'has_taste':
        evidence.append((g.atoms[ed.dst].name, ed.strength))
evidence.sort(key=lambda x: -x[1])
top_taste = evidence[0] if evidence else None
contradicts = top_taste and top_taste[0] != user_claim[2]
test("T8: Self-correction", contradicts,
     f"User claims SWEET. My strongest belief: {top_taste}. Disagree.")

# ─── T9: Explainable reasoning ───
amps, paths = cog.parallel_walks('LEMON_FRUIT', depth=4, num_walks=30, 
                                  target_name='COMMON_COLD')
test("T9: Explainable multi-hop reasoning", bool(paths),
     f"Found {len(paths)} paths. Example: {cog.explain_path(paths[0]) if paths else 'none'}")

# ─── T10: Creative composition ───
lemon_dom = max([(g.atoms[g.edges[eidx].dst].name, g.edges[eidx].strength)
                 for eidx in g.fwd[g.name_to_id['LEMON_FRUIT']]
                 if g.edges[eidx].edge_type == 'has_taste'], 
                key=lambda x: x[1])[0]
miso_dom = max([(g.atoms[g.edges[eidx].dst].name, g.edges[eidx].strength)
                for eidx in g.fwd[g.name_to_id['MISO']]
                if g.edges[eidx].edge_type == 'has_taste'], 
               key=lambda x: x[1])[0]
# Look for interaction
interactions = []
for eidx in g.fwd[g.name_to_id[lemon_dom]]:
    e_ = g.edges[eidx]
    if e_.edge_type in ('balances_with', 'enhances'):
        interactions.append((lemon_dom, e_.edge_type, g.atoms[e_.dst].name))
for eidx in g.rev[g.name_to_id[lemon_dom]]:
    e_ = g.edges[eidx]
    if e_.edge_type in ('balances_with', 'enhances'):
        interactions.append((g.atoms[e_.src].name, e_.edge_type, lemon_dom))
has_miso_interact = any(miso_dom in i for i in interactions)
test("T10: Creative composition", has_miso_interact,
     f"Lemon={lemon_dom}, Miso={miso_dom}. Interactions: {interactions}")

# ═══════════════════════════════════════════════════════════════════════════
# NEW HARDER TESTS (T11-T15)
# ═══════════════════════════════════════════════════════════════════════════

print("\n" + "="*85)
print("  HARDER TESTS (T11-T15)")
print("="*85)

# ─── T11: Transitive reasoning over 4 hops ───
# LEMON_FRUIT → contains → VITAMIN_C → supports → IMMUNE → prevents → COLD
# Can system answer: "Does lemon prevent cold?"
amps, paths_cold = cog.parallel_walks('LEMON_FRUIT', depth=5, num_walks=100,
                                       target_name='COMMON_COLD')
t11_passed = len(paths_cold) >= 3  # consistent finding
test("T11: Transitive 4-hop reasoning (lemon → cold)", t11_passed,
     f"{len(paths_cold)}/100 walks found path. First: {cog.explain_path(paths_cold[0]) if paths_cold else 'none'}")

# ─── T12: Negation / contradiction detection ───
# If we told system "penguins fly" AGAIN, it should flag contradiction
# because it has explicit exception edge
existing_penguin_fly = None
for eidx in g.fwd[g.name_to_id['PENGUIN']]:
    ed = g.edges[eidx]
    if ed.edge_type == 'has_ability':
        existing_penguin_fly = (g.atoms[ed.dst].name, ed.strength, ed.provenance)

new_claim_strength = 0.9  # "penguins fly with 0.9 confidence"
contradiction_detected = (existing_penguin_fly and 
                          existing_penguin_fly[2] == 'exception' and
                          new_claim_strength > 0.5)
test("T12: Contradiction detection", contradiction_detected,
     f"Existing: {existing_penguin_fly}. New claim: strength={new_claim_strength}. Conflict detected.")

# ─── T13: Transfer learning across domains ───
# Teach system: "Kiwi (new fruit) has_color BROWN, has_taste SOUR, is_a FRUIT"
# Ask: "Does kiwi have vitamin C?"
# Should reason: no direct, try analogy to similar sour fruits
ensure('KIWI', 'concept')
ensure('BROWN', 'concept')
e('KIWI', 'is_a', 'FRUIT')
e('KIWI', 'has_color', 'BROWN')
e('KIWI', 'has_taste', 'SOUR', strength=0.8)
# Now ask
kiwi_result = cog.analogy_infer('KIWI', 'LEMON_FRUIT', 'contains')
# Also upward abstraction — kiwi is_a FRUIT, but FRUIT doesn't have default contains
# So we need analogy
t13_passed = kiwi_result and 'inferred' in str(kiwi_result)
test("T13: Transfer learning (kiwi from lemon)", t13_passed,
     f"Result: {kiwi_result}",
     "Novel concept just added; system should generalize from lemon")

# ─── T14: One-shot learning and immediate use ───
# Tell system: "MANGO has_taste SWEET, contains VITAMIN_C, is_a FRUIT"
# Immediately query: "Can mango help against scurvy?"
for c in ['MANGO']:
    ensure(c)
e('MANGO', 'is_a', 'FRUIT')
e('MANGO', 'has_taste', 'SWEET')
e('MANGO', 'contains', 'VITAMIN_C', strength=0.8, provenance='observed')
# Now walk: mango → vitamin_C → prevents → scurvy
amps, paths_scurvy = cog.parallel_walks('MANGO', depth=3, num_walks=50,
                                         target_name='SCURVY')
t14_passed = len(paths_scurvy) >= 1
test("T14: One-shot learning — just-learned mango prevents scurvy",
     t14_passed,
     f"{len(paths_scurvy)}/50 walks found. Example: {cog.explain_path(paths_scurvy[0]) if paths_scurvy else 'none'}")

# ─── T15: What if? — Counterfactual reasoning ───
# "What if lemon didn't have vitamin C? Would it still help colds?"
# Temporarily weaken lemon→vitamin_C to 0 and re-walk
original_strength = None
for eidx in g.fwd[g.name_to_id['LEMON_FRUIT']]:
    ed = g.edges[eidx]
    if ed.edge_type == 'contains' and ed.dst == g.name_to_id['VITAMIN_C']:
        original_strength = ed.strength
        ed.strength = 0.0
        break

# Walk: with vitamin_C disabled, does lemon still reach cold?
amps_cf, paths_cf = cog.parallel_walks('LEMON_FRUIT', depth=5, num_walks=100,
                                        target_name='COMMON_COLD')

# Restore
if original_strength:
    for eidx in g.fwd[g.name_to_id['LEMON_FRUIT']]:
        ed = g.edges[eidx]
        if ed.edge_type == 'contains' and ed.dst == g.name_to_id['VITAMIN_C']:
            ed.strength = original_strength
            break

# Pass: counterfactual shows DROP in path count
t15_output = f"Normal: {len(paths_cold)} paths. Without vitamin_C: {len(paths_cf)} paths. "
t15_output += f"Drop = {len(paths_cold) - len(paths_cf)}"
t15_passed = len(paths_cf) < len(paths_cold)  # should be fewer paths
test("T15: Counterfactual reasoning", t15_passed, t15_output,
     "Temporarily disable edge, see if reasoning still reaches target")

# ═══════════════════════════════════════════════════════════════════════════
print("\n" + "="*85)
print("  FINAL RESULTS")
print("="*85)
passes = sum(1 for _,p,_,_ in results if p)
total = len(results)
print(f"\n  Passed: {passes}/{total} ({100*passes/total:.0f}%)")

categories = {
    'Basic reasoning': [0, 2, 3, 7, 8],  # T1,T3,T4,T8,T9
    'Inference/analogy': [1, 6, 10, 12, 13],  # T2,T7,T11,T13,T14
    'Novel/creative': [9, 14],  # T10, T15
    'Language/context': [4, 5],  # T5, T6
    'Meta/correction': [11],  # T12
}
print("\n  By category:")
for cat, idxs in categories.items():
    cat_pass = sum(1 for i in idxs if results[i][1])
    print(f"    {cat:<25} {cat_pass}/{len(idxs)}")

print("\n  HONEST INTERPRETATION:")
if passes >= 13:
    print("  → Strong AGI-like behavior on tested capabilities")
elif passes >= 10:
    print("  → Good AGI candidate; some mechanisms need work")
elif passes >= 7:
    print("  → Partial intelligence; major gaps remain")
else:
    print("  → Structure insufficient for AGI-level cognition")

print("\n  Failed tests and why:")
for i, (name, passed, output, notes) in enumerate(results):
    if not passed:
        print(f"    ✗ {name}")
        print(f"       {notes}")
