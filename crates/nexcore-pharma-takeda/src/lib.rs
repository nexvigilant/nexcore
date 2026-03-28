#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Takeda Pharmaceutical Company pharmaceutical company model.
//!
//! Provides static catalog data for Takeda's marketed products and pipeline
//! candidates, implementing the [`CompanyAnalysis`] trait from `nexcore-pharma`.
//!
//! ## Coverage
//!
//! - 12 marketed products across GI/Immunology, Rare Disease, Neuroscience,
//!   Oncology, and Infectious Disease
//! - Includes Exkivity (mobocertinib), voluntarily withdrawn October 2023
//! - 4 pipeline candidates (TAK-279, TAK-861, TAK-999, soticlestat)
//! - Key safety signals: ICLUSIG arterial occlusive events (boxed warning),
//!   Vyvanse abuse potential (boxed warning), Entyvio infection risk

mod catalog;

pub use catalog::{company, pipeline, products, safety_communications};

use nexcore_pharma::{
    Company, CompanyAnalysis, Phase, PipelineCandidate, Product, SignalSummary, TherapeuticArea,
};

/// Takeda Pharmaceutical Company analysis handle.
pub struct Takeda {
    company: Company,
}

impl Takeda {
    /// Load the Takeda company model.
    pub fn load() -> Self {
        Self {
            company: catalog::company(),
        }
    }
}

impl CompanyAnalysis for Takeda {
    fn company(&self) -> &Company {
        &self.company
    }

    fn signal_portfolio(&self) -> Vec<&SignalSummary> {
        self.company
            .products
            .iter()
            .flat_map(|p| p.safety_profile.signals.iter())
            .collect()
    }

    fn pipeline_by_phase(&self, phase: Phase) -> Vec<&PipelineCandidate> {
        self.company
            .pipeline
            .iter()
            .filter(|c| c.phase == phase)
            .collect()
    }

    fn products_with_boxed_warnings(&self) -> Vec<&Product> {
        self.company
            .products
            .iter()
            .filter(|p| p.safety_profile.boxed_warning)
            .collect()
    }

    fn therapeutic_focus(&self) -> Vec<(TherapeuticArea, usize)> {
        let mut counts: std::collections::HashMap<TherapeuticArea, usize> =
            std::collections::HashMap::new();
        for product in &self.company.products {
            *counts.entry(product.therapeutic_area).or_insert(0) += 1;
        }
        let mut result: Vec<(TherapeuticArea, usize)> = counts.into_iter().collect();
        result.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.to_string().cmp(&b.0.to_string()))
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn company_loads() {
        let takeda = Takeda::load();
        let co = takeda.company();
        assert_eq!(co.ticker.as_deref(), Some("TAK"));
        assert_eq!(co.name, "Takeda Pharmaceutical Company");
    }

    #[test]
    fn product_count_matches() {
        let takeda = Takeda::load();
        assert_eq!(
            takeda.company().products.len(),
            12,
            "expected 12 marketed products"
        );
    }

    #[test]
    fn pipeline_is_non_empty() {
        let takeda = Takeda::load();
        assert!(!takeda.company().pipeline.is_empty());
    }

    #[test]
    fn exkivity_is_labeled_withdrawn() {
        let takeda = Takeda::load();
        let mobocertinib = takeda
            .company()
            .products
            .iter()
            .find(|p| p.generic_name == "mobocertinib");
        let product = mobocertinib.expect("mobocertinib must be present in catalog");
        assert!(
            !product.safety_profile.label_warnings.is_empty(),
            "mobocertinib (Exkivity) withdrawal must appear in label_warnings"
        );
        let has_withdrawal = product
            .safety_profile
            .label_warnings
            .iter()
            .any(|w| w.contains("withdrawn"));
        assert!(has_withdrawal, "label_warnings must reference withdrawal");
    }

    #[test]
    fn iclusig_has_boxed_warning() {
        let takeda = Takeda::load();
        let iclusig = takeda
            .company()
            .products
            .iter()
            .find(|p| p.generic_name == "ponatinib");
        let product = iclusig.expect("ponatinib must be present in catalog");
        assert!(
            product.safety_profile.boxed_warning,
            "ponatinib (ICLUSIG) carries a boxed warning for arterial occlusive events"
        );
        assert!(
            product.safety_profile.rems,
            "ponatinib (ICLUSIG) requires a REMS program"
        );
    }

    #[test]
    fn signal_portfolio_non_empty() {
        let takeda = Takeda::load();
        assert!(
            !takeda.signal_portfolio().is_empty(),
            "Takeda portfolio must have at least one signal"
        );
    }

    #[test]
    fn products_with_boxed_warnings_includes_vyvanse_and_iclusig() {
        let takeda = Takeda::load();
        let boxed: Vec<&str> = takeda
            .products_with_boxed_warnings()
            .iter()
            .map(|p| p.generic_name.as_str())
            .collect();
        assert!(
            boxed.contains(&"lisdexamfetamine"),
            "Vyvanse must have boxed warning"
        );
        assert!(
            boxed.contains(&"ponatinib"),
            "ICLUSIG must have boxed warning"
        );
    }

    #[test]
    fn pipeline_by_phase3_returns_three_candidates() {
        let takeda = Takeda::load();
        let phase3 = takeda.pipeline_by_phase(Phase::Phase3);
        assert_eq!(phase3.len(), 3, "TAK-279, TAK-861, soticlestat are Phase 3");
    }
}
