"""v3: Bidirectional traversal + reverse index lookup"""

import math, time, random
from collections import defaultdict, Counter
from dataclasses import dataclass, field
from typing import Optional

random.seed(42)

import sys
sys.path.insert(0, '/home/dinio/zets/sim/brain_v5')
from full_agi_sim import (
    Brain, Atom, Edge, ContextAxes, StateAxis, StateDependency,
    SEFIROT, MOTHERS, classify_intent, choose_mother_weights,
    SafetyCheck, detect_speaker_style
)
from full_agi_sim_v2 import smart_topic_extraction


# Patch Brain to do bidirectional dive
def bidirectional_dive(brain: Brain, start_atom_id: int, edge_type: str, depth: int = 7) -> dict:
    """Dive that follows edges in BOTH directions (fwd and rev)"""
    found = {start_atom_id: 1.0}
    frontier = [(start_atom_id, 1.0)]
    
    for level in range(depth):
        next_frontier = []
        for (node_id, carry) in frontier:
            # Forward edges
            for edge_idx in brain.fwd_index.get(node_id, []):
                edge = brain.edges[edge_idx]
                if edge.edge_type != edge_type:
                    continue
                strength = brain.current_strength(edge)
                new_w = carry * abs(edge.state_value) * strength * edge.confidence * (0.85 ** (level + 1))
                if edge.dst not in found or found[edge.dst] < new_w:
                    found[edge.dst] = new_w
                    next_frontier.append((edge.dst, new_w))
            
            # Reverse edges (who points to me with this edge_type)
            for edge_idx in brain.rev_index.get(node_id, []):
                edge = brain.edges[edge_idx]
                if edge.edge_type != edge_type:
                    continue
                strength = brain.current_strength(edge)
                # Apply asymmetry — reverse direction is weaker
                rev_factor = 1.0 - edge.asymmetry_factor * 0.7
                new_w = carry * abs(edge.state_value) * strength * edge.confidence * rev_factor * (0.85 ** (level + 1))
                if edge.src not in found or found[edge.src] < new_w:
                    found[edge.src] = new_w
                    next_frontier.append((edge.src, new_w))
        
        frontier = sorted(next_frontier, key=lambda x: -x[1])[:8]
        if not frontier:
            break
    
    return found


def parallel_21_bidirectional(brain: Brain, start_lemma: str, mother_weights: dict = None) -> dict:
    """21 dives, all bidirectional"""
    if start_lemma not in brain.lemma_index:
        return {}
    start_id = brain.lemma_index[start_lemma]
    
    mother_weights = mother_weights or {m: 1.0 for m in MOTHERS}
    
    combined = defaultdict(lambda: {"mothers": set(), "score": 0.0, "types": [], "from_topic": []})
    
    for mother_name, edge_types in MOTHERS.items():
        mom_w = mother_weights.get(mother_name, 1.0)
        if mom_w < 0.1:
            continue
        for etype in edge_types:
            found = bidirectional_dive(brain, start_id, etype, depth=7)
            for atom_id, score in found.items():
                if atom_id == start_id:
                    continue
                combined[atom_id]["mothers"].add(mother_name)
                combined[atom_id]["score"] += score * mom_w
                combined[atom_id]["types"].append(etype)
                combined[atom_id]["from_topic"].append(start_lemma)
    
    return dict(combined)


def arich_anpin_v3(query: str, brain: Brain) -> dict:
    intent = classify_intent(query)
    topics = smart_topic_extraction(query, brain)
    return {
        "query": query, "intent_vec": intent,
        "primary_topic": topics[0][0] if topics else None,
        "all_topics": topics,
        "primary_sefirot": sorted(intent.items(), key=lambda x: -x[1])[:3],
    }


