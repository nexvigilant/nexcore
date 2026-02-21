#![allow(dead_code)]
//! Signal detection screen — Slint bindings for 5-metric display.
//!
//! ## Primitive Grounding
//! - κ (Comparison): each metric is a comparison of observed vs expected
//! - N (Quantity): numeric metric values
//! - ∂ (Boundary): threshold indicators
//! - ∃ (Existence): signal_detected flag
//!
//! ## Tier: T3

use nexcore_watch_core::SignalResult;

/// Signal screen view model — formatted metrics for Slint binding.
///
/// ## Primitive: μ (Mapping) — domain → view
/// ## Tier: T2-C
#[derive(Debug, Clone)]
pub struct SignalViewModel {
    /// Drug name — λ (Location)
    pub drug: String,
    /// Event name — λ (Location)
    pub event: String,
    /// PRR formatted — κ (Comparison)
    pub prr_text: String,
    /// ROR formatted — κ (Comparison)
    pub ror_text: String,
    /// IC formatted — κ + ρ (Comparison + Recursion)
    pub ic_text: String,
    /// EBGM formatted — κ + ν (Comparison + Frequency)
    pub ebgm_text: String,
    /// Chi-squared formatted — κ + Σ (Comparison + Sum)
    pub chi2_text: String,
    /// Signal detected — ∃ (Existence)
    pub signal_detected: bool,
    /// Signal indicator text
    pub signal_text: &'static str,
    /// Signal indicator color hex
    pub signal_color: &'static str,
}

impl SignalViewModel {
    /// Build view model from SignalResult.
    ///
    /// ## Primitive: μ (Mapping) — SignalResult → ViewModel
    /// ## Tier: T2-C
    #[must_use]
    pub fn from_result(result: &SignalResult) -> Self {
        Self {
            drug: result.drug.clone(),
            event: result.event.clone(),
            prr_text: format!("{:.2}", result.prr),
            ror_text: format!("{:.2}", result.ror),
            ic_text: format!("{:.2}", result.ic),
            ebgm_text: format!("{:.2}", result.ebgm),
            chi2_text: format!("{:.2}", result.chi_squared),
            signal_detected: result.signal_detected,
            signal_text: if result.signal_detected {
                "SIGNAL"
            } else {
                "NO SIGNAL"
            },
            signal_color: if result.signal_detected {
                "#F44336"
            } else {
                "#4CAF50"
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_detected_view_model() {
        let result = SignalResult::compute_all("Aspirin", "GI Bleed", 15.0, 100.0, 20.0, 10000.0);
        let vm = SignalViewModel::from_result(&result);
        assert_eq!(vm.drug, "Aspirin");
        assert_eq!(vm.event, "GI Bleed");
        assert!(vm.signal_detected);
        assert_eq!(vm.signal_text, "SIGNAL");
        assert_eq!(vm.signal_color, "#F44336");
    }

    #[test]
    fn no_signal_view_model() {
        let result = SignalResult::compute_all("Placebo", "Headache", 10.0, 90.0, 100.0, 900.0);
        let vm = SignalViewModel::from_result(&result);
        assert!(!vm.signal_detected);
        assert_eq!(vm.signal_text, "NO SIGNAL");
        assert_eq!(vm.signal_color, "#4CAF50");
    }
}
