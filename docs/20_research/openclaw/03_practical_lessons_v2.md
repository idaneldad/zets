# OpenClaw — Practical Lessons V2: Capabilities-as-Graph + Anti-Copying + Deduplication

**Date:** 2026-04-23  
**Status:** Synthesis — actionable engineering plan  
**Inputs:** OpenClaw deep-read + 3 AI consultations (gpt-4o, Gemini-2.5-flash, Claude-Sonnet-4)  
**Supersedes:** `02_lessons_v1.md` (initial analysis, kept for trace)

---

## TL;DR

Three things from OpenClaw that ZETS can imitate with the **graph method** to keep code thin and reuse maximal:

1. **Skills become Capability Atoms.** A skill is not a code blob — it is a graph node referenced by many procedures. Storage = O(unique capabilities), not O(procedures).
2. **External code → graph procedures via behavioral extraction.** Read TypeScript/Python from any repo, distill to **what it does** (inputs/outputs/side-effects/test-cases), discard the source, regenerate fresh implementation. Original by construction, not by find-and-replace.
3. **Deduplication cascade.** Every new capability passes through 4 filters (semantic embed → behavioral test → sense-key match → hash) before being added. Near-duplicates become version-edges, not new nodes.

Plus an **operational lesson** OpenClaw teaches that we missed: hot-reload + audit trails + graceful degradation + fallback edges.

---

## חלק 1 — שבירת כלים (Breaking Vessels)

לפני התשובה — מה אסור להניח?

**הנחה 1 שצריכה להישבר:** "ZETS תוכל להעתיק קוד מ-OpenClaw."  
**שבירה:** OpenClaw הוא MIT, אבל המוניטין שלנו זה לא העניין — העניין הוא שאם אנחנו רק מעתיקים, אנחנו לא **מבינים**, ולא מסוגלים לשפר. אם ZETS תהיה רק "OpenClaw בRust" — היא תורש את כל החולשות שלהם.

**הנחה 2 שצריכה להישבר:** "פרוצדורה = bytecode."  
**שבירה:** פרוצדורה = **graph node** שיש לו שם, signature, when-to-use, וedges לסבסטרט שלו. Bytecode הוא רק העלים האחרונים בdAG. רוב ה-procedures יהיו composition (CallProcedure→CallProcedure→...) ולא bytecode חדש.

**הנחה 3 שצריכה להישבר:** "כדי לקרוא לcapability צריך to know שמה."  
**שבירה:** הצורה הנכונה — **find_capability_by_intent**. אומרים מה רוצים (sense), הגרף מוצא את הproceduure המתאימה. זו לא service discovery רגילה — זו walk סמנטית.

**הנחה 4 שצריכה להישבר:** "Deduplication = hashing."  
**שבירה:** שני implementations שונות לחלוטין יכולות להיות **התנהגותית זהות**. שתי implementations זהות בbytes יכולות להיות **התנהגותית שונות** (אחת async, אחת sync). דדופ = behavioral, לא syntactic.

**הנחה 5 שצריכה להישבר:** "AI generation מספיק כדי להיות 'original'."  
**שבירה:** LLM שנמצא להם source code → translate ל-Rust = transliteration עם "rust accent". זה **לא original**. Original דורש שלב אבסטרקציה ביניים שמשליך את הsource.

---

## חלק 2 — תשובות מ-3 ה-AIs

### Q1: PROCEDURE-AS-GRAPH מינימיזציה

**הסכמה מלאה (3/3):** Capability atoms + composition edges, **לא** bytecode blobs.

המבנה (Anthropic Claude נתן את הניסוח החד ביותר):
```rust
// Atom שהוא capability — ייחודי, משותף לרבים
AtomType::Capability {
    id: "http_post",
    inputs: ["url", "headers", "body"],
    outputs: ["response", "status_code"],
    implementation: BytecodeRef(vm_opcodes),  // bytecode רק כאן, פעם אחת
}

// Procedure = composition graph
AtomType::Procedure {
    id: "send_whatsapp",
    graph: [
        Step(0): "build_whatsapp_payload"  → inputs: ["message", "phone"]
        Step(1): "build_oauth_headers"     → inputs: ["whatsapp_token"]
        Step(2): "http_post"               → inputs: [Step(0), Step(1), URL]
        Step(3): "parse_whatsapp_response" → inputs: [Step(2)]
    ]
}
```

