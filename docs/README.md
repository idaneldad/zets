# ZETS Documentation Index

**Last reorganized:** 23.04.2026  
**Purpose:** A navigable docs tree so you (and Claude in future sessions) can find things by intent, not by date.

---

## How to use this tree

The folder numbers are **priority order for an LLM-agent** that needs to understand ZETS:

| # | Folder | Read when |
|---|--------|-----------|
| **00** | `doctrine/` | **Always first.** Engineering rules, architectural DNA, what to build and what NOT. The bible. |
| **10** | `architecture/` | When you need to understand HOW the current system works. |
| **20** | `research/` | When evaluating an external project (OpenClaw, Cyc, etc) for ideas to absorb. |
| **30** | `decisions/` | ADRs — why we chose X over Y. Consult before re-opening a settled question. |
| **40** | `ai_consultations/` | Past gpt-4o / Gemini / Anthropic / Groq sessions. Synthesis already done — read for raw input. |
| **50** | `working/` | Active sprints, designs in progress, scratch notes. |
| **90** | `archive/` | Old docs kept for trace, but not active. |

---

## Folder overviews

### 00_doctrine — start here
- `01_engineering_rules.md` — Idan's 4-5 binding rules: no flattery, prototype-first, git as RAG, fresh learning only

### 10_architecture — what is built
- `00_current_state.md` — high-level snapshot
- `01_system_overview.md` — the master architecture
- `04_cognitive_kinds.md` / `05_cognitive_modes.md` — 16 CognitiveKinds + modes
- `08_body_mind.md` — sense/cognition layering
- `09_ingestion.md` — how data enters the graph
- `20_inventory_v1.md` — full module inventory (36K LOC, 100+ modules)

### 20_research — external projects analyzed
- `multilingual_extraction.md` — what works for Hebrew + 47 other langs
- `openclaw/`
  - `00_initial_integration_spec.md` — first sketch (legacy)
  - `01_external_analysis.md` — first deep look (legacy)
  - `02_lessons_v1.md` — synthesis after deep code-read
  - `03_practical_lessons_v2.md` — **this is the latest, most actionable one (23.04.26)**

### 30_decisions — ADRs (Architecture Decision Records)
**Format:** `NNNN_short_title.md`, with sections: Context, Decision, Consequences.  
Once landed, ADRs are immutable — supersede with a new ADR if needed.

### 40_ai_consultations — raw AI inputs
Filenames: `YYYYMMDD_topic_AI_Vn.md`. These are **raw replies**, not synthesis. The doctrine + research docs are where the synthesis lives.

### 50_working — work in progress
The dump zone. Files here move to architecture / research / decisions when stable.

### 90_archive — old but kept

---

## How to add a new doc

1. **Decide the bucket** using the table above.
2. **Pick a number** that doesn't conflict with siblings.
3. **Add yourself** to this README's relevant section.
4. **First line of every doc:** `# <Title>` then `**date:** YYYY.MM.DD` and a 2-line summary.
5. **Cross-link** other relevant docs at the bottom.

## Naming conventions

- Numbered prefixes for ordering (01, 02, 10, 20...)
- Snake_case filenames
- Date-prefixed for time-bounded docs (consultations, sprints)
- `_VN` suffix for versioned design docs

## When to retire a doc

Move to `90_archive/` when:
- Superseded by a newer version (link from new doc)
- Subject obsolete (e.g. an architecture we abandoned)
- Speculative and not pursued

**Never delete.** Git history matters. Move = preserve.
