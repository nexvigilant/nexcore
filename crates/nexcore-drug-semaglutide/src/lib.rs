//! # nexcore-drug-semaglutide
//!
//! Semaglutide (Ozempic/Wegovy/Rybelsus) — GLP-1 receptor agonist from Novo Nordisk.
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

/// Semaglutide drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_semaglutide::Semaglutide;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let s = Semaglutide::new();
/// assert_eq!(s.drug().generic_name, "semaglutide");
/// assert!(s.drug().has_boxed_warning());
/// assert!(s.signal_count() >= 4);
/// ```
pub struct Semaglutide {
    drug: Drug,
}

impl Semaglutide {
    /// Construct a new `Semaglutide` instance with the canonical drug data.
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

impl Default for Semaglutide {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Semaglutide {
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
    fn semaglutide_loads() {
        let s = Semaglutide::new();
        assert_eq!(s.drug().generic_name, "semaglutide");
        assert_eq!(s.drug().brand_names[0], "Ozempic");
    }

    #[test]
    fn semaglutide_has_boxed_warning() {
        let s = Semaglutide::new();
        assert!(s.drug().has_boxed_warning());
    }

    #[test]
    fn semaglutide_signal_count() {
        let s = Semaglutide::new();
        assert!(s.signal_count() >= 4);
    }

    #[test]
    fn semaglutide_suicidal_ideation_is_off_label() {
        let s = Semaglutide::new();
        let off = s.off_label_signals();
        assert!(
            off.iter().any(|sig| sig.event.contains("ideation")),
            "suicidal ideation should be off-label signal"
        );
    }

    #[test]
    fn semaglutide_three_brand_names() {
        let s = Semaglutide::new();
        assert_eq!(s.drug().brand_names.len(), 3);
    }
}
