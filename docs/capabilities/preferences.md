# Preferences — User Preference Store

**Module:** `src/preferences/`
**Status:** 🟢 0.80 / 1.00
**Landed:** 23.04.2026 (Agent C, Sonnet 4.6)
**Tests:** 56 passing
**LOC:** ~1,635

## מה המשימה

ZETS זוכר **שיחות** (ConversationStore), **עובדות על אנשים** (PersonalGraph), **סודות** (Vault). אבל לא זכר **העדפות** — איך המשתמש רוצה ש-ZETS יתנהג:
- טון (רשמי / לא רשמי / ישיר)
- אורך (קצר / בינוני / ארוך)
- פורמט (bullet / prose / code / table)
- שפה
- רמת פירוט
- סגנון תגובה

## הקריטריון להצלחה

- [x] Store מבוסס per-owner (משתמש ב-IdentityId מ-PersonalGraph)
- [x] 11 standard keys מוגדרים (Tone, Length, Format, Language, ...)
- [x] Explicit preferences — user sets it explicitly
- [x] Inferred preferences — detected from conversation history
- [x] 5 inference signals: message length, formality markers, language, format markers, corrective patterns
- [x] Conflict resolution: Explicit > Inferred > Default
- [x] History preservation (who set what when)
- [x] Visibility integration with PersonalGraph ACL

## איך בוחנים (56 tests)

### QA
- Set explicit → retrieve correctly
- Per-owner isolation (A doesn't see B's preferences)
- Inference from short messages → Length::Short
- Inference from formal writing → Tone::Formal
- Inference from Hebrew → Language::He
- Explicit overrides inferred
- Later explicit overrides earlier explicit
- No inference below 3 messages (not enough signal)
- No inference on low consistency (<80% agreement)
- History preserved across updates
- Visibility tier respected

### Scenario tests (real-world)
- **Idan scenario:** 5 short Hebrew direct messages → infers Length::Short, Language::He, Tone::Direct

## באחריות

**גרף** (fully graph-native). מבוסס על:
- `PersonalGraph::IdentityId` (קיים)
- `HistoryEntry` מ-Reader (קיים)
- אין תלות ב-external services

## קוד

```
src/preferences/
├── mod.rs               (57 lines)  — module + re-exports
├── key.rs               (124 lines) — PreferenceKey (normalized, hierarchical)
├── value.rs             (150 lines) — PreferenceValue + PreferenceOrigin
├── standard_keys.rs     (160 lines) — 11 well-known keys with defaults
├── conflict.rs          (160 lines) — conflict resolution
├── inference.rs         (551 lines) — infer from HistoryEntry slices
├── store.rs             (446 lines) — main PreferenceStore API
└── README.md            (83 lines)  — Usage docs
```

## Interface

```rust
pub struct PreferenceStore { /* ... */ }

pub struct PreferenceKey(pub String);

pub struct PreferenceValue {
    pub value: String,
    pub origin: PreferenceOrigin,
    pub set_at_ms: i64,
    pub confidence: f32,     // 0..1
}

pub enum PreferenceOrigin {
    Explicit { by: IdentityId },
    Inferred { from_messages: Vec<String> },
    Default,
}

impl PreferenceStore {
    pub fn get(&self, owner: &IdentityId, key: &str) -> Option<&PreferenceValue>;
    pub fn set_explicit(&mut self, owner: &IdentityId, key: &str, value: impl Into<String>, by: &IdentityId, now_ms: i64);
    pub fn infer_from_conversation(&mut self, owner: &IdentityId, messages: &[HistoryEntry], now_ms: i64) -> Vec<PreferenceKey>;
    pub fn effective(&self, owner: &IdentityId, key: &str) -> Option<String>;
    pub fn history(&self, owner: &IdentityId, key: &str) -> Vec<&PreferenceValue>;
}
```

## Standard Keys

```rust
pub enum StandardKey {
    Tone,              // "formal" | "casual" | "direct" | "warm"
    Length,            // "short" | "medium" | "long"
    Format,            // "bullet_list" | "prose" | "code" | "table"
    Language,          // "he" | "en" | "auto"
    DetailLevel,       // "brief" | "standard" | "detailed" | "exhaustive"
    ResponseStyle,     // "question_back" | "confirm" | "suggest" | "command"
    HumorLevel,
    HedgingAllowed,    // bool
    UseEmoji,          // bool
    CodeLanguage,
    NameForm,          // "first" | "full" | "nickname"
}
```

## Inference Signals (5)

| Signal | Detects | Output |
|--------|---------|--------|
| Message length | Avg <50 chars OR >200 | Length::Short / Long |
| Formality | Formal markers (from Reader::style) | Tone::Formal / Casual |
| Language dominance | Explicit or majority | Language::He / En |
| Format markers | "bullet list", "just tell me" | Format / Length |
| Corrective | "don't do X", "I said shorter" | Explicit rule |

### Inference confidence
- `signal_strength / 5.0` capped at 1.0
- Requires ≥3 messages before inferring
- Requires ≥80% consistency across messages

## פער (מה חסר להגיע ל-1.00)

1. **Integration with Reader** — currently Reader detects style; Preferences uses it. Wire the Reader output automatically (today: manual call)
2. **Preference decay** — if user hasn't expressed a preference for months, reduce confidence
3. **Multi-device sync** — same user across devices should share preferences (requires PersonalGraph federation)
4. **Visual preferences** — color schemes, accessibility needs

## Impact על HumannessScore

Cat 3 (Memory & Personal Knowledge): 0.72 → 0.82 (+0.10)
Cat 1 (Conversational) indirectly: better alignment with user style
