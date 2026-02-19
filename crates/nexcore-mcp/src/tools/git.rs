//! Git tools: structured CLI wrappers for git operations
//!
//! Wraps git commands and returns structured JSON responses.
//! Highway Class III (Orchestration) — local operations, <500ms SLA.

use crate::params::git::{
    GitBranchParams, GitCheckoutParams, GitCommitParams, GitDiffParams, GitLogParams,
    GitPushParams, GitStashParams, GitStatusParams, StashAction,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Resolve git working directory.
fn resolve_git_path(path: &Option<String>) -> Option<PathBuf> {
    path.as_ref().map(PathBuf::from)
}

/// Run a git command and capture output.
fn run_git(args: &[&str], path: &Option<String>) -> Result<(bool, String, String, u128), McpError> {
    let start = Instant::now();
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(p) = resolve_git_path(path) {
        cmd.current_dir(p);
    }
    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute git: {e}").into(),
        data: None,
    })?;
    let elapsed = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.success(), stdout, stderr, elapsed))
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

/// Sensitive file patterns that should never be committed.
const SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    "credentials",
    ".key",
    ".pem",
    ".p12",
    ".pfx",
    "id_rsa",
    "id_ed25519",
    ".secret",
];

/// Check if a filename matches sensitive patterns.
fn is_sensitive_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    SENSITIVE_PATTERNS.iter().any(|pat| lower.contains(pat))
}

