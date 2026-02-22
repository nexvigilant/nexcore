//! Causality Assessment Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! RUCAM and UCAS causality assessment input parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// =============================================================================
// RUCAM Parameters
// =============================================================================

/// Reaction type for RUCAM assessment
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum RucamReactionType {
    /// ALT-predominant liver injury
    Hepatocellular,
    /// ALP-predominant liver injury
    Cholestatic,
    /// Both ALT and ALP elevated
    Mixed,
}

/// Serology result for alternative cause investigation
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum SerologyResultParam {
    /// Test result was positive
    Positive,
    /// Test result was negative
    Negative,
    /// Test was not performed
    #[default]
    NotDone,
}

/// Yes/No/NotApplicable for alternative causes
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum YesNoNaParam {
    /// Condition is present
    Yes,
    /// Condition is absent
    No,
    /// Not applicable
    #[default]
    NotApplicable,
}

/// Rechallenge result after re-exposure
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum RechallengeResultParam {
    /// Reaction recurred on re-exposure
    Positive,
    /// No reaction on re-exposure
    Negative,
    /// Outcome was inconclusive
    NotConclusive,
}

/// Concomitant drug information for RUCAM
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ConcomitantDrugsParam {
    /// Count of known hepatotoxic concomitant drugs (default: 0)
    #[serde(default)]
    pub hepatotoxic_count: u32,
    /// Are there known drug-drug interactions? (default: false)
    #[serde(default)]
    pub interactions: bool,
}

/// Alternative causes investigation for RUCAM
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlternativeCausesParam {
    /// Hepatitis A serology result
    #[serde(default)]
    pub hepatitis_a: SerologyResultParam,
    /// Hepatitis B serology result
    #[serde(default)]
    pub hepatitis_b: SerologyResultParam,
    /// Hepatitis C serology result
    #[serde(default)]
    pub hepatitis_c: SerologyResultParam,
    /// CMV or EBV serology result
    #[serde(default)]
    pub cmv_ebv: SerologyResultParam,
    /// Biliary ultrasound/sonography findings
    #[serde(default)]
    pub biliary_sonography: SerologyResultParam,
    /// History of alcohol abuse
    #[serde(default)]
    pub alcoholism: YesNoNaParam,
    /// Pre-existing liver complications
    #[serde(default)]
    pub underlying_complications: YesNoNaParam,
}

/// Previous hepatotoxicity information for the suspected drug
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PreviousHepatotoxicityParam {
    /// Drug is labeled as hepatotoxic (default: false)
    #[serde(default)]
    pub labeled_hepatotoxic: bool,
    /// Published case reports exist (default: false)
    #[serde(default)]
    pub published_reports: bool,
    /// This specific reaction type is documented (default: false)
    #[serde(default)]
    pub reaction_known: bool,
}

/// Parameters for RUCAM hepatotoxicity causality assessment.
///
/// Scores range from -4 to +14 across 7 assessment areas:
/// temporal relationship, course of reaction, risk factors,
/// concomitant drugs, alternative causes, previous info, rechallenge.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RucamParams {
    /// Days from drug start to reaction onset
    pub time_to_onset: u32,
    /// Type of liver reaction pattern
    pub reaction_type: RucamReactionType,
    /// Was the drug withdrawn? (default: false)
    #[serde(default)]
    pub drug_withdrawn: bool,
    /// Days from withdrawal to improvement (omit if not withdrawn)
    pub time_to_improvement: Option<u32>,
    /// Percentage decrease in liver values after withdrawal (0-100)
    pub percentage_decrease: Option<f64>,
    /// Patient age in years
    pub age: u32,
    /// Alcohol use (default: false)
    #[serde(default)]
    pub alcohol: bool,
    /// Pregnancy (default: false)
    #[serde(default)]
    pub pregnancy: bool,
    /// Concomitant hepatotoxic drug information
    #[serde(default)]
    pub concomitant_drugs: ConcomitantDrugsParam,
    /// Alternative causes investigation results
    #[serde(default)]
    pub alternative_causes: AlternativeCausesParam,
    /// Previous hepatotoxicity information for the drug
    #[serde(default)]
    pub previous_hepatotoxicity: PreviousHepatotoxicityParam,
    /// Was rechallenge performed? (default: false)
    #[serde(default)]
    pub rechallenge_performed: bool,
    /// Result of rechallenge (omit if not performed)
    pub rechallenge_result: Option<RechallengeResultParam>,
}

// =============================================================================
// UCAS Parameters
// =============================================================================

/// Criterion response for UCAS (yes/no/unknown)
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum CriterionResponseParam {
    /// Criterion is met
    Yes,
    /// Criterion is not met
    No,
    /// Insufficient information
    #[default]
    Unknown,
}

/// Parameters for UCAS (Universal Causality Assessment Scale).
///
/// Domain-agnostic causality assessment with 8 criteria.
/// Scores range from -3 to +14. Integrates with ToV via sigmoid
/// recognition component R = sigmoid(score, mu=5, sigma=2).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UcasParams {
    /// 1. Did harm occur after exposure with plausible latency? (Yes=+2, No=-1, Unknown=0)
    #[serde(default)]
    pub temporal_relationship: CriterionResponseParam,
    /// 2. Did harm improve when intervention removed? (Yes=+2, Unknown/No=0)
    #[serde(default)]
    pub dechallenge: CriterionResponseParam,
    /// 3. Did harm recur when intervention reintroduced? (Yes=+3, Unknown/No=0)
    #[serde(default)]
    pub rechallenge: CriterionResponseParam,
    /// 4. Is there a known mechanism for this harm? (Yes=+2, Unknown/No=0)
    #[serde(default)]
    pub mechanistic_plausibility: CriterionResponseParam,
    /// 5. Are other plausible causes present? (Yes=-2, No=+1, Unknown=0)
    #[serde(default)]
    pub alternative_explanations: CriterionResponseParam,
    /// 6. Dose-response relationship? (Yes=+2, Unknown/No=0)
    #[serde(default)]
    pub dose_response: CriterionResponseParam,
    /// 7. Has this association been reported before? (Yes=+1, Unknown/No=0)
    #[serde(default)]
    pub prior_evidence: CriterionResponseParam,
    /// 8. Is this harm characteristic of this intervention class? (Yes=+1, Unknown/No=0)
    #[serde(default)]
    pub specificity: CriterionResponseParam,
}
