//! HTTP Request tool — MCP-native curl replacement.
//!
//! Makes HTTP requests to any URL, including localhost.
//! Replaces curl for testing and integration scenarios.
//!
//! Primitives: → Causality + λ Location + μ Mapping + ∂ Boundary

use crate::params::http::HttpRequestParams;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::time::Duration;

/// Execute an HTTP request.
pub async fn http_request(params: HttpRequestParams) -> Result<CallToolResult, McpError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(params.timeout_secs))
        .no_proxy()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| {
            McpError::internal_error(format!("failed to create HTTP client: {e}"), None)
        })?;

    let method = params.method.to_uppercase();
    let req_method = match method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        other => {
            return Ok(CallToolResult::error(vec![Content::text(
                json!({
                    "success": false,
                    "error": format!("unsupported HTTP method: {other}")
                })
                .to_string(),
            )]));
        }
    };

    let mut req = client.request(req_method.clone(), &params.url);

    // Add headers
    for header_str in &params.headers {
        if let Some((key, value)) = header_str.split_once('=') {
            req = req.header(key.trim(), value.trim());
        }
    }

    // Add body
    if let Some(ref body) = params.body {
        req = req.body(body.clone());
        // Auto-set content-type if not explicitly provided
        if !params
            .headers
            .iter()
            .any(|h| h.to_lowercase().starts_with("content-type"))
        {
            // Detect JSON body
            if body.starts_with('{') || body.starts_with('[') {
                req = req.header("content-type", "application/json");
            }
        }
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let status_text = resp.status().canonical_reason().unwrap_or("Unknown");

            // Collect response headers
            let resp_headers: Vec<(String, String)> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<binary>").to_string()))
                .collect();

            let body_text = resp.text().await.unwrap_or_default();

            if params.body_only {
                Ok(CallToolResult::success(vec![Content::text(body_text)]))
            } else {
                let result = json!({
                    "success": true,
                    "status": status,
                    "status_text": status_text,
                    "headers": resp_headers.into_iter()
                        .map(|(k, v)| json!({k: v}))
                        .collect::<Vec<_>>(),
                    "body": body_text,
                    "method": method,
                    "url": params.url,
                });

                if status >= 200 && status < 400 {
                    Ok(CallToolResult::success(vec![Content::text(
                        result.to_string(),
                    )]))
                } else {
                    Ok(CallToolResult::error(vec![Content::text(
                        result.to_string(),
                    )]))
                }
            }
        }
        Err(e) => {
            let error_kind = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "connection_refused"
            } else if e.is_redirect() {
                "redirect_error"
            } else {
                "request_failed"
            };

            Ok(CallToolResult::error(vec![Content::text(
                json!({
                    "success": false,
                    "error": error_kind,
                    "message": e.to_string(),
                    "url": params.url,
                    "method": method,
                })
                .to_string(),
            )]))
        }
    }
}
