//! Jupyter & Voila tools: kernel management, server status, and notebook rendering.
//!
//! Calls the Jupyter REST API at localhost:8888 (configurable).
//! Token auto-discovery: JUPYTER_TOKEN env → `jupyter server list` parsing.

use crate::params::jupyter::{
    JupyterKernelsParams, JupyterKernelspecsParams, JupyterNotebookCreateParams,
    JupyterNotebookExecuteParams, JupyterPipelineParams, JupyterStatusParams, VoilaListParams,
    VoilaRenderParams, VoilaStatusParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::time::Instant;

const DEFAULT_URL: &str = "http://localhost:8888";
const TIMEOUT_SECS: u64 = 15;

/// Build a reqwest client and resolve base URL + token.
async fn jupyter_client(
    url: &Option<String>,
    token: &Option<String>,
) -> Result<(reqwest::Client, String, String), nexcore_error::NexError> {
    let base = url
        .as_deref()
        .unwrap_or(DEFAULT_URL)
        .trim_end_matches('/')
        .to_string();

    let tok = match token {
        Some(t) => t.clone(),
        None => std::env::var("JUPYTER_TOKEN")
            .map_err(|e| nexcore_error::NexError::new(format!("{e}")))
            .or_else(|_| discover_token())
            .map_err(|e| {
                nexcore_error::NexError::new(format!(
                    "No token: set JUPYTER_TOKEN env or pass token param. Discovery failed: {e}"
                ))
            })?,
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| nexcore_error::nexerror!("Failed to create HTTP client: {e}"))?;

    Ok((client, base, tok))
}

/// Discover token from `jupyter server list` output.
fn discover_token() -> Result<String, nexcore_error::NexError> {
    let output = std::process::Command::new("jupyter")
        .args(["server", "list"])
        .output()
        .map_err(|e| nexcore_error::nexerror!("Failed to run `jupyter server list`: {e}"))?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Parse lines like: http://localhost:8888/?token=abc123 :: /home/user
    for line in text.lines() {
        if let Some(pos) = line.find("token=") {
            let after = &line[pos + 6..];
            let tok = after
                .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                .next()
                .unwrap_or("");
            if !tok.is_empty() {
                return Ok(tok.to_string());
            }
        }
    }
    Err(nexcore_error::NexError::new(
        "No running Jupyter server with token found",
    ))
}

fn success_result(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_default(),
    )]))
}

fn error_result(msg: &impl std::fmt::Display) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![Content::text(
        json!({"success": false, "error": msg.to_string()}).to_string(),
    )]))
}

// ============================================================================
// Jupyter Core Tools
// ============================================================================

/// List running Jupyter kernels.
pub async fn kernels(params: JupyterKernelsParams) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let start = Instant::now();
    let resp = client
        .get(format!("{base}/api/kernels"))
        .query(&[("token", &token)])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!([]));
            let kernels: Vec<serde_json::Value> = match body.as_array() {
                Some(arr) => arr
                    .iter()
                    .map(|k| {
                        json!({
                            "id": k.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            "name": k.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            "state": k.get("execution_state").and_then(|v| v.as_str()).unwrap_or("unknown"),
                            "last_activity": k.get("last_activity").and_then(|v| v.as_str()).unwrap_or(""),
                        })
                    })
                    .collect(),
                None => vec![],
            };
            success_result(json!({
                "success": true,
                "count": kernels.len(),
                "kernels": kernels,
                "elapsed_ms": start.elapsed().as_millis(),
            }))
        }
        Ok(r) => error_result(&format!("Jupyter API returned {}", r.status())),
        Err(e) => error_result(&format!("Failed to reach Jupyter: {e}")),
    }
}

