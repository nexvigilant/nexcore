//! GCloud tools: Google Cloud CLI wrapper for GCP operations
//!
//! Wraps common gcloud commands for project management, compute, storage,
//! secrets, Firestore, Cloud Run, Cloud Functions, IAM, and logging.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;

/// Run gcloud command with JSON output format
async fn run_gcloud(
    args: &[&str],
    timeout_secs: u64,
) -> Result<serde_json::Value, nexcore_error::NexError> {
    let mut cmd_args: Vec<&str> = args.to_vec();
    cmd_args.push("--format=json");

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        Command::new("gcloud")
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| nexcore_error::NexError::new(format!("Command timed out after {timeout_secs}s")))?
    .map_err(|e| nexcore_error::NexError::new(format!("Failed to execute gcloud: {e}")))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            Ok(json!({"success": true, "data": {}}))
        } else {
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            Ok(json!({"success": true, "data": data}))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(json!({"success": false, "error": stderr.trim()}))
    }
}

/// Run gcloud command with text output (no JSON format)
async fn run_gcloud_text(
    args: &[&str],
    timeout_secs: u64,
) -> Result<serde_json::Value, nexcore_error::NexError> {
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        Command::new("gcloud")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| nexcore_error::NexError::new(format!("Command timed out after {timeout_secs}s")))?
    .map_err(|e| nexcore_error::NexError::new(format!("Failed to execute gcloud: {e}")))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(json!({"success": true, "output": stdout.trim()}))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(json!({"success": false, "error": stderr.trim()}))
    }
}

fn format_result(
    result: Result<serde_json::Value, nexcore_error::NexError>,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(v) => Ok(CallToolResult::success(vec![Content::text(v.to_string())])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({"success": false, "error": e.to_string()}).to_string(),
        )])),
    }
}

// ============================================================================
// Authentication & Configuration
// ============================================================================

/// List authenticated accounts
pub async fn auth_list() -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["auth", "list"], 60).await)
}

/// List current gcloud configuration
pub async fn config_list() -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["config", "list"], 60).await)
}

/// Get a specific config property
pub async fn config_get(property: &str) -> Result<CallToolResult, McpError> {
    format_result(run_gcloud_text(&["config", "get", property], 60).await)
}

/// Set a config property
pub async fn config_set(property: &str, value: &str) -> Result<CallToolResult, McpError> {
    format_result(run_gcloud_text(&["config", "set", property, value], 60).await)
}

// ============================================================================
// Projects
// ============================================================================

/// List accessible GCP projects
pub async fn projects_list() -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["projects", "list"], 60).await)
}

/// Describe a specific project
pub async fn projects_describe(project_id: &str) -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["projects", "describe", project_id], 60).await)
}

/// Get IAM policy for a project
pub async fn projects_get_iam_policy(project: &str) -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["projects", "get-iam-policy", project], 60).await)
}

// ============================================================================
// Secret Manager
// ============================================================================

/// List secrets
pub async fn secrets_list(project: Option<&str>) -> Result<CallToolResult, McpError> {
    let mut args = vec!["secrets", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 60).await)
}

/// Access a secret version
pub async fn secrets_versions_access(
    secret_name: &str,
    version: &str,
    project: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let secret_flag = format!("--secret={secret_name}");
    let mut args = vec!["secrets", "versions", "access", version, &secret_flag];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud_text(&args, 60).await)
}

// ============================================================================
// Cloud Storage
// ============================================================================

/// List storage buckets
pub async fn storage_buckets_list(project: Option<&str>) -> Result<CallToolResult, McpError> {
    let mut args = vec!["storage", "buckets", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 60).await)
}

/// List objects in a GCS path
pub async fn storage_ls(path: &str) -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["storage", "ls", path], 60).await)
}

