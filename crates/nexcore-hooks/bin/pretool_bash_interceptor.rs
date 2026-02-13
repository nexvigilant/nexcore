//! MCP Bash Interceptor Hook
//!
//! Event: PreToolUse (Bash)
//!
//! When Claude is about to run a Bash command that has an MCP equivalent,
//! this hook suggests using the MCP tool instead.
//!
//! This enforces the "MCP-first" philosophy and prevents reinventing the wheel
//! with shell commands when we have optimized Rust implementations.

use nexcore_hooks::{exit_success_auto, exit_warn, read_input};

/// Bash command patterns that have MCP equivalents
struct BashToMcp {
    /// Bash command patterns (substring match)
    bash_patterns: &'static [&'static str],
    /// Equivalent MCP tool
    mcp_tool: &'static str,
    /// Why MCP is better
    reason: &'static str,
}

/// Mapping of Bash commands to MCP tools
const BASH_TO_MCP: &[BashToMcp] = &[
    // Hashing
    BashToMcp {
        bash_patterns: &["sha256sum", "shasum -a 256", "openssl dgst -sha256"],
        mcp_tool: "mcp__nexcore__foundation_sha256",
        reason: "Programmatic access, 20x faster for multiple hashes",
    },
    // GCloud commands
    BashToMcp {
        bash_patterns: &["gcloud secrets list"],
        mcp_tool: "mcp__nexcore__gcloud_secrets_list",
        reason: "Type-safe with structured output",
    },
    BashToMcp {
        bash_patterns: &["gcloud secrets versions access"],
        mcp_tool: "mcp__nexcore__gcloud_secrets_versions_access",
        reason: "Direct access without shell escaping issues",
    },
    BashToMcp {
        bash_patterns: &["gcloud run services list"],
        mcp_tool: "mcp__nexcore__gcloud_run_services_list",
        reason: "Structured JSON output",
    },
    BashToMcp {
        bash_patterns: &["gcloud run services describe"],
        mcp_tool: "mcp__nexcore__gcloud_run_services_describe",
        reason: "Type-safe service description",
    },
    BashToMcp {
        bash_patterns: &["gcloud storage ls", "gsutil ls"],
        mcp_tool: "mcp__nexcore__gcloud_storage_ls",
        reason: "Structured listing with metadata",
    },
    BashToMcp {
        bash_patterns: &["gcloud storage cp", "gsutil cp"],
        mcp_tool: "mcp__nexcore__gcloud_storage_cp",
        reason: "Progress tracking and error handling",
    },
    BashToMcp {
        bash_patterns: &["gcloud projects list"],
        mcp_tool: "mcp__nexcore__gcloud_projects_list",
        reason: "Structured project listing",
    },
    BashToMcp {
        bash_patterns: &["gcloud config list"],
        mcp_tool: "mcp__nexcore__gcloud_config_list",
        reason: "Typed configuration access",
    },
    BashToMcp {
        bash_patterns: &["gcloud logging read"],
        mcp_tool: "mcp__nexcore__gcloud_logging_read",
        reason: "Structured log entries with filtering",
    },
    BashToMcp {
        bash_patterns: &["gcloud auth list"],
        mcp_tool: "mcp__nexcore__gcloud_auth_list",
        reason: "Authenticated accounts listing",
    },
    // YAML processing
    BashToMcp {
        bash_patterns: &[
            "yq",
            "python -c \"import yaml\"",
            "python3 -c \"import yaml\"",
        ],
        mcp_tool: "mcp__nexcore__foundation_yaml_parse",
        reason: "7x faster with validation",
    },
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process Bash tool calls
    if input.tool_name.as_deref() != Some("Bash") {
        exit_success_auto();
    }

    // Get the command being run
    let command = match input
        .tool_input
        .as_ref()
        .and_then(|v| v.get("command"))
        .and_then(|c| c.as_str())
    {
        Some(c) => c,
        None => exit_success_auto(),
    };

    // Check for MCP alternatives
    let alternatives: Vec<_> = BASH_TO_MCP
        .iter()
        .filter(|b| b.bash_patterns.iter().any(|p| command.contains(p)))
        .collect();

    if alternatives.is_empty() {
        exit_success_auto();
    }

    // Build warning message
    let mut msg = String::new();
    msg.push_str("🔄 MCP ALTERNATIVE AVAILABLE\n\n");
    msg.push_str(&format!(
        "Command: `{}`\n\n",
        command.chars().take(80).collect::<String>()
    ));

    alternatives.iter().for_each(|alt| {
        msg.push_str(&format!("→ Consider: `{}`\n", alt.mcp_tool));
        msg.push_str(&format!("  Reason: {}\n\n", alt.reason));
    });

    msg.push_str("MCP tools provide type-safety, better error handling, and are often faster.\n");
    msg.push_str("Proceed if Bash is necessary for this specific use case.");

    // Warn but allow (exit code 1)
    exit_warn(&msg);
}
