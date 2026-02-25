//! NIST Cybersecurity Framework (CSF) 2.0 Models.
//!
//! Data models for NIST CSF alignment including core functions, implementation
//! tiers, maturity levels, and healthcare-specific compliance mappings.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NIST CSF 2.0 Core Functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NistFunction {
    /// GV - Governance and oversight.
    Govern,
    /// ID - Asset management and risk assessment.
    Identify,
    /// PR - Safeguards and access control.
    Protect,
    /// DE - Continuous monitoring and detection.
    Detect,
    /// RS - Incident response and communication.
    Respond,
    /// RC - Recovery planning and improvements.
    Recover,
}

impl NistFunction {
    /// Get the function code (e.g., "GV", "ID").
    pub fn code(&self) -> &'static str {
        match self {
            Self::Govern => "GV",
            Self::Identify => "ID",
            Self::Protect => "PR",
            Self::Detect => "DE",
            Self::Respond => "RS",
            Self::Recover => "RC",
        }
    }

    /// Get the function description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Govern => "Governance and oversight of cybersecurity strategy",
            Self::Identify => "Asset management and risk assessment",
            Self::Protect => "Safeguards and access control implementation",
            Self::Detect => "Continuous monitoring and anomaly detection",
            Self::Respond => "Incident response and communication",
            Self::Recover => "Recovery planning and continuous improvement",
        }
    }
}

/// NIST CSF Implementation Tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImplementationTier {
    /// Tier 1: Partial - Ad-hoc, reactive.
    Partial,
    /// Tier 2: Risk Informed - Risk-aware but not organization-wide.
    #[default]
    RiskInformed,
    /// Tier 3: Repeatable - Formally approved and organization-wide.
    Repeatable,
    /// Tier 4: Adaptive - Continuously improving and adapting.
    Adaptive,
}

impl ImplementationTier {
    /// Get numeric tier level (1-4).
    pub fn level(&self) -> i32 {
        match self {
            Self::Partial => 1,
            Self::RiskInformed => 2,
            Self::Repeatable => 3,
            Self::Adaptive => 4,
        }
    }

    /// Create from implementation score percentage.
    pub fn from_score(score: f64) -> Self {
        if score >= 85.0 {
            Self::Adaptive
        } else if score >= 70.0 {
            Self::Repeatable
        } else if score >= 50.0 {
            Self::RiskInformed
        } else {
            Self::Partial
        }
    }
}

/// Control maturity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaturityLevel {
    /// 0% - Not implemented.
    NotImplemented,
    /// 25% - Initial/Ad-hoc.
    #[default]
    Initial,
    /// 50% - Developing/Documented.
    Developing,
    /// 75% - Defined/Managed.
    Defined,
    /// 100% - Optimized/Measured.
    Optimized,
}

impl MaturityLevel {
    /// Get implementation percentage.
    pub fn percentage(&self) -> f64 {
        match self {
            Self::NotImplemented => 0.0,
            Self::Initial => 25.0,
            Self::Developing => 50.0,
            Self::Defined => 75.0,
            Self::Optimized => 100.0,
        }
    }

    /// Create from implementation score percentage.
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            Self::Optimized
        } else if score >= 75.0 {
            Self::Defined
        } else if score >= 50.0 {
            Self::Developing
        } else if score >= 25.0 {
            Self::Initial
        } else {
            Self::NotImplemented
        }
    }
}

/// Risk assessment levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NistRiskLevel {
    VeryLow,
    Low,
    #[default]
    Moderate,
    High,
    VeryHigh,
}

impl NistRiskLevel {
    /// Get numeric risk weight (1-5).
    pub fn weight(&self) -> i32 {
        match self {
            Self::VeryLow => 1,
            Self::Low => 2,
            Self::Moderate => 3,
            Self::High => 4,
            Self::VeryHigh => 5,
        }
    }