/// Copy files to/from GCS
pub async fn storage_cp(
    source: &str,
    destination: &str,
    recursive: bool,
) -> Result<CallToolResult, McpError> {
    let mut args = vec!["storage", "cp"];
    if recursive {
        args.push("-r");
    }
    args.extend([source, destination]);
    format_result(run_gcloud_text(&args, 300).await)
}

// ============================================================================
// Firestore
// ============================================================================

/// List Firestore indexes
pub async fn firestore_indexes_list(project: Option<&str>) -> Result<CallToolResult, McpError> {
    let mut args = vec!["firestore", "indexes", "composite", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 60).await)
}

// ============================================================================
// Compute Engine
// ============================================================================

/// List compute instances
pub async fn compute_instances_list(
    project: Option<&str>,
    zone: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let mut args = vec!["compute", "instances", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    if let Some(z) = zone {
        args.extend(["--zones", z]);
    }
    format_result(run_gcloud(&args, 60).await)
}

/// List compute regions
pub async fn compute_regions_list() -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["compute", "regions", "list"], 60).await)
}

/// List compute zones
pub async fn compute_zones_list() -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["compute", "zones", "list"], 60).await)
}

// ============================================================================
// Cloud Run
// ============================================================================

/// List Cloud Run services
pub async fn run_services_list(
    project: Option<&str>,
    region: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let mut args = vec!["run", "services", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    if let Some(r) = region {
        args.extend(["--region", r]);
    }
    format_result(run_gcloud(&args, 60).await)
}

/// Describe a Cloud Run service
pub async fn run_services_describe(
    service_name: &str,
    region: &str,
    project: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let mut args = vec![
        "run",
        "services",
        "describe",
        service_name,
        "--region",
        region,
    ];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 60).await)
}

// ============================================================================
// Cloud Functions
// ============================================================================

/// List Cloud Functions
pub async fn functions_list(
    project: Option<&str>,
    region: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let mut args = vec!["functions", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    if let Some(r) = region {
        args.extend(["--regions", r]);
    }
    format_result(run_gcloud(&args, 60).await)
}

/// Describe a Cloud Function
pub async fn functions_describe(
    function_name: &str,
    region: &str,
    project: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let mut args = vec!["functions", "describe", function_name, "--region", region];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 60).await)
}

// ============================================================================
// IAM
// ============================================================================

/// List service accounts
pub async fn iam_service_accounts_list(project: Option<&str>) -> Result<CallToolResult, McpError> {
    let mut args = vec!["iam", "service-accounts", "list"];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 60).await)
}

// ============================================================================
// Logging
// ============================================================================

/// Read log entries
pub async fn logging_read(
    filter: &str,
    limit: u32,
    project: Option<&str>,
) -> Result<CallToolResult, McpError> {
    let limit_str = format!("--limit={limit}");
    let mut args = vec!["logging", "read", filter, &limit_str];
    if let Some(p) = project {
        args.extend(["--project", p]);
    }
    format_result(run_gcloud(&args, 120).await)
}

// ============================================================================
// Firebase
// ============================================================================

/// List Firebase projects
pub async fn firebase_projects_list() -> Result<CallToolResult, McpError> {
    format_result(run_gcloud(&["firebase", "projects", "list"], 60).await)
}

// ============================================================================
// Generic Command
// ============================================================================

/// Run arbitrary gcloud command with safety checks
pub async fn run_command(command: &str, timeout: u64) -> Result<CallToolResult, McpError> {
    // Check for dangerous patterns
    let dangerous = ["delete", "destroy", "remove", "--force", "-f"];
    let lower = command.to_lowercase();
    if dangerous.iter().any(|p| lower.contains(p)) {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({
                "success": false,
                "error": "Command contains potentially destructive operations. Please confirm with the user before running destructive commands.",
                "command": format!("gcloud {command}")
            }).to_string(),
        )]));
    }

    // Parse and execute
    let args: Vec<&str> = command.split_whitespace().collect();
    format_result(run_gcloud_text(&args, timeout).await)
}
