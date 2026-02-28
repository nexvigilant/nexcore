//! User terminal preferences — persisted per tenant+user pair.
//!
//! Every field uses a strong type with validation (clamping for numerics,
//! non-empty for strings, enums for fixed choices). The [`PreferencesStore`]
//! trait abstracts persistence so the in-memory implementation can be swapped
//! for a database-backed store without changing call sites.
//!
//! ## Primitive Grounding
//!
//! `π(Persistence) + ∂(Boundary) + κ(Comparison) + ς(State) + μ(Mapping)`
#![allow(
    clippy::disallowed_types,
    reason = "preference lookup is O(1) by (TenantId, UserId); these IDs lack Ord"
)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use vr_core::ids::{TenantId, UserId};

// ── Strong Types ────────────────────────────────────────────

/// Font size in points. Clamped to [`Self::MIN`]–[`Self::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontSize(u8);

impl FontSize {
    /// Minimum allowed font size.
    pub const MIN: u8 = 8;
    /// Maximum allowed font size.
    pub const MAX: u8 = 32;
    /// Default font size matching the current xterm.js hardcode.
    pub const DEFAULT: u8 = 14;

    /// Create a new font size, clamping to [`Self::MIN`]–[`Self::MAX`].
    #[must_use]
    pub fn new(size: u8) -> Self {
        Self(size.clamp(Self::MIN, Self::MAX))
    }

    /// The numeric value.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

impl Default for FontSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// Font family CSS string. Falls back to [`Self::DEFAULT`] if empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontFamily(String);

impl FontFamily {
    /// Default font stack matching `terminal-client.tsx`.
    pub const DEFAULT: &'static str =
        "\"JetBrains Mono\", \"Fira Code\", \"Cascadia Code\", monospace";

    /// Create from string, falling back to [`Self::DEFAULT`] if empty/whitespace.
    #[must_use]
    pub fn new(family: impl Into<String>) -> Self {
        let f = family.into();
        if f.trim().is_empty() {
            Self(Self::DEFAULT.to_string())
        } else {
            Self(f)
        }
    }

    /// The CSS font-family string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        Self(Self::DEFAULT.to_string())
    }
}

/// Line height multiplier. Clamped to [`Self::MIN`]–[`Self::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineHeight(f32);

impl LineHeight {
    /// Minimum line height multiplier.
    pub const MIN: f32 = 1.0;
    /// Maximum line height multiplier.
    pub const MAX: f32 = 2.0;
    /// Default matching `terminal-client.tsx`.
    pub const DEFAULT: f32 = 1.2;

    /// Create a new line height, clamping to [`Self::MIN`]–[`Self::MAX`].
    #[must_use]
    pub fn new(height: f32) -> Self {
        Self(height.clamp(Self::MIN, Self::MAX))
    }

    /// The numeric multiplier.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for LineHeight {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// Cursor rendering style.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorStyle {
    /// Filled block cursor.
    Block,
    /// Thin underline cursor.
    Underline,
    /// Thin vertical bar cursor.
    Bar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

/// Scrollback buffer size in lines. Clamped to [`Self::MIN`]–[`Self::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrollbackSize(u32);

impl ScrollbackSize {
    /// Minimum scrollback buffer.
    pub const MIN: u32 = 500;
    /// Maximum scrollback buffer.
    pub const MAX: u32 = 50_000;
    /// Default matching `terminal-client.tsx`.
    pub const DEFAULT: u32 = 5_000;

    /// Create a new scrollback size, clamping to [`Self::MIN`]–[`Self::MAX`].
    #[must_use]
    pub fn new(size: u32) -> Self {
        Self(size.clamp(Self::MIN, Self::MAX))
    }

    /// The numeric value in lines.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl Default for ScrollbackSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// Terminal color scheme identifier.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
    /// Default NexVigilant dark theme (emerald/cyan/gold accents).
    NexvigilantDark,
    /// High-contrast theme for accessibility.
    HighContrast,
    /// Light background theme.
    Light,
    /// Solarized dark variant.
    SolarizedDark,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::NexvigilantDark
    }
}

