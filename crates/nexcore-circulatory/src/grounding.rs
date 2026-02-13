//! # GroundsTo implementations for nexcore-circulatory types
//!
//! Connects the circulatory system model to the Lex Primitiva type system.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::claude_code::{
    BloodPayload, CirculatoryHealth, FlowDirection, FrankStarling, McpHeartbeat, McpScope,
    McpServer, McpTransport, PortalFiltration, SelectivePerfusion, ToolCall, ToolResult,
};
use crate::{
    BloodCell, BloodPressure, CellKind, CirculatoryError, Destination, Enriched, Platelet, Pulse,
    RouteDecision,
};

// ---------------------------------------------------------------------------
// CellKind -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for CellKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- 3-variant cell classification
            LexPrimitiva::Comparison, // kappa -- kind drives priority/routing
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// BloodCell -- Mapping dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for BloodCell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- carries data from source to destination
            LexPrimitiva::Product,  // times -- composite: source + kind + payload + priority
            LexPrimitiva::Sequence, // sigma -- ordered in bloodstream
            LexPrimitiva::State,    // varsigma -- processed flag, mutable carrier
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.65)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Enriched -- Causality dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for Enriched {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- raw cell -> enriched with metadata
            LexPrimitiva::Product,   // times -- adds timestamp + priority
            LexPrimitiva::Quantity,  // N -- priority score
        ])
        .with_dominant(LexPrimitiva::Causality, 0.75)
    }
}

// ---------------------------------------------------------------------------
// Destination -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for Destination {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- variant-based routing target
            LexPrimitiva::Location, // lambda -- where data goes
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

// ---------------------------------------------------------------------------
// RouteDecision -- Causality dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for RouteDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- cell kind -> destination
            LexPrimitiva::Mapping,   // mu -- routing function
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// BloodPressure -- Quantity dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for BloodPressure {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric ratio (available/total)
            LexPrimitiva::Boundary,   // partial -- healthy range constraints
            LexPrimitiva::Comparison, // kappa -- above/below threshold
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.70)
    }
}

// ---------------------------------------------------------------------------
// Pulse -- Sequence dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for Pulse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- ordered heartbeat cycle
            LexPrimitiva::Frequency, // nu -- rhythmic repetition
            LexPrimitiva::Quantity,  // N -- cells pumped count
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

// ---------------------------------------------------------------------------
// Platelet -- Boundary dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for Platelet {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // partial -- seals breaches
            LexPrimitiva::Causality, // arrow -- wound -> repair
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// CirculatoryError -- Boundary dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for CirculatoryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- circulation failures
            LexPrimitiva::Sum,      // Sigma -- 3-variant error
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ===========================================================================
// Claude Code MCP Transport Types
// ===========================================================================

// ---------------------------------------------------------------------------
// McpTransport -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for McpTransport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- 2-variant transport (stdio/http)
            LexPrimitiva::Location, // lambda -- where data flows (local/remote)
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// McpScope -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for McpScope {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- 3-variant scope
            LexPrimitiva::Boundary, // partial -- scope boundaries (local/project/user)
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// FlowDirection -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------
impl GroundsTo for FlowDirection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Sigma -- 2-variant direction
            LexPrimitiva::Causality, // arrow -- arterial (out) vs venous (return)
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// McpServer -- Existence dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for McpServer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- server registered or not
            LexPrimitiva::Mapping,   // mu -- name -> tools mapping
            LexPrimitiva::Boundary,  // partial -- scope boundary
        ])
        .with_dominant(LexPrimitiva::Existence, 0.65)
    }
}

// ---------------------------------------------------------------------------
// McpHeartbeat -- Frequency dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for McpHeartbeat {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // nu -- rhythmic health check cycle
            LexPrimitiva::Quantity,   // N -- latency measurement
            LexPrimitiva::Comparison, // kappa -- responsive vs unresponsive
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.70)
    }
}

// ---------------------------------------------------------------------------
// ToolCall -- Causality dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for ToolCall {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- agent invokes tool
            LexPrimitiva::Sequence,  // sigma -- ordered call sequence
            LexPrimitiva::Mapping,   // mu -- params -> server mapping
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

// ---------------------------------------------------------------------------
// ToolResult -- Causality dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for ToolResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- tool -> result
            LexPrimitiva::Sequence,  // sigma -- ordered result sequence
            LexPrimitiva::Quantity,  // N -- result size in tokens
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

// ---------------------------------------------------------------------------
// PortalFiltration -- Boundary dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for PortalFiltration {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- max token threshold
            LexPrimitiva::Quantity, // N -- token counts
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// SelectivePerfusion -- Quantity dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for SelectivePerfusion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- tool counts
            LexPrimitiva::Boundary,   // partial -- active vs deferred threshold
            LexPrimitiva::Comparison, // kappa -- perfusion ratio
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.70)
    }
}

// ---------------------------------------------------------------------------
// BloodPayload -- Product dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for BloodPayload {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,  // times -- composite of red/white/platelets/plasma
            LexPrimitiva::Quantity, // N -- cell counts
        ])
        .with_dominant(LexPrimitiva::Product, 0.80)
    }
}

