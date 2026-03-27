//! Helix Computing MCP tools.
//!
//! Conservation law ∃ = ∂(×(ς, ∅)) as computable geometry.
//! Delegates to `nexcore-helix` crate for all computation.

use crate::params::helix::{
    ConservationCheckParams, HelixAdvanceParams, HelixEncodeParams, HelixPositionParams,
    MutualismTestParams,
};
use nexcore_helix::{
    ConservationInput, Turn, WeakestPrimitive,
    binding_laws, can_advance, conservation, derivatives, helix_position, mutualism_test, vice_risk,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn r4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

fn unit_check(name: &str, val: f64) -> Result<f64, McpError> {
    if (0.0..=1.0).contains(&val) {
        Ok(val)
    } else {
        Err(McpError::invalid_params(
            format!("{name} must be in [0,1], got {val}"),
            None,
        ))
    }
}

fn ok_json(v: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&v).unwrap_or_default(),
    )]))
}

/// Conservation check: ∃ = ∂(×(ς, ∅)).
pub fn conservation_check(params: ConservationCheckParams) -> Result<CallToolResult, McpError> {
    let b = unit_check("boundary", params.boundary)?;
    let s = unit_check("state", params.state)?;
    let v = unit_check("void", params.void)?;

    let result = conservation(ConservationInput {
        boundary: b,
        state: s,
        void: v,
    });
    let d = derivatives(ConservationInput {
        boundary: b,
        state: s,
        void: v,
    });

    ok_json(json!({
        "existence": r4(result.existence),
        "boundary": b,
        "state": s,
        "void": v,
        "weakest_primitive": result.weakest.symbol(),
        "classification": result.classification.label(),
        "formula": "∃ = ∂(×(ς, ∅))",
        "d_existence_d_boundary": r4(d.d_boundary),
        "d_existence_d_state": r4(d.d_state),
        "d_existence_d_void": r4(d.d_void),
        "highest_leverage": d.highest_leverage.symbol(),
        "vice_risk": vice_risk(result.weakest),
        "binding_laws": binding_laws(result.weakest),
    }))
}

/// Helix position: 3D coordinates on the knowledge helix.
pub fn position(params: HelixPositionParams) -> Result<CallToolResult, McpError> {
    let turn = Turn::from_index(params.turn).ok_or_else(|| {
        McpError::invalid_params("turn must be 0-4", None)
    })?;

    let pos = helix_position(turn, params.theta);

    ok_json(json!({
        "turn": params.turn,
        "turn_name": turn.name(),
        "what": turn.what(),
        "encoding": turn.encoding(),
        "altitude": r4(pos.z),
        "theta": r4(pos.theta),
        "x": r4(pos.x),
        "y": r4(pos.y),
        "z": r4(pos.z),
        "helix_properties": {
            "advances": "→ — moves to higher resolution with each turn",
            "returns": "κ — same angular truth revisited at each altitude",
            "bounds": "∂ — radius separates inside from outside"
        }
    }))
}

/// Mutualism test: does an action serve shared ∃?
pub fn mutualism(params: MutualismTestParams) -> Result<CallToolResult, McpError> {
    let sb = unit_check("existence_self_before", params.existence_self_before)?;
    let sa = unit_check("existence_self_after", params.existence_self_after)?;
    let ob = unit_check("existence_other_before", params.existence_other_before)?;
    let oa = unit_check("existence_other_after", params.existence_other_after)?;

    let result = mutualism_test(sb, sa, ob, oa);

    ok_json(json!({
        "mutualistic": result.mutualistic,
        "delta_self": r4(result.delta_self),
        "delta_other": r4(result.delta_other),
        "net_existence": r4(result.net_existence),
        "classification": result.classification.label(),
        "conservation_holds": result.conservation_holds,
    }))
}

/// Advance gate: can the system climb to the next helix turn?
pub fn advance(params: HelixAdvanceParams) -> Result<CallToolResult, McpError> {
    let turn = Turn::from_index(params.current_turn).ok_or_else(|| {
        McpError::invalid_params("current_turn must be 0-4", None)
    })?;
    let existence = unit_check("current_existence", params.current_existence)?;

    let next = turn.next().ok_or_else(|| {
        McpError::invalid_params("Already at turn 4 (Mutualism). The helix is complete.", None)
    })?;

    let ready = can_advance(turn, existence);

    ok_json(json!({
        "from_turn": params.current_turn,
        "to_turn": params.current_turn + 1,
        "from_name": turn.name(),
        "to_name": next.name(),
        "can_advance": ready,
        "current_existence": r4(existence),
        "requirement": next.what(),
    }))
}

/// Encode a concept through all 5 helix turns.
pub fn encode(params: HelixEncodeParams) -> Result<CallToolResult, McpError> {
    let b = unit_check("boundary", params.boundary)?;
    let s = unit_check("state", params.state)?;
    let v = unit_check("void", params.void)?;

    let input = ConservationInput {
        boundary: b,
        state: s,
        void: v,
    };
    let result = conservation(input);
    let d = derivatives(input);

    let balance = 1.0 - (s - v).abs();
    let mutualism_score = r4(b * balance * result.existence);

    let turns = vec![
        json!({
            "turn": 0, "name": "Primitives",
            "encoding": format!("{} composed of {} T1 primitives: [{}]",
                params.concept, params.primitives.len(), params.primitives.join(", "))
        }),
        json!({
            "turn": 1, "name": "Conservation",
            "encoding": format!("∃={:.3} = ∂({:.2}) × ς({:.2}) × ∅({:.2})",
                result.existence, b, s, v)
        }),
        json!({
            "turn": 2, "name": "Crystalbook",
            "encoding": format!("Governed by Laws {:?}. Vice risk: {}",
                binding_laws(result.weakest), vice_risk(result.weakest))
        }),
        json!({
            "turn": 3, "name": "Derivative Identity",
            "encoding": format!("∂∃/∂∂={}, ∂∃/∂ς={}, ∂∃/∂∅={}. Highest leverage: {}",
                r4(d.d_boundary), r4(d.d_state), r4(d.d_void), d.highest_leverage.symbol())
        }),
        json!({
            "turn": 4, "name": "Mutualism",
            "encoding": format!("Score: {:.3}. {}",
                mutualism_score,
                if mutualism_score >= 0.3 { "Serves shared existence." }
                else if mutualism_score >= 0.1 { "Partial mutualism." }
                else { "Low mutualism signal." })
        }),
    ];

    ok_json(json!({
        "concept": params.concept,
        "turns": turns,
        "existence": r4(result.existence),
        "helix_complete": true,
        "mutualism_score": mutualism_score,
        "weakest": result.weakest.symbol(),
        "classification": result.classification.label(),
    }))
}
