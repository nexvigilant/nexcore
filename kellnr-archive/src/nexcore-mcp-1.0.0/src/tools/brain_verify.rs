//! Brain verification tools
//!
//! Tier: T3 (μ Mapping + ∂ Boundary)

use crate::params::BrainVerifyParams;
use antitransformer::pipeline::{self, AnalysisConfig};
use nexcore_brain::implicit::ImplicitKnowledge;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Verify the statistical naturalness of all patterns in the implicit brain.
pub fn brain_verify_engrams(_params: BrainVerifyParams) -> Result<CallToolResult, McpError> {
    let Ok(brain) = ImplicitKnowledge::load() else {
        return Err(McpError::invalid_params("Failed to load brain", None));
    };

    let config = AnalysisConfig {
        threshold: 0.55,
        window_size: 50,
    };

    let patterns = brain.list_patterns();
    let mut drifting_patterns = Vec::new();

    for pattern in patterns {
        let mut text = pattern.description.clone();
        for example in &pattern.examples {
            text.push_str(" ");
            text.push_str(example);
        }

        if text.len() < 100 {
            continue;
        }

        let result = pipeline::analyze(&text, &config);

        if result.verdict == "generated" {
            drifting_patterns.push(json!({
                "id": pattern.id,
                "type": pattern.pattern_type,
                "probability": result.probability,
                "confidence": result.confidence
            }));
        }
    }

    let response = json!({
        "status": "complete",
        "total_patterns": patterns.len(),
        "drifting_count": drifting_patterns.len(),
        "drifting_patterns": drifting_patterns,
        "note": if drifting_patterns.is_empty() {
            "All engrams show healthy human-balanced statistical profiles."
        } else {
            "Some engrams show low-entropy signatures indicative of model pollution."
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
