//! Browser & Chrome DevTools Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Navigation, input, debugging, network, emulation, and performance profiling.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for creating a new browser page
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserNewPageParams {
    /// URL to navigate to
    pub url: String,
    /// Whether to wait for page load (default: true)
    #[serde(default = "default_true")]
    pub wait_for_load: bool,
}

fn default_true() -> bool {
    true
}

/// Parameters for navigating the current page
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserNavigateParams {
    /// Navigation type: "url", "back", "forward", or "reload"
    #[serde(default = "default_nav_type")]
    pub nav_type: String,
    /// Target URL (required when nav_type is "url")
    #[serde(default)]
    pub url: Option<String>,
    /// Whether to ignore cache on reload
    #[serde(default)]
    pub ignore_cache: bool,
    /// Timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_nav_type() -> String {
    "url".to_string()
}

fn default_timeout_ms() -> u64 {
    30000
}

/// Parameters for selecting a page by ID
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserSelectPageParams {
    /// Page ID to select (from list_pages)
    pub page_id: String,
    /// Whether to bring the page to front
    #[serde(default)]
    pub bring_to_front: bool,
}

/// Parameters for closing a page
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserClosePageParams {
    /// Page ID to close (from list_pages)
    pub page_id: String,
}

/// Parameters for listing pages
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserListPagesParams {}

/// Parameters for waiting for text on page
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserWaitForParams {
    /// Text to wait for on the page
    pub text: String,
    /// Timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

/// Parameters for clicking an element
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserClickParams {
    /// CSS selector for the element to click
    pub selector: String,
    /// Whether to double-click (default: false)
    #[serde(default)]
    pub double_click: bool,
    /// Click at specific coordinates instead of selector
    #[serde(default)]
    pub x: Option<i32>,
    /// Y coordinate (requires x)
    #[serde(default)]
    pub y: Option<i32>,
}

/// Parameters for hovering over an element
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserHoverParams {
    /// CSS selector for the element to hover
    pub selector: String,
}

/// Parameters for filling an input field
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserFillParams {
    /// CSS selector for the input element
    pub selector: String,
    /// Value to fill in
    pub value: String,
}

/// A single form field to fill
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserFormField {
    /// CSS selector for the input element
    pub selector: String,
    /// Value to fill in
    pub value: String,
}

/// Parameters for filling multiple form fields at once
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserFillFormParams {
    /// Array of {selector, value} pairs
    pub fields: Vec<BrowserFormField>,
}

/// Parameters for dragging an element
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserDragParams {
    /// CSS selector for the element to drag
    pub from_selector: String,
    /// CSS selector for the drop target
    pub to_selector: String,
}

/// Parameters for pressing a key
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPressKeyParams {
    /// Key or key combination (e.g., "Enter", "Control+A", "Control+Shift+R")
    pub key: String,
}

/// Parameters for uploading a file
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserUploadFileParams {
    /// CSS selector for the file input element
    pub selector: String,
    /// Local file path to upload
    pub file_path: String,
}

/// Parameters for handling browser dialogs
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserHandleDialogParams {
    /// Action: "accept" or "dismiss"
    pub action: String,
    /// Optional prompt text (for prompt dialogs)
    #[serde(default)]
    pub prompt_text: Option<String>,
}

/// Parameters for taking a screenshot
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserScreenshotParams {
    /// Format: "png", "jpeg", or "webp" (default: "png")
    #[serde(default = "default_png")]
    pub format: String,
    /// Quality for JPEG/WebP (0-100, default: 80)
    #[serde(default = "default_quality")]
    pub quality: u8,
    /// CSS selector for element to screenshot
    #[serde(default)]
    pub selector: Option<String>,
    /// Capture full page instead of viewport
    #[serde(default)]
    pub full_page: bool,
}

fn default_png() -> String {
    "png".to_string()
}

fn default_quality() -> u8 {
    80
}

/// Parameters for taking a DOM snapshot
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserSnapshotParams {
    /// Include verbose accessibility tree information
    #[serde(default)]
    pub verbose: bool,
}

/// Parameters for evaluating JavaScript
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserEvaluateParams {
    /// JavaScript expression or function to evaluate
    pub expression: String,
    /// Arguments to pass to the function (as JSON)
    #[serde(default)]
    pub args: Option<serde_json::Value>,
}

/// Parameters for listing console messages
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserListConsoleMessagesParams {
    /// Maximum number of messages to return
    #[serde(default)]
    pub limit: Option<usize>,
    /// Filter by message type
    #[serde(default)]
    pub message_type: Option<String>,
}

/// Parameters for getting a specific console message
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserGetConsoleMessageParams {
    /// Message ID
    pub message_id: u32,
}

/// Parameters for listing network requests
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserListNetworkRequestsParams {
    /// Maximum number of requests to return
    #[serde(default)]
    pub limit: Option<usize>,
    /// Filter by resource type
    #[serde(default)]
    pub resource_type: Option<String>,
}

/// Parameters for getting a specific network request
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserGetNetworkRequestParams {
    /// Request ID
    pub request_id: u32,
}

/// Parameters for emulating device/network conditions
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserEmulateParams {
    /// Network condition
    #[serde(default)]
    pub network_condition: Option<String>,
    /// CPU throttling rate
    #[serde(default)]
    pub cpu_throttle: Option<u8>,
    /// Geolocation latitude
    #[serde(default)]
    pub geo_latitude: Option<f64>,
    /// Geolocation longitude
    #[serde(default)]
    pub geo_longitude: Option<f64>,
}

/// Parameters for resizing the page
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserResizePageParams {
    /// Page width in pixels
    pub width: u32,
    /// Page height in pixels
    pub height: u32,
}

/// Parameters for starting a performance trace
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPerfStartTraceParams {
    /// Reload the page after starting trace
    #[serde(default)]
    pub reload: bool,
    /// Automatically stop after ~5 seconds
    #[serde(default)]
    pub auto_stop: bool,
    /// File path to save raw trace data
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Parameters for stopping a performance trace
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPerfStopTraceParams {
    /// File path to save raw trace data
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Parameters for analyzing a performance insight
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPerfAnalyzeParams {
    /// Insight set ID from the trace results
    pub insight_set_id: String,
    /// Insight name
    pub insight_name: String,
}
