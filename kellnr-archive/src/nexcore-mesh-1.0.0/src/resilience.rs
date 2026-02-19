//! Resilience: homeostasis-driven mesh health maintenance.
//!
//! ## Primitive Foundation
//! - `ResilienceConfig`: ∂ (Boundary) — thresholds and limits
//! - `ResilienceLoop`: ρ (Recursion) dominant — self-correcting feedback loop
//!
//! ## Transfer Primitive Reuse
//! - `Homeostasis` — maintain target neighbor count within tolerance
//! - `FeedbackLoop` — adjust discovery rate based on neighbor delta

use nexcore_primitives::transfer::{FeedbackLoop, Homeostasis};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// ResilienceConfig — Thresholds for mesh health
// ============================================================================

/// Configuration for the resilience subsystem.
///
/// Tier: T2-C | Dominant: ∂ (Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Target number of healthy neighbors
    pub target_neighbors: usize,
    /// Acceptable deviation from target (homeostasis tolerance)
    pub neighbor_tolerance: f64,
    /// Correction gain for homeostasis (how aggressively to correct)
    pub correction_gain: f64,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Number of missed heartbeats before marking neighbor as failed
    pub missed_heartbeat_threshold: u64,
    /// Circuit breaker cooldown before attempting reset
    pub breaker_cooldown: Duration,
    /// Feedback loop gain for discovery rate adjustment
    pub discovery_feedback_gain: f64,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            target_neighbors: 8,
            neighbor_tolerance: 2.0,
            correction_gain: 0.5,
            heartbeat_interval: Duration::from_secs(3),
            missed_heartbeat_threshold: 3,
            breaker_cooldown: Duration::from_secs(30),
            discovery_feedback_gain: 0.3,
        }
    }
}

// ============================================================================
// ResilienceState — Current health metrics
// ============================================================================

/// Snapshot of current mesh health metrics.
///
/// Tier: T2-C | Dominant: κ (Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceState {
    /// Current number of reachable neighbors
    pub reachable_neighbors: usize,
    /// Current number of unreachable neighbors (circuit breaker open)
    pub unreachable_neighbors: usize,
    /// Total heartbeats sent
    pub heartbeats_sent: u64,
    /// Total heartbeats received (acks)
    pub heartbeats_received: u64,
    /// Number of circuit breaker trips
    pub breaker_trips: u64,
    /// Number of circuit breaker resets
    pub breaker_resets: u64,
    /// Current homeostasis error (distance from target)
    pub homeostasis_error: f64,
    /// Whether the mesh is in a healthy state
    pub is_healthy: bool,
}

impl Default for ResilienceState {
    fn default() -> Self {
        Self {
            reachable_neighbors: 0,
            unreachable_neighbors: 0,
            heartbeats_sent: 0,
            heartbeats_received: 0,
            breaker_trips: 0,
            breaker_resets: 0,
            homeostasis_error: 0.0,
            is_healthy: false,
        }
    }
}

// ============================================================================
// ResilienceLoop — Self-correcting health maintenance
// ============================================================================

/// Self-correcting mesh health loop using transfer primitives.
///
/// Tier: T3 | Dominant: ρ (Recursion)
///
/// Uses:
/// - `Homeostasis` (from nexcore-primitives) to maintain target neighbor count
/// - `FeedbackLoop` to adjust discovery announce rate
///
/// The loop periodically:
/// 1. Senses current neighbor count
/// 2. Compares against homeostasis setpoint
/// 3. Adjusts discovery rate (faster when below target, slower when above)
/// 4. Attempts circuit breaker resets on downed links
#[derive(Debug)]
pub struct ResilienceLoop {
    /// Node ID of the owning node
    pub node_id: String,
    /// Configuration
    pub config: ResilienceConfig,
    /// Homeostasis controller for neighbor count
    pub homeostasis: Homeostasis,
    /// Feedback loop for discovery rate adjustment
    pub discovery_feedback: FeedbackLoop,
    /// Current state metrics
    pub state: ResilienceState,
    /// Number of resilience ticks completed
    pub ticks: u64,
    /// Whether the loop is running
    pub running: bool,
}

