//! Helix Computing Parameters
//! Conservation law ∃ = ∂(×(ς, ∅)) as computable geometry.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for conservation check: ∃ = ∂(×(ς, ∅)).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ConservationCheckParams {
    /// ∂ — boundary sharpness [0,1]. How sharply defined is inside vs outside?
    pub boundary: f64,
    /// ς — state richness [0,1]. How much observable state exists?
    pub state: f64,
    /// ∅ — void clarity [0,1]. How clearly defined is what the system is NOT?
    pub void: f64,
}

/// Parameters for helix position lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HelixPositionParams {
    /// Helix turn number (0-4). 0=Primitives, 1=Conservation, 2=Crystalbook, 3=Derivative Identity, 4=Mutualism.
    pub turn: usize,
    /// Angular position within the turn [0, 2π]. Default: 0.
    #[serde(default)]
    pub theta: f64,
}

/// Parameters for mutualism test.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MutualismTestParams {
    /// ∃ of self before the action [0,1].
    pub existence_self_before: f64,
    /// ∃ of self after the action [0,1].
    pub existence_self_after: f64,
    /// ∃ of other(s) before the action [0,1].
    pub existence_other_before: f64,
    /// ∃ of other(s) after the action [0,1].
    pub existence_other_after: f64,
}

/// Parameters for helix advance gate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HelixAdvanceParams {
    /// Current helix turn (0-3).
    pub current_turn: usize,
    /// Current ∃ score [0,1].
    pub current_existence: f64,
}

/// Parameters for helix encode.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HelixEncodeParams {
    /// The concept to encode.
    pub concept: String,
    /// T1 primitives composing this concept.
    pub primitives: Vec<String>,
    /// ∂ — boundary sharpness [0,1].
    pub boundary: f64,
    /// ς — state richness [0,1].
    pub state: f64,
    /// ∅ — void clarity [0,1].
    pub void: f64,
}
