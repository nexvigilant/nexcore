//! Company analysis trait.
//!
//! `CompanyAnalysis` is the query interface over a [`Company`] aggregate.
//! Implementors gain a standard set of analytical views: signal portfolio,
//! pipeline by phase, products with boxed warnings, and therapeutic focus
//! with product counts.
//!
//! The trait is object-safe — all methods return owned or reference types
//! with no associated types or generic parameters.

use crate::{Company, Phase, PipelineCandidate, Product, SignalSummary, TherapeuticArea};

/// Analytical query interface over a pharmaceutical company.
///
/// Provides domain-level views without requiring callers to traverse
/// the raw [`Company`] aggregate fields directly.
///
/// # Implementing
///
/// The blanket `DefaultCompanyAnalysis` struct in this module implements
/// the trait over `&Company` for convenience. For richer implementations
/// (e.g. with cached indexes), implement the trait on your own wrapper type.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::{Company, CompanyId, TherapeuticArea, Phase};
/// use nexcore_pharma::analysis::DefaultAnalysis;
/// use nexcore_pharma::CompanyAnalysis;
///
/// let company = Company {
///     id: CompanyId::new("example-pharma"),
///     name: "Example Pharma".to_string(),
///     ticker: None,
///     headquarters: None,
///     therapeutic_areas: vec![TherapeuticArea::Oncology],
///     products: vec![],
///     pipeline: vec![],
///     safety_communications: vec![],
/// };
///
/// let analysis = DefaultAnalysis::new(&company);
/// assert_eq!(analysis.company().id.as_str(), "example-pharma");
/// assert!(analysis.signal_portfolio().is_empty());
/// assert!(analysis.pipeline_by_phase(Phase::Phase3).is_empty());
/// assert!(analysis.products_with_boxed_warnings().is_empty());
/// ```
pub trait CompanyAnalysis {
    /// Returns a reference to the underlying company aggregate.
    fn company(&self) -> &Company;

    /// Returns all signal summaries across all products, flattened.
    ///
    /// Signals are returned in product order (outer) then signal order
    /// (inner). No deduplication is performed.
    fn signal_portfolio(&self) -> Vec<&SignalSummary>;

    /// Returns all pipeline candidates matching exactly the given phase.
    fn pipeline_by_phase(&self, phase: Phase) -> Vec<&PipelineCandidate>;

    /// Returns all products that carry an FDA boxed warning.
    fn products_with_boxed_warnings(&self) -> Vec<&Product>;

    /// Returns each therapeutic area alongside the count of products
    /// classified under it, sorted descending by count.
    ///
    /// Areas with zero products are omitted.
    fn therapeutic_focus(&self) -> Vec<(TherapeuticArea, usize)>;
}

/// Default implementation of [`CompanyAnalysis`] that operates directly
/// over a `&Company` reference without additional indexing.
pub struct DefaultAnalysis<'a> {
    company: &'a Company,
}

impl<'a> DefaultAnalysis<'a> {
    /// Wrap a company reference for analysis.
    pub fn new(company: &'a Company) -> Self {
        Self { company }
    }
}

