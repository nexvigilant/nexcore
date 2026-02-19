//! MCP Lock system implementation
//!
//! Provides agent-level locking for the MCP server operational state.
//! Uses brain_coordination primitives for synchronization.

use crate::params::{McpLockParams, McpUnlockParams};
use nexcore_brain::coordination::{AgentId, CoordinationRegistry, LockDuration, LockStatus};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Acquire an agent lock on an MCP resource
pub fn mcp_lock(params: McpLockParams) -> Result<CallToolResult, McpError> {
    let mut registry = CoordinationRegistry::load()
        .map_err(|e| McpError::internal_error(format!("Registry load failed: {}", e), None))?;
    let path = Path::new(&params.path);
    let agent_id = AgentId(params.agent_id.clone());
    let ttl = LockDuration(params.ttl_seconds);

    match registry.acquire_lock(path, agent_id, ttl) {
        Ok(success) => {
            let json = if success {
                json!({
                    "success": true,
                    "agent_id": params.agent_id,
                    "path": params.path,
                    "status": "Acquired"
                })
            } else {
                json!({
                    "success": false,
                    "error": "Resource is currently locked by another agent",
                    "path": params.path
                })
            };
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "success": false,
                "error": format!("Failed to acquire lock: {}", e)
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Release an agent lock on an MCP resource
pub fn mcp_unlock(params: McpUnlockParams) -> Result<CallToolResult, McpError> {
    let mut registry = CoordinationRegistry::load()
        .map_err(|e| McpError::internal_error(format!("Registry load failed: {}", e), None))?;
    let path = Path::new(&params.path);
    let agent_id = AgentId(params.agent_id.clone());

    match registry.release_lock(path, &agent_id) {
        Ok(success) => {
            if success {
                let _ = nexcore_brain::coordination::log_access(
                    &agent_id,
                    path,
                    "mcp_lock_release_success",
                );
            }
            let json = if success {
                json!({
                    "success": true,
                    "path": params.path,
                    "status": "Released"
                })
            } else {
                json!({
                    "success": false,
                    "error": "Lock not found or held by different agent",
                    "path": params.path
                })
            };
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "success": false,
                "error": format!("Failed to release lock: {}", e)
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Check status of an MCP resource lock
pub fn mcp_lock_status(
    params: crate::params::GcloudStoragePathParams,
) -> Result<CallToolResult, McpError> {
    let registry = CoordinationRegistry::load()
        .map_err(|e| McpError::internal_error(format!("Registry load failed: {}", e), None))?;
    let path = Path::new(&params.path);

    match registry.check_status(path) {
        Ok(status) => {
            let json = json!({
                "path": params.path,
                "status": match status {
                    LockStatus::Vacant => "Vacant",
                    LockStatus::Occupied => "Occupied",
                }
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "success": false,
                "error": format!("Status check failed: {}", e)
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}
