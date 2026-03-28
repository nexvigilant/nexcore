#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Eli Lilly and Company pharmaceutical company model.
//!
//! Provides static catalog data for Lilly's marketed products and pipeline
//! candidates, implementing the [`CompanyAnalysis`] trait from `nexcore-pharma`.

mod catalog;

pub use catalog::company;

use nexcore_pharma::{Company, CompanyAnalysis, Phase, Product, SignalSummary, TherapeuticArea};

/// Eli Lilly company analysis handle.
pub struct Lilly {
    data: Company,
}

impl Lilly {
    /// Load the Lilly company model.
    pub fn load() -> Self {
        Self {
            data: catalog::company(),
        }
    }
}

impl CompanyAnalysis for Lilly {
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
        let lilly = Lilly::load();
        let co = lilly.company();
        assert_eq!(co.ticker.as_deref(), Some("LLY"));
        assert_eq!(co.name, "Eli Lilly and Company");
    }

    #[test]
    fn product_count_matches() {
        let lilly = Lilly::load();
        assert_eq!(
            lilly.company().products.len(),
            11,
            "expected 11 marketed products"
        );
    }

    #[test]
    fn pipeline_is_non_empty() {
        let lilly = Lilly::load();
        assert!(!lilly.company().pipeline.is_empty());
    }

    #[test]
    fn tirzepatide_present() {
        let lilly = Lilly::load();
        let found = lilly
            .company()
            .products
            .iter()
            .any(|p| p.generic_name == "tirzepatide");
        assert!(found, "tirzepatide must be in catalog");
    }
}
