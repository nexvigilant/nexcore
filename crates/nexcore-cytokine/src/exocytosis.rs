//! Exocytosis - vesicle bundling and release with confirmation.
//!
//! ## T1 Grounding
//!
//! - `SignalBundle` → ∂ (boundary) - signals packaged for membrane crossing
//! - `ReleaseConfirmation` → → (causality) - emission causes receptor acknowledgment
//! - `MembraneGate` → κ (comparison) - readiness check before release
//!
//! ## Biological Analog
//!
//! Exocytosis releases vesicle contents outside the cell:
//! 1. **Vesicle formation**: Signals packaged in membrane-bound container
//! 2. **Transport**: Vesicle moves to cell membrane
//! 3. **Docking**: SNARE proteins align vesicle with membrane
//! 4. **Fusion**: Vesicle fuses with membrane, releasing contents
//! 5. **Confirmation**: Receptor binding confirms delivery
//!
//! ## Claude Code Analog
//!
//! Batched signal emission with delivery confirmation:
//! - **Bundle**: Collect related signals into a single emission batch
//! - **Gate check**: Verify recipient is ready before emitting
//! - **Release**: Atomic emission of the entire bundle
//! - **Acknowledgment**: Track which bundles were received

use crate::Cytokine;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// A bundle of signals packaged for release.
///
/// # Tier: T2-C
/// Grounds to: ∂ (boundary — packaged unit) + σ (sequence — ordered signals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalBundle {
    /// Unique bundle identifier
    pub id: String,
    /// Signals in this bundle
    pub signals: Vec<Cytokine>,
    /// When the bundle was created
    pub created_at: DateTime,
    /// Target recipient (None = broadcast)
    pub target: Option<String>,
    /// Bundle state
    pub state: BundleState,
}

/// State of a signal bundle in the exocytic pathway.
///
/// # Tier: T2-P
/// Grounds to: ς (state)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleState {
    /// Being assembled (signals still being added)
    Forming,
    /// Ready for release, awaiting gate check
    Docked,
    /// Released to the environment
    Released,
    /// Delivery confirmed by recipient
    Acknowledged,
    /// Delivery failed
    Failed,
}

impl SignalBundle {
    /// Create a new bundle being assembled.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            signals: Vec::new(),
            created_at: DateTime::now(),
            target: None,
            state: BundleState::Forming,
        }
    }

    /// Set the target recipient.
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Add a signal to the bundle.
    pub fn add(&mut self, signal: Cytokine) {
        self.signals.push(signal);
    }

    /// Add a signal (builder pattern).
    #[must_use]
    pub fn with_signal(mut self, signal: Cytokine) -> Self {
        self.signals.push(signal);
        self
    }

    /// Get number of signals in the bundle.
    pub fn size(&self) -> usize {
        self.signals.len()
    }

    /// Check if the bundle is empty.
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    /// Seal the bundle (no more signals can be added).
    pub fn seal(&mut self) {
        if self.state == BundleState::Forming {
            self.state = BundleState::Docked;
        }
    }

    /// Mark as released.
    pub fn release(&mut self) {
        if self.state == BundleState::Docked {
            self.state = BundleState::Released;
        }
    }

    /// Mark as acknowledged.
    pub fn acknowledge(&mut self) {
        if self.state == BundleState::Released {
            self.state = BundleState::Acknowledged;
        }
    }

    /// Mark as failed.
    pub fn fail(&mut self) {
        self.state = BundleState::Failed;
    }
}

/// Gate that controls when bundles can be released.
///
/// # Tier: T2-P
/// Grounds to: κ (comparison — readiness check)
pub trait MembraneGate: Send + Sync {
    /// Check if the gate is open for release.
    fn is_open(&self) -> bool;

    /// Get reason if gate is closed.
    fn closed_reason(&self) -> Option<String>;
}

/// A simple gate that is always open.
#[derive(Debug, Default)]
pub struct AlwaysOpenGate;

impl MembraneGate for AlwaysOpenGate {
    fn is_open(&self) -> bool {
        true
    }

    fn closed_reason(&self) -> Option<String> {
        None
    }
}

/// A gate that limits release rate (signals per period).
#[derive(Debug)]
pub struct RateLimitGate {
    /// Maximum releases per window
    max_per_window: usize,
    /// Current count in this window
    current_count: usize,
}

impl RateLimitGate {
    /// Create a rate-limiting gate.
    pub fn new(max_per_window: usize) -> Self {
        Self {
            max_per_window,
            current_count: 0,
        }
    }

    /// Record a release.
    pub fn record_release(&mut self) {
        self.current_count += 1;
    }

    /// Reset the window counter.
    pub fn reset_window(&mut self) {
        self.current_count = 0;
    }
}

impl MembraneGate for RateLimitGate {
    fn is_open(&self) -> bool {
        self.current_count < self.max_per_window
    }

    fn closed_reason(&self) -> Option<String> {
        if self.is_open() {
            None
        } else {
            Some(format!(
                "rate limit reached: {}/{}",
                self.current_count, self.max_per_window
            ))
        }
    }
}

/// Exocytic emitter that bundles signals and releases them atomically.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ∂ (boundary crossing) + → (causality of emission)
#[derive(Debug)]
pub struct ExocyticEmitter {
    /// Emitter identifier
    id: String,
    /// Bundles being prepared
    pending: Vec<SignalBundle>,
    /// Released bundles (history)
    released: Vec<SignalBundle>,
    /// Lifetime statistics
    total_bundles_released: u64,
    total_signals_released: u64,
}

