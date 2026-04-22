# Autonomous Learning — עיצוב עם שבירת כלים

**תאריך:** 23.04.2026
**בקשת עידן:** "לימוד אינטנסיבי אוטונומי לגמרי... טריליוני נתונים... חכמה...
רמות אמינות... dedup חד-חד ערכי"

---

## חשיבה ראשונה (לפני שבירת כלים)

האינסטינקט הראשון שלי:
1. בנה Rust crawler עם tokio
2. אלפי workers concurrent
3. scrape הכל
4. store במזיכרון

**זו טעות. כל הנחה כאן שבורה.**

---

## שבירת כלים #1 — "יותר נתונים = יותר ידע"

**השבר:** לא. יותר נתונים לרוב = יותר duplicates + noise + cognitive dissonance.

אם אני מוריד 1M articles על "climate change" מ-100 sources, ZETS לא **יודע** פי-100.
הוא רואה 100 נוסחאות שונות של אותן 20 עובדות מרכזיות. הגרף **גדל** אבל הידע
**לא גדל**.

**המסקנה החדשה:**
"טריליוני נתונים" = **טריליון observations של core facts, מקושרים דרך hash_registry.**
הגידול הוא בעוצמת-סיגנל (signal strength per fact), לא בכמות נפחים.

### תוצאה מעשית:
- אל תמדוד את ZETS ב-atom count. מדוד ב-**unique facts**.
- כל fact שנצפה מ-10 sources עצמאיים = confidence 0.95.
- כל fact שנצפה מ-source אחד = confidence 0.5.
- **לא** להוסיף את ה-fact 10 פעמים — להוסיף 9 **edges** של `corroborated_by`.

---

## שבירת כלים #2 — "מהיר = טוב"

**השבר:** להיפך. Crawler מהיר = crawler חסום.

Wikipedia חוסם ב-1 request/second מ-IP לא-רשום. Reddit דורש OAuth. CloudFlare
חוסם לפי fingerprint. Google חוסם headlessly. אם אני "יעיל" = אני **אנעל עצמי מחוץ**
לרוב האינטרנט השימושי תוך ימים.

**המסקנה החדשה:**
"crawler חכם" = **crawler איטי מוכוון**.
- 1 request per 3 seconds per domain (conservative)
- Round-robin בין ~100 domains → effective ~30 requests/second total (mostly idle)
- 30 req/sec × 86,400 sec/day = 2.5M requests/day  
- ב-corpus ממוצע: 2.5M articles/day ≈ 1B articles/year ב-domain אחד משמעותי

זה **מספיק**. לא צריך להעיף.

### תוצאה מעשית:
- **Politeness-first**, לא performance-first.
- Budget: 2-3 req/domain/sec מקס
- **Identify as bot**: `User-Agent: ZETS-Learner/0.1 (contact: idan@chooz.co.il)`
- **Respect robots.txt** — אם אסור, לא ללכת
- **Cache**: `If-Modified-Since` / ETag → לא להוריד שוב

---

## שבירת כלים #3 — "scrape HTML"

**השבר:** HTML הוא הדרך ה**גרועה ביותר** לקבל structured data.

דוגמה: רוצה לקרוא NYT article.
- HTML path: fetch page, run readability, extract text, parse entities → **בלתי-יציב**
- RSS path: `https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml` → מבנה XML יציב
- API path: חלק מהmedia outlets מציעים — יותר יציב, מסומן שימוש

**המסקנה החדשה:**
**סדר העדיפות:** MCP > API > RSS/Atom > Sitemap > HTML (last resort).

### מקורות structured שקיימים (selection):
- **Wikidata**: 110M+ items, JSON dumps חינם
- **Wikipedia**: API חינם, dump חודשי
- **ConceptNet**: 34M+ edges, download מלא
- **OpenAlex** (was: Microsoft Academic Graph): 250M+ papers, free API
- **PubMed**: 35M+ biomed papers, E-utilities API
- **arXiv**: 2M+ preprints, OAI-PMH API
- **GitHub GraphQL API**: code + issues + repos
- **Common Crawl**: petabyte-scale web, free monthly dumps
- **Reddit API**: OAuth required but high-signal
- **StackExchange API**: free, 10K req/day per key
- **HuggingFace datasets**: thousands of curated corpora
- **Project Gutenberg**: 70K books, free
- **Internet Archive**: free API, archives everything
- **RSS/Atom**: thousands of outlets (news, blogs, scholarly)
- **Anthropic MCP servers**: growing catalog

