//! Google Sheets tools — read, write, search spreadsheets via Sheets API v4.
//!
//! Consolidated from `gsheets-mcp` satellite server.
//! 7 tools: list_sheets, read_range, batch_read, write_range, append, metadata, search.
//!
//! Auth: service account JWT or gcloud ADC (refresh token), lazy-initialized.
//!
//! Tier: T3 (μ Mapping + σ Sequence + ∂ Boundary + ς State + π Persistence)

use std::path::PathBuf;
use std::sync::OnceLock;

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::params::{
    GsheetsAppendParams, GsheetsBatchReadParams, GsheetsReadRangeParams, GsheetsSearchParams,
    GsheetsSpreadsheetIdParams, GsheetsWriteRangeParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};

// ============================================================================
// Constants
// ============================================================================

const SHEETS_BASE: &str = "https://sheets.googleapis.com/v4/spreadsheets";
const TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const SHEETS_SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets";
const REFRESH_MARGIN_SECS: i64 = 300;

// ============================================================================
// Lazy state
// ============================================================================

static CLIENT: OnceLock<RwLock<Option<SheetsClient>>> = OnceLock::new();

fn client_lock() -> &'static RwLock<Option<SheetsClient>> {
    CLIENT.get_or_init(|| RwLock::new(None))
}

async fn get_client() -> Result<SheetsClient, McpError> {
    // Check if already initialized
    {
        let guard = client_lock().read().await;
        if let Some(ref c) = *guard {
            return Ok(c.clone());
        }
    }

    // Initialize
    let mut guard = client_lock().write().await;
    if let Some(ref c) = *guard {
        return Ok(c.clone());
    }

    let client = SheetsClient::new()
        .await
        .map_err(|e| McpError::new(ErrorCode(500), format!("Sheets auth: {e}"), None))?;
    *guard = Some(client.clone());
    Ok(client)
}

// ============================================================================
// Auth types
// ============================================================================

#[derive(Debug, nexcore_error::Error)]
enum AuthError {
    #[error("no credentials found")]
    KeyNotFound,
    #[error("failed to read credentials: {0}")]
    KeyRead(String),
    #[error("failed to parse credentials: {0}")]
    KeyParse(String),
    #[error("JWT encoding failed: {0}")]
    JwtEncode(String),
    #[error("token exchange failed: {0}")]
    TokenExchange(String),
    #[error("token response missing access_token")]
    MissingAccessToken,
}

#[derive(Debug, Clone)]
enum CredentialSource {
    ServiceAccount(ServiceAccountKey),
    AuthorizedUser(UserCredentials),
}

#[derive(Debug, Clone, Deserialize)]
struct ServiceAccountKey {
    private_key_id: Option<String>,
    private_key: String,
    client_email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UserCredentials {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Clone)]
struct TokenCache {
    access_token: String,
    expires_at: i64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<i64>,
}

