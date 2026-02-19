//! Kellnr Registry HTTP tools (15).
//! Consolidated from kellnr-mcp registry functions in lib.rs.

use crate::params::kellnr::{
    KellnrCrateNameParams, KellnrCrateOwnerParams, KellnrCrateVersionParams,
    KellnrListAllCratesParams, KellnrSearchCratesParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn text_result(text: String) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text)])
}

fn json_result(value: serde_json::Value) -> CallToolResult {
    text_result(serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()))
}

fn registry_url() -> String {
    std::env::var("KELLNR_REGISTRY_URL").unwrap_or_else(|_| "https://crates.nexvigilant.com".into())
}

fn auth_token() -> String {
    std::env::var("KELLNR_AUTH_TOKEN").unwrap_or_default()
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20").replace('/', "%2F")
}

async fn registry_get(url: &str) -> CallToolResult {
    match reqwest::Client::new()
        .get(url)
        .header("Authorization", auth_token())
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(body) => text_result(body),
            Err(e) => text_result(format!("Error reading response: {e}")),
        },
        Err(e) => text_result(format!("HTTP error: {e}")),
    }
}

async fn registry_put(url: &str, body: &serde_json::Value) -> CallToolResult {
    match reqwest::Client::new()
        .put(url)
        .header("Authorization", auth_token())
        .json(body)
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(body) => text_result(body),
            Err(e) => text_result(format!("Error reading response: {e}")),
        },
        Err(e) => text_result(format!("HTTP error: {e}")),
    }
}

async fn registry_put_no_body(url: &str) -> CallToolResult {
    match reqwest::Client::new()
        .put(url)
        .header("Authorization", auth_token())
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(body) => text_result(body),
            Err(e) => text_result(format!("Error reading response: {e}")),
        },
        Err(e) => text_result(format!("HTTP error: {e}")),
    }
}

async fn registry_delete(url: &str, body: &serde_json::Value) -> CallToolResult {
    match reqwest::Client::new()
        .delete(url)
        .header("Authorization", auth_token())
        .json(body)
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(body) => text_result(body),
            Err(e) => text_result(format!("Error reading response: {e}")),
        },
        Err(e) => text_result(format!("HTTP error: {e}")),
    }
}

async fn registry_delete_no_body(url: &str) -> CallToolResult {
    match reqwest::Client::new()
        .delete(url)
        .header("Authorization", auth_token())
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(body) => text_result(body),
            Err(e) => text_result(format!("Error reading response: {e}")),
        },
        Err(e) => text_result(format!("HTTP error: {e}")),
    }
}

// =========================================================================
// 15 registry tools
// =========================================================================

/// Search crates by name, keyword, or description.
pub async fn search_crates(params: KellnrSearchCratesParams) -> Result<CallToolResult, McpError> {
    let limit = params.per_page.unwrap_or(20);
    let url = format!(
        "{}/api/v1/crates?q={}&per_page={}",
        registry_url(),
        urlencoding(&params.query),
        limit
    );
    Ok(registry_get(&url).await)
}

/// Get all versions and metadata for a crate.
pub async fn get_crate_metadata(params: KellnrCrateNameParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}",
        registry_url(),
        urlencoding(&params.crate_name)
    );
    Ok(registry_get(&url).await)
}

/// List versions with yank status for a crate.
pub async fn list_crate_versions(
    params: KellnrCrateNameParams,
) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/versions",
        registry_url(),
        urlencoding(&params.crate_name)
    );
    Ok(registry_get(&url).await)
}

/// Get specific version details for a crate.
pub async fn get_version_details(
    params: KellnrCrateVersionParams,
) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/{}",
        registry_url(),
        urlencoding(&params.crate_name),
        urlencoding(&params.version)
    );
    Ok(registry_get(&url).await)
}

