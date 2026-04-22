# שפות תכנות ב-ZETS — סטטוס + עיצוב

**תאריך:** 22.04.2026
**בקשת עידן:** "מה הסטטוס לגבי שפות תכנות שזה גם מאוד דומה בין שפות התכנות וצריך
סקריפטים של מערכת הפעלה וIT-system ותקשורת"

---

## הסטטוס: **אפס**. לא קיים ב-ZETS היום.

נאמן לעקרון honesty: אין atoms של `for`, אין edges של `calls`, אין הבנה של `import`. 
Corpus קיים רק Wikipedia טבעי. שום דבר מ-code.

**אבל התובנה שלך נכונה:** שפות תכנות הן **חומר עדיף** ל-ZETS מטקסט טבעי, כי:
1. גרמטיקה קשוחה (אין ambiguity של שפה טבעית)
2. Cross-language mapping קל (for-loops דומים מאוד בין Python/Rust/JS/Go)
3. הרבה corpora חינם (GitHub, stack exchange)
4. Verifiable — אפשר להריץ ולבדוק

---

## שפות תכנות באמת דומות — הדמיון

### רמה 1: concepts אוניברסליים (קיימים בכל שפה)

| concept | Python | Rust | JavaScript | Go |
|---------|--------|------|-------------|-----|
| loop | `for x in xs:` | `for x in xs {` | `for (let x of xs) {` | `for _, x := range xs {` |
| condition | `if a:` | `if a {` | `if (a) {` | `if a {` |
| function | `def f():` | `fn f() {` | `function f() {` | `func f() {` |
| list | `[1, 2]` | `vec![1, 2]` | `[1, 2]` | `[]int{1, 2}` |
| hash | `{'a': 1}` | `HashMap::from([("a", 1)])` | `{a: 1}` | `map[string]int{"a": 1}` |
| null | `None` | `None` | `null` | `nil` |

אלה **אותו atom סמנטי**. ב-ZETS: atom `concept:iteration` עם edges ל-`syntax:py_for`, `syntax:rust_for`, וכו'.

### רמה 2: concepts שיטתיים (paradigm-specific)

- Ownership (Rust בלבד): `borrow`, `lifetime`, `'static`
- GIL (Python בלבד): `threading.Lock`, `multiprocessing`
- Async runtime (varied): `async/await` (Python/Rust/JS), `goroutine` (Go), `actor` (Erlang)
- Classes vs structs vs traits: `class` (Python/JS), `struct + impl` (Rust), `type + interface` (Go)

כאן Cross-language עדיין עובד, אבל עם **קונטקסט** — "async ב-Rust דורש runtime מסוג X, בPython זה event loop".

### רמה 3: ecosystem-specific

- Python: pandas, numpy, flask, django
- Rust: serde, tokio, axum
- JS: react, express, next
- Go: gin, cobra

פה cross-mapping נהיה חלש. `pandas.DataFrame` ≠ `polars.DataFrame` (Rust) בדיוק אבל "כמעט".

---

## מה OS scripts + IT + networking מוסיפים

### Shell scripting (bash/zsh/fish/pwsh)

- Pipelines: `cat x | grep y | sort | uniq` — **גרף של transformations**!
- Redirections: `> file`, `2>&1`, `| tee`
- Control flow: `if`, `for`, `while`, `case`

ZETS-fit: shell הוא **כבר גרף** (pipelines). Trivial לייצוג.

### Configuration (YAML, TOML, JSON, HCL, INI)

כל אחת יכולה להיות atoms + edges:
- `{"server": {"port": 8080}}` → atom `server` has_attribute `port=8080`
- Cross-config equivalence: `server.port` (YAML) = `[server]\nport` (TOML) = `server.port` (HCL)

ZETS-fit: מושלם. Configs הם grid structure, ZETS עושה גרפים.

### Network protocols (HTTP, DNS, TCP, SSH)

- HTTP: method + URL + headers + body + status. **יחסים ברורים.**
- DNS: hostname → IP (או NS, MX, TXT). Graph of lookups.
- TCP handshake: SYN → SYN-ACK → ACK. **State machine.**
- SSH: auth flow, channels, tunneling.

ZETS-fit: טוב. Protocols הם state machines + messages, שניהם גרפים.

### System operations

- Systemd units (service dependencies)
- Cron expressions (time grammar)
- Docker compose (container relationships)
- Git (DAG of commits)

ZETS-fit: Git הוא **כבר content-addressable graph**, אולי הכי קרוב ל-ZETS.

---

## למה זה טוב לנו (לא רק מעניין)

1. **ZETS הופך למעשי ל-developer tooling.** חיפוש ידע על API, auto-complete with provenance, code understanding without LLM.
2. **Verifiable learning.** ZETS יכול ללמוד סקריפט + להריץ אותו + לבדוק שהוא עובד. **ground truth** = הרצה.
3. **CHOOZ internal tooling** (בעתיד — כרגע עידן אמר לדלג). כל הפרודקט של CHOOZ הוא קוד. ZETS יכול להיות memory layer של צוות הפיתוח.
4. **Cross-language translation.** "איך עושים X ב-Python?" — ZETS מתרגם מ-Rust ל-Python דרך ה-atoms המשותפים.

---

## מה לא עובד טוב ב-ZETS הנוכחי

1. **אין parser לקוד.** `ingestion.rs` מעבד טקסט טבעי, לא AST. יצטרך משהו חדש.
2. **אין concept של scope/namespace.** Wikipedia text הוא flat. קוד הוא nested.
3. **Type system.** ZETS יכול לייצג "x is_a int" אבל לא "x: Vec<HashMap<String, f64>>".
4. **Execution.** ZETS לא יכול **להריץ** את הקוד שהוא יודע עליו. Verification מחייב execution harness.