// ---------------------------------------------------------------------------
// FrankStarling -- Quantity dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for FrankStarling {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- timeout measurements
            LexPrimitiva::Frequency, // nu -- load-dependent adjustment
            LexPrimitiva::Boundary,  // partial -- min/max timeout bounds
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.70)
    }
}

// ---------------------------------------------------------------------------
// CirculatoryHealth -- State dominant (T2-C)
// ---------------------------------------------------------------------------
impl GroundsTo for CirculatoryHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- health state assessment
            LexPrimitiva::Comparison, // kappa -- healthy vs unhealthy
            LexPrimitiva::Boundary,   // partial -- threshold conditions
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn cell_kind_is_t2p_sum() {
        let comp = CellKind::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(CellKind::tier(), Tier::T2Primitive);
    }

    #[test]
    fn blood_cell_is_t2c_mapping() {
        let comp = BloodCell::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(BloodCell::tier(), Tier::T2Composite);
    }

    #[test]
    fn enriched_is_causality() {
        assert_eq!(
            Enriched::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn destination_is_sum_with_location() {
        let comp = Destination::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Location));
    }

    #[test]
    fn blood_pressure_is_quantity() {
        assert_eq!(
            BloodPressure::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn pulse_has_frequency() {
        let comp = Pulse::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Frequency));
    }

    #[test]
    fn platelet_is_boundary() {
        assert_eq!(Platelet::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(CellKind::dominant_primitive().is_some());
        assert!(BloodCell::dominant_primitive().is_some());
        assert!(Enriched::dominant_primitive().is_some());
        assert!(Destination::dominant_primitive().is_some());
        assert!(RouteDecision::dominant_primitive().is_some());
        assert!(BloodPressure::dominant_primitive().is_some());
        assert!(Pulse::dominant_primitive().is_some());
        assert!(Platelet::dominant_primitive().is_some());
        assert!(CirculatoryError::dominant_primitive().is_some());
    }

    // -----------------------------------------------------------------------
    // Claude Code MCP Transport Types Tests
    // -----------------------------------------------------------------------

    #[test]
    fn mcp_transport_is_t2p_sum() {
        let comp = McpTransport::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.unique().contains(&LexPrimitiva::Location));
        assert_eq!(McpTransport::tier(), Tier::T2Primitive);
    }

    #[test]
    fn mcp_scope_is_sum_with_boundary() {
        let comp = McpScope::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        assert_eq!(McpScope::tier(), Tier::T2Primitive);
    }

    #[test]
    fn flow_direction_is_sum() {
        assert_eq!(FlowDirection::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(FlowDirection::tier(), Tier::T2Primitive);
    }

    #[test]
    fn mcp_server_is_t2p_existence() {
        let comp = McpServer::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert!(comp.unique().contains(&LexPrimitiva::Mapping));
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        assert_eq!(McpServer::tier(), Tier::T2Primitive);
    }

    #[test]
    fn mcp_heartbeat_is_frequency() {
        let comp = McpHeartbeat::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!(comp.unique().contains(&LexPrimitiva::Quantity));
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
        assert_eq!(McpHeartbeat::tier(), Tier::T2Primitive);
    }

    #[test]
    fn tool_call_is_causality() {
        assert_eq!(
            ToolCall::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(ToolCall::tier(), Tier::T2Primitive);
    }

    #[test]
    fn tool_result_is_causality() {
        let comp = ToolResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.unique().contains(&LexPrimitiva::Sequence));
        assert!(comp.unique().contains(&LexPrimitiva::Quantity));
        assert_eq!(ToolResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn portal_filtration_is_boundary() {
        assert_eq!(
            PortalFiltration::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(PortalFiltration::tier(), Tier::T2Primitive);
    }

    #[test]
    fn selective_perfusion_is_quantity() {
        let comp = SelectivePerfusion::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
        assert_eq!(SelectivePerfusion::tier(), Tier::T2Primitive);
    }

    #[test]
    fn blood_payload_is_product() {
        assert_eq!(
            BloodPayload::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
        assert_eq!(BloodPayload::tier(), Tier::T2Primitive);
    }

    #[test]
    fn frank_starling_is_quantity() {
        let comp = FrankStarling::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.unique().contains(&LexPrimitiva::Frequency));
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        assert_eq!(FrankStarling::tier(), Tier::T2Primitive);
    }

    #[test]
    fn circulatory_health_is_state() {
        let comp = CirculatoryHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        assert_eq!(CirculatoryHealth::tier(), Tier::T2Primitive);
    }

    #[test]
    fn all_claude_code_types_have_dominant() {
        assert!(McpTransport::dominant_primitive().is_some());
        assert!(McpScope::dominant_primitive().is_some());
        assert!(FlowDirection::dominant_primitive().is_some());
        assert!(McpServer::dominant_primitive().is_some());
        assert!(McpHeartbeat::dominant_primitive().is_some());
        assert!(ToolCall::dominant_primitive().is_some());
        assert!(ToolResult::dominant_primitive().is_some());
        assert!(PortalFiltration::dominant_primitive().is_some());
        assert!(SelectivePerfusion::dominant_primitive().is_some());
        assert!(BloodPayload::dominant_primitive().is_some());
        assert!(FrankStarling::dominant_primitive().is_some());
        assert!(CirculatoryHealth::dominant_primitive().is_some());
    }
}