**Storage win:** http_post existed once. send_whatsapp + send_telegram + send_slack share it. כשמוסיפים send_line — http_post **לא נכפל**.

**מה ZETS כבר יש:** `procedure_atom.rs` עם `ProcedureStep::CallProcedure`. **חסר:** edge type `CompositionEdge` ב-`bitflag_edge.rs`, ו-CapabilityRegistry שמאתר atomes לפי intent.

**Gemini הוסיף:** Input/Output Signatures **הם עצמם atoms** מקושרים דרך `HAS_INPUT_SCHEMA` / `HAS_OUTPUT_SCHEMA` edges. כך type-checking הוא graph traversal, לא kompilation.

### Q2: INGESTION של קוד חיצוני (anti-copying)

**אסטרטגיה משולשת (Anthropic):** Extract → Abstract → Resynthesize

```
External code (TypeScript/Python/Go)
        ↓
Stage 1: Behavioral Spec
   - parse AST, extract function signatures, docstrings, test cases
   - DO NOT store source code, only: inputs, outputs, side_effects, test_cases
   - if dynamic analysis safe: run in sandbox, observe I/O
        ↓
Stage 2: Canonical Capability
   - group by semantic similarity (use sense_graph)
   - "fetch URL" + "HTTP GET" + "download file" → same canonical purpose
   - store: purpose, input_types, output_types, constraints
        ↓
Stage 3: Synthesize Fresh Implementation
   - LLM: "Write Rust function that takes [inputs] produces [outputs] with behavior: [canonical_description]"
   - LLM does NOT see original source — only the canonical spec
   - validate against extracted test cases
   - if test passes: ProcedureAtom registered with TrustLevel::Learned
```

**Anti-copying mechanism (Gemini הסביר היטב):**
- The LLM gets **canonical description**, not source. Cannot transliterate what it doesn't see.
- Output is constrained to **ZETS procedure_atom schema** — different mental model from TypeScript code.
- Existing graph context is supplied — LLM is encouraged to **compose** existing atoms, not create new ones.
- Iterative refinement: "If still too similar, use different algorithmic approach or compose existing primitives in novel way."

**אזהרה (gpt-4o):** plagiarism detection לא מספיקה כstop-gap. הצעה: השווה semantic embedding של הוצא מול הinput — אם דומה > 0.95, סמן לreview ידני. אנחנו רוצים behavioral similarity, לא code similarity.

**הצעת implementation (משלב 3 התשובות):**

```rust
struct CapabilityIngester {
    // Stage 1
    fn extract_behavior(repo_url: &str, file: &str) -> Vec<BehaviorSpec> {
        // walk AST (using tree-sitter for TS/Py/Go),
        // for each function: signature + tests + docstring
        // return BehaviorSpec — NEVER the source
    }
    
    // Stage 2
    fn canonicalize(spec: BehaviorSpec) -> CanonicalCapability {
        // generate canonical English description
        // cluster against existing capabilities via sense_graph + embedding
        // if clusters with existing → return existing AtomId (deduplication!)
        // else → new CanonicalCapability
    }
    
    // Stage 3
    fn synthesize(cap: CanonicalCapability) -> ProcedureAtom {
        let prompt = format!(
            "Write a ZETS procedure that takes {:?} and produces {:?}.
             Behavior: {}.
             Existing ZETS capabilities you may compose: {:?}.
             Output a procedure_atom in TOML format. Do not output any code 
             that resembles existing implementations. Maximize compositional 
             reuse over new bytecode.",
            cap.inputs, cap.outputs, cap.purpose, list_existing()
        );
        let proc = llm.complete(prompt);
        validate_against(proc, cap.test_cases)?;
        ProcedureAtom { trust_level: Learned, ..proc }
    }
}
```