**לא צריך HTML scraping ברוב המקרים.**

---

## שבירת כלים #4 — "trust scoring = אני בוחר tier לאתר"

**השבר:** manual tiering לא scalable. ואני לא האובייקטיב.

לא בסדר: "אני חושב ש-BBC ≥ Al Jazeera" (bias פוליטי מוטמע).
במקום:
- **tier seeded from meta-signals**:
  - HTTPS + established age (WHOIS) + PageRank + citation incoming
  - Peer-review signals (arXiv, journals)
  - Domain category (.gov, .edu, .com, .info, newly-registered)
- **cross-corroboration** is the actual signal, not my judgment
- source-tier is a prior; **corroboration is the posterior**

**המסקנה החדשה:**
Trust = Bayesian.
- Prior: source tier (tech-measured, not opinionated)
- Evidence: cross-source corroboration, coherence with existing knowledge
- Posterior: per-fact confidence, updated as more sources arrive

### מודל prior (tiers A-F):
| Tier | Criterion | Example | Prior Confidence |
|------|-----------|---------|------------------|
| A | Peer-reviewed, structured | arXiv, PubMed, Nature | 0.90 |
| B | Reference/encyclopedic | Wikipedia, Wikidata, Britannica | 0.80 |
| C | Primary source, gov/edu | .gov, .edu, NASA, WHO | 0.75 |
| D | Established journalism | Reuters, AP, BBC, NYT | 0.70 |
| E | Industry/organizational | company blogs, foundations | 0.60 |
| F | Personal blog, social | Medium, Twitter, Reddit | 0.40 |
| X | Unknown/suspicious | new domains, parked, sketchy | 0.20 |

### Evidence updates:
- N sources in same tier agree → confidence stays or climbs modestly
- Sources in DIFFERENT tiers agree → confidence climbs significantly (independence)
- Sources disagree → split into 2 Hypothesis edges, mark `contradicts`
- Peer-reviewed contradicts blog → peer-reviewed wins, blog gets `Hypothesis` tag

---

## שבירת כלים #5 — "dedup = hash content"

**השבר:** content-hash אחד לא מספיק. דוגמאות:

- אותו ציטוט בעיצובים שונים: "I have a dream" בHTML כולל sur-quotes, vs כtext פשוט → hash שונה, **אותו ציטוט**
- Typos: "dog" vs "dogs" — 2 hashes, 1 concept
- Translation: "cat" vs "חתול" — 2 hashes, 1 concept
- Reformulation: "Einstein was born in 1879" vs "1879 is the birth year of Einstein" — same **fact**

**המסקנה החדשה:**
שכבות dedup:
1. **Byte-hash** (FNV-1a — כבר יש) — זהות-בייט
2. **Normalized-hash** — lowercase + punct strip + whitespace normalize → hash
3. **Quote-hash** — טוקנים מילוליים בלבד, ignore order of surrounding text
4. **Fact-hash** — (entity, relation, entity) triple → hash. דורש NER + relation extraction
5. **Semantic-hash** — embedding similarity > 0.95 → treat as duplicate (deferred — requires embeddings)

### Dedup על canonical:
- HTML יש `<link rel="canonical" href="...">` — זו ה-URL המקורית
- RSS יש `<link>` tag
- כל fetching step הראשון: extract canonical, lookup ב-hash_registry, אם קיים → רק עדכון מקור, לא atom חדש

---

## שבירת כלים #6 — "לא יודע מה אני לא יודע"

**השבר:** autonomous-learning לא הולך להצליח אם אין לו **goal**. "תלמד הכל" = תתנוון.

ZETS צריך לדעת **מה** ללמוד. איך הוא יודע?

