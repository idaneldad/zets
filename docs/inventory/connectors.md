# 🔌 חיבורים — Connectors Inventory

**Last updated:** 23.04.2026
**Last verified:** 23.04.2026 (against `src/connectors/seed.rs`)

---

## מה המשימה

ZETS צריך להיות מסוגל להתחבר לכלים חיצוניים: שליחת emails, קריאת calendar, שליחת הודעות ל-WhatsApp, עדכון Google Sheets, וכו'. זה קריטי ל-Tier B Category 13 (Task Orchestration).

## איך תיחשב הצלחה

- **Definition** — יש procedure template מוגדר לפעולה (e.g. `send_message`)
- **Registration** — הbundle רשום ב-`ConnectorRegistry`
- **Runtime execution** — הpipeline באמת מבצע קריאת HTTP ומקבל תגובה
- **Authentication** — מפתחות/tokens נטענים מה-vault (לא מ-.env)
- **Error handling** — failures מתועדים ב-ErrorStore
- **E2E test** — test שבודק integration מלא (mock server)

### 4 רמות הצלחה

| רמה | מה יש | דוגמה |
|-----|--------|--------|
| L1: Defined | ProcedureTemplate + bundle | `gmail_bundle()` |
| L2: Registered | ב-TemplateRegistry | שאיל: "איזה procedure לשליחת email?" |
| L3: Callable | פונקציה callable, lib wired | `registry.call("gmail.send", ...)` |
| L4: Tested E2E | ריצה מול mock server | integration test עובר |
| L5: Production | ריצה מול שירות אמיתי עם rate limits | deployed |

---

## 📊 טבלת חיבורים — מצב אמיתי

| # | Connector | L1 Def | L2 Reg | L3 Callable | L4 E2E | L5 Prod | באחריות | נבדק אחרון |
|---|-----------|:------:|:------:|:-----------:|:------:|:-------:|---------|:-----------:|
| 1 | Gmail | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 2 | Google Calendar | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 3 | Google Drive | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 4 | Google Sheets | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 5 | Slack | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 6 | Telegram | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 7 | WhatsApp (GreenAPI) | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 8 | SMTP (email generic) | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |
| 9 | OpenAI API | ✅ | ✅ | ❌ | ❌ | ❌ | חיצוני | 23.04 |

**סיכום:** 9/9 defined, 9/9 registered. **0/9 callable**. **0/9 tested E2E**. **0/9 in production**.

---

## ⚠️ הפער האמיתי — ההבדל בין "מוגדר" ל"עובד"

כל 9 ה-bundles הם **ProcedureTemplate definitions**. הם מתארים:
- איזה steps ההפעלה כוללת
- אילו parameters נדרשים
- אילו secrets (מהvault)
- מה ה-output expected

**מה שחסר:** ה-**runtime** שמבצע את זה. אין קוד שקורא ל-`reqwest::post(...)` או `tokio::net::TcpStream`.

### למה זה חשוב

אם משתמש מבקש "שלח לי email עם דוח היום":
1. ✅ Reader מזהה intent
2. ✅ ProcedureTemplate "send_email" נמצא
3. ✅ Binding של slots נבנה
4. ❌ **נתקע כאן** — אין שכבת executor

### מה צריך כדי להזיז את זה

**CapabilityOrchestrator module** (עתיד, `src/orchestrator/` או `src/capability_runtime/`):
- קריאה ל-procedure → לקיחת bundle → הפעלת HTTP
- Retrieval של secrets מ-vault
- Timeout + retry + rate limit
- Error → ErrorStore
- Audit → graph atoms

זה 2-3 ימי עבודה של קוד. **לא בוצע עדיין.**

---

## 📚 מפת החיבורים — מה יש ב-`seed.rs`

### Gmail Bundle (`gmail_bundle()`)
- Procedure: `send_email`
  - Slots: to (email), subject (text), body (text)
  - Secrets: `{owner}/oauth/gmail`
  - API: Gmail REST v1 `users.messages.send`
- Procedure: `read_inbox`
- Procedure: `search_messages`

### Google Calendar Bundle
- Procedure: `list_events`, `create_event`, `update_event`, `delete_event`

### Google Drive Bundle
- Procedure: `list_files`, `upload_file`, `download_file`

### Google Sheets Bundle
- Procedure: `read_range`, `append_row`, `update_cell`

### Slack Bundle
- Procedure: `post_message`, `list_channels`, `search`
- Auth: Bot token (xoxb-)

### Telegram Bundle
- Procedure: `send_message`, `send_photo`, `get_updates`
- Auth: Bot token

### WhatsApp Bundle (GreenAPI)
- Procedure: `send_message`, `send_file`, `get_chat_history`
- Auth: instanceId + token (from GreenAPI account)

### SMTP Bundle
- Procedure: `send_email` (generic)
- Auth: SMTP credentials (host/port/user/pass)
- **זמין כ-fallback** — עידן מחזיק credentials ל-info@pirsum10.co.il

### OpenAI Bundle
- Procedure: `chat_completion`, `embeddings`, `images_generate`
- Auth: API key

---

## 📋 חיבורי MCP — מצב שונה

MCP connectors הם לא "bundles" שלנו אלא **protocols** ש-ZETS לקוח (כלקוח MCP).

המצב: ZETS מפעיל MCP server משלו (port 3145), אבל **לא** מפעיל MCP clients לשרתים אחרים. כלומר, Claude/Claude Code יכולים לדבר עם ZETS, אבל ZETS לא עדיין קורא ל-MCP servers אחרים.

**פוטנציאל:** להתחבר ל-MCP Registry של Anthropic ולצרוך filesystem, GitHub, Slack, Postgres servers שם.

---

## 🎯 צפי השלמה

| שלב | מה | זמן משוער | גיין ל-HumannessScore |
|-----|-----|:---------:|:--------------------:|
| L3 (Callable) | CapabilityOrchestrator + HTTP executor | 2-3 ימים | +0.03 |
| L4 (E2E tested) | Mock server + integration tests per connector | 3-4 ימים | +0.02 |
| L5 (Production) | Gmail + Calendar ראשונים מול API אמיתי | 1-2 ימים | +0.01 |
| MCP client | לצרוך MCP servers חיצוניים | 2-3 ימים | +0.02 |

---

## בדיקות QA + TEST

| Test | סוג | מה נבדק | נבדק אחרון | סטטוס |
|------|:---:|---------|:-----------:|:-----:|
| connectors/seed tests | QA | bundles מוגדרים נכון | 23.04 | ✅ 40 tests |
| connectors/registry tests | QA | registration + lookup | 23.04 | ✅ |
| (missing) | TEST | latency HTTP call | — | 🔴 אין runtime |
| (missing) | TEST | retry on 5xx | — | 🔴 אין runtime |
| (missing) | TEST | rate limit respect | — | 🔴 אין runtime |
| (missing) | QA | auth token retrieval from vault | — | 🔴 חסר integration |

---

## היסטוריית שינויים (במסמך זה)

| תאריך | שינוי |
|:-----:|-------|
| 23.04.2026 | Audit ראשון. תיקון הtclaim שקיימים 9 חיבורים עובדים — בפועל 9 definitions ו-0 runtime |
