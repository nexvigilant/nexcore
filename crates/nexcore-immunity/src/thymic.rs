// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Thymic selection — observation mode for NMD calibration.
//!
//! ## Biology Analog
//!
//! In biology, T-cells undergo thymic selection before entering circulation:
//! - **Positive selection**: Cells that recognize self-MHC survive
//! - **Negative selection**: Cells that react too strongly to self are eliminated
//!
//! This dual filter prevents autoimmune disease (overreacting to normal
//! tissue) while ensuring functional immunity.
//!
//! ## Purpose
//!
//! For the first N pipeline runs per task category, the NMD system operates
//! in observation mode: anomalies trigger `Stall` verdicts only (never
//! `Degrade`). This prevents the surveillance system from aborting pipelines
//! before it has calibrated its thresholds to the actual execution patterns.
//!
//! After the observation window completes, the accumulated statistics can
//! inform threshold adjustments via the adaptive learning engine.
//!
//! ## Primitive Grounding: ∂(Boundary) + ν(Frequency) + ς(State)

use crate::co_translational::UpfVerdict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for thymic selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThymicConfig {
    /// Number of observation runs before allowing degradation (per category).
    pub observation_window: u32,
    /// Whether thymic selection is globally enabled.
    pub enabled: bool,
}

impl Default for ThymicConfig {
    fn default() -> Self {
        Self {
            observation_window: 20,
            enabled: true,
        }
    }
}

/// Per-category observation state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryObservation {
    /// Number of runs observed for this category.
    pub runs_observed: u32,
    /// Number of verdicts that would have been Degrade.
    pub suppressed_degrades: u32,
    /// Number of stalls observed.
    pub stalls_observed: u32,
    /// Whether this category has graduated from observation.
    pub graduated: bool,
}

/// The thymic selection gate.
///
/// Wraps UPF verdicts: during the observation window, `Degrade` verdicts
/// are downgraded to `Stall` (observation only, no pipeline abort).
/// After graduation, verdicts pass through unmodified.
#[derive(Debug, Clone)]
pub struct ThymicGate {
    config: ThymicConfig,
    observations: HashMap<String, CategoryObservation>,
}