impl<'a> CompanyAnalysis for DefaultAnalysis<'a> {
    fn company(&self) -> &Company {
        self.company
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
        // Sort descending by count, then by display name for stable ordering
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
    use crate::{
        Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, SignalSummary,
        SignalVerdict, TherapeuticArea,
    };

    fn make_product(
        name: &str,
        area: TherapeuticArea,
        boxed: bool,
        signals: Vec<SignalSummary>,
    ) -> Product {
        Product {
            generic_name: name.to_string(),
            brand_names: vec![],
            rxcui: None,
            therapeutic_area: area,
            approval_year: None,
            safety_profile: SafetyProfile {
                boxed_warning: boxed,
                rems: false,
                signals,
                label_warnings: vec![],
            },
        }
    }

    fn make_signal(event: &str, verdict: SignalVerdict) -> SignalSummary {
        SignalSummary {
            event: event.to_string(),
            prr: 2.0,
            ror: 2.5,
            cases: 100,
            on_label: false,
            verdict,
        }
    }

    fn make_candidate(name: &str, phase: Phase, area: TherapeuticArea) -> PipelineCandidate {
        PipelineCandidate {
            name: name.to_string(),
            mechanism: "unknown".to_string(),
            phase,
            indication: "test indication".to_string(),
            therapeutic_area: area,
        }
    }

    fn make_company() -> Company {
        Company {
            id: CompanyId::new("test-co"),
            name: "Test Co".to_string(),
            ticker: None,
            headquarters: None,
            therapeutic_areas: vec![TherapeuticArea::Oncology, TherapeuticArea::Cardiovascular],
            products: vec![
                make_product(
                    "drugA",
                    TherapeuticArea::Oncology,
                    true,
                    vec![make_signal("nausea", SignalVerdict::Strong)],
                ),
                make_product(
                    "drugB",
                    TherapeuticArea::Oncology,
                    false,
                    vec![
                        make_signal("fatigue", SignalVerdict::Moderate),
                        make_signal("rash", SignalVerdict::Noise),
                    ],
                ),
                make_product("drugC", TherapeuticArea::Cardiovascular, false, vec![]),
            ],
            pipeline: vec![
                make_candidate("cand1", Phase::Phase3, TherapeuticArea::Oncology),
                make_candidate("cand2", Phase::Phase3, TherapeuticArea::Oncology),
                make_candidate("cand3", Phase::Phase1, TherapeuticArea::Cardiovascular),
            ],
            safety_communications: vec![],
        }
    }

    #[test]
    fn company_ref_returns_same_company() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        assert_eq!(analysis.company().id.as_str(), "test-co");
    }

    #[test]
    fn signal_portfolio_flattens_all_products() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        // drugA: 1, drugB: 2, drugC: 0  → total 3
        assert_eq!(analysis.signal_portfolio().len(), 3);
    }

    #[test]
    fn signal_portfolio_empty_when_no_products() {
        let co = Company {
            id: CompanyId::new("empty"),
            name: "Empty Pharma".to_string(),
            ticker: None,
            headquarters: None,
            therapeutic_areas: vec![],
            products: vec![],
            pipeline: vec![],
            safety_communications: vec![],
        };
        let analysis = DefaultAnalysis::new(&co);
        assert!(analysis.signal_portfolio().is_empty());
    }

    #[test]
    fn pipeline_by_phase_filters_correctly() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        let phase3 = analysis.pipeline_by_phase(Phase::Phase3);
        assert_eq!(phase3.len(), 2);
        assert!(phase3.iter().all(|c| c.phase == Phase::Phase3));

        let phase1 = analysis.pipeline_by_phase(Phase::Phase1);
        assert_eq!(phase1.len(), 1);

        let phase2 = analysis.pipeline_by_phase(Phase::Phase2);
        assert!(phase2.is_empty());
    }

    #[test]
    fn products_with_boxed_warnings_filters_correctly() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        let boxed = analysis.products_with_boxed_warnings();
        assert_eq!(boxed.len(), 1);
        assert_eq!(boxed[0].generic_name, "drugA");
    }

    #[test]
    fn products_with_boxed_warnings_empty_when_none() {
        let mut co = make_company();
        for p in &mut co.products {
            p.safety_profile.boxed_warning = false;
        }
        let analysis = DefaultAnalysis::new(&co);
        assert!(analysis.products_with_boxed_warnings().is_empty());
    }

    #[test]
    fn therapeutic_focus_counts_by_area() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        let focus = analysis.therapeutic_focus();
        // Oncology: 2, Cardiovascular: 1
        assert_eq!(focus.len(), 2);
        assert_eq!(focus[0].0, TherapeuticArea::Oncology);
        assert_eq!(focus[0].1, 2);
        assert_eq!(focus[1].0, TherapeuticArea::Cardiovascular);
        assert_eq!(focus[1].1, 1);
    }

    #[test]
    fn therapeutic_focus_sorted_descending_by_count() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        let focus = analysis.therapeutic_focus();
        for i in 0..focus.len().saturating_sub(1) {
            assert!(
                focus[i].1 >= focus[i + 1].1,
                "focus not sorted descending: {} < {}",
                focus[i].1,
                focus[i + 1].1
            );
        }
    }

    #[test]
    fn therapeutic_focus_omits_zero_count_areas() {
        let co = make_company();
        let analysis = DefaultAnalysis::new(&co);
        let focus = analysis.therapeutic_focus();
        // No products in Neuroscience, Metabolic, etc.
        assert!(focus.iter().all(|(_, count)| *count > 0));
    }
}
