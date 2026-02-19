//! Lex Primitiva Synthesis MCP tools — Level 5 Evolution.
//!
//! Exposes nexcore-synth functionality as MCP tools.

use crate::params::LexPrimitivaSynthParams;
use nexcore_synth::SynthEngine;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Run the evolution loop to synthesize a new primitive candidate.
pub fn lex_primitiva_synth(params: LexPrimitivaSynthParams) -> Result<CallToolResult, McpError> {
    let engine = SynthEngine::new();
    
    let candidate = engine.evolve(&params.description, &params.sample_data)
        .map_err(|e| McpError::internal_error(format!("Evolution failed: {e}"), None))?;

    let result = json!({
        "status": "evolved",
        "candidate_id": candidate.id,
        "name": candidate.name,
        "tier": format!("{:?}", candidate.tier),
        "composition": candidate.composition,
        "dominant_primitive": candidate.dominant_primitive,
        "confidence": candidate.confidence,
        "derivation_path": candidate.derivation_path,
        "recommendation": "Candidate added to pending evolution queue. Use lex_primitiva_promote to finalize."
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