    /// Calculate combined risk from likelihood and impact.
    pub fn from_likelihood_impact(likelihood: Self, impact: Self) -> Self {
        let combined = likelihood.weight() * impact.weight();
        if combined >= 20 {
            Self::VeryHigh
        } else if combined >= 12 {
            Self::High
        } else if combined >= 6 {
            Self::Moderate
        } else if combined >= 3 {
            Self::Low
        } else {
            Self::VeryLow
        }
    }
}

/// Healthcare compliance frameworks mapped to NIST CSF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NistComplianceFramework {
    Hipaa,
    Hitech,
    Fda21CfrPart11,
    Nist80066,
    Iso27001,
    Soc2,
    Hhs405d,
}

/// NIST CSF Subcategory definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistSubcategory {
    /// Subcategory ID (e.g., "GV.OC-01").
    pub subcategory_id: String,
    pub function: NistFunction,
    /// Category ID (e.g., "GV.OC").
    pub category: String,
    pub title: String,
    pub description: String,
    pub healthcare_guidance: String,
    /// Priority: high, medium, low.
    pub priority_level: String,
    #[serde(default)]
    pub applicable_frameworks: Vec<NistComplianceFramework>,
    #[serde(default)]
    pub example_controls: Vec<String>,
    #[serde(default)]
    pub measurement_criteria: Vec<String>,
}

impl NistSubcategory {
    /// Check if subcategory is high priority.
    pub fn is_high_priority(&self) -> bool {
        self.priority_level == "high"
    }

    /// Check if subcategory applies to a specific framework.
    pub fn applies_to(&self, framework: NistComplianceFramework) -> bool {
        self.applicable_frameworks.contains(&framework)
    }
}

/// Assessment of a specific NIST CSF control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlAssessment {
    pub subcategory_id: String,
    pub maturity_level: MaturityLevel,
    /// Implementation percentage (0-100).
    pub implementation_percentage: f64,
    #[serde(default)]
    pub evidence_provided: Vec<String>,
    #[serde(default)]
    pub gaps_identified: Vec<String>,
    #[serde(default)]
    pub recommendations: Vec<String>,
    pub risk_level: NistRiskLevel,
    #[serde(default = "DateTime::now")]
    pub last_assessed: DateTime,
    pub assessed_by: String,
    pub next_assessment_due: DateTime,
    #[serde(default)]
    pub related_controls: Vec<String>,
    /// Framework -> Control ID mapping.
    #[serde(default)]
    pub compliance_mappings: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl ControlAssessment {
    /// Check if control needs reassessment (> 90 days).
    pub fn needs_reassessment(&self) -> bool {
        DateTime::now() > self.next_assessment_due
    }

    /// Calculate gap percentage.
    pub fn gap_percentage(&self) -> f64 {
        100.0 - self.implementation_percentage
    }
}

/// Assessment of an entire NIST CSF function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionAssessment {
    pub function: NistFunction,
    pub overall_maturity: MaturityLevel,
    pub implementation_percentage: f64,
    #[serde(default)]
    pub subcategory_assessments: Vec<ControlAssessment>,
    #[serde(default)]
    pub key_strengths: Vec<String>,
    #[serde(default)]
    pub key_gaps: Vec<String>,
    #[serde(default)]
    pub priority_recommendations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_implementation_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_timeline_months: Option<i32>,
    pub assigned_owner: String,
    #[serde(default = "DateTime::now")]
    pub last_updated: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl FunctionAssessment {
    /// Calculate average implementation from subcategory assessments.
    pub fn calculate_implementation(&self) -> f64 {
        if self.subcategory_assessments.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .subcategory_assessments
            .iter()
            .map(|a| a.implementation_percentage)
            .sum();
        sum / self.subcategory_assessments.len() as f64
    }
}

