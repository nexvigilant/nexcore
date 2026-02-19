//! Discovery protocol: neighbor announcement and detection.
//!
//! ## Primitive Foundation
//! - `DiscoveryMessage`: Σ (Sum) — announce/response variants
//! - `DiscoveryConfig`: ∂ (Boundary) + ν (Frequency)
//! - Discovery loop: ν (Frequency) dominant — periodic announce/listen

use crate::node::{MeshMessage, MessageKind};
use crate::topology::RouteQuality;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// DiscoveryMessage — Announce/Response protocol
// ============================================================================

/// Discovery protocol messages.
///
/// Tier: T2-P | Dominant: Σ (Sum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    /// Periodic announcement: "I exist, here are my capabilities"
    Announce {
        /// Announcing node ID
        node_id: String,
        /// Number of current neighbors
        neighbor_count: usize,
        /// Available capacity for new neighbors
        available_capacity: usize,
    },
    /// Response to an announcement: "I want to connect"
    Response {
        /// Responding node ID
        node_id: String,
        /// Proposed link quality estimate
        estimated_quality: RouteQuality,
    },
}

impl DiscoveryMessage {
    /// Create an announce message.
    pub fn announce(node_id: impl Into<String>, neighbor_count: usize, capacity: usize) -> Self {
        Self::Announce {
            node_id: node_id.into(),
            neighbor_count,
            available_capacity: capacity,
        }
    }

    /// Create a response message.
    pub fn response(node_id: impl Into<String>, quality: RouteQuality) -> Self {
        Self::Response {
            node_id: node_id.into(),
            estimated_quality: quality,
        }
    }

    /// Convert to a MeshMessage for transmission.
    pub fn to_mesh_message(&self, source: &str, max_ttl: u8) -> Option<MeshMessage> {
        let payload = serde_json::to_value(self).ok()?;
        Some(MeshMessage::broadcast(
            source,
            MessageKind::Discovery,
            payload,
            max_ttl,
        ))
    }

    /// Parse a DiscoveryMessage from a MeshMessage payload.
    pub fn from_mesh_message(msg: &MeshMessage) -> Option<Self> {
        if msg.kind != MessageKind::Discovery {
            return None;
        }
        serde_json::from_value(msg.payload.clone()).ok()
    }
}

// ============================================================================
// DiscoveryConfig — Configuration for the discovery loop
// ============================================================================

/// Configuration for the discovery protocol.
///
/// Tier: T2-C | Dominant: ν (Frequency)
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Interval between announce broadcasts
    pub announce_interval: Duration,
    /// TTL for discovery messages (limited range)
    pub discovery_ttl: u8,
    /// Minimum interval between announces (rate limit)
    pub min_announce_interval: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            announce_interval: Duration::from_secs(5),
            discovery_ttl: 3,
            min_announce_interval: Duration::from_secs(1),
        }
    }
}

// ============================================================================
// DiscoveryLoop — Async discovery state machine
// ============================================================================

/// State machine for the discovery protocol loop.
///
/// Tier: T3 | Dominant: ν (Frequency)
///
/// Periodically broadcasts announce messages and processes responses
/// to discover and connect with new neighbors.
#[derive(Debug)]
pub struct DiscoveryLoop {
    /// Node ID of the owning node
    pub node_id: String,
    /// Discovery configuration
    pub config: DiscoveryConfig,
    /// Number of announces sent
    pub announces_sent: u64,
    /// Number of responses received
    pub responses_received: u64,
    /// Whether the loop is running
    pub running: bool,
}

impl DiscoveryLoop {
    /// Create a new discovery loop.
    pub fn new(node_id: impl Into<String>, config: DiscoveryConfig) -> Self {
        Self {
            node_id: node_id.into(),
            config,
            announces_sent: 0,
            responses_received: 0,
            running: false,
        }
    }

    /// Generate an announce message based on current state.
    pub fn generate_announce(
        &mut self,
        neighbor_count: usize,
        available_capacity: usize,
    ) -> DiscoveryMessage {
        self.announces_sent += 1;
        DiscoveryMessage::announce(&self.node_id, neighbor_count, available_capacity)
    }

    /// Process a received discovery message.
    pub fn process_message(&mut self, msg: &DiscoveryMessage) -> DiscoveryAction {
        match msg {
            DiscoveryMessage::Announce {
                node_id,
                available_capacity,
                ..
            } => {
                if node_id == &self.node_id {
                    return DiscoveryAction::Ignore;
                }
                if *available_capacity == 0 {
                    return DiscoveryAction::Ignore;
                }
                // Respond with default quality estimate
                DiscoveryAction::RespondTo(node_id.clone())
            }
            DiscoveryMessage::Response { node_id, .. } => {
                if node_id == &self.node_id {
                    return DiscoveryAction::Ignore;
                }
                self.responses_received += 1;
                DiscoveryAction::AddNeighbor(node_id.clone())
            }
        }
    }

