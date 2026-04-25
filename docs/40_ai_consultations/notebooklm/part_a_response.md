# NotebookLM Response — Part A (Questions 1-7) — Gap Designs

**תאריך:** 25.04.2026

## Q1 — Edge Schema ⭐
**תשובה:** 32 bits per edge in CSR:
- **24 bits** = target ID (up to 16.7M atoms)
- **8 bits** = edge type (256 kinds)

**Sefer Yetzirah mapping:**
- **3 Mothers (אמש)** = infrastructure edges: identity / containment / causality
- **7 Doubles (בגדכפרת)** = symmetric/bidirectional edges
- **12 Simples (הוזחטיכלמנסעצק)** = attribute + time edges

**Confidence:** N/A but well-reasoned
**⚠️ קונפליקט עם §0 ABI:** קבענו EdgeKind=u16 (לפי clarity audit). NotebookLM מציע 8 ביט. צריך reconciliation.

## Q2 — Global Workspace ⭐
**Salience formula (deterministic!):**
```
Salience(A) = (Degree_Centrality × 0.3) + (Recent_Visits × 0.7)
```

**Decay:** Recent_Visits × 0.95 per micro-sleep cycle  
**Top-20 array:** read in O(1) from cache by all modules (Planner, Evaluator)  
**Entry/Exit:** atom enters if Salience > weakest in list

**Confidence:** Implementable מיידית

## Q3 — Affective State ⭐ Implementable Now
**Concrete update equations:**
```
על הצלחה ביעד:
  Confidence = min(255, Confidence + 20)
  Frustration = max(0, Frustration - 50)

על מבוי סתום:
  Frustration = min(255, Frustration + 15)

על אטום שטרם בוקר:
  Curiosity = min(255, Curiosity + 5)

על סיבוכיות:
  Fatigue = min(255, Fatigue + Nodes_Visited/100)
```

**Walk modulation:**
```
Depth_Limit = Base_Depth × (1 + (Confidence - Fatigue) / 255)
Breadth_Limit = Base_Breadth × (1 + Curiosity / 255)
```

**Confidence:** 10/10 implementable

## Q4 — Learned Ranker
**ארכיטקטורה:** Cross-encoder TRM-like, **~7M params**  
**Quantization:** INT8 → **<10MB RAM, <10ms inference**  
**Bootstrap:** Hebrew Wikipedia disambiguation pages + Wikidata + basic relations

**Confidence:** Concrete numbers, implementable

## Q5 — NL Realization
**LM role = stylistic polish ONLY**  
- Template assembly first (deterministic)
- LM polish AFTER for stylistic flow
- For fact-critical responses: **disable LM, return raw template**

**Pronoun resolution:** deterministic **backward walk** in path graph — finds first atom whose attribute edges (gender/number) match current verb/adjective.

**Confidence:** Zero hallucination preserved by design

## Q6 — Morphological Rules
**Coverage:** **5,000-8,000 rules** cover most Hebrew  
**Encoding (bitmask):** Condition + Action + Priority (single u64?)  
**Matching:** Bitwise AND against target atom  
**Conflict:** highest Priority wins immediately — **no decision tree**

**Confidence:** Fast (bitwise), deterministic

## Q7 — Code Quarantine
**4-tier hierarchy:**
- **L0 Untrusted:** read-only sandbox
- **L1 Tested:** promotion after 100 successful simulation runs, zero errors, resource compliance. Allowed: write to temp graph
- **L2 Verified:** requires Critic approval. Allowed: modify edges
- **L3 Core:** infrastructure. **Any single security breach → drops to L0**

**Confidence:** Implementable as state machine

