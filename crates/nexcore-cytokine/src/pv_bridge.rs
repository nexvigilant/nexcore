//! # PV Signal Bridge — Pharmacovigilance ↔ Cytokine Integration
//!
//! Maps pharmacovigilance signal detection events to typed cytokine signals,
//! enabling the biological signaling infrastructure to carry PV-domain information.
//!
//! ## Innovation Scan 001 — Goal 2 (Score: 8.20)
//!
//! ```text
//! PV Algorithm Result → PvSignalType → Cytokine(family, severity, payload)
//!                                          ↓
//!                          CascadeRule → escalation chain
//! ```
//!
//! ## ToV Alignment: V3 Conservation
//! No signal data lost in the PV→Cytokine translation.
//! Every PV metric is preserved in the cytokine payload.
//!
//! ## Tier: T2-C (→ + N + μ + ∂ + ρ)

use crate::{
    CascadeResponse, CascadeRule, Cytokine, CytokineFamily, ReceptorFilter, Scope, ThreatLevel,
};
use serde::{Deserialize, Serialize};

// ─── PV Signal Types ─────────────────────────────────────────────────────────

/// Classification of pharmacovigilance signal events.
///
/// Each variant maps to a specific point in the PV pipeline where
/// a cytokine should be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvSignalType {
    /// A disproportionality algorithm detected a signal above threshold.
    /// Trigger: PRR >= 2.0, ROR LCI > 1.0, IC025 > 0, EB05 >= 2.0
    SignalDetected,

    /// Signal strength exceeds critical threshold (e.g., PRR >= 5.0).
    /// Warrants immediate escalation.
    ThresholdExceeded,

    /// Causality assessment completed (Naranjo or WHO-UMC).
    CausalityAssessed,

    /// Schema drift detected in incoming PV data.
    DriftDetected,

    /// Batch signal screening completed.
    BatchCompleted,

    /// A previously detected signal was refuted on deeper analysis.
    SignalRefuted,
}

impl PvSignalType {
    /// Map signal type to the appropriate cytokine family.
    ///
    /// - `SignalDetected` → IL-1 (alarm)
    /// - `ThresholdExceeded` → IFN-γ (amplify response)
    /// - `CausalityAssessed` → TGF-β (regulate/modulate)
    /// - `DriftDetected` → IL-6 (acute phase response)
    /// - `BatchCompleted` → IL-10 (suppression — wind down)
    /// - `SignalRefuted` → IL-10 (suppression — stand down)
    #[must_use]
    pub const fn cytokine_family(&self) -> CytokineFamily {
        match self {
            Self::SignalDetected => CytokineFamily::Il1,
            Self::ThresholdExceeded => CytokineFamily::IfnGamma,
            Self::CausalityAssessed => CytokineFamily::TgfBeta,
            Self::DriftDetected => CytokineFamily::Il6,
            Self::BatchCompleted | Self::SignalRefuted => CytokineFamily::Il10,
        }
    }

    /// Default scope for this signal type.
    #[must_use]
    pub const fn default_scope(&self) -> Scope {
        match self {
            Self::ThresholdExceeded | Self::DriftDetected => Scope::Systemic,
            Self::SignalDetected | Self::CausalityAssessed => Scope::Endocrine,
            Self::BatchCompleted | Self::SignalRefuted => Scope::Paracrine,
        }
    }

    /// Human-readable label for display.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::SignalDetected => "pv_signal_detected",
            Self::ThresholdExceeded => "pv_threshold_exceeded",
            Self::CausalityAssessed => "pv_causality_assessed",
            Self::DriftDetected => "pv_drift_detected",
            Self::BatchCompleted => "pv_batch_completed",
            Self::SignalRefuted => "pv_signal_refuted",
        }
    }
}

// ─── PV Signal Metrics ───────────────────────────────────────────────────────

/// Metrics from a PV signal detection algorithm.
///
/// Carries enough information to reconstruct the full signal context
/// on the receiving end (ToV V3: Conservation — no data lost).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvSignalMetrics {
    /// Which algorithm produced the result (PRR, ROR, IC, EBGM, ChiSquare).
    pub algorithm: String,
    /// Computed metric value.
    pub value: f64,
    /// Threshold that was used for comparison.
    pub threshold: f64,
    /// Drug name (if applicable).
    pub drug: Option<String>,
    /// Adverse event term (if applicable).
    pub adverse_event: Option<String>,
    /// Number of cases in the signal.
    pub case_count: Option<u64>,
    /// Confidence interval lower bound (if applicable).
    pub ci_lower: Option<f64>,
}