// ============================================================================
// Sheets API response types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct SpreadsheetMeta {
    #[serde(rename = "spreadsheetId")]
    spreadsheet_id: String,
    properties: SpreadsheetProperties,
    sheets: Vec<SheetEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct SpreadsheetProperties {
    title: String,
    locale: Option<String>,
    #[serde(rename = "timeZone")]
    time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SheetEntry {
    properties: SheetProperties,
}

#[derive(Debug, Clone, Deserialize)]
struct SheetProperties {
    #[serde(rename = "sheetId")]
    sheet_id: u64,
    title: String,
    index: u32,
    #[serde(rename = "sheetType")]
    sheet_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ValueRange {
    range: Option<String>,
    #[serde(default)]
    values: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
struct BatchGetResponse {
    #[serde(rename = "valueRanges", default)]
    value_ranges: Vec<ValueRange>,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdateResponse {
    #[serde(rename = "updatedRange")]
    updated_range: Option<String>,
    #[serde(rename = "updatedRows")]
    updated_rows: Option<u32>,
    #[serde(rename = "updatedCells")]
    updated_cells: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct AppendResponse {
    #[serde(rename = "tableRange")]
    table_range: Option<String>,
    updates: Option<UpdateResponse>,
}

// ============================================================================
// SheetsClient — auth + HTTP
// ============================================================================

#[derive(Clone)]
struct SheetsClient {
    source: std::sync::Arc<CredentialSource>,
    token: std::sync::Arc<RwLock<Option<TokenCache>>>,
    http: reqwest::Client,
}

impl SheetsClient {
    async fn new() -> Result<Self, AuthError> {
        let source = load_credentials().await?;
        Ok(Self {
            source: std::sync::Arc::new(source),
            token: std::sync::Arc::new(RwLock::new(None)),
            http: reqwest::Client::new(),
        })
    }

    async fn get_token(&self) -> Result<String, AuthError> {
        {
            let guard = self.token.read().await;
            if let Some(ref cached) = *guard {
                if DateTime::now().timestamp() < cached.expires_at - REFRESH_MARGIN_SECS {
                    return Ok(cached.access_token.clone());
                }
            }
        }
        let mut guard = self.token.write().await;
        if let Some(ref cached) = *guard {
            if DateTime::now().timestamp() < cached.expires_at - REFRESH_MARGIN_SECS {
                return Ok(cached.access_token.clone());
            }
        }
        let token_resp = self.acquire_token().await?;
        let access_token = token_resp.access_token.clone();
        let expires_at = DateTime::now().timestamp() + token_resp.expires_in.unwrap_or(3600);
        *guard = Some(TokenCache {
            access_token: access_token.clone(),
            expires_at,
        });
        Ok(access_token)
    }

    async fn acquire_token(&self) -> Result<TokenResponse, AuthError> {
        match self.source.as_ref() {
            CredentialSource::ServiceAccount(key) => {
                let now = DateTime::now().timestamp();
                let claims = JwtClaims {
                    iss: key.client_email.clone(),
                    scope: SHEETS_SCOPE.to_string(),
                    aud: TOKEN_URI.to_string(),
                    iat: now,
                    exp: now + 3600,
                };
                let mut header = Header::new(Algorithm::RS256);
                if let Some(ref kid) = key.private_key_id {
                    header.kid = Some(kid.clone());
                }
                let encoding_key = EncodingKey::from_rsa_pem(key.private_key.as_bytes())
                    .map_err(|e| AuthError::JwtEncode(e.to_string()))?;
                let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key)
                    .map_err(|e| AuthError::JwtEncode(e.to_string()))?;
                let resp = self
                    .http
                    .post(TOKEN_URI)
                    .form(&[
                        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                        ("assertion", &jwt),
                    ])
                    .send()
                    .await
                    .map_err(|e| AuthError::TokenExchange(e.to_string()))?;
                parse_token_response(resp).await
            }
            CredentialSource::AuthorizedUser(creds) => {
                let resp = self
                    .http
                    .post(TOKEN_URI)
                    .form(&[
                        ("grant_type", "refresh_token"),
                        ("client_id", &creds.client_id),
                        ("client_secret", &creds.client_secret),
                        ("refresh_token", &creds.refresh_token),
                    ])
                    .send()
                    .await
                    .map_err(|e| AuthError::TokenExchange(e.to_string()))?;
                parse_token_response(resp).await
            }
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let token = self.get_token().await.map_err(|e| format!("auth: {e}"))?;
        let resp = self
            .http
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| format!("HTTP: {e}"))?;
        if resp.status().as_u16() == 401 {
            let token = self
                .get_token()
                .await
                .map_err(|e| format!("auth refresh: {e}"))?;
            let resp = self
                .http
                .get(url)
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| format!("HTTP retry: {e}"))?;
            return parse_api_response(resp).await;
        }
        parse_api_response(resp).await
    }

    async fn put_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T, String> {
        let token = self.get_token().await.map_err(|e| format!("auth: {e}"))?;
        let resp = self
            .http
            .put(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("HTTP: {e}"))?;
        parse_api_response(resp).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T, String> {
        let token = self.get_token().await.map_err(|e| format!("auth: {e}"))?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("HTTP: {e}"))?;
        parse_api_response(resp).await
    }
}

// ============================================================================
// Credential loading
// ============================================================================

async fn load_credentials() -> Result<CredentialSource, AuthError> {
    if let Ok(path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return load_credential_file(&p).await;
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let sa_path = PathBuf::from(&home).join(".config/gsheets-mcp/service-account.json");
    if sa_path.exists() {
        return load_credential_file(&sa_path).await;
    }
    let adc_path = PathBuf::from(&home).join(".config/gcloud/application_default_credentials.json");
    if adc_path.exists() {
        return load_credential_file(&adc_path).await;
    }
    Err(AuthError::KeyNotFound)
}

async fn load_credential_file(path: &PathBuf) -> Result<CredentialSource, AuthError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AuthError::KeyRead(e.to_string()))?;
    let raw: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| AuthError::KeyParse(e.to_string()))?;
    let cred_type = raw
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    match cred_type {
        "service_account" => {
            let key: ServiceAccountKey =
                serde_json::from_str(&content).map_err(|e| AuthError::KeyParse(e.to_string()))?;
            info!("gsheets: loaded service account key");
            Ok(CredentialSource::ServiceAccount(key))
        }
        "authorized_user" => {
            let creds: UserCredentials =
                serde_json::from_str(&content).map_err(|e| AuthError::KeyParse(e.to_string()))?;
            info!("gsheets: loaded authorized_user credentials");
            Ok(CredentialSource::AuthorizedUser(creds))
        }
        other => Err(AuthError::KeyParse(format!(
            "unsupported credential type: {other}"
        ))),
    }
}

async fn parse_token_response(resp: reqwest::Response) -> Result<TokenResponse, AuthError> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read body>".into());
        return Err(AuthError::TokenExchange(format!("HTTP {status}: {body}")));
    }
    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| AuthError::TokenExchange(e.to_string()))?;
    if token_resp.access_token.is_empty() {
        return Err(AuthError::MissingAccessToken);
    }
    Ok(token_resp)
}

