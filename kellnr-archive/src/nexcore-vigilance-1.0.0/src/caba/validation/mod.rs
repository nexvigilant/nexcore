//! Prerequisite Validation System
//!
//! Migrated from Python `domains/regulatory/caba/caba/validation/prerequisite_validator.py`.
//!
//! ## UACA Hierarchy
//!
//! - **L1 Atoms**: Individual validation checks (<20 LOC)
//! - **L2 Molecules**: Composite validators (<50 LOC)
//!
//! ## Validation Components
//!
//! - Domain proficiency requirements
//! - Prerequisite EPA/CPA completions
//! - Certifications and assessments
//! - Innovation readiness for EPA10
//! - Quantitative metrics for senior levels

use crate::caba::Score;
use crate::caba::domain::{DomainCategory, DomainRequirement, RequirementType};
use crate::caba::proficiency::ProficiencyLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// L0 Quark - Validation weight constants.
pub mod weights {
    /// Domain requirement weight in EPA validation
    pub const EPA_DOMAIN_WEIGHT: f64 = 0.5;
    /// Prerequisite weight in EPA validation
    pub const EPA_PREREQUISITE_WEIGHT: f64 = 0.2;
    /// Certification weight in EPA validation
    pub const EPA_CERTIFICATION_WEIGHT: f64 = 0.1;
    /// Innovation weight in EPA validation
    pub const EPA_INNOVATION_WEIGHT: f64 = 0.2;

    /// Domain weight in CPA validation
    pub const CPA_DOMAIN_WEIGHT: f64 = 0.4;
    /// EPA weight in CPA validation
    pub const CPA_EPA_WEIGHT: f64 = 0.4;
    /// CPA8 special weight
    pub const CPA_CPA8_WEIGHT: f64 = 0.2;

    /// Minimum readiness score to be considered ready
    pub const READINESS_THRESHOLD: f64 = 0.8;
}

/// Result of prerequisite validation for EPA or CPA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteValidationResult {
    /// EPA or CPA ID being validated
    pub entity_id: String,
    /// Entity type: "EPA" or "CPA"
    pub entity_type: String,
    /// Whether professional is ready
    pub ready: bool,
    /// Overall readiness score [0.0, 1.0]
    pub readiness_score: Score,
    /// Domain requirements met status
    pub domain_requirements_met: HashMap<String, bool>,
    /// Prerequisite completions status
    pub prerequisite_completions: HashMap<String, bool>,
    /// Certifications met status
    pub certifications_met: HashMap<String, bool>,
    /// Blocking issues
    #[serde(default)]
    pub blockers: Vec<String>,
    /// Non-blocking warnings
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Recommendations for improvement
    #[serde(default)]
    pub recommendations: Vec<String>,
    /// Additional details
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

impl PrerequisiteValidationResult {
    /// Create a new validation result for an entity.
    #[must_use]
    pub fn new(entity_id: String, entity_type: &str) -> Self {
        Self {
            entity_id,
            entity_type: entity_type.to_string(),
            ready: true,
            readiness_score: Score::MAX,
            domain_requirements_met: HashMap::new(),
            prerequisite_completions: HashMap::new(),
            certifications_met: HashMap::new(),
            blockers: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            details: HashMap::new(),
        }
    }

    /// Add a blocker (makes ready = false).
    pub fn add_blocker(&mut self, blocker: String) {
        self.blockers.push(blocker);
        self.ready = false;
    }

    /// Add a warning (doesn't affect ready status).
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Record domain requirement status.
    pub fn record_domain_status(&mut self, domain: &str, met: bool) {
        // CLONE: HashMap requires owned key; domain_key computed once per call
        self.domain_requirements_met.insert(domain.to_string(), met);
    }

    /// Record prerequisite completion status.
    pub fn record_prerequisite_status(&mut self, epa_id: &str, met: bool) {
        // CLONE: HashMap requires owned key; epa_id passed as &str to avoid caller allocation
        self.prerequisite_completions
            .insert(epa_id.to_string(), met);
    }

