//! Product and safety profile types.
//!
//! Models a marketed pharmaceutical product, its associated safety profile,
//! and signal-level summaries for competitive and regulatory analysis.

use serde::{Deserialize, Serialize};

use crate::TherapeuticArea;

/// A marketed pharmaceutical product.
///
/// Carries identity (generic/brand names, RxCUI), classification
/// (therapeutic area, approval year), and an attached safety profile.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::{Product, SafetyProfile, TherapeuticArea};
///
/// let product = Product {
///     generic_name: "semaglutide".to_string(),
///     brand_names: vec!["Ozempic".to_string(), "Wegovy".to_string()],
///     rxcui: Some("2200644".to_string()),
///     therapeutic_area: TherapeuticArea::Metabolic,
///     approval_year: Some(2017),
///     safety_profile: SafetyProfile::default(),
/// };
/// assert_eq!(product.generic_name, "semaglutide");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// INN / generic drug name
    pub generic_name: String,
    /// All brand names under which the product is marketed
    pub brand_names: Vec<String>,
    /// RxNorm concept unique identifier, if known
    pub rxcui: Option<String>,
    /// Primary therapeutic classification
    pub therapeutic_area: TherapeuticArea,
    /// Year of first regulatory approval, if known
    pub approval_year: Option<u16>,
    /// Aggregated safety profile
    pub safety_profile: SafetyProfile,
}

/// Aggregated safety profile for a product.
///
/// Summarises the most clinically significant safety attributes:
/// boxed warning presence, REMS requirement, detected signals,
/// and label-level warnings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetyProfile {
    /// Product carries an FDA boxed warning
    pub boxed_warning: bool,
    /// Product requires a Risk Evaluation and Mitigation Strategy
    pub rems: bool,
    /// Detected pharmacovigilance signals
    pub signals: Vec<SignalSummary>,
    /// Label warnings (Section 5 of US prescribing information)
    pub label_warnings: Vec<String>,
}

impl SafetyProfile {
    /// Returns `true` if any signal meets the `Strong` verdict threshold.
    pub fn has_strong_signal(&self) -> bool {
        self.signals
            .iter()
            .any(|s| matches!(s.verdict, SignalVerdict::Strong))
    }

    /// Returns `true` if the profile has any elevated safety concern
    /// (boxed warning, REMS, or a strong/moderate signal).
    pub fn is_elevated(&self) -> bool {
        self.boxed_warning
            || self.rems
            || self
                .signals
                .iter()
                .any(|s| matches!(s.verdict, SignalVerdict::Strong | SignalVerdict::Moderate))
    }
}

/// Disproportionality signal summary for a single adverse event.
///
/// Carries the computed PRR and ROR statistics, case count, label
/// status, and an overall verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSummary {
    /// MedDRA preferred term or verbatim adverse event description
    pub event: String,
    /// Proportional Reporting Ratio
    pub prr: f64,
    /// Reporting Odds Ratio
    pub ror: f64,
    /// Number of cases in the reporting database
    pub cases: u64,
    /// Whether the event appears in the current product label
    pub on_label: bool,
    /// Overall signal strength verdict
    pub verdict: SignalVerdict,
}

impl SignalSummary {
    /// Returns `true` if this signal is unlabelled and meets at least
    /// the `Moderate` verdict threshold — the standard triage criterion.
    pub fn requires_triage(&self) -> bool {
        !self.on_label
            && matches!(
                self.verdict,
                SignalVerdict::Strong | SignalVerdict::Moderate
            )
    }
}

/// Overall pharmacovigilance signal strength verdict.
///
/// Determined by combining disproportionality statistics (PRR, ROR),
/// case counts, and clinical context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalVerdict {
    /// Consistent, strong disproportionality across multiple metrics
    Strong,
    /// Moderate signal, warrants further evaluation
    Moderate,
    /// Weak signal, monitor but no immediate action required
    Weak,
    /// Below detection threshold, likely background noise
    Noise,
}