// ── Preferences Aggregate ───────────────────────────────────

/// Complete user terminal preferences.
///
/// Every field has a sensible default matching the current hardcoded values
/// in `terminal-client.tsx`. Serializes to/from JSON for WebSocket transport
/// and persistence.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPreferences {
    /// Font size in points.
    pub font_size: FontSize,
    /// CSS font-family string.
    pub font_family: FontFamily,
    /// Line height multiplier.
    pub line_height: LineHeight,
    /// Cursor rendering style.
    pub cursor_style: CursorStyle,
    /// Whether the cursor blinks.
    pub cursor_blink: bool,
    /// Scrollback buffer size in lines.
    pub scrollback: ScrollbackSize,
    /// Color scheme identifier.
    pub color_scheme: ColorScheme,
}

impl Default for TerminalPreferences {
    fn default() -> Self {
        Self {
            font_size: FontSize::default(),
            font_family: FontFamily::default(),
            line_height: LineHeight::default(),
            cursor_style: CursorStyle::default(),
            cursor_blink: true,
            scrollback: ScrollbackSize::default(),
            color_scheme: ColorScheme::default(),
        }
    }
}

impl TerminalPreferences {
    /// Check structural equality of all fields.
    ///
    /// Separate from `PartialEq` because `LineHeight` contains `f32`.
    /// Uses bitwise comparison for the float field which is safe here
    /// because values always come from `f32::clamp` (no NaN).
    #[must_use]
    pub fn fields_eq(&self, other: &Self) -> bool {
        self.font_size == other.font_size
            && self.font_family == other.font_family
            && self.line_height.0.to_bits() == other.line_height.0.to_bits()
            && self.cursor_style == other.cursor_style
            && self.cursor_blink == other.cursor_blink
            && self.scrollback == other.scrollback
            && self.color_scheme == other.color_scheme
    }
}

// ── Error Type ──────────────────────────────────────────────

/// Errors from preferences operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum PreferencesError {
    /// Persistence layer failure.
    StorageError(String),
    /// Validation failure on update.
    ValidationError(String),
}

impl std::fmt::Display for PreferencesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageError(msg) => write!(f, "preferences storage error: {msg}"),
            Self::ValidationError(msg) => write!(f, "preferences validation error: {msg}"),
        }
    }
}

impl std::error::Error for PreferencesError {}

// ── Store Trait ─────────────────────────────────────────────