    /// Record certification status.
    pub fn record_certification_status(&mut self, cert: &str, met: bool) {
        // CLONE: HashMap requires owned key; cert passed as &str to avoid caller allocation
        self.certifications_met.insert(cert.to_string(), met);
    }
}

/// Validates professional readiness for EPAs and CPAs.
///
/// # L2 Molecule - Composite validator
#[derive(Debug, Default)]
pub struct PrerequisiteValidator;

impl PrerequisiteValidator {
    /// Create a new validator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Validate domain proficiency requirements.
    ///
    /// # L1 Atom - Domain validation (<20 LOC)
    ///
    /// Returns score from 0.0 to 1.0.
    pub fn validate_domain_requirements(
        &self,
        required_domains: &[DomainRequirement],
        professional_domains: &HashMap<DomainCategory, ProficiencyLevel>,
        result: &mut PrerequisiteValidationResult,
    ) -> f64 {
        if required_domains.is_empty() {
            return 1.0;
        }

        let mut met_count = 0;
        for req in required_domains {
            let domain_key = req.domain.as_str();
            let current_level = professional_domains.get(&req.domain);

            let is_met = current_level.is_some_and(|level| req.is_met_by(*level));
            result.record_domain_status(domain_key, is_met);

            if is_met {
                met_count += 1;
            } else if req.requirement_type == RequirementType::Primary {
                let current = current_level.map_or("None".to_string(), |l| l.to_string());
                result.add_blocker(format!(
                    "Domain {}: requires {}, current level is {}",
                    req.domain, req.minimum_level, current
                ));
            } else {
                result.add_warning(format!(
                    "Supporting domain {} below recommended level",
                    req.domain
                ));
            }
        }

        met_count as f64 / required_domains.len() as f64
    }

    /// Validate prerequisite EPA completions.
    ///
    /// # L1 Atom - Prerequisite validation (<20 LOC)
    ///
    /// Returns score from 0.0 to 1.0.
    pub fn validate_prerequisite_epas(
        &self,
        required_epas: &[String],
        completed_epas: &[String],
        result: &mut PrerequisiteValidationResult,
    ) -> f64 {
        if required_epas.is_empty() {
            return 1.0;
        }

        let mut met_count = 0;
        for epa_id in required_epas {
            let is_completed = completed_epas.contains(epa_id);
            result.record_prerequisite_status(epa_id, is_completed);

            if is_completed {
                met_count += 1;
            } else {
                result.add_blocker(format!("Prerequisite EPA {} not completed", epa_id));
            }
        }

        met_count as f64 / required_epas.len() as f64
    }

    /// Validate required certifications.
    ///
    /// # L1 Atom - Certification validation (<20 LOC)
    ///
    /// Returns score from 0.0 to 1.0.
    pub fn validate_certifications(
        &self,
        required_certs: &[String],
        professional_certs: &[String],
        result: &mut PrerequisiteValidationResult,
    ) -> f64 {
        if required_certs.is_empty() {
            return 1.0;
        }

        let mut met_count = 0;
        for cert in required_certs {
            let has_cert = professional_certs.contains(cert);
            result.record_certification_status(cert, has_cert);

            if has_cert {
                met_count += 1;
            } else {
                result.add_blocker(format!("Required certification missing: {}", cert));
            }
        }

        met_count as f64 / required_certs.len() as f64
    }

