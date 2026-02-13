//! MCP Usage Tracker Hook
//!
//! Event: PostToolUse
//!
//! Records when MCP tools are actually used for Phase 2 CTVP efficacy tracking.
//! This enables measurement of Capability Achievement Rate (CAR):
//!   CAR = sessions_with_mcp_usage / sessions_with_mcp_suggestions
//!
//! The hook fires after any tool use and checks if the tool was an MCP tool.
//! If so, it records the usage event to correlate with prior suggestions.
//!
//! ## Phase 3 CTVP: Canary Rollout
//!
//! Respects `~/.claude/mcp_efficacy_config.toml` settings:
//! - `feature_flags.enabled` - Master switch
//! - `feature_flags.rollout_percentage` - Canary percentage (0-100)

use nexcore_hooks::mcp_config::McpEfficacyConfig;
use nexcore_hooks::mcp_efficacy::with_efficacy_registry;
use nexcore_hooks::{exit_success_auto, read_input};

/// Prefix for NexCore MCP tools
const MCP_PREFIX: &str = "mcp__nexcore__";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Load config for canary rollout (Phase 3 CTVP)
    let config = McpEfficacyConfig::load();

    // Check canary rollout - skip if not in rollout cohort
    if !config.should_track(&input.session_id) {
        exit_success_auto();
    }

    // Only track PostToolUse events
    if input.hook_event_name != "PostToolUse" {
        exit_success_auto();
    }

    // Get the tool name
    let tool_name = match input.tool_name.as_deref() {
        Some(t) => t,
        None => exit_success_auto(),
    };

    // Only track MCP tool usage
    if !tool_name.starts_with(MCP_PREFIX) {
        exit_success_auto();
    }

    // Record the usage event
    let _ = with_efficacy_registry(|registry| {
        registry.record_usage(&input.session_id, tool_name);
    });

    exit_success_auto();
}
