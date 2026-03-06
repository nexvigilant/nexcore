//! Hook testing parameters.

use schemars::JsonSchema;
use serde::Deserialize;

/// Test a single hook with mock input.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HookTestParams {
    /// Hook script filename (e.g., "test-failure-extractor.sh").
    pub hook_name: String,
    /// Event type to simulate (e.g., "PostToolUse", "SessionStart").
    pub event_type: String,
    /// Mock JSON input to pipe to the hook's stdin.
    pub mock_input: serde_json::Value,
}

/// Test all hooks in the hooks directory.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HookTestAllParams {
    /// Filter by event type (optional — tests all if omitted).
    pub event_type: Option<String>,
}
