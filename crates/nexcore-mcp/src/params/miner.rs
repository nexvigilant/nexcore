//! Borrow Miner Parameters (Resource Discovery)
//!
//! Signal checking for drug-event pairs.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for signal check.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalCheckParams {
    /// Drug name.
    pub drug: String,
    /// Adverse event.
    pub event: String,
}