### Q3: דדופליקציה מדורגת (4 שכבות)

**Gemini נתן את הסדר הנכון:**

| שכבה | מתי | איך | יתרון | חיסרון |
|------|------|------|--------|---------|
| **1. Semantic Embedding** | first pass תמיד | Sentence-BERT על description, cosine ≥ 0.9 | תופס conceptual similarity, מהיר | requires LLM/embed model |
| **2. Behavioral test** | אחרי 1 חיובי | run test inputs דרך שני המועמדים, השווה outputs | functional equivalence אמיתי | יקר חישובית |
| **3. Sense-key match** | edge cases / high-trust | sense_graph WordNet synsets + bitflag axes | precision מדויק | דורש human |
| **4. Hash exact** | integrity / versioning | SHA256 of canonical form | מהיר | רק עבור duplicates מדויקים |

**Near-duplicates (http_post_v1 vs http_post_v2_with_retry):** לא יוצרים atom חדש מאפס. במקום זה:
- Atom אחד שורש: `http_post`
- שני sub-atoms: `_v1`, `_v2_with_retry`
- Edge `IS_VARIANT_OF` בין v1↔v2 (סוג חדש ב-bitflag_edge)
- Edge `IMPROVES_UPON` מ-v2 ל-v1
- Metadata: version, has_retry_logic, network_resilience_score

**Selection time:** ה-system_graph בוחר variant לפי policy (`use latest stable` / `use retry-capable if network unreliable`).

### Q4 (operational): מה לחקות מ-OpenClaw

**Anthropic הצביע על 5 patterns:**

1. **Hot-reload capability registration** — manifest scan ↔ אין צורך restart
2. **Lazy bytecode load** — מתי שצריך, מ-mtreemap cache
3. **Execution sandboxing** — TrustLevel + allowed_tools enforce ב-VM
4. **Graceful degradation** — `FallbackEdge`: send_whatsapp → send_sms → send_email
5. **Audit trail** — כל invocation מתועדת: timestamp, input_hash, output_hash, execution_time, trust_level, caller_identity

**שלוש שכבות שאפשר לאמץ מיידית** (לא דורש שינוי ארכיטקטוני גדול):
- **Audit trail:** opcode חדש `AuditLog = 70` שכותב ל-WAL כל invocation.
- **Fallback edges:** הוספה ל-`bitflag_edge.rs` של axis חדש `FallbackTo`.
- **Hot-reload:** scanner שרץ ב-background על `data/procedures/` ומעדכן ProcedureRegistry.

---

## חלק 3 — Synthesis: ארכיטקטורה ל-ZETS

### 3.1 Tools Graph (השאלה של עידן)

עידן שאל: **"גרף של TOOLS שיוכל להכיל עליו את היכולות וירוצו בזמן אמת."**

זו לא רק רעיון טוב — זו ההתחייבות שכבר עשינו. הצורה שלו:

```
                        TOOLS GRAPH
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
  primitive_tools       composition_tools    domain_tools
   (leaf bytecode)     (CallProcedure DAG)  (per-domain wrappers)
        │                   │                   │
   ┌────┴────┐          ┌───┴────┐         ┌───┴────┐
   │         │          │        │         │        │
http_post  json_parse  send_msg search    medical  legal
   │           │          │       │          │        │
   └─reused by─┴──────────┴───────┴──────────┴────────┘
                           ▲
                           │
                  Procedures use these
```

**היררכיה:**
- **Level 0 (primitive_tools):** ~50 capabilities — http, json, regex, hashing, time, math
- **Level 1 (composition_tools):** ~200 — send_message_via_X, fetch_and_parse_Y
- **Level 2 (domain_tools):** ~1000 — domain-specific wrappers (medical_lookup, legal_search, ...)
- **Level 3 (user_procedures):** unlimited — what users compose

