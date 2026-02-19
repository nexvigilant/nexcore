//! Browser event source for Vigil
//!
//! Subscribes to browser events from nexcore-browser and maps them
//! to Vigil events for the EventBus.

use async_trait::async_trait;
use nexcore_browser::events::BrowserEvent;
use nexcore_browser::state::subscribe_events;
use tracing::{debug, info, warn};

use crate::events::EventBus;
use crate::models::{Event, Urgency};
use crate::sources::Source;

/// Browser event source
///
/// Subscribes to nexcore-browser broadcast channel and emits
/// Vigil events for console errors, network failures, page loads, etc.
pub struct BrowserSource {
    event_bus: EventBus,
}

impl BrowserSource {
    /// Create a new browser source
    #[must_use]
    pub fn new(event_bus: EventBus) -> Self {
        Self { event_bus }
    }

    /// Map browser event to Vigil event
    fn map_event(browser_event: &BrowserEvent) -> Event {
        let (event_type, priority, payload) = match browser_event {
            BrowserEvent::ConsoleMessage {
                level,
                text,
                url,
                page_id,
                timestamp,
                ..
            } => {
                let priority = match level.as_str() {
                    "error" => Urgency::High,
                    "warning" => Urgency::Normal,
                    _ => Urgency::Low,
                };
                let event_type = match level.as_str() {
                    "error" => "console_error",
                    "warning" => "console_warning",
                    _ => "console_log",
                };
                let payload = serde_json::json!({
                    "level": level,
                    "text": text,
                    "url": url,
                    "page_id": page_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
                (event_type, priority, payload)
            }
            BrowserEvent::NetworkFailure {
                url,
                method,
                error,
                page_id,
                timestamp,
                ..
            } => {
                let payload = serde_json::json!({
                    "url": url,
                    "method": method,
                    "error": error,
                    "page_id": page_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
                ("network_failure", Urgency::High, payload)
            }
            BrowserEvent::NetworkComplete {
                url,
                method,
                status,
                size,
                duration_ms,
                page_id,
                timestamp,
                ..
            } => {
                let priority = if *status >= 400 {
                    Urgency::Normal
                } else {
                    Urgency::Low
                };
                let payload = serde_json::json!({
                    "url": url,
                    "method": method,
                    "status": status,
                    "size": size,
                    "duration_ms": duration_ms,
                    "page_id": page_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
                ("network_complete", priority, payload)
            }
            BrowserEvent::PageLoaded {
                url,
                title,
                load_time_ms,
                page_id,
                timestamp,
            } => {
                let payload = serde_json::json!({
                    "url": url,
                    "title": title,
                    "load_time_ms": load_time_ms,
                    "page_id": page_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
                ("page_loaded", Urgency::Low, payload)
            }
            BrowserEvent::PerformanceViolation {
                violation_type,
                value,
                threshold,
                page_id,
                timestamp,
            } => {
                let payload = serde_json::json!({
                    "violation_type": violation_type,
                    "value": value,
                    "threshold": threshold,
                    "page_id": page_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
                ("performance_violation", Urgency::Normal, payload)
            }
            BrowserEvent::PageCrashed {
                page_id,
                error,
                timestamp,
            } => {
                let payload = serde_json::json!({
                    "page_id": page_id,
                    "error": error,
                    "timestamp": timestamp.to_rfc3339(),
                });
                ("page_crashed", Urgency::Critical, payload)
            }
            BrowserEvent::BrowserDisconnected { reason, timestamp } => {
                let payload = serde_json::json!({
                    "reason": reason,
                    "timestamp": timestamp.to_rfc3339(),
                });
                ("browser_disconnected", Urgency::High, payload)
            }
        };

        Event {
            source: "browser".to_string(),
            event_type: event_type.to_string(),
            payload,
            priority,
            ..Default::default()
        }
    }
}

#[async_trait]
impl Source for BrowserSource {
    async fn run(&self) -> anyhow::Result<()> {
        info!("BrowserSource starting - subscribing to browser events");

        let mut rx = subscribe_events();
        let mut lagged_count: u64 = 0;

        loop {
            match rx.recv().await {
                Ok(browser_event) => {
                    let vigil_event = Self::map_event(&browser_event);
                    debug!(
                        event_type = %vigil_event.event_type,
                        priority = ?vigil_event.priority,
                        "Browser event received"
                    );

                    // Emit to EventBus (logs internally on backpressure)
                    self.event_bus.emit(vigil_event).await;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    lagged_count += n;
                    warn!(
                        lagged = n,
                        total_lagged = lagged_count,
                        "BrowserSource lagged behind - some events dropped"
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    info!("Browser event channel closed - BrowserSource stopping");
                    break;
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "browser"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_map_console_error() {
        let browser_event = BrowserEvent::ConsoleMessage {
            level: "error".to_string(),
            text: "Test error".to_string(),
            url: Some("https://example.com".to_string()),
            line: Some(10),
            column: Some(5),
            timestamp: Utc::now(),
            page_id: "page_1".to_string(),
        };

        let vigil_event = BrowserSource::map_event(&browser_event);
        assert_eq!(vigil_event.event_type, "console_error");
        assert_eq!(vigil_event.priority, Urgency::High);
        assert_eq!(vigil_event.source, "browser");
    }

    #[test]
    fn test_map_network_failure() {
        let browser_event = BrowserEvent::NetworkFailure {
            url: "https://api.example.com/data".to_string(),
            method: "GET".to_string(),
            error: "Connection refused".to_string(),
            request_id: "req_1".to_string(),
            timestamp: Utc::now(),
            page_id: "page_1".to_string(),
        };

        let vigil_event = BrowserSource::map_event(&browser_event);
        assert_eq!(vigil_event.event_type, "network_failure");
        assert_eq!(vigil_event.priority, Urgency::High);
    }

    #[test]
    fn test_map_page_crashed() {
        let browser_event = BrowserEvent::PageCrashed {
            page_id: "page_1".to_string(),
            error: Some("Out of memory".to_string()),
            timestamp: Utc::now(),
        };

        let vigil_event = BrowserSource::map_event(&browser_event);
        assert_eq!(vigil_event.event_type, "page_crashed");
        assert_eq!(vigil_event.priority, Urgency::Critical);
    }
}
