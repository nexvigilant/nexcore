//! # Signal Threshold
//!
//! Configurable threshold engine for signal detection.
//! Applies Evans, strict, or custom criteria to `DetectionResult`s.
//!
//! ## T1 Primitives: State (ς) + Boundary (∂) + Comparison (κ)
//! - **State (ς)**: Encapsulated `ThresholdConfig` governing the evaluation context.
//! - **Boundary (∂)**: Definition of pass/fail limits for signal significance.
//! - **Comparison (κ)**: Predicate logic applied to quantify signal strength against boundaries.

use crate::core::{DetectionResult, Threshold, ThresholdConfig};

/// Evans-criteria threshold evaluator.
#[non_exhaustive]
pub struct EvansThreshold {
    /// Active threshold configuration.
    pub config: ThresholdConfig,
}

impl EvansThreshold {
    /// Create with default Evans thresholds.
    pub fn new() -> Self {
        Self {
            config: ThresholdConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: ThresholdConfig) -> Self {
        Self { config }
    }
}

impl Default for EvansThreshold {
    fn default() -> Self {
        Self::new()
    }
}

impl Threshold for EvansThreshold {
    fn apply(&self, result: &DetectionResult) -> bool {
        let prr_pass = result.prr.map_or(false, |p| p.0 >= self.config.prr_min);
        let chi_pass = result.chi_square.0 >= self.config.chi_square_min;
        let count_pass = result.table.a >= self.config.case_count_min;
        prr_pass && chi_pass && count_pass
    }
}

/// Multi-criteria threshold that requires ALL sub-thresholds to pass.
pub struct CompositeThreshold {
    thresholds: Vec<Box<dyn Threshold>>,
}

impl CompositeThreshold {
    /// Create empty composite.
    pub fn new() -> Self {
        Self {
            thresholds: Vec::new(),
        }
    }

    /// Add a threshold criterion.
    pub fn add(mut self, t: Box<dyn Threshold>) -> Self {
        self.thresholds.push(t);
        self
    }
}

impl Default for CompositeThreshold {
    fn default() -> Self {
        Self::new()
    }
}

impl Threshold for CompositeThreshold {
    fn apply(&self, result: &DetectionResult) -> bool {
        self.thresholds.iter().all(|t| t.apply(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use nexcore_chrono::DateTime;

    fn make_result(prr_val: f64, chi_val: f64, case_count: u64) -> DetectionResult {
        DetectionResult {
            pair: DrugEventPair::new("drug", "event"),
            table: ContingencyTable {
                a: case_count,
                b: 100,
                c: 20,
                d: 10_000,
            },
            prr: Some(Prr(prr_val)),
            ror: Some(Ror(1.0)),
            ic: Some(Ic(1.0)),
            ebgm: Some(Ebgm(1.0)),
            chi_square: ChiSquare(chi_val),
            strength: SignalStrength::from_prr(prr_val),
            detected_at: DateTime::now(),
        }
    }

    #[test]
    fn evans_pass() {
        let t = EvansThreshold::new();
        let r = make_result(2.5, 5.0, 5);
        assert!(t.apply(&r));
    }

    #[test]
    fn evans_fail_low_prr() {
        let t = EvansThreshold::new();
        let r = make_result(1.0, 5.0, 5);
        assert!(!t.apply(&r));
    }

    #[test]
    fn evans_fail_low_chi() {
        let t = EvansThreshold::new();
        let r = make_result(3.0, 1.0, 5);
        assert!(!t.apply(&r));
    }
}
