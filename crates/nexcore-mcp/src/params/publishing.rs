//! Parameters for publishing MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Publish a DOCX manuscript to EPUB.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PublishDocxParams {
    /// Path to the input .docx file.
    pub input_path: String,
    /// Directory to write the output .epub file.
    pub output_dir: String,
    /// Book title (defaults to DOCX embedded title).
    pub title: Option<String>,
    /// Author name (defaults to DOCX embedded author).
    pub author: Option<String>,
    /// Language code (default: "en").
    pub language: Option<String>,
    /// Book description / blurb.
    pub description: Option<String>,
    /// Publishing target: "epub", "kindle", or "both" (default: "epub").
    pub target: Option<String>,
    /// Path to cover image (optional, JPEG or PNG).
    pub cover_path: Option<String>,
}

/// Read and extract content from an existing EPUB file.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadEpubParams {
    /// Path to the .epub file.
    pub path: String,
    /// Chapter index to read (0-based). If omitted, returns metadata and TOC only.
    pub chapter: Option<usize>,
    /// Output format: "text" (plain text) or "html" (default: "text").
    pub format: Option<String>,
}
