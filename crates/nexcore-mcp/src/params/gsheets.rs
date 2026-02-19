//! Google Sheets Parameters
//!
//! Reading, writing, appending, and searching in Google Sheets.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for gsheets tools that only need a spreadsheet ID.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsSpreadsheetIdParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
}

/// Parameters for gsheets_read_range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsReadRangeParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range.
    pub range: String,
}

/// Parameters for gsheets_batch_read.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsBatchReadParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// List of A1 notation ranges to read.
    pub ranges: Vec<String>,
}

/// Parameters for gsheets_write_range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsWriteRangeParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range to write to.
    pub range: String,
    /// 2D array of values to write.
    pub values: Vec<Vec<serde_json::Value>>,
}

/// Parameters for gsheets_append.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsAppendParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range to append to.
    pub range: String,
    /// 2D array of values to append.
    pub values: Vec<Vec<serde_json::Value>>,
}

/// Parameters for gsheets_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsSearchParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// Substring to search for.
    pub query: String,
    /// Optional range to search within.
    pub range: Option<String>,
}