/// Abstraction over preferences persistence.
///
/// Initial implementation: [`InMemoryPreferencesStore`].
/// Future: database-backed (nexcore-brain or Firestore) without API change.
pub trait PreferencesStore: Send + Sync {
    /// Load preferences for a tenant+user. Returns defaults if none saved.
    fn load(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<TerminalPreferences, PreferencesError>> + Send;

    /// Save full preferences for a tenant+user.
    fn save(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        prefs: &TerminalPreferences,
    ) -> impl std::future::Future<Output = Result<(), PreferencesError>> + Send;

    /// Return default preferences (no I/O).
    fn defaults(&self) -> TerminalPreferences {
        TerminalPreferences::default()
    }
}

// ── In-Memory Implementation ────────────────────────────────

/// In-memory preferences store using the same `RwLock<HashMap>` pattern
/// as [`crate::registry::SessionRegistry`].
pub struct InMemoryPreferencesStore {
    store: RwLock<HashMap<(TenantId, UserId), TerminalPreferences>>,
}

impl InMemoryPreferencesStore {
    /// Create a new empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPreferencesStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesStore for InMemoryPreferencesStore {
    async fn load(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<TerminalPreferences, PreferencesError> {
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
        prefs: &TerminalPreferences,
    ) -> Result<(), PreferencesError> {
        let mut store = self.store.write().await;
        store.insert((*tenant_id, *user_id), prefs.clone());
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── FontSize clamping ───────────────────────────────

    #[test]
    fn font_size_clamps_below_min() {
        assert_eq!(FontSize::new(3).value(), FontSize::MIN);
    }

    #[test]
    fn font_size_clamps_above_max() {
        assert_eq!(FontSize::new(99).value(), FontSize::MAX);
    }

    #[test]
    fn font_size_normal() {
        assert_eq!(FontSize::new(16).value(), 16);
    }

    #[test]
    fn font_size_default() {
        assert_eq!(FontSize::default().value(), 14);
    }

    // ── FontFamily validation ───────────────────────────

    #[test]
    fn font_family_empty_uses_default() {
        assert_eq!(FontFamily::new("").as_str(), FontFamily::DEFAULT);
    }

    #[test]
    fn font_family_whitespace_uses_default() {
        assert_eq!(FontFamily::new("   ").as_str(), FontFamily::DEFAULT);
    }

    #[test]
    fn font_family_preserves_custom() {
        assert_eq!(FontFamily::new("Hack").as_str(), "Hack");
    }

    // ── LineHeight clamping ─────────────────────────────

    #[test]
    fn line_height_clamps_below_min() {
        assert_eq!(LineHeight::new(0.5).value(), LineHeight::MIN);
    }

    #[test]
    fn line_height_clamps_above_max() {
        assert_eq!(LineHeight::new(5.0).value(), LineHeight::MAX);
    }

    #[test]
    fn line_height_normal() {
        let lh = LineHeight::new(1.5);
        assert!((lh.value() - 1.5).abs() < 0.001);
    }

    // ── ScrollbackSize clamping ─────────────────────────

    #[test]
    fn scrollback_clamps_below_min() {
        assert_eq!(ScrollbackSize::new(100).value(), ScrollbackSize::MIN);
    }

    #[test]
    fn scrollback_clamps_above_max() {
        assert_eq!(ScrollbackSize::new(100_000).value(), ScrollbackSize::MAX);
    }

    #[test]
    fn scrollback_normal() {
        assert_eq!(ScrollbackSize::new(10_000).value(), 10_000);
    }

    // ── Enum defaults ───────────────────────────────────

    #[test]
    fn cursor_style_default_is_block() {
        assert_eq!(CursorStyle::default(), CursorStyle::Block);
    }

    #[test]
    fn color_scheme_default_is_nexvigilant_dark() {
        assert_eq!(ColorScheme::default(), ColorScheme::NexvigilantDark);
    }

    // ── TerminalPreferences defaults ────────────────────

    #[test]
    fn preferences_defaults_match_design() {
        let prefs = TerminalPreferences::default();
        assert_eq!(prefs.font_size.value(), 14);
        assert_eq!(prefs.font_family.as_str(), FontFamily::DEFAULT);
        assert!((prefs.line_height.value() - 1.2).abs() < 0.001);
        assert_eq!(prefs.cursor_style, CursorStyle::Block);
        assert!(prefs.cursor_blink);
        assert_eq!(prefs.scrollback.value(), 5_000);
        assert_eq!(prefs.color_scheme, ColorScheme::NexvigilantDark);
    }

    // ── Serialization roundtrip ─────────────────────────

    #[test]
    fn preferences_serialize_roundtrip() {
        let original = TerminalPreferences::default();
        let json = serde_json::to_string(&original).unwrap_or_default();
        assert!(
            !json.is_empty(),
            "serialization should produce non-empty JSON"
        );

        let restored: TerminalPreferences =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert!(original.fields_eq(&restored));
    }

    #[test]
    fn preferences_serialize_custom_values() {
        let prefs = TerminalPreferences {
            font_size: FontSize::new(20),
            font_family: FontFamily::new("Hack"),
            line_height: LineHeight::new(1.5),
            cursor_style: CursorStyle::Bar,
            cursor_blink: false,
            scrollback: ScrollbackSize::new(10_000),
            color_scheme: ColorScheme::SolarizedDark,
        };

        let json = serde_json::to_string(&prefs).unwrap_or_default();
        let restored: TerminalPreferences =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(restored.font_size.value(), 20);
        assert_eq!(restored.font_family.as_str(), "Hack");
        assert_eq!(restored.cursor_style, CursorStyle::Bar);
        assert!(!restored.cursor_blink);
        assert_eq!(restored.scrollback.value(), 10_000);
        assert_eq!(restored.color_scheme, ColorScheme::SolarizedDark);
    }

    #[test]
    fn cursor_style_serde_roundtrip() {
        let json = serde_json::to_string(&CursorStyle::Underline).unwrap_or_default();
        assert_eq!(json, "\"underline\"");
    }

    #[test]
    fn color_scheme_serde_roundtrip() {
        let json = serde_json::to_string(&ColorScheme::HighContrast).unwrap_or_default();
        assert_eq!(json, "\"high_contrast\"");
    }

    // ── Store CRUD ──────────────────────────────────────

    #[tokio::test]
    async fn store_load_returns_defaults() {
        let store = InMemoryPreferencesStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let prefs = store
            .load(&tenant, &user)
            .await
            .expect("load should succeed");
        assert!(prefs.fields_eq(&TerminalPreferences::default()));
    }

    #[tokio::test]
    async fn store_save_and_load() {
        let store = InMemoryPreferencesStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let mut prefs = TerminalPreferences::default();
        prefs.font_size = FontSize::new(20);
        prefs.cursor_blink = false;

        store
            .save(&tenant, &user, &prefs)
            .await
            .expect("save should succeed");

        let loaded = store
            .load(&tenant, &user)
            .await
            .expect("load should succeed");
        assert_eq!(loaded.font_size.value(), 20);
        assert!(!loaded.cursor_blink);
    }

    #[tokio::test]
    async fn store_overwrite() {
        let store = InMemoryPreferencesStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let mut prefs = TerminalPreferences::default();
        prefs.font_size = FontSize::new(18);
        store
            .save(&tenant, &user, &prefs)
            .await
            .expect("first save should succeed");

        prefs.font_size = FontSize::new(24);
        prefs.color_scheme = ColorScheme::HighContrast;
        store
            .save(&tenant, &user, &prefs)
            .await
            .expect("overwrite should succeed");

        let loaded = store
            .load(&tenant, &user)
            .await
            .expect("load should succeed");
        assert_eq!(loaded.font_size.value(), 24);
        assert_eq!(loaded.color_scheme, ColorScheme::HighContrast);
    }

    #[tokio::test]
    async fn store_isolates_users() {
        let store = InMemoryPreferencesStore::new();
        let tenant = TenantId::new();
        let user_a = UserId::new();
        let user_b = UserId::new();

        let mut prefs_a = TerminalPreferences::default();
        prefs_a.font_size = FontSize::new(10);

        let mut prefs_b = TerminalPreferences::default();
        prefs_b.font_size = FontSize::new(30);

        store
            .save(&tenant, &user_a, &prefs_a)
            .await
            .expect("save a");
        store
            .save(&tenant, &user_b, &prefs_b)
            .await
            .expect("save b");

        let loaded_a = store.load(&tenant, &user_a).await.expect("load a");
        let loaded_b = store.load(&tenant, &user_b).await.expect("load b");

        assert_eq!(loaded_a.font_size.value(), 10);
        assert_eq!(loaded_b.font_size.value(), 30);
    }

    #[test]
    fn defaults_method_returns_default() {
        let store = InMemoryPreferencesStore::new();
        let defaults = store.defaults();
        assert!(defaults.fields_eq(&TerminalPreferences::default()));
    }

    // ── Error Display ───────────────────────────────────

    #[test]
    fn error_display_storage() {
        let err = PreferencesError::StorageError("disk full".to_string());
        assert_eq!(err.to_string(), "preferences storage error: disk full");
    }

    #[test]
    fn error_display_validation() {
        let err = PreferencesError::ValidationError("invalid key".to_string());
        assert_eq!(err.to_string(), "preferences validation error: invalid key");
    }
}