---

## עיצוב הצעה — בהדרגה

### Phase P1 — Atoms בסיסיים של שפת תכנות

```rust
// סוגי atoms חדשים:
AtomKind::Keyword      // for, if, while, return, fn, def, class
AtomKind::Operator     // +, ==, ->, =>, <-
AtomKind::Identifier   // user-defined names
AtomKind::Literal      // 42, "hello", true

// edges חדשים:
RelationDef { code: 0x20, name: "syntax_of" },     // token is syntax of lang
RelationDef { code: 0x21, name: "equivalent_to" }, // cross-lang concept match
RelationDef { code: 0x22, name: "imports" },
RelationDef { code: 0x23, name: "calls" },
RelationDef { code: 0x24, name: "defines" },
RelationDef { code: 0x25, name: "returns_type" },
```

### Phase P2 — Ingest sample corpus

- `data/corpora/code_python_stdlib.jsonl` — stdlib function docs
- `data/corpora/code_rust_stdlib.jsonl`
- `ingest_code.rs` binary שמקבל JSONL של {lang, file, ast} ומחלץ atoms

Start small: 500 functions from Python stdlib + 500 from Rust std. Cross-map.

### Phase P3 — Queries מסוג "איך עושים X"

- "איך פותחים קובץ ב-Python?" → atom `file_open` → edges to `open(path)` (Python) + `File::open(path)` (Rust)
- "מה ההבדל בין HashMap לdict?" → walks over `equivalent_to` + attribute edges
- "קוד זה עובד?" → execute in sandbox, check output (Track C product idea)

### Phase P4 — System ops

- `data/corpora/shell_commands.jsonl` — man pages של הכלים הנפוצים
- `data/corpora/http_endpoints.jsonl` — REST patterns
- `data/corpora/systemd_units.jsonl` — unit syntax

### Phase P5 — Verification (Track C)

- ZETS מקבל code + expected output
- מריץ ב-sandbox (docker/firejail)
- מאמת result
- מסמן provenance:Asserted אם passes, Hypothesis אם conflicts

---

## הקשר ל-hash registry + personas

**Hash registry** פותר dedup גם לקוד:
- `def open(path)` (Python) ו-`fn open(path)` (Rust) — שמות **זהים**, content_hash שונה (בגלל AtomKind shift)
- אבל ה-concept של "open file" = atom משותף, יצבעו אליו שני הסינטקסים

**Personas** מקבלים שפות תכנות ייעודיות:
- Noam (developer, Rust+graphs): native Rust, יודע Python
- Adrian (creative): יותר תחומי יצירה, פחות קוד
- Elad (מקצועי): כל השפות בקפדנות, מיקוד על best practices
- Yam (ילדה): **לא** code — register filter חוסם

---

## Summary

**סטטוס נוכחי**: אפס קיים ב-ZETS. Corpus ריק.

**הערכה**: 3-4 phases של עבודה לפני שיהיה useful.

**עדיפות** (דעתי): **נמוכה ביחס ל-D1 ולשפות טבעיות**. השקעה ב-code-ingestion קודם תגרום ל:
- Corpus bloat (GitHub = petabytes)
- Complexity creep (parsers per language)
- Distraction מ-core language learning

**המלצה**: לחכות. לאחר ש-ZETS יציב עם 10+ שפות טבעיות ו-hash registry פעיל,
**אז** להוסיף programming languages כ-domain nosex נוסף. Phase 20+.

**חריג אחד שכן כדאי עכשיו**: **command-line tools atoms** — 200 פקודות shell עם flags + usage. זה מספיק קטן כדי להתחיל איתו, ופיד מיידי לכל persona שעובד בעבודה (Eli, Elad, Yoram, Noam, Idan).

---

## מה הייתי עושה בתור הבא

לא Programming languages ב-phase זה. מה שעידן באמת רוצה:

1. ✅ **Hash registry מימוש** — עשינו בתור זה
2. **Phase 13.1 (register attribute)** — קל, ROI מיידי על ה-16 personas
3. **Phase 12 (dialect overlay)** — אחרי register attribute
4. **Language re-learn pipeline** (התובנה הגאונית של עידן מהturn הקודם)
5. Programming languages — **Phase 20+, לא עכשיו**

---

## UPDATE 22.04.2026 evening — seed implementation shipped

After Idan supplied a 24-dimension classification (9 paradigms × 4 execution × 5 types × 6 purposes) plus 7 automation use-cases, a seed taxonomy snapshot was built:

**Files:**
- `src/bin/seed_programming_taxonomy.rs` — builds the snapshot.
- `src/bin/prog_lang_query.rs` — queries the taxonomy.
- `data/baseline/programming_taxonomy_v1.atoms` — 86 atoms, 284 edges, 5.3KB.

**Verified live:**
- "What for Linux server admin?" → `bash, python`
- "Functional languages?" → 19 languages (Clojure, Haskell, F#, Rust with functional features, etc.)
- "Python profile?" → `scripting+OO+imperative+functional, dynamic, interpreted, used for: api_client, automation_general, browser_automation, data_science, test_automation, web_scraping, etc.`
- "Rust profile?" → `imperative+functional, trait_based, compiled, systems programming`

**Not yet done:**
- Bridge to Wikipedia snapshot (wiki has 330+ "Python" occurrences; hash_registry can connect)
- Execution verification (Track C)
- Ingestion of function-level details from stdlib corpora

**Total cost:** 5.3KB on disk, builds in seconds, answers queries in <1ms. Proves the concept without blowing up scope.
