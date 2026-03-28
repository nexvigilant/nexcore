//! # nexcore-drug-tirzepatide
//!
//! Tirzepatide (Mounjaro/Zepbound) — GLP-1/GIP dual agonist from Eli Lilly.
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
    analysis::{ComparisonResult, DefaultDrugAnalysis, DrugAnalysis, SignalComparison},
};

/// Tirzepatide drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_tirzepatide::Tirzepatide;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let t = Tirzepatide::new();
/// assert_eq!(t.drug().generic_name, "tirzepatide");
/// assert!(t.drug().has_boxed_warning());
/// assert!(t.signal_count() >= 4);
/// ```
pub struct Tirzepatide {
    drug: Drug,
}

impl Tirzepatide {
    /// Construct a new `Tirzepatide` instance with the canonical drug data.
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

impl Default for Tirzepatide {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Tirzepatide {
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
    fn tirzepatide_loads() {
        let t = Tirzepatide::new();
        assert_eq!(t.drug().generic_name, "tirzepatide");
        assert_eq!(t.drug().brand_names[0], "Mounjaro");
    }

    #[test]
    fn tirzepatide_has_boxed_warning() {
        let t = Tirzepatide::new();
        assert!(t.drug().has_boxed_warning());
    }

    #[test]
    fn tirzepatide_signal_count() {
        let t = Tirzepatide::new();
        assert!(t.signal_count() >= 4);
    }

    #[test]
    fn tirzepatide_strongest_signal_is_gastroparesis() {
        let t = Tirzepatide::new();
        let s = t.strongest_signal().expect("has signals");
        assert_eq!(s.event, "Gastroparesis");
    }

    #[test]
    fn tirzepatide_off_label_signals_present() {
        let t = Tirzepatide::new();
        assert!(!t.off_label_signals().is_empty());
    }

    #[test]
    fn tirzepatide_compare_signals_with_self_all_neutral() {
        let t = Tirzepatide::new();
        let t2 = Tirzepatide::new();
        let comparisons = t.compare_signals(&t2);
        assert!(!comparisons.is_empty());
        assert!(
            comparisons
                .iter()
                .all(|c| c.advantage == ComparisonResult::Neutral)
        );
    }
}
