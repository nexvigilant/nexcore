//! # nexcore-drug-dapagliflozin
//!
//! Dapagliflozin (Farxiga) — SGLT2 inhibitor from AstraZeneca.
//! Static drug safety model implementing [`DrugAnalysis`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod catalog;

use nexcore_drug::{
    Drug, SignalEntry,
    analysis::{DefaultDrugAnalysis, DrugAnalysis, SignalComparison},
};

/// Dapagliflozin drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_dapagliflozin::Dapagliflozin;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let d = Dapagliflozin::new();
/// assert_eq!(d.drug().generic_name, "dapagliflozin");
/// assert!(!d.drug().has_boxed_warning());
/// assert!(d.signal_count() >= 5);
/// ```
pub struct Dapagliflozin {
    drug: Drug,
}

impl Dapagliflozin {
    /// Construct a new `Dapagliflozin` instance with the canonical drug data.
    pub fn new() -> Self {
        Self {
            drug: catalog::drug(),
        }
    }

    /// Total number of safety signals in the portfolio.
    pub fn signal_count(&self) -> usize {
        self.drug.signal_count()
    }
}

impl Default for Dapagliflozin {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Dapagliflozin {
    fn drug(&self) -> &Drug {
        &self.drug
    }

    fn signal_portfolio(&self) -> &[SignalEntry] {
        &self.drug.safety_signals
    }

    fn strongest_signal(&self) -> Option<&SignalEntry> {
        self.drug.strongest_signal()
    }

    fn on_label_signals(&self) -> Vec<&SignalEntry> {
        self.drug.on_label_signals()
    }

    fn off_label_signals(&self) -> Vec<&SignalEntry> {
        self.drug.off_label_signals()
    }

    fn compare_signals(&self, other: &dyn DrugAnalysis) -> Vec<SignalComparison> {
        DefaultDrugAnalysis::new(&self.drug).compare_signals(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dapagliflozin_loads() {
        let d = Dapagliflozin::new();
        assert_eq!(d.drug().generic_name, "dapagliflozin");
        assert_eq!(d.drug().brand_names[0], "Farxiga");
    }

    #[test]
    fn dapagliflozin_no_boxed_warning() {
        let d = Dapagliflozin::new();
        assert!(!d.drug().has_boxed_warning());
    }

    #[test]
    fn dapagliflozin_signal_count() {
        let d = Dapagliflozin::new();
        assert!(d.signal_count() >= 5);
    }

    #[test]
    fn dapagliflozin_amputation_is_off_label() {
        let d = Dapagliflozin::new();
        let off = d.off_label_signals();
        assert!(
            off.iter().any(|s| s.event.contains("amputation")),
            "lower limb amputation should be off-label signal"
        );
    }

    #[test]
    fn dapagliflozin_fournier_gangrene_present() {
        let d = Dapagliflozin::new();
        assert!(
            d.signal_portfolio()
                .iter()
                .any(|s| s.event.contains("Fournier"))
        );
    }
}
