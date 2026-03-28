//! # nexcore-drug-adalimumab
//!
//! Adalimumab (Humira) — anti-TNF monoclonal antibody from AbbVie.
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

/// Adalimumab drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_adalimumab::Adalimumab;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let a = Adalimumab::new();
/// assert_eq!(a.drug().generic_name, "adalimumab");
/// assert!(a.drug().has_boxed_warning());
/// assert!(a.signal_count() >= 4);
/// ```
pub struct Adalimumab {
    drug: Drug,
}

impl Adalimumab {
    /// Construct a new `Adalimumab` instance with the canonical drug data.
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

impl Default for Adalimumab {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Adalimumab {
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
    fn adalimumab_loads() {
        let a = Adalimumab::new();
        assert_eq!(a.drug().generic_name, "adalimumab");
        assert_eq!(a.drug().brand_names[0], "Humira");
    }

    #[test]
    fn adalimumab_has_boxed_warning() {
        let a = Adalimumab::new();
        assert!(a.drug().has_boxed_warning());
    }

    #[test]
    fn adalimumab_signal_count() {
        let a = Adalimumab::new();
        assert!(a.signal_count() >= 4);
    }

    #[test]
    fn adalimumab_serious_infection_is_strongest() {
        let a = Adalimumab::new();
        let s = a.strongest_signal().expect("has signals");
        assert!(
            s.event.to_lowercase().contains("infection"),
            "strongest signal should be serious infection"
        );
    }

    #[test]
    fn adalimumab_owner_is_abbvie() {
        let a = Adalimumab::new();
        assert_eq!(a.drug().owner.as_deref(), Some("AbbVie Inc."));
    }
}
