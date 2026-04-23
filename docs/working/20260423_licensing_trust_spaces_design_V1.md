# Licensing + Trust Spaces Design — מרחבי אמון, הכנסה משולבת, בלי פח

**תאריך:** 23.04.2026  
**בקשת עידן:**
1. שני מצבי רישוי: **preinstalled** (כמו Windows ב-laptop) + **regular installation** (cloud/client/local)
2. זיהוי לא-חובה ב-offline, השלמה כשיש אינטרנט
3. מצב **היברידי** — רוב מקומי + חלק cloud + hypothetically co-training
4. **מרחבי אמון** — כמו ילד שלא הולך עם זרים אבל עונה לדוד (למידה מכלי AI אחרים)

---

## שבירת כלים #1 — "license key זה מספר ייחודי"

**השבר:** לא. license key = **atoms בגרף עם proof chain**.

מפתחות SaaS רגילים: "XXX-YYY-ZZZ" → שרת מאמת → on/off. אבל:
- **Offline**: מה קורה כשאין אינטרנט?
- **Revocation**: איך יודעים שהכרטיס בוטל?
- **Granular**: איך רק "advanced search" ללא "teach"?
- **Hybrid**: איך חלק offline חלק online?

**התובנה החדשה:** ZETS כבר הוא graph — **הcapability עצמה היא atom**. הlicense היא **edge חתום** מ-authority atom אל user atom:

```
[Authority:Anthropic-Root] ──signs──> [License:xxx:issued-to:device-id-abc]
   │
   └──granted──> [Capability:query:1000/hr]
   └──granted──> [Capability:teach]
   └──granted──> [Capability:custom_domains]
   └──expires-at──> [Date:2027-01-01]
```

**כל capability = atom + edge עם חתימה.** offline: אני קורא את הgraph, מוודא שהחתימות תקינות, רץ. online: רציתי לעדכן — שואל את ה-authority אם יש מחדש.

**יתרון:** אותו מודל ללקפיטליזם **מכל** השאלות שלך.

---

## שבירת כלים #2 — "preinstalled = רשומה מראש"

**השבר:** לא. preinstalled = **OEM signing + deferred activation**.

איך Windows עושה את זה ב-laptops:
- OEM (Dell/HP/Lenovo) מקבל "SLIC" — System Licensed Internal Code — **אובייקט חתום ב-BIOS**
- כשWindows רץ → קורא ACPI table → מוצא SLIC → סומך עליו → activated
- **אין online check** — OEM signed it, Microsoft כבר יודעת

**עבור ZETS:** 
- יש `installer.zets_enc` (כבר קיים ב-`src/encrypted_installer.rs`)
- בעת ייצור של הlaptop — ה-OEM מוסיף:
  - **atom** `device:id:abc123` (fingerprint של המכונה)
  - **atom** `license:tier:pro`
  - **edge** חתום במפתח פרטי של Anthropic/ZETS-authority
- כשZETS עולה → מוצא את ה-atoms → מוודא חתימה → פועל
- **אפס online check**. **אפס פרטיות דלוז**. **אפס download מאולץ**

**כשיש אינטרנט (לא חובה):**
- ZETS שולח `POST /api/v1/register` עם fingerprint
- מקבל `ticket_id` לצורך תמיכה + updates
- **לא** שולח שאילתות, שיחות, תוכן

---

## שבירת כלים #3 — "cloud AI = API key + monthly bill"

**השבר:** בשביל ZETS זה שגוי. **4 מודלים מקבילים**, המשתמש בוחר:

| מודל | Where compute | Privacy | Needs internet | דמיון חיצוני |
|------|---------------|---------|----------------|-------------|
| **Local-only** | Device | 100% private | לא חובה | Local LLM |
| **Client-assisted** | Device + occasional API | chosen queries cloud | לפעמים | Claude Code |
| **Cloud** | Server | Anthropic-grade TLS | תמיד | ChatGPT web |
| **Hybrid** | Both, user decides per-query | Per-query granular | לפעמים | **חדשני — לא קיים באף כלי AI** |

