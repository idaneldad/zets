# OpenClaw Deep Analysis — What ZETS Has, What ZETS Needs

**תאריך:** 23.04.2026  
**מקור:** YouTube "מה זה OPENCLAW ולמה גם אתם חייבים לעבור אליו?" (Benny Farber) + github.com/openclaw/openclaw (362K stars)

---

## חלק 1 — מה OpenClaw עושה (מהקוד)

### סקלה
| מדד | ערך |
|-----|------|
| Stars | 362,574 |
| Forks | 74,073 |
| Updated | 2026-04-23 (היום) |
| שפה | TypeScript / Node 24 |
| Sponsors | OpenAI, GitHub, NVIDIA, Vercel, Blacksmith, Convex |
| Extensions | 117 (plugins: anthropic/openai/gemini/groq/discord/whatsapp...) |
| Skills | 53 (1password/apple-notes/bear/github/gemini/canvas/...) |
| Channels | 82 (WhatsApp/Telegram/Slack/Discord/iMessage/Matrix/...) |
| Agents | 826 files under src/agents/ |

### הארכיטקטורה ברמת המפתח

```
User message via channel (WhatsApp/Telegram/Slack/...)
         ↓
┌────────── Gateway (single control plane, local-first) ───────────┐
│                                                                    │
│  Channel binding → Agent routing → Session → Context Engine       │
│                                          ↓                         │
│                             Prompt built with:                    │
│                              - <available_skills> XML block      │
│                              - active memory (recall)             │
│                              - transcript + tool history          │
│                                          ↓                         │
│                                   LLM (30+ providers)              │
│                                          ↓                         │
│                         Tool call → Skill → Tool execution        │
│                              ↓                                    │
│                       Sandbox (main=native, non-main=Docker/SSH)  │
└────────────────────────────────────────────────────────────────────┘
         ↓
Response delivered to channel
```

### השלד של skill (הcontract שלהם)

כל skill = קובץ `SKILL.md` עם YAML frontmatter + Markdown body:

```yaml
---
name: github
description: "GitHub ops via gh CLI: issues, PRs, CI runs..."
metadata:
  openclaw:
    emoji: 🐙
    requires:
      bins: ["gh"]
    install:
      - id: brew
        kind: brew
        formula: gh
        bins: ["gh"]
allowed-tools: ["message", "bash", "read"]
---

# GitHub Skill

## When to Use
✅ Checking PR status...

## When NOT to Use
❌ Local git operations → use git directly

## Setup
...
```

**זה מטא-פרוצדורה**: description + preconditions (requires) + install procedure + allowed-tools permission + when/not-when context hints.

### המנגנון של permissions — 5 שכבות

1. **DM pairing** (default): שולח לא מוכר מקבל pairing code, ההודעה **לא מעובדת** עד אישור
   ```typescript
   dmPolicy: "pairing"  // unknown senders get pairing code, bot does not process message
   allowFrom: ["+1234567890"]  // allowlist
   ```

2. **Channel action gating**: פעולות רגישות כבויות כברירת מחדל
   ```typescript
   channels.discord.actions.roles: false       // default OFF
   channels.discord.actions.moderation: false  // default OFF
   channels.discord.actions.presence: false    // default OFF
   channels.discord.actions.channels: false    // default OFF
   ```

3. **Allowed-tools per skill**: כל skill מוגבל לtools שהוכרזו
   ```yaml
   allowed-tools: ["message"]  # discord skill can ONLY send messages
   ```

4. **Sandbox mode**: non-main sessions רצים ב-Docker sandbox
   ```typescript
   agents.defaults.sandbox.mode: "non-main"
   // backends: docker (default) / ssh / openshell
   ```

5. **Sandbox image** (Dockerfile.sandbox):
   - Debian bookworm-slim, `useradd sandbox`, `USER sandbox`
   - רק `bash/ca-certificates/curl/git/jq/python3/ripgrep` — minimal
   - אין root, אין network by default

### איך הוא משתפר? (השאלה שלך)

**4 מנגנונים שזיהיתי בקוד:**

1. **Active Memory extension** (`extensions/active-memory/`)
   - שומר session transcripts
   - recall ב-sessions עתידיים
   - cache TTL=15s, max 1000 entries
   - Search mode: "qmd_search" — query-mode-descriptor
   - זו למידה **לא-בעיקר-LLM**: שמירת זכרונות מתומצתים, retrieval על בסיס relevance

