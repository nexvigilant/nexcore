//! Core Company type.
//!
//! A pharmaceutical company is the central aggregate in this domain —
//! it owns products, a pipeline, and a history of safety communications.

use serde::{Deserialize, Serialize};

use crate::{CompanyId, PipelineCandidate, Product, SafetyCommunication, TherapeuticArea};

/// A pharmaceutical company with its full domain profile.
///
/// Aggregates marketed products, pipeline candidates, and safety
/// communications under a single typed identity.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::{Company, CompanyId, TherapeuticArea};
///
/// let company = Company {
///     id: CompanyId::new("pfizer-inc"),
///     name: "Pfizer Inc.".to_string(),
///     ticker: Some("PFE".to_string()),
///     headquarters: Some("New York, NY".to_string()),
///     therapeutic_areas: vec![TherapeuticArea::Oncology, TherapeuticArea::Vaccines],
///     products: vec![],
///     pipeline: vec![],
///     safety_communications: vec![],
/// };
/// assert_eq!(company.id.as_str(), "pfizer-inc");
/// assert_eq!(company.product_count(), 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    /// Stable machine-readable identifier
    pub id: CompanyId,
    /// Full legal or trading name
    pub name: String,
    /// Stock ticker symbol, if publicly traded
    pub ticker: Option<String>,
    /// Primary headquarters location
    pub headquarters: Option<String>,
    /// Declared therapeutic focus areas
    pub therapeutic_areas: Vec<TherapeuticArea>,
    /// Marketed products
    pub products: Vec<Product>,
    /// Pipeline candidates in development
    pub pipeline: Vec<PipelineCandidate>,
    /// Issued safety communications
    pub safety_communications: Vec<SafetyCommunication>,
}

impl Company {
    /// Number of marketed products.
    pub fn product_count(&self) -> usize {
        self.products.len()
    }

    /// Number of pipeline candidates.
    pub fn pipeline_count(&self) -> usize {
        self.pipeline.len()
    }

    /// Number of safety communications issued.
    pub fn safety_comm_count(&self) -> usize {
        self.safety_communications.len()
    }

    /// Returns `true` if the company has any declared focus in the
    /// given therapeutic area.
    pub fn operates_in(&self, area: TherapeuticArea) -> bool {
        self.therapeutic_areas.contains(&area)
    }

    /// Returns `true` if the company is publicly traded (ticker present).
    pub fn is_public(&self) -> bool {
        self.ticker.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Phase, SafetyProfile, SignalSummary, SignalVerdict,
        safety_comm::{CommType, SafetyCommunication},
    };

    fn minimal_company() -> Company {
        Company {
            id: CompanyId::new("test-pharma"),
            name: "Test Pharma Ltd.".to_string(),
            ticker: None,
            headquarters: None,
            therapeutic_areas: vec![TherapeuticArea::Oncology],
            products: vec![],
            pipeline: vec![],
            safety_communications: vec![],
        }
    }

    #[test]
    fn company_construction_minimal() {
        let c = minimal_company();
        assert_eq!(c.id.as_str(), "test-pharma");
        assert_eq!(c.name, "Test Pharma Ltd.");
        assert!(!c.is_public());
        assert_eq!(c.product_count(), 0);
        assert_eq!(c.pipeline_count(), 0);
        assert_eq!(c.safety_comm_count(), 0);
    }

    #[test]
    fn company_is_public_when_ticker_set() {
        let mut c = minimal_company();
        assert!(!c.is_public());
        c.ticker = Some("TEST".to_string());
        assert!(c.is_public());
    }

    #[test]
    fn company_operates_in_declared_area() {
        let c = minimal_company();
        assert!(c.operates_in(TherapeuticArea::Oncology));
        assert!(!c.operates_in(TherapeuticArea::Cardiovascular));
    }

    #[test]
    fn company_product_count_tracks_vec_length() {
        let mut c = minimal_company();
        c.products.push(crate::Product {
            generic_name: "testdrug".to_string(),
            brand_names: vec![],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: None,
            safety_profile: SafetyProfile::default(),
        });
        assert_eq!(c.product_count(), 1);
    }

    #[test]
    fn company_pipeline_count_tracks_vec_length() {
        let mut c = minimal_company();
        c.pipeline.push(crate::PipelineCandidate {
            name: "XYZ-001".to_string(),
            mechanism: "mAb".to_string(),
            phase: Phase::Phase1,
            indication: "Solid tumours".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        });
        assert_eq!(c.pipeline_count(), 1);
    }

    #[test]
    fn company_safety_comm_count_tracks_vec_length() {
        let mut c = minimal_company();
        c.safety_communications.push(SafetyCommunication {
            title: "Notice".to_string(),
            date: "2024-01-01".to_string(),
            comm_type: CommType::SafetyUpdate,
            product: "testdrug".to_string(),
            summary: "Safety update summary.".to_string(),
        });
        assert_eq!(c.safety_comm_count(), 1);
    }

    #[test]
    fn company_serializes_round_trip() {
        let c = Company {
            id: CompanyId::new("roche"),
            name: "Roche".to_string(),
            ticker: Some("ROG".to_string()),
            headquarters: Some("Basel, Switzerland".to_string()),
            therapeutic_areas: vec![TherapeuticArea::Oncology, TherapeuticArea::Immunology],
            products: vec![crate::Product {
                generic_name: "bevacizumab".to_string(),
                brand_names: vec!["Avastin".to_string()],
                rxcui: Some("354891".to_string()),
                therapeutic_area: TherapeuticArea::Oncology,
                approval_year: Some(2004),
                safety_profile: SafetyProfile {
                    boxed_warning: true,
                    rems: false,
                    signals: vec![SignalSummary {
                        event: "hypertension".to_string(),
                        prr: 3.2,
                        ror: 4.1,
                        cases: 1250,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    }],
                    label_warnings: vec!["Gastrointestinal perforations".to_string()],
                },
            }],
            pipeline: vec![],
            safety_communications: vec![],
        };

        let json = serde_json::to_string(&c).expect("serialization cannot fail");
        let parsed: Company = serde_json::from_str(&json).expect("deserialization cannot fail");
        assert_eq!(parsed.id.as_str(), "roche");
        assert_eq!(parsed.products.len(), 1);
        assert!(parsed.products[0].safety_profile.boxed_warning);
    }
}