impl ExocyticEmitter {
    /// Create a new exocytic emitter.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            pending: Vec::new(),
            released: Vec::new(),
            total_bundles_released: 0,
            total_signals_released: 0,
        }
    }

    /// Start a new bundle.
    pub fn begin_bundle(&mut self) -> &mut SignalBundle {
        let bundle_id = format!(
            "{}-bundle-{}",
            self.id,
            self.total_bundles_released + self.pending.len() as u64 + 1
        );
        self.pending.push(SignalBundle::new(bundle_id));
        let idx = self.pending.len() - 1;
        &mut self.pending[idx]
    }

    /// Release all docked bundles (gate check performed per bundle).
    ///
    /// Returns the signals from successfully released bundles.
    pub fn release_all(&mut self, gate: &dyn MembraneGate) -> Vec<Cytokine> {
        let mut released_signals = Vec::new();

        for bundle in &mut self.pending {
            if bundle.state == BundleState::Forming {
                bundle.seal();
            }

            if bundle.state == BundleState::Docked && gate.is_open() {
                bundle.release();
                released_signals.extend(bundle.signals.clone());
                self.total_bundles_released += 1;
                self.total_signals_released += bundle.signals.len() as u64;
            }
        }

        // Move released bundles to history
        let (released, still_pending): (Vec<_>, Vec<_>) = self
            .pending
            .drain(..)
            .partition(|b| b.state == BundleState::Released);

        self.released.extend(released);
        self.pending = still_pending;

        released_signals
    }

    /// Get pending bundle count.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get released bundle count (history).
    pub fn released_count(&self) -> usize {
        self.released.len()
    }

    /// Get statistics.
    pub fn stats(&self) -> ExocyticStats {
        ExocyticStats {
            id: self.id.clone(),
            pending_bundles: self.pending.len(),
            released_bundles: self.released.len(),
            total_bundles_released: self.total_bundles_released,
            total_signals_released: self.total_signals_released,
        }
    }
}

/// Statistics for exocytic emitter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExocyticStats {
    /// Emitter ID
    pub id: String,
    /// Pending bundles
    pub pending_bundles: usize,
    /// Released bundles in history
    pub released_bundles: usize,
    /// Lifetime bundle count
    pub total_bundles_released: u64,
    /// Lifetime signal count
    pub total_signals_released: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreatLevel;

    #[test]
    fn test_signal_bundle_creation() {
        let bundle = SignalBundle::new("bundle_1");
        assert!(bundle.is_empty());
        assert_eq!(bundle.state, BundleState::Forming);
    }

    #[test]
    fn test_signal_bundle_assembly() {
        let bundle = SignalBundle::new("test")
            .with_signal(Cytokine::alarm("alert_1"))
            .with_signal(Cytokine::alarm("alert_2"))
            .with_target("guardian");

        assert_eq!(bundle.size(), 2);
        assert_eq!(bundle.target, Some("guardian".to_string()));
    }

    #[test]
    fn test_bundle_lifecycle() {
        let mut bundle = SignalBundle::new("lifecycle");
        bundle.add(Cytokine::alarm("test"));

        assert_eq!(bundle.state, BundleState::Forming);

        bundle.seal();
        assert_eq!(bundle.state, BundleState::Docked);

        bundle.release();
        assert_eq!(bundle.state, BundleState::Released);

        bundle.acknowledge();
        assert_eq!(bundle.state, BundleState::Acknowledged);
    }

    #[test]
    fn test_always_open_gate() {
        let gate = AlwaysOpenGate;
        assert!(gate.is_open());
        assert!(gate.closed_reason().is_none());
    }

    #[test]
    fn test_rate_limit_gate() {
        let mut gate = RateLimitGate::new(2);
        assert!(gate.is_open());

        gate.record_release();
        assert!(gate.is_open());

        gate.record_release();
        assert!(!gate.is_open());
        assert!(gate.closed_reason().is_some());

        gate.reset_window();
        assert!(gate.is_open());
    }

    #[test]
    fn test_exocytic_emitter_release() {
        let mut emitter = ExocyticEmitter::new("test_emitter");

        // Build a bundle
        let bundle = emitter.begin_bundle();
        bundle.add(Cytokine::alarm("alert_1"));
        bundle.add(Cytokine::alarm("alert_2"));

        let gate = AlwaysOpenGate;
        let signals = emitter.release_all(&gate);

        assert_eq!(signals.len(), 2);
        assert_eq!(emitter.pending_count(), 0);
        assert_eq!(emitter.released_count(), 1);
    }

    #[test]
    fn test_exocytic_emitter_gate_blocked() {
        let mut emitter = ExocyticEmitter::new("blocked");

        let bundle = emitter.begin_bundle();
        bundle.add(Cytokine::alarm("test"));

        let mut gate = RateLimitGate::new(0); // Always closed

        let signals = emitter.release_all(&gate);
        assert!(signals.is_empty());
        assert_eq!(emitter.pending_count(), 1); // Still pending
    }

    #[test]
    fn test_exocytic_stats() {
        let mut emitter = ExocyticEmitter::new("stats_test");

        let bundle = emitter.begin_bundle();
        bundle.add(Cytokine::alarm("a"));
        bundle.add(Cytokine::alarm("b"));

        let gate = AlwaysOpenGate;
        emitter.release_all(&gate);

        let stats = emitter.stats();
        assert_eq!(stats.total_bundles_released, 1);
        assert_eq!(stats.total_signals_released, 2);
    }

    #[test]
    fn test_bundle_fail_state() {
        let mut bundle = SignalBundle::new("failing");
        bundle.add(Cytokine::alarm("test"));
        bundle.seal();
        bundle.fail();
        assert_eq!(bundle.state, BundleState::Failed);
    }
}
