# Preferences — Per-owner preference store with inference

ZETS remembers **how** the owner wants to be served, not just what they know.
This module stores and infers behavioral preferences from conversation history.

## Architecture

```
src/preferences/
    mod.rs              ← module declaration + re-exports
    key.rs              ← PreferenceKey — hierarchical, normalized keys
    value.rs            ← PreferenceValue + PreferenceOrigin
    standard_keys.rs    ← StandardKey enum of well-known keys
    conflict.rs         ← conflict resolution logic
    inference.rs        ← infer_preferences() from HistoryEntry slices
    store.rs            ← PreferenceStore — main public API
```

## Quick start

```rust
use zets::preferences::{PreferenceStore, StandardKey};
use zets::personal_graph::{IdentityId, IdentityKind};
use zets::reader::input::{Author, HistoryEntry};

let owner = IdentityId::new(IdentityKind::Person, "idan");
let mut store = PreferenceStore::new();

// Set explicitly
store.set_explicit(&owner, "tone", "formal", &owner, now_ms);

// Infer from conversation history
let inferred_keys = store.infer_from_conversation(&owner, &messages, now_ms);

// Read — falls through to system default if not set
let tone = store.effective(&owner, "tone");       // Some("formal")
let length = store.effective(&owner, "length");   // Some("medium") — default
```

## Standard keys

| Key             | Possible values                                 | Default     |
|-----------------|-------------------------------------------------|-------------|
| `tone`          | `formal` · `casual` · `direct` · `warm`        | `direct`    |
| `length`        | `short` · `medium` · `long`                    | `medium`    |
| `format`        | `bullet_list` · `prose` · `code` · `table`     | `prose`     |
| `language`      | `he` · `en` · `auto`                           | `auto`      |
| `detail_level`  | `brief` · `standard` · `detailed` · `exhaustive`| `standard` |
| `response_style`| `question_back` · `confirm` · `suggest` · `command`| `suggest`|
| `humor_level`   | `none` · `subtle` · `frequent`                 | `none`      |
| `hedging_allowed`| `true` · `false`                              | `true`      |
| `use_emoji`     | `true` · `false`                               | `false`     |
| `code.language` | any language name                               | *(none)*    |
| `name_form`     | `first` · `full` · `nickname`                  | `first`     |

## Inference signals

The inference engine analyzes `HistoryEntry` slices and detects:

1. **Message length** — avg < 50 chars → `length=short`; avg > 200 → `length=long`
2. **Language** — dominant Hebrew/Latin script, or explicit "in Hebrew" / "in English"
3. **Tone/formality** — casual markers (lol, yo, gonna) → `tone=casual`; formal markers → `tone=formal`
4. **Format** — "bullet list", "in a list" → `format=bullet_list`
5. **Corrective patterns** — "shorter", "too long", "be brief" → `length=short`

### Inference requirements

- Minimum 3 owner messages (ZETS messages are excluded)
- 80% consistency across messages for a signal to fire
- Confidence = `min(1.0, signal_count / 5.0)`

## Conflict resolution

Priority: **Explicit > Inferred > Default**

- Newer Explicit beats older Explicit
- Higher-confidence Inferred beats lower-confidence Inferred
- Full history preserved — `store.history(owner, key)` returns all past values

## Owner isolation

Each owner's preferences are completely isolated. Owner A cannot see or affect
Owner B's preferences. The store is keyed by `(owner_id, preference_key)`.
