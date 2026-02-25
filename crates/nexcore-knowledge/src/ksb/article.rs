//! KSB Article types and parsing.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// KSB Domain enumeration (D01-D15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KsbDomain {
    /// D01 - Foundations of Pharmacovigilance
    D01Foundations,
    /// D02 - Regulatory Framework
    D02Regulatory,
    /// D03 - Data Collection and Management
    D03DataCollection,
    /// D04 - Signal Detection and Evaluation
    D04SignalDetection,
    /// D05 - Risk Management
    D05RiskManagement,
    /// D06 - Clinical Trials Safety
    D06ClinicalTrials,
    /// D07 - Benefit-Risk Assessment
    D07BenefitRisk,
    /// D08 - Communication and Reporting
    D08Communication,
    /// D09 - Quality Systems
    D09Quality,
    /// D10 - Technology and Innovation
    D10Technology,
    /// D11 - Specialized Populations
    D11Specialized,
    /// D12 - Global Pharmacovigilance
    D12Global,
    /// D13 - Emerging Topics
    D13Emerging,
    /// D14 - Leadership and Management
    D14Leadership,
    /// D15 - Integration and Application
    D15Integration,
}

impl KsbDomain {
    /// Parse domain from string (e.g., "D01", "d01").
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "D01" => Some(Self::D01Foundations),
            "D02" => Some(Self::D02Regulatory),
            "D03" => Some(Self::D03DataCollection),
            "D04" => Some(Self::D04SignalDetection),
            "D05" => Some(Self::D05RiskManagement),
            "D06" => Some(Self::D06ClinicalTrials),
            "D07" => Some(Self::D07BenefitRisk),
            "D08" => Some(Self::D08Communication),
            "D09" => Some(Self::D09Quality),
            "D10" => Some(Self::D10Technology),
            "D11" => Some(Self::D11Specialized),
            "D12" => Some(Self::D12Global),
            "D13" => Some(Self::D13Emerging),
            "D14" => Some(Self::D14Leadership),
            "D15" => Some(Self::D15Integration),
            _ => None,
        }
    }

    /// Get domain code (e.g., "D01").
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::D01Foundations => "D01",
            Self::D02Regulatory => "D02",
            Self::D03DataCollection => "D03",
            Self::D04SignalDetection => "D04",
            Self::D05RiskManagement => "D05",
            Self::D06ClinicalTrials => "D06",
            Self::D07BenefitRisk => "D07",
            Self::D08Communication => "D08",
            Self::D09Quality => "D09",
            Self::D10Technology => "D10",
            Self::D11Specialized => "D11",
            Self::D12Global => "D12",
            Self::D13Emerging => "D13",
            Self::D14Leadership => "D14",
            Self::D15Integration => "D15",
        }
    }

    /// Get domain name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::D01Foundations => "Foundations of Pharmacovigilance",
            Self::D02Regulatory => "Regulatory Framework",
            Self::D03DataCollection => "Data Collection and Management",
            Self::D04SignalDetection => "Signal Detection and Evaluation",
            Self::D05RiskManagement => "Risk Management",
            Self::D06ClinicalTrials => "Clinical Trials Safety",
            Self::D07BenefitRisk => "Benefit-Risk Assessment",
            Self::D08Communication => "Communication and Reporting",
            Self::D09Quality => "Quality Systems",
            Self::D10Technology => "Technology and Innovation",
            Self::D11Specialized => "Specialized Populations",
            Self::D12Global => "Global Pharmacovigilance",
            Self::D13Emerging => "Emerging Topics",
            Self::D14Leadership => "Leadership and Management",
            Self::D15Integration => "Integration and Application",
        }
    }

    /// Get all domains.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::D01Foundations,
            Self::D02Regulatory,
            Self::D03DataCollection,
            Self::D04SignalDetection,
            Self::D05RiskManagement,
            Self::D06ClinicalTrials,
            Self::D07BenefitRisk,
            Self::D08Communication,
            Self::D09Quality,
            Self::D10Technology,
            Self::D11Specialized,
            Self::D12Global,
            Self::D13Emerging,
            Self::D14Leadership,
            Self::D15Integration,
        ]
    }
}

