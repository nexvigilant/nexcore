//! Hook error identification parameters.

use schemars::JsonSchema;
use serde::Deserialize;

/// Identify failing hooks by scanning settings.json registrations.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HookErrorIdentifyParams {
    /// Filter by event type (e.g., "PostToolUse", "SessionStart"). Scans all if omitted.
    pub event: Option<String>,
    /// Filter by matcher pattern (e.g., "Edit", "Bash"). Scans all if omitted.
    pub matcher: Option<String>,
}
