//! GitHub CLI (gh) tools: structured wrappers for PR and issue operations
//!
//! Highway Class IV (Service) — network calls, <5000ms SLA.
//! Uses tokio::process::Command for async execution.

use crate::params::gh::{
    GhApiParams, GhIssueViewParams, GhPrCreateParams, GhPrListParams, GhPrViewParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;

/// Run gh command with timeout and capture output.
async fn run_gh(
    args: &[&str],
    path: &Option<String>,
    timeout_secs: u64,
) -> Result<(bool, String, String), nexcore_error::NexError> {
    let mut cmd = Command::new("gh");
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(p) = path {
        cmd.current_dir(p);
    }

    let output = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output())
        .await
        .map_err(|_| nexcore_error::nexerror!("Command timed out after {timeout_secs}s"))?
        .map_err(|e| nexcore_error::nexerror!("Failed to execute gh: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.success(), stdout, stderr))
}

fn format_result(result: serde_json::Value, success: bool) -> Result<CallToolResult, McpError> {
    let content = vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )];
    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Create a pull request.
pub async fn gh_pr_create(params: GhPrCreateParams) -> Result<CallToolResult, McpError> {
    let mut args = vec!["pr", "create", "--title", &params.title];

    let body_owned: String;
    if let Some(ref body) = params.body {
        body_owned = body.clone();
        args.extend(["--body", &body_owned]);
    }

    let base_owned: String;
    if let Some(ref base) = params.base {
        base_owned = base.clone();
        args.extend(["--base", &base_owned]);
    }

    if params.draft {
        args.push("--draft");
    }

    match run_gh(&args, &params.path, 30).await {
        Ok((success, stdout, stderr)) => {
            let result = json!({
                "command": "gh pr create",
                "success": success,
                "title": params.title,
                "draft": params.draft,
                "url": stdout.trim(),
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// View a pull request.
pub async fn gh_pr_view(params: GhPrViewParams) -> Result<CallToolResult, McpError> {
    let num_str: String;
    let mut args = vec![
        "pr",
        "view",
        "--json",
        "number,title,state,body,author,baseRefName,headRefName,url,additions,deletions,changedFiles",
    ];

    if let Some(n) = params.number {
        num_str = n.to_string();
        args.insert(2, &num_str);
    }

    match run_gh(&args, &params.path, 15).await {
        Ok((success, stdout, stderr)) => {
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            let result = json!({
                "command": "gh pr view",
                "success": success,
                "data": data,
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// List pull requests.
pub async fn gh_pr_list(params: GhPrListParams) -> Result<CallToolResult, McpError> {
    let limit_str = params.limit.to_string();
    let mut args = vec![
        "pr",
        "list",
        "--json",
        "number,title,state,author,headRefName,createdAt,url",
        "--limit",
        &limit_str,
    ];

    let state_owned: String;
    if let Some(ref state) = params.state {
        state_owned = state.clone();
        args.extend(["--state", &state_owned]);
    }

    match run_gh(&args, &params.path, 15).await {
        Ok((success, stdout, stderr)) => {
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            let count = data.as_array().map(|a| a.len()).unwrap_or(0);
            let result = json!({
                "command": "gh pr list",
                "success": success,
                "count": count,
                "prs": data,
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// View an issue.
pub async fn gh_issue_view(params: GhIssueViewParams) -> Result<CallToolResult, McpError> {
    let num_str = params.number.to_string();
    let args = vec![
        "issue",
        "view",
        &num_str,
        "--json",
        "number,title,state,body,author,labels,assignees,createdAt,url",
    ];

    match run_gh(&args, &params.path, 15).await {
        Ok((success, stdout, stderr)) => {
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            let result = json!({
                "command": "gh issue view",
                "success": success,
                "data": data,
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// Call the GitHub REST API.
pub async fn gh_api(params: GhApiParams) -> Result<CallToolResult, McpError> {
    let method = params.method.as_deref().unwrap_or("GET");

    // Safety: block DELETE unless explicitly allowed
    if method.eq_ignore_ascii_case("DELETE") && !params.allow_delete {
        let result = json!({
            "command": "gh api",
            "success": false,
            "error": "BLOCKED: DELETE method requires explicit confirmation. Set allow_delete=true to proceed.",
            "endpoint": params.endpoint,
        });
        return format_result(result, false);
    }

    let mut args = vec!["api", &params.endpoint, "--method", method];

    let body_owned: String;
    if let Some(ref body) = params.body {
        body_owned = body.clone();
        args.extend(["--input", "-"]);
        // For simplicity, pass body via --raw-field or just use -f
        // Actually, gh api supports --body or raw stdin. Let's use -H and raw.
        // Simplest approach: use --input with the body
        args = vec![
            "api",
            &params.endpoint,
            "--method",
            method,
            "--raw-field",
            &body_owned,
        ];
    }

    match run_gh(&args, &None, 30).await {
        Ok((success, stdout, stderr)) => {
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            let result = json!({
                "command": "gh api",
                "success": success,
                "endpoint": params.endpoint,
                "method": method,
                "data": data,
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}