/// Proficiency level (L1-L5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    /// L1 - Novice (direct supervision required)
    L1Novice,
    /// L2 - Advanced Beginner (some independence)
    L2AdvancedBeginner,
    /// L3 - Competent (works independently)
    L3Competent,
    /// L4 - Proficient (handles complexity)
    L4Proficient,
    /// L5 - Expert (teaches others)
    L5Expert,
}

impl ProficiencyLevel {
    /// Parse from string (e.g., "L1", "l1").
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "L1" => Some(Self::L1Novice),
            "L2" => Some(Self::L2AdvancedBeginner),
            "L3" => Some(Self::L3Competent),
            "L4" => Some(Self::L4Proficient),
            "L5" => Some(Self::L5Expert),
            _ => None,
        }
    }
}

/// KSB Article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbArticle {
    /// Article ID (e.g., "KSB-D01-K0001").
    pub id: String,
    /// Domain.
    pub domain: KsbDomain,
    /// Article title.
    pub title: String,
    /// Description/summary.
    pub description: String,
    /// Full content (markdown).
    pub content: String,
    /// Proficiency level.
    pub proficiency_level: Option<ProficiencyLevel>,
    /// Bloom's taxonomy level.
    pub bloom_level: Option<String>,
    /// EPA mappings.
    pub epa_mapping: Option<String>,
    /// CPA mappings.
    pub cpa_mapping: Option<String>,
    /// Regulatory references.
    pub regulatory_refs: Option<String>,
    /// Trigger phrases.
    pub triggers: Vec<String>,
    /// File path.
    pub path: String,
}

impl KsbArticle {
    /// Parse article from SKILL.md file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn from_path(path: &Path) -> Result<Self, crate::KnowledgeError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::KnowledgeError::ReadError(e.to_string()))?;

        Self::parse(&content, path.to_string_lossy().to_string())
    }

    /// Parse article from content string.
    ///
    /// # Errors
    ///
    /// Returns error if content cannot be parsed.
    pub fn parse(content: &str, path: String) -> Result<Self, crate::KnowledgeError> {
        // Extract frontmatter
        if !content.starts_with("---") {
            return Err(crate::KnowledgeError::ParseError(
                "Missing frontmatter".to_string(),
            ));
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(crate::KnowledgeError::ParseError(
                "Invalid frontmatter format".to_string(),
            ));
        }

        let frontmatter = parts[1].trim();
        let body = parts[2].trim();

        // Parse YAML frontmatter
        let yaml: serde_yml::Value = serde_yml::from_str(frontmatter)
            .map_err(|e| crate::KnowledgeError::ParseError(e.to_string()))?;

        let name = yaml["name"].as_str().unwrap_or("").to_string();

        let description = yaml["description"].as_str().unwrap_or("").to_string();

        let domain_str = yaml["domain"].as_str().unwrap_or("D01");

        let domain = KsbDomain::from_str(domain_str).unwrap_or(KsbDomain::D01Foundations);

        let proficiency_level = yaml["proficiency_level"]
            .as_str()
            .and_then(ProficiencyLevel::from_str);

        let bloom_level = yaml["bloom_level"].as_str().map(String::from);

        let epa_mapping = yaml["epa_mapping"].as_str().map(String::from);

        let cpa_mapping = yaml["cpa_mapping"].as_str().map(String::from);

        let regulatory_refs = yaml["regulatory_refs"].as_str().map(String::from);

        let triggers: Vec<String> = yaml["triggers"]
            .as_sequence()
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Extract title from body (first # heading)
        let title = body
            .lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").to_string())
            .unwrap_or_else(|| name.clone());

        Ok(Self {
            id: name,
            domain,
            title,
            description,
            content: body.to_string(),
            proficiency_level,
            bloom_level,
            epa_mapping,
            cpa_mapping,
            regulatory_refs,
            triggers,
            path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_parsing() {
        assert_eq!(KsbDomain::from_str("D01"), Some(KsbDomain::D01Foundations));
        assert_eq!(
            KsbDomain::from_str("d04"),
            Some(KsbDomain::D04SignalDetection)
        );
        assert_eq!(KsbDomain::from_str("invalid"), None);
    }

    #[test]
    fn test_proficiency_parsing() {
        assert_eq!(
            ProficiencyLevel::from_str("L1"),
            Some(ProficiencyLevel::L1Novice)
        );
        assert_eq!(
            ProficiencyLevel::from_str("l5"),
            Some(ProficiencyLevel::L5Expert)
        );
    }
}