המודל **הכי מעניין הוא 4 — hybrid**. הוא ייחודי לZETS כי רק אנחנו מבינים:
- "השאלה הזו רגישה" (user preference atom) → local only
- "השאלה הזו דורשת corpus חיצוני" → route to cloud
- "השאלה בודקת graph personal" → local only
- "השאלה גנרית" → local ינסה, cloud אם נכשל

זה **graph-based routing**. לא parameter של התקנה — **החלטה per-query**.

---

## שבירת כלים #4 — "cross-pollination = הזיקי נתונים"

**השבר:** אפשר ללמוד מ-peer **בלי לחשוף נתונים**.

הפתרון: **federated learning** אבל בגרסה graph-native:

- Device A למד: `concept:X ──has_property──> value:Y` עם confidence 0.8
- Device A שולח ל-peer שרת: **hash(concept:X)** + **confidence delta** (לא התוכן!)
- שרת מצטבר hashes מ-1000 devices
- אם 800/1000 מסכימים על `hash(X)` → שרת מסמן `X` כ-widely-corroborated
- Device B, שאין לו X, יכול **לבקש** אותו (pull-based, לא push)
- Device B מקבל X **רק אם tier שלו מאפשר + user approve**

**אפס הדלפת תוכן**. hash-only sync. המשתמש שולט ב-in/out.

---

## שבירת כלים #5 — "מרחבי אמון = whitelist/blacklist"

**השבר (הכי חשוב):** לא. **מרחבי אמון הם graph תת-מבני**.

הרעיון של עידן: "ילד עונה לדוד, לא לזרים." זה רעיון **עמוק**. במונחי graph:

```
[user] ──trust:high──> [family:dad]  ──can-ask──> anything
[user] ──trust:high──> [family:uncle] ──can-ask──> anything except:medical
[user] ──trust:medium──> [friend:rotem] ──can-ask──> movies,games
[user] ──trust:low──> [stranger:*] ──can-ask──> public-info-only
[user] ──trust:none──> [unknown:*] ──can-ask──> NOTHING
```

**חוקים נגזרים אוטומטית מהגרף:**
- שאילתה ל-ZETS: "מה הכתובת שלי?" מ-stranger → **denied** (trust:low)
- שאילתה: "מה הכתובת של עידן?" מ-family:dad → **allowed**
- שאילתה: "איזה תרופות אני לוקח?" מ-family:uncle → **denied** (specific exclusion)
- שאילתה: "איך מבשלים אורז?" מ-stranger → **allowed** (public-info-only allowance)

**מה שנחמד:** אין "list". יש **graph**. העשרה ביום בהדרגה. המשתמש מסמן "רוטם" → עיצוב graph משתנה → ZETS יודע מי מי אפילו לשאלות חדשות.

---

## שבירת כלים #6 — "אל תיפול בפח זה חוק"

**השבר:** זה **חישוב Bayesian על confidence של intent**.

הכלים הגדולים (Claude, GPT) **נכשלים** בזה לפעמים. הסיבה: הם מחליטים "harmful / not harmful" ב-boolean. ZETS יכול יותר טוב כי יש לו graph:

```
user_query → 
  extract: {intent, target, context}
  
  check graph:
    intent.similar_to: [known_harmful_intents]?  confidence
    target.protected: [user_pii, minors, financials]?  confidence
    context.authorized_by_user: explicit_opt_in?  confidence
  
  if any confidence > 0.7: REFUSE with reason as atom
  if sum confidence > 1.2: FLAG for user review
  else: PROCEED
```

**כל סירוב = atom חדש בגרף**. אם הגענו שגיאה ("סירבנו אבל אמרת אחר כך שזה ok"), זה edge "false_positive" שמעדן את ההחלטה הבאה. **ZETS לומד מטעויות בדיוק כמו ילד שלומד.**

### Learn from the AI giants — מה כבר הוכח עובד?

מכל מה שידוע על Claude + GPT + Gemini + Cortex:

| Pattern | זה עוזר |
|---------|---------|
| **Layered prompting** (system > developer > user) | כן — hierarchical trust |
| **Refuse list hardcoded** | **כן, אבל קצר** — רק red lines ממש קשים |
| **Contextual refusal** | **כן** — context-aware > list-based |
| **User-specific safety** | **כן** — מה שתועבר לchild != adult |
| **Sandboxed execution** | **כן** — tools לא כל יכולים הכל |
| **Audit log** | **חובה** — כל הפעולות רשומות |
| **Override mechanism** | **כן, קפדני** — admin יכול להתערב |
| **Privacy by design** | **חובה** — minimum data collected |

---

## הארכיטקטורה — ZETS-native

### שכבה 1: License Atoms

```rust
enum LicenseKind {
    OEM { device_id: DeviceId, issued_by: Authority },
    Subscription { customer_id: String, expires: SystemTime },
    Trial { starts: SystemTime, duration: Duration },
    Community { anonymous: bool },
    Enterprise { org_id: String, seats: u32 },
}

struct License {
    kind: LicenseKind,
    tier: Tier,                    // Free/Personal/Pro/Enterprise
    capabilities: Vec<Capability>, // granular
    signature: Ed25519Signature,   // signed by authority
}
```

**verification:**
- Load license atom
- Verify signature with known authority public keys
- Check `expires > now` (or unlimited for OEM)
- Activate corresponding capability atoms
- No internet required if signature valid

### שכבה 2: Activation Workflow (optional online)

```
Device first boot:
  1. Read OEM installer.zets_enc (if exists)  → atoms loaded, license valid
  OR
  1. Load fresh → no license → "free" tier only

  2. Optional: user connects → POST /register
     {device_fingerprint, license_id, user_email?}
  
  3. Server response:
     - ticket_id for support
     - optional updated license (new features)
     - authority keys refresh (key rotation)
  
  4. No internet → works fully in free tier
                   OR with OEM license
```

### שכבה 3: Trust Spaces (חדש, הרעיון של עידן)

```rust
enum TrustLevel {
    Stranger = 0,    // public info only
    Acquaintance = 1,
    Friend = 2,
    Family = 3,
    Self = 4,
    Authority = 5,   // admin, parent
}

struct TrustRelation {
    from: EntityId,      // who's asking
    to: EntityId,        // who they're asking about
    level: TrustLevel,
    scope: Vec<Domain>,  // medical/financial/personal
    exceptions: Vec<String>,
    granted_at: SystemTime,
    granted_by: EntityId,
}
```

**enforcement:**
- Every query → extract (asker_entity, subject_entity, scope)
- Lookup `TrustRelation` in graph
- Apply to query path: can walk only edges where trust allows
- Filter results: redact atoms where trust < atom's required level

### שכבה 4: Hybrid routing (query-level)

```rust
struct QueryPolicy {
    privacy_level: PrivacyLevel,  // Local | Internal | Anonymized | Public
    hints: QueryHints,            // complexity, corpus-needed
}

fn route_query(q: &Query) -> Route {
    match (q.policy.privacy_level, q.hints) {
        (Local, _) => Route::LocalOnly,
        (Public, Complex | CorpusHeavy) => Route::CloudFirst,
        (Anonymized, _) => Route::LocalWithCloudFallback { hash_only: true },
        (Internal, _) => Route::LocalWithCloudFallback { hash_only: false },
        _ => Route::LocalOnly,
    }
}
```

**עיצוב גרף של query routing:**
- זה לא config בקובץ, זה **atoms**
- משתמש יכול להגיד: `preference:query-routing:my:medical → LocalOnly`
- ZETS בוגר ב-behavior: "זה דומה לרפואה" → same routing

### שכבה 5: Federated Learning (opt-in)

```rust
struct FederatedSync {
    enabled: bool,
    endpoint: String,                   // "https://zets-federation.org"
    sync_what: SyncScope,               // Concepts | ConfidenceDeltas | Never
    exclude_domains: Vec<Domain>,        // never share medical/financial
}

enum SyncScope {
    Never,                              // default for privacy-focused
    HashesOnly,                         // corroboration without content
    ConfidenceDeltas,                   // "I learned X more than I knew"
    FullConcepts(Approval),             // explicit concept sharing
}
```