/// List available kernel specifications.
pub async fn kernelspecs(params: JupyterKernelspecsParams) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let start = Instant::now();
    let resp = client
        .get(format!("{base}/api/kernelspecs"))
        .query(&[("token", &token)])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!({}));
            let default_kernel = body.get("default").and_then(|v| v.as_str()).unwrap_or("");
            let specs: Vec<serde_json::Value> = body
                .get("kernelspecs")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(name, spec)| {
                            let display = spec
                                .get("spec")
                                .and_then(|s| s.get("display_name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or(name);
                            let language = spec
                                .get("spec")
                                .and_then(|s| s.get("language"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            json!({
                                "name": name,
                                "display_name": display,
                                "language": language,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            success_result(json!({
                "success": true,
                "default": default_kernel,
                "count": specs.len(),
                "kernelspecs": specs,
                "elapsed_ms": start.elapsed().as_millis(),
            }))
        }
        Ok(r) => error_result(&format!("Jupyter API returned {}", r.status())),
        Err(e) => error_result(&format!("Failed to reach Jupyter: {e}")),
    }
}

/// Jupyter server status — version, kernel count, extensions.
pub async fn status(params: JupyterStatusParams) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let start = Instant::now();

    // Fetch status + kernels in parallel
    let (status_resp, kernels_resp) = tokio::join!(
        client
            .get(format!("{base}/api/status"))
            .query(&[("token", &token)])
            .send(),
        client
            .get(format!("{base}/api/kernels"))
            .query(&[("token", &token)])
            .send(),
    );

    let version = match status_resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!({}));
            body.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string()
        }
        _ => "unreachable".to_string(),
    };

    let kernel_count = match kernels_resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!([]));
            body.as_array().map(|a| a.len()).unwrap_or(0)
        }
        _ => 0,
    };

    success_result(json!({
        "success": true,
        "server_url": base,
        "version": version,
        "running_kernels": kernel_count,
        "elapsed_ms": start.elapsed().as_millis(),
    }))
}

// ============================================================================
// Voila Tools
// ============================================================================

/// Check if Voila can render a specific notebook. Returns render URL.
pub async fn voila_render(params: VoilaRenderParams) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let notebook = params.notebook.trim_start_matches('/');
    let render_url = format!("{base}/voila/render/{notebook}");
    let start = Instant::now();

    // HEAD request to check if Voila can serve it
    let resp = client
        .get(&render_url)
        .query(&[("token", &token)])
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status_code = r.status().as_u16();
            let reachable = r.status().is_success();
            success_result(json!({
                "success": reachable,
                "notebook": notebook,
                "render_url": render_url,
                "status_code": status_code,
                "message": if reachable { "Notebook rendered successfully via Voila" } else { "Voila could not render this notebook" },
                "elapsed_ms": start.elapsed().as_millis(),
            }))
        }
        Err(e) => error_result(&format!("Failed to reach Voila: {e}")),
    }
}

/// Check Voila extension status.
pub async fn voila_status(params: VoilaStatusParams) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let start = Instant::now();

    // Check if /voila/ endpoint responds
    let voila_resp = client
        .get(format!("{base}/voila/"))
        .query(&[("token", &token)])
        .send()
        .await;

    let voila_active = match &voila_resp {
        Ok(r) => r.status().is_success() || r.status().as_u16() == 302,
        Err(_) => false,
    };

    let voila_status_code = match &voila_resp {
        Ok(r) => r.status().as_u16(),
        Err(_) => 0,
    };

    // List notebooks from root
    let notebooks_resp = client
        .get(format!("{base}/api/contents/"))
        .query(&[("token", &token), ("type", &"notebook".to_string())])
        .send()
        .await;

    let notebooks: Vec<String> = match notebooks_resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!({}));
            body.get("content")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|item| {
                            item.get("type").and_then(|v| v.as_str()) == Some("notebook")
                        })
                        .filter_map(|item| {
                            item.get("name").and_then(|v| v.as_str()).map(String::from)
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        _ => vec![],
    };

    success_result(json!({
        "success": true,
        "voila_active": voila_active,
        "voila_status_code": voila_status_code,
        "renderable_notebooks": notebooks,
        "notebook_count": notebooks.len(),
        "server_url": base,
        "elapsed_ms": start.elapsed().as_millis(),
    }))
}

