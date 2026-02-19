use chrono::{DateTime, Utc};
use std::time::Duration;

/// Events emitted by the cloud platform during operation.
///
/// Tier: T2-C (→ Causality + ς State + ν Frequency)
/// Each event captures a causal state transition observed at a point in time.
#[derive(Debug, Clone)]
pub enum CloudEvent {
    /// A service process was spawned successfully.
    ServiceStarted {
        name: String,
        pid: u32,
        at: DateTime<Utc>,
    },

    /// A service passed its health check.
    ServiceHealthy { name: String, at: DateTime<Utc> },

    /// A service failed its health check.
    HealthCheckFailed {
        name: String,
        reason: String,
        at: DateTime<Utc>,
    },

    /// A service process exited unexpectedly.
    ServiceCrashed {
        name: String,
        exit_code: Option<i32>,
        at: DateTime<Utc>,
    },

    /// A restart has been scheduled after a crash.
    RestartScheduled {
        name: String,
        attempt: u32,
        backoff: Duration,
        at: DateTime<Utc>,
    },

    /// A service was stopped cleanly.
    ServiceStopped { name: String, at: DateTime<Utc> },

    /// The reverse proxy handled a request.
    ProxyRequest {
        route: String,
        backend: String,
        status: u16,
        latency_ms: u64,
    },

    /// Platform-level lifecycle event.
    PlatformStarted {
        name: String,
        services: usize,
        at: DateTime<Utc>,
    },

    /// Platform shutdown initiated.
    PlatformStopping { at: DateTime<Utc> },
}

/// Simple broadcast channel for cloud events.
///
/// Tier: T2-P (μ Mapping) — maps events to listeners.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: tokio::sync::broadcast::Sender<CloudEvent>,
}

impl EventBus {
    /// Create a new event bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: CloudEvent) {
        // Ignore send errors (no receivers is fine during startup/shutdown)
        let _ = self.sender.send(event);
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CloudEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_new() {
        let bus = EventBus::new(64);
        // Bus should be functional
        bus.emit(CloudEvent::PlatformStopping {
            at: chrono::Utc::now(),
        });
    }

    #[test]
    fn event_bus_default() {
        let bus = EventBus::default();
        // Should not panic
        bus.emit(CloudEvent::PlatformStopping {
            at: chrono::Utc::now(),
        });
    }

    #[test]
    fn event_bus_subscribe_receives() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.emit(CloudEvent::ServiceStopped {
            name: "web".to_string(),
            at: chrono::Utc::now(),
        });

        let event = rx.try_recv();
        assert!(event.is_ok());
        if let Ok(CloudEvent::ServiceStopped { name, .. }) = event {
            assert_eq!(name, "web");
        } else {
            panic!("expected ServiceStopped event");
        }
    }

    #[test]
    fn event_bus_emit_without_subscribers() {
        // Should not panic even with no subscribers
        let bus = EventBus::new(8);
        bus.emit(CloudEvent::PlatformStarted {
            name: "test".to_string(),
            services: 0,
            at: chrono::Utc::now(),
        });
    }

    #[test]
    fn event_bus_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.emit(CloudEvent::ServiceHealthy {
            name: "api".to_string(),
            at: chrono::Utc::now(),
        });

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }

    #[test]
    fn cloud_event_debug() {
        let events = vec![
            CloudEvent::ServiceStarted {
                name: "a".to_string(),
                pid: 123,
                at: chrono::Utc::now(),
            },
            CloudEvent::ServiceHealthy {
                name: "b".to_string(),
                at: chrono::Utc::now(),
            },
            CloudEvent::HealthCheckFailed {
                name: "c".to_string(),
                reason: "timeout".to_string(),
                at: chrono::Utc::now(),
            },
            CloudEvent::ServiceCrashed {
                name: "d".to_string(),
                exit_code: Some(1),
                at: chrono::Utc::now(),
            },
            CloudEvent::ServiceCrashed {
                name: "e".to_string(),
                exit_code: None,
                at: chrono::Utc::now(),
            },
            CloudEvent::RestartScheduled {
                name: "f".to_string(),
                attempt: 3,
                backoff: Duration::from_secs(4),
                at: chrono::Utc::now(),
            },
            CloudEvent::ServiceStopped {
                name: "g".to_string(),
                at: chrono::Utc::now(),
            },
            CloudEvent::ProxyRequest {
                route: "/api".to_string(),
                backend: "api".to_string(),
                status: 200,
                latency_ms: 5,
            },
            CloudEvent::PlatformStarted {
                name: "prod".to_string(),
                services: 3,
                at: chrono::Utc::now(),
            },
            CloudEvent::PlatformStopping {
                at: chrono::Utc::now(),
            },
        ];

        // All variants should produce non-empty debug output
        for event in events {
            let debug = format!("{event:?}");
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn cloud_event_clone() {
        let event = CloudEvent::ProxyRequest {
            route: "/test".to_string(),
            backend: "web".to_string(),
            status: 404,
            latency_ms: 12,
        };
        let cloned = event.clone();
        if let CloudEvent::ProxyRequest { status, .. } = cloned {
            assert_eq!(status, 404);
        } else {
            panic!("clone should preserve variant");
        }
    }
}
