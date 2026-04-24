"""
ZETS Full AGI Simulation v2 — תיקון לבעיות זיהוי topic
"""

import math, time, random
from collections import defaultdict, Counter
from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional

random.seed(42)

# Reuse most from v1 — just fix the broken parts
import sys
sys.path.insert(0, '/home/dinio/zets/sim/brain_v5')
from full_agi_sim import (
    Brain, Atom, Edge, ContextAxes, StateAxis, StateDependency,
    SEFIROT, MOTHERS, classify_intent, choose_mother_weights,
    SafetyCheck, detect_speaker_style
)


def smart_topic_extraction(query: str, brain: Brain) -> list:
    """
    Improved: try to find ALL atom-lemmas mentioned in query.
    Returns list of (lemma, atom_id), sorted by length (longest first = most specific).
    """
    # Strip Hebrew prefixes
    PREFIXES = ['ה', 'ש', 'מ', 'ל', 'ב', 'כ', 'ו']
    
    # Get all words from query
    words = query.replace("?", " ").replace("!", " ").replace(",", " ").replace("-", "-").split()
    
    candidates = []
    
    # Try matching each word + each known lemma
    for lemma, atom_id in brain.lemma_index.items():
        # Direct match
        if lemma in query:
            candidates.append((lemma, atom_id, len(lemma)))
            continue
        
        # Match without prefixes
        for word in words:
            stripped = word
            for _ in range(2):
                if stripped and stripped[0] in PREFIXES and len(stripped) > 2:
                    test = stripped[1:]
                    if test == lemma or lemma in test or test in lemma:
                        candidates.append((lemma, atom_id, len(lemma)))
                        break
                    stripped = test
            
            # Substring match
            if len(word) >= 3 and len(lemma) >= 3:
                if word in lemma or lemma in word:
                    candidates.append((lemma, atom_id, len(lemma)))
    
    # Dedup, sort by length (longest = most specific match)
    seen = set()
    unique = []
    for lemma, aid, length in sorted(candidates, key=lambda x: -x[2]):
        if aid not in seen:
            seen.add(aid)
            unique.append((lemma, aid))
    
    return unique


def arich_anpin_v2(query: str, brain: Brain) -> dict:
    """Improved Arich — finds multiple topics"""
    intent = classify_intent(query)
    
    topics = smart_topic_extraction(query, brain)
    primary_topic = topics[0][0] if topics else None
    
    return {
        "query": query,
        "intent_vec": intent,
        "primary_topic": primary_topic,
        "all_topics": topics,
        "primary_sefirot": sorted(intent.items(), key=lambda x: -x[1])[:3],
    }


def abba_ima_parallel_v2(brain: Brain, arich_out: dict) -> dict:
    """If multiple topics, run dives from each"""
    if not arich_out["all_topics"]:
        return {"all_results": {}, "flash": [], "structure": {}, "multi_confirmed": []}
    
    mother_weights = choose_mother_weights(arich_out["intent_vec"])
    
    # Run dives from all topics
    combined_results = defaultdict(lambda: {"mothers": set(), "score": 0.0, "types": [], "from_topic": []})
    
    for lemma, atom_id in arich_out["all_topics"][:3]:  # top 3 topics
        results = brain.parallel_21_dives(lemma, mother_weights=mother_weights)
        for aid, data in results.items():
            combined_results[aid]["mothers"].update(data["mothers"])
            combined_results[aid]["score"] += data["score"]
            combined_results[aid]["types"].extend(data["types"])
            combined_results[aid]["from_topic"].append(lemma)
    
    # Sort
    flash = sorted(combined_results.items(), key=lambda x: -x[1]["score"])[:8]
    
    # Group by mother
    structure = defaultdict(list)
    for aid, data in combined_results.items():
        for m in data["mothers"]:
            structure[m].append((aid, data["score"]))
    for m in structure:
        structure[m].sort(key=lambda x: -x[1])
        structure[m] = structure[m][:5]
    
    # Multi-mother
    multi = [(aid, d) for aid, d in combined_results.items() if len(d["mothers"]) >= 2]
    multi.sort(key=lambda x: -x[1]["score"])
    
    return {
        "all_results": dict(combined_results),
        "flash": flash,
        "structure": dict(structure),
        "multi_confirmed": multi[:5],
        "n_topics": len(arich_out["all_topics"]),
    }


def zeir_anpin_v2(brain: Brain, abba_ima: dict, arich_out: dict) -> dict:
    if not abba_ima["all_results"]:
        return {"status": "empty"}
    
    top_facts = []
    for atom_id, data in abba_ima["flash"][:8]:
        atom = brain.atoms[atom_id]
        top_facts.append({
            "lemma": atom.lemma,
            "score": data["score"],
            "types": data["types"],
            "mothers": data["mothers"],
            "from_topic": data["from_topic"],
        })
    
    confirmed = [(brain.atoms[aid].lemma, d) for aid, d in abba_ima["multi_confirmed"]]
    
    per_mother = {m: [(brain.atoms[aid].lemma, s) for aid, s in items]
                  for m, items in abba_ima["structure"].items()}
    
    return {
        "status": "ok",
        "top_facts": top_facts,
        "confirmed": confirmed,
        "per_mother": per_mother,
        "primary_topic": arich_out["primary_topic"],
    }


