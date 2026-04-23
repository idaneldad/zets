# Multi-Interface Architecture — GUI + CLI + API + MCP

**תאריך:** 23.04.2026
**בקשת עידן:** "כל client וserver — gui מקומי + web responsive + exe/app +
CLI + API + MCP. API ו-MCP עם מגבלות לפי רישיון והמחמיר מבין רישיון ולקוח."

---

## תשובה קצרה: כן, הגיוני, אפשרי, ויש דרך אחת נכונה

**עקרון מרכזי:** ONE binary (`zets_node`), FOUR thin adapters. מדיניות מרכזית
(license × customer) נאכפת פעם אחת, כל הממשקים מחייבים אותה.

---

## המציאות כיום (בדיקה חיה)

- ✅ קיים `zets-gui/dist/index.html` — 11KB vanilla JS, WhatsApp-style, dark theme, RTL Hebrew
- ✅ קיימים endpoints `/api/*` ב-`mcp/zets_http_api.py` (Python wrapper)
- ✅ קיים MCP server ב-`mcp/zets_mcp_server.py` (כתובת `:3144`)
- ❌ אין CLI מאוחד (כל binary Rust = CLI נפרד)
- ❌ אין licensing enforcement בכלל
- ❌ אין shared rate limiter בין interfaces

---

## הארכיטקטורה המוצעת

```
                ┌─────────────────────────────────┐
                │    ZetsNode (Rust, one binary)   │
                │                                  │
                │   ┌───────────────────────────┐  │
                │   │   Policy Gate             │  │
                │   │   license × customer      │  │
                │   │   rate limit (per key)    │  │
                │   │   audit log               │  │
                │   └──────────┬────────────────┘  │
                │              │                   │
                │   ┌──────────▼────────────────┐  │
                │   │   Business Logic          │  │
                │   │   query/teach/export/...  │  │
                │   │   graph, inference, sync  │  │
                │   └───────────────────────────┘  │
                └──────────────┬──────────────────┘
                               │
       ┌───────────────┬───────┼───────┬────────────────┐
       │               │       │       │                │
  ┌────▼────┐   ┌──────▼───┐ ┌─▼───┐ ┌─▼─────┐    ┌────▼────┐
  │ Web GUI │   │   CLI    │ │ API │ │  MCP  │    │ Desktop │
  │(browser)│   │(terminal)│ │HTTP │ │(SSE)  │    │(WebView)│
  └─────────┘   └──────────┘ └─────┘ └───────┘    └─────────┘
```

**4 adapters, business logic אחת.**

---

## כל interface בפירוט

### 1. Web GUI (localhost)

- **Served from:** אותו binary, `--port=3147 --gui` → serves `/gui/*` static
- **Auth:** session cookie (local). אין צורך ב-API key
- **Technology:** vanilla JS, responsive, RTL Hebrew, dark mode (כמו שיש)
- **Rate limit:** כן — אותו bucket של המשתמש
- **URL:** `http://localhost:3147/gui/`

**למה static vanilla JS ולא React/Vue?**
- 11KB (React: 200KB+)
- אפס build step
- עובד בכל דפדפן, ישר מ-Rust process
- אין dependencies לחדש, אין vulnerability updates

### 2. CLI (terminal)

```bash
zets_node query "מה זה Python?"
zets_node teach "Python is created by Guido"
zets_node status
zets_node gui        # launches local browser
zets_node serve      # runs as daemon
```

- **Auth:** none (local process, runs as same user)
- **Rate limit:** כן — אותו bucket
- **Implementation:** `src/bin/zets_node.rs` עם `clap` args

### 3. HTTP API (public)

```
POST /api/v1/query      { "q": "..." }
POST /api/v1/teach      { "fact": "..." }
POST /api/v1/export     { "format": "jsonl" }
GET  /api/v1/health
GET  /api/v1/stats
```

- **Auth:** `X-API-Key: <key>` על כל endpoint בtier ≥ personal
- **Rate limit:** הכל בsame bucket שלindex
- **Headers responses:**
  - `X-RateLimit-Remaining: N`
  - `X-RateLimit-Limit: N`
  - `X-Tier: free|personal|pro|enterprise`
- **Errors:** 401 (no key), 403 (action not allowed), 429 (rate limit)

### 4. MCP (for Claude / other LLMs)

```
SSE endpoint: /mcp/sse
Tools: zets_query, zets_teach, zets_export, zets_stats
```

- **Auth:** Bearer token או `X-API-Key` (same mechanism)
- **Rate limit:** **same bucket as HTTP API** — משתמש לא יוכל לעקוף
- **Tools exposed לפי tier:**
  - free: zets_query בלבד
  - personal: + zets_teach
  - pro: + zets_export, + zets_stats
  - enterprise: הכל

### 5. Desktop app (exe/mac/linux app)

**לא עוד process חדש.** זו אריזה של WebGUI + embedded zets_node:

- **Linux:** AppImage או `.deb`/`.rpm`. זה זz node + 11KB HTML + Electron/Tauri wrapper
- **Mac:** `.app` בundle. zets_node binary + HTML.
- **Windows:** `.exe`. זz node + HTML.

**הדרך הכי יעילה:** **Tauri** (Rust-native, 5MB bundle, משתמש ב-system webview)
- VS Electron (150MB+ לכל app, ships כל כרום)
- Tauri משתמש בWebview2 (Windows), WebKit (Mac), WebKitGTK (Linux)

---

