//! Parameter types for microgram MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for running a microgram chain.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgChainRunParams {
    /// Chain name (e.g. "station-openvigil-pipeline"). Resolved from ~/Projects/rsk-core/rsk/chains/.
    pub chain: String,
    /// JSON input for the chain (as a string).
    pub input: String,
}

/// Parameters for testing a microgram chain.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgChainTestParams {
    /// Chain name to test (e.g. "station-openvigil-pipeline"). If omitted, tests all chains.
    pub chain: Option<String>,
}

/// Parameters for running a single microgram.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgRunParams {
    /// Microgram path or name (e.g. "signal-to-causality" or "station/config-openvigil-triage").
    pub path: String,
    /// JSON input for the microgram (as a string).
    pub input: String,
}

/// Parameters for testing a single microgram.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgTestParams {
    /// Microgram path or name to self-test.
    pub path: String,
}

/// Parameters for testing all micrograms.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgTestAllParams {
    /// Micrograms directory (default: ~/Projects/rsk-core/rsk/micrograms).
    pub dir: Option<String>,
}

/// Parameters for microgram catalog.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgCatalogParams {
    /// Micrograms directory (default: ~/Projects/rsk-core/rsk/micrograms).
    pub dir: Option<String>,
}

/// Parameters for microgram coverage.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgCoverageParams {
    /// Micrograms directory (default: ~/Projects/rsk-core/rsk/micrograms).
    pub dir: Option<String>,
}

/// Parameters for microgram benchmark.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgBenchParams {
    /// Micrograms directory (default: ~/Projects/rsk-core/rsk/micrograms).
    pub dir: Option<String>,
    /// Number of iterations (default: 100).
    pub iterations: Option<u32>,
}
