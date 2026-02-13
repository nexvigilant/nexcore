// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # SchemaImmuneSystem
//!
//! **Tier**: T3 (exists + kappa + mu + pi + rho + nu)
//! **Bridge**: immunity (antipattern detection) + ribosome (schema drift)
//! **Confidence**: 0.85
//!
//! Self-healing schema validation that learns from recurring drift patterns
//! and generates "antibodies" to auto-block or auto-fix known violations.

use core::fmt;
use std::collections::BTreeMap;

/// Threat classification (mirrors immunity PAMP/DAMP).
///
/// ## Tier: T2-P (exists + kappa)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatType {
    /// Pathogen-Associated — external data threat.
    Pamp,
    /// Damage-Associated — internal corruption.
    Damp,
}

/// Response strategy for a detected drift.
///
/// ## Tier: T2-P (causality + kappa)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStrategy {
    /// Block the data entirely.
    Block,
    /// Allow but warn.
    Warn,
    /// Attempt auto-fix.
    AutoFix,
}

/// A learned antibody for a schema drift pattern.
///
/// ## Tier: T2-C (exists + kappa + mu + pi)
#[derive(Debug, Clone)]
pub struct DriftAntibody {
    /// Unique identifier.
    pub id: String,
    /// Pattern description.
    pub pattern: String,
    /// Threat classification.
    pub threat_type: ThreatType,
    /// Response when pattern is detected.
    pub response: ResponseStrategy,
    /// Number of times this antibody has been triggered.
    pub trigger_count: u64,
    /// When this antibody was created (timestamp).
    pub created_at: u64,
}

/// Result of schema immune validation.
#[derive(Debug, Clone)]
pub struct ImmuneResult {
    /// Whether the data passed validation.
    pub passed: bool,
    /// Antibodies that were triggered.
    pub triggered_antibodies: Vec<String>,
    /// Drift violations found.
    pub violations: Vec<String>,
    /// Whether auto-fix was applied.
    pub auto_fixed: bool,
}

/// Schema immune system — learns from recurring drift patterns.
///
/// ## Tier: T3 (exists + kappa + mu + pi + rho + nu)
/// Dominant: kappa (comparison — fundamentally about matching patterns)
///
/// Innovation: bridges immunity and ribosome crates.
/// The immune system LEARNS which schema violations recur,
/// generates antibodies, and applies automated responses.
#[derive(Debug, Clone)]
pub struct SchemaImmuneSystem {
    /// Learned antibodies by ID.
    antibodies: BTreeMap<String, DriftAntibody>,
    /// Violation history: pattern -> occurrence count.
    violation_history: BTreeMap<String, u64>,
    /// Recurrence threshold: auto-generate antibody after N occurrences.
    recurrence_threshold: u64,
    /// Default response for new antibodies.
    default_response: ResponseStrategy,
    /// Next antibody ID.
    next_id: u64,
}

