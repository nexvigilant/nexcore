//! Strategic analysis service with AI-powered capability gap detection.
//!
//! Uses the model orchestrator to analyze business challenges and identify capability gaps.

use crate::ai::ModelOrchestrator;
use nexcore_error::Result;
use serde::{Deserialize, Serialize};

/// A capability gap identified in strategic analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGap {
    /// Name of the capability
    pub capability_name: String,
    /// Category (e.g., "Technology", "Process", "People")
    pub capability_category: String,
    /// Detailed description
    pub description: String,
    /// Current maturity level (0-5)
    pub current_maturity: u8,
    /// Required maturity level (0-5)
    pub required_maturity: u8,
    /// Gap size (required - current)
    pub gap_size: u8,
    /// Priority (High, Medium, Low)
    pub priority: String,
    /// Business impact statement
    pub business_impact: String,
    /// Implementation complexity
    pub complexity: String,
    /// Estimated effort
    pub estimated_effort: String,
    /// Information needed to proceed
    pub information_needs: Vec<String>,
}

/// Result of strategic analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicAnalysis {
    /// Analysis title
    pub title: String,
    /// Chosen strategic approach
    pub chosen_manner_of_winning: String,
    /// Identified capability gaps
    pub capability_gaps: Vec<CapabilityGap>,
    /// Success metrics
    pub success_metrics: Vec<serde_json::Value>,
}

/// AI-powered strategic analyzer.
pub struct StrategicAnalyzer {
    orchestrator: ModelOrchestrator,
}

impl StrategicAnalyzer {
    /// Create a new strategic analyzer.
    #[must_use]
    pub fn new(orchestrator: ModelOrchestrator) -> Self {
        Self { orchestrator }
    }

    /// Analyze a business challenge and identify capability gaps.
    ///
    /// # Errors
    ///
    /// Returns error if AI generation or JSON parsing fails.
    pub async fn analyze(
        &self,
        challenge: &str,
        organization: Option<&str>,
        industry: Option<&str>,
    ) -> Result<StrategicAnalysis> {
        let org = organization.unwrap_or("unknown organization");
        let ind = industry.unwrap_or("unknown industry");

        let prompt = format!(
            r#"You are a strategic business analyst.
Organization: {org}
Industry: {ind}
Challenge: {challenge}

Perform a capability gap analysis and return a JSON object with:
- title: Analysis title
- chosen_manner_of_winning: Strategic approach
- capability_gaps: Array of gaps with capability_name, capability_category, description, current_maturity (0-5), required_maturity (0-5), gap_size, priority, business_impact, complexity, estimated_effort, information_needs
- success_metrics: Array of metric definitions

Return only valid JSON."#
        );

        let response = self.orchestrator.generate(&prompt).await?;
        let analysis: StrategicAnalysis = serde_json::from_str(&response)?;
        Ok(analysis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_gap_serialization() {
        let gap = CapabilityGap {
            capability_name: "Cloud Infrastructure".to_string(),
            capability_category: "Technology".to_string(),
            description: "Cloud deployment capabilities".to_string(),
            current_maturity: 2,
            required_maturity: 4,
            gap_size: 2,
            priority: "High".to_string(),
            business_impact: "Critical for scaling".to_string(),
            complexity: "Medium".to_string(),
            estimated_effort: "3 months".to_string(),
            information_needs: vec!["Current architecture".to_string()],
        };

        let json = serde_json::to_string(&gap).unwrap_or_default();
        assert!(json.contains("Cloud Infrastructure"));
    }
}