**runtime resolution:** משתמש מבקש "תשלח לדוד שהפגישה נדחתה" → intent_parse → find_capability("send_message", channel="whatsapp") → walks Level 2 → resolves to procedure → executes via VM.

### 3.2 איך ZETS לומדת capabilities מ-API חדש שמעולם לא ראתה

**זו השאלה הקריטית של עידן.** התשובה היא **5-step learning loop:**

```
Step 1: DISCOVER — קבלת spec
  - URL לdocs / OpenAPI swagger / GitHub repo / npm package
  - או: user prompt "תלמד API של GreenAPI"

Step 2: PARSE — extract behavioral spec
  - אם יש OpenAPI: parse endpoints, params, responses → BehaviorSpec
  - אם יש docs: LLM extract endpoints + examples
  - אם יש repo: tree-sitter על קוד דוגמה
  - תוצאה: list of {endpoint, inputs, outputs, auth_required, side_effects}

Step 3: ABSTRACT — match against existing capabilities
  - "POST request to send message" → matches http_post + send_message pattern
  - sense_graph cluster: "communication.send.text"
  - reuse existing primitives where possible

Step 4: SYNTHESIZE — propose new procedure
  - if all sub-capabilities exist: just compose (no new bytecode)
  - if some missing: synthesize new primitive (Stage 3 of ingestion)
  - register as TrustLevel::Learned, allowed_tools constrained, sandbox-only

Step 5: VERIFY — test in sandbox
  - run procedure with test inputs (from API docs examples)
  - if outputs match: promote to OwnerVerified-pending (waits for owner approval)
  - if fail: log failure, generate improvement goal for next iteration
```

**Concrete example: GreenAPI WhatsApp**

```
Input: https://greenapi.com/docs/api/sending/SendMessage/
       (or: "ZETS, learn how to send WhatsApp via GreenAPI")

Step 1 (DISCOVER): user prompt + URL
Step 2 (PARSE): 
   - LLM reads docs, extracts:
     POST /waInstance{instanceId}/sendMessage/{token}
     body: {chatId: string, message: string}
     auth: token in URL
     response: {idMessage: string}
Step 3 (ABSTRACT):
   - http_post exists ✓ (reuse)
   - build_request exists ✓ (reuse)
   - parse_json exists ✓ (reuse)
   - new: build_greenapi_url (template substitution) — propose new primitive
   - new: send_whatsapp_via_greenapi (composition of existing) — propose
Step 4 (SYNTHESIZE):
   - LLM writes procedure_atom TOML:
     [procedure.send_whatsapp_via_greenapi]
     when_to_use = ["communication.send.whatsapp"]
     allowed_tools = ["http_post", "build_greenapi_url", "parse_json"]
     trust_level = "Learned"
     steps = [
       { call = "build_greenapi_url", args = {...} },
       { call = "build_request", args = {...} },
       { call = "http_post" },
       { call = "parse_json" },
     ]
Step 5 (VERIFY):
   - test in sandbox with mock server
   - if 200 OK + idMessage returned → promote
```

**אחרי שלמדה — שמורה לעד בגרף.** הפעם הבאה שמישהו אומר "תשלח whatsapp" → procedure נמצאת → רצה. **בלי learning מחדש.**

### 3.3 Capability Registry — איפה הכל יושב