/// List crate owners (users and teams).
pub async fn list_owners(params: KellnrCrateNameParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/owners",
        registry_url(),
        urlencoding(&params.crate_name)
    );
    Ok(registry_get(&url).await)
}

/// Add a user as owner of a crate.
pub async fn add_owner(params: KellnrCrateOwnerParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/owners",
        registry_url(),
        urlencoding(&params.crate_name)
    );
    let body = json!({"users": [params.username]});
    Ok(registry_put(&url, &body).await)
}

/// Remove an owner from a crate.
pub async fn remove_owner(params: KellnrCrateOwnerParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/owners",
        registry_url(),
        urlencoding(&params.crate_name)
    );
    let body = json!({"users": [params.username]});
    Ok(registry_delete(&url, &body).await)
}

/// Mark a version as unavailable (yank).
pub async fn yank_version(params: KellnrCrateVersionParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/{}/yank",
        registry_url(),
        urlencoding(&params.crate_name),
        urlencoding(&params.version)
    );
    Ok(registry_delete_no_body(&url).await)
}

/// Mark a yanked version as available again (unyank).
pub async fn unyank_version(params: KellnrCrateVersionParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/{}/unyank",
        registry_url(),
        urlencoding(&params.crate_name),
        urlencoding(&params.version)
    );
    Ok(registry_put_no_body(&url).await)
}

/// List all crates in the registry.
pub async fn list_all_crates(
    params: KellnrListAllCratesParams,
) -> Result<CallToolResult, McpError> {
    let limit = params.per_page.unwrap_or(100);
    let url = format!("{}/api/v1/crates?per_page={}", registry_url(), limit);
    Ok(registry_get(&url).await)
}

/// Get dependency graph for a specific version.
pub async fn get_dependencies(
    params: KellnrCrateVersionParams,
) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/{}/dependencies",
        registry_url(),
        urlencoding(&params.crate_name),
        urlencoding(&params.version)
    );
    Ok(registry_get(&url).await)
}

/// Get reverse dependencies (crates that depend on this one).
pub async fn get_dependents(params: KellnrCrateNameParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/reverse_dependencies",
        registry_url(),
        urlencoding(&params.crate_name)
    );
    Ok(registry_get(&url).await)
}

/// Check registry connectivity and health.
pub async fn health_check() -> Result<CallToolResult, McpError> {
    let url = format!("{}/api/v1/health", registry_url());
    Ok(registry_get(&url).await)
}

/// Download a .crate file (returns download URL).
pub async fn download_crate(params: KellnrCrateVersionParams) -> Result<CallToolResult, McpError> {
    let url = format!(
        "{}/api/v1/crates/{}/{}/download",
        registry_url(),
        urlencoding(&params.crate_name),
        urlencoding(&params.version)
    );
    Ok(text_result(format!("Download URL: {url}")))
}

/// Get registry statistics (total crates, versions, downloads).
pub async fn registry_stats() -> Result<CallToolResult, McpError> {
    let url = format!("{}/api/v1/crates?per_page=1000", registry_url());
    match reqwest::Client::new()
        .get(&url)
        .header("Authorization", auth_token())
        .send()
        .await
    {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let crates = data["crates"].as_array();
                let total_crates = crates.map(|c| c.len()).unwrap_or(0);
                let total_versions: usize = crates
                    .map(|c| c.iter().filter_map(|cr| cr["max_version"].as_str()).count())
                    .unwrap_or(0);
                let total_downloads: u64 = crates
                    .map(|c| c.iter().filter_map(|cr| cr["downloads"].as_u64()).sum())
                    .unwrap_or(0);
                Ok(json_result(json!({
                    "success": true,
                    "total_crates": total_crates,
                    "total_versions": total_versions,
                    "total_downloads": total_downloads
                })))
            }
            Err(e) => Ok(text_result(format!("Error parsing response: {e}"))),
        },
        Err(e) => Ok(text_result(format!("HTTP error: {e}"))),
    }
}