## Licensing — enforcement חכם

### המודל — הוכח בPython prototype (py_testers/test_multi_interface_v1.py)

**4 tiers:**

| Tier | Queries/hour | Teach | Export | Offline packs | Custom domains | API key? |
|------|--------------|-------|--------|----------------|-----------------|----------|
| free | 10 | ❌ | ❌ | ❌ | ❌ | ❌ |
| personal | 1,000 | ✅ | ❌ | ✅ | ❌ | ✅ |
| pro | 100,000 | ✅ | ✅ | ✅ | ✅ | ✅ |
| enterprise | ∞ | ✅ | ✅ | ✅ | ✅ | ✅ |

### CustomerOverride — המחמיר תמיד ינצח

**עידן שאל:** "API ו-MCP צריך מגבלות לפי רישיון ולפי רצון הלקוח המחמירה מבינהם."

```python
effective_limit = min(license_tier_limit, customer_chosen_limit)
```

**דוגמאות אמיתיות:**

| License | Customer wants | Effective |
|---------|-----------------|-----------|
| pro (100K/h) | 100/h | **100/h** (customer stricter) |
| pro (100K/h) | 500K/h | **100K/h** (license caps) |
| free (10/h) | 10K/h | **10/h** (license caps) |
| enterprise (∞) | 1000/h | **1000/h** (customer cap wins) |

**גם action-level:**
- License allows `teach` + customer sets `allow_teach=False` → denied
- License blocks `export` + customer wants `export=True` → still denied (license wins)

**זה הוכיח בprototype:** test #3, #4 — stricter תמיד ינצח.

### Rate limit — **bucket משותף בין כל 4 הinterfaces**

קריטי:

```python
rate_key = api_key or "local"  # NOT per-interface
```

אחרת: משתמש עם free tier יכול לשלוח 10 ב-API, 10 ב-CLI, 10 ב-MCP = 30 בפועל. עוקף.

**Prototype test #5 הוכיח:** 10 queries ב-mix של CLI+API+MCP → 11th 429, ללא קשר ל-interface.

### API key management

- Free tier: לא נדרש (rate limit לפי IP אבל פחות חמור)
- Paid tiers: **חובה**. Key stored ב-zets_node config. Revoke-able, rotate-able
- Enterprise: multi-key (team sharing)

---

## Prototype results (py_testers/test_multi_interface_v1.py — 200 lines, 8/8 tests)

```
[1] Free tier enforces 10 queries/hour limit:
  ✓ free tier 11th query → 429 (rate_limit)

[2] Free tier blocks /teach action:
  ✓ free tier /teach → 403 (teach_not_allowed_in_tier)

[3] Customer override STRICTER than license wins:
  license=pro (100000/hr), customer wants 100/hr
  ✓ stricter of (pro=100000, custom=100) = 100

[4] Customer CANNOT exceed license:
  license=free (10/hr), customer asks for 10000/hr
  ✓ customer cannot exceed license (10)

[5] CLI + API + MCP share SAME rate limit bucket:
  after 10 mixed → 11th on API → 429
                 → 12th on MCP → error=rate_limit

[6] API key required on paid tiers:
  ✓ personal tier without api_key → 401
  ✓ personal tier WITH api_key → 200

[7] Enterprise unlimited but audited:
  ✓ 50 queries → all 200, audit log = 50 entries

[8] Customer child_safe override:
  ✓ (flag propagates to Rust for register ≥ 5 filter)
```

---

## מבנה הפרויקט המוצע

### Repo: `zets` (pure Rust)
```
src/
├── policy/                    ← חדש
│   ├── mod.rs
│   ├── license.rs             ← Tier enum, TierLimits struct
│   ├── customer.rs            ← CustomerOverride
│   └── rate_limiter.rs
├── adapters/                  ← חדש
│   ├── mod.rs
│   ├── gui.rs                 ← serves static GUI + session auth
│   ├── cli.rs                 ← clap-based CLI
│   ├── http_api.rs            ← REST /api/v1/*
│   └── mcp.rs                 ← SSE /mcp endpoint
├── bin/
│   └── zets_node.rs           ← unified binary using all adapters
└── ... (existing src/)
```

### Bundled with binary:
```
zets_node/
├── bin/zets_node              ← single Rust binary
├── gui/index.html             ← 11KB static (from zets-gui/dist/)
├── gui/assets/                ← images, fonts
└── config/default.toml        ← default config
```

### Desktop app (Tauri):
```
zets-desktop/
├── src-tauri/                 ← Tauri wrapper (Rust)
│   ├── src/main.rs            ← spawns zets_node in-process
│   └── tauri.conf.json
├── dist/index.html            ← same GUI as local web
└── icons/
```

---

## Summary — answers לשאלות של עידן

| שאלה | תשובה |
|------|--------|
| הגיוני GUI + CLI + API + MCP? | **כן** — one binary, 4 thin adapters |
| אפשרי? | **כן** — Python prototype הוכיח 8/8 tests |
| איך הכי יעיל? | **Tauri** לdesktop (5MB), embedded vanilla JS לgui, SSE לMCP |
| API+MCP מגבלות לפי רישיון? | **כן** — policy gate לפני כל request |
| לפי לקוח? | **כן** — CustomerOverride struct |
| המחמיר מנצח? | **כן** — `min(license_limit, customer_limit)` |

**לא יישום Rust עדיין.** מחכה לאישורך בהתאם ל-CLAUDE_RULES §4.
