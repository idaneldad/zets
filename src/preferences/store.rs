//! # PreferenceStore — per-owner preference storage with history
//!
//! Stores preferences keyed by (IdentityId, PreferenceKey). Each
//! preference has a full history — the active value is the last entry,
//! all previous entries are preserved for audit.
//!
//! ## Thread safety
//! This is a simple in-memory store without locking. If concurrent
//! access is needed, wrap in `Arc<Mutex<PreferenceStore>>`.

use std::collections::HashMap;

use crate::personal_graph::IdentityId;
use crate::preferences::conflict::resolve;
use crate::preferences::inference::infer_preferences;
use crate::preferences::key::PreferenceKey;
use crate::preferences::standard_keys::StandardKey;
use crate::preferences::value::{PreferenceOrigin, PreferenceValue};
use crate::reader::input::HistoryEntry;

/// Per-owner, per-key history of preference values.
/// Index 0 = oldest, last = current active value.
type History = Vec<PreferenceValue>;

/// The preference store. One instance per ZETS installation.
#[derive(Debug, Clone, Default)]
pub struct PreferenceStore {
    /// (owner_id_string, key_string) → history of values
    data: HashMap<(String, String), History>,
}

impl PreferenceStore {
    /// Create an empty store.
    pub fn new() -> Self {
        PreferenceStore {
            data: HashMap::new(),
        }
    }

    // ─── Read ────────────────────────────────────────────────────────

    /// Get the current active value for `(owner, key)`.
    /// Returns `None` if no value has been set.
    pub fn get(&self, owner: &IdentityId, key: &str) -> Option<&PreferenceValue> {
        let k = key.to_lowercase();
        self.data
            .get(&(owner.0.clone(), k))
            .and_then(|h| h.last())
    }

    /// Get all current preferences for an owner as (key, value) pairs.
    pub fn all_for(&self, owner: &IdentityId) -> Vec<(PreferenceKey, &PreferenceValue)> {
        self.data
            .iter()
            .filter(|((oid, _), _)| oid == &owner.0)
            .filter_map(|((_, key), history)| {
                history.last().map(|v| (PreferenceKey::new(key.clone()), v))
            })
            .collect()
    }

