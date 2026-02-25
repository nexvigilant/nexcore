//! # Signal Validate
//!
//! Validates detection results for quality and consistency.
//! Checks minimum case counts, metric consistency, and confidence bounds.
//!
//! ## T1 Primitives: Mapping (μ) + Comparison (κ)
//! - **Mapping (μ)**: Transformation of `DetectionResult` into a structured `ValidationReport`.
//! - **Comparison (κ)**: Atomic evaluation of metrics against configured thresholds.

use crate::core::{
    DetectionResult, Result, ThresholdConfig, Validate, ValidationCheck, ValidationReport,
};

/// Standard validator using configurable thresholds.
pub struct StandardValidator {
    config: ThresholdConfig,
}

impl StandardValidator {
    /// Create with Evans defaults.
    pub fn new() -> Self {
        Self {
            config: ThresholdConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: ThresholdConfig) -> Self {
        Self { config }
    }

    fn check_case_count(&self, result: &DetectionResult) -> ValidationCheck {
        let passed = result.table.a >= self.config.case_count_min;
        ValidationCheck {
            name: "case_count".into(),
            passed,
            message: format!(
                "cases={}, min={}",
                result.table.a, self.config.case_count_min
            ),
        }
    }

    fn check_prr(&self, result: &DetectionResult) -> ValidationCheck {
        let passed = result.prr.map_or(false, |p| p.0 >= self.config.prr_min);
        ValidationCheck {
            name: "prr_threshold".into(),
            passed,
            message: format!(
                "prr={}, min={}",
                result.prr.map_or(0.0, |p| p.0),
                self.config.prr_min
            ),
        }
    }

    fn check_chi_square(&self, result: &DetectionResult) -> ValidationCheck {
        let passed = result.chi_square.0 >= self.config.chi_square_min;
        ValidationCheck {
            name: "chi_square".into(),
            passed,
            message: format!(
                "chi_sq={:.3}, min={:.3}",
                result.chi_square.0, self.config.chi_square_min
            ),
        }
    }

    fn check_table_consistency(&self, result: &DetectionResult) -> ValidationCheck {
        let total = result.table.total();
        let passed = total > 0;
        ValidationCheck {
            name: "table_consistency".into(),
            passed,
            message: format!("total={total}"),
        }
    }
}

impl Default for StandardValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validate for StandardValidator {
    fn validate(&self, result: &DetectionResult) -> Result<ValidationReport> {
        let checks = vec![
            self.check_case_count(result),
            self.check_prr(result),
            self.check_chi_square(result),
            self.check_table_consistency(result),
        ];
        let passed = checks.iter().all(|c| c.passed);
        Ok(ValidationReport {
            pair: result.pair.clone(),
            passed,
            checks,
        })
    }
}

/// Batch validate multiple results.
pub fn validate_batch(
    validator: &dyn Validate,
    results: &[DetectionResult],
) -> Result<Vec<ValidationReport>> {
    results.iter().map(|r| validator.validate(r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use nexcore_chrono::DateTime;

    fn strong_result() -> DetectionResult {
        DetectionResult {
            pair: DrugEventPair::new("aspirin", "bleeding"),
            table: ContingencyTable {
                a: 15,
                b: 100,
                c: 20,
                d: 10_000,
            },
            prr: Some(Prr(3.5)),
            ror: Some(Ror(7.5)),
            ic: Some(Ic(1.8)),
            ebgm: Some(Ebgm(2.5)),
            chi_square: ChiSquare(12.0),
            strength: SignalStrength::Strong,
            detected_at: DateTime::now(),
        }
    }

    fn weak_result() -> DetectionResult {
        DetectionResult {
            pair: DrugEventPair::new("drug_b", "event_b"),
            table: ContingencyTable {
                a: 1,
                b: 500,
                c: 200,
                d: 50_000,
            },
            prr: Some(Prr(0.5)),
            ror: Some(Ror(0.5)),
            ic: Some(Ic(-1.0)),
            ebgm: Some(Ebgm(0.3)),
            chi_square: ChiSquare(0.1),
            strength: SignalStrength::None,
            detected_at: DateTime::now(),
        }
    }

    #[test]
    fn strong_passes_validation() {
        let v = StandardValidator::new();
        let report = v.validate(&strong_result()).unwrap();
        assert!(report.passed);
        assert!(report.checks.iter().all(|c| c.passed));
    }

    #[test]
    fn weak_fails_validation() {
        let v = StandardValidator::new();
        let report = v.validate(&weak_result()).unwrap();
        assert!(!report.passed);
    }

    #[test]
    fn batch_validates() {
        let v = StandardValidator::new();
        let results = vec![strong_result(), weak_result()];
        let reports = validate_batch(&v, &results).unwrap();
        assert_eq!(reports.len(), 2);
        assert!(reports[0].passed);
        assert!(!reports[1].passed);
    }
}