def nukva_v2(za_out: dict, arich_out: dict, speaker_style: dict, safety_check: dict) -> str:
    """Better response generation — actually constructs varied answers"""
    if not safety_check["safe"]:
        return safety_check["refusal_message"]
    
    if za_out["status"] == "empty":
        return "אני לא מכיר את זה עדיין. תספר לי יותר ואלמד."
    
    formality = speaker_style.get("formality", "neutral")
    depth = speaker_style.get("depth", "medium")
    intent = arich_out["intent_vec"]
    
    primary_topic = za_out.get("primary_topic")
    confirmed = za_out.get("confirmed", [])
    per_mother = za_out.get("per_mother", {})
    top_facts = za_out.get("top_facts", [])
    
    # Determine query "shape" by sefirot dominance
    top_sefira = arich_out["primary_sefirot"][0][0] if arich_out["primary_sefirot"] else "malkhut"
    
    parts = []
    
    # ── Question type: Sensory ("what color", "what taste") ──
    if intent.get("daat", 0) > 0.5 or intent.get("malkhut", 0) > 0.4:
        if "Sensory" in per_mother and per_mother["Sensory"]:
            sens_items = per_mother["Sensory"][:3]
            sens_str = ", ".join([s[0] for s in sens_items])
            if formality == "casual":
                parts.append(f"מבחינת חושים — {sens_str}.")
            else:
                parts.append(f"התכונות החושיות: {sens_str}.")
    
    # ── Question type: Functional/cause ──
    if intent.get("bina", 0) > 0.5 or intent.get("yesod", 0) > 0.4:
        if "Functional" in per_mother and per_mother["Functional"]:
            func_items = per_mother["Functional"][:3]
            func_str = ", ".join([s[0] for s in func_items])
            parts.append(f"מבחינה תפקודית — {func_str}.")
    
    # ── Question type: Abstract/symbolic ──
    if intent.get("chokhma", 0) > 0.5 or intent.get("tiferet", 0) > 0.4:
        if "Abstract" in per_mother and per_mother["Abstract"]:
            abs_items = per_mother["Abstract"][:3]
            abs_str = ", ".join([s[0] for s in abs_items])
            parts.append(f"מבחינה מושגית — {abs_str}.")
    
    # ── Recommendation ── (chesed)
    if intent.get("chesed", 0) > 0.5:
        if confirmed:
            top_recs = ", ".join([c[0] for c in confirmed[:3]])
            parts.append(f"הדברים הכי קשורים שאני ממליץ: {top_recs}.")
        elif top_facts:
            recs = ", ".join([f["lemma"] for f in top_facts[:3]])
            parts.append(f"אני מציע לבחון: {recs}.")
    
    # ── Comparison ── (tiferet)
    if intent.get("tiferet", 0) > 0.4 and len(arich_out["all_topics"]) >= 2:
        topics = [t[0] for t in arich_out["all_topics"][:2]]
        # Find common features
        common_in_both = []
        for f in top_facts:
            if len(set(f["from_topic"])) >= 2:
                common_in_both.append(f["lemma"])
        if common_in_both:
            parts.append(f"המשותף בין {topics[0]} ל-{topics[1]}: {', '.join(common_in_both[:3])}.")
        else:
            parts.append(f"בין {topics[0]} ל-{topics[1]} יש הבדלים — בוא אפרט.")
    
    # ── Cross-domain (multi-confirmed) ──
    if confirmed and not parts:
        confirmed_str = ", ".join([c[0] for c in confirmed[:5]])
        parts.append(f"החיבורים החזקים ביותר: {confirmed_str}.")
    
    # ── Fallback: just show top facts ──
    if not parts and top_facts:
        facts_str = ", ".join([f["lemma"] for f in top_facts[:5]])
        parts.append(f"מצאתי קשרים ל: {facts_str}.")
    
    if not parts:
        return "אני יודע על זה אבל לא מצאתי תשובה ספציפית. שאל מזווית אחרת?"
    
    # Casual opener
    if formality == "casual" and parts:
        opener = random.choice(["טוב,", "אז ככה,", "בקצרה,"])
        return f"{opener} {' '.join(parts)}"
    elif formality == "formal":
        return "התשובה: " + " ".join(parts)
    else:
        return " ".join(parts)


def answer_v2(brain: Brain, query: str, user_profile: dict = None) -> dict:
    trace = {"query": query, "steps": []}
    
    arich = arich_anpin_v2(query, brain)
    trace["steps"].append({
        "stage": "ArichAnpin",
        "primary_topic": arich["primary_topic"],
        "all_topics": [t[0] for t in arich["all_topics"]],
        "top_sefirot": arich["primary_sefirot"]
    })
    
    safety = SafetyCheck()
    safety_result = safety.check(query, user_profile)
    trace["steps"].append({"stage": "Safety", "result": safety_result})
    
    if not safety_result["safe"]:
        # Skip dives if unsafe
        return {
            "response": safety_result["refusal_message"],
            "trace": trace
        }
    
    abba_ima = abba_ima_parallel_v2(brain, arich)
    trace["steps"].append({
        "stage": "Abba+Ima",
        "n_topics_used": abba_ima.get("n_topics", 0),
        "total_nodes_found": len(abba_ima["all_results"]),
        "multi_confirmed": len(abba_ima["multi_confirmed"]),
    })
    
    za = zeir_anpin_v2(brain, abba_ima, arich)
    trace["steps"].append({"stage": "ZeirAnpin", "status": za["status"]})
    
    style = detect_speaker_style(query, user_profile)
    response = nukva_v2(za, arich, style, safety_result)
    trace["steps"].append({"stage": "Nukva", "style": style})
    
    return {"response": response, "trace": trace}
