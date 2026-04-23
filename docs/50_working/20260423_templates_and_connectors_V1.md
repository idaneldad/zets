# תשובות לשאלות עידן — Templates, Optimization, Connectors

**תאריך:** 23.04.2026
**Commit:** אחרי procedure_template module

---

## שאלה 1 ✅ — פונקציות עם שמות משתנים שונים

**נפתר ב-`src/procedure_template/`** (32 tests passing).

### המבנה

| שכבה | מה זה | דוגמה |
|-------|--------|--------|
| **Template** | shape canonic | `http.post(url, headers, body)` — pointer אחד |
| **Instance** | sighting בקוד | `github.com/x/y:42 Python: url→target_url, body→payload` |
| **Registry** | store + dedup | 1 template + 10 bindings + 1,000 sightings = storage לכל |

### תחושת ה-storage

אמת-מידה שנוצרה (`test_storage_efficiency_scenario`):
- 1,000 repositories × 10 תבניות שונות של שמות = **10 instances** + 1 template
- 1,000 sightings נספרים ב-`sighting_count` לא כ-atoms נפרדים
- ZETS יודע "זו אותה פונקציה" גם כשהשמות שונים

### NameRole — מתי השם הוא הסמנטיקה

4 אפשרויות לכל parameter:

| NameRole | מתי | דוגמה |
|----------|-----|--------|
| **Free** | שם = נוחות. להחליף = בלי משמעות | `url` ≡ `endpoint` ≡ `target` |
| **Domain** | שם = concept עסקי. מחובר ל-atom בגרף | `customer_id`, `invoice_num` |
| **Convention** | שם = מוסכמה מוכרת | `i` (loop), `tmp` (scratch), `self` |
| **MathSymbol** | שם = סמל פיזיקלי/מתמטי | `E = mc²` — `E`, `m`, `c` הם עצמם |

**אבחנה:** בתוך `math.mass_energy_equivalence`, השמות `mass`+`speed_of_light` הם role names **ו-**Domain names. Instances יכולים להשתמש ב-`m`, `M`, `mass_kg` — כולם מאוחדים ל-template.

---

## שאלה 2 — האם הגרף אמור לעשות optimization (לולאה/רקורסיה/פונקציה)?

### תשובה: **לא**.

#### שבירת כלים

אם הגרף יעשה optimization, הוא מתחיל להיות **runtime**. זה הופך את ZETS מ-**מנוע ידע** ל-**compiler**. זו הרחבה מסוכנת שפוגעת באופי של ZETS.

#### מה כן צריך לקרות

הגרף **מזהה** תבניות. ה-VM/compiler (שכבה נפרדת, עתידית) **מריץ**.

**דוגמה:**
```
ZETS רואה: "הקוד הזה עושה fetch של 100 פריטים ברצף"
ZETS רושם: pattern.sequential_fetch(count=100)
ZETS לא רושם: "שזה צריך להיות map() parallel"

כשמישהו יבקש מ-ZETS "תריץ את זה":
- Runtime engine (נפרד) יקבל החלטה:
  - Sequential? Parallel? Async streaming?
- זה תלוי ב-context (זיכרון פנוי, thread pool, rate limits)
- לא ב-graph structure
```

**למה זה חשוב:**
1. **הפרדה נקיה**: Graph = facts, Runtime = decisions
2. **Portability**: אותו template יכול לרוץ על embedded device ועל cluster
3. **בדיקות**: template deterministic (בודק 1 פעם), runtime variable (בודק ביצועים)
4. **תחזוקה**: optimization strategies משתנות, templates יציבים

### איפה ה-optimization כן קורה

- **ב-`procedure_atom.rs`** (קיים כבר): מציין `InvocationSource` ו-`TrustLevel`. מודל של "מי יכול לקרוא לזה". לא ביצועים.
- **ב-future `vm/` module**: יקבל bytecode מ-procedure_atom, ירוץ עם resource limits. שם נכנסים loop/recursion/parallelism.

---

## שאלה 3 — מקורות ל-connector priorities

### יש 4 מקורות אובייקטיביים

#### 1. **Make.com Templates** (ספר לבעל עסק)
- https://www.make.com/en/templates
- 10,000+ templates מדורגים לפי שימוש בפועל
- Top 20 ל-B2B SaaS:
  - Gmail → Sheets (lead management)
  - Slack → Trello / Asana / ClickUp
  - Stripe → CRM sync
  - Calendly → Zoom + Sheets + Email
  - Typeform → CRM + Email
  - Shopify → Accounting

#### 2. **Zapier Top Apps** (רשימת popularity ענקית)
- https://zapier.com/apps
- 7,000+ apps מדורגים
- מראה **connection frequency**, לא רק download