    /// Generate recommendations based on validation results.
    ///
    /// # L1 Atom - Recommendation generation (<20 LOC)
    pub fn generate_recommendations(&self, result: &mut PrerequisiteValidationResult) {
        let unmet_domains: Vec<_> = result
            .domain_requirements_met
            .iter()
            .filter(|(_, met)| !**met)
            .map(|(d, _)| d.as_str())
            .take(3)
            .collect();

        if !unmet_domains.is_empty() {
            result.recommendations.push(format!(
                "Focus on developing proficiency in: {}",
                unmet_domains.join(", ")
            ));
        }

        let incomplete_epas: Vec<_> = result
            .prerequisite_completions
            .iter()
            .filter(|(_, met)| !**met)
            .map(|(e, _)| e.as_str())
            .take(3)
            .collect();

        if !incomplete_epas.is_empty() {
            result.recommendations.push(format!(
                "Complete prerequisite EPAs: {}",
                incomplete_epas.join(", ")
            ));
        }

        if result.readiness_score.value() < 0.5 {
            result.recommendations.push(
                "Readiness score is low - consider working with mentor to create development plan"
                    .to_string(),
            );
        } else if result.readiness_score.value() < weights::READINESS_THRESHOLD {
            result.recommendations.push(
                "Close to ready - address blockers above to meet entry requirements".to_string(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_creation() {
        let result = PrerequisiteValidationResult::new("EPA-1".to_string(), "EPA");
        assert!(result.ready);
        assert_eq!(result.entity_type, "EPA");
    }

    #[test]
    fn test_blocker_makes_not_ready() {
        let mut result = PrerequisiteValidationResult::new("EPA-1".to_string(), "EPA");
        assert!(result.ready);

        result.add_blocker("Missing prerequisite".to_string());
        assert!(!result.ready);
        assert_eq!(result.blockers.len(), 1);
    }

    #[test]
    fn test_validate_prerequisites_empty() {
        let validator = PrerequisiteValidator::new();
        let mut result = PrerequisiteValidationResult::new("EPA-1".to_string(), "EPA");

        let score = validator.validate_prerequisite_epas(&[], &[], &mut result);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_validate_prerequisites_met() {
        let validator = PrerequisiteValidator::new();
        let mut result = PrerequisiteValidationResult::new("EPA-5".to_string(), "EPA");

        let required = vec!["EPA-1".to_string(), "EPA-2".to_string()];
        let completed = vec![
            "EPA-1".to_string(),
            "EPA-2".to_string(),
            "EPA-3".to_string(),
        ];

        let score = validator.validate_prerequisite_epas(&required, &completed, &mut result);
        assert!((score - 1.0).abs() < f64::EPSILON);
        assert!(result.blockers.is_empty());
    }

    #[test]
    fn test_validate_prerequisites_partial() {
        let validator = PrerequisiteValidator::new();
        let mut result = PrerequisiteValidationResult::new("EPA-5".to_string(), "EPA");

        let required = vec!["EPA-1".to_string(), "EPA-2".to_string()];
        let completed = vec!["EPA-1".to_string()];

        let score = validator.validate_prerequisite_epas(&required, &completed, &mut result);
        assert!((score - 0.5).abs() < f64::EPSILON);
        assert_eq!(result.blockers.len(), 1);
    }

    #[test]
    fn test_validate_domain_requirements_met() {
        use crate::caba::domain::{DomainCategory, DomainRequirement};
        use crate::caba::proficiency::ProficiencyLevel;

        let validator = PrerequisiteValidator::new();
        let mut result = PrerequisiteValidationResult::new("EPA-3".to_string(), "EPA");

        let required = vec![DomainRequirement::primary(
            DomainCategory::D1ProfessionalFoundations,
            ProficiencyLevel::L3Competent,
        )];
        let mut professional = HashMap::new();
        professional.insert(
            DomainCategory::D1ProfessionalFoundations,
            ProficiencyLevel::L4Proficient,
        );

        let score = validator.validate_domain_requirements(&required, &professional, &mut result);
        assert!((score - 1.0).abs() < f64::EPSILON);
        assert!(result.blockers.is_empty());
    }

    #[test]
    fn test_validate_domain_requirements_not_met() {
        use crate::caba::domain::{DomainCategory, DomainRequirement};
        use crate::caba::proficiency::ProficiencyLevel;

        let validator = PrerequisiteValidator::new();
        let mut result = PrerequisiteValidationResult::new("EPA-3".to_string(), "EPA");

        let required = vec![DomainRequirement::primary(
            DomainCategory::D1ProfessionalFoundations,
            ProficiencyLevel::L4Proficient,
        )];
        let mut professional = HashMap::new();
        professional.insert(
            DomainCategory::D1ProfessionalFoundations,
            ProficiencyLevel::L2AdvancedBeginner,
        );

        let score = validator.validate_domain_requirements(&required, &professional, &mut result);
        assert!((score - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.blockers.len(), 1);
    }
}