async fn parse_api_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}"));
    }
    let text = resp.text().await.map_err(|e| format!("read: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse: {e}"))
}

// ============================================================================
// Tool implementations
// ============================================================================

/// List all sheet tabs in a Google Spreadsheet.
pub async fn gsheets_list_sheets(
    params: GsheetsSpreadsheetIdParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let url = format!(
        "{SHEETS_BASE}/{}?fields=spreadsheetId,properties,sheets.properties",
        params.spreadsheet_id
    );
    let meta: SpreadsheetMeta = client.get_json(&url).await.map_err(sheets_err)?;

    let mut lines = Vec::new();
    lines.push(format!("Spreadsheet: {}", meta.properties.title));
    lines.push(format!("Tabs ({}): ", meta.sheets.len()));
    for sheet in &meta.sheets {
        let p = &sheet.properties;
        lines.push(format!(
            "  [{}] {} (id={}, type={})",
            p.index,
            p.title,
            p.sheet_id,
            p.sheet_type.as_deref().unwrap_or("GRID")
        ));
    }
    Ok(text_result(&lines.join("\n")))
}

/// Read a cell range from a Google Spreadsheet.
pub async fn gsheets_read_range(
    params: GsheetsReadRangeParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let encoded = urlenc(&params.range);
    let url = format!("{SHEETS_BASE}/{}/values/{encoded}", params.spreadsheet_id);
    let vr: ValueRange = client.get_json(&url).await.map_err(sheets_err)?;
    Ok(text_result(&format_value_range(&vr)))
}

/// Read multiple cell ranges in one call.
pub async fn gsheets_batch_read(
    params: GsheetsBatchReadParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let range_params: String = params
        .ranges
        .iter()
        .map(|r| format!("ranges={}", urlenc(r)))
        .collect::<Vec<_>>()
        .join("&");
    let url = format!(
        "{SHEETS_BASE}/{}/values:batchGet?{range_params}",
        params.spreadsheet_id
    );
    let batch: BatchGetResponse = client.get_json(&url).await.map_err(sheets_err)?;

    let mut out = Vec::new();
    for vr in &batch.value_ranges {
        if let Some(ref range) = vr.range {
            out.push(format!("--- {range} ---"));
        }
        out.push(format_value_range(vr));
        out.push(String::new());
    }
    Ok(text_result(&out.join("\n")))
}

/// Write values to a cell range.
pub async fn gsheets_write_range(
    params: GsheetsWriteRangeParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let encoded = urlenc(&params.range);
    let url = format!(
        "{SHEETS_BASE}/{}/values/{encoded}?valueInputOption=USER_ENTERED",
        params.spreadsheet_id
    );
    let body = json!({
        "range": params.range,
        "majorDimension": "ROWS",
        "values": params.values,
    });
    let resp: UpdateResponse = client.put_json(&url, &body).await.map_err(sheets_err)?;
    let summary = json!({
        "updatedRange": resp.updated_range,
        "updatedRows": resp.updated_rows,
        "updatedCells": resp.updated_cells,
    });
    Ok(text_result(
        &serde_json::to_string_pretty(&summary).unwrap_or_default(),
    ))
}

