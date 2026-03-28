//! # nexcore-drug-donanemab
//!
//! Donanemab (Kisunla) — anti-amyloid antibody from Eli Lilly for early Alzheimer's.
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

/// Donanemab drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_donanemab::Donanemab;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let d = Donanemab::new();
/// assert_eq!(d.drug().generic_name, "donanemab");
/// assert!(d.drug().has_boxed_warning());
/// assert!(d.signal_count() >= 4);
/// ```
pub struct Donanemab {
    drug: Drug,
}

impl Donanemab {
    /// Construct a new `Donanemab` instance with the canonical drug data.
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

impl Default for Donanemab {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Donanemab {
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
    fn donanemab_loads() {
        let d = Donanemab::new();
        assert_eq!(d.drug().generic_name, "donanemab");
        assert_eq!(d.drug().brand_names[0], "Kisunla");
    }

    #[test]
    fn donanemab_has_boxed_warning() {
        let d = Donanemab::new();
        assert!(d.drug().has_boxed_warning());
    }

    #[test]
    fn donanemab_signal_count() {
        let d = Donanemab::new();
        assert!(d.signal_count() >= 4);
    }

    #[test]
    fn donanemab_aria_e_is_strongest_signal() {
        let d = Donanemab::new();
        let s = d.strongest_signal().expect("has signals");
        assert!(
            s.event.contains("ARIA-E") || s.event.contains("Oedema"),
            "strongest signal should be ARIA-E"
        );
    }

    #[test]
    fn donanemab_all_signals_on_label() {
        let d = Donanemab::new();
        assert!(d.off_label_signals().is_empty());
    }
}
