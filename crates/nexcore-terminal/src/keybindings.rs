//! User terminal keybindings — persisted per tenant+user pair.
//!
//! Maps keyboard combos to terminal actions with per-user customisation.
//! The [`KeybindingsStore`] trait abstracts persistence following the same
//! pattern as [`crate::preferences::PreferencesStore`] and
//! [`crate::layout::LayoutStore`].
//!
//! ## Primitive Grounding
//!
//! `π(Persistence) + μ(Mapping) + κ(Comparison) + ∂(Boundary) + ς(State)`
#![allow(
    clippy::disallowed_types,
    reason = "keybinding lookup is O(1) by (TenantId, UserId); these IDs lack Ord"
)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use vr_core::ids::{TenantId, UserId};

// ── Key Modifiers ──────────────────────────────────────────

/// Modifier keys held during a keyboard shortcut.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyModifiers {
    /// Control key.
    pub ctrl: bool,
    /// Shift key.
    pub shift: bool,
    /// Alt / Option key.
    pub alt: bool,
    /// Meta / Command / Windows key.
    pub meta: bool,
}

impl KeyModifiers {
    /// Shorthand: Ctrl only.
    #[must_use]
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::default()
        }
    }

    /// Shorthand: Ctrl+Shift.
    #[must_use]
    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Self::default()
        }
    }

    /// No modifiers held.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }
}

// ── Key Combo ──────────────────────────────────────────────

/// A keyboard combination: modifier keys + a single key name.
///
/// The key is always stored lowercase. `+` is normalised to `=`
/// (same physical key on most layouts).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    /// Modifier keys held.
    pub modifiers: KeyModifiers,
    /// Lowercase key name (e.g. `"d"`, `"arrowleft"`, `"escape"`).
    pub key: String,
}

impl KeyCombo {
    /// Create a new key combo with validation.
    ///
    /// - Lowercases the key name
    /// - Normalises `"+"` to `"="`
    /// - Returns [`KeybindingError::InvalidKey`] if key is empty
    pub fn new(modifiers: KeyModifiers, key: impl Into<String>) -> Result<Self, KeybindingError> {
        let mut k = key.into().trim().to_lowercase();
        if k.is_empty() {
            return Err(KeybindingError::InvalidKey(
                "key must be non-empty".to_string(),
            ));
        }
        // "+" and "=" are the same physical key; normalise to "="
        if k == "+" {
            k = "=".to_string();
        }
        Ok(Self { modifiers, key: k })
    }
}

// ── Action ─────────────────────────────────────────────────

/// Terminal action triggered by a keybinding.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeybindingAction {
    /// Increase font size by 1pt.
    FontIncrease,
    /// Decrease font size by 1pt.
    FontDecrease,
    /// Reset font size to default (14pt).
    FontReset,
    /// Split the focused pane vertically (side-by-side).
    SplitVertical,
    /// Split the focused pane horizontally (top/bottom).
    SplitHorizontal,
    /// Close the focused pane.
    ClosePane,
    /// Move focus to the pane on the left.
    FocusLeft,
    /// Move focus to the pane on the right.
    FocusRight,
    /// Move focus to the pane above.
    FocusUp,
    /// Move focus to the pane below.
    FocusDown,
    /// Cycle through terminal modes (shell → regulatory → ai → hybrid).
    CycleMode,
    /// Open the in-terminal search overlay.
    OpenSearch,
    /// Close the search overlay and clear results.
    CloseSearch,
}

// ── Scope ──────────────────────────────────────────────────

/// Where a keybinding is handled.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeybindingScope {
    /// Container-level: font zoom, split, close, focus navigation.
    Global,
    /// Per-pane: mode cycling, search open/close.
    Pane,
}

// ── Keybinding ─────────────────────────────────────────────