impl PvSignalMetrics {
    /// Create metrics for a basic signal detection result.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value: f64, threshold: f64) -> Self {
        Self {
            algorithm: algorithm.into(),
            value,
            threshold,
            drug: None,
            adverse_event: None,
            case_count: None,
            ci_lower: None,
        }
    }

    /// Set the drug name.
    #[must_use]
    pub fn with_drug(mut self, drug: impl Into<String>) -> Self {
        self.drug = Some(drug.into());
        self
    }

    /// Set the adverse event term.
    #[must_use]
    pub fn with_adverse_event(mut self, ae: impl Into<String>) -> Self {
        self.adverse_event = Some(ae.into());
        self
    }

    /// Set the case count.
    #[must_use]
    pub fn with_case_count(mut self, count: u64) -> Self {
        self.case_count = Some(count);
        self
    }

    /// Set the confidence interval lower bound.
    #[must_use]
    pub fn with_ci_lower(mut self, ci: f64) -> Self {
        self.ci_lower = Some(ci);
        self
    }

    /// How far above threshold is the signal? Ratio of value/threshold.
    /// Returns 0.0 if threshold is zero.
    #[must_use]
    pub fn exceedance_ratio(&self) -> f64 {
        if self.threshold <= 0.0 {
            return 0.0;
        }
        self.value / self.threshold
    }

    /// Is the signal above its threshold?
    #[must_use]
    pub fn is_above_threshold(&self) -> bool {
        self.value >= self.threshold
    }
}

// ─── Severity Mapping ────────────────────────────────────────────────────────

/// Map PV signal strength to cytokine threat level.
///
/// Uses exceedance ratio (value / threshold) to determine severity:
/// - < 1.0: Trace (below threshold)
/// - 1.0 – 1.5: Low (marginal signal)
/// - 1.5 – 2.5: Medium (clear signal)
/// - 2.5 – 4.0: High (strong signal)
/// - >= 4.0: Critical (very strong signal)
#[must_use]
pub fn pv_severity(metrics: &PvSignalMetrics) -> ThreatLevel {
    let ratio = metrics.exceedance_ratio();
    if ratio < 1.0 {
        ThreatLevel::Trace
    } else if ratio < 1.5 {
        ThreatLevel::Low
    } else if ratio < 2.5 {
        ThreatLevel::Medium
    } else if ratio < 4.0 {
        ThreatLevel::High
    } else {
        ThreatLevel::Critical
    }
}

// ─── Cytokine Construction ──────────────────────────────────────────────────

/// Convert a PV signal event into a fully typed cytokine.
///
/// The cytokine payload preserves ALL metric data (ToV V3 Conservation).
/// The severity is derived from the exceedance ratio.
/// The family is determined by the signal type.
#[must_use]
pub fn pv_to_cytokine(signal_type: PvSignalType, metrics: &PvSignalMetrics) -> Cytokine {
    let severity = pv_severity(metrics);

    // Override: ThresholdExceeded always High or above
    let severity = if signal_type == PvSignalType::ThresholdExceeded && severity < ThreatLevel::High
    {
        ThreatLevel::High
    } else {
        severity
    };

    let payload = serde_json::json!({
        "signal_type": signal_type.label(),
        "algorithm": metrics.algorithm,
        "value": metrics.value,
        "threshold": metrics.threshold,
        "exceedance_ratio": metrics.exceedance_ratio(),
        "drug": metrics.drug,
        "adverse_event": metrics.adverse_event,
        "case_count": metrics.case_count,
        "ci_lower": metrics.ci_lower,
    });

    Cytokine::new(signal_type.cytokine_family(), signal_type.label())
        .with_severity(severity)
        .with_scope(signal_type.default_scope())
        .with_payload(payload)
        .with_source("pv-pipeline")
}

// ─── Cascade Patterns ────────────────────────────────────────────────────────

/// Pre-built cascade patterns for PV signal escalation.
pub mod pv_cascades {
    use super::{CascadeResponse, CascadeRule, CytokineFamily, ReceptorFilter, Scope, ThreatLevel};

