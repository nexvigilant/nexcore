// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NeuroendocrineCoordinator
//!
//! **Tier**: T3 (causality + mu + rho + N + lambda + pi + state)
//! **Bridge**: cytokine (fast signaling) + hormones (slow state modulation)
//! **Confidence**: 0.89
//!
//! Unifies fast event-driven signaling with slow persistent state changes.
//! Burst of fast signals → triggers slow state shift.
//! Slow state thresholds → enables/disables fast signal families.

use core::fmt;

/// Fast signal severity (cytokine-equivalent).
pub type SignalSeverity = u8;

/// A fast signal event.
///
/// ## Tier: T2-P (causality + N)
#[derive(Debug, Clone)]
pub struct FastSignal {
    /// Signal family/category.
    pub family: String,
    /// Severity (0-255).
    pub severity: SignalSeverity,
    /// Monotonic timestamp.
    pub timestamp: u64,
}

/// Slow state level (hormone-equivalent).
///
/// ## Tier: T2-P (state + N)
#[derive(Debug, Clone, Copy)]
pub struct SlowState {
    /// Current level (0.0 to 1.0).
    pub level: f64,
    /// Baseline level (natural resting state).
    pub baseline: f64,
    /// Decay rate per tick (how fast it returns to baseline).
    pub decay_rate: f64,
}

impl SlowState {
    /// Create a new slow state.
    #[must_use]
    pub fn new(baseline: f64) -> Self {
        Self {
            level: baseline,
            baseline,
            decay_rate: 0.05,
        }
    }

    /// Apply a stimulus (increase level).
    pub fn stimulate(&mut self, amount: f64) {
        self.level = (self.level + amount).min(1.0);
    }

    /// Decay toward baseline by one tick.
    pub fn decay(&mut self) {
        let diff = self.level - self.baseline;
        self.level -= diff * self.decay_rate;
    }

    /// Whether level exceeds a threshold.
    #[must_use]
    pub fn exceeds(&self, threshold: f64) -> bool {
        self.level > threshold
    }
}

/// An amplification rule: fast signals → slow state changes.
///
/// ## Tier: T2-C (causality + mu + N)
#[derive(Debug, Clone)]
pub struct AmplificationRule {
    /// Fast signal family that triggers this rule.
    pub signal_family: String,
    /// Slow state to modify.
    pub target_state: String,
    /// Minimum burst count to trigger.
    pub burst_threshold: usize,
    /// Amount to stimulate the slow state.
    pub stimulus_amount: f64,
}

/// Coordinates fast (event) and slow (persistent state) signaling.
///
/// ## Tier: T3 (causality + mu + rho + N + lambda + pi + state)
/// Dominant: causality (this is fundamentally about cause→effect across timescales)
///
/// Innovation: bridges cytokine and hormones crates.
#[derive(Debug, Clone)]
pub struct NeuroendocrineCoordinator {
    /// Slow state channels by name.
    states: std::collections::BTreeMap<String, SlowState>,
    /// Amplification rules.
    rules: Vec<AmplificationRule>,
    /// Recent fast signals (ring buffer).
    signal_buffer: Vec<FastSignal>,
    /// Buffer capacity.
    buffer_capacity: usize,
    /// Tick counter.
    tick_count: u64,
}

impl NeuroendocrineCoordinator {
    /// Create a new coordinator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: std::collections::BTreeMap::new(),
            rules: Vec::new(),
            signal_buffer: Vec::new(),
            buffer_capacity: 100,
            tick_count: 0,
        }
    }

    /// Register a slow state channel.
    pub fn register_state(&mut self, name: impl Into<String>, baseline: f64) {
        self.states.insert(name.into(), SlowState::new(baseline));
    }

    /// Add an amplification rule.
    pub fn add_rule(&mut self, rule: AmplificationRule) {
        self.rules.push(rule);
    }

    /// Receive a fast signal.
    pub fn on_signal(&mut self, signal: FastSignal) {
        self.signal_buffer.push(signal);

        // Trim buffer
        if self.signal_buffer.len() > self.buffer_capacity {
            self.signal_buffer.remove(0);
        }

        // Check burst rules
        self.evaluate_bursts();
    }

    /// Advance time by one tick — decay all slow states.
    pub fn tick(&mut self) {
        self.tick_count += 1;
        for state in self.states.values_mut() {
            state.decay();
        }
    }

    /// Get a slow state level.
    #[must_use]
    pub fn state_level(&self, name: &str) -> Option<f64> {
        self.states.get(name).map(|s| s.level)
    }

    /// Check if a slow state exceeds a threshold.
    #[must_use]
    pub fn state_exceeds(&self, name: &str, threshold: f64) -> bool {
        self.states.get(name).is_some_and(|s| s.exceeds(threshold))
    }

    /// Check if the system is in crisis mode.
    /// Crisis = any state > 0.8.
    #[must_use]
    pub fn is_crisis(&self) -> bool {
        self.states.values().any(|s| s.level > 0.8)
    }

    /// Evaluate burst rules against recent signal buffer.
    fn evaluate_bursts(&mut self) {
        for rule in &self.rules {
            let burst_count = self
                .signal_buffer
                .iter()
                .filter(|s| s.family == rule.signal_family)
                .count();

            if burst_count >= rule.burst_threshold
                && let Some(state) = self.states.get_mut(&rule.target_state)
            {
                state.stimulate(rule.stimulus_amount);
            }
        }
    }
}

impl Default for NeuroendocrineCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NeuroendocrineCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NeuroendocrineCoordinator({} states, {} rules, tick {})",
            self.states.len(),
            self.rules.len(),
            self.tick_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_state_decay() {
        let mut state = SlowState::new(0.5);
        state.stimulate(0.3); // 0.8
        assert!((state.level - 0.8).abs() < 1e-10);

        state.decay();
        assert!(state.level < 0.8); // decaying toward 0.5
    }

    #[test]
    fn test_burst_triggers_state_change() {
        let mut coord = NeuroendocrineCoordinator::new();
        coord.register_state("stress", 0.3);
        coord.add_rule(AmplificationRule {
            signal_family: "error".into(),
            target_state: "stress".into(),
            burst_threshold: 3,
            stimulus_amount: 0.2,
        });

        // Send 3 error signals (burst threshold)
        for i in 0..3 {
            coord.on_signal(FastSignal {
                family: "error".into(),
                severity: 5,
                timestamp: i,
            });
        }

        // Stress should have increased
        let stress = coord.state_level("stress").unwrap_or(0.0);
        assert!(stress > 0.3, "Stress should increase from burst: {stress}");
    }

    #[test]
    fn test_decay_over_ticks() {
        let mut coord = NeuroendocrineCoordinator::new();
        coord.register_state("adrenaline", 0.2);

        // Manually stimulate
        if let Some(state) = coord.states.get_mut("adrenaline") {
            state.stimulate(0.6); // jump to 0.8
        }

        // Decay over many ticks
        for _ in 0..100 {
            coord.tick();
        }

        let level = coord.state_level("adrenaline").unwrap_or(1.0);
        assert!(
            (level - 0.2).abs() < 0.05,
            "Should decay back near baseline: {level}"
        );
    }

    #[test]
    fn test_crisis_detection() {
        let mut coord = NeuroendocrineCoordinator::new();
        coord.register_state("cortisol", 0.3);

        assert!(!coord.is_crisis());

        if let Some(state) = coord.states.get_mut("cortisol") {
            state.stimulate(0.6); // 0.9 > 0.8 crisis
        }

        assert!(coord.is_crisis());
    }
}