impl std::fmt::Display for SignalVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Strong => "Strong",
            Self::Moderate => "Moderate",
            Self::Weak => "Weak",
            Self::Noise => "Noise",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(verdict: SignalVerdict, on_label: bool) -> SignalSummary {
        SignalSummary {
            event: "nausea".to_string(),
            prr: 2.5,
            ror: 3.1,
            cases: 42,
            on_label,
            verdict,
        }
    }

    #[test]
    fn safety_profile_default_is_clean() {
        let profile = SafetyProfile::default();
        assert!(!profile.boxed_warning);
        assert!(!profile.rems);
        assert!(profile.signals.is_empty());
        assert!(profile.label_warnings.is_empty());
        assert!(!profile.has_strong_signal());
        assert!(!profile.is_elevated());
    }

    #[test]
    fn has_strong_signal_detects_strong_only() {
        let mut profile = SafetyProfile::default();
        profile
            .signals
            .push(make_signal(SignalVerdict::Weak, false));
        assert!(!profile.has_strong_signal());
        profile
            .signals
            .push(make_signal(SignalVerdict::Strong, false));
        assert!(profile.has_strong_signal());
    }

    #[test]
    fn is_elevated_on_boxed_warning() {
        let profile = SafetyProfile {
            boxed_warning: true,
            ..Default::default()
        };
        assert!(profile.is_elevated());
    }

    #[test]
    fn is_elevated_on_rems() {
        let profile = SafetyProfile {
            rems: true,
            ..Default::default()
        };
        assert!(profile.is_elevated());
    }

    #[test]
    fn is_elevated_on_strong_signal() {
        let mut profile = SafetyProfile::default();
        profile
            .signals
            .push(make_signal(SignalVerdict::Strong, true));
        assert!(profile.is_elevated());
    }

    #[test]
    fn signal_requires_triage_unlabelled_moderate() {
        let s = make_signal(SignalVerdict::Moderate, false);
        assert!(s.requires_triage());
    }

    #[test]
    fn signal_does_not_require_triage_when_on_label() {
        let s = make_signal(SignalVerdict::Strong, true);
        assert!(!s.requires_triage());
    }

    #[test]
    fn signal_does_not_require_triage_when_weak() {
        let s = make_signal(SignalVerdict::Weak, false);
        assert!(!s.requires_triage());
    }

    #[test]
    fn signal_verdict_display() {
        assert_eq!(SignalVerdict::Strong.to_string(), "Strong");
        assert_eq!(SignalVerdict::Moderate.to_string(), "Moderate");
        assert_eq!(SignalVerdict::Weak.to_string(), "Weak");
        assert_eq!(SignalVerdict::Noise.to_string(), "Noise");
    }

    #[test]
    fn product_constructs() {
        let p = Product {
            generic_name: "atorvastatin".to_string(),
            brand_names: vec!["Lipitor".to_string()],
            rxcui: Some("83367".to_string()),
            therapeutic_area: TherapeuticArea::Cardiovascular,
            approval_year: Some(1996),
            safety_profile: SafetyProfile::default(),
        };
        assert_eq!(p.generic_name, "atorvastatin");
        assert_eq!(p.brand_names.len(), 1);
        assert_eq!(p.approval_year, Some(1996));
    }

    #[test]
    fn product_serializes_round_trip() {
        let p = Product {
            generic_name: "pembrolizumab".to_string(),
            brand_names: vec!["Keytruda".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2014),
            safety_profile: SafetyProfile {
                boxed_warning: false,
                rems: false,
                signals: vec![make_signal(SignalVerdict::Moderate, false)],
                label_warnings: vec!["Immune-mediated adverse reactions".to_string()],
            },
        };
        let json = serde_json::to_string(&p).expect("serialization cannot fail");
        let parsed: Product = serde_json::from_str(&json).expect("deserialization cannot fail");
        assert_eq!(parsed.generic_name, "pembrolizumab");
        assert_eq!(parsed.safety_profile.signals.len(), 1);
    }

    #[test]
    fn signal_verdict_serializes_round_trip() {
        for verdict in [
            SignalVerdict::Strong,
            SignalVerdict::Moderate,
            SignalVerdict::Weak,
            SignalVerdict::Noise,
        ] {
            let json =
                serde_json::to_string(&verdict).expect("serialization cannot fail on valid enum");
            let parsed: SignalVerdict =
                serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
            assert_eq!(verdict, parsed);
        }
    }
}