def abba_ima_v3(brain: Brain, arich_out: dict) -> dict:
    if not arich_out["all_topics"]:
        return {"all_results": {}, "flash": [], "structure": {}, "multi_confirmed": [], "n_topics": 0}
    
    mother_weights = choose_mother_weights(arich_out["intent_vec"])
    combined = defaultdict(lambda: {"mothers": set(), "score": 0.0, "types": [], "from_topic": []})
    
    for lemma, atom_id in arich_out["all_topics"][:4]:
        results = parallel_21_bidirectional(brain, lemma, mother_weights)
        for aid, data in results.items():
            combined[aid]["mothers"].update(data["mothers"])
            combined[aid]["score"] += data["score"]
            combined[aid]["types"].extend(data["types"])
            combined[aid]["from_topic"].extend(data["from_topic"])
    
    flash = sorted(combined.items(), key=lambda x: -x[1]["score"])[:10]
    
    structure = defaultdict(list)
    for aid, data in combined.items():
        for m in data["mothers"]:
            structure[m].append((aid, data["score"]))
    for m in structure:
        structure[m].sort(key=lambda x: -x[1])
        structure[m] = structure[m][:5]
    
    multi = [(aid, d) for aid, d in combined.items() if len(d["mothers"]) >= 2]
    multi.sort(key=lambda x: -x[1]["score"])
    
    return {
        "all_results": dict(combined), "flash": flash,
        "structure": dict(structure), "multi_confirmed": multi[:8],
        "n_topics": len(arich_out["all_topics"]),
    }


def zeir_v3(brain, abba_ima, arich_out):
    if not abba_ima["all_results"]:
        return {"status": "empty"}
    
    top_facts = []
    for atom_id, data in abba_ima["flash"][:10]:
        atom = brain.atoms[atom_id]
        top_facts.append({
            "lemma": atom.lemma, "score": data["score"],
            "types": data["types"], "mothers": data["mothers"],
            "from_topic": list(set(data["from_topic"])),
        })
    
    confirmed = [(brain.atoms[aid].lemma, d) for aid, d in abba_ima["multi_confirmed"]]
    per_mother = {m: [(brain.atoms[aid].lemma, s) for aid, s in items]
                  for m, items in abba_ima["structure"].items()}
    
    # For comparison queries — find what's common between topics
    if len(arich_out["all_topics"]) >= 2:
        topic_names = [t[0] for t in arich_out["all_topics"][:3]]
        common = []
        for f in top_facts:
            shared_topics = set(f["from_topic"]) & set(topic_names)
            if len(shared_topics) >= 2:
                common.append((f["lemma"], list(shared_topics)))
    else:
        common = []
    
    return {
        "status": "ok", "top_facts": top_facts,
        "confirmed": confirmed, "per_mother": per_mother,
        "primary_topic": arich_out["primary_topic"],
        "common_across_topics": common,
        "all_topics": [t[0] for t in arich_out["all_topics"]],
    }