    /// History of all values for `(owner, key)`, oldest first.
    /// Returns an empty slice if no values have been set.
    pub fn history(&self, owner: &IdentityId, key: &str) -> Vec<&PreferenceValue> {
        let k = key.to_lowercase();
        self.data
            .get(&(owner.0.clone(), k))
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// The effective value: active preference, or system default if not set.
    ///
    /// Returns the value string if found, `None` if no value and no default.
    pub fn effective(&self, owner: &IdentityId, key: &str) -> Option<String> {
        // First: check explicit/inferred value
        if let Some(v) = self.get(owner, key) {
            return Some(v.value.clone());
        }
        // Fall through to system default
        StandardKey::from_str(&key.to_lowercase()).and_then(|sk| {
            sk.default_value().map(|s| s.to_string())
        })
    }

    // ─── Write ───────────────────────────────────────────────────────

    /// Set a preference explicitly (e.g. user said "prefer English").
    ///
    /// Resolves conflict with any existing value using priority rules:
    /// Explicit > Inferred. Newer Explicit > Older Explicit.
    pub fn set_explicit(
        &mut self,
        owner: &IdentityId,
        key: &str,
        value: impl Into<String>,
        by: &IdentityId,
        now_ms: i64,
    ) {
        let new_val = PreferenceValue::explicit(value, by.clone(), now_ms);
        self.upsert(owner, key, new_val);
    }

    /// Infer preferences from a conversation history slice.
    ///
    /// Returns the keys that were added or updated.
    pub fn infer_from_conversation(
        &mut self,
        owner: &IdentityId,
        messages: &[HistoryEntry],
        now_ms: i64,
    ) -> Vec<PreferenceKey> {
        let inferred = infer_preferences(messages, now_ms);
        let mut updated_keys = Vec::new();

        for ip in inferred {
            let key_str = ip.key.as_str().to_string();
            self.upsert(owner, &key_str, ip.value);
            updated_keys.push(ip.key);
        }

        updated_keys
    }

    /// Remove a preference for an owner.
    pub fn unset(&mut self, owner: &IdentityId, key: &str) {
        let k = key.to_lowercase();
        self.data.remove(&(owner.0.clone(), k));
    }

    // ─── Internal ────────────────────────────────────────────────────

    /// Insert or update a preference, preserving history and applying
    /// conflict resolution.
    fn upsert(&mut self, owner: &IdentityId, key: &str, new_val: PreferenceValue) {
        let map_key = (owner.0.clone(), key.to_lowercase());
        let history = self.data.entry(map_key).or_default();

        if let Some(existing) = history.last().cloned() {
            let (winner, _conflict) = resolve(existing, new_val);
            // Always append — winner might be the old value if new didn't win,
            // but we still store it if it was genuinely new.
            // More precisely: only append if the winner is the new value.
            // We detect this by checking if winner's set_at_ms matches the new value.
            let last = history.last().unwrap();
            if winner.set_at_ms != last.set_at_ms
                || winner.value != last.value
                || !same_origin_kind(&winner.origin, &last.origin)
            {
                history.push(winner);
            }
        } else {
            history.push(new_val);
        }
    }
}

fn same_origin_kind(a: &PreferenceOrigin, b: &PreferenceOrigin) -> bool {
    match (a, b) {
        (PreferenceOrigin::Explicit { .. }, PreferenceOrigin::Explicit { .. }) => true,
        (PreferenceOrigin::Inferred { .. }, PreferenceOrigin::Inferred { .. }) => true,
        (PreferenceOrigin::Default, PreferenceOrigin::Default) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::{IdentityId, IdentityKind};
    use crate::reader::input::{Author, HistoryEntry};

    fn owner() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }

    fn other_owner() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "roni")
    }

    fn entry(ts: i64, content: &str) -> HistoryEntry {
        HistoryEntry {
            ts_ms: ts,
            who: Author::FromSource,
            content: content.into(),
        }
    }

    #[test]
    fn test_empty_store() {
        let store = PreferenceStore::new();
        assert!(store.get(&owner(), "tone").is_none());
        assert!(store.all_for(&owner()).is_empty());
    }

    #[test]
    fn test_set_explicit_and_get() {
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "tone", "formal", &owner(), 1000);
        let val = store.get(&owner(), "tone");
        assert!(val.is_some());
        assert_eq!(val.unwrap().value, "formal");
        assert!(val.unwrap().is_explicit());
    }

    #[test]
    fn test_set_owner_isolated() {
        // A and B don't see each other's preferences
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "tone", "formal", &owner(), 1000);
        store.set_explicit(&other_owner(), "tone", "casual", &other_owner(), 1000);

        assert_eq!(store.get(&owner(), "tone").unwrap().value, "formal");
        assert_eq!(store.get(&other_owner(), "tone").unwrap().value, "casual");
    }

    #[test]
    fn test_explicit_overrides_inferred() {
        let mut store = PreferenceStore::new();

        // First, infer from short messages
        let msgs: Vec<HistoryEntry> = (0..5).map(|i| entry(i, "hi")).collect();
        store.infer_from_conversation(&owner(), &msgs, 6000);

        // Now set explicitly
        store.set_explicit(&owner(), "length", "long", &owner(), 7000);
        assert_eq!(store.get(&owner(), "length").unwrap().value, "long");
        assert!(store.get(&owner(), "length").unwrap().is_explicit());
    }

    #[test]
    fn test_later_explicit_overrides_earlier_explicit() {
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "tone", "formal", &owner(), 1000);
        store.set_explicit(&owner(), "tone", "casual", &owner(), 2000);
        assert_eq!(store.get(&owner(), "tone").unwrap().value, "casual");
    }

    #[test]
    fn test_history_preserved() {
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "tone", "formal", &owner(), 1000);
        store.set_explicit(&owner(), "tone", "casual", &owner(), 2000);
        let hist = store.history(&owner(), "tone");
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].value, "formal");
        assert_eq!(hist[1].value, "casual");
    }

    #[test]
    fn test_unset_removes_value() {
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "tone", "formal", &owner(), 1000);
        assert!(store.get(&owner(), "tone").is_some());
        store.unset(&owner(), "tone");
        assert!(store.get(&owner(), "tone").is_none());
    }

    #[test]
    fn test_effective_falls_through_to_default() {
        let store = PreferenceStore::new();
        // "length" has default "medium"
        assert_eq!(store.effective(&owner(), "length"), Some("medium".into()));
    }

    #[test]
    fn test_effective_returns_set_value_over_default() {
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "length", "short", &owner(), 1000);
        assert_eq!(store.effective(&owner(), "length"), Some("short".into()));
    }

    #[test]
    fn test_no_inference_below_3_messages() {
        let mut store = PreferenceStore::new();
        let msgs = vec![entry(1, "hi"), entry(2, "ok")];
        let keys = store.infer_from_conversation(&owner(), &msgs, 3000);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_inferred_length_short_from_short_messages() {
        let mut store = PreferenceStore::new();
        let msgs: Vec<HistoryEntry> = vec![
            entry(1, "כן"),
            entry(2, "תמשיך"),
            entry(3, "בסדר"),
            entry(4, "טוב"),
            entry(5, "אוקי"),
        ];
        store.infer_from_conversation(&owner(), &msgs, 6000);
        assert_eq!(store.effective(&owner(), "length"), Some("short".into()));
    }

    #[test]
    fn test_inferred_length_long_from_long_messages() {
        let mut store = PreferenceStore::new();
        // All 5 messages are clearly > 200 chars (checked via char count)
        let msgs = vec![
            entry(1, "This is a really long message about various topics that goes on and on with many words to make it exceed two hundred chars, so we can test the length detection works correctly with long messages from the user here."),
            entry(2, "Another very long message that talks about many things in great detail, providing extensive context and background information about the subject at hand, which makes this message exceed the two hundred character threshold comfortably."),
            entry(3, "Yet another lengthy response from the user with a lot of content and information packed into it, definitely more than two hundred characters of text to make the test reliable and the inference fires correctly every single time."),
            entry(4, "A fourth lengthy message to ensure consistency across multiple examples, since we need eighty percent consistency to infer the preference correctly here, and this message is well over two hundred characters long."),
            entry(5, "And a fifth long message to help push the signal past the threshold and achieve minimum confidence needed for the length preference to be inferred correctly from the conversation history entries in the test suite."),
        ];
        store.infer_from_conversation(&owner(), &msgs, 6000);
        assert_eq!(store.effective(&owner(), "length"), Some("long".into()));
    }

    #[test]
    fn test_inferred_language_from_dominant() {
        let mut store = PreferenceStore::new();
        let msgs = vec![
            entry(1, "תמשיך"),
            entry(2, "כן, זה"),
            entry(3, "בקצרה"),
            entry(4, "נקודות בלבד"),
            entry(5, "עכשיו הבא"),
        ];
        store.infer_from_conversation(&owner(), &msgs, 6000);
        assert_eq!(store.effective(&owner(), "language"), Some("he".into()));
    }

    #[test]
    fn test_inferred_formality_from_style_features() {
        let mut store = PreferenceStore::new();
        let msgs = vec![
            entry(1, "hey yo what's up lol"),
            entry(2, "gonna check btw"),
            entry(3, "wanna grab coffee yo"),
            entry(4, "hey lol"),
            entry(5, "yo btw wanna"),
        ];
        store.infer_from_conversation(&owner(), &msgs, 6000);
        let tone = store.effective(&owner(), "tone");
        assert_eq!(tone, Some("casual".into()));
    }

    #[test]
    fn test_no_inference_on_low_consistency() {
        let mut store = PreferenceStore::new();
        // Mix of genuinely short and genuinely long messages — inconsistent signal
        // 3 short (< 50 chars), 2 long (> 200 chars) → consistency 0.6 < 0.8 threshold
        let msgs = vec![
            entry(1, "hi"),
            entry(2, "A message with lots of words that provides extensive detail and context about many things going on in the situation, making it well over two hundred characters for sure here."),
            entry(3, "ok"),
            entry(4, "Another wordy message with extensive content and background information to ensure this one also exceeds the two hundred character threshold needed for long detection purposes."),
            entry(5, "short"),
        ];
        store.infer_from_conversation(&owner(), &msgs, 6000);
        // Falls back to default "medium" because consistency is too low
        assert_eq!(store.effective(&owner(), "length"), Some("medium".into()));
    }

    #[test]
    fn test_all_for_owner() {
        let mut store = PreferenceStore::new();
        store.set_explicit(&owner(), "tone", "formal", &owner(), 1000);
        store.set_explicit(&owner(), "language", "en", &owner(), 1000);

        let all = store.all_for(&owner());
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_standard_keys_have_defaults() {
        let store = PreferenceStore::new();
        let test_keys = ["tone", "length", "format", "language"];
        for k in &test_keys {
            let val = store.effective(&owner(), k);
            assert!(val.is_some(), "Standard key '{}' should have a default", k);
        }
    }

    #[test]
    fn test_hebrew_explicit_preference() {
        let mut store = PreferenceStore::new();
        let msgs = vec![
            entry(1, "please answer בעברית"),
            entry(2, "תמשיך"),
            entry(3, "כן"),
        ];
        store.infer_from_conversation(&owner(), &msgs, 4000);
        assert_eq!(store.effective(&owner(), "language"), Some("he".into()));
    }

    #[test]
    fn test_english_explicit_preference() {
        let mut store = PreferenceStore::new();
        let msgs = vec![
            entry(1, "please respond in English"),
            entry(2, "yes"),
            entry(3, "ok"),
        ];
        store.infer_from_conversation(&owner(), &msgs, 4000);
        assert_eq!(store.effective(&owner(), "language"), Some("en".into()));
    }

    #[test]
    fn test_idan_preference_inference() {
        // Idan writes 5 short, direct messages in Hebrew.
        // Expected: infer Length::Short, Language::He
        let owner_id = IdentityId::new(IdentityKind::Person, "idan");
        let messages = vec![
            HistoryEntry {
                ts_ms: 1000,
                who: Author::FromSource,
                content: "תמשיך".into(),
            },
            HistoryEntry {
                ts_ms: 2000,
                who: Author::FromSource,
                content: "כן, זה".into(),
            },
            HistoryEntry {
                ts_ms: 3000,
                who: Author::FromSource,
                content: "בקצרה".into(),
            },
            HistoryEntry {
                ts_ms: 4000,
                who: Author::FromSource,
                content: "נקודות בלבד".into(),
            },
            HistoryEntry {
                ts_ms: 5000,
                who: Author::FromSource,
                content: "עכשיו הבא".into(),
            },
        ];

        let mut store = PreferenceStore::new();
        let inferred = store.infer_from_conversation(&owner_id, &messages, 6000);

        assert!(inferred.len() >= 2);
        assert_eq!(
            store.effective(&owner_id, "length"),
            Some("short".into())
        );
        assert_eq!(
            store.effective(&owner_id, "language"),
            Some("he".into())
        );
    }
}
