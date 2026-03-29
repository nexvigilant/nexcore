//! Apoptosis - programmed cell death with graceful shutdown.
//!
//! ## T1 Grounding
//!
//! - `ApoptosisSignal` → ∅ (void) - termination/nullification
//! - `ShutdownPhase` → ∂ (boundary) - demarcation between alive and dead
//! - `GracePeriod` → N (quantity) - countdown timer
//! - `PostMortem` → ς (state) - final state capture before void
//!
//! ## Biological Analog
//!
//! Apoptosis is orderly, programmed cell death:
//! 1. **Initiation**: Death signal received (TNF-α, internal damage)
//! 2. **Execution**: Caspase cascade activates (irreversible commitment)
//! 3. **Dismantling**: Cell components are packaged for recycling
//! 4. **Clearance**: Neighboring cells absorb fragments (phagocytosis)
//!
//! ## Claude Code Analog
//!
//! Graceful process/component shutdown with state preservation:
//! - **Initiation**: TNF-α signal triggers shutdown sequence
//! - **Grace period**: Configurable timeout for in-flight work to complete
//! - **Post-mortem**: Capture final state, logs, metrics before termination
//! - **Cleanup**: Release resources, notify dependents

use crate::Cytokine;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Phase of the apoptotic shutdown process.
///
/// # Tier: T2-P
/// Grounds to: ∂ (boundary — each phase is a checkpoint)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownPhase {
    /// Normal operation — no shutdown initiated
    Alive,
    /// Death signal received, grace period started
    Initiated,
    /// Committed to shutdown, draining in-flight work
    Executing,
    /// Final state captured, releasing resources
    Dismantling,
    /// Shutdown complete, awaiting cleanup
    Dead,
}

impl ShutdownPhase {
    /// Check if shutdown is past the point of no return.
    pub fn is_committed(&self) -> bool {
        matches!(self, Self::Executing | Self::Dismantling | Self::Dead)
    }

    /// Check if the process is still alive.
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Alive)
    }

    /// Check if shutdown is complete.
    pub fn is_dead(&self) -> bool {
        matches!(self, Self::Dead)
    }
}

/// Post-mortem record captured before termination.
///
/// # Tier: T2-C
/// Grounds to: ς (state snapshot) + ∅ (captured at boundary of void)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMortem {
    /// Component that was terminated
    pub component_id: String,
    /// Why shutdown was triggered
    pub cause: String,
    /// When the death signal was received
    pub initiated_at: DateTime,
    /// When shutdown completed
    pub completed_at: Option<DateTime>,
    /// Final phase reached
    pub final_phase: ShutdownPhase,
    /// Arbitrary state snapshot
    pub state_snapshot: serde_json::Value,
    /// Whether shutdown was graceful
    pub graceful: bool,
}

/// Apoptotic shutdown controller.
///
/// Manages the orderly shutdown of a component with grace period
/// and post-mortem capture.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ∅ (void) + ∂ (boundary between alive/dead)
#[derive(Debug)]
pub struct ApoptosisController {
    /// Component being controlled
    component_id: String,
    /// Current shutdown phase
    phase: ShutdownPhase,
    /// Grace period in seconds before forced termination
    grace_period_secs: u32,
    /// When shutdown was initiated
    initiated_at: Option<DateTime>,
    /// The triggering signal
    trigger: Option<Cytokine>,
    /// Post-mortem record (built during shutdown)
    post_mortem: Option<PostMortem>,
    /// Callbacks registered for each phase transition
    phase_log: Vec<(ShutdownPhase, DateTime)>,
}

impl ApoptosisController {
    /// Create a new controller for a component.
    pub fn new(component_id: impl Into<String>, grace_period_secs: u32) -> Self {
        Self {
            component_id: component_id.into(),
            phase: ShutdownPhase::Alive,
            grace_period_secs,
            initiated_at: None,
            trigger: None,
            post_mortem: None,
            phase_log: Vec::new(),
        }
    }

    /// Get current phase.
    pub fn phase(&self) -> ShutdownPhase {
        self.phase
    }

    /// Get the component ID.
    pub fn component_id(&self) -> &str {
        &self.component_id
    }

    /// Initiate shutdown with a triggering signal.
    ///
    /// Returns `false` if already shutting down.
    pub fn initiate(&mut self, trigger: Cytokine) -> bool {
        if !self.phase.is_alive() {
            return false;
        }

        let now = DateTime::now();
        self.phase = ShutdownPhase::Initiated;
        self.initiated_at = Some(now);
        self.trigger = Some(trigger);
        self.phase_log.push((ShutdownPhase::Initiated, now));
        true
    }

    /// Advance to the next shutdown phase.
    ///
    /// Returns the new phase, or `None` if already dead.
    pub fn advance(&mut self) -> Option<ShutdownPhase> {
        let now = DateTime::now();
        let next = match self.phase {
            // Must call initiate() first
            ShutdownPhase::Alive | ShutdownPhase::Dead => return None,
            ShutdownPhase::Initiated => ShutdownPhase::Executing,
            ShutdownPhase::Executing => ShutdownPhase::Dismantling,
            ShutdownPhase::Dismantling => ShutdownPhase::Dead,
        };

        self.phase = next;
        self.phase_log.push((next, now));
        Some(next)
    }

    /// Check if the grace period has elapsed.
    pub fn grace_period_elapsed(&self) -> bool {
        self.initiated_at.is_some_and(|initiated| {
            let elapsed = DateTime::now()
                .signed_duration_since(initiated)
                .num_seconds();
            elapsed >= i64::from(self.grace_period_secs)
        })
    }