```rust
pub struct CapabilityRegistry {
    /// All capability atoms, indexed by AtomId
    capabilities: HashMap<AtomId, ProcedureAtom>,
    
    /// Sense-key index: "communication.send.whatsapp" → [AtomId,...]
    by_sense: HashMap<SenseKey, Vec<AtomId>>,
    
    /// Input/output type index for composition matching
    by_io_signature: HashMap<IoSignature, Vec<AtomId>>,
    
    /// Hash index for exact dedup
    by_canonical_hash: HashMap<[u8;32], AtomId>,
    
    /// Embedding index for semantic dedup (uses mtreemap)
    embedding_index: MTreeMap<Embedding, AtomId>,
    
    /// Variant relationships
    variants: HashMap<AtomId, Vec<(AtomId, VariantRelation)>>,
}

impl CapabilityRegistry {
    /// THE main API: "find me a capability that does X"
    pub fn find(&self, intent: &Intent) -> Vec<&ProcedureAtom> {
        // 1. exact sense match
        if let Some(ids) = self.by_sense.get(&intent.sense_key) {
            return ids.iter().map(|id| &self.capabilities[id]).collect();
        }
        // 2. semantic similarity
        let embed = embed(&intent.description);
        let candidates = self.embedding_index.knn(embed, k=10);
        // 3. filter by I/O compatibility
        candidates.into_iter()
            .filter(|c| self.io_compatible(c, intent))
            .collect()
    }
    
    /// Add new — passes through dedup cascade
    pub fn add(&mut self, candidate: ProcedureAtom) -> RegisterResult {
        // Layer 4: hash exact
        let hash = canonical_hash(&candidate);
        if let Some(&existing) = self.by_canonical_hash.get(&hash) {
            return RegisterResult::DuplicateExact(existing);
        }
        // Layer 1: semantic
        let embed = embed(&candidate.description);
        for (other_embed, &other_id) in self.embedding_index.knn(embed.clone(), k=5) {
            if cosine(&embed, &other_embed) > 0.92 {
                // Layer 2: behavioral
                if behaviorally_equivalent(&candidate, &self.capabilities[&other_id]) {
                    return RegisterResult::DuplicateBehavioral(other_id);
                }
                // near-duplicate → variant edge, not new node
                if cosine(&embed, &other_embed) > 0.85 {
                    self.add_as_variant(other_id, candidate);
                    return RegisterResult::AddedAsVariant(other_id);
                }
            }
        }
        // truly new
        let id = self.next_id();
        self.capabilities.insert(id, candidate);
        // ... index updates ...
        RegisterResult::Added(id)
    }
}
```

### 3.4 איך זה רזה (מינימום נתונים)

נניח 1000 procedures. מבלי הגרף:
- 1000 × ~5KB bytecode = **5MB**

עם capability graph:
- 50 primitives × 5KB bytecode = **250KB**
- 1000 procedure_atoms × ~200B (steps + metadata) = **200KB**
- **Total: 450KB. 11× פחות.**

ועוד יותר חשוב — כשמוסיפים procedure 1001, צריך **רק 200B** חדשים, לא 5KB.

זו ההצדקה האמיתית לגרף-מבוסס.

---

## חלק 4 — תוכנית יישום (פעולות, לא פילוסופיה)

### Sprint 1 (שבוע 1) — Tools Graph foundation
- [ ] `src/capability_registry.rs` — CapabilityRegistry struct + add/find/dedup cascade
- [ ] Extend `bitflag_edge.rs` — new edge types: `CompositionOf`, `VariantOf`, `ImprovesOn`, `FallbackTo`
- [ ] `src/procedure/io_signature.rs` — typed input/output schemas as graph atoms
- [ ] 50 seed primitive capabilities (http_get, http_post, json_parse, regex_match, ...)
- [ ] Tests: 30+ unit, including dedup cascade

### Sprint 2 (שבוע 2) — Ingestion pipeline
- [ ] `src/ingestion/external_repo.rs` — clone + tree-sitter walk
- [ ] `src/ingestion/behavior_spec.rs` — extract BehaviorSpec from AST
- [ ] `src/ingestion/canonicalize.rs` — cluster via sense_graph
- [ ] `src/ingestion/synthesize.rs` — LLM call with anti-copying constraints
- [ ] `src/ingestion/verify.rs` — sandbox execution + test validation
- [ ] First test: ingest one OpenClaw skill (github skill) end-to-end

### Sprint 3 (שבוע 3) — API learning
- [ ] `src/learning/api_discovery.rs` — OpenAPI/Swagger parser
- [ ] `src/learning/api_synthesizer.rs` — generate procedures from API spec
- [ ] First test: learn GreenAPI WhatsApp from docs URL → working procedure

