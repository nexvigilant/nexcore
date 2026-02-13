// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # DampedOscillator
//!
//! **Tier**: T2-C (nu + rho + N + partial)
//! **Fills pair gap**: Frequency x Recursion (previously unexplored)
//!
//! Models recursive frequency convergence — a value that oscillates
//! around a target, with each recursive iteration reducing the amplitude.
//!
//! Physical analog: spring-mass system with damping.
//! Computing analog: PID controller overshoot, adaptive threshold convergence.

use core::fmt;

/// Damping regime.
///
/// ## Tier: T2-P (nu + kappa)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DampingRegime {
    /// System oscillates with decreasing amplitude (0 < zeta < 1).
    Underdamped,
    /// System returns to equilibrium without oscillating (zeta = 1).
    CriticallyDamped,
    /// System returns slowly without oscillating (zeta > 1).
    Overdamped,
}

/// State of the oscillator at a given iteration.
#[derive(Debug, Clone, Copy)]
pub struct OscillatorState {
    /// Current value.
    pub value: f64,
    /// Current velocity (rate of change).
    pub velocity: f64,
    /// Iteration number.
    pub iteration: u64,
    /// Current amplitude of oscillation.
    pub amplitude: f64,
}

/// A damped oscillator that converges recursively.
///
/// ## Tier: T2-C (nu + rho + N + partial)
/// Dominant: nu (Frequency)
///
/// Innovation: fills the Frequency x Recursion gap.
/// Each recursive step reduces the oscillation frequency/amplitude
/// until convergence is reached within an epsilon boundary.
#[derive(Debug, Clone)]
pub struct DampedOscillator {
    /// Target equilibrium value.
    target: f64,
    /// Current value.
    current: f64,
    /// Current velocity.
    velocity: f64,
    /// Damping ratio (zeta). 0-1: underdamped, 1: critical, >1: overdamped.
    damping_ratio: f64,
    /// Natural frequency (omega_n). Higher = faster oscillation.
    natural_frequency: f64,
    /// Convergence epsilon.
    epsilon: f64,
    /// Iteration counter.
    iteration: u64,
    /// History of states.
    history: Vec<OscillatorState>,
    /// Maximum iterations.
    max_iterations: u64,
}

impl DampedOscillator {
    /// Create a new damped oscillator.
    ///
    /// - `initial`: starting value
    /// - `target`: equilibrium value to converge toward
    /// - `damping_ratio`: 0-1 underdamped, 1 critical, >1 overdamped
    /// - `natural_frequency`: oscillation speed (0.1 = slow, 1.0 = fast)
    #[must_use]
    pub fn new(initial: f64, target: f64, damping_ratio: f64, natural_frequency: f64) -> Self {
        let state = OscillatorState {
            value: initial,
            velocity: 0.0,
            iteration: 0,
            amplitude: (initial - target).abs(),
        };

        Self {
            target,
            current: initial,
            velocity: 0.0,
            damping_ratio: damping_ratio.max(0.0),
            natural_frequency: natural_frequency.max(0.01),
            epsilon: 0.001,
            iteration: 0,
            history: vec![state],
            max_iterations: 1000,
        }
    }