/// A single keybinding: combo → action at a given scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keybinding {
    /// The key combination that triggers this binding.
    pub combo: KeyCombo,
    /// The action to perform.
    pub action: KeybindingAction,
    /// Where this binding is evaluated.
    pub scope: KeybindingScope,
    /// Whether this binding is active. Disabled bindings are skipped
    /// during resolution but preserved for user re-enablement.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

// ── Keybinding Set ─────────────────────────────────────────

/// Complete set of keybindings for one user. The canonical source of
/// truth for keyboard shortcut → action mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingSet {
    /// All keybindings (enabled and disabled).
    pub bindings: Vec<Keybinding>,
}

impl Default for KeybindingSet {
    fn default() -> Self {
        default_keybindings()
    }
}

impl KeybindingSet {
    /// Find the first **enabled** binding matching a key combo.
    #[must_use]
    pub fn find_by_combo(&self, combo: &KeyCombo) -> Option<&Keybinding> {
        self.bindings
            .iter()
            .find(|b| b.enabled && b.combo == *combo)
    }

    /// Find the first **enabled** binding for a given action.
    #[must_use]
    pub fn find_by_action(&self, action: KeybindingAction) -> Option<&Keybinding> {
        self.bindings
            .iter()
            .find(|b| b.enabled && b.action == action)
    }

    /// Validate the binding set for conflicts.
    ///
    /// Two **enabled** bindings sharing the same combo is a conflict.
    pub fn validate(&self) -> Result<(), KeybindingError> {
        let mut seen: HashMap<&KeyCombo, KeybindingAction> = HashMap::new();
        for b in &self.bindings {
            if !b.enabled {
                continue;
            }
            if let Some(&existing) = seen.get(&b.combo) {
                if existing != b.action {
                    return Err(KeybindingError::ConflictingBindings {
                        combo: b.combo.clone(),
                        actions: vec![existing, b.action],
                    });
                }
            } else {
                seen.insert(&b.combo, b.action);
            }
        }
        Ok(())
    }

    /// The full set of default keybindings matching the current hardcoded
    /// shortcuts in `terminal-client.tsx` and `TerminalPane.tsx`.
    #[must_use]
    pub fn default_set() -> Self {
        default_keybindings()
    }
}

// ── Default Bindings ───────────────────────────────────────

/// Build one binding. Infallible because all default keys are known-valid.
fn bind(
    modifiers: KeyModifiers,
    key: &str,
    action: KeybindingAction,
    scope: KeybindingScope,
) -> Keybinding {
    Keybinding {
        combo: KeyCombo::new(modifiers, key).expect("default keybinding keys are always valid"),
        action,
        scope,
        enabled: true,
    }
}

/// Returns the 13 default keybindings matching the current terminal shortcuts.
///
/// # Global (container-level)
///
/// | Combo | Action |
/// |-------|--------|
/// | Ctrl+= | Font increase |
/// | Ctrl+- | Font decrease |
/// | Ctrl+0 | Font reset |
/// | Ctrl+Shift+D | Split vertical |
/// | Ctrl+Shift+E | Split horizontal |
/// | Ctrl+Shift+W | Close pane |
/// | Ctrl+Shift+ArrowLeft | Focus left |
/// | Ctrl+Shift+ArrowRight | Focus right |
/// | Ctrl+Shift+ArrowUp | Focus up |
/// | Ctrl+Shift+ArrowDown | Focus down |
///
/// # Pane (per-pane)
///
/// | Combo | Action |
/// |-------|--------|
/// | Ctrl+Shift+M | Cycle mode |
/// | Ctrl+F | Open search |
/// | Escape | Close search |
#[must_use]
pub fn default_keybindings() -> KeybindingSet {
    use KeybindingAction::*;
    use KeybindingScope::*;

    let ctrl = KeyModifiers::ctrl();
    let cs = KeyModifiers::ctrl_shift();
    let none = KeyModifiers::none();

    KeybindingSet {
        bindings: vec![
            // Global — font zoom
            bind(ctrl, "=", FontIncrease, Global),
            bind(ctrl, "-", FontDecrease, Global),
            bind(ctrl, "0", FontReset, Global),
            // Global — split / close
            bind(cs, "d", SplitVertical, Global),
            bind(cs, "e", SplitHorizontal, Global),
            bind(cs, "w", ClosePane, Global),
            // Global — focus navigation
            bind(cs, "arrowleft", FocusLeft, Global),
            bind(cs, "arrowright", FocusRight, Global),
            bind(cs, "arrowup", FocusUp, Global),
            bind(cs, "arrowdown", FocusDown, Global),
            // Pane — mode & search
            bind(cs, "m", CycleMode, Pane),
            bind(ctrl, "f", OpenSearch, Pane),
            bind(none, "escape", CloseSearch, Pane),
        ],
    }
}