/// NIST CSF Organizational Profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalProfile {
    pub profile_id: String,
    pub organization_name: String,
    /// Sector (e.g., "Healthcare and Public Health").
    pub sector: String,
    /// Size: small, medium, large, enterprise.
    pub organization_size: String,
    /// Risk tolerance: low, moderate, high.
    pub risk_tolerance: String,
    pub current_tier: ImplementationTier,
    pub target_tier: ImplementationTier,
    #[serde(default)]
    pub business_objectives: Vec<String>,
    #[serde(default)]
    pub regulatory_requirements: Vec<NistComplianceFramework>,
    #[serde(default)]
    pub critical_assets: Vec<String>,
    #[serde(default)]
    pub threat_landscape: Vec<String>,
    #[serde(default = "DateTime::now")]
    pub created_date: DateTime,
    #[serde(default = "DateTime::now")]
    pub last_updated: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl OrganizationalProfile {
    /// Calculate tier gap.
    pub fn tier_gap(&self) -> i32 {
        self.target_tier.level() - self.current_tier.level()
    }

    /// Check if organization has reached target tier.
    pub fn target_achieved(&self) -> bool {
        self.current_tier.level() >= self.target_tier.level()
    }
}

/// Gap analysis results and remediation plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    pub analysis_id: String,
    pub organization_profile_id: String,
    #[serde(default = "DateTime::now")]
    pub analysis_date: DateTime,
    /// Percentage gap to target.
    pub overall_maturity_gap: f64,
    /// Function -> gap percentage.
    #[serde(default)]
    pub function_gaps: HashMap<String, f64>,
    /// High-priority gaps.
    #[serde(default)]
    pub critical_gaps: Vec<String>,
    /// Low-effort, high-impact improvements.
    #[serde(default)]
    pub quick_wins: Vec<String>,
    /// Strategic improvements.
    #[serde(default)]
    pub long_term_initiatives: Vec<String>,
    pub estimated_total_cost: f64,
    pub estimated_timeline_months: i32,
    #[serde(default)]
    pub roi_analysis: HashMap<String, serde_json::Value>,
    /// Function -> risk reduction percentage.
    #[serde(default)]
    pub risk_reduction_impact: HashMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyst_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl GapAnalysis {
    /// Get the function with the largest gap.
    pub fn largest_gap_function(&self) -> Option<(&String, &f64)> {
        self.function_gaps
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Check if organization needs immediate attention (> 50% gap).
    pub fn needs_immediate_attention(&self) -> bool {
        self.overall_maturity_gap > 50.0
    }
}

/// NIST CSF-aligned risk assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistRiskAssessment {
    pub assessment_id: String,
    pub asset_category: String,
    pub threat_description: String,
    pub vulnerability_description: String,
    #[serde(default)]
    pub current_controls: Vec<String>,
    pub likelihood: NistRiskLevel,
    pub impact: NistRiskLevel,
    pub overall_risk: NistRiskLevel,
    pub residual_risk: NistRiskLevel,
    #[serde(default)]
    pub recommended_controls: Vec<String>,
    /// Related NIST subcategory IDs.
    #[serde(default)]
    pub nist_subcategories: Vec<String>,
    #[serde(default)]
    pub compliance_impact: Vec<NistComplianceFramework>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_of_mitigation: Option<f64>,
    /// Timeline to implement in months.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline_to_implement: Option<i32>,
    pub risk_owner: String,
    #[serde(default = "DateTime::now")]
    pub last_reviewed: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl NistRiskAssessment {
    /// Check if risk is acceptable (low or very low).
    pub fn is_acceptable(&self) -> bool {
        matches!(
            self.residual_risk,
            NistRiskLevel::Low | NistRiskLevel::VeryLow
        )
    }

    /// Check if risk requires immediate action.
    pub fn requires_immediate_action(&self) -> bool {
        matches!(
            self.overall_risk,
            NistRiskLevel::High | NistRiskLevel::VeryHigh
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_function_code() {
        assert_eq!(NistFunction::Govern.code(), "GV");
        assert_eq!(NistFunction::Identify.code(), "ID");
        assert_eq!(NistFunction::Protect.code(), "PR");
        assert_eq!(NistFunction::Detect.code(), "DE");
        assert_eq!(NistFunction::Respond.code(), "RS");
        assert_eq!(NistFunction::Recover.code(), "RC");
    }

    #[test]
    fn test_implementation_tier_from_score() {
        assert_eq!(
            ImplementationTier::from_score(90.0),
            ImplementationTier::Adaptive
        );
        assert_eq!(
            ImplementationTier::from_score(75.0),
            ImplementationTier::Repeatable
        );
        assert_eq!(
            ImplementationTier::from_score(55.0),
            ImplementationTier::RiskInformed
        );
        assert_eq!(
            ImplementationTier::from_score(30.0),
            ImplementationTier::Partial
        );
    }

    #[test]
    fn test_maturity_level_from_score() {
        assert_eq!(MaturityLevel::from_score(95.0), MaturityLevel::Optimized);
        assert_eq!(MaturityLevel::from_score(80.0), MaturityLevel::Defined);
        assert_eq!(MaturityLevel::from_score(60.0), MaturityLevel::Developing);
        assert_eq!(MaturityLevel::from_score(30.0), MaturityLevel::Initial);
        assert_eq!(
            MaturityLevel::from_score(10.0),
            MaturityLevel::NotImplemented
        );
    }

    #[test]
    fn test_risk_level_calculation() {
        let risk = NistRiskLevel::from_likelihood_impact(NistRiskLevel::High, NistRiskLevel::High);
        assert_eq!(risk, NistRiskLevel::High);

        let low_risk =
            NistRiskLevel::from_likelihood_impact(NistRiskLevel::Low, NistRiskLevel::Low);
        assert_eq!(low_risk, NistRiskLevel::Low);
    }

    #[test]
    fn test_organizational_profile_tier_gap() {
        let profile = OrganizationalProfile {
            profile_id: "prof-1".to_string(),
            organization_name: "Test Org".to_string(),
            sector: "Healthcare".to_string(),
            organization_size: "medium".to_string(),
            risk_tolerance: "low".to_string(),
            current_tier: ImplementationTier::RiskInformed,
            target_tier: ImplementationTier::Adaptive,
            business_objectives: vec![],
            regulatory_requirements: vec![NistComplianceFramework::Hipaa],
            critical_assets: vec![],
            threat_landscape: vec![],
            created_date: DateTime::now(),
            last_updated: DateTime::now(),
            tenant_id: None,
        };
        assert_eq!(profile.tier_gap(), 2); // 4 - 2
        assert!(!profile.target_achieved());
    }

    #[test]
    fn test_gap_analysis_attention() {
        let gap = GapAnalysis {
            analysis_id: "gap-1".to_string(),
            organization_profile_id: "prof-1".to_string(),
            analysis_date: DateTime::now(),
            overall_maturity_gap: 60.0,
            function_gaps: HashMap::new(),
            critical_gaps: vec!["Gap 1".to_string()],
            quick_wins: vec![],
            long_term_initiatives: vec![],
            estimated_total_cost: 100000.0,
            estimated_timeline_months: 12,
            roi_analysis: HashMap::new(),
            risk_reduction_impact: HashMap::new(),
            analyst_notes: None,
            tenant_id: None,
        };
        assert!(gap.needs_immediate_attention());
    }

    #[test]
    fn test_implementation_tier_default() {
        assert_eq!(
            ImplementationTier::default(),
            ImplementationTier::RiskInformed
        );
    }

    #[test]
    fn test_maturity_level_default() {
        assert_eq!(MaturityLevel::default(), MaturityLevel::Initial);
    }

    #[test]
    fn test_risk_level_default() {
        assert_eq!(NistRiskLevel::default(), NistRiskLevel::Moderate);
    }
}
