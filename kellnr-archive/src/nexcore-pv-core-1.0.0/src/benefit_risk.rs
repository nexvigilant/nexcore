//! Quantitative Benefit-Risk Index (QBRI)
//!
//! Bridges signal detection (risk) with effectiveness endpoints (benefit)
//! to produce a unified decision metric.
//!
//! ## Equation
//!
//! ```text
//! QBRI = (B × Pb × Ub) / (R × Pr × Sr × Tr)
//! ```

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// BENEFIT COMPONENT
// ═══════════════════════════════════════════════════════════════════════════

/// Benefit assessment from clinical trial effectiveness data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitAssessment {
    /// Effect size magnitude (HR inverse or OR)
    pub magnitude: f64,
    /// Probability of benefit (1 - p_value)
    pub probability: f64,
    /// Unmet medical need multiplier [1, 10]
    pub unmet_need: f64,
}

impl BenefitAssessment {
    /// Create from trial results.
    #[must_use]
    pub fn from_trial(effect_size: f64, p_value: f64, unmet_need: f64) -> Self {
        Self {
            magnitude: effect_size.abs(),
            probability: (1.0 - p_value).clamp(0.0, 1.0),
            unmet_need: unmet_need.clamp(1.0, 10.0),
        }
    }

    /// Compute benefit score: B × Pb × Ub
    #[must_use]
    pub fn score(&self) -> f64 {
        self.magnitude * self.probability * self.unmet_need
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RISK COMPONENT
// ═══════════════════════════════════════════════════════════════════════════

/// Risk assessment from signal detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Signal strength (PRR, ROR, or EBGM)
    pub magnitude: f64,
    /// Probability of causal relationship
    pub probability: f64,
    /// Severity (Hartwig-Siegel 1-7)
    pub severity: f64,
    /// Treatability (reversible=0.5, irreversible=1.0)
    pub treatability: f64,
}

impl RiskAssessment {
    /// Create from signal detection results.
    #[must_use]
    pub fn from_signal(signal: f64, prob: f64, severity: u8, reversible: bool) -> Self {
        Self {
            magnitude: signal.max(1.0),
            probability: prob.clamp(0.0, 1.0),
            severity: (severity as f64).clamp(1.0, 7.0),
            treatability: if reversible { 0.5 } else { 1.0 },
        }
    }

