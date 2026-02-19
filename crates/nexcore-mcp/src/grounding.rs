//! Grounding for nexcore-mcp
//!
//! T1 Lex Primitiva grounding for MCP server and core types.

use crate::NexCoreMcpServer;
use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

impl GroundsTo for NexCoreMcpServer {
    fn primitive_composition() -> PrimitiveComposition {
        // MCP Server: State (SkillRegistry) × Mapping (ToolRouter) × Persistence (Config)
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for crate::composites::UnifiedCommand {
    fn primitive_composition() -> PrimitiveComposition {
        // UnifiedCommand: Mapping (command) × Product (params)
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

impl GroundsTo for crate::composites::McpServerStatus {
    fn primitive_composition() -> PrimitiveComposition {
        // McpServerStatus: State (connected) × Quantity (tools)
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

impl GroundsTo for crate::composites::AgentLock {
    fn primitive_composition() -> PrimitiveComposition {
        // AgentLock: Boundary (expiry) × Location (path) × State (agent_id)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Location,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}