**cross-pollination אמת:**
- 1000 devices לומדים ש-"Python is a language" — hash agrees → global confidence goes up
- אף אחד לא חולק **מה** הם שאלו, **מתי**, **למה**
- Device מקבל "זה corroborated ב-1000 devices" כadvice, לא ידיעה מאולצת
- יכול לדחות

---

## מה כל המבני חותם?

### מבחן עקביות (self-check)

- [ ] Offline לחלוטין → ZETS רץ? **כן** (OEM license or community tier)
- [ ] Internet זמני → activation אופציונלית? **כן** (ticket_id flow)
- [ ] Child mode → אוטו refuse? **כן** (trust spaces + tier filter)
- [ ] Hybrid query → some local, some cloud? **כן** (per-query routing)
- [ ] Learn from peers without leaking? **כן** (hash-only federation)
- [ ] No single vendor lock-in? **כן** (authority keys are pluggable — ZETS-Anthropic + community CAs)
- [ ] Safety with transparency? **כן** (every refuse = atom w/ reason)
- [ ] Adjustable by user? **כן** (graph atoms are editable)

---

## Phased implementation (Rust)

**Phase 1 — Local licensing (שבוע 1):**
- `src/licensing/mod.rs` — License atom + verification
- `src/licensing/authority.rs` — public key trust chain
- `src/licensing/oem.rs` — OEM installer extension
- Leverage קיים: `crypto.rs`, `encrypted_installer.rs`

**Phase 2 — Activation endpoint (שבוע 2):**
- `src/licensing/activation.rs` — HTTP client for optional register
- `zets-tools/activation_server/` — receives registrations
- Offline fallback always works

**Phase 3 — Trust spaces (שבוע 3):**
- `src/trust/mod.rs` — TrustLevel + TrustRelation
- `src/trust/query_filter.rs` — walks filtered by trust
- UI: graph editing for trust definitions

**Phase 4 — Hybrid routing (שבוע 4):**
- `src/routing/mod.rs` — QueryPolicy + Route enum
- `src/routing/cloud_client.rs` — talks to zets-cloud when needed
- User-configurable via atoms

**Phase 5 — Federated learning (שבוע 5+):**
- `src/federation/mod.rs` — hash-sync protocol
- `zets-tools/federation_server/` — aggregation service
- Opt-in, default OFF

---

## העיקרון שחוזר על עצמו

**כל מה שלמעלה = atoms ו-edges בגרף.** זה הכוח של ZETS.

- License = atom
- Capability = atom + edge
- Trust relation = edge
- Privacy preference = atom
- Federation rule = atom
- Refusal log = atom

משתמש יכול **לראות** את המצב (graph viewer), **לעדכן** (edit atoms), **לחקור** (walk). **אין blackbox**.

---

## Python prototype — להוכחה

בנה `py_testers/test_licensing_trust_v1.py`:
- OEM license → immediate activation
- Trust space → child can't query adult content
- Hybrid routing → "medical" atoms stay local
- Federation → hash sync preserves privacy
- Offline mode → works without server

---

## Summary — שאלות עידן, תשובות

| שאלה | איך זה יעבוד |
|------|---------------|
| Pre-installed units (OEM) | License as signed atom in `installer.zets_enc`. No online required. |
| Regular install + optional activation | Local bootstrap → opt-in register for ticket_id + updates |
| Cloud-only / client / local | User choice via graph atoms. Hybrid routes per-query. |
| Hybrid (local + cloud + peers) | QueryPolicy decides route. Federation opt-in, hash-only |
| Learn from other AIs (incl. Claude) | Structured imports via authorized channels, audit-logged, confidence-weighted |
| Trust spaces like a child | Graph sub-structure; asker entity × subject × domain → filtered walk |
| Prevent privacy leaks | 3 layers: trust filter + privacy atom + audit log |

**אופטימלית לimplementation ב-phased Rust, אחרי אישור עידן.**

Git: commits from this session in `idaneldad/zets` branch main.