impl ResilienceLoop {
    /// Create a new resilience loop.
    pub fn new(node_id: impl Into<String>, config: ResilienceConfig) -> Self {
        let homeostasis = Homeostasis::new(
            config.target_neighbors as f64,
            config.neighbor_tolerance,
            config.correction_gain,
        );
        let discovery_feedback = FeedbackLoop::new(
            config.target_neighbors as f64,
            config.discovery_feedback_gain,
        );

        Self {
            node_id: node_id.into(),
            config,
            homeostasis,
            discovery_feedback,
            state: ResilienceState::default(),
            ticks: 0,
            running: false,
        }
    }

    /// Perform one resilience tick: sense, compare, decide.
    ///
    /// Returns a `ResilienceAction` indicating what the node should do.
    pub fn tick(&mut self, reachable_count: usize, unreachable_count: usize) -> ResilienceAction {
        self.ticks += 1;

        // Update homeostasis with current neighbor count
        self.homeostasis.current = reachable_count as f64;
        let error = self.homeostasis.error();
        let in_tolerance = self.homeostasis.in_tolerance();

        // Update feedback loop
        self.discovery_feedback.current = reachable_count as f64;

        // Update state
        self.state.reachable_neighbors = reachable_count;
        self.state.unreachable_neighbors = unreachable_count;
        self.state.homeostasis_error = error;
        self.state.is_healthy = in_tolerance && reachable_count > 0;

        // Decide action
        if reachable_count == 0 && unreachable_count > 0 {
            // All neighbors down — attempt resets
            self.state.breaker_resets += 1;
            return ResilienceAction::ResetBreakers;
        }

        if !in_tolerance {
            if error > 0.0 {
                // Below target: need more neighbors
                let urgency = (error / self.config.target_neighbors as f64).min(1.0);
                return ResilienceAction::AccelerateDiscovery { urgency };
            }
            // Above target: too many neighbors (rare, but handle)
            return ResilienceAction::DecelerateDiscovery;
        }

        if unreachable_count > 0 {
            // Some down but within tolerance — try periodic resets
            if self.ticks % 10 == 0 {
                self.state.breaker_resets += 1;
                return ResilienceAction::ResetBreakers;
            }
        }

        ResilienceAction::Nominal
    }

    /// Record that a heartbeat was sent.
    pub fn record_heartbeat_sent(&mut self) {
        self.state.heartbeats_sent += 1;
    }

    /// Record that a heartbeat ack was received.
    pub fn record_heartbeat_received(&mut self) {
        self.state.heartbeats_received += 1;
    }

    /// Record a circuit breaker trip.
    pub fn record_breaker_trip(&mut self) {
        self.state.breaker_trips += 1;
    }

    /// Get the recommended discovery interval multiplier.
    ///
    /// Returns a value where:
    /// - < 1.0 means discover faster (below target)
    /// - = 1.0 means nominal rate
    /// - > 1.0 means discover slower (above target)
    pub fn discovery_rate_multiplier(&self) -> f64 {
        let correction = self.discovery_feedback.correction();
        if correction.abs() < f64::EPSILON {
            return 1.0;
        }
        // Invert: negative correction (below target) → faster (< 1.0)
        let multiplier = 1.0 / (1.0 + correction.abs());
        if correction > 0.0 {
            multiplier // below target: faster
        } else {
            1.0 / multiplier // above target: slower
        }
    }

    /// Start the resilience loop.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the resilience loop.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get current health assessment.
    pub fn health_summary(&self) -> ResilienceState {
        self.state.clone()
    }
}

/// Action recommended by the resilience loop.
#[derive(Debug, Clone, PartialEq)]
pub enum ResilienceAction {
    /// Everything is nominal — no action needed
    Nominal,
    /// Speed up discovery to find more neighbors
    AccelerateDiscovery {
        /// How urgent (0.0 to 1.0, where 1.0 = max urgency)
        urgency: f64,
    },
    /// Slow down discovery (above target neighbor count)
    DecelerateDiscovery,
    /// Attempt to reset all open circuit breakers
    ResetBreakers,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_loop() -> ResilienceLoop {
        ResilienceLoop::new("node-1", ResilienceConfig::default())
    }

    #[test]
    fn resilience_config_default() {
        let cfg = ResilienceConfig::default();
        assert_eq!(cfg.target_neighbors, 8);
        assert!((cfg.neighbor_tolerance - 2.0).abs() < f64::EPSILON);
        assert_eq!(cfg.missed_heartbeat_threshold, 3);
    }

    #[test]
    fn resilience_loop_nominal_when_on_target() {
        let mut rl = default_loop();
        // Target is 8, tolerance is 2 → range [6, 10]
        let action = rl.tick(8, 0);
        assert_eq!(action, ResilienceAction::Nominal);
        assert!(rl.state.is_healthy);
    }

