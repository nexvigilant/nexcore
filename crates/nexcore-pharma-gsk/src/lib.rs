#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! GSK plc pharmaceutical company model.
//!
//! Provides static catalog data for GSK's marketed products and pipeline
//! candidates, implementing the [`CompanyAnalysis`] trait from `nexcore-pharma`.

mod catalog;

pub use catalog::company;

use nexcore_pharma::{
    Company, CompanyAnalysis, Phase, PipelineCandidate, Product, SignalSummary, TherapeuticArea,
};

/// GSK company analysis handle.
pub struct Gsk {
    data: Company,
}

impl Gsk {
    /// Load the GSK company model.
    pub fn load() -> Self {
        Self {
            data: catalog::company(),
        }
    }
}

impl CompanyAnalysis for Gsk {
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
        let co = Gsk::load();
        assert_eq!(co.company().ticker.as_deref(), Some("GSK"));
        assert_eq!(co.company().name, "GSK plc");
    }

    #[test]
    fn product_count_matches() {
        assert_eq!(Gsk::load().company().products.len(), 8);
    }

    #[test]
    fn pipeline_is_non_empty() {
        assert!(!Gsk::load().company().pipeline.is_empty());
    }

    #[test]
    fn shingrix_present() {
        let found = Gsk::load()
            .company()
            .products
            .iter()
            .any(|p| p.brand_names.iter().any(|b| b == "Shingrix"));
        assert!(found, "Shingrix must be in catalog");
    }
}