    /// Signal detection cascade: IL-1 (alarm) → IFN-γ (amplify for review).
    ///
    /// When a PV signal is detected, amplify the response so reviewers
    /// are notified and additional algorithms can be triggered.
    pub fn signal_escalation() -> CascadeRule {
        CascadeRule::new(
            "pv_signal_escalation",
            ReceptorFilter::family(CytokineFamily::Il1).with_name("pv_signal_detected"),
        )
        .with_response(
            CascadeResponse::new(CytokineFamily::IfnGamma, "pv_escalate_review")
                .with_severity(ThreatLevel::High)
                .with_scope(Scope::Endocrine),
        )
    }

    /// Critical threshold cascade: IFN-γ (amplify) → TNF-α (halt pipeline).
    ///
    /// When a signal exceeds critical threshold, emit a termination signal
    /// to halt further data ingestion until reviewed.
    pub fn critical_halt() -> CascadeRule {
        CascadeRule::new(
            "pv_critical_halt",
            ReceptorFilter::family(CytokineFamily::IfnGamma)
                .with_name("pv_threshold_exceeded")
                .with_min_severity(ThreatLevel::Critical),
        )
        .with_response(
            CascadeResponse::new(CytokineFamily::TnfAlpha, "pv_halt_pipeline")
                .with_severity(ThreatLevel::Critical)
                .with_scope(Scope::Systemic),
        )
        .with_max_depth(1) // No recursive halts
    }

    /// Drift response cascade: IL-6 (acute) → IL-1 (alarm to guardian).
    ///
    /// Schema drift in PV data triggers an alarm to the Guardian system
    /// for investigation.
    pub fn drift_alarm() -> CascadeRule {
        CascadeRule::new(
            "pv_drift_alarm",
            ReceptorFilter::family(CytokineFamily::Il6).with_name("pv_drift_detected"),
        )
        .with_response(
            CascadeResponse::new(CytokineFamily::Il1, "pv_drift_guardian_alert")
                .with_severity(ThreatLevel::High)
                .with_scope(Scope::Systemic),
        )
        .with_max_depth(1)
    }

    /// Batch completion cascade: IL-10 (suppress) → TGF-β (regulate).
    ///
    /// After batch screening completes, modulate the system back to
    /// normal operating parameters.
    pub fn batch_cooldown() -> CascadeRule {
        CascadeRule::new(
            "pv_batch_cooldown",
            ReceptorFilter::family(CytokineFamily::Il10).with_name("pv_batch_completed"),
        )
        .with_response(
            CascadeResponse::new(CytokineFamily::TgfBeta, "pv_normalize_thresholds")
                .with_scope(Scope::Paracrine),
        )
    }

    /// Get all PV cascade rules as a vector.
    ///
    /// Register these with a `CytokineBus` to enable automatic
    /// PV signal escalation.
    #[must_use]
    pub fn all_pv_cascades() -> Vec<CascadeRule> {
        vec![
            signal_escalation(),
            critical_halt(),
            drift_alarm(),
            batch_cooldown(),
        ]
    }
}

// ─── Convenience Constructors ────────────────────────────────────────────────

/// Create a cytokine for a PRR signal detection.
#[must_use]
pub fn prr_signal(value: f64, drug: &str, ae: &str) -> Cytokine {
    let metrics = PvSignalMetrics::new("PRR", value, 2.0)
        .with_drug(drug)
        .with_adverse_event(ae);

    let signal_type = if value >= 10.0 {
        PvSignalType::ThresholdExceeded
    } else if value >= 2.0 {
        PvSignalType::SignalDetected
    } else {
        PvSignalType::SignalRefuted
    };

    pv_to_cytokine(signal_type, &metrics)
}

/// Create a cytokine for an ROR signal detection.
#[must_use]
pub fn ror_signal(value: f64, ci_lower: f64, drug: &str, ae: &str) -> Cytokine {
    let metrics = PvSignalMetrics::new("ROR", value, 1.0)
        .with_drug(drug)
        .with_adverse_event(ae)
        .with_ci_lower(ci_lower);

    let signal_type = if ci_lower > 1.0 {
        PvSignalType::SignalDetected
    } else {
        PvSignalType::SignalRefuted
    };

    pv_to_cytokine(signal_type, &metrics)
}

/// Create a cytokine for a drift detection event.
#[must_use]
pub fn drift_signal(drift_score: f64, threshold: f64) -> Cytokine {
    let metrics = PvSignalMetrics::new("DriftDetector", drift_score, threshold);
    pv_to_cytokine(PvSignalType::DriftDetected, &metrics)
}