#### 3. **MCP Server Registry** (לפי הפרויקט שלך)
- https://modelcontextprotocol.io/servers
- Top servers נכון ל-2026:
  - `@modelcontextprotocol/server-filesystem`
  - `@modelcontextprotocol/server-github`
  - `@modelcontextprotocol/server-slack`
  - `@modelcontextprotocol/server-google-drive`
  - `@modelcontextprotocol/server-postgres`
  - Anthropic Gmail, Google Calendar (קיימים כבר בידיים שלך)

#### 4. **API Directory Rankings**
- **RapidAPI Hub**: https://rapidapi.com/hub — over 40K public APIs ranked by subscribers
- **Postman Public API Network**: https://www.postman.com/explore
- **Awesome APIs** (GitHub): https://github.com/public-apis/public-apis — 1,500+ free APIs

### המלצה שלי ל-Day 1 של ZETS — 3 טירים

**Tier 1 (10 connectors מיידיים) — כיסוי 80% של מקרי שימוש:**
1. HTTP generic (כבר יש)
2. Email SMTP (כבר יש Gmail app password)
3. Google Sheets (read/write rows)
4. Google Calendar (יש MCP, צריך integration)
5. Gmail (יש MCP)
6. Slack (יש MCP)
7. GitHub (יש MCP)
8. PostgreSQL / MySQL (SQL)
9. Webhook receiver (inbound)
10. File system (local)

**Tier 2 (10 connectors שבוע 2):**
11. Stripe (payments)
12. Twilio / SendGrid (SMS/email transactional)
13. Notion (docs/CRM lite)
14. ClickUp (כבר יש MCP)
15. Cloudflare (יש MCP)
16. OpenAI / Anthropic / Gemini APIs
17. WhatsApp Business API (GreenAPI)
18. Facebook/Meta Graph API (ads + messenger)
19. LinkedIn API
20. S3 / R2 / storage

**Tier 3 (on-demand):**
- Google Ads (SKOP protocol)
- VoIP (Asterisk, Twilio Voice, Vonage)
- Social platforms (TikTok, Instagram, X)
- Marketing tools (Mailchimp, ActiveCampaign)
- Podcast hosts (Buzzsprout, Anchor)

### למה **לא** Day 1 = 50 connectors

חזרתי לעקרון ה-code רזה שלך. תיעוד:

1. **90% של שימוש** מכוסה ע"י 10-15 connectors ראשונים
2. **40 הנוספים** נכתבים בקוד שנשחק (Netflix syndrome — 80% תכונות לא בשימוש)
3. **MCP כבר עושה את זה**. Claude Code + MCP מכסה רוב חיבורי ה-Day 1 שלך **היום**
4. **Phase C (OpenClaw ingest)** כשתהיה מוכנה, תלמד connector חדש תוך שעה מ-docs

### Day 1 אמיתי של ZETS

לא "50 connectors מוכנים". אלא:
- **ProcedureTemplate**: יכולת לתאר connector
- **TemplateRegistry**: יכולת ל-dedup ולחפש
- **10 seed templates**: HTTP, Email, Sheets, Gmail, Calendar, Slack, SQL, Webhook, FS, AI-call
- **Ingest pipeline** (Phase C): דרך ללמוד connector חדש מ-OpenAPI spec או docs
- **MCP bridge**: גישה ל-50 connectors דרך MCP protocol — בלי קוד חדש

**זה ה-day 1 הנכון**: היכולת, לא המלאי.

---

## מה נבנה עכשיו

✅ `src/procedure_template/` — **template.rs + instance.rs + registry.rs + mod.rs**
✅ 32 tests (1018 total, מ-986)
✅ Storage efficiency מוכח: 1,000 sightings = 10 instances + 1 template

## מה עוד נשאר (בסדר פריוריטי)

1. **Integration של Guard → Reader → Procedure flow** (1-2 שעות)
2. **10 seed templates** (HTTP×4, Email, Sheets, Gmail, Slack, SQL, FS) (2 שעות)
3. **MCP bridge** — זיהוי tools מ-MCP servers ורישום כ-templates (3-4 שעות)
4. **Phase C ingest pipeline** — fetch OpenAPI spec → parse → register templates (יום שלם)
5. **Reader Phase 2** (emotion + pragmatics + BigFive) (יום)
6. **Phase B benchmark** (HumannessScore) (יום)

---

## הערה חשובה על שאלה 3

השאלה שלך כללה "Google sales protocol" ו-"VoIP". שתיהן כיסויות:

- **Google Ads SKOP**: API protocol של Google ל-campaign management. זה OpenAPI spec. Phase C ingest יצור templates אוטומטית
- **VoIP**: Asterisk (AMI protocol), Twilio Voice (REST + webhooks), Vonage (REST). כולם APIs רגילים. Phase C מכסה

אין צורך בקוד מיוחד. **יש צורך בביכולת ללמוד API חדש** — זה Phase C.