    /// Start the discovery loop.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the discovery loop.
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// Action resulting from processing a discovery message.
#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryAction {
    /// Ignore this message (self-message or no capacity)
    Ignore,
    /// Send a response to the given node
    RespondTo(String),
    /// Add the given node as a neighbor
    AddNeighbor(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_announce_creates_correctly() {
        let msg = DiscoveryMessage::announce("node-1", 3, 5);
        match msg {
            DiscoveryMessage::Announce {
                node_id,
                neighbor_count,
                available_capacity,
            } => {
                assert_eq!(node_id, "node-1");
                assert_eq!(neighbor_count, 3);
                assert_eq!(available_capacity, 5);
            }
            _ => panic!("expected Announce"),
        }
    }

    #[test]
    fn discovery_response_creates_correctly() {
        let quality = RouteQuality::new(50.0, 0.95, 1);
        let msg = DiscoveryMessage::response("node-2", quality);
        match msg {
            DiscoveryMessage::Response { node_id, .. } => {
                assert_eq!(node_id, "node-2");
            }
            _ => panic!("expected Response"),
        }
    }

    #[test]
    fn discovery_to_mesh_message_roundtrip() {
        let disc = DiscoveryMessage::announce("node-1", 2, 6);
        let mesh_msg = disc.to_mesh_message("node-1", 3);
        assert!(mesh_msg.is_some());
        let mesh_msg = mesh_msg.as_ref();
        assert_eq!(mesh_msg.map(|m| &m.kind), Some(&MessageKind::Discovery));
        assert!(mesh_msg.map(|m| m.is_broadcast()).unwrap_or(false));

        let parsed = DiscoveryMessage::from_mesh_message(
            mesh_msg
                .as_ref()
                .unwrap_or_else(|| panic!("expected mesh message")),
        );
        assert!(parsed.is_some());
    }

    #[test]
    fn discovery_from_wrong_kind_returns_none() {
        let msg = MeshMessage::data("src", "dst", serde_json::json!(null), 10);
        assert!(DiscoveryMessage::from_mesh_message(&msg).is_none());
    }

    #[test]
    fn discovery_loop_generate_announce() {
        let mut dl = DiscoveryLoop::new("node-1", DiscoveryConfig::default());
        let ann = dl.generate_announce(3, 5);
        assert_eq!(dl.announces_sent, 1);
        match ann {
            DiscoveryMessage::Announce {
                neighbor_count,
                available_capacity,
                ..
            } => {
                assert_eq!(neighbor_count, 3);
                assert_eq!(available_capacity, 5);
            }
            _ => panic!("expected Announce"),
        }
    }

    #[test]
    fn discovery_loop_process_announce_from_other() {
        let mut dl = DiscoveryLoop::new("node-1", DiscoveryConfig::default());
        let msg = DiscoveryMessage::announce("node-2", 1, 5);
        let action = dl.process_message(&msg);
        assert_eq!(action, DiscoveryAction::RespondTo("node-2".to_string()));
    }

    #[test]
    fn discovery_loop_process_announce_from_self() {
        let mut dl = DiscoveryLoop::new("node-1", DiscoveryConfig::default());
        let msg = DiscoveryMessage::announce("node-1", 1, 5);
        let action = dl.process_message(&msg);
        assert_eq!(action, DiscoveryAction::Ignore);
    }

    #[test]
    fn discovery_loop_process_announce_no_capacity() {
        let mut dl = DiscoveryLoop::new("node-1", DiscoveryConfig::default());
        let msg = DiscoveryMessage::announce("node-2", 10, 0);
        let action = dl.process_message(&msg);
        assert_eq!(action, DiscoveryAction::Ignore);
    }

    #[test]
    fn discovery_loop_process_response() {
        let mut dl = DiscoveryLoop::new("node-1", DiscoveryConfig::default());
        let msg = DiscoveryMessage::response("node-2", RouteQuality::new(50.0, 0.9, 1));
        let action = dl.process_message(&msg);
        assert_eq!(action, DiscoveryAction::AddNeighbor("node-2".to_string()));
        assert_eq!(dl.responses_received, 1);
    }

    #[test]
    fn discovery_loop_start_stop() {
        let mut dl = DiscoveryLoop::new("node-1", DiscoveryConfig::default());
        assert!(!dl.running);
        dl.start();
        assert!(dl.running);
        dl.stop();
        assert!(!dl.running);
    }

    #[test]
    fn discovery_config_default() {
        let cfg = DiscoveryConfig::default();
        assert_eq!(cfg.announce_interval, Duration::from_secs(5));
        assert_eq!(cfg.discovery_ttl, 3);
    }
}