impl Default for ThymicGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ThymicGate {
    /// Create with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ThymicConfig::default(),
            observations: HashMap::new(),
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: ThymicConfig) -> Self {
        Self {
            config,
            observations: HashMap::new(),
        }
    }

    /// Filter a UPF verdict through thymic selection.
    ///
    /// - If disabled: pass through unchanged.
    /// - If category has graduated: pass through unchanged.
    /// - If in observation window: `Degrade` → `Stall`, others unchanged.
    ///
    /// Always increments the run counter for the category.
    pub fn filter_verdict(&mut self, category: &str, verdict: UpfVerdict) -> UpfVerdict {
        if !self.config.enabled {
            return verdict;
        }

        let obs = self
            .observations
            .entry(category.to_string())
            .or_default();

        obs.runs_observed += 1;

        // Check if graduated
        if obs.graduated {
            return verdict;
        }

        // Check if this run completes the observation window
        if obs.runs_observed >= self.config.observation_window {
            obs.graduated = true;
            // This run is the graduation run — still allow full verdict
            return verdict;
        }

        // In observation window: downgrade Degrade to Stall
        match verdict {
            UpfVerdict::Degrade { anomalies } => {
                obs.suppressed_degrades += 1;
                UpfVerdict::Stall { anomalies }
            }
            UpfVerdict::Stall { .. } => {
                obs.stalls_observed += 1;
                verdict
            }
            UpfVerdict::Continue => verdict,
        }
    }

    /// Check if a category has graduated from observation.
    #[must_use]
    pub fn is_graduated(&self, category: &str) -> bool {
        self.observations
            .get(category)
            .is_some_and(|obs| obs.graduated)
    }

    /// Get observation state for a category.
    #[must_use]
    pub fn observation(&self, category: &str) -> Option<&CategoryObservation> {
        self.observations.get(category)
    }

    /// Get all category observations.
    #[must_use]
    pub fn all_observations(&self) -> &HashMap<String, CategoryObservation> {
        &self.observations
    }

    /// Get the suppressed degradation rate for a category.
    ///
    /// This indicates how often the UPF would have aborted during observation.
    /// A high rate suggests thresholds may need loosening for this category.
    #[must_use]
    pub fn suppressed_degrade_rate(&self, category: &str) -> f32 {
        self.observations.get(category).map_or(0.0, |obs| {
            if obs.runs_observed == 0 {
                0.0
            } else {
                #[allow(clippy::cast_precision_loss)] // u32 values within f32 mantissa range for practical use
                { obs.suppressed_degrades as f32 / obs.runs_observed as f32 }
            }
        })
    }

    /// Manually graduate a category (skip remaining observation).
    pub fn force_graduate(&mut self, category: &str) {
        let obs = self
            .observations
            .entry(category.to_string())
            .or_default();
        obs.graduated = true;
    }

    /// Reset observation for a category (re-enter observation mode).
    pub fn reset_category(&mut self, category: &str) {
        self.observations.remove(category);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::co_translational::{UpfAnomaly, UpfChannel};

    fn make_anomalies() -> Vec<UpfAnomaly> {
        vec![UpfAnomaly {
            channel: UpfChannel::Upf2,
            description: "test anomaly".to_string(),
            severity: 0.8,
        }]
    }

    #[test]
    fn test_continue_passes_through() {
        let mut gate = ThymicGate::new();
        let result = gate.filter_verdict("Explore", UpfVerdict::Continue);
        assert!(matches!(result, UpfVerdict::Continue));
    }

    #[test]
    fn test_degrade_suppressed_during_observation() {
        let mut gate = ThymicGate::new();
        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };

        let result = gate.filter_verdict("Explore", verdict);

        // Should be downgraded to Stall
        assert!(matches!(result, UpfVerdict::Stall { .. }));
        assert!(!gate.is_graduated("Explore"));

        let obs = gate.observation("Explore");
        assert!(obs.is_some());
        assert_eq!(obs.map(|o| o.suppressed_degrades).unwrap_or(0), 1);
    }

    #[test]
    fn test_stall_passes_through_during_observation() {
        let mut gate = ThymicGate::new();
        let verdict = UpfVerdict::Stall {
            anomalies: make_anomalies(),
        };

        let result = gate.filter_verdict("Mutate", verdict);
        assert!(matches!(result, UpfVerdict::Stall { .. }));

        let obs = gate.observation("Mutate");
        assert_eq!(obs.map(|o| o.stalls_observed).unwrap_or(0), 1);
    }

    #[test]
    fn test_graduation_after_window() {
        let config = ThymicConfig {
            observation_window: 3,
            enabled: true,
        };
        let mut gate = ThymicGate::with_config(config);

        // Runs 1-2: in observation, Degrade → Stall
        for _ in 0..2 {
            let verdict = UpfVerdict::Degrade {
                anomalies: make_anomalies(),
            };
            let result = gate.filter_verdict("Compute", verdict);
            assert!(matches!(result, UpfVerdict::Stall { .. }));
        }

        assert!(!gate.is_graduated("Compute"));

        // Run 3: reaches window → graduates, Degrade passes through
        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };
        let result = gate.filter_verdict("Compute", verdict);
        assert!(matches!(result, UpfVerdict::Degrade { .. }));
        assert!(gate.is_graduated("Compute"));
    }

    #[test]
    fn test_post_graduation_degrade_passes() {
        let config = ThymicConfig {
            observation_window: 2,
            enabled: true,
        };
        let mut gate = ThymicGate::with_config(config);

        // Graduate
        gate.filter_verdict("Explore", UpfVerdict::Continue);
        gate.filter_verdict("Explore", UpfVerdict::Continue);
        assert!(gate.is_graduated("Explore"));

        // Post-graduation: Degrade passes through
        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };
        let result = gate.filter_verdict("Explore", verdict);
        assert!(matches!(result, UpfVerdict::Degrade { .. }));
    }

    #[test]
    fn test_disabled_gate_passes_all() {
        let config = ThymicConfig {
            observation_window: 100,
            enabled: false,
        };
        let mut gate = ThymicGate::with_config(config);

        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };
        let result = gate.filter_verdict("Explore", verdict);
        assert!(matches!(result, UpfVerdict::Degrade { .. }));
    }

    #[test]
    fn test_force_graduate() {
        let mut gate = ThymicGate::new();
        assert!(!gate.is_graduated("Explore"));

        gate.force_graduate("Explore");
        assert!(gate.is_graduated("Explore"));

        // Degrade passes through after force graduation
        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };
        let result = gate.filter_verdict("Explore", verdict);
        assert!(matches!(result, UpfVerdict::Degrade { .. }));
    }

    #[test]
    fn test_reset_category() {
        let config = ThymicConfig {
            observation_window: 2,
            enabled: true,
        };
        let mut gate = ThymicGate::with_config(config);

        // Graduate
        gate.filter_verdict("Explore", UpfVerdict::Continue);
        gate.filter_verdict("Explore", UpfVerdict::Continue);
        assert!(gate.is_graduated("Explore"));

        // Reset
        gate.reset_category("Explore");
        assert!(!gate.is_graduated("Explore"));

        // Back in observation mode: Degrade → Stall
        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };
        let result = gate.filter_verdict("Explore", verdict);
        assert!(matches!(result, UpfVerdict::Stall { .. }));
    }

    #[test]
    fn test_suppressed_degrade_rate() {
        let config = ThymicConfig {
            observation_window: 10,
            enabled: true,
        };
        let mut gate = ThymicGate::with_config(config);

        // 2 degrades out of 5 runs = 40%
        gate.filter_verdict(
            "Explore",
            UpfVerdict::Degrade {
                anomalies: make_anomalies(),
            },
        );
        gate.filter_verdict("Explore", UpfVerdict::Continue);
        gate.filter_verdict(
            "Explore",
            UpfVerdict::Degrade {
                anomalies: make_anomalies(),
            },
        );
        gate.filter_verdict("Explore", UpfVerdict::Continue);
        gate.filter_verdict("Explore", UpfVerdict::Continue);

        let rate = gate.suppressed_degrade_rate("Explore");
        assert!((rate - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_independent_categories() {
        let config = ThymicConfig {
            observation_window: 3,
            enabled: true,
        };
        let mut gate = ThymicGate::with_config(config);

        // Graduate Explore
        for _ in 0..3 {
            gate.filter_verdict("Explore", UpfVerdict::Continue);
        }
        assert!(gate.is_graduated("Explore"));

        // Mutate still in observation
        assert!(!gate.is_graduated("Mutate"));
        let verdict = UpfVerdict::Degrade {
            anomalies: make_anomalies(),
        };
        let result = gate.filter_verdict("Mutate", verdict);
        assert!(matches!(result, UpfVerdict::Stall { .. }));
    }

    #[test]
    fn test_unknown_category_rate() {
        let gate = ThymicGate::new();
        assert!((gate.suppressed_degrade_rate("Unknown") - 0.0).abs() < f32::EPSILON);
    }
}
