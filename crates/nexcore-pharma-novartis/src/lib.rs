#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Novartis AG pharmaceutical company model.
//!
//! Provides static catalog data for Novartis' marketed products and pipeline
//! candidates, implementing the [`CompanyAnalysis`] trait from `nexcore-pharma`.

mod catalog;

pub use catalog::company;

use nexcore_pharma::{
    Company, CompanyAnalysis, Phase, PipelineCandidate, Product, SignalSummary, TherapeuticArea,
};

/// Novartis company analysis handle.
pub struct Novartis {
    data: Company,
}

impl Novartis {
    /// Load the Novartis company model.
    pub fn load() -> Self {
        Self {
            data: catalog::company(),
        }
    }
}

impl CompanyAnalysis for Novartis {
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

    fn pipeline_by_phase(&self, phase: Phase) -> Vec<&PipelineCandidate> {
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
        let co = Novartis::load();
        assert_eq!(co.company().ticker.as_deref(), Some("NVS"));
        assert_eq!(co.company().name, "Novartis AG");
    }

    #[test]
    fn product_count_matches() {
        assert_eq!(Novartis::load().company().products.len(), 10);
    }

    #[test]
    fn pipeline_is_non_empty() {
        assert!(!Novartis::load().company().pipeline.is_empty());
    }

    #[test]
    fn zolgensma_present() {
        let found = Novartis::load()
            .company()
            .products
            .iter()
            .any(|p| p.generic_name == "onasemnogene abeparvovec");
        assert!(found, "Zolgensma/onasemnogene must be in catalog");
    }
}
