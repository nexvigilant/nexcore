//! CDP event handlers for routing browser events
//!
//! Routes Chrome DevTools Protocol events to collectors and broadcasts
//! them for Vigil integration.

use nexcore_chrono::DateTime;
use tokio::sync::broadcast;
use tracing::{debug, warn};

use crate::collectors::console::{ConsoleEntry, ConsoleLevel, get_console_collector};
use crate::collectors::network::{NetworkEntry, NetworkStatus, get_network_collector};
use crate::events::BrowserEvent;

/// Route a console message event to collectors and broadcast
pub fn handle_console_message(
    level: &str,
    text: String,
    url: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
    page_id: &str,
    tx: &broadcast::Sender<BrowserEvent>,
) {
    let timestamp = DateTime::now();
    let level_enum = ConsoleLevel::parse_cdp(level);

    // Add to collector
    let entry = ConsoleEntry {
        level: level_enum,
        text: text.clone(),
        url: url.clone(),
        line,
        column,
        timestamp,
        page_id: page_id.to_string(),
    };
    get_console_collector().push(entry);

    // Broadcast event
    let event = BrowserEvent::ConsoleMessage {
        level: level.to_string(),
        text,
        url,
        line,
        column,
        timestamp,
        page_id: page_id.to_string(),
    };

    if let Err(e) = tx.send(event) {
        debug!("No subscribers for browser event: {e}");
    }
}

/// Route a network request start event
pub fn handle_network_request_start(
    request_id: String,
    url: String,
    method: String,
    resource_type: Option<String>,
    page_id: &str,
    _tx: &broadcast::Sender<BrowserEvent>,
) {
    let entry = NetworkEntry {
        request_id,
        url,
        method,
        status: NetworkStatus::Pending,
        status_code: None,
        response_size: 0,
        duration_ms: None,
        error: None,
        resource_type,
        started_at: DateTime::now(),
        completed_at: None,
        page_id: page_id.to_string(),
    };
    get_network_collector().push(entry);
}

/// Route a network request completion event
pub fn handle_network_request_complete(
    request_id: &str,
    status_code: u16,
    response_size: u64,
    page_id: &str,
    tx: &broadcast::Sender<BrowserEvent>,
) {
    let timestamp = DateTime::now();
    let collector = get_network_collector();

    // Get the original entry for URL/method info
    let original = collector.get_by_id(request_id);

    collector.update(request_id, |entry| {
        entry.status = NetworkStatus::from_status_code(status_code);
        entry.status_code = Some(status_code);
        entry.response_size = response_size;
        entry.completed_at = Some(timestamp);

        // Calculate duration
        let duration = timestamp.signed_duration_since(entry.started_at);
        entry.duration_ms = Some(duration.num_milliseconds().max(0) as u64);
    });

    // Broadcast completion event
    if let Some(orig) = original {
        let duration_ms = timestamp
            .signed_duration_since(orig.started_at)
            .num_milliseconds()
            .max(0) as u64;

        let event = BrowserEvent::NetworkComplete {
            url: orig.url,
            method: orig.method,
            status: status_code,
            size: response_size,
            duration_ms,
            request_id: request_id.to_string(),
            timestamp,
            page_id: page_id.to_string(),
        };

        if let Err(e) = tx.send(event) {
            debug!("No subscribers for browser event: {e}");
        }
    }
}

/// Route a network request failure event
pub fn handle_network_request_failed(
    request_id: &str,
    error: String,
    page_id: &str,
    tx: &broadcast::Sender<BrowserEvent>,
) {
    let timestamp = DateTime::now();
    let collector = get_network_collector();

    // Get the original entry for URL/method info
    let original = collector.get_by_id(request_id);

    collector.update(request_id, |entry| {
        entry.status = NetworkStatus::Failed;
        entry.error = Some(error.clone());
        entry.completed_at = Some(timestamp);
    });

    // Broadcast failure event
    if let Some(orig) = original {
        let event = BrowserEvent::NetworkFailure {
            url: orig.url,
            method: orig.method,
            error,
            request_id: request_id.to_string(),
            timestamp,
            page_id: page_id.to_string(),
        };

        if let Err(e) = tx.send(event) {
            debug!("No subscribers for browser event: {e}");
        }
    } else {
        warn!("Network failure for unknown request: {request_id}");
    }
}

/// Route a page load event
pub fn handle_page_loaded(
    url: String,
    title: Option<String>,
    load_time_ms: Option<u64>,
    page_id: &str,
    tx: &broadcast::Sender<BrowserEvent>,
) {
    let event = BrowserEvent::PageLoaded {
        url,
        title,
        load_time_ms,
        page_id: page_id.to_string(),
        timestamp: DateTime::now(),
    };

    if let Err(e) = tx.send(event) {
        debug!("No subscribers for browser event: {e}");
    }
}

/// Route a page crash event
pub fn handle_page_crashed(
    page_id: &str,
    error: Option<String>,
    tx: &broadcast::Sender<BrowserEvent>,
) {
    let event = BrowserEvent::PageCrashed {
        page_id: page_id.to_string(),
        error,
        timestamp: DateTime::now(),
    };

    if let Err(e) = tx.send(event) {
        debug!("No subscribers for browser event: {e}");
    }
}

/// Route a browser disconnection event
pub fn handle_browser_disconnected(reason: String, tx: &broadcast::Sender<BrowserEvent>) {
    let event = BrowserEvent::BrowserDisconnected {
        reason,
        timestamp: DateTime::now(),
    };

    if let Err(e) = tx.send(event) {
        debug!("No subscribers for browser event: {e}");
    }
}