    #[test]
    fn resilience_loop_nominal_within_tolerance() {
        let mut rl = default_loop();
        let action = rl.tick(7, 0); // 7 is within [6, 10]
        assert_eq!(action, ResilienceAction::Nominal);
        assert!(rl.state.is_healthy);
    }

    #[test]
    fn resilience_loop_accelerate_below_target() {
        let mut rl = default_loop();
        let action = rl.tick(3, 0); // 3 < 6 (target - tolerance)
        assert!(matches!(
            action,
            ResilienceAction::AccelerateDiscovery { .. }
        ));
        assert!(!rl.state.is_healthy);
    }

    #[test]
    fn resilience_loop_decelerate_above_target() {
        let mut rl = default_loop();
        let action = rl.tick(15, 0); // 15 > 10 (target + tolerance)
        assert_eq!(action, ResilienceAction::DecelerateDiscovery);
    }

    #[test]
    fn resilience_loop_reset_breakers_all_down() {
        let mut rl = default_loop();
        let action = rl.tick(0, 5); // zero reachable, some unreachable
        assert_eq!(action, ResilienceAction::ResetBreakers);
    }

    #[test]
    fn resilience_loop_tracks_ticks() {
        let mut rl = default_loop();
        rl.tick(8, 0);
        rl.tick(8, 0);
        rl.tick(8, 0);
        assert_eq!(rl.ticks, 3);
    }

    #[test]
    fn resilience_loop_heartbeat_tracking() {
        let mut rl = default_loop();
        rl.record_heartbeat_sent();
        rl.record_heartbeat_sent();
        rl.record_heartbeat_received();
        assert_eq!(rl.state.heartbeats_sent, 2);
        assert_eq!(rl.state.heartbeats_received, 1);
    }

    #[test]
    fn resilience_loop_breaker_trip_tracking() {
        let mut rl = default_loop();
        rl.record_breaker_trip();
        rl.record_breaker_trip();
        assert_eq!(rl.state.breaker_trips, 2);
    }

    #[test]
    fn resilience_loop_discovery_rate_nominal() {
        let mut rl = default_loop();
        // Tick with target neighbor count to reach nominal state
        rl.tick(8, 0);
        let rate = rl.discovery_rate_multiplier();
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn resilience_loop_start_stop() {
        let mut rl = default_loop();
        assert!(!rl.running);
        rl.start();
        assert!(rl.running);
        rl.stop();
        assert!(!rl.running);
    }

    #[test]
    fn resilience_loop_health_summary() {
        let mut rl = default_loop();
        rl.tick(8, 1);
        let summary = rl.health_summary();
        assert_eq!(summary.reachable_neighbors, 8);
        assert_eq!(summary.unreachable_neighbors, 1);
        assert!(summary.is_healthy);
    }

    #[test]
    fn resilience_state_default() {
        let state = ResilienceState::default();
        assert_eq!(state.reachable_neighbors, 0);
        assert!(!state.is_healthy);
    }

    #[test]
    fn resilience_loop_accelerate_urgency_proportional() {
        let mut rl = default_loop();
        let action_mild = rl.tick(5, 0); // slightly below
        let urgency_mild = match action_mild {
            ResilienceAction::AccelerateDiscovery { urgency } => urgency,
            _ => 0.0,
        };

        let mut rl2 = default_loop();
        let action_severe = rl2.tick(1, 0); // severely below
        let urgency_severe = match action_severe {
            ResilienceAction::AccelerateDiscovery { urgency } => urgency,
            _ => 0.0,
        };

        assert!(urgency_severe > urgency_mild);
    }

    #[test]
    fn resilience_loop_periodic_reset_with_unreachable() {
        let mut rl = default_loop();
        // Within tolerance (7 reachable) but 2 unreachable
        // Should get ResetBreakers on tick 10
        for i in 1..=10 {
            let action = rl.tick(7, 2);
            if i == 10 {
                assert_eq!(action, ResilienceAction::ResetBreakers);
            } else {
                assert_eq!(action, ResilienceAction::Nominal);
            }
        }
    }

    #[test]
    fn resilience_loop_not_healthy_with_zero_reachable() {
        let mut rl = default_loop();
        rl.tick(0, 0);
        assert!(!rl.state.is_healthy);
    }
}