/// Create a cytokine for batch completion.
#[must_use]
pub fn batch_complete(algorithms_run: u64, signals_found: u64) -> Cytokine {
    // Precision loss acceptable: signal counts are small enough for f64
    #[allow(
        clippy::cast_precision_loss,
        reason = "Count-to-f64 conversion for bounded runtime metrics"
    )]
    let metrics = PvSignalMetrics::new("BatchScreen", signals_found as f64, 0.0)
        .with_case_count(algorithms_run);
    pv_to_cytokine(PvSignalType::BatchCompleted, &metrics)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    // ── Severity mapping ──────────────────────────────────────────────────

    #[test]
    fn test_below_threshold_is_trace() {
        let m = PvSignalMetrics::new("PRR", 1.5, 2.0);
        assert_eq!(pv_severity(&m), ThreatLevel::Trace);
    }

    #[test]
    fn test_marginal_signal_is_low() {
        let m = PvSignalMetrics::new("PRR", 2.5, 2.0); // ratio 1.25
        assert_eq!(pv_severity(&m), ThreatLevel::Low);
    }

    #[test]
    fn test_clear_signal_is_medium() {
        let m = PvSignalMetrics::new("PRR", 4.0, 2.0); // ratio 2.0
        assert_eq!(pv_severity(&m), ThreatLevel::Medium);
    }

    #[test]
    fn test_strong_signal_is_high() {
        let m = PvSignalMetrics::new("PRR", 6.0, 2.0); // ratio 3.0
        assert_eq!(pv_severity(&m), ThreatLevel::High);
    }

    #[test]
    fn test_very_strong_signal_is_critical() {
        let m = PvSignalMetrics::new("PRR", 10.0, 2.0); // ratio 5.0
        assert_eq!(pv_severity(&m), ThreatLevel::Critical);
    }

    #[test]
    fn test_zero_threshold_returns_zero_ratio() {
        let m = PvSignalMetrics::new("PRR", 5.0, 0.0);
        assert!((m.exceedance_ratio() - 0.0).abs() < f64::EPSILON);
    }

    // ── Signal type mapping ───────────────────────────────────────────────

    #[test]
    fn test_signal_detected_maps_to_il1() {
        assert_eq!(
            PvSignalType::SignalDetected.cytokine_family(),
            CytokineFamily::Il1
        );
    }

    #[test]
    fn test_threshold_exceeded_maps_to_ifn_gamma() {
        assert_eq!(
            PvSignalType::ThresholdExceeded.cytokine_family(),
            CytokineFamily::IfnGamma
        );
    }

    #[test]
    fn test_drift_maps_to_il6() {
        assert_eq!(
            PvSignalType::DriftDetected.cytokine_family(),
            CytokineFamily::Il6
        );
    }

    #[test]
    fn test_batch_complete_maps_to_il10() {
        assert_eq!(
            PvSignalType::BatchCompleted.cytokine_family(),
            CytokineFamily::Il10
        );
    }

    // ── Cytokine construction ─────────────────────────────────────────────

    #[test]
    fn test_pv_to_cytokine_preserves_all_metrics() {
        let m = PvSignalMetrics::new("PRR", 5.0, 2.0)
            .with_drug("aspirin")
            .with_adverse_event("headache")
            .with_case_count(42)
            .with_ci_lower(1.5);

        let c = pv_to_cytokine(PvSignalType::SignalDetected, &m);

        assert_eq!(c.family, CytokineFamily::Il1);
        assert_eq!(c.name, "pv_signal_detected");

        // Verify payload conservation (ToV V3)
        let p = &c.payload;
        assert_eq!(p["algorithm"], "PRR");
        assert_eq!(p["value"], 5.0);
        assert_eq!(p["threshold"], 2.0);
        assert_eq!(p["drug"], "aspirin");
        assert_eq!(p["adverse_event"], "headache");
        assert_eq!(p["case_count"], 42);
        assert_eq!(p["ci_lower"], 1.5);
    }

    #[test]
    fn test_threshold_exceeded_forces_high_minimum() {
        let m = PvSignalMetrics::new("PRR", 2.1, 2.0); // ratio 1.05 → normally Low
        let c = pv_to_cytokine(PvSignalType::ThresholdExceeded, &m);
        assert!(c.severity >= ThreatLevel::High);
    }

    #[test]
    fn test_systemic_scope_for_threshold_exceeded() {
        let c = pv_to_cytokine(
            PvSignalType::ThresholdExceeded,
            &PvSignalMetrics::new("PRR", 10.0, 2.0),
        );
        assert_eq!(c.scope, Scope::Systemic);
    }

    // ── Convenience constructors ──────────────────────────────────────────

    #[test]
    fn test_prr_signal_above_threshold() {
        let c = prr_signal(3.5, "ibuprofen", "gi_bleed");
        assert_eq!(c.family, CytokineFamily::Il1); // SignalDetected
        assert_eq!(c.payload["drug"], "ibuprofen");
    }

    #[test]
    fn test_prr_signal_critical() {
        let c = prr_signal(12.0, "drug_x", "hepatotoxicity");
        assert_eq!(c.family, CytokineFamily::IfnGamma); // ThresholdExceeded
        assert!(c.severity >= ThreatLevel::High);
    }

    #[test]
    fn test_prr_signal_below_threshold() {
        let c = prr_signal(1.5, "placebo", "nothing");
        assert_eq!(c.family, CytokineFamily::Il10); // SignalRefuted
    }

    #[test]
    fn test_ror_signal_with_ci() {
        let c = ror_signal(2.5, 1.3, "warfarin", "bleeding");
        assert_eq!(c.family, CytokineFamily::Il1); // ci_lower > 1 → detected
        assert_eq!(c.payload["ci_lower"], 1.3);
    }

    #[test]
    fn test_ror_signal_refuted() {
        let c = ror_signal(1.8, 0.7, "sugar_pill", "nothing");
        assert_eq!(c.family, CytokineFamily::Il10); // ci_lower <= 1 → refuted
    }

    #[test]
    fn test_drift_signal() {
        let c = drift_signal(0.35, 0.25);
        assert_eq!(c.family, CytokineFamily::Il6);
        assert_eq!(c.scope, Scope::Systemic);
    }

    #[test]
    fn test_batch_complete_signal() {
        let c = batch_complete(5, 3);
        assert_eq!(c.family, CytokineFamily::Il10);
        assert_eq!(c.scope, Scope::Paracrine);
    }

    // ── Cascade patterns ──────────────────────────────────────────────────

    #[test]
    fn test_signal_escalation_cascade() {
        let rule = pv_cascades::signal_escalation();
        assert!(rule.active);
        assert_eq!(rule.responses.len(), 1);
        assert_eq!(rule.responses[0].family, CytokineFamily::IfnGamma);
    }

    #[test]
    fn test_critical_halt_cascade() {
        let rule = pv_cascades::critical_halt();
        assert_eq!(rule.max_depth, 1);
        assert_eq!(rule.responses[0].family, CytokineFamily::TnfAlpha);
    }

    #[test]
    fn test_drift_alarm_cascade() {
        let rule = pv_cascades::drift_alarm();
        assert_eq!(rule.responses[0].family, CytokineFamily::Il1);
    }

    #[test]
    fn test_all_pv_cascades_count() {
        let cascades = pv_cascades::all_pv_cascades();
        assert_eq!(cascades.len(), 4);
    }

    #[test]
    fn test_signal_escalation_matches_pv_alarm() {
        let rule = pv_cascades::signal_escalation();
        let signal = prr_signal(3.5, "drug", "ae");
        assert!(rule.matches(&signal, 0));
    }

    #[test]
    fn test_signal_escalation_ignores_non_pv() {
        let rule = pv_cascades::signal_escalation();
        let signal = Cytokine::alarm("generic_alarm");
        // IL-1 family matches but name doesn't
        assert!(!rule.matches(&signal, 0));
    }

    // ── Metrics API ───────────────────────────────────────────────────────

    #[test]
    fn test_metrics_is_above_threshold() {
        let above = PvSignalMetrics::new("PRR", 3.0, 2.0);
        assert!(above.is_above_threshold());

        let below = PvSignalMetrics::new("PRR", 1.5, 2.0);
        assert!(!below.is_above_threshold());

        let equal = PvSignalMetrics::new("PRR", 2.0, 2.0);
        assert!(equal.is_above_threshold());
    }

    #[test]
    fn test_exceedance_ratio() {
        let m = PvSignalMetrics::new("PRR", 6.0, 2.0);
        assert!((m.exceedance_ratio() - 3.0).abs() < f64::EPSILON);
    }
}