    /// Compute risk score: R × Pr × Sr × Tr
    #[must_use]
    pub fn score(&self) -> f64 {
        self.magnitude * self.probability * self.severity * self.treatability
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// REGULATORY DECISION
// ═══════════════════════════════════════════════════════════════════════════

/// Regulatory decision categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulatoryDecision {
    Approve,
    ApproveWithRems,
    RequestMoreData,
    Reject,
}

impl RegulatoryDecision {
    #[allow(dead_code)]
    fn to_index(self) -> usize {
        match self {
            Self::Approve => 0,
            Self::ApproveWithRems => 1,
            Self::RequestMoreData => 2,
            Self::Reject => 3,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// THRESHOLDS (CONSTANTS TO DERIVE)
// ═══════════════════════════════════════════════════════════════════════════

/// Decision thresholds for QBRI interpretation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QbriThresholds {
    /// QBRI > τ_approve → Approve
    pub tau_approve: f64,
    /// QBRI ∈ [τ_monitor, τ_approve] → Approve with REMS
    pub tau_monitor: f64,
    /// QBRI ∈ [τ_uncertain, τ_monitor] → Request data
    pub tau_uncertain: f64,
}

impl Default for QbriThresholds {
    fn default() -> Self {
        Self {
            tau_approve: 2.0,
            tau_monitor: 1.0,
            tau_uncertain: 0.5,
        }
    }
}

impl QbriThresholds {
    #[must_use]
    pub fn new(approve: f64, monitor: f64, uncertain: f64) -> Self {
        Self {
            tau_approve: approve,
            tau_monitor: monitor,
            tau_uncertain: uncertain,
        }
    }

    #[must_use]
    pub fn decide(&self, qbri: f64) -> RegulatoryDecision {
        if qbri >= self.tau_approve {
            RegulatoryDecision::Approve
        } else if qbri >= self.tau_monitor {
            RegulatoryDecision::ApproveWithRems
        } else if qbri >= self.tau_uncertain {
            RegulatoryDecision::RequestMoreData
        } else {
            RegulatoryDecision::Reject
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// QBRI RESULT
// ═══════════════════════════════════════════════════════════════════════════

/// QBRI computation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QbriResult {
    pub index: f64,
    pub benefit_score: f64,
    pub risk_score: f64,
    pub decision: RegulatoryDecision,
    pub confidence: f64,
}

/// Compute QBRI = (B × Pb × Ub) / (R × Pr × Sr × Tr)
#[must_use]
pub fn compute_qbri(
    benefit: &BenefitAssessment,
    risk: &RiskAssessment,
    t: &QbriThresholds,
) -> QbriResult {
    let b = benefit.score();
    let r = risk.score();
    let index = if r > 0.0 { (b / r).min(100.0) } else { 100.0 };
    let confidence = (benefit.probability * risk.probability).sqrt();

    QbriResult {
        index,
        benefit_score: b,
        risk_score: r,
        decision: t.decide(index),
        confidence,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HISTORICAL DATA FOR DERIVATION
// ═══════════════════════════════════════════════════════════════════════════

/// Historical drug decision for threshold derivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDecision {
    pub drug_id: String,
    pub benefit: BenefitAssessment,
    pub risk: RiskAssessment,
    pub actual_decision: RegulatoryDecision,
}

/// Threshold optimization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdOptimizationResult {
    pub thresholds: QbriThresholds,
    pub accuracy: f64,
    pub n_drugs: usize,
}

use rayon::prelude::*;

/// Derive optimal thresholds via grid search.
#[must_use]
pub fn derive_thresholds(history: &[HistoricalDecision]) -> ThresholdOptimizationResult {
    if history.is_empty() {
        return ThresholdOptimizationResult {
            thresholds: QbriThresholds::default(),
            accuracy: 0.0,
            n_drugs: 0,
        };
    }

    // Pre-calculate raw QBRI values once
    let qbris: Vec<f64> = history.iter().map(compute_raw_qbri).collect();
    let (best, acc) = grid_search_parallel(&qbris, history);

    ThresholdOptimizationResult {
        thresholds: best,
        accuracy: acc,
        n_drugs: history.len(),
    }
}

fn compute_raw_qbri(h: &HistoricalDecision) -> f64 {
    let b = h.benefit.score();
    let r = h.risk.score();
    if r > 0.0 { (b / r).min(100.0) } else { 100.0 }
}

fn grid_search_parallel(qbris: &[f64], history: &[HistoricalDecision]) -> (QbriThresholds, f64) {
    (1..=50)
        .into_par_iter()
        .map(|a| find_best_for_alpha(a, qbris, history))
        .max_by(|(_, acc1), (_, acc2)| acc1.partial_cmp(acc2).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((QbriThresholds::default(), 0.0))
}

fn find_best_for_alpha(
    a: i32,
    qbris: &[f64],
    history: &[HistoricalDecision],
) -> (QbriThresholds, f64) {
    let mut best = QbriThresholds::default();
    let mut best_acc = 0.0;
    let a_val = a as f64 / 10.0;

    for m in 1..=a {
        let m_val = m as f64 / 10.0;
        for u in 1..=m {
            let u_val = u as f64 / 10.0;
            let t = QbriThresholds::new(a_val, m_val, u_val);
            let acc = evaluate(qbris, history, &t);
            if acc > best_acc {
                best_acc = acc;
                best = t;
            }
        }
    }
    (best, best_acc)
}

fn evaluate(qbris: &[f64], history: &[HistoricalDecision], t: &QbriThresholds) -> f64 {
    let correct = qbris
        .iter()
        .zip(history)
        .filter(|(q, h)| t.decide(**q) == h.actual_decision)
        .count();
    correct as f64 / history.len() as f64
}

// ═══════════════════════════════════════════════════════════════════════════
// SYNTHETIC DATA
// ═══════════════════════════════════════════════════════════════════════════

fn drug(
    id: &str,
    b: (f64, f64, f64),
    r: (f64, f64, u8, bool),
    d: RegulatoryDecision,
) -> HistoricalDecision {
    HistoricalDecision {
        drug_id: id.into(),
        benefit: BenefitAssessment::from_trial(b.0, b.1, b.2),
        risk: RiskAssessment::from_signal(r.0, r.1, r.2, r.3),
        actual_decision: d,
    }
}

/// Generate synthetic historical decisions for testing.
#[must_use]
pub fn generate_synthetic_data() -> Vec<HistoricalDecision> {
    use RegulatoryDecision::*;
    vec![
        drug("IMATINIB", (2.5, 0.001, 8.0), (1.5, 0.3, 2, true), Approve),
        drug(
            "ROSIGLITAZONE",
            (1.3, 0.01, 5.0),
            (2.5, 0.6, 4, false),
            ApproveWithRems,
        ),
        drug("ROFECOXIB", (1.1, 0.04, 3.0), (3.5, 0.8, 6, false), Reject),
        drug(
            "ADUCANUMAB",
            (1.2, 0.05, 9.0),
            (1.8, 0.4, 3, true),
            ApproveWithRems,
        ),
        drug(
            "PEMBROLIZUMAB",
            (2.0, 0.0001, 7.0),
            (2.0, 0.5, 3, true),
            Approve,
        ),
        drug(
            "TROGLITAZONE",
            (1.4, 0.02, 5.0),
            (4.0, 0.9, 7, false),
            Reject,
        ),
        drug(
            "ETEPLIRSEN",
            (1.1, 0.08, 10.0),
            (1.2, 0.2, 2, true),
            ApproveWithRems,
        ),
        drug(
            "LORCASERIN",
            (1.05, 0.03, 4.0),
            (2.8, 0.7, 5, false),
            Reject,
        ),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benefit_score() {
        let b = BenefitAssessment::from_trial(2.0, 0.01, 5.0);
        assert!((b.score() - 9.9).abs() < 0.01);
    }

    #[test]
    fn test_risk_score() {
        let r = RiskAssessment::from_signal(3.0, 0.5, 4, true);
        assert!((r.score() - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_qbri_high_benefit() {
        let b = BenefitAssessment::from_trial(2.5, 0.001, 8.0);
        let r = RiskAssessment::from_signal(1.5, 0.3, 2, true);
        let res = compute_qbri(&b, &r, &QbriThresholds::default());
        assert!(res.index > 2.0);
        assert_eq!(res.decision, RegulatoryDecision::Approve);
    }

    #[test]
    fn test_qbri_high_risk() {
        let b = BenefitAssessment::from_trial(1.1, 0.04, 3.0);
        let r = RiskAssessment::from_signal(3.5, 0.8, 6, false);
        let res = compute_qbri(&b, &r, &QbriThresholds::default());
        assert!(res.index < 0.5);
        assert_eq!(res.decision, RegulatoryDecision::Reject);
    }

    #[test]
    fn test_threshold_derivation() {
        let data = generate_synthetic_data();
        let res = derive_thresholds(&data);
        assert!(res.accuracy >= 0.5);
        assert_eq!(res.n_drugs, 8);
    }

    #[test]
    fn test_zero_risk() {
        let b = BenefitAssessment::from_trial(2.0, 0.01, 5.0);
        let r = RiskAssessment {
            magnitude: 1.0,
            probability: 0.0,
            severity: 1.0,
            treatability: 0.5,
        };
        let res = compute_qbri(&b, &r, &QbriThresholds::default());
        assert!(res.index >= 100.0);
    }
}