**המסקנה החדשה:**
3 מקורות לgoals:
1. **Gap detection** (מ-hash_registry): מה personas שואלים שאין להם תשובה? מה ZETS
   מסמן כ-Hypothesis שלא אומת?
2. **Curiosity scores**: מה יש frequency של שאלות בלי תשובה טובה?
3. **User-directed**: עידן מסמן topic priority → system מעדיף sources באותו domain

זה הופך את ה-learner מ-"תחטוף הכל" ל-**goal-directed**. פחות נתונים, יותר רלוונטיים.

---

## ארכיטקטורה המעודכנת (אחרי שבירת כלים)

```
                   ┌──────────────────────────────┐
                   │   Gap Detector               │
                   │  (hash_registry queries +    │
                   │   persona unanswered Qs)     │
                   └──────────────┬───────────────┘
                                  │ goals
                   ┌──────────────▼───────────────┐
                   │   Source Selector            │
                   │  1. MCP server catalog       │
                   │  2. Structured APIs          │
                   │  3. RSS/Atom feeds           │
                   │  4. HTML (last resort)       │
                   └──────────────┬───────────────┘
                                  │ URL queue
                   ┌──────────────▼───────────────┐
                   │   Politeness Gate            │
                   │  - robots.txt                │
                   │  - per-domain rate limit     │
                   │  - User-Agent identification │
                   │  - Crawl-delay respect       │
                   └──────────────┬───────────────┘
                                  │ fetch
                   ┌──────────────▼───────────────┐
                   │   Fetcher (async)            │
                   │  - ETag/If-Mod-Since         │
                   │  - Backoff on 429/503        │
                   │  - 2-3 concurrent max        │
                   └──────────────┬───────────────┘
                                  │ raw
                   ┌──────────────▼───────────────┐
                   │   Extractor                  │
                   │  - canonical URL             │
                   │  - title/author/date         │
                   │  - clean text (readability)  │
                   │  - entities (NER)            │
                   └──────────────┬───────────────┘
                                  │ structured
                   ┌──────────────▼───────────────┐
                   │   Dedup Pipeline             │
                   │  1. byte-hash                │
                   │  2. normalized-hash          │
                   │  3. quote-hash               │
                   │  4. canonical lookup         │
                   └──────────────┬───────────────┘
                                  │ novel/dup
                   ┌──────────────▼───────────────┐
                   │   Trust Scorer               │
                   │  - source tier prior         │
                   │  - corroboration count       │
                   │  - contradiction flag        │
                   └──────────────┬───────────────┘
                                  │ scored
                   ┌──────────────▼───────────────┐
                   │   Graph Ingestor             │
                   │  - atoms (via content_hash)  │
                   │  - edges w/ provenance tag   │
                   │  - register attribute        │
                   │  - hash_registry updates     │
                   └──────────────────────────────┘
```

---

## Politeness contract (NON-NEGOTIABLE)

1. **User-Agent identification**: `ZETS-Learner/0.1 (+https://github.com/idaneldad/zets; contact: idan@chooz.co.il)`
2. **robots.txt honor**: parse בתחילת כל domain, cache 24h, לעולם לא להתעלם
3. **Crawl-delay**: לפחות 3 שניות per domain. אם `Crawl-delay` ב-robots.txt גדול יותר — כבוד.
4. **Concurrency**: לא יותר מ-2 requests concurrent per domain.
5. **Global budget**: 10 req/sec מקס על כל האתרים ביחד
6. **Error backoff**: 429 → double wait. 503 → 10-minute cool-down. 4xx persistent → blacklist domain 24h.
7. **No login-bypass**: אם אתר דורש auth, עוצר. לא headless bypass. לא fake CAPTCHA.
8. **No paywall-bypass**: אם paywall — respect. השתמש ב-archive.org או cached APIs אם זמין.
9. **PII never**: מסנן לדוגמה emails, phone numbers, addresses out of corpus. עידן אמר — אחריות.
10. **Copyright-aware**: fair-use ל-facts. לא לאחסן מאמרים שלמים verbatim מאתרים מסחריים. אחסן facts + citation.

---