/// git status --porcelain with structured output.
pub fn git_status(params: GitStatusParams) -> Result<CallToolResult, McpError> {
    let (success, stdout, stderr, elapsed) =
        run_git(&["status", "--porcelain", "-b"], &params.path)?;

    let mut branch = String::new();
    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut staged = 0u32;
    let mut modified = 0u32;
    let mut untracked = 0u32;

    for line in stdout.lines() {
        if let Some(b) = line.strip_prefix("## ") {
            branch = b.split("...").next().unwrap_or(b).to_string();
            continue;
        }
        if line.len() < 4 {
            continue;
        }
        let index = &line[0..1];
        let worktree = &line[1..2];
        let file = line[3..].trim();

        match (index, worktree) {
            ("?", "?") => {
                untracked += 1;
                files.push(json!({"status": "untracked", "file": file}));
            }
            _ => {
                if index != " " && index != "?" {
                    staged += 1;
                }
                if worktree != " " && worktree != "?" {
                    modified += 1;
                }
                files.push(json!({
                    "index": index.trim(),
                    "worktree": worktree.trim(),
                    "file": file,
                }));
            }
        }
    }

    let result = json!({
        "command": "git status",
        "success": success,
        "elapsed_ms": elapsed,
        "branch": branch,
        "summary": {
            "staged": staged,
            "modified": modified,
            "untracked": untracked,
            "total": files.len(),
        },
        "files": files,
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git diff with structured output.
pub fn git_diff(params: GitDiffParams) -> Result<CallToolResult, McpError> {
    let mut args = vec!["diff", "--stat"];
    if params.staged {
        args.push("--staged");
    }

    // Build owned strings for ref_spec and file so they live long enough
    let ref_owned: String;
    let file_owned: String;

    if let Some(ref r) = params.ref_spec {
        ref_owned = r.clone();
        args.push(&ref_owned);
    }
    if let Some(ref f) = params.file {
        file_owned = f.clone();
        args.push("--");
        args.push(&file_owned);
    }

    let (success, stat_out, stderr, elapsed) = run_git(&args, &params.path)?;

    // Also get the full diff (limited)
    let mut full_args = vec!["diff"];
    if params.staged {
        full_args.push("--staged");
    }

    let ref_owned2: String;
    let file_owned2: String;

    if let Some(ref r) = params.ref_spec {
        ref_owned2 = r.clone();
        full_args.push(&ref_owned2);
    }
    if let Some(ref f) = params.file {
        file_owned2 = f.clone();
        full_args.push("--");
        full_args.push(&file_owned2);
    }

    let (_, full_out, _, _) = run_git(&full_args, &params.path)?;

    // Truncate if too large
    let diff_text = if full_out.len() > 50_000 {
        format!(
            "{}...\n[truncated — {} bytes total]",
            &full_out[..50_000],
            full_out.len()
        )
    } else {
        full_out
    };

    let result = json!({
        "command": "git diff",
        "success": success,
        "elapsed_ms": elapsed,
        "staged": params.staged,
        "stat": stat_out.trim(),
        "diff": diff_text,
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git log with structured output.
pub fn git_log(params: GitLogParams) -> Result<CallToolResult, McpError> {
    let count_str = format!("-{}", params.count);
    let mut args = vec![
        "log",
        &count_str,
        "--format=%H%n%an%n%ae%n%aI%n%s%n---END---",
    ];
    if params.oneline {
        args = vec!["log", &count_str, "--oneline"];
    }

    let (success, stdout, stderr, elapsed) = run_git(&args, &params.path)?;

    let commits: Vec<serde_json::Value> = if params.oneline {
        stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| json!(l.trim()))
            .collect()
    } else {
        let mut result = Vec::new();
        let mut lines = stdout.lines().peekable();
        while lines.peek().is_some() {
            let hash = lines.next().unwrap_or_default().trim().to_string();
            let author = lines.next().unwrap_or_default().trim().to_string();
            let email = lines.next().unwrap_or_default().trim().to_string();
            let date = lines.next().unwrap_or_default().trim().to_string();
            let subject = lines.next().unwrap_or_default().trim().to_string();
            // Consume the ---END--- separator
            if let Some(sep) = lines.next() {
                if sep.trim() != "---END---" {
                    // Skip unexpected lines
                    continue;
                }
            }
            if !hash.is_empty() {
                result.push(json!({
                    "hash": hash,
                    "author": author,
                    "email": email,
                    "date": date,
                    "subject": subject,
                }));
            }
        }
        result
    };

    let result = json!({
        "command": "git log",
        "success": success,
        "elapsed_ms": elapsed,
        "count": commits.len(),
        "commits": commits,
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git add + git commit with safety checks.
pub fn git_commit(params: GitCommitParams) -> Result<CallToolResult, McpError> {
    // Safety: check for sensitive files
    let blocked: Vec<&String> = params
        .files
        .iter()
        .filter(|f| is_sensitive_file(f))
        .collect();

    if !blocked.is_empty() {
        let result = json!({
            "command": "git commit",
            "success": false,
            "error": "BLOCKED: Refusing to commit files that may contain secrets",
            "blocked_files": blocked,
            "hint": "Remove sensitive files from the list or add them to .gitignore",
        });
        return format_result(result, false);
    }

    // Stage files if specified
    if !params.files.is_empty() {
        let file_refs: Vec<&str> = params.files.iter().map(|s| s.as_str()).collect();
        let mut add_args = vec!["add"];
        add_args.extend(file_refs.iter());
        let (add_ok, _, add_stderr, _) = run_git(&add_args, &params.path)?;
        if !add_ok {
            let result = json!({
                "command": "git add",
                "success": false,
                "error": add_stderr.trim(),
            });
            return format_result(result, false);
        }
    }

    // Commit
    let (success, stdout, stderr, elapsed) =
        run_git(&["commit", "-m", &params.message], &params.path)?;

    let result = json!({
        "command": "git commit",
        "success": success,
        "elapsed_ms": elapsed,
        "message": params.message,
        "files_staged": params.files,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git branch with list/create/delete modes.
pub fn git_branch(params: GitBranchParams) -> Result<CallToolResult, McpError> {
    if let Some(ref name) = params.create {
        let (success, stdout, stderr, elapsed) = run_git(&["branch", name], &params.path)?;
        let result = json!({
            "command": "git branch (create)",
            "success": success,
            "elapsed_ms": elapsed,
            "created": name,
            "output": stdout.trim(),
            "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
        });
        return format_result(result, success);
    }

    if let Some(ref name) = params.delete {
        let (success, stdout, stderr, elapsed) = run_git(&["branch", "-d", name], &params.path)?;
        let result = json!({
            "command": "git branch (delete)",
            "success": success,
            "elapsed_ms": elapsed,
            "deleted": name,
            "output": stdout.trim(),
            "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
        });
        return format_result(result, success);
    }

    // Default: list
    let args = if params.list {
        vec![
            "branch",
            "-a",
            "--format=%(refname:short) %(objectname:short) %(upstream:short)",
        ]
    } else {
        vec![
            "branch",
            "--format=%(refname:short) %(objectname:short) %(upstream:short)",
        ]
    };
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
    let (success, stdout, stderr, elapsed) = run_git(&arg_refs, &params.path)?;

    let branches: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(3, ' ').collect();
            json!({
                "name": parts.first().unwrap_or(&""),
                "commit": parts.get(1).unwrap_or(&""),
                "upstream": parts.get(2).unwrap_or(&""),
            })
        })
        .collect();

    let result = json!({
        "command": "git branch (list)",
        "success": success,
        "elapsed_ms": elapsed,
        "count": branches.len(),
        "branches": branches,
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git checkout with optional -b flag.
pub fn git_checkout(params: GitCheckoutParams) -> Result<CallToolResult, McpError> {
    let args = if params.create {
        vec!["checkout", "-b", &params.target]
    } else {
        vec!["checkout", &params.target]
    };
    let (success, stdout, stderr, elapsed) = run_git(&args, &params.path)?;

    let result = json!({
        "command": "git checkout",
        "success": success,
        "elapsed_ms": elapsed,
        "target": params.target,
        "created": params.create,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git push with safety checks.
pub fn git_push(params: GitPushParams) -> Result<CallToolResult, McpError> {
    // Safety: block force push
    if params.force {
        let result = json!({
            "command": "git push",
            "success": false,
            "error": "BLOCKED: Force push is disabled by default. This is a destructive operation that can overwrite remote history.",
            "hint": "If you absolutely need to force push, use the Bash tool with explicit user confirmation.",
        });
        return format_result(result, false);
    }

    // Safety: warn on push to main/master
    if let Some(ref branch) = params.branch {
        let lower = branch.to_lowercase();
        if lower == "main" || lower == "master" {
            let result = json!({
                "command": "git push",
                "success": false,
                "error": "BLOCKED: Direct push to main/master. Use a feature branch and create a PR instead.",
                "branch": branch,
            });
            return format_result(result, false);
        }
    }

    let remote = params.remote.as_deref().unwrap_or("origin");
    let mut args = vec!["push"];
    if params.set_upstream {
        args.push("-u");
    }
    args.push(remote);
    if let Some(ref branch) = params.branch {
        args.push(branch);
    }

    let (success, stdout, stderr, elapsed) = run_git(&args, &params.path)?;

    let result = json!({
        "command": "git push",
        "success": success,
        "elapsed_ms": elapsed,
        "remote": remote,
        "branch": params.branch,
        "set_upstream": params.set_upstream,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// git stash with push/pop/list/drop actions.
pub fn git_stash(params: GitStashParams) -> Result<CallToolResult, McpError> {
    let args = match params.action {
        StashAction::Push => {
            if let Some(ref msg) = params.message {
                vec!["stash", "push", "-m", msg.as_str()]
            } else {
                vec!["stash", "push"]
            }
        }
        StashAction::Pop => vec!["stash", "pop"],
        StashAction::List => vec!["stash", "list"],
        StashAction::Drop => vec!["stash", "drop"],
    };

    let (success, stdout, stderr, elapsed) = run_git(&args, &params.path)?;

    let action_name = match params.action {
        StashAction::Push => "push",
        StashAction::Pop => "pop",
        StashAction::List => "list",
        StashAction::Drop => "drop",
    };

    let result = json!({
        "command": format!("git stash {action_name}"),
        "success": success,
        "elapsed_ms": elapsed,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}