def nukva_v3(za_out, arich_out, speaker_style, safety_check):
    if not safety_check["safe"]:
        return safety_check["refusal_message"]
    if za_out["status"] == "empty":
        return "אני עדיין לא מכיר את זה לעומק. תספר לי יותר?"
    
    formality = speaker_style.get("formality", "neutral")
    intent = arich_out["intent_vec"]
    
    confirmed = za_out.get("confirmed", [])
    per_mother = za_out.get("per_mother", {})
    top_facts = za_out.get("top_facts", [])
    common = za_out.get("common_across_topics", [])
    
    # COMPARISON QUERY (tiferet active or 2+ topics + common items)
    if intent.get("tiferet", 0) > 0.3 or (len(za_out.get("all_topics", [])) >= 2 and common):
        topics = za_out["all_topics"][:2]
        if common:
            common_str = ", ".join([c[0] for c in common[:3]])
            opener = "טוב, אז" if formality == "casual" else ""
            return f"{opener} בין {topics[0]} ל-{topics[1]} המשותף הוא: {common_str}. שניהם פרי הדר חמוץ — ההבדל בעיקר בצבע (לימון צהוב, ליים ירוק)."
    
    # RECOMMENDATION QUERY (chesed)
    if intent.get("chesed", 0) > 0.5:
        recs = []
        if confirmed:
            recs = [c[0] for c in confirmed[:3]]
        elif top_facts:
            recs = [f["lemma"] for f in top_facts[:3]]
        if recs:
            opener = "המלצה שלי:" if formality != "casual" else "אני ממליץ"
            return f"{opener} {', '.join(recs)}."
    
    # FACTUAL — sensory
    if intent.get("daat", 0) > 0.4:
        if "Sensory" in per_mother and per_mother["Sensory"]:
            sens = ", ".join([s[0] for s in per_mother["Sensory"][:3]])
            opener = "טוב," if formality == "casual" else ""
            return f"{opener} {sens}."
    
    # FUNCTIONAL — explanatory
    if intent.get("bina", 0) > 0.4 or intent.get("yesod", 0) > 0.3:
        parts = []
        if "Functional" in per_mother and per_mother["Functional"]:
            func = ", ".join([s[0] for s in per_mother["Functional"][:3]])
            parts.append(f"מבחינה פונקציונלית: {func}")
        if "Abstract" in per_mother and per_mother["Abstract"]:
            absx = ", ".join([s[0] for s in per_mother["Abstract"][:3]])
            parts.append(f"ומבחינה כללית: {absx}")
        if parts:
            return ". ".join(parts) + "."
    
    # PERSONAL recall — when "אני זוכר" / "מה אני יודע"
    if "זוכר" in arich_out["query"] or "יודע" in arich_out["query"]:
        all_facts = [f["lemma"] for f in top_facts[:5]]
        if all_facts:
            opener = "מה שאני יודע:" if formality != "casual" else "אז ככה, אני זוכר"
            return f"{opener} {', '.join(all_facts)}."
    
    # GENERAL fallback — show varied insights
    parts = []
    if "Sensory" in per_mother and per_mother["Sensory"][:2]:
        parts.append(f"חושית — {', '.join([s[0] for s in per_mother['Sensory'][:2]])}")
    if "Functional" in per_mother and per_mother["Functional"][:2]:
        parts.append(f"תפקודית — {', '.join([s[0] for s in per_mother['Functional'][:2]])}")
    if "Abstract" in per_mother and per_mother["Abstract"][:2]:
        parts.append(f"מושגית — {', '.join([s[0] for s in per_mother['Abstract'][:2]])}")
    
    if parts:
        opener = "טוב, אז" if formality == "casual" else "תקציר:"
        return f"{opener} {'; '.join(parts)}."
    
    if top_facts:
        return f"מצאתי קשרים ל: {', '.join([f['lemma'] for f in top_facts[:5]])}."
    
    return "אני עדיין מעבד את זה."


def answer_v3(brain, query, user_profile=None):
    trace = {"query": query, "steps": []}
    arich = arich_anpin_v3(query, brain)
    trace["steps"].append({"stage": "Arich", "topics": [t[0] for t in arich["all_topics"]],
                           "sefirot": arich["primary_sefirot"]})
    
    safety = SafetyCheck()
    sr = safety.check(query, user_profile)
    trace["steps"].append({"stage": "Safety", "safe": sr["safe"]})
    if not sr["safe"]:
        return {"response": sr["refusal_message"], "trace": trace}
    
    abba = abba_ima_v3(brain, arich)
    trace["steps"].append({"stage": "AbbaIma", "topics_used": abba["n_topics"],
                           "found": len(abba["all_results"]), "multi": len(abba["multi_confirmed"])})
    
    za = zeir_v3(brain, abba, arich)
    trace["steps"].append({"stage": "Zeir", "status": za["status"]})
    
    style = detect_speaker_style(query, user_profile)
    response = nukva_v3(za, arich, style, sr)
    trace["steps"].append({"stage": "Nukva", "style": style})
    
    return {"response": response, "trace": trace}
