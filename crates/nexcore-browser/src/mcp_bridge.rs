//! Chrome DevTools MCP Bridge
//!
//! Bridges Chrome DevTools MCP tools to nexcore-browser collectors for Guardian sensing.
//! Tier: T2-C (Composed bridge between T3 domains)

use nexcore_chrono::DateTime;
use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, info};

use crate::collectors::console::{ConsoleEntry, ConsoleLevel, get_console_collector};
use crate::collectors::network::{NetworkEntry, NetworkStatus, get_network_collector};
use crate::events::BrowserEvent;

/// Bridge errors
#[derive(Debug, Error)]
pub enum McpBridgeError {
    /// JSON parsing failed
    #[error("Failed to parse MCP response: {0}")]
    ParseError(String),
    /// MCP tool returned an error
    #[error("MCP tool error: {0}")]
    McpError(String),
}

/// Console message from MCP `list_console_messages`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpConsoleMessage {
    #[serde(rename = "msgid")]
    pub msg_id: Option<u32>,
    #[serde(rename = "type")]
    pub level: String,
    pub text: String,
    pub url: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// Network request from MCP `list_network_requests`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpNetworkRequest {
    #[serde(rename = "reqid")]
    pub req_id: Option<u32>,
    pub url: String,
    pub method: String,
    pub status: Option<u16>,
    #[serde(rename = "resourceType")]
    pub resource_type: Option<String>,
    #[serde(rename = "encodedDataLength")]
    pub response_size: Option<u64>,
    pub duration: Option<f64>,
    pub error: Option<String>,
}

// ────────────────────────────────────────────────────────────────
// T1 Primitives: Pure parsing functions (σ - Sequence)
// ────────────────────────────────────────────────────────────────

fn parse_console_messages(json: &str) -> Result<Vec<McpConsoleMessage>, McpBridgeError> {
    #[derive(Deserialize)]
    struct Wrapped {
        messages: Vec<McpConsoleMessage>,
    }

    serde_json::from_str::<Wrapped>(json)
        .map(|w| w.messages)
        .or_else(|_| serde_json::from_str::<Vec<McpConsoleMessage>>(json))
        .map_err(|e: serde_json::Error| McpBridgeError::ParseError(e.to_string()))
}

fn parse_network_requests(json: &str) -> Result<Vec<McpNetworkRequest>, McpBridgeError> {
    #[derive(Deserialize)]
    struct Wrapped {
        requests: Vec<McpNetworkRequest>,
    }

    serde_json::from_str::<Wrapped>(json)
        .map(|w| w.requests)
        .or_else(|_| serde_json::from_str::<Vec<McpNetworkRequest>>(json))
        .map_err(|e: serde_json::Error| McpBridgeError::ParseError(e.to_string()))
}

// ────────────────────────────────────────────────────────────────
// T2-P Primitives: Type transformations (ρ - Mapping)
// ────────────────────────────────────────────────────────────────

fn mcp_to_console_entry(msg: McpConsoleMessage, page_id: &str) -> ConsoleEntry {
    ConsoleEntry {
        level: ConsoleLevel::parse_cdp(&msg.level),
        text: msg.text,
        url: msg.url,
        line: msg.line,
        column: msg.column,
        timestamp: DateTime::now(),
        page_id: page_id.to_string(),
    }
}

fn derive_network_status(req: &McpNetworkRequest) -> NetworkStatus {
    match (&req.status, &req.error) {
        (Some(code), _) => NetworkStatus::from_status_code(*code),
        (None, Some(_)) => NetworkStatus::Failed,
        (None, None) => NetworkStatus::Pending,
    }
}

fn mcp_to_network_entry(req: McpNetworkRequest, page_id: &str, idx: usize) -> NetworkEntry {
    let request_id = req
        .req_id
        .map_or_else(|| format!("mcp_{idx}"), |id| format!("mcp_{id}"));
    let status = derive_network_status(&req);
    let now = DateTime::now();

    NetworkEntry {
        request_id,
        url: req.url,
        method: req.method,
        status,
        status_code: req.status,
        response_size: req.response_size.unwrap_or(0),
        duration_ms: req.duration.map(|d| d as u64),
        error: req.error,
        resource_type: req.resource_type,
        started_at: now,
        completed_at: Some(now),
        page_id: page_id.to_string(),
    }
}

// ────────────────────────────────────────────────────────────────
// T2-C Composed: Sync functions
// ────────────────────────────────────────────────────────────────

/// Sync console messages from MCP JSON response
pub fn sync_console_from_json(json: &str, page_id: &str) -> Result<usize, McpBridgeError> {
    let messages = parse_console_messages(json)?;
    let collector = get_console_collector();
    let count = messages.len();

    for msg in messages {
        let entry = mcp_to_console_entry(msg, page_id);
        collector.push(entry);
    }

    debug!("Synced {count} console messages from MCP");
    Ok(count)
}