2. **Dynamic skill installation** (`src/agents/skills-install-download.ts`)
   - skills נטענים runtime — **לא צריך restart**
   - install spec: brew/apt/npm/go/uv
   - `fetchWithSsrFGuard` — SSRF protection
   - pinned writes (לא overwrite of existing files)

3. **ClawHub** (`clawhub.ai` — skill marketplace, חיצוני)
   - skills publish שם first
   - community plugin listing
   - bar גבוה להוספת skill לcore

4. **MCP via mcporter** (`github.com/steipete/mcporter`)
   - add/change MCP servers **without restart**
   - keeps core lean

**הערה קריטית:** OpenClaw **לא** לומד ידע סמנטי מהעולם — הוא לומד **אילו skills קיימים** ו**אילו זיכרונות רלוונטיים לsession**. הidea של "learn from 17M articles" שלנו זה משהו שלהם אין.

---

## חלק 2 — מה יש לנו (ZETS) ומה אין

### יש לנו ≠ להם

| יכולת | ZETS | OpenClaw |
|-------|------|----------|
| Knowledge graph (17.5M articles) | ✅ | ❌ |
| Sense graph (שלום≠hello) | ✅ | ❌ |
| 16 CognitiveKinds | ✅ | ❌ |
| BitflagRelation (6 axes) | ✅ | ❌ |
| BPE+Merkle fold (lossless) | ✅ | ❌ |
| mtreemap (98% cache hit) | ✅ | ❌ |
| system_graph VM (32 opcodes) | ✅ | ❌ |
| Mmap-based, 80MB RAM | ✅ | ❌ |
| Multi-language (48) | ✅ | Partial (via LLM) |
| Kabbalistic cognitive model | ✅ | ❌ |

### יש להם ≠ לנו

| יכולת | ZETS | OpenClaw |
|-------|------|----------|
| Channel integrations (WhatsApp etc) | ❌ | 82 |
| Skill registry (procedure marketplace) | 4 routes | 53 skills + ClawHub |
| LLM provider plugins (30+) | 2 (Gemini/OpenAI) | 30+ |
| Dynamic skill install | ❌ | ✅ |
| DM pairing / allowlist | ❌ | ✅ |
| allowed-tools per procedure | ❌ | ✅ |
| Docker sandbox per session | Partial | ✅ |
| Multi-agent routing by channel | Static 16 | Dynamic 826 |
| MCP hotswap | ❌ | ✅ via mcporter |
| Mobile apps (iOS/Android) | ❌ | ✅ |
| Canvas UI | ❌ | ✅ |
| Voice wake + TTS | ❌ | ✅ |

### ההשוואה המעמיקה

**OpenClaw = agent framework.** יודע להעביר הודעות לערוצים שונים ולקרוא לtools. **מוחו = הLLM**. Skills מכוונים את הLLM.

**ZETS = brain.** יודע, מבין, זוכר, לומד מ-17.5M articles. **אין לו gateway לערוצים, אין לו dynamic skill installer, אין לו permission model**.

**השילוב הנכון:** ZETS brain + OpenClaw patterns = מוצר שלאף אחד אין.

---

## חלק 3 — מה להביא לZETS מ-OpenClaw

### 3.1 Procedure Registry (הדבר הראשון)

**היום ב-ZETS:** יש `src/system_graph/routes.rs` עם 4 routes hardcoded. זה לא מספיק.

**הצעה:** Procedure = atom בגרף + Markdown-style metadata + bytecode.

```rust
pub struct ProcedureAtom {
    pub id: AtomId,
    pub name: String,                    // "send_whatsapp_via_greenapi"
    pub description: String,              // "Send a WhatsApp message via GreenAPI"
    pub when_to_use: Vec<String>,         // sense keys that match this procedure
    pub when_not_to_use: Vec<String>,
    pub allowed_tools: Vec<ToolId>,       // ← מודל ההרשאות של OpenClaw
    pub required_permissions: Vec<PermissionId>,
    pub preconditions: GraphQuery,
    pub postconditions: GraphQuery,
    pub install: Option<InstallSpec>,     // optional setup
    pub steps: Vec<ProcedureStep>,        // DAG
    pub trust_level: TrustLevel,          // System/Verified/Learned/Experimental
    pub owner: Option<AtomId>,            // who added this
    pub version: u32,
}
```

**כמה כזה פרוצדורה:** יש על הגרף. ZETS לומדת procedures מ-docs (תכנית פרוצדורה→DAG), שומרת כ-atoms, מחפשת אותן לפי sense matching.

### 3.2 Permission Layer (התשובה לשאלת ההרשאות שלך)

**5 שכבות — מותאם מ-OpenClaw:**

