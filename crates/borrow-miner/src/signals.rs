//! PV Signal Detection Types - FDA FAERS Integration
//!
//! ## Primitive Tiers (per Codex)
//! - T2-P: PRR, ROR, IC, EB05 (newtype wrappers)
//! - T2-C: SignalResult (composed metrics)
//! - T3: DrugEvent (domain-specific FDA entity)

use serde::{Deserialize, Serialize};
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// T2-P SIGNAL METRIC WRAPPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Proportional Reporting Ratio
/// Tier: T2-P (newtype over f64)
/// Threshold: >=2.0 indicates potential signal
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PRR(pub f64);

impl PRR {
    pub const THRESHOLD: f64 = 2.0;

    pub fn is_signal(&self) -> bool {
        self.0 >= Self::THRESHOLD
    }

    pub fn strength(&self) -> SignalStrength {
        if self.0 >= 5.0 {
            SignalStrength::Strong
        } else if self.0 >= Self::THRESHOLD {
            SignalStrength::Moderate
        } else if self.0 >= 1.5 {
            SignalStrength::Weak
        } else {
            SignalStrength::None
        }
    }
}

impl fmt::Display for PRR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PRR={:.2}", self.0)
    }
}

/// Reporting Odds Ratio
/// Tier: T2-P (newtype over f64)
/// Signal if lower 95% CI > 1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ROR(pub f64);

impl ROR {
    pub fn is_signal(&self, ci_lower: f64) -> bool {
        ci_lower > 1.0
    }
}

impl fmt::Display for ROR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ROR={:.2}", self.0)
    }
}

/// Information Component (Bayesian)
/// Tier: T2-P (newtype over f64)
/// Signal if IC025 (lower 95% CI) > 0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct IC(pub f64);

impl IC {
    pub fn is_signal(&self, ic025: f64) -> bool {
        ic025 > 0.0
    }
}

impl fmt::Display for IC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IC={:.2}", self.0)
    }
}

/// Empirical Bayes Geometric Mean (lower 5% bound)
/// Tier: T2-P (newtype over f64)
/// Threshold: >=2.0 indicates signal
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EB05(pub f64);

impl EB05 {
    pub const THRESHOLD: f64 = 2.0;

    pub fn is_signal(&self) -> bool {
        self.0 >= Self::THRESHOLD
    }
}

impl fmt::Display for EB05 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EB05={:.2}", self.0)
    }
}

/// Case count for signal detection
/// Tier: T2-P (newtype over u32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CaseCount(pub u32);

impl CaseCount {
    pub const MIN_FOR_SIGNAL: u32 = 3;

    pub fn sufficient(&self) -> bool {
        self.0 >= Self::MIN_FOR_SIGNAL
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-C COMPOSITE TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Signal strength classification
/// Tier: T2-P (enum over discrete states)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalStrength {
    None,
    Weak,
    Moderate,
    Strong,
}

impl SignalStrength {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::None => "⚪",
            Self::Weak => "🟡",
            Self::Moderate => "🟠",
            Self::Strong => "🔴",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::None => "#94a3b8",
            Self::Weak => "#fbbf24",
            Self::Moderate => "#f97316",
            Self::Strong => "#ef4444",
        }
    }

    pub fn points(&self) -> u64 {
        match self {
            Self::None => 5,
            Self::Weak => 15,
            Self::Moderate => 40,
            Self::Strong => 100,
        }
    }
}

/// Complete signal detection result
/// Tier: T2-C (composed of T2-P primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    pub prr: PRR,
    pub ror: ROR,
    pub ic: IC,
    pub eb05: EB05,
    pub case_count: CaseCount,
    pub chi_square: f64,
}