impl SchemaImmuneSystem {
    /// Create a new schema immune system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            antibodies: BTreeMap::new(),
            violation_history: BTreeMap::new(),
            recurrence_threshold: 3,
            default_response: ResponseStrategy::Warn,
            next_id: 0,
        }
    }

    /// Set the recurrence threshold for auto-antibody generation.
    #[must_use]
    pub fn with_threshold(mut self, threshold: u64) -> Self {
        self.recurrence_threshold = threshold.max(1);
        self
    }

    /// Set the default response for auto-generated antibodies.
    #[must_use]
    pub fn with_default_response(mut self, response: ResponseStrategy) -> Self {
        self.default_response = response;
        self
    }

    /// Manually add an antibody.
    pub fn add_antibody(&mut self, antibody: DriftAntibody) {
        self.antibodies.insert(antibody.id.clone(), antibody);
    }

    /// Report a violation. If it recurs past threshold, auto-generates an antibody.
    pub fn report_violation(
        &mut self,
        pattern: impl Into<String>,
        threat_type: ThreatType,
        timestamp: u64,
    ) {
        let pattern = pattern.into();
        let count = self.violation_history.entry(pattern.clone()).or_insert(0);
        *count += 1;

        // Auto-generate antibody if threshold reached
        if *count == self.recurrence_threshold && !self.has_antibody_for(&pattern) {
            let id = format!("auto-{}", self.next_id);
            self.next_id += 1;

            self.antibodies.insert(
                id.clone(),
                DriftAntibody {
                    id,
                    pattern: pattern.clone(),
                    threat_type,
                    response: self.default_response,
                    trigger_count: 0,
                    created_at: timestamp,
                },
            );
        }
    }

    /// Validate data against known antibodies.
    #[must_use]
    pub fn validate(&self, violations: &[String]) -> ImmuneResult {
        let mut triggered = Vec::new();
        let mut blocked = false;
        let mut auto_fixed = false;

        for violation in violations {
            for (id, antibody) in &self.antibodies {
                if violation.contains(&antibody.pattern) {
                    triggered.push(id.clone());

                    match antibody.response {
                        ResponseStrategy::Block => blocked = true,
                        ResponseStrategy::AutoFix => auto_fixed = true,
                        ResponseStrategy::Warn => {}
                    }
                }
            }
        }

        ImmuneResult {
            passed: !blocked,
            triggered_antibodies: triggered,
            violations: violations.to_vec(),
            auto_fixed,
        }
    }

    /// Update trigger counts for matched antibodies.
    pub fn record_triggers(&mut self, antibody_ids: &[String]) {
        for id in antibody_ids {
            if let Some(ab) = self.antibodies.get_mut(id) {
                ab.trigger_count += 1;
            }
        }
    }

    /// Check if an antibody exists for a pattern.
    #[must_use]
    pub fn has_antibody_for(&self, pattern: &str) -> bool {
        self.antibodies.values().any(|ab| ab.pattern == pattern)
    }

    /// Total antibodies.
    #[must_use]
    pub fn antibody_count(&self) -> usize {
        self.antibodies.len()
    }

    /// Total unique violations seen.
    #[must_use]
    pub fn unique_violations(&self) -> usize {
        self.violation_history.len()
    }
}

impl Default for SchemaImmuneSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SchemaImmuneSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SchemaImmuneSystem({} antibodies, {} unique violations)",
            self.antibody_count(),
            self.unique_violations(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_antibody_generation() {
        let mut immune = SchemaImmuneSystem::new().with_threshold(3);

        // Report same violation 3 times
        immune.report_violation("missing_field:email", ThreatType::Pamp, 1);
        immune.report_violation("missing_field:email", ThreatType::Pamp, 2);
        assert_eq!(immune.antibody_count(), 0);

        immune.report_violation("missing_field:email", ThreatType::Pamp, 3);
        assert_eq!(immune.antibody_count(), 1); // auto-generated!
    }

    #[test]
    fn test_validate_with_antibody() {
        let mut immune = SchemaImmuneSystem::new();
        immune.add_antibody(DriftAntibody {
            id: "block-null-name".into(),
            pattern: "null_name".into(),
            threat_type: ThreatType::Pamp,
            response: ResponseStrategy::Block,
            trigger_count: 0,
            created_at: 0,
        });

        let result = immune.validate(&["null_name detected".into()]);
        assert!(!result.passed); // blocked
        assert_eq!(result.triggered_antibodies.len(), 1);
    }

    #[test]
    fn test_warn_does_not_block() {
        let mut immune = SchemaImmuneSystem::new();
        immune.add_antibody(DriftAntibody {
            id: "warn-extra".into(),
            pattern: "extra_field".into(),
            threat_type: ThreatType::Damp,
            response: ResponseStrategy::Warn,
            trigger_count: 0,
            created_at: 0,
        });

        let result = immune.validate(&["extra_field:foo".into()]);
        assert!(result.passed); // warned but not blocked
        assert_eq!(result.triggered_antibodies.len(), 1);
    }

    #[test]
    fn test_autofix_flag() {
        let mut immune = SchemaImmuneSystem::new();
        immune.add_antibody(DriftAntibody {
            id: "fix-range".into(),
            pattern: "range_exceeded".into(),
            threat_type: ThreatType::Damp,
            response: ResponseStrategy::AutoFix,
            trigger_count: 0,
            created_at: 0,
        });

        let result = immune.validate(&["range_exceeded:score>100".into()]);
        assert!(result.passed);
        assert!(result.auto_fixed);
    }

    #[test]
    fn test_no_duplicate_antibodies() {
        let mut immune = SchemaImmuneSystem::new().with_threshold(2);

        // Report 4 times — should only generate 1 antibody
        for i in 0..4 {
            immune.report_violation("dup_pattern", ThreatType::Pamp, i);
        }

        assert_eq!(immune.antibody_count(), 1);
    }
}