    /// Force immediate death (skip remaining phases).
    pub fn force_kill(&mut self) {
        let now = DateTime::now();
        self.phase = ShutdownPhase::Dead;
        self.phase_log.push((ShutdownPhase::Dead, now));
    }

    /// Complete shutdown and produce post-mortem record.
    pub fn complete(&mut self, state_snapshot: serde_json::Value) -> PostMortem {
        let now = DateTime::now();

        if self.phase != ShutdownPhase::Dead {
            self.phase = ShutdownPhase::Dead;
            self.phase_log.push((ShutdownPhase::Dead, now));
        }

        let pm = PostMortem {
            component_id: self.component_id.clone(),
            cause: self.trigger.as_ref().map_or_else(
                || "unknown".to_string(),
                |t| format!("{}: {}", t.family, t.name),
            ),
            initiated_at: self.initiated_at.unwrap_or(now),
            completed_at: Some(now),
            final_phase: ShutdownPhase::Dead,
            state_snapshot,
            graceful: self.phase_log.len() >= 4, // All phases traversed
        };

        self.post_mortem = Some(pm.clone());
        pm
    }

    /// Generate cleanup signals for dependents.
    ///
    /// Emits IL-10 (suppress) to calm cascades triggered by the death.
    pub fn cleanup_signals(&self) -> Vec<Cytokine> {
        if !self.phase.is_dead() {
            return Vec::new();
        }

        vec![
            Cytokine::suppress(format!("apoptosis_cleanup:{}", self.component_id))
                .with_source(&self.component_id),
        ]
    }

    /// Get the phase transition log.
    pub fn phase_log(&self) -> &[(ShutdownPhase, DateTime)] {
        &self.phase_log
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::CytokineFamily;

    #[test]
    fn test_shutdown_phase_lifecycle() {
        assert!(ShutdownPhase::Alive.is_alive());
        assert!(!ShutdownPhase::Alive.is_committed());
        assert!(!ShutdownPhase::Alive.is_dead());

        assert!(ShutdownPhase::Executing.is_committed());
        assert!(ShutdownPhase::Dismantling.is_committed());
        assert!(ShutdownPhase::Dead.is_dead());
    }

    #[test]
    fn test_apoptosis_full_lifecycle() {
        let mut ctrl = ApoptosisController::new("test_component", 30);
        assert_eq!(ctrl.phase(), ShutdownPhase::Alive);

        // Initiate with TNF-α
        let trigger = Cytokine::terminate("component_failed");
        assert!(ctrl.initiate(trigger));

        assert_eq!(ctrl.phase(), ShutdownPhase::Initiated);

        // Advance through phases
        assert_eq!(ctrl.advance(), Some(ShutdownPhase::Executing));
        assert!(ctrl.phase().is_committed());

        assert_eq!(ctrl.advance(), Some(ShutdownPhase::Dismantling));
        assert_eq!(ctrl.advance(), Some(ShutdownPhase::Dead));

        // Already dead — no more advancement
        assert_eq!(ctrl.advance(), None);
    }

    #[test]
    fn test_apoptosis_double_initiate() {
        let mut ctrl = ApoptosisController::new("test", 10);
        let trigger = Cytokine::terminate("first");
        assert!(ctrl.initiate(trigger));

        // Second initiation should fail
        let trigger2 = Cytokine::terminate("second");
        assert!(!ctrl.initiate(trigger2));
    }

    #[test]
    fn test_apoptosis_force_kill() {
        let mut ctrl = ApoptosisController::new("test", 60);
        let trigger = Cytokine::terminate("emergency");
        ctrl.initiate(trigger);

        ctrl.force_kill();
        assert!(ctrl.phase().is_dead());
    }

    #[test]
    fn test_post_mortem_generation() {
        let mut ctrl = ApoptosisController::new("widget_42", 5);
        let trigger = Cytokine::terminate("oom");
        ctrl.initiate(trigger);
        ctrl.advance(); // Executing
        ctrl.advance(); // Dismantling
        ctrl.advance(); // Dead

        let pm = ctrl.complete(serde_json::json!({"last_request": "GET /api"}));
        assert_eq!(pm.component_id, "widget_42");
        assert!(pm.completed_at.is_some());
        assert!(pm.graceful);
        assert_eq!(pm.final_phase, ShutdownPhase::Dead);
    }

    #[test]
    fn test_forced_kill_not_graceful() {
        let mut ctrl = ApoptosisController::new("test", 5);
        ctrl.initiate(Cytokine::terminate("panic"));
        ctrl.force_kill();

        let pm = ctrl.complete(serde_json::json!({}));
        assert!(!pm.graceful); // Skipped phases = not graceful
    }

    #[test]
    fn test_cleanup_signals() {
        let mut ctrl = ApoptosisController::new("dead_component", 5);
        ctrl.initiate(Cytokine::terminate("test"));

        // Not dead yet — no cleanup signals
        assert!(ctrl.cleanup_signals().is_empty());

        ctrl.force_kill();

        let signals = ctrl.cleanup_signals();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].family, CytokineFamily::Il10);
    }

    #[test]
    fn test_alive_cannot_advance() {
        let mut ctrl = ApoptosisController::new("test", 5);
        assert_eq!(ctrl.advance(), None); // Must initiate first
    }

    #[test]
    fn test_phase_log_tracking() {
        let mut ctrl = ApoptosisController::new("test", 5);
        ctrl.initiate(Cytokine::terminate("test"));
        ctrl.advance();
        ctrl.advance();
        ctrl.advance();

        let log = ctrl.phase_log();
        assert_eq!(log.len(), 4);
        assert_eq!(log[0].0, ShutdownPhase::Initiated);
        assert_eq!(log[3].0, ShutdownPhase::Dead);
    }
}
