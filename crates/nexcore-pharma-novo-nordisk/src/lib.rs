#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Novo Nordisk A/S pharmaceutical company model.
//!
//! Provides static catalog data for Novo Nordisk's marketed products and
//! pipeline candidates, implementing the [`CompanyAnalysis`] trait from
//! `nexcore-pharma`.
//!
//! Novo Nordisk is the GLP-1 pioneer — dominant in diabetes and obesity
//! with semaglutide (Ozempic, Wegovy, Rybelsus) and a deep insulin
//! franchise. Secondary focus in rare blood disorders (hemophilia) and
//! growth hormone.

mod catalog;

pub use catalog::company;

use nexcore_pharma::{Company, CompanyAnalysis, Phase, Product, SignalSummary, TherapeuticArea};

/// Novo Nordisk company analysis handle.
pub struct NovoNordisk {
    data: Company,
}

impl NovoNordisk {
    /// Load the Novo Nordisk company model.
    pub fn load() -> Self {
        Self {
            data: catalog::company(),
        }
    }
}

impl CompanyAnalysis for NovoNordisk {
    fn company(&self) -> &Company {
        &self.data
    }

    fn signal_portfolio(&self) -> Vec<&SignalSummary> {
        self.data
            .products
            .iter()
            .flat_map(|p| p.safety_profile.signals.iter())
            .collect()
    }

    fn pipeline_by_phase(&self, phase: Phase) -> Vec<&nexcore_pharma::PipelineCandidate> {
        self.data
            .pipeline
            .iter()
            .filter(|c| c.phase == phase)
            .collect()
    }

    fn products_with_boxed_warnings(&self) -> Vec<&Product> {
        self.data
            .products
            .iter()
            .filter(|p| p.safety_profile.boxed_warning)
            .collect()
    }

    fn therapeutic_focus(&self) -> Vec<(TherapeuticArea, usize)> {
        let mut counts: std::collections::HashMap<TherapeuticArea, usize> =
            std::collections::HashMap::new();
        for product in &self.data.products {
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
        let nn = NovoNordisk::load();
        let co = nn.company();
        assert_eq!(co.ticker.as_deref(), Some("NVO"));
        assert_eq!(co.name, "Novo Nordisk A/S");
    }

    #[test]
    fn product_count_matches() {
        let nn = NovoNordisk::load();
        assert_eq!(
            nn.company().products.len(),
            11,
            "expected 11 marketed products"
        );
    }

    #[test]
    fn pipeline_is_non_empty() {
        let nn = NovoNordisk::load();
        assert!(!nn.company().pipeline.is_empty());
    }

    #[test]
    fn semaglutide_present() {
        let nn = NovoNordisk::load();
        let found = nn
            .company()
            .products
            .iter()
            .any(|p| p.generic_name == "semaglutide");
        assert!(found, "semaglutide (Ozempic) must be in catalog");
    }

    #[test]
    fn glp1_products_have_boxed_warning() {
        let nn = NovoNordisk::load();
        let glp1_names = [
            "semaglutide",
            "semaglutide 2.4mg",
            "oral semaglutide",
            "liraglutide",
            "liraglutide 3mg",
        ];
        for name in &glp1_names {
            let product = nn
                .company()
                .products
                .iter()
                .find(|p| p.generic_name.as_str() == *name)
                .unwrap_or_else(|| panic!("product {name} not found"));
            assert!(
                product.safety_profile.boxed_warning,
                "{name} must carry boxed warning"
            );
        }
    }

    #[test]
    fn signal_portfolio_is_non_empty() {
        let nn = NovoNordisk::load();
        assert!(
            !nn.signal_portfolio().is_empty(),
            "semaglutide signals must be present"
        );
    }

    #[test]
    fn phase3_pipeline_present() {
        let nn = NovoNordisk::load();
        let phase3 = nn.pipeline_by_phase(Phase::Phase3);
        assert!(!phase3.is_empty(), "CagriSema and others must be Phase 3");
    }

    #[test]
    fn metabolic_dominates_therapeutic_focus() {
        let nn = NovoNordisk::load();
        let focus = nn.therapeutic_focus();
        assert!(!focus.is_empty(), "therapeutic focus must not be empty");
        assert_eq!(
            focus[0].0,
            TherapeuticArea::Metabolic,
            "Metabolic must be top therapeutic area"
        );
    }

    #[test]
    fn safety_communications_present() {
        let nn = NovoNordisk::load();
        assert!(
            !nn.company().safety_communications.is_empty(),
            "Ozempic gastroparesis update must be present"
        );
    }
}