/// List .ipynb files with metadata (kernel, size).
pub async fn voila_list(params: VoilaListParams) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let path = params.path.as_deref().unwrap_or("");
    let start = Instant::now();

    let resp = client
        .get(format!("{base}/api/contents/{path}"))
        .query(&[("token", &token)])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!({}));

            // Could be a single file or directory listing
            let items = if body.get("type").and_then(|v| v.as_str()) == Some("directory") {
                body.get("content")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default()
            } else {
                // Single file response
                vec![body.clone()]
            };

            let notebooks: Vec<serde_json::Value> = items
                .iter()
                .filter(|item| {
                    item.get("name")
                        .and_then(|v| v.as_str())
                        .is_some_and(|n| n.ends_with(".ipynb"))
                })
                .map(|item| {
                    json!({
                        "name": item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "path": item.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                        "size": item.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                        "last_modified": item.get("last_modified").and_then(|v| v.as_str()).unwrap_or(""),
                        "voila_url": format!("{base}/voila/render/{}", item.get("path").and_then(|v| v.as_str()).unwrap_or("")),
                    })
                })
                .collect();

            success_result(json!({
                "success": true,
                "path": if path.is_empty() { "/" } else { path },
                "count": notebooks.len(),
                "notebooks": notebooks,
                "elapsed_ms": start.elapsed().as_millis(),
            }))
        }
        Ok(r) => error_result(&format!("Jupyter API returned {}", r.status())),
        Err(e) => error_result(&format!("Failed to reach Jupyter: {e}")),
    }
}

// ============================================================================
// Pipeline Tools
// ============================================================================

/// Build nbformat v4 JSON for a notebook.
fn build_notebook_json(
    cells: &[crate::params::jupyter::NotebookCell],
    kernel: &str,
) -> serde_json::Value {
    let nb_cells: Vec<serde_json::Value> = cells
        .iter()
        .map(|c| {
            let ct = if c.cell_type == "code" {
                "code"
            } else {
                "markdown"
            };
            let mut cell = json!({
                "cell_type": ct,
                "source": c.source,
                "metadata": {},
            });
            if ct == "code" {
                cell["outputs"] = json!([]);
                cell["execution_count"] = json!(null);
            }
            cell
        })
        .collect();

    json!({
        "nbformat": 4,
        "nbformat_minor": 5,
        "metadata": {
            "kernelspec": {
                "display_name": kernel,
                "language": "python",
                "name": kernel,
            },
            "language_info": {
                "name": "python",
                "version": "3.11.0",
            }
        },
        "cells": nb_cells,
    })
}

/// Create a notebook with cells in one call via Jupyter REST API.
pub async fn notebook_create(
    params: JupyterNotebookCreateParams,
) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let kernel = params.kernel.as_deref().unwrap_or("python3");
    let path = params.path.trim_start_matches('/');
    let start = Instant::now();

    let notebook_json = build_notebook_json(&params.cells, kernel);

    // PUT /api/contents/{path} with notebook content
    let body = json!({
        "type": "notebook",
        "content": notebook_json,
    });

    let resp = client
        .put(format!("{base}/api/contents/{path}"))
        .query(&[("token", &token)])
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() || r.status().as_u16() == 201 => success_result(json!({
            "success": true,
            "path": path,
            "cell_count": params.cells.len(),
            "kernel": kernel,
            "voila_url": format!("{base}/voila/render/{path}"),
            "elapsed_ms": start.elapsed().as_millis(),
        })),
        Ok(r) => {
            let status = r.status().as_u16();
            let body_text = r.text().await.unwrap_or_default();
            error_result(&format!(
                "Jupyter API returned {status} creating notebook: {body_text}"
            ))
        }
        Err(e) => error_result(&format!("Failed to reach Jupyter: {e}")),
    }
}

