//! Parameter and response types for Google Sheets MCP tools.
//!
//! All param structs derive `Deserialize + JsonSchema` — never use raw `serde_json::Value`.
//! Tier: T2-C (μ Mapping + ∂ Boundary + ∃ Existence)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MCP Tool Param Structs
// ---------------------------------------------------------------------------

/// Param for tools that only need a spreadsheet ID.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SpreadsheetIdParam {
    /// The Google Sheets spreadsheet ID (from the URL).
    pub spreadsheet_id: String,
}

/// Read a single range of cells.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadRangeParam {
    /// The Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range, e.g. `"Sheet1!A1:C10"`.
    pub range: String,
}

/// Write values to a cell range.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WriteRangeParam {
    /// The Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range, e.g. `"Sheet1!A1:C10"`.
    pub range: String,
    /// Row-major 2D array of cell values.
    pub values: Vec<Vec<String>>,
}

/// Read multiple ranges in a single call.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BatchReadParam {
    /// The Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// List of A1 notation ranges.
    pub ranges: Vec<String>,
}

/// Append rows to the end of a range.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AppendParam {
    /// The Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range (table anchor), e.g. `"Sheet1!A1"`.
    pub range: String,
    /// Row-major 2D array of values to append.
    pub values: Vec<Vec<String>>,
}

/// Search cells for a value.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchParam {
    /// The Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// Substring to search for (case-insensitive).
    pub query: String,
    /// Optional A1 notation range to limit search scope.
    pub range: Option<String>,
}

// ---------------------------------------------------------------------------
// Google Sheets API v4 Response Types (subset)
// ---------------------------------------------------------------------------

/// Top-level spreadsheet metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetMeta {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    pub properties: SpreadsheetProperties,
    pub sheets: Vec<SheetEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetProperties {
    pub title: String,
    pub locale: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetEntry {
    pub properties: SheetProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetProperties {
    #[serde(rename = "sheetId")]
    pub sheet_id: u64,
    pub title: String,
    pub index: u32,
    #[serde(rename = "sheetType")]
    pub sheet_type: Option<String>,
}

/// Response from `values.get` / `values.batchGet`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    pub range: Option<String>,
    #[serde(rename = "majorDimension", default)]
    pub major_dimension: Option<String>,
    #[serde(default)]
    pub values: Vec<Vec<serde_json::Value>>,
}

/// Wrapper for `values:batchGet` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "valueRanges", default)]
    pub value_ranges: Vec<ValueRange>,
}

/// Response from `values.update` / `values.append`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "updatedRange")]
    pub updated_range: Option<String>,
    #[serde(rename = "updatedRows")]
    pub updated_rows: Option<u32>,
    #[serde(rename = "updatedColumns")]
    pub updated_columns: Option<u32>,
    #[serde(rename = "updatedCells")]
    pub updated_cells: Option<u32>,
}

/// Wrapper for append response (has an extra layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "tableRange")]
    pub table_range: Option<String>,
    pub updates: Option<UpdateResponse>,
}

// ---------------------------------------------------------------------------
// Service Account Key (from GCP JSON key file)
// ---------------------------------------------------------------------------

/// Service account JSON key format from GCP console.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    pub r#type: Option<String>,
    pub project_id: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: String,
    pub client_email: String,
    pub client_id: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_read_range_param() {
        let json = r#"{"spreadsheet_id":"abc123","range":"Sheet1!A1:B5"}"#;
        let param: ReadRangeParam = serde_json::from_str(json).expect("deserialize ReadRangeParam");
        assert_eq!(param.spreadsheet_id, "abc123");
        assert_eq!(param.range, "Sheet1!A1:B5");
    }

    #[test]
    fn deserialize_service_account_key() {
        let json = r#"{
            "type": "service_account",
            "project_id": "test",
            "private_key_id": "key1",
            "private_key": "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----\n",
            "client_email": "test@test.iam.gserviceaccount.com",
            "client_id": "123"
        }"#;
        let key: ServiceAccountKey =
            serde_json::from_str(json).expect("deserialize ServiceAccountKey");
        assert_eq!(key.client_email, "test@test.iam.gserviceaccount.com");
        assert!(key.private_key.contains("RSA PRIVATE KEY"));
    }

    #[test]
    fn deserialize_spreadsheet_meta() {
        let json = r#"{
            "spreadsheetId": "abc",
            "properties": {"title": "Test Sheet"},
            "sheets": [
                {"properties": {"sheetId": 0, "title": "Sheet1", "index": 0}}
            ]
        }"#;
        let meta: SpreadsheetMeta =
            serde_json::from_str(json).expect("deserialize SpreadsheetMeta");
        assert_eq!(meta.properties.title, "Test Sheet");
        assert_eq!(meta.sheets.len(), 1);
        assert_eq!(meta.sheets[0].properties.title, "Sheet1");
    }

    #[test]
    fn deserialize_value_range_with_empty_values() {
        let json = r#"{"range": "Sheet1!A1:B2"}"#;
        let vr: ValueRange = serde_json::from_str(json).expect("deserialize ValueRange");
        assert!(vr.values.is_empty());
    }

    #[test]
    fn search_param_optional_range() {
        let json = r#"{"spreadsheet_id":"abc","query":"hello"}"#;
        let param: SearchParam = serde_json::from_str(json).expect("deserialize SearchParam");
        assert!(param.range.is_none());
    }
}
