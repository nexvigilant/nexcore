//! # nexcore-drug-apixaban
//!
//! Apixaban (Eliquis) — Factor Xa inhibitor from Pfizer/Bristol-Myers Squibb.
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

/// Apixaban drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_apixaban::Apixaban;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let a = Apixaban::new();
/// assert_eq!(a.drug().generic_name, "apixaban");
/// assert!(a.drug().has_boxed_warning());
/// assert!(a.signal_count() >= 4);
/// ```
pub struct Apixaban {
    drug: Drug,
}

impl Apixaban {
    /// Construct a new `Apixaban` instance with the canonical drug data.
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

impl Default for Apixaban {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Apixaban {
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
    fn apixaban_loads() {
        let a = Apixaban::new();
        assert_eq!(a.drug().generic_name, "apixaban");
        assert_eq!(a.drug().brand_names[0], "Eliquis");
    }

    #[test]
    fn apixaban_has_boxed_warning() {
        let a = Apixaban::new();
        assert!(a.drug().has_boxed_warning());
    }

    #[test]
    fn apixaban_signal_count() {
        let a = Apixaban::new();
        assert!(a.signal_count() >= 4);
    }

    #[test]
    fn apixaban_spinal_haematoma_present() {
        let a = Apixaban::new();
        assert!(
            a.signal_portfolio()
                .iter()
                .any(|s| s.event.contains("epidural") || s.event.contains("haematoma"))
        );
    }

    #[test]
    fn apixaban_all_signals_on_label() {
        let a = Apixaban::new();
        assert!(a.off_label_signals().is_empty());
    }
}
