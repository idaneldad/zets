# Repo Split Design — Clean ZETS + Separate Tools

**תאריך:** 23.04.2026
**בקשת עידן:** "פרויקט zets נקי... הכל rust וגרפים... כלים כshell/GUI בפרויקט נפרד"
**חוק:** לבדוק מציאות קודם, לא להציע בעיוור.

---

## מה יש היום (אמת מאומתת)

### בpרויקט `zets` הנוכחי:

```
zets/
├── Cargo.toml, Cargo.lock, build.rs      ← Rust build
├── src/           45 .rs files           ← ✓ Rust core (נשאר)
├── src/bin/       41 binaries            ← ✓ Rust CLIs (רובם נשאר)
├── tests/         integration tests      ← ✓ נשאר (+ פה יכנס py_testers)
├── target/        Rust artifacts         ← ✓ ignored in git
├── data/
│   ├── baseline/    wiki snapshot 158MB  ← ✓ נשאר (graphs)
│   ├── core/        core data            ← ✓ נשאר
│   ├── packs/       16 language packs    ← ✓ נשאר (learning content)
│   ├── wikipedia_dumps/  42 lang × X MB  ← ✓ נשאר (learning content)
│   ├── topics/      seed topic lists     ← ✓ נשאר
│   ├── seeds/       ingestion seeds      ← ✓ נשאר
│   ├── hebrew, english, multilingual     ← ✓ נשאר
│   ├── clients/     16 persona JSONs     ← ✗ פרוטוטיפ Python — יזרק
│   ├── autonomous/  RSS ingested items   ← ✓ נשאר (learning content)
│   └── autonomous_cache/ robots+etag     ← ✓ נשאר (fetcher state)
│
├── mcp/                                  ← ✗ כולו Python — יוצא
│   ├── zets_client.py       (Python mock, לא אמיתי)
│   ├── zets_http_api.py     (Python wrapper → subprocess Rust)
│   ├── zets_mcp_server.py   (Python MCP bridge)
│   ├── personas.py          (Python persona data)
│   ├── multi_client.py, v2, v3
│   ├── build_persona_snapshots.py
│   ├── ask_ai.py            (AI consultation helper)
│   ├── autonomous/          (8 autonomous learner py files)
│   ├── deploy.sh
│   └── logs/
│
├── scripts/                              ← ✓ נשאר (רובם bash)
├── research/                             ← ✗ חוץ (Python POCs)
│   ├── hopfield_poc.py
│   ├── hopfield_enhanced.py
│   └── .venv/   (gitignored, 85MB)
│
├── docs/                                 ← ✓ נשאר (RAG)
│   └── working/                          ← ✓ Git-as-RAG per CLAUDE_RULES
│
└── (root .md files)                      ← ✓ נשאר
```

### Python files בתוך הpרויקט (23 total, excluding venv):

| קובץ | תפקיד | להשאיר? |
|------|--------|----------|
| `mcp/zets_client.py` | mock persona server (לא Rust) | ❌ retire — replace with Rust `zets_node` |
| `mcp/zets_http_api.py` | HTTP wrapper מעל Rust binaries | 🔶 retire when Rust node has HTTP |
| `mcp/zets_mcp_server.py` | MCP bridge ל-Claude | 🔶 tool (separate repo) |
| `mcp/personas.py` | 16 persona definitions | ❌ retire (data → TOML seed files) |
| `mcp/multi_client*.py` | orchestrators | 🔶 tool (separate repo) |
| `mcp/build_persona_snapshots.py` | persona init | 🔶 tool (separate repo) |
| `mcp/ask_ai.py` | consultation helper | 🔶 tool (separate repo) |
| `mcp/autonomous/*.py` (8 files) | crawlers, RSS, wiki parse | 🔶 tool (separate repo) |
| `research/hopfield*.py` | POCs | 🔶 research (separate repo) |
| `tests/test_unified_node_v1.py` | Python tester for Rust | ✓ stays in py_testers/ |

---

## הצעה: 3 Repos

### Repo 1: `zets` (clean, Rust-only)

**תוכן:**
```
zets/
├── Cargo.toml, Cargo.lock, build.rs
├── src/                 # 45 core .rs files
│   ├── atoms.rs
│   ├── engine.rs
│   ├── hash_registry.rs
│   ├── inference.rs
│   ├── ingestion.rs
│   ├── learning_layer.rs
│   ├── mmap_core.rs
│   ├── pack.rs
│   ├── piece_graph.rs
│   ├── relations.rs
│   ├── smart_walk.rs
│   └── ... (45 files)
├── src/bin/
│   └── zets_node.rs     # THE unified binary (server + persona modes)
├── tests/               # Rust integration tests
├── py_testers/          # NEW — Python prototypes (NOT packaged)
│   ├── README.md        # "disposable prototypes. test logic before Rust."
│   └── .gitignore       # exclude from cargo package
├── data/
│   ├── baseline/        # ingested graphs (binary)
│   ├── packs/           # language packs (binary)
│   ├── seeds/           # seed configs (toml/json)
│   ├── wikipedia_dumps/ # raw learning content (gitignored, too big)
│   └── ...
├── docs/
│   ├── working/         # CLAUDE_RULES + design docs (RAG)
│   └── README.md
├── install/             # NEW — installation instructions
│   ├── linux.sh         # systemd unit setup
│   ├── macos.sh
│   └── README.md
├── .gitignore
└── README.md            # "ZETS is a Rust-only knowledge engine"
```

**עקרון:** `cargo build --release` מייצר `zets_node` binary אחד. אפס Python.  
**מטרה:** אמינות, התקנה נקייה, clean packaging.

### Repo 2: `zets-tools` (כל הPython — operations + learning ops)

**תוכן:**
```
zets-tools/
├── autonomous/          # RSS harvester, Wikipedia downloader
│   ├── night_learner.py
│   ├── multi_lang_wiki.py
│   ├── polite_fetcher.py
│   ├── rss_ingestor.py
│   ├── source_tiers.py
│   └── trust_scorer.py
├── persona_manager/     # multi-client orchestration
│   ├── multi_client.py
│   ├── build_persona_snapshots.py
│   └── personas.toml    # data, not code
├── mcp_bridge/          # Claude MCP integration
│   ├── zets_mcp_server.py
│   └── ask_ai.py
├── requirements.txt     # pip deps
├── install/
└── README.md
```

**עקרון:** הכל `pip install` / `uv`. לא חלק מ-zets binary. הם **דוברים** עם zets עדרך HTTP API.

**מה הם עושים:**
- הורדת שפות → wikipedia API / dumps → קורא לzets `POST /ingest`
- RSS harvesting → posts ל-zets
- Persona management → spawn קליינטים של zets_node
- Claude MCP integration → wraps zets HTTP

### Repo 3: `zets-gui` (עתידי — G