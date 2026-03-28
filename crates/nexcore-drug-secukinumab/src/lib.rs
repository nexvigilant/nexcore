//! # nexcore-drug-secukinumab
//!
//! Secukinumab (Cosentyx) — anti-IL-17A antibody from Novartis.
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

/// Secukinumab drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_secukinumab::Secukinumab;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let s = Secukinumab::new();
/// assert_eq!(s.drug().generic_name, "secukinumab");
/// assert!(!s.drug().has_boxed_warning());
/// assert!(s.signal_count() >= 4);
/// ```
pub struct Secukinumab {
    drug: Drug,
}

impl Secukinumab {
    /// Construct a new `Secukinumab` instance with the canonical drug data.
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

impl Default for Secukinumab {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Secukinumab {
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
    fn secukinumab_loads() {
        let s = Secukinumab::new();
        assert_eq!(s.drug().generic_name, "secukinumab");
        assert_eq!(s.drug().brand_names[0], "Cosentyx");
    }

    #[test]
    fn secukinumab_no_boxed_warning() {
        let s = Secukinumab::new();
        assert!(!s.drug().has_boxed_warning());
    }

    #[test]
    fn secukinumab_signal_count() {
        let s = Secukinumab::new();
        assert!(s.signal_count() >= 4);
    }

    #[test]
    fn secukinumab_candida_is_strongest_signal() {
        let s = Secukinumab::new();
        let sig = s.strongest_signal().expect("has signals");
        assert!(
            sig.event.to_lowercase().contains("candida"),
            "strongest signal should be candida infection"
        );
    }

    #[test]
    fn secukinumab_ibd_signal_present() {
        let s = Secukinumab::new();
        assert!(
            s.signal_portfolio()
                .iter()
                .any(|sig| sig.event.contains("bowel") || sig.event.contains("IBD"))
        );
    }
}
