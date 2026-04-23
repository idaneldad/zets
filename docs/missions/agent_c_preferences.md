# MISSION: Agent C — Preference Store

**Branch:** `feat/preference-store`
**Estimated time:** 3-4 hours
**Priority:** MEDIUM-HIGH — improves Cat 3 (Memory) from 0.25 → 0.85 on this specific test

---

## Context

ZETS remembers conversations (ConversationStore), facts about people
(PersonalGraph), and secrets (Vault). But it does NOT remember
**preferences** — how the user prefers ZETS to behave.

Example: Idan consistently writes concise messages and dislikes
flattery. ZETS should:
1. Detect this pattern in conversation history
2. Store it as a preference
3. Honor it consistently

Your task: Build `src/preferences/` — preference store with explicit
setting + inference from conversation.

---

## Rules of engagement

1. **Branch:** `feat/preference-store` from main
2. **Scope:** NEW files in `src/preferences/` only
3. You may NOT modify:
   - `src/lib.rs` (Idan does registration)
   - `src/personal_graph/` (USE it, don't change)
   - `src/conversation/` (READ from it, don't change)
   - `Cargo.toml` (no new deps)
4. **Tests:** 15+ tests passing
5. **No hallucinations:** Unclear → STOP, write to `QUESTIONS.md`

---

## Interface contract

```rust
use std::collections::HashMap;
use crate::personal_graph::{IdentityId, Visibility};
use crate::reader::input::HistoryEntry;

pub struct PreferenceStore {
    // per-owner preferences
}

pub struct PreferenceKey(pub String);  // parseable, hierarchical
pub struct PreferenceValue {
    pub value: String,
    pub origin: PreferenceOrigin,
    pub set_at_ms: i64,
    pub confidence: f32,    // 0..1: how sure we are this is a real preference
}

pub enum PreferenceOrigin {
    Explicit { by: IdentityId },              // user or owner set it
    Inferred { from_messages: Vec<String> },  // detected from conversation
    Default,                                   // system default
}

pub enum PreferenceConflict {
    Override { old: PreferenceValue, new: PreferenceValue },
    Merge { values: Vec<PreferenceValue> },
    Keep { reason: String },
}

impl PreferenceStore {
    pub fn new() -> Self;
    
    // Get a single preference. Returns None if not set.
    pub fn get(&self, owner: &IdentityId, key: &str) -> Option<&PreferenceValue>;
    
    // Get all preferences for an owner.
    pub fn all_for(&self, owner: &IdentityId) -> Vec<(&PreferenceKey, &PreferenceValue)>;
    
    // Set explicitly.
    pub fn set_explicit(
        &mut self,
        owner: &IdentityId,
        key: &str,
        value: impl Into<String>,
        by: &IdentityId,
        now_ms: i64,
    );
    
    // Infer from conversation; adds/updates entries.
    pub fn infer_from_conversation(
        &mut self,
        owner: &IdentityId,
        messages: &[HistoryEntry],
        now_ms: i64,
    ) -> Vec<PreferenceKey>;  // returns keys that were added/updated
    
    // Resolve conflicts — explicit beats inferred.
    pub fn effective(&self, owner: &IdentityId, key: &str) -> Option<String>;
    
    // History of changes.
    pub fn history(&self, owner: &IdentityId, key: &str) -> Vec<&PreferenceValue>;
    
    // Remove.
    pub fn unset(&mut self, owner: &IdentityId, key: &str);
}
```

---

## Files to create

```
src/preferences/
    mod.rs              ← module declaration + docs
    store.rs            ← PreferenceStore main impl
    key.rs              ← PreferenceKey standardization
    value.rs            ← PreferenceValue + PreferenceOrigin
    inference.rs        ← infer_from_conversation logic
    standard_keys.rs    ← enum of well-known keys
    conflict.rs         ← conflict resolution
```

---

## Standard preference keys (in standard_keys.rs)

```rust
/// Well-known preference keys. Use these rather than free-form strings.
pub enum StandardKey {
    Tone,              // "formal" | "casual" | "direct" | "warm"
    Length,            // "short" | "medium" | "long"
    Format,            // "bullet_list" | "prose" | "code" | "table"
    Language,          // "he" | "en" | "auto"
    DetailLevel,       // "brief" | "standard" | "detailed" | "exhaustive"
    ResponseStyle,     // "question_back" | "confirm" | "suggest" | "command"
    HumorLevel,        // 0..1 or "none" | "subtle" | "frequent"
    HedgingAllowed,    // bool
    UseEmoji,          // bool
    CodeLanguage,      // default programming lang
    NameForm,          // "first" | "full" | "nickname"
}

impl StandardKey {
    pub fn as_key(&self) -> PreferenceKey { ... }
    pub fn default_value(&self) -> Option<&'static str>;
}
```

---

## Inference logic (inference.rs)

Goal: look at recent messages, detect patterns, output preferences.

### Signals

1. **Message length** → length preference
   - Avg length of owner's messages < 50 chars → Length::Short
   - > 200 chars → Length::Long

2. **Formality markers** (from reader/style.rs)
   - If style.formality > 0.6 consistently → Tone::Formal
   - < 0.4 → Tone::Casual

3. **Explicit language preference**
   - Patterns: "in Hebrew", "בעברית", "in English"
   - Or: pick dominant language in messages

4. **Format markers**
   - "bullet list", "numbered", "in a list" → Format::BulletList
   - "just tell me", "briefly" → Length::Short

5. **Corrective patterns**
   - "don't do X" → infer avoid-X preference
   - "I said shorter" → Length::Short explicit

### Inference confidence

```
signal_strength = count of evidence
total_messages = count of messages analyzed
confidence = min(1.0, signal_strength / 5.0)

// require at least 3 messages before inferring anything
// require consistency (80% agreement across messages)
```

### Conflict resolution

- Explicit > Inferred
- Newer Explicit > Older Explicit
- Higher confidence Inferred > Lower
- If multiple contradictory Explicits exist recently → log warning, use newest

---

## Tests (15+)

```rust
#[test] fn test_empty_store() {}
#[test] fn test_set_explicit_and_get() {}
#[test] fn test_set_owner_isolated() {}  // A and B don't see each other
#[test] fn test_inferred_length_short_from_short_messages() {}
#[test] fn test_inferred_length_long_from_long_messages() {}
#[test] fn test_inferred_formality_from_style_features() {}
#[test] fn test_inferred_language_from_dominant() {}
#[test] fn test_explicit_overrides_inferred() {}
#[test] fn test_later_explicit_overrides_earlier_explicit() {}
#[test] fn test_no_inference_below_3_messages() {}
#[test] fn test_no_inference_on_low_consistency() {}
#[test] fn test_history_preserved() {}
#[test] fn test_unset_removes_value() {}
#[test] fn test_standard_keys_have_defaults() {}
#[test] fn test_effective_falls_through_to_default() {}
#[test] fn test_hebrew_explicit_preference() {}
#[test] fn test_english_explicit_preference() {}
```

---

## Scenario tests (realistic end-to-end)

```rust
#[test]
fn test_idan_preference_inference() {
    // Idan writes 5 short, direct messages in Hebrew.
    // Expected: infer Length::Short, Language::He, Tone::Direct
    
    let owner = IdentityId::new(IdentityKind::Person, "idan");
    let messages = vec![
        HistoryEntry { ts_ms: 1000, who: Author::FromSource, content: "תמשיך".into() },
        HistoryEntry { ts_ms: 2000, who: Author::FromSource, content: "כן, זה".into() },
        HistoryEntry { ts_ms: 3000, who: Author::FromSource, content: "בקצרה".into() },
        HistoryEntry { ts_ms: 4000, who: Author::FromSource, content: "נקודות בלבד".into() },
        HistoryEntry { ts_ms: 5000, who: Author::FromSource, content: "עכשיו הבא".into() },
    ];
    
    let mut store = PreferenceStore::new();
    let inferred = store.infer_from_conversation(&owner, &messages, 6000);
    
    assert!(inferred.len() >= 2);
    assert_eq!(store.effective(&owner, "length"), Some("short".into()));
    assert_eq!(store.effective(&owner, "language"), Some("he".into()));
}
```

---

## Done criteria

1. ✅ 7 source files in `src/preferences/`
2. ✅ 15+ tests passing
3. ✅ `cargo build --lib` clean
4. ✅ README.md in `src/preferences/`
5. ✅ Branch `feat/preference-store` pushed
6. ✅ PR created to main

---

## When blocked

Write to `QUESTIONS.md` at repo root. Stop. Wait.

---

## Final instruction

```bash
cargo test --lib preferences
# X passed, 0 failed

git push origin feat/preference-store
```

PR title: "Add preference store with inference from conversation". Tag Idan.

Go.
