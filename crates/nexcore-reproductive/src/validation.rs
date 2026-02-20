//! # Reproduction System Validation
//!
//! Logic for tracking and validating reproduction metrics:
//! - Artifact Save Fidelity
//! - Subagent Spawn Rate
//! - Session Rebirth Success
//! - Replication Success Rate

use serde::{Deserialize, Serialize};

use crate::genetics::GenomeRequirement;
use crate::mitosis::MitoticRepair;
use crate::phenotypes::TissuePhenotype;

/// Metrics for reproductive system performance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReproductionMetrics {
    /// Number of successful artifact saves.
    pub artifact_saves_success: usize,
    /// Total number of artifact save attempts.
    pub artifact_saves_total: usize,
    /// Number of successful subagent spawns.
    pub spawns_success: usize,
    /// Total number of subagent spawn attempts.
    pub spawns_total: usize,
    /// Number of successful session rebirths (pattern persistence).
    pub rebirths_success: usize,
    /// Total number of session rebirth attempts.
    pub rebirths_total: usize,
    /// Count of integrity verification failures.
    pub integrity_failures: usize,
    /// Count of specialized phenotypes correctly differentiated.
    pub phenotypes_differentiated: usize,
    /// Count of "Lethal Mutations" blocked by GeneticGuard.
    pub lethal_mutations_blocked: usize,
}

impl ReproductionMetrics {
    /// Creates a new empty metrics container.
    pub fn new() -> Self {
        Self {
            artifact_saves_success: 0,
            artifact_saves_total: 0,
            spawns_success: 0,
            spawns_total: 0,
            rebirths_success: 0,
            rebirths_total: 0,
            integrity_failures: 0,
            phenotypes_differentiated: 0,
            lethal_mutations_blocked: 0,
        }
    }

    /// Calculates the artifact save fidelity (0.0 to 1.0).
    pub fn artifact_save_fidelity(&self) -> f64 {
        if self.artifact_saves_total == 0 {
            return 1.0;
        }
        let base_fidelity = self.artifact_saves_success as f64 / self.artifact_saves_total as f64;
        // Adjust for integrity failures
        let integrity_penalty =
            self.integrity_failures as f64 / self.artifact_saves_total.max(1) as f64;
        (base_fidelity - integrity_penalty).max(0.0)
    }

    /// Calculates the subagent spawn rate (0.0 to 1.0).
    pub fn spawn_rate(&self) -> f64 {
        if self.spawns_total == 0 {
            return 1.0;
        }
        self.spawns_success as f64 / self.spawns_total as f64
    }

    /// Calculates the session rebirth success rate (0.0 to 1.0).
    pub fn rebirth_success_rate(&self) -> f64 {
        if self.rebirths_total == 0 {
            return 1.0;
        }
        self.rebirths_success as f64 / self.rebirths_total as f64
    }

    /// Calculates the overall replication success rate metric.
    /// Formula: (successful spawns / total spawn attempts) * 100
    pub fn replication_success_rate(&self) -> f64 {
        if self.spawns_total == 0 {
            return 0.0;
        }
        (self.spawns_success as f64 / self.spawns_total as f64) * 100.0
    }

    /// Validates if metrics meet the targets.
    pub fn meets_targets(&self) -> bool {
        self.artifact_save_fidelity() >= 0.999
            && self.spawn_rate() >= 1.0
            && self.rebirth_success_rate() >= 0.995
    }
}

/// A validator that exercises the reproductive system components.
pub struct ReproductiveValidator {
    pub metrics: ReproductionMetrics,
}

impl ReproductiveValidator {
    pub fn new() -> Self {
        Self {
            metrics: ReproductionMetrics::new(),
        }
    }

    /// Records an artifact save attempt.
    pub fn record_artifact_save(&mut self, success: bool) {
        self.metrics.artifact_saves_total += 1;
        if success {
            self.metrics.artifact_saves_success += 1;
        }
    }

    /// Records a subagent spawn attempt.
    pub fn record_spawn(&mut self, success: bool) {
        self.metrics.spawns_total += 1;
        if success {
            self.metrics.spawns_success += 1;
        }
    }

    /// Records a session rebirth attempt.
    pub fn record_rebirth(&mut self, success: bool) {
        self.metrics.rebirths_total += 1;
        if success {
            self.metrics.rebirths_success += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_calculation() {
        let mut metrics = ReproductionMetrics::new();
        metrics.artifact_saves_total = 1000;
        metrics.artifact_saves_success = 999;

        metrics.spawns_total = 10;
        metrics.spawns_success = 10;

        metrics.rebirths_total = 1000;
        metrics.rebirths_success = 995;

        assert!(metrics.artifact_save_fidelity() >= 0.999);
        assert_eq!(metrics.spawn_rate(), 1.0);
        assert!(metrics.rebirth_success_rate() >= 0.995);
        assert_eq!(metrics.replication_success_rate(), 100.0);
        assert!(metrics.meets_targets());
    }

    #[test]
    fn test_replication_success_rate_formula() {
        let mut metrics = ReproductionMetrics::new();
        metrics.spawns_total = 100;
        metrics.spawns_success = 85;
        // (85/100) * 100 = 85.0
        assert_eq!(metrics.replication_success_rate(), 85.0);
    }
}
