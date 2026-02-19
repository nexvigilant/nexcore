//! Filesystem tools: structured wrappers for mkdir, copy, move, chmod
//!
//! Highway Class I (Foundation) — local operations, <10ms SLA.
//! Uses Rust std::fs where possible, falls back to Command for chmod.

use crate::params::fs::{FsChmodParams, FsCopyParams, FsMkdirParams, FsMoveParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

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

/// Create directory (optionally with parents).
pub fn fs_mkdir(params: FsMkdirParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();

    let result = if params.parents {
        std::fs::create_dir_all(&params.path)
    } else {
        std::fs::create_dir(&params.path)
    };

    let elapsed = start.elapsed().as_millis();

    match result {
        Ok(()) => format_result(
            json!({
                "command": "fs_mkdir",
                "success": true,
                "elapsed_ms": elapsed,
                "path": params.path,
                "parents": params.parents,
            }),
            true,
        ),
        Err(e) => format_result(
            json!({
                "command": "fs_mkdir",
                "success": false,
                "elapsed_ms": elapsed,
                "path": params.path,
                "error": e.to_string(),
            }),
            false,
        ),
    }
}

/// Copy file or directory.
pub fn fs_copy(params: FsCopyParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();

    let result = if params.recursive {
        // Use cp -a for recursive to preserve attributes
        Command::new("cp")
            .args(["-a", &params.source, &params.dest])
            .output()
    } else {
        Command::new("cp")
            .args([&params.source, &params.dest])
            .output()
    };

    let elapsed = start.elapsed().as_millis();

    match result {
        Ok(output) if output.status.success() => format_result(
            json!({
                "command": "fs_copy",
                "success": true,
                "elapsed_ms": elapsed,
                "source": params.source,
                "dest": params.dest,
                "recursive": params.recursive,
            }),
            true,
        ),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format_result(
                json!({
                    "command": "fs_copy",
                    "success": false,
                    "elapsed_ms": elapsed,
                    "source": params.source,
                    "dest": params.dest,
                    "error": stderr.trim(),
                }),
                false,
            )
        }
        Err(e) => format_result(
            json!({
                "command": "fs_copy",
                "success": false,
                "elapsed_ms": elapsed,
                "error": e.to_string(),
            }),
            false,
        ),
    }
}

/// Move file or directory (safe: cp + verify + rm for directories).
pub fn fs_move(params: FsMoveParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let source_path = Path::new(&params.source);

    // For directories, use safe move pattern: cp -a + verify + rm
    if source_path.is_dir() {
        // Step 1: Copy
        let cp_output = Command::new("cp")
            .args(["-a", &params.source, &params.dest])
            .output()
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: format!("Failed to copy: {e}").into(),
                data: None,
            })?;

        if !cp_output.status.success() {
            let stderr = String::from_utf8_lossy(&cp_output.stderr);
            return format_result(
                json!({
                    "command": "fs_move",
                    "success": false,
                    "error": format!("Copy phase failed: {}", stderr.trim()),
                    "source": params.source,
                    "dest": params.dest,
                }),
                false,
            );
        }

        // Step 2: Verify destination exists
        let dest_path = Path::new(&params.dest);
        if !dest_path.exists() {
            return format_result(
                json!({
                    "command": "fs_move",
                    "success": false,
                    "error": "Verification failed: destination does not exist after copy",
                    "source": params.source,
                    "dest": params.dest,
                }),
                false,
            );
        }

        // Step 3: Remove source
        let rm_output = Command::new("rm")
            .args(["-rf", &params.source])
            .output()
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: format!("Failed to remove source: {e}").into(),
                data: None,
            })?;

        let elapsed = start.elapsed().as_millis();
        if rm_output.status.success() {
            format_result(
                json!({
                    "command": "fs_move",
                    "success": true,
                    "elapsed_ms": elapsed,
                    "source": params.source,
                    "dest": params.dest,
                    "method": "safe_move (cp -a + verify + rm)",
                }),
                true,
            )
        } else {
            let stderr = String::from_utf8_lossy(&rm_output.stderr);
            format_result(
                json!({
                    "command": "fs_move",
                    "success": false,
                    "elapsed_ms": elapsed,
                    "error": format!("Source removal failed (data is safe at destination): {}", stderr.trim()),
                    "source": params.source,
                    "dest": params.dest,
                }),
                false,
            )
        }
    } else {
        // Simple file move
        let output = Command::new("mv")
            .args([&params.source, &params.dest])
            .output()
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: format!("Failed to move: {e}").into(),
                data: None,
            })?;

        let elapsed = start.elapsed().as_millis();
        if output.status.success() {
            format_result(
                json!({
                    "command": "fs_move",
                    "success": true,
                    "elapsed_ms": elapsed,
                    "source": params.source,
                    "dest": params.dest,
                }),
                true,
            )
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format_result(
                json!({
                    "command": "fs_move",
                    "success": false,
                    "elapsed_ms": elapsed,
                    "error": stderr.trim(),
                    "source": params.source,
                    "dest": params.dest,
                }),
                false,
            )
        }
    }
}

/// Change file permissions (blocks 777, warns on recursive).
pub fn fs_chmod(params: FsChmodParams) -> Result<CallToolResult, McpError> {
    // Safety: block 777
    if params.mode == "777" {
        return format_result(
            json!({
                "command": "fs_chmod",
                "success": false,
                "error": "BLOCKED: Mode 777 (world-writable) is not allowed. Use a more restrictive mode like 755 or 750.",
                "path": params.path,
                "mode": params.mode,
            }),
            false,
        );
    }

    let start = Instant::now();
    let mut args = vec![];
    if params.recursive {
        args.push("-R");
    }
    args.push(&params.mode);
    args.push(&params.path);

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
    let output = Command::new("chmod")
        .args(&arg_refs)
        .output()
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to execute chmod: {e}").into(),
            data: None,
        })?;

    let elapsed = start.elapsed().as_millis();
    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr);

    let result = json!({
        "command": "fs_chmod",
        "success": success,
        "elapsed_ms": elapsed,
        "path": params.path,
        "mode": params.mode,
        "recursive": params.recursive,
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}