/// Execute all cells in a notebook using `jupyter nbconvert --execute --inplace`.
pub async fn notebook_execute(
    params: JupyterNotebookExecuteParams,
) -> Result<CallToolResult, McpError> {
    let (client, base, token) = match jupyter_client(&params.url, &params.token).await {
        Ok(c) => c,
        Err(e) => return error_result(&e),
    };

    let path = params.path.trim_start_matches('/');
    let timeout = params.timeout.unwrap_or(120);
    let start = Instant::now();

    // Step 1: Verify the notebook exists via Jupyter API
    let contents_resp = client
        .get(format!("{base}/api/contents/{path}"))
        .query(&[("token", &token), ("content", &"0".to_string())])
        .send()
        .await;

    match &contents_resp {
        Ok(r) if !r.status().is_success() => {
            return error_result(&format!(
                "Notebook '{}' not found (HTTP {})",
                path,
                r.status()
            ));
        }
        Err(e) => return error_result(&format!("Failed to check notebook existence: {e}")),
        _ => {}
    }

    // Step 2: Execute via nbconvert subprocess (Strategy C — simple, reliable)
    let mut cmd_args = vec![
        "nbconvert".to_string(),
        "--to".to_string(),
        "notebook".to_string(),
        "--execute".to_string(),
        "--inplace".to_string(),
        format!("--ExecutePreprocessor.timeout={timeout}"),
    ];

    if let Some(ref kernel) = params.kernel {
        cmd_args.push(format!("--ExecutePreprocessor.kernel_name={kernel}"));
    }

    cmd_args.push(path.to_string());

    let exec_result = tokio::process::Command::new("jupyter")
        .args(&cmd_args)
        .output()
        .await;

    let (exec_success, exec_stderr) = match exec_result {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            (output.status.success(), stderr)
        }
        Err(e) => (false, format!("Failed to run jupyter nbconvert: {e}")),
    };

    if !exec_success {
        return error_result(&format!("Notebook execution failed: {exec_stderr}"));
    }

    // Step 3: Read the executed notebook back to extract outputs
    let nb_resp = client
        .get(format!("{base}/api/contents/{path}"))
        .query(&[("token", &token)])
        .send()
        .await;

    let cell_results: Vec<serde_json::Value> = match nb_resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or(json!({}));
            body.get("content")
                .and_then(|c| c.get("cells"))
                .and_then(|c| c.as_array())
                .map(|cells| {
                    cells
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| {
                            c.get("cell_type").and_then(|v| v.as_str()) == Some("code")
                        })
                        .map(|(i, c)| {
                            let outputs = c.get("outputs").and_then(|o| o.as_array());

                            // Extract text output
                            let output_text = outputs
                                .map(|outs| {
                                    outs.iter()
                                        .filter_map(|o| {
                                            // stream output
                                            if let Some(text) =
                                                o.get("text").and_then(|t| t.as_str())
                                            {
                                                return Some(text.to_string());
                                            }
                                            // execute_result or display_data
                                            if let Some(data) = o.get("data") {
                                                if let Some(text) = data
                                                    .get("text/plain")
                                                    .and_then(|t| t.as_str())
                                                {
                                                    return Some(text.to_string());
                                                }
                                            }
                                            None
                                        })
                                        .collect::<Vec<_>>()
                                        .join("")
                                })
                                .unwrap_or_default();

                            // Check for errors
                            let has_error = outputs
                                .map(|outs| {
                                    outs.iter().any(|o| {
                                        o.get("output_type").and_then(|t| t.as_str())
                                            == Some("error")
                                    })
                                })
                                .unwrap_or(false);

                            let error_info = if has_error {
                                outputs.and_then(|outs| {
                                    outs.iter().find_map(|o| {
                                        if o.get("output_type").and_then(|t| t.as_str())
                                            == Some("error")
                                        {
                                            Some(json!({
                                                "error_name": o.get("ename").and_then(|v| v.as_str()).unwrap_or(""),
                                                "error_value": o.get("evalue").and_then(|v| v.as_str()).unwrap_or(""),
                                            }))
                                        } else {
                                            None
                                        }
                                    })
                                })
                            } else {
                                None
                            };

                            let mut result = json!({
                                "index": i,
                                "status": if has_error { "error" } else { "ok" },
                                "output_text": output_text,
                            });

                            if let Some(err) = error_info {
                                result["error_name"] =
                                    err.get("error_name").cloned().unwrap_or(json!(""));
                                result["error_value"] =
                                    err.get("error_value").cloned().unwrap_or(json!(""));
                            }

                            result
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        _ => vec![],
    };

    let elapsed = start.elapsed().as_millis();

    success_result(json!({
        "success": true,
        "path": path,
        "cells": cell_results,
        "total_elapsed_ms": elapsed,
        "voila_url": format!("{base}/voila/render/{path}"),
    }))
}

/// Full pipeline: create→execute→render in one call.
pub async fn pipeline(params: JupyterPipelineParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let path = params.path.trim_start_matches('/').to_string();
    let kernel = params
        .kernel
        .clone()
        .unwrap_or_else(|| "python3".to_string());
    let timeout = params.timeout.unwrap_or(120);

    let mut stages = json!({
        "create": null,
        "execute": null,
        "render": null,
    });

    // Stage 1: Create (if cells provided)
    if let Some(ref cells) = params.cells {
        let create_result = notebook_create(JupyterNotebookCreateParams {
            path: path.clone(),
            cells: cells.clone(),
            kernel: Some(kernel.clone()),
            url: params.url.clone(),
            token: params.token.clone(),
        })
        .await;

        match &create_result {
            Ok(r) => {
                let text = r
                    .content
                    .first()
                    .and_then(|c| match &c.raw {
                        rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                let stage_ok = parsed
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                stages["create"] = json!({
                    "success": stage_ok,
                    "path": &path,
                    "cell_count": cells.len(),
                });
                if !stage_ok {
                    return success_result(json!({
                        "success": false,
                        "stages": stages,
                        "failed_at": "create",
                        "error": parsed.get("error").cloned().unwrap_or(json!("Create failed")),
                        "total_elapsed_ms": start.elapsed().as_millis(),
                    }));
                }
            }
            Err(_) => {
                stages["create"] = json!({"success": false});
                return success_result(json!({
                    "success": false,
                    "stages": stages,
                    "failed_at": "create",
                    "total_elapsed_ms": start.elapsed().as_millis(),
                }));
            }
        }
    }

    // Stage 2: Execute
    if params.execute {
        let exec_result = notebook_execute(JupyterNotebookExecuteParams {
            path: path.clone(),
            kernel: Some(kernel.clone()),
            timeout: Some(timeout),
            url: params.url.clone(),
            token: params.token.clone(),
        })
        .await;

        match &exec_result {
            Ok(r) => {
                let text = r
                    .content
                    .first()
                    .and_then(|c| match &c.raw {
                        rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                let stage_ok = parsed
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                stages["execute"] = json!({
                    "success": stage_ok,
                    "cells": parsed.get("cells").cloned().unwrap_or(json!([])),
                    "elapsed_ms": parsed.get("total_elapsed_ms").cloned().unwrap_or(json!(0)),
                });
                if !stage_ok {
                    return success_result(json!({
                        "success": false,
                        "stages": stages,
                        "failed_at": "execute",
                        "error": parsed.get("error").cloned().unwrap_or(json!("Execute failed")),
                        "total_elapsed_ms": start.elapsed().as_millis(),
                    }));
                }
            }
            Err(_) => {
                stages["execute"] = json!({"success": false});
                return success_result(json!({
                    "success": false,
                    "stages": stages,
                    "failed_at": "execute",
                    "total_elapsed_ms": start.elapsed().as_millis(),
                }));
            }
        }
    }

    // Stage 3: Voila render check
    let mut voila_url = format!(
        "{}/voila/render/{path}",
        params
            .url
            .as_deref()
            .unwrap_or(DEFAULT_URL)
            .trim_end_matches('/')
    );
    if params.render {
        let render_result = voila_render(VoilaRenderParams {
            notebook: path.clone(),
            url: params.url.clone(),
            token: params.token.clone(),
        })
        .await;

        match &render_result {
            Ok(r) => {
                let text = r
                    .content
                    .first()
                    .and_then(|c| match &c.raw {
                        rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                let stage_ok = parsed
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if let Some(url) = parsed.get("render_url").and_then(|v| v.as_str()) {
                    voila_url = url.to_string();
                }
                stages["render"] = json!({
                    "success": stage_ok,
                    "voila_url": &voila_url,
                    "status_code": parsed.get("status_code").cloned().unwrap_or(json!(0)),
                });
            }
            Err(_) => {
                stages["render"] = json!({"success": false});
            }
        }
    }

    // Overall success: all attempted stages succeeded
    let overall = ["create", "execute", "render"].iter().all(|stage| {
        stages.get(stage).map_or(true, |v| {
            v.is_null() || v.get("success").and_then(|s| s.as_bool()).unwrap_or(true)
        })
    });

    success_result(json!({
        "success": overall,
        "stages": stages,
        "voila_url": voila_url,
        "total_elapsed_ms": start.elapsed().as_millis(),
    }))
}