    /// Set convergence epsilon.
    #[must_use]
    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon.abs().max(1e-10);
        self
    }

    /// Set maximum iterations.
    #[must_use]
    pub fn with_max_iterations(mut self, max: u64) -> Self {
        self.max_iterations = max.max(1);
        self
    }

    /// Execute one iteration of the oscillator.
    ///
    /// Uses the damped harmonic oscillator equation:
    /// x'' + 2*zeta*omega_n*x' + omega_n^2*(x - target) = 0
    pub fn step(&mut self) -> OscillatorState {
        let displacement = self.current - self.target;
        let omega_n = self.natural_frequency;
        let zeta = self.damping_ratio;

        // Acceleration from spring + damping forces
        let acceleration = -2.0 * zeta * omega_n * self.velocity - omega_n * omega_n * displacement;

        // Euler integration (simple but effective for discrete steps)
        self.velocity += acceleration;
        self.current += self.velocity;
        self.iteration += 1;

        let state = OscillatorState {
            value: self.current,
            velocity: self.velocity,
            iteration: self.iteration,
            amplitude: (self.current - self.target).abs(),
        };

        self.history.push(state);
        state
    }

    /// Run until convergence or max iterations.
    /// Returns the number of iterations taken.
    pub fn converge(&mut self) -> u64 {
        let start = self.iteration;

        while self.iteration < self.max_iterations {
            let state = self.step();
            if state.amplitude < self.epsilon && state.velocity.abs() < self.epsilon {
                break;
            }
        }

        self.iteration - start
    }

    /// Whether the oscillator has converged.
    #[must_use]
    pub fn is_converged(&self) -> bool {
        let amplitude = (self.current - self.target).abs();
        amplitude < self.epsilon && self.velocity.abs() < self.epsilon
    }

    /// Current damping regime.
    #[must_use]
    pub fn regime(&self) -> DampingRegime {
        if self.damping_ratio < 1.0 {
            DampingRegime::Underdamped
        } else if (self.damping_ratio - 1.0).abs() < 1e-10 {
            DampingRegime::CriticallyDamped
        } else {
            DampingRegime::Overdamped
        }
    }

    /// Current value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.current
    }

    /// Target value.
    #[must_use]
    pub fn target(&self) -> f64 {
        self.target
    }

    /// Current iteration.
    #[must_use]
    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    /// Current amplitude (distance from target).
    #[must_use]
    pub fn amplitude(&self) -> f64 {
        (self.current - self.target).abs()
    }

    /// State history.
    #[must_use]
    pub fn history(&self) -> &[OscillatorState] {
        &self.history
    }
}

impl fmt::Display for DampedOscillator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DampedOscillator({:.3} -> {:.3}, {:?}, iter {}, amp {:.4})",
            self.current,
            self.target,
            self.regime(),
            self.iteration,
            self.amplitude(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underdamped_oscillation() {
        let mut osc = DampedOscillator::new(10.0, 0.0, 0.3, 0.5).with_epsilon(0.01);

        // Should oscillate past target
        let mut crossed_zero = false;
        for _ in 0..50 {
            let state = osc.step();
            if state.value < 0.0 {
                crossed_zero = true;
                break;
            }
        }

        assert!(crossed_zero, "Underdamped should oscillate past target");
        assert_eq!(osc.regime(), DampingRegime::Underdamped);
    }

    #[test]
    fn test_critically_damped_no_overshoot() {
        let mut osc = DampedOscillator::new(10.0, 0.0, 1.0, 0.3).with_epsilon(0.01);

        // Critically damped should NOT overshoot significantly
        for _ in 0..100 {
            osc.step();
        }

        // Should converge without going far below zero
        // (small numerical overshoot is OK)
        let min_value = osc
            .history()
            .iter()
            .map(|s| s.value)
            .fold(f64::MAX, f64::min);

        assert!(
            min_value > -1.0,
            "Critically damped should barely overshoot"
        );
        assert_eq!(osc.regime(), DampingRegime::CriticallyDamped);
    }

    #[test]
    fn test_convergence() {
        let mut osc = DampedOscillator::new(5.0, 0.0, 0.7, 0.4)
            .with_epsilon(0.01)
            .with_max_iterations(500);

        let iterations = osc.converge();
        assert!(osc.is_converged());
        assert!(iterations > 0);
        assert!(osc.amplitude() < 0.01);
    }

    #[test]
    fn test_overdamped() {
        let osc = DampedOscillator::new(10.0, 0.0, 2.0, 0.3);
        assert_eq!(osc.regime(), DampingRegime::Overdamped);
    }

    #[test]
    fn test_history_tracking() {
        let mut osc = DampedOscillator::new(5.0, 0.0, 0.5, 0.3);

        for _ in 0..10 {
            osc.step();
        }

        assert_eq!(osc.history().len(), 11); // initial + 10 steps
        assert_eq!(osc.iteration(), 10);
    }
}
