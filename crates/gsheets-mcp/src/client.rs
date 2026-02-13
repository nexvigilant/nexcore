//! Google Sheets API v4 HTTP client.
//!
//! Thin `reqwest` wrapper with automatic token refresh on 401.
//! Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)

use tracing::{debug, warn};

use crate::auth::AuthManager;
use crate::types::{AppendResponse, BatchGetResponse, SpreadsheetMeta, UpdateResponse, ValueRange};

/// Base URL for Google Sheets API v4.
const SHEETS_BASE: &str = "https://sheets.googleapis.com/v4/spreadsheets";

/// Errors from the Sheets API client.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("auth error: {0}")]
    Auth(#[from] crate::auth::AuthError),
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("failed to parse response: {0}")]
    Parse(String),
    #[error("Sheets API error {status}: {body}")]
    Api { status: u16, body: String },
}

/// Google Sheets API v4 client with automatic token management.
#[derive(Clone)]
pub struct SheetsClient {
    auth: AuthManager,
    http: reqwest::Client,
}

impl SheetsClient {
    /// Create a new client, loading service account credentials.
    pub async fn new() -> Result<Self, ClientError> {
        let auth = AuthManager::new().await?;
        Ok(Self {
            auth,
            http: reqwest::Client::new(),
        })
    }

    /// Get spreadsheet metadata (title, sheets, locale).
    pub async fn get_spreadsheet(
        &self,
        spreadsheet_id: &str,
    ) -> Result<SpreadsheetMeta, ClientError> {
        let url = format!(
            "{SHEETS_BASE}/{spreadsheet_id}?fields=spreadsheetId,properties,sheets.properties"
        );
        self.get_json(&url).await
    }

    /// Read a single range of cells.
    pub async fn read_range(
        &self,
        spreadsheet_id: &str,
        range: &str,
    ) -> Result<ValueRange, ClientError> {
        let encoded_range = urlencoding(range);
        let url = format!("{SHEETS_BASE}/{spreadsheet_id}/values/{encoded_range}");
        self.get_json(&url).await
    }

    /// Read multiple ranges in a single call.
    pub async fn batch_read(
        &self,
        spreadsheet_id: &str,
        ranges: &[String],
    ) -> Result<BatchGetResponse, ClientError> {
        let range_params: String = ranges
            .iter()
            .map(|r| format!("ranges={}", urlencoding(r)))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{SHEETS_BASE}/{spreadsheet_id}/values:batchGet?{range_params}");
        self.get_json(&url).await
    }

    /// Write values to a cell range.
    pub async fn write_range(
        &self,
        spreadsheet_id: &str,
        range: &str,
        values: Vec<Vec<String>>,
    ) -> Result<UpdateResponse, ClientError> {
        let encoded_range = urlencoding(range);
        let url = format!(
            "{SHEETS_BASE}/{spreadsheet_id}/values/{encoded_range}?valueInputOption=USER_ENTERED"
        );
        let body = serde_json::json!({
            "range": range,
            "majorDimension": "ROWS",
            "values": values,
        });
        self.put_json(&url, &body).await
    }

    /// Append rows to the end of a range.
    pub async fn append_values(
        &self,
        spreadsheet_id: &str,
        range: &str,
        values: Vec<Vec<String>>,
    ) -> Result<AppendResponse, ClientError> {
        let encoded_range = urlencoding(range);
        let url = format!(
            "{SHEETS_BASE}/{spreadsheet_id}/values/{encoded_range}:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS"
        );
        let body = serde_json::json!({
            "range": range,
            "majorDimension": "ROWS",
            "values": values,
        });
        self.post_json(&url, &body).await
    }

    // -----------------------------------------------------------------------
    // Internal HTTP helpers with auto-retry on 401
    // -----------------------------------------------------------------------

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ClientError> {
        let token = self.auth.get_token().await?;
        let resp = self
            .http
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        // Retry once on 401 with refreshed token.
        if resp.status().as_u16() == 401 {
            warn!("Got 401, refreshing token and retrying");
            let token = self.auth.get_token().await?;
            let resp = self
                .http
                .get(url)
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))?;
            return self.parse_response(resp).await;
        }

        self.parse_response(resp).await
    }

    async fn put_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T, ClientError> {
        let token = self.auth.get_token().await?;
        let resp = self
            .http
            .put(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if resp.status().as_u16() == 401 {
            warn!("Got 401 on PUT, refreshing token and retrying");
            let token = self.auth.get_token().await?;
            let resp = self
                .http
                .put(url)
                .bearer_auth(&token)
                .json(body)
                .send()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))?;
            return self.parse_response(resp).await;
        }

        self.parse_response(resp).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T, ClientError> {
        let token = self.auth.get_token().await?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if resp.status().as_u16() == 401 {
            warn!("Got 401 on POST, refreshing token and retrying");
            let token = self.auth.get_token().await?;
            let resp = self
                .http
                .post(url)
                .bearer_auth(&token)
                .json(body)
                .send()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))?;
            return self.parse_response(resp).await;
        }

        self.parse_response(resp).await
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, ClientError> {
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".into());
            return Err(ClientError::Api { status, body });
        }

        let text = resp
            .text()
            .await
            .map_err(|e| ClientError::Parse(e.to_string()))?;

        debug!(response_len = text.len(), "Parsing API response");

        serde_json::from_str(&text)
            .map_err(|e| ClientError::Parse(format!("{e}: {}", &text[..text.len().min(500)])))
    }
}

/// Minimal URL encoding for range strings (handles `!`, spaces, `:`).
fn urlencoding(s: &str) -> String {
    // Range strings like "Sheet1!A1:B5" need the `!` encoded for URL path segments.
    // reqwest won't auto-encode path segments, so we do it manually.
    s.replace(' ', "%20")
        .replace('!', "%21")
        .replace('#', "%23")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encoding_preserves_colon() {
        let encoded = urlencoding("Sheet1!A1:B5");
        assert_eq!(encoded, "Sheet1%21A1:B5");
    }

    #[test]
    fn url_encoding_handles_spaces() {
        let encoded = urlencoding("My Sheet!A1:B5");
        assert_eq!(encoded, "My%20Sheet%21A1:B5");
    }

    #[test]
    fn sheets_base_url_format() {
        let id = "abc123";
        let url = format!("{SHEETS_BASE}/{id}");
        assert_eq!(url, "https://sheets.googleapis.com/v4/spreadsheets/abc123");
    }
}
