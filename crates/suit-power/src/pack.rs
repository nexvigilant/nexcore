//! # Federated Pack Coordinator
//! Manages current sharing and state for redundant battery packs.

/// The state of an individual battery module.
pub struct PackState {
    /// Total energy available in the pack.
    pub energy: f32,
}