**שכבה 1 — Identity Pairing:**
```rust
pub enum IdentityTrust {
    Owner,              // עידן — full access
    Paired(PeerId),     // אושר via pairing code
    Unknown,            // לא מעובד עד pairing
    Blocked,            // חסום פעיל
}
```

כל הודעה נכנסת מערוץ חיצוני → check `identity_trust(sender)`. Unknown → מחזיר pairing code, לא מעבד את ההודעה.

**שכבה 2 — Channel action gating:**
```rust
pub struct ChannelPolicy {
    pub channel: String,            // "whatsapp"
    pub actions: HashMap<String, bool>,  // "send": true, "forward": false
    pub allow_from: Vec<PeerId>,   // allowlist
    pub rate_limit: RateLimit,
}
```

**שכבה 3 — Procedure allowed-tools:**
כל procedure מכריזה `allowed_tools`. ה-VM בודק לפני כל `CallTool` opcode שהtool ב-allowed list.

**שכבה 4 — Trust-level execution:**
```rust
pub enum TrustLevel {
    System,          // hardcoded in Rust, highest
    OwnerVerified,   // owner approved this procedure explicitly
    Learned,         // extracted from corpus, sandbox-only until verified
    Experimental,    // simulation-only, no real execution
}
```
- `System` + `OwnerVerified` → direct execution
- `Learned` → runs in sandbox transaction, commits on success
- `Experimental` → simulation mode only (ZETS returns what WOULD happen, doesn't do it)

**שכבה 5 — Sandbox:**
- Docker sandbox for `Learned` procedures that need network/filesystem
- VM-only sandbox for pure graph-walks (`ConceptLookup`, `EdgeTraverse`)
- Rate limit per procedure per user

### 3.3 WhatsApp/GreenAPI example — פרוצדורות כמסלולים של מסלולים

**זה בדיוק מה שאתה תיארת.** User says: "תשלח לדוד שהפגישה נדחתה ל-11"

```
Graph walk:
  
  intent_parse(text)
    → SEND_MESSAGE(recipient="דוד", content="הפגישה נדחתה ל-11")
    
  find_procedure(SEND_MESSAGE, user_context={channel: "whatsapp"})
    → procedure_atom: "send_whatsapp_via_greenapi"

  execute(send_whatsapp_via_greenapi, args):
    Step 1: resolve_recipient("דוד")
      → sub-procedure: lookup_contact(name)
          → graph query: atom[name:"דוד", kind:Person] → phone_number
    
    Step 2: check_permission(send_whatsapp, sender=Owner, recipient=David)
      → policy check: Owner→anyone: ALLOWED
    
    Step 3: compose_message(content="הפגישה נדחתה ל-11")
      → sub-procedure: hebrew_text_normalize
          → sub-procedure: unicode_nfc → strip_zero_width
    
    Step 4: greenapi_send(phone, content)
      → sub-procedure: http_post
          → sub-procedure: build_request
          → sub-procedure: fetch_with_ssrf_guard
          → sub-procedure: parse_json_response
    
    Step 5: log_sent(message_id, recipient, content)
      → graph write: new atom[kind:SentMessage] → edge(OWNER→sent)
```

**כל step הוא atom בגרף שמפנה ל-procedure אחר. הכל מסלולים של מסלולים.**

הפעם הבאה שצריך לשלוח WhatsApp — אותם steps. הפעם הבאה שצריך לשלוח SMS — אותו `Step 4` נקרא אחרת (`sms_send` במקום `greenapi_send`), אבל `Step 3` (compose_message) ו-`Step 5` (log_sent) זהים. **Reusability ברמת ה-step.**

ואם ZETS לומדת procedure חדש מדוק ("כדי לשלוח לLINE, קרא ל-LINE API...") — procedure חדש נוצר עם `Step 4: line_send`, ה-3 שלבים האחרים **reuse**.

### 3.4 Client-specific learning

**היום:** ZETS יש לו מוח אחד, 16 personas אבל static.

**הצעה:** 
```rust
pub struct ClientScope {
    pub client_id: ClientId,
    pub private_graph: ScopedGraph,     // atoms שנראים רק ללקוח זה
    pub shared_graph: &SharedGraph,     // atoms משותפים (Wikipedia, procedures)
    pub learned_procedures: Vec<ProcedureId>,  // procedures שנלמדו ספציפית לו
    pub allowed_procedures: HashSet<ProcedureId>,  // מה מותר לו להפעיל
}
```

כל client:
- רואה את הshared brain (wikipedia, פרוצדורות system)
- לומד procedures פרטיים (שמות של אנשי קשר, העדפות)
- מוגבל ל-procedures שmoutar לו

**Self-improvement per client:** ZETS מזהה pattern של שאילתות חוזרות של לקוח → מציע procedure חדש → אחרי אישור owner → נוסף ל-`learned_procedures`.

---

## חלק 4 — תוכנית יישום

### פאזה A — Permission Model (שבוע 1)
1. `src/permission/identity.rs` — IdentityTrust + pairing
2. `src/permission/channel_policy.rs` — ChannelPolicy + action gating
3. `src/permission/trust_level.rs` — TrustLevel enum + dispatch
4. Integration לsystem_graph VM: opcode `CheckPermission(proc_id) = 55`
5. Tests: 30+ unit tests

### פאזה B — Procedure Registry (שבוע 2)
1. `src/procedure/atom.rs` — ProcedureAtom struct
2. `src/procedure/manifest.rs` — Markdown+YAML parser לSKILL.md
3. `src/procedure/store.rs` — ProcedureStore (list/add/find/call)
4. `src/procedure/dispatch.rs` — match intent → find procedure → execute
5. Bootstrap: 10 seed procedures (send_whatsapp, http_get, hebrew_normalize, ...)

### פאזה C — Client Scoping (שבוע 3)
1. `src/scope/client.rs` — ClientScope
2. `src/scope/graph_scoping.rs` — atom visibility rules
3. Integration לsearch — respect client scope
4. Tests: client A doesn't see client B atoms

### פאזה D — Dynamic skill install (שבוע 4)
1. `src/procedure/install.rs` — InstallSpec (brew/apt/npm/cargo)
2. `src/procedure/loader.rs` — runtime load without restart
3. Registry fetch from remote (ClawHub equivalent for ZETS)
4. SSRF guard, pinned writes, deny-by-default

---

## חלק 5 — מה **לא** להביא מ-OpenClaw

1. **TypeScript/Node** — אנחנו Rust. שומרים את המערכת רזה.
2. **LLM-centric design** — ZETS brain > LLM. LLM הוא generator-of-last-resort, לא ה-core.
3. **30+ LLM provider plugins** — overkill. 2-3 מספיקים.
4. **Mobile apps** — לא בעדיפות עכשיו.
5. **Canvas UI** — לא עכשיו.
6. **Voice wake** — לא עכשיו.

---

## חלק 6 — ההחלטות שחייבות עכשיו

### החלטה 1: איפה הcode של procedure יושב?
- **אופציה A:** Rust בלבד, procedures = bytecode בגרף
- **אופציה B:** Markdown SKILL.md + JSON manifest (כמו OpenClaw), Rust מטען
- **אופציה C:** Hybrid — manifest + bytecode for atomic ops, sub-graph for composition

**המלצה:** **C — Hybrid**. Manifest (Markdown YAML) לdescription/permissions/when-to-use. Bytecode ל-primitive ops. DAG של procedure_atoms לcomposition.

### החלטה 2: איפה פרוצדורות נשמרות?
- `data/procedures/system/` — system procedures (Rust-hardcoded skeleton + Markdown doc)
- `data/procedures/learned/` — learned from corpus
- `data/procedures/owner/` — owner-defined
- `data/procedures/client_<id>/` — per-client learned

### החלטה 3: מי מוסיף procedures?
- **System:** hardcoded + bootstrap.rs
- **Owner (עידן):** דרך CLI `zets procedure add path/to/SKILL.md`
- **Learned:** auto-extracted, stays in `Learned` trust level עד אישור
- **NEVER from untrusted channel input** — שכבת DMpairing חוסמת מראש

---

## סיכום

**שלושה דברים ש-ZETS חייבת לאמץ מ-OpenClaw:**
1. **allowed-tools per procedure** (permission containment)
2. **DM pairing + allowlist** (identity trust)
3. **Dynamic procedure install** (skills hotswap)

**שלושה דברים ש-ZETS **לא** צריכה לאמץ:**
1. LLM-as-brain — אנחנו המוח
2. TypeScript — אנחנו Rust
3. 30+ providers — 2-3 מספיק

**הarchitectural insight המרכזי שלך:**
> "כל המערכת תהיה מסלולים של מסלולים"

זה לא metaphor. זה בדיוק מה שעושים — procedures כ-atoms בגרף, כל procedure מכילה steps שכל אחד הוא atom, שכל אחד (אם לא primitive) הוא procedure אחרת. הזמן לבנות זה עכשיו.