/// Append rows to the end of a table.
pub async fn gsheets_append(params: GsheetsAppendParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let encoded = urlenc(&params.range);
    let url = format!(
        "{SHEETS_BASE}/{}/values/{encoded}:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
        params.spreadsheet_id
    );
    let body = json!({
        "range": params.range,
        "majorDimension": "ROWS",
        "values": params.values,
    });
    let resp: AppendResponse = client.post_json(&url, &body).await.map_err(sheets_err)?;
    let summary = json!({
        "tableRange": resp.table_range,
        "updatedRows": resp.updates.as_ref().and_then(|u| u.updated_rows),
        "updatedCells": resp.updates.as_ref().and_then(|u| u.updated_cells),
    });
    Ok(text_result(
        &serde_json::to_string_pretty(&summary).unwrap_or_default(),
    ))
}

/// Get spreadsheet metadata.
pub async fn gsheets_metadata(
    params: GsheetsSpreadsheetIdParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let url = format!(
        "{SHEETS_BASE}/{}?fields=spreadsheetId,properties,sheets.properties",
        params.spreadsheet_id
    );
    let meta: SpreadsheetMeta = client.get_json(&url).await.map_err(sheets_err)?;
    let payload = json!({
        "spreadsheetId": meta.spreadsheet_id,
        "title": meta.properties.title,
        "locale": meta.properties.locale,
        "timeZone": meta.properties.time_zone,
        "sheetCount": meta.sheets.len(),
        "sheets": meta.sheets.iter().map(|s| json!({
            "title": s.properties.title,
            "sheetId": s.properties.sheet_id,
            "index": s.properties.index,
            "type": s.properties.sheet_type,
        })).collect::<Vec<_>>(),
    });
    Ok(text_result(
        &serde_json::to_string_pretty(&payload).unwrap_or_default(),
    ))
}

/// Search for a substring across all cells in a range.
pub async fn gsheets_search(params: GsheetsSearchParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let ranges_to_search = if let Some(ref range) = params.range {
        vec![range.clone()]
    } else {
        let url = format!(
            "{SHEETS_BASE}/{}?fields=spreadsheetId,properties,sheets.properties",
            params.spreadsheet_id
        );
        let meta: SpreadsheetMeta = client.get_json(&url).await.map_err(sheets_err)?;
        meta.sheets
            .iter()
            .map(|s| s.properties.title.clone())
            .collect()
    };

    let query_lower = params.query.to_lowercase();
    let mut matches = Vec::new();

    for range in &ranges_to_search {
        let encoded = urlenc(range);
        let url = format!("{SHEETS_BASE}/{}/values/{encoded}", params.spreadsheet_id);
        let vr: ValueRange = match client.get_json(&url).await {
            Ok(v) => v,
            Err(_) => continue,
        };
        for (row_idx, row) in vr.values.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let cell_str = cell_to_string(cell);
                if cell_str.to_lowercase().contains(&query_lower) {
                    let col_letter = col_index_to_letter(col_idx);
                    matches.push(format!("{range}!{col_letter}{}: {cell_str}", row_idx + 1));
                }
            }
        }
    }

    if matches.is_empty() {
        Ok(text_result(&format!(
            "No matches found for '{}'",
            params.query
        )))
    } else {
        let header = format!(
            "Found {} match(es) for '{}':\n",
            matches.len(),
            params.query
        );
        Ok(text_result(&format!("{header}{}", matches.join("\n"))))
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn format_value_range(vr: &ValueRange) -> String {
    if vr.values.is_empty() {
        return "(empty range)".to_string();
    }
    let mut lines = Vec::new();
    if let Some(ref range) = vr.range {
        lines.push(format!("Range: {range}"));
    }
    lines.push(format!("Rows: {}", vr.values.len()));
    for (i, row) in vr.values.iter().enumerate() {
        let cells: Vec<String> = row.iter().map(cell_to_string).collect();
        lines.push(format!("  [{}] {}", i + 1, cells.join(" | ")));
    }
    lines.join("\n")
}

fn cell_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn col_index_to_letter(idx: usize) -> String {
    let mut result = String::new();
    let mut n = idx;
    loop {
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    result
}

fn urlenc(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('!', "%21")
        .replace('#', "%23")
}

fn sheets_err(e: String) -> McpError {
    McpError::new(ErrorCode(500), e, None)
}

fn text_result(s: &str) -> CallToolResult {
    CallToolResult::success(vec![Content::text(s)])
}