// ── Error Type ─────────────────────────────────────────────

/// Errors from keybinding operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum KeybindingError {
    /// Persistence layer failure.
    StorageError(String),
    /// Invalid key name (empty or un-normalisable).
    InvalidKey(String),
    /// Two enabled bindings share the same combo for different actions.
    ConflictingBindings {
        /// The duplicated combo.
        combo: KeyCombo,
        /// The conflicting actions.
        actions: Vec<KeybindingAction>,
    },
}

impl std::fmt::Display for KeybindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageError(msg) => write!(f, "keybinding storage error: {msg}"),
            Self::InvalidKey(msg) => write!(f, "invalid key: {msg}"),
            Self::ConflictingBindings { combo, actions } => {
                write!(f, "conflicting bindings for {:?}: {actions:?}", combo.key)
            }
        }
    }
}

impl std::error::Error for KeybindingError {}

// ── Store Trait ─────────────────────────────────────────────

/// Abstraction over keybinding persistence.
///
/// Initial implementation: [`InMemoryKeybindingsStore`].
/// Future: database-backed without API change.
pub trait KeybindingsStore: Send + Sync {
    /// Load keybindings for a tenant+user. Returns [`default_keybindings()`] if none saved.
    fn load(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<KeybindingSet, KeybindingError>> + Send;

    /// Save keybindings for a tenant+user. Validates before persisting.
    fn save(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        set: &KeybindingSet,
    ) -> impl std::future::Future<Output = Result<(), KeybindingError>> + Send;

    /// Return default keybindings (no I/O).
    fn defaults(&self) -> KeybindingSet {
        default_keybindings()
    }
}

// ── In-Memory Implementation ────────────────────────────────

/// In-memory keybinding store using the same `RwLock<HashMap>` pattern
/// as [`crate::preferences::InMemoryPreferencesStore`].
pub struct InMemoryKeybindingsStore {
    store: RwLock<HashMap<(TenantId, UserId), KeybindingSet>>,
}

impl InMemoryKeybindingsStore {
    /// Create a new empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryKeybindingsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeybindingsStore for InMemoryKeybindingsStore {
    async fn load(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<KeybindingSet, KeybindingError> {
        let store = self.store.read().await;
        Ok(store
            .get(&(*tenant_id, *user_id))
            .cloned()
            .unwrap_or_default())
    }

    async fn save(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        set: &KeybindingSet,
    ) -> Result<(), KeybindingError> {
        set.validate()?;
        let mut store = self.store.write().await;
        store.insert((*tenant_id, *user_id), set.clone());
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── KeyModifiers ────────────────────────────────────

    #[test]
    fn modifiers_default_is_all_false() {
        let m = KeyModifiers::default();
        assert!(!m.ctrl && !m.shift && !m.alt && !m.meta);
    }

    // ── KeyCombo ────────────────────────────────────────

    #[test]
    fn combo_lowercases_key() {
        let combo = KeyCombo::new(KeyModifiers::ctrl(), "D").expect("valid");
        assert_eq!(combo.key, "d");
    }

    #[test]
    fn combo_normalises_plus_to_equals() {
        let combo = KeyCombo::new(KeyModifiers::ctrl(), "+").expect("valid");
        assert_eq!(combo.key, "=");
    }

    #[test]
    fn combo_rejects_empty_key() {
        let result = KeyCombo::new(KeyModifiers::none(), "");
        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(err.to_string().contains("non-empty"));
    }

    #[test]
    fn combo_rejects_whitespace_only_key() {
        let result = KeyCombo::new(KeyModifiers::none(), "   ");
        assert!(result.is_err());
    }

    // ── Serde ───────────────────────────────────────────

    #[test]
    fn action_serializes_to_snake_case() {
        let json = serde_json::to_string(&KeybindingAction::FontIncrease).unwrap_or_default();
        assert_eq!(json, "\"font_increase\"");
    }

    #[test]
    fn scope_serializes_to_snake_case() {
        let json = serde_json::to_string(&KeybindingScope::Global).unwrap_or_default();
        assert_eq!(json, "\"global\"");
    }

    #[test]
    fn keybinding_serde_roundtrip() {
        let binding = Keybinding {
            combo: KeyCombo::new(KeyModifiers::ctrl_shift(), "d").expect("valid"),
            action: KeybindingAction::SplitVertical,
            scope: KeybindingScope::Global,
            enabled: true,
        };
        let json = serde_json::to_string(&binding).unwrap_or_default();
        assert!(!json.is_empty());
        let restored: Keybinding =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(restored.action, KeybindingAction::SplitVertical);
        assert_eq!(restored.combo.key, "d");
        assert!(restored.enabled);
    }

    #[test]
    fn keybinding_enabled_defaults_to_true() {
        // JSON without "enabled" field — serde default should be true
        let json = r#"{
            "combo": {"modifiers": {"ctrl": true, "shift": false, "alt": false, "meta": false}, "key": "f"},
            "action": "open_search",
            "scope": "pane"
        }"#;
        let binding: Keybinding = serde_json::from_str(json).expect("should parse");
        assert!(binding.enabled);
    }

    // ── KeybindingSet ───────────────────────────────────

    #[test]
    fn default_set_returns_13_bindings() {
        let set = KeybindingSet::default_set();
        assert_eq!(set.bindings.len(), 13);
    }

    #[test]
    fn default_set_all_enabled() {
        let set = KeybindingSet::default_set();
        assert!(set.bindings.iter().all(|b| b.enabled));
    }

    #[test]
    fn find_by_combo_finds_matching() {
        let set = KeybindingSet::default_set();
        let combo = KeyCombo::new(KeyModifiers::ctrl_shift(), "d").expect("valid");
        let found = set.find_by_combo(&combo);
        assert!(found.is_some());
        assert_eq!(
            found.expect("just checked").action,
            KeybindingAction::SplitVertical
        );
    }

    #[test]
    fn find_by_combo_returns_none_for_unknown() {
        let set = KeybindingSet::default_set();
        let combo = KeyCombo::new(KeyModifiers::ctrl(), "z").expect("valid");
        assert!(set.find_by_combo(&combo).is_none());
    }

    #[test]
    fn find_by_combo_skips_disabled() {
        let mut set = KeybindingSet::default_set();
        // Disable the split-vertical binding
        for b in &mut set.bindings {
            if b.action == KeybindingAction::SplitVertical {
                b.enabled = false;
            }
        }
        let combo = KeyCombo::new(KeyModifiers::ctrl_shift(), "d").expect("valid");
        assert!(set.find_by_combo(&combo).is_none());
    }

    #[test]
    fn find_by_action_finds_first_match() {
        let set = KeybindingSet::default_set();
        let found = set.find_by_action(KeybindingAction::CloseSearch);
        assert!(found.is_some());
        assert_eq!(found.expect("just checked").combo.key, "escape");
    }

    #[test]
    fn validate_passes_on_default_set() {
        let set = KeybindingSet::default_set();
        assert!(set.validate().is_ok());
    }

    #[test]
    fn validate_catches_duplicate_combo_conflict() {
        let mut set = KeybindingSet::default_set();
        // Add a second enabled binding with the same combo as FontIncrease (Ctrl+=)
        set.bindings.push(Keybinding {
            combo: KeyCombo::new(KeyModifiers::ctrl(), "=").expect("valid"),
            action: KeybindingAction::FontReset, // different action, same combo
            scope: KeybindingScope::Global,
            enabled: true,
        });
        let result = set.validate();
        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(err.to_string().contains("conflicting"));
    }

    #[test]
    fn validate_ignores_disabled_duplicates() {
        let mut set = KeybindingSet::default_set();
        // Add a disabled duplicate — should not conflict
        set.bindings.push(Keybinding {
            combo: KeyCombo::new(KeyModifiers::ctrl(), "=").expect("valid"),
            action: KeybindingAction::FontReset,
            scope: KeybindingScope::Global,
            enabled: false,
        });
        assert!(set.validate().is_ok());
    }

    // ── Store ───────────────────────────────────────────

    #[tokio::test]
    async fn store_load_returns_defaults_on_miss() {
        let store = InMemoryKeybindingsStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();
        let set = store.load(&tenant, &user).await.expect("load");
        assert_eq!(set.bindings.len(), 13);
    }

    #[tokio::test]
    async fn store_save_and_load_roundtrip() {
        let store = InMemoryKeybindingsStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let mut set = KeybindingSet::default_set();
        // Disable one binding to prove persistence works
        for b in &mut set.bindings {
            if b.action == KeybindingAction::OpenSearch {
                b.enabled = false;
            }
        }

        store.save(&tenant, &user, &set).await.expect("save");
        let loaded = store.load(&tenant, &user).await.expect("load");

        let search = loaded.find_by_action(KeybindingAction::OpenSearch);
        // find_by_action only returns enabled — should be None
        assert!(search.is_none());
    }

    #[tokio::test]
    async fn store_save_rejects_conflicting_set() {
        let store = InMemoryKeybindingsStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let mut set = KeybindingSet::default_set();
        set.bindings.push(Keybinding {
            combo: KeyCombo::new(KeyModifiers::ctrl(), "=").expect("valid"),
            action: KeybindingAction::ClosePane,
            scope: KeybindingScope::Global,
            enabled: true,
        });

        let result = store.save(&tenant, &user, &set).await;
        assert!(result.is_err());
    }

    // ── Error Display ───────────────────────────────────

    #[test]
    fn error_display_storage() {
        let err = KeybindingError::StorageError("disk full".to_string());
        assert_eq!(err.to_string(), "keybinding storage error: disk full");
    }

    #[test]
    fn error_display_invalid_key() {
        let err = KeybindingError::InvalidKey("empty".to_string());
        assert_eq!(err.to_string(), "invalid key: empty");
    }

    #[test]
    fn error_display_conflicting() {
        let err = KeybindingError::ConflictingBindings {
            combo: KeyCombo::new(KeyModifiers::ctrl(), "=").expect("valid"),
            actions: vec![KeybindingAction::FontIncrease, KeybindingAction::FontReset],
        };
        let msg = err.to_string();
        assert!(msg.contains("conflicting"));
        assert!(msg.contains("="));
    }

    // ── Convenience ─────────────────────────────────────

    #[test]
    fn default_keybindings_fn_matches_default_set() {
        let a = default_keybindings();
        let b = KeybindingSet::default_set();
        assert_eq!(a.bindings.len(), b.bindings.len());
    }

    #[test]
    fn defaults_method_returns_default() {
        let store = InMemoryKeybindingsStore::new();
        let defaults = store.defaults();
        assert_eq!(defaults.bindings.len(), 13);
    }
}
