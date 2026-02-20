//! Parameters for relay fidelity tools.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for creating a relay chain and computing fidelity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RelayChainComputeParams {
    /// Ordered list of relay hops. Each hop has a stage name and fidelity (0.0-1.0).
    pub hops: Vec<RelayHopInput>,
    /// Minimum acceptable total fidelity (default: 0.80 safety-critical).
    #[serde(default = "default_f_min")]
    pub f_min: f64,
}

/// A single relay hop input.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RelayHopInput {
    /// Stage name (e.g., "ingest", "detect", "evaluate").
    pub stage: String,
    /// Fidelity of this hop (0.0-1.0). How much essential information survives.
    pub fidelity: f64,
    /// Activation threshold. Signal must meet this to activate the relay.
    #[serde(default)]
    pub threshold: f64,
    /// Whether the hop activated. Defaults to true.
    #[serde(default = "default_true")]
    pub activated: bool,
}

/// Parameters for computing composed fidelity from a list of values.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RelayFidelityComposeParams {
    /// List of fidelity values to compose multiplicatively.
    pub values: Vec<f64>,
}

fn default_f_min() -> f64 {
    0.80
}

fn default_true() -> bool {
    true
}