impl SignalResult {
    /// Determine overall signal strength from all metrics
    pub fn overall_strength(&self) -> SignalStrength {
        if !self.case_count.sufficient() {
            return SignalStrength::None;
        }

        let prr_signal = self.prr.is_signal();
        let eb05_signal = self.eb05.is_signal();
        let chi_significant = self.chi_square >= 3.841; // p < 0.05

        if prr_signal && eb05_signal && chi_significant {
            self.prr.strength()
        } else if prr_signal || eb05_signal {
            SignalStrength::Weak
        } else {
            SignalStrength::None
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T3 DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Drug-Event pair from FAERS
/// Tier: T3 (domain-specific entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEvent {
    pub drug_name: String,
    pub event_term: String,
    pub case_count: CaseCount,
    pub signal: Option<SignalResult>,
}

impl DrugEvent {
    pub fn new(drug: impl Into<String>, event: impl Into<String>, cases: u32) -> Self {
        Self {
            drug_name: drug.into(),
            event_term: event.into(),
            case_count: CaseCount(cases),
            signal: None,
        }
    }

    pub fn with_signal(mut self, signal: SignalResult) -> Self {
        self.signal = Some(signal);
        self
    }

    pub fn display_name(&self) -> String {
        format!("{}→{}", self.drug_name, self.event_term)
    }

    pub fn strength(&self) -> SignalStrength {
        self.signal
            .as_ref()
            .map(|s| s.overall_strength())
            .unwrap_or(SignalStrength::None)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SAMPLE DATA (for testing without live API)
// ═══════════════════════════════════════════════════════════════════════════

/// Generate sample drug-event pairs for game testing
pub fn sample_drug_events() -> Vec<DrugEvent> {
    vec![
        DrugEvent::new("Aspirin", "GI Bleeding", 838).with_signal(SignalResult {
            prr: PRR(3.2),
            ror: ROR(3.5),
            ic: IC(1.8),
            eb05: EB05(2.8),
            case_count: CaseCount(838),
            chi_square: 245.6,
        }),
        DrugEvent::new("Metformin", "Lactic Acidosis", 156).with_signal(SignalResult {
            prr: PRR(5.1),
            ror: ROR(5.8),
            ic: IC(2.4),
            eb05: EB05(4.2),
            case_count: CaseCount(156),
            chi_square: 89.3,
        }),
        DrugEvent::new("Warfarin", "Hemorrhage", 2341).with_signal(SignalResult {
            prr: PRR(8.7),
            ror: ROR(9.2),
            ic: IC(3.1),
            eb05: EB05(7.5),
            case_count: CaseCount(2341),
            chi_square: 1205.8,
        }),
        DrugEvent::new("Lisinopril", "Cough", 445),
        DrugEvent::new("Atorvastatin", "Myalgia", 312),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prr_threshold_works() {
        assert!(!PRR(1.5).is_signal());
        assert!(PRR(2.0).is_signal());
        assert!(PRR(3.0).is_signal());
    }

    #[test]
    fn signal_strength_points_increase() {
        assert!(SignalStrength::Weak.points() > SignalStrength::None.points());
        assert!(SignalStrength::Moderate.points() > SignalStrength::Weak.points());
        assert!(SignalStrength::Strong.points() > SignalStrength::Moderate.points());
    }

    #[test]
    fn case_count_minimum_enforced() {
        assert!(!CaseCount(2).sufficient());
        assert!(CaseCount(3).sufficient());
        assert!(CaseCount(100).sufficient());
    }

    #[test]
    fn drug_event_display_name() {
        let de = DrugEvent::new("Aspirin", "Bleeding", 100);
        assert_eq!(de.display_name(), "Aspirin→Bleeding");
    }

    #[test]
    fn sample_data_has_mixed_signals() {
        let samples = sample_drug_events();
        let with_signal: Vec<_> = samples.iter().filter(|d| d.signal.is_some()).collect();
        let without: Vec<_> = samples.iter().filter(|d| d.signal.is_none()).collect();

        assert!(!with_signal.is_empty());
        assert!(!without.is_empty());
    }
}