### Sprint 4 (שבוע 4) — Operational patterns
- [ ] Hot-reload procedure scanner (background task)
- [ ] Audit trail opcode (`AuditLog = 70`) + WAL integration
- [ ] Fallback edge resolution in VM
- [ ] Lazy bytecode load via mtreemap

### Continuous: anti-copying enforcement
- [ ] Pre-commit hook: scan for verbatim source from known repos
- [ ] Embedding-similarity check in CapabilityRegistry::add
- [ ] Manual review queue for high-trust procedures

---

## חלק 5 — מה **לא** לעשות

חוקי "no-go" שעולים מהניתוח:

1. **לא לתעד procedure ש-bytecode ארוך יותר מ-1KB** בלי decomposition. Bytecode ארוך = capability שלא נשבר לprimitives. שבור אותו.

2. **לא להוסיף procedure בלי `when_to_use` sense_keys.** בלי זה אין discovery, ה-procedure רוחג בגרף ולא נמצאת.

3. **לא ל-ingest מ-repo בלי sandbox.** אין ingestion-then-trust. כל מה שנכנס מבחוץ → TrustLevel::Learned, רץ בsandbox, ממתין ל-owner approval.

4. **לא ל-call LLM עם source code חיצוני.** LLM מקבל **רק canonical spec**. Source code נשאר ב-staging area, נמחק אחרי extraction.

5. **לא ליצור variant אם מספיק לעדכן.** אם capability v1 שגויה → תקן v1, אל תוסיף v2. רק כאשר יש שינוי behavioral אמיתי (retry, async, batch) → version edge.

6. **לא להפיץ procedures ב-marketplace ללא owner sign.** Capability registry יכול להיות ציבורי, אבל רק owner-signed procedures מורצים אצל owners אחרים.

---

## חלק 6 — מטריקות הצלחה

איך נדע שזה עובד?

1. **Reuse ratio:** sum(steps_per_proc) / unique_capabilities. יעד: > 5× (כל primitive בשימוש לפחות 5 פעמים).
2. **Storage efficiency:** total_bytes / total_procedures. יעד: < 500B/procedure בממוצע.
3. **Dedup hit rate:** של add() calls שמחזירים DuplicateBehavioral. יעד: > 30% (חצי מה-add מצליחים, חצי כפולים).
4. **Time-to-learn:** מ-API URL חדש ל-working procedure. יעד: < 5 דקות autonomous.
5. **Failure recovery:** אחוז של failed executions שמתאוששים דרך fallback. יעד: > 80%.

---

## הפניות

- `00_doctrine/01_engineering_rules.md` — Idan's binding rules
- `10_architecture/06_system_graph_vm.md` — VM and 32 opcodes
- `20_research/openclaw/01_external_analysis.md` — initial OpenClaw read
- `40_ai_consultations/20260423_openclaw_lessons_3ai.md` — raw AI inputs (this synthesis)
- `30_decisions/0004_procedure_atom_model.md` — TBD ADR for procedure_atom design
- `30_decisions/0005_capability_registry_model.md` — TBD ADR for this lessons doc

---

## נקודה לחשוב עליה

עידן אמר: "המטרה שהקוד שלנו יהיה רזה מאוד ויעיל מאוד שיפעיל סקריפטים על הגרף ויוכל ליצור סקריפטים מתוך זה שהוא לומד ממקורות."

זה **בדיוק** מה שמתואר כאן. אבל זה דורש משמעת:
- כל פיסת קוד שאי פעם נכתב צריכה לעבור דרך CapabilityRegistry::add
- LLM אסור לראות source code, רק canonical specs
- Storage = O(unique), לא O(total)

אם נשמור על המשמעת — אחרי שנה ZETS תהיה מוצר שלא ניתן להעתיק. כי הvalue שלו לא בקוד אלא ב**recombination graph** של 5000+ capabilities שנלמדו מ-1000+ external sources, כל אחד deduplicated, גירסה מאומתת, embedding indexed.

זה לא project. זו **שיטת ייצור.**
