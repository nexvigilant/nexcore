//! Context of Use (COU) - Core type definitions
//!
//! ## T1 Grounding
//!
//! - **ContextOfUse**: λ (Location) + μ (Mapping)
//!   - Location: Where/how the model is used in decision process
//!   - Mapping: What is modeled → What is produced
//!
//! - **DecisionQuestion**: ∃ (Existence)
//!   - The specific question requiring an answer
//!
//! - **ModelPurpose**: μ (Mapping)
//!   - Input domain → Output domain transformation

use serde::{Deserialize, Serialize};
use std::fmt;

/// Step 1: The specific regulatory question requiring an answer
///
/// T1 Grounding: ∃ (Existence) — What decision exists to be made?
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionQuestion(String);

impl DecisionQuestion {
    /// Creates a new decision question
    ///
    /// ## Validation
    ///
    /// - Must be non-empty
    /// - Should end with '?'
    pub fn new(question: impl Into<String>) -> Result<Self, ValidationError> {
        let s = question.into();
        if s.trim().is_empty() {
            return Err(ValidationError::EmptyQuestion);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DecisionQuestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// What the AI model transforms (input domain → output domain)
///
/// T1 Grounding: μ (Mapping)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPurpose {
    /// Input domain (what is measured/observed)
    input_domain: String,
    /// Output domain (what is predicted/classified)
    output_domain: String,
    /// Transformation description
    description: String,
}

impl ModelPurpose {
    pub fn new(
        input_domain: impl Into<String>,
        output_domain: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let input = input_domain.into();
        let output = output_domain.into();
        let desc = description.into();

        if input.trim().is_empty() || output.trim().is_empty() {
            return Err(ValidationError::EmptyDomain);
        }

        Ok(Self {
            input_domain: input,
            output_domain: output,
            description: desc,
        })
    }

    pub fn input_domain(&self) -> &str {
        &self.input_domain
    }

    pub fn output_domain(&self) -> &str {
        &self.output_domain
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// How AI model outputs are integrated with other evidence sources
///
/// T1 Grounding: σ (Sequence) — Multiple evidence sources in order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceIntegration {
    /// AI is sole determinant (highest risk)
    Sole,
    /// AI is primary, other sources confirmatory
    Primary { confirmatory: Vec<String> },
    /// AI is one of many equal sources
    Contributory { other_sources: Vec<String> },
    /// AI is supplementary to primary evidence
    Supplementary { primary_source: String },
}

impl EvidenceIntegration {
    /// Returns the number of non-AI evidence sources
    ///
    /// T1 Grounding: N (Quantity)
    pub fn evidence_count(&self) -> usize {
        match self {
            Self::Sole => 0,
            Self::Primary { confirmatory } => confirmatory.len(),
            Self::Contributory { other_sources } => other_sources.len(),
            Self::Supplementary { .. } => 1,
        }
    }

    /// Returns true if AI is the primary decision driver
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Sole | Self::Primary { .. })
    }
}

/// Step 2: Specific role and scope of AI model for question of interest
///
/// T1 Grounding: λ (Location) + μ (Mapping) + σ (Sequence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextOfUse {
    /// The decision question being addressed
    question: DecisionQuestion,
    /// What the model does (input → output mapping)
    purpose: ModelPurpose,
    /// How outputs integrate with other evidence
    integration: EvidenceIntegration,
    /// Regulatory context (IND, NDA, BLA, etc.)
    regulatory_context: RegulatoryContext,
}

impl ContextOfUse {
    pub fn new(
        question: DecisionQuestion,
        purpose: ModelPurpose,
        integration: EvidenceIntegration,
        regulatory_context: RegulatoryContext,
    ) -> Self {
        Self {
            question,
            purpose,
            integration,
            regulatory_context,
        }
    }

    pub fn question(&self) -> &DecisionQuestion {
        &self.question
    }

    pub fn purpose(&self) -> &ModelPurpose {
        &self.purpose
    }

    pub fn integration(&self) -> &EvidenceIntegration {
        &self.integration
    }

    pub fn regulatory_context(&self) -> &RegulatoryContext {
        &self.regulatory_context
    }

    /// Validates that COU is well-formed for regulatory submission
    ///
    /// T1 Grounding: κ (Comparison) — Check against requirements
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Question must be non-empty (checked at construction)
        // Purpose domains must be distinct
        if self.purpose.input_domain() == self.purpose.output_domain() {
            return Err(ValidationError::IdenticalDomains);
        }
        Ok(())
    }
}

/// Regulatory submission context
///
/// T1 Grounding: λ (Location) — Where in drug lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegulatoryContext {
    /// Investigational New Drug application
    Ind,
    /// New Drug Application
    Nda,
    /// Biologics License Application
    Bla,
    /// Postmarketing surveillance
    Postmarket,
    /// Manufacturing
    Manufacturing,
    /// Other (with code)
    Other,
}

impl fmt::Display for RegulatoryContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ind => write!(f, "IND"),
            Self::Nda => write!(f, "NDA"),
            Self::Bla => write!(f, "BLA"),
            Self::Postmarket => write!(f, "Postmarket"),
            Self::Manufacturing => write!(f, "Manufacturing"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Validation errors for FDA types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyQuestion,
    EmptyDomain,
    IdenticalDomains,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuestion => write!(f, "Decision question cannot be empty"),
            Self::EmptyDomain => write!(f, "Domain cannot be empty"),
            Self::IdenticalDomains => write!(f, "Input and output domains must differ"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_question_valid() {
        let q = DecisionQuestion::new("Is drug X safe for population Y?");
        assert!(q.is_ok());
    }

    #[test]
    fn test_decision_question_empty() {
        let q = DecisionQuestion::new("");
        assert!(matches!(q, Err(ValidationError::EmptyQuestion)));
    }

    #[test]
    fn test_model_purpose_mapping() {
        let purpose = ModelPurpose::new(
            "Patient demographics + adverse events",
            "Signal strength score",
            "PRR calculation",
        )
        .ok()
        .unwrap_or_else(|| {
            panic!("Should succeed");
        });
        assert_eq!(
            purpose.input_domain(),
            "Patient demographics + adverse events"
        );
        assert_eq!(purpose.output_domain(), "Signal strength score");
    }

    #[test]
    fn test_evidence_integration_sole() {
        let ei = EvidenceIntegration::Sole;
        assert_eq!(ei.evidence_count(), 0);
        assert!(ei.is_primary());
    }

    #[test]
    fn test_evidence_integration_contributory() {
        let ei = EvidenceIntegration::Contributory {
            other_sources: vec!["Clinical trials".into(), "Literature".into()],
        };
        assert_eq!(ei.evidence_count(), 2);
        assert!(!ei.is_primary());
    }

    #[test]
    fn test_context_of_use_validation() {
        let question = DecisionQuestion::new("Should we approve this drug?")
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        let purpose = ModelPurpose::new("AE reports", "AE reports", "Identity")
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        let cou = ContextOfUse::new(
            question,
            purpose,
            EvidenceIntegration::Sole,
            RegulatoryContext::Nda,
        );

        let result = cou.validate();
        assert!(matches!(result, Err(ValidationError::IdenticalDomains)));
    }

    #[test]
    fn test_regulatory_context_display() {
        assert_eq!(RegulatoryContext::Ind.to_string(), "IND");
        assert_eq!(RegulatoryContext::Postmarket.to_string(), "Postmarket");
    }
}