## Source catalog (MVP — 20 sources, tier-seeded)

### Tier A (Peer-reviewed, structured)
- **arXiv.org** — OAI-PMH API — 2M+ preprints
- **PubMed** — E-utilities API — 35M+ biomed
- **OpenAlex** — REST API — 250M+ papers

### Tier B (Reference)
- **Wikipedia** — MediaWiki API — 60M+ articles (280 langs)
- **Wikidata** — SPARQL endpoint — 110M+ items
- **ConceptNet** — download — 34M+ edges
- **WordNet** — download — ~150K synsets

### Tier C (Primary source, gov/edu)
- **data.gov** — open datasets
- **NASA ADS** — astronomy bibliographic
- **NIH PMC** — biomedical full text
- **Project Gutenberg** — 70K books

### Tier D (Journalism)
- **Reuters** — RSS feeds (business, world, sports)
- **BBC** — RSS feeds
- **AP** — RSS feeds
- **NYT** — RSS (topic feeds)

### Tier E (Industry)
- **StackExchange** — API (generous free tier)
- **GitHub** — GraphQL API

### Tier F (Community)
- **Reddit** — OAuth API (rate-limited)
- **HackerNews** — Firebase API (free, fast)

---

## Storage model — טריליונים (scalable, not hyperbolic)

מסר: "טריליון" לא אפשרי במובן אמיתי. אבל **מיליארדים** — כן.

### Math:
- Current ZETS: 211K atoms, 158MB = 750 bytes/atom (edges dominate)
- Scale to 1B atoms = ~750GB. Doable on one SSD.
- Scale to 10B atoms = 7.5TB. Doable on one server.
- 1 trillion = 750TB. Requires distributed storage, cold tiering, aggressive dedup.

### Compression strategies (to apply):
- Bit-packed edge weights (5-bit instead of 8-bit for cold edges): -37% size
- Prefix compression on atom data (title with common prefix): -20%
- Sparse edge encoding (for sparse graphs): variable
- Column-oriented cold storage: -30% for rarely-read edges

**מסקנה**: target 10B atoms over 1-2 years of careful ingestion. Trillion
רק עם dramatic changes (distributed, embedding compression, specialized hardware).

---

## Metrics — כדי לדעת שזה עובד

1. **Coverage growth**: unique-facts/day (after dedup). Target: 10K-100K/day steady.
2. **Corroboration density**: mean sources per fact. Target: climbing from 1 → 3-5.
3. **Confidence distribution**: % of facts at conf > 0.7 (high-confidence).
4. **Gap closure rate**: % of previously-unanswered questions now answerable.
5. **Politeness violations**: 429/503 errors, must be < 0.1% of requests.
6. **Storage efficiency**: bytes-per-unique-fact. Should DECREASE over time (dedup kicks in).

---

## מה לממש בתור הזה (מוגבל, מציאותי)

### לא עכשיו:
- Full Rust async crawler (weeks of work)
- NER/relation extraction (needs models)
- Complete source catalog (all 20 sources)
- Distributed storage

### כן עכשיו (MVP):
1. **Trust tier seed data** — 100 domains tagged A-F
2. **Politeness-first fetcher** (Python prototype)
3. **RSS ingestor** — 5 feeds, fetch once, extract title/link/date/summary
4. **Dedup via hash_registry** — link to existing atoms
5. **Run once**, 50 items, verify flow works end-to-end
6. **Store in snapshot** — `autonomous_v1.atoms` separate, low-risk

זה ~400 שורות Python. לא production. **proof-of-concept**.

---

## מה **לא** לעשות (lessons pre-registered)

- **לא** לרוץ loops אוטונומיים בלי stop-switch
- **לא** לאחסן מאמרים verbatim — רק facts + citation
- **לא** להתעלם מ-robots.txt אף פעם
- **לא** לרכז request אחרי השעות הלילה (politer distribution)
- **לא** לאחסן PII
- **לא** להתחבר למקורות שדורשים auth אם אין לי account legit
- **לא** לעקוף paywall
- **לא** לייצר false certainty — fact ממקור יחיד = Hypothesis, לא Asserted
