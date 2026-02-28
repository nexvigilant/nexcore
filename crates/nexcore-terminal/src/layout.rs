//! Terminal split-pane layout persistence — stored per tenant+user pair.
//!
//! The layout is a binary tree: [`LeafNode`] (single pane) or [`SplitNode`]
//! (two children with a direction and ratio). [`TerminalLayout`] wraps the tree
//! with a version field and focused-pane tracker.
//!
//! The [`LayoutStore`] trait abstracts persistence (in-memory, then database)
//! following the same pattern as [`crate::preferences::PreferencesStore`].
//!
//! ## Primitive Grounding
//!
//! `π(Persistence) + ρ(Recursion) + ∂(Boundary) + ς(State) + μ(Mapping)`
#![allow(
    clippy::disallowed_types,
    reason = "layout lookup is O(1) by (TenantId, UserId); these IDs lack Ord"
)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use vr_core::ids::{TenantId, UserId};

// ── Split Direction ──────────────────────────────────────────

/// Direction of a split: horizontal (top/bottom) or vertical (left/right).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    /// Left-right split.
    Horizontal,
    /// Top-bottom split.
    Vertical,
}

// ── Leaf Node ────────────────────────────────────────────────

/// A single terminal pane — the leaf of the layout tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeafNode {
    /// Unique pane identifier.
    pub id: String,
    /// Terminal mode for this pane (e.g. "shell", "regulatory", "ai").
    pub mode: String,
    /// Session ID if a backend session is attached, `None` before connect.
    pub session_id: Option<String>,
}

// ── Split Node ───────────────────────────────────────────────

/// Split ratio clamped to [`Self::MIN`]–[`Self::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplitRatio(f32);

impl SplitRatio {
    /// Minimum ratio (10%).
    pub const MIN: f32 = 0.1;
    /// Maximum ratio (90%).
    pub const MAX: f32 = 0.9;
    /// Default split (50/50).
    pub const DEFAULT: f32 = 0.5;

    /// Create a new ratio, clamping to [`Self::MIN`]–[`Self::MAX`].
    #[must_use]
    pub fn new(ratio: f32) -> Self {
        Self(ratio.clamp(Self::MIN, Self::MAX))
    }

    /// The numeric value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for SplitRatio {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// A split dividing two child layout nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitNode {
    /// Unique identifier for this split.
    pub id: String,
    /// Direction of the split.
    pub direction: SplitDirection,
    /// Ratio of space allocated to the first child (0.1–0.9).
    pub ratio: SplitRatio,
    /// The two child nodes: `[first, second]`.
    pub children: Box<[LayoutNode; 2]>,
}

// ── Layout Node (recursive enum) ─────────────────────────────

/// A node in the layout binary tree.
///
/// Internally tagged with `"type"` for JSON: `{"type":"leaf",...}` or `{"type":"split",...}`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayoutNode {
    /// A terminal pane.
    Leaf(LeafNode),
    /// A split containing two children.
    Split(SplitNode),
}

impl LayoutNode {
    /// Get the ID of this node (leaf or split).
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Leaf(leaf) => &leaf.id,
            Self::Split(split) => &split.id,
        }
    }
}

// ── Terminal Layout ──────────────────────────────────────────

/// Complete layout state for a user's terminal.
///
/// Persisted per tenant+user pair so split-pane configurations
/// survive page refreshes.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalLayout {
    /// Schema version for forward compatibility.
    pub version: u8,
    /// Root of the layout tree.
    pub root: LayoutNode,
    /// ID of the currently focused pane.
    pub focused_pane: String,
}

/// The current layout schema version.
pub const LAYOUT_VERSION: u8 = 1;

/// Create a default single-pane layout.
#[must_use]
pub fn default_layout() -> TerminalLayout {
    let pane_id = "pane-1".to_string();
    TerminalLayout {
        version: LAYOUT_VERSION,
        root: LayoutNode::Leaf(LeafNode {
            id: pane_id.clone(),
            mode: "shell".to_string(),
            session_id: None,
        }),
        focused_pane: pane_id,
    }
}

impl Default for TerminalLayout {
    fn default() -> Self {
        default_layout()
    }
}

// ── Error Type ──────────────────────────────────────────────

/// Errors from layout operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum LayoutError {
    /// Persistence layer failure.
    StorageError(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageError(msg) => write!(f, "layout storage error: {msg}"),
        }
    }
}

impl std::error::Error for LayoutError {}