/// Sync network requests from MCP JSON response
pub fn sync_network_from_json(json: &str, page_id: &str) -> Result<usize, McpBridgeError> {
    let requests = parse_network_requests(json)?;
    let collector = get_network_collector();
    let count = requests.len();

    for (idx, req) in requests.into_iter().enumerate() {
        let entry = mcp_to_network_entry(req, page_id, idx);
        collector.push(entry);
    }

    debug!("Synced {count} network requests from MCP");
    Ok(count)
}

// ────────────────────────────────────────────────────────────────
// T3 Domain: MCP Bridge for Guardian integration
// ────────────────────────────────────────────────────────────────

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct McpBridgeConfig {
    pub default_page_id: String,
    pub clear_before_sync: bool,
}

impl Default for McpBridgeConfig {
    fn default() -> Self {
        Self {
            default_page_id: "mcp_page".to_string(),
            clear_before_sync: false,
        }
    }
}

/// MCP Bridge for continuous monitoring
pub struct McpBridge {
    config: McpBridgeConfig,
    event_tx: Option<broadcast::Sender<BrowserEvent>>,
    last_console_count: usize,
    last_network_count: usize,
}

impl McpBridge {
    /// Create a new MCP bridge
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: McpBridgeConfig::default(),
            event_tx: None,
            last_console_count: 0,
            last_network_count: 0,
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: McpBridgeConfig) -> Self {
        Self {
            config,
            event_tx: None,
            last_console_count: 0,
            last_network_count: 0,
        }
    }

    /// Set event broadcaster for Vigil integration
    #[must_use]
    pub fn with_event_tx(mut self, tx: broadcast::Sender<BrowserEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Sync console messages
    pub fn sync_console(&mut self, json: &str) -> Result<usize, McpBridgeError> {
        let page_id = self.config.default_page_id.clone();

        if self.config.clear_before_sync {
            get_console_collector().clear();
        }

        let total = sync_console_from_json(json, &page_id)?;
        let new_count = total.saturating_sub(self.last_console_count);
        self.last_console_count = total;

        self.broadcast_new_errors(new_count, &page_id);
        Ok(new_count)
    }

    /// Sync network requests
    pub fn sync_network(&mut self, json: &str) -> Result<usize, McpBridgeError> {
        let page_id = self.config.default_page_id.clone();

        if self.config.clear_before_sync {
            get_network_collector().clear();
        }

        let total = sync_network_from_json(json, &page_id)?;
        let new_count = total.saturating_sub(self.last_network_count);
        self.last_network_count = total;

        self.broadcast_new_failures(new_count, &page_id);
        Ok(new_count)
    }

    fn broadcast_new_errors(&self, count: usize, page_id: &str) {
        let Some(ref tx) = self.event_tx else { return };
        if count == 0 {
            return;
        }

        for error in get_console_collector()
            .get_errors()
            .iter()
            .rev()
            .take(count)
        {
            let event = BrowserEvent::ConsoleMessage {
                level: "error".to_string(),
                text: error.text.clone(),
                url: error.url.clone(),
                line: error.line,
                column: error.column,
                timestamp: error.timestamp,
                page_id: page_id.to_string(),
            };
            let _ = tx.send(event);
        }
    }

    fn broadcast_new_failures(&self, count: usize, page_id: &str) {
        let Some(ref tx) = self.event_tx else { return };
        if count == 0 {
            return;
        }

        for failure in get_network_collector()
            .get_failures()
            .iter()
            .rev()
            .take(count)
        {
            let event = BrowserEvent::NetworkFailure {
                url: failure.url.clone(),
                method: failure.method.clone(),
                error: failure.error.clone().unwrap_or_default(),
                request_id: failure.request_id.clone(),
                timestamp: failure.started_at,
                page_id: page_id.to_string(),
            };
            let _ = tx.send(event);
        }
    }

    /// Get sync statistics
    #[must_use]
    pub fn stats(&self) -> McpBridgeStats {
        let console = get_console_collector().stats();
        let network = get_network_collector().stats();

        McpBridgeStats {
            console_total: console.current_count,
            console_errors: console.error_count,
            console_warnings: console.warning_count,
            network_total: network.current_count,
            network_errors: network.error_count,
            network_failures: network.failure_count,
            network_failure_rate: get_network_collector().failure_rate(),
        }
    }

    /// Clear all collected data
    pub fn clear(&mut self) {
        get_console_collector().clear();
        get_network_collector().clear();
        self.last_console_count = 0;
        self.last_network_count = 0;
        info!("MCP bridge collectors cleared");
    }
}

impl Default for McpBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize)]
pub struct McpBridgeStats {
    pub console_total: usize,
    pub console_errors: usize,
    pub console_warnings: usize,
    pub network_total: usize,
    pub network_errors: usize,
    pub network_failures: usize,
    pub network_failure_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_console() {
        let json = r#"[{"type": "error", "text": "Error"}]"#;
        get_console_collector().clear();
        let count = sync_console_from_json(json, "test").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sync_network() {
        let json = r#"[{"url": "https://x.com", "method": "GET", "status": 200}]"#;
        get_network_collector().clear();
        let count = sync_network_from_json(json, "test").unwrap();
        assert_eq!(count, 1);
    }
}
