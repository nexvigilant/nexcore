//! Fire suppression subsystem for suit safety.

/// Fire suppression controller.
#[derive(Debug, Clone)]
pub struct FireSuppression {
    /// Whether the suppression system is armed.
    pub armed: bool,
    /// Remaining suppressant charge (0.0 to 1.0).
    pub charge: f64,
}

impl FireSuppression {
    /// Create a new fire suppression system.
    pub fn new() -> Self {
        Self {
            armed: true,
            charge: 1.0,
        }
    }

    /// Check if suppression is ready.
    pub fn is_ready(&self) -> bool {
        self.armed && self.charge > 0.1
    }
}

impl Default for FireSuppression {
    fn default() -> Self {
        Self::new()
    }
}