// ── Store Trait ─────────────────────────────────────────────

/// Abstraction over layout persistence.
///
/// Initial implementation: [`InMemoryLayoutStore`].
/// Future: database-backed without API change.
pub trait LayoutStore: Send + Sync {
    /// Load layout for a tenant+user. Returns [`default_layout()`] if none saved.
    fn load(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<TerminalLayout, LayoutError>> + Send;

    /// Save layout for a tenant+user.
    fn save(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        layout: &TerminalLayout,
    ) -> impl std::future::Future<Output = Result<(), LayoutError>> + Send;

    /// Return default layout (no I/O).
    fn defaults(&self) -> TerminalLayout {
        default_layout()
    }
}

// ── In-Memory Implementation ────────────────────────────────

/// In-memory layout store keyed by `(TenantId, UserId)`.
pub struct InMemoryLayoutStore {
    store: RwLock<HashMap<(TenantId, UserId), TerminalLayout>>,
}

impl InMemoryLayoutStore {
    /// Create a new empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryLayoutStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutStore for InMemoryLayoutStore {
    async fn load(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<TerminalLayout, LayoutError> {
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
        layout: &TerminalLayout,
    ) -> Result<(), LayoutError> {
        let mut store = self.store.write().await;
        store.insert((*tenant_id, *user_id), layout.clone());
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── SplitRatio clamping ─────────────────────────────

    #[test]
    fn ratio_clamps_below_min() {
        assert!((SplitRatio::new(0.0).value() - SplitRatio::MIN).abs() < 0.001);
    }

    #[test]
    fn ratio_clamps_above_max() {
        assert!((SplitRatio::new(1.0).value() - SplitRatio::MAX).abs() < 0.001);
    }

    #[test]
    fn ratio_normal() {
        assert!((SplitRatio::new(0.3).value() - 0.3).abs() < 0.001);
    }

    #[test]
    fn ratio_default_is_half() {
        assert!((SplitRatio::default().value() - 0.5).abs() < 0.001);
    }

    // ── Default layout ──────────────────────────────────

    #[test]
    fn default_layout_is_single_pane() {
        let layout = default_layout();
        assert_eq!(layout.version, LAYOUT_VERSION);
        assert_eq!(layout.focused_pane, "pane-1");
        match &layout.root {
            LayoutNode::Leaf(leaf) => {
                assert_eq!(leaf.id, "pane-1");
                assert_eq!(leaf.mode, "shell");
                assert!(leaf.session_id.is_none());
            }
            LayoutNode::Split(_) => panic!("expected leaf, got split"),
        }
    }

    // ── LayoutNode::id() ────────────────────────────────

    #[test]
    fn node_id_returns_leaf_id() {
        let node = LayoutNode::Leaf(LeafNode {
            id: "pane-42".to_string(),
            mode: "ai".to_string(),
            session_id: None,
        });
        assert_eq!(node.id(), "pane-42");
    }

    #[test]
    fn node_id_returns_split_id() {
        let node = LayoutNode::Split(SplitNode {
            id: "split-1".to_string(),
            direction: SplitDirection::Vertical,
            ratio: SplitRatio::default(),
            children: Box::new([
                LayoutNode::Leaf(LeafNode {
                    id: "a".to_string(),
                    mode: "shell".to_string(),
                    session_id: None,
                }),
                LayoutNode::Leaf(LeafNode {
                    id: "b".to_string(),
                    mode: "shell".to_string(),
                    session_id: None,
                }),
            ]),
        });
        assert_eq!(node.id(), "split-1");
    }

    // ── Serialization roundtrip ─────────────────────────

    #[test]
    fn layout_serialize_roundtrip_single_pane() {
        let original = default_layout();
        let json = serde_json::to_string(&original).unwrap_or_default();
        assert!(
            !json.is_empty(),
            "serialization should produce non-empty JSON"
        );
        assert!(json.contains("\"type\":\"leaf\""));

        let restored: TerminalLayout =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(original, restored);
    }

    #[test]
    fn layout_serialize_roundtrip_split() {
        let layout = TerminalLayout {
            version: LAYOUT_VERSION,
            root: LayoutNode::Split(SplitNode {
                id: "split-1".to_string(),
                direction: SplitDirection::Vertical,
                ratio: SplitRatio::new(0.6),
                children: Box::new([
                    LayoutNode::Leaf(LeafNode {
                        id: "pane-1".to_string(),
                        mode: "shell".to_string(),
                        session_id: Some("sess-abc".to_string()),
                    }),
                    LayoutNode::Leaf(LeafNode {
                        id: "pane-2".to_string(),
                        mode: "regulatory".to_string(),
                        session_id: None,
                    }),
                ]),
            }),
            focused_pane: "pane-2".to_string(),
        };

        let json = serde_json::to_string(&layout).unwrap_or_default();
        assert!(json.contains("\"type\":\"split\""));
        assert!(json.contains("\"type\":\"leaf\""));
        assert!(json.contains("\"vertical\""));
        assert!(json.contains("sess-abc"));

        let restored: TerminalLayout =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(layout, restored);
    }

    #[test]
    fn split_direction_serde() {
        let h = serde_json::to_string(&SplitDirection::Horizontal).unwrap_or_default();
        assert_eq!(h, "\"horizontal\"");
        let v = serde_json::to_string(&SplitDirection::Vertical).unwrap_or_default();
        assert_eq!(v, "\"vertical\"");
    }

    // ── Store CRUD ──────────────────────────────────────

    #[tokio::test]
    async fn store_load_returns_defaults() {
        let store = InMemoryLayoutStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let layout = store
            .load(&tenant, &user)
            .await
            .expect("load should succeed");
        assert_eq!(layout, default_layout());
    }

    #[tokio::test]
    async fn store_save_and_load() {
        let store = InMemoryLayoutStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let mut layout = default_layout();
        layout.focused_pane = "pane-99".to_string();

        store
            .save(&tenant, &user, &layout)
            .await
            .expect("save should succeed");

        let loaded = store
            .load(&tenant, &user)
            .await
            .expect("load should succeed");
        assert_eq!(loaded.focused_pane, "pane-99");
    }

    #[tokio::test]
    async fn store_overwrite() {
        let store = InMemoryLayoutStore::new();
        let tenant = TenantId::new();
        let user = UserId::new();

        let layout_v1 = default_layout();
        store
            .save(&tenant, &user, &layout_v1)
            .await
            .expect("first save");

        let layout_v2 = TerminalLayout {
            version: LAYOUT_VERSION,
            root: LayoutNode::Split(SplitNode {
                id: "split-1".to_string(),
                direction: SplitDirection::Horizontal,
                ratio: SplitRatio::new(0.4),
                children: Box::new([
                    LayoutNode::Leaf(LeafNode {
                        id: "pane-1".to_string(),
                        mode: "shell".to_string(),
                        session_id: None,
                    }),
                    LayoutNode::Leaf(LeafNode {
                        id: "pane-2".to_string(),
                        mode: "ai".to_string(),
                        session_id: None,
                    }),
                ]),
            }),
            focused_pane: "pane-2".to_string(),
        };
        store
            .save(&tenant, &user, &layout_v2)
            .await
            .expect("overwrite");

        let loaded = store.load(&tenant, &user).await.expect("load");
        assert_eq!(loaded.focused_pane, "pane-2");
        match &loaded.root {
            LayoutNode::Split(s) => {
                assert_eq!(s.direction, SplitDirection::Horizontal);
            }
            LayoutNode::Leaf(_) => panic!("expected split after overwrite"),
        }
    }

    #[tokio::test]
    async fn store_isolates_users() {
        let store = InMemoryLayoutStore::new();
        let tenant = TenantId::new();
        let user_a = UserId::new();
        let user_b = UserId::new();

        let mut layout_a = default_layout();
        layout_a.focused_pane = "user-a-pane".to_string();

        let mut layout_b = default_layout();
        layout_b.focused_pane = "user-b-pane".to_string();

        store
            .save(&tenant, &user_a, &layout_a)
            .await
            .expect("save a");
        store
            .save(&tenant, &user_b, &layout_b)
            .await
            .expect("save b");

        let loaded_a = store.load(&tenant, &user_a).await.expect("load a");
        let loaded_b = store.load(&tenant, &user_b).await.expect("load b");

        assert_eq!(loaded_a.focused_pane, "user-a-pane");
        assert_eq!(loaded_b.focused_pane, "user-b-pane");
    }

    #[test]
    fn defaults_method_returns_default_layout() {
        let store = InMemoryLayoutStore::new();
        assert_eq!(store.defaults(), default_layout());
    }

    // ── Error Display ───────────────────────────────────

    #[test]
    fn error_display_storage() {
        let err = LayoutError::StorageError("disk full".to_string());
        assert_eq!(err.to_string(), "layout storage error: disk full");
    }
}
