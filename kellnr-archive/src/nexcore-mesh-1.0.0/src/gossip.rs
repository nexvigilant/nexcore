//! Gossip protocol: route table propagation with loop prevention.
//!
//! ## Primitive Foundation
//! - `GossipMessage`: μ (Mapping) + σ (Sequence)
//! - `GossipLoop`: ν (Frequency) dominant — periodic route sharing
//! - Loop prevention: ∂ (Boundary) — origin tracking prevents infinite gossip

use crate::node::{MeshMessage, MessageKind};
use crate::routing::Route;
use crate::topology::RouteQuality;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

// ============================================================================
// GossipMessage — Route advertisement
// ============================================================================

/// A gossip message advertising known routes.
///
/// Tier: T2-C | Dominant: μ (Mapping)
///
/// Contains route advertisements from the sender's routing table.
/// Each entry includes the destination, quality, and hop count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    /// Sender node ID
    pub sender: String,
    /// Route advertisements
    pub routes: Vec<GossipRouteEntry>,
    /// Nodes that have already forwarded this gossip (loop prevention)
    pub visited: HashSet<String>,
}

/// A single route advertisement in a gossip message.
///
/// Tier: T2-P | Dominant: λ (Location)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipRouteEntry {
    /// Destination node ID
    pub destination: String,
    /// Route quality from sender's perspective
    pub quality: RouteQuality,
    /// Total hop count from sender to destination
    pub hop_count: u8,
}

impl GossipMessage {
    /// Create a new gossip message from the sender's routing table.
    pub fn from_routes(sender: impl Into<String>, routes: &[Route]) -> Self {
        let sender_id = sender.into();
        let entries: Vec<GossipRouteEntry> = routes
            .iter()
            .map(|r| GossipRouteEntry {
                destination: r.destination.clone(),
                quality: r.quality.clone(),
                hop_count: r.path.hop_count() as u8,
            })
            .collect();

        let mut visited = HashSet::new();
        visited.insert(sender_id.clone());

        Self {
            sender: sender_id,
            routes: entries,
            visited,
        }
    }

    /// Whether a node has already seen this gossip.
    pub fn has_visited(&self, node_id: &str) -> bool {
        self.visited.contains(node_id)
    }

    /// Mark a node as having forwarded this gossip.
    pub fn mark_visited(&mut self, node_id: impl Into<String>) {
        self.visited.insert(node_id.into());
    }

    /// Number of route entries.
    pub fn entry_count(&self) -> usize {
        self.routes.len()
    }

    /// Convert to a MeshMessage for transmission.
    pub fn to_mesh_message(&self, source: &str, max_ttl: u8) -> Option<MeshMessage> {
        let payload = serde_json::to_value(self).ok()?;
        Some(MeshMessage::broadcast(
            source,
            MessageKind::Gossip,
            payload,
            max_ttl,
        ))
    }

    /// Parse from a MeshMessage.
    pub fn from_mesh_message(msg: &MeshMessage) -> Option<Self> {
        if msg.kind != MessageKind::Gossip {
            return None;
        }
        serde_json::from_value(msg.payload.clone()).ok()
    }
}

// ============================================================================
// GossipConfig — Configuration for gossip protocol
// ============================================================================

/// Configuration for the gossip protocol.
///
/// Tier: T2-C | Dominant: ν (Frequency)
#[derive(Debug, Clone)]
pub struct GossipConfig {
    /// Interval between gossip rounds
    pub gossip_interval: Duration,
    /// Maximum route entries per gossip message
    pub max_entries_per_message: usize,
    /// TTL for gossip messages
    pub gossip_ttl: u8,
    /// Maximum gossip fan-out (neighbors to send to per round)
    pub max_fan_out: usize,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            gossip_interval: Duration::from_secs(10),
            max_entries_per_message: 20,
            gossip_ttl: 4,
            max_fan_out: 3,
        }
    }
}

// ============================================================================
// GossipLoop — Async gossip state machine
// ============================================================================

/// State machine for the gossip protocol loop.
///
/// Tier: T3 | Dominant: ν (Frequency)
///
/// Periodically shares routing table excerpts with neighbors
/// to propagate route knowledge through the mesh.
#[derive(Debug)]
pub struct GossipLoop {
    /// Node ID of the owning node
    pub node_id: String,
    /// Gossip configuration
    pub config: GossipConfig,
    /// Number of gossip rounds completed
    pub rounds_completed: u64,
    /// Total route entries gossiped
    pub entries_gossiped: u64,
    /// Total gossip messages received
    pub messages_received: u64,
    /// Whether the loop is running
    pub running: bool,
}

impl GossipLoop {
    /// Create a new gossip loop.
    pub fn new(node_id: impl Into<String>, config: GossipConfig) -> Self {
        Self {
            node_id: node_id.into(),
            config,
            rounds_completed: 0,
            entries_gossiped: 0,
            messages_received: 0,
            running: false,
        }
    }

    /// Generate a gossip message from current routing knowledge.
    pub fn generate_gossip(&mut self, routes: &[Route]) -> GossipMessage {
        let limited: Vec<Route> = routes
            .iter()
            .take(self.config.max_entries_per_message)
            .cloned()
            .collect();
        self.rounds_completed += 1;
        self.entries_gossiped += limited.len() as u64;
        GossipMessage::from_routes(&self.node_id, &limited)
    }

    /// Process a received gossip message. Returns routes to potentially add.
    pub fn process_gossip(&mut self, msg: &GossipMessage) -> Vec<GossipRouteEntry> {
        if msg.sender == self.node_id {
            return Vec::new();
        }
        self.messages_received += 1;

        // Filter out routes we are the destination for
        msg.routes
            .iter()
            .filter(|entry| entry.destination != self.node_id)
            .cloned()
            .collect()
    }

    /// Start the gossip loop.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the gossip loop.
    pub fn stop(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::Path;

    fn make_route(dest: &str, reliability: f64) -> Route {
        let mut path = Path::new("self", 16);
        let _ = path.add_hop(dest);
        Route::new(dest, dest, path, RouteQuality::new(50.0, reliability, 1))
    }

    #[test]
    fn gossip_message_from_routes() {
        let routes = vec![make_route("d1", 0.9), make_route("d2", 0.8)];
        let gm = GossipMessage::from_routes("node-1", &routes);
        assert_eq!(gm.sender, "node-1");
        assert_eq!(gm.entry_count(), 2);
        assert!(gm.has_visited("node-1"));
        assert!(!gm.has_visited("node-2"));
    }

    #[test]
    fn gossip_message_mark_visited() {
        let routes = vec![make_route("d1", 0.9)];
        let mut gm = GossipMessage::from_routes("node-1", &routes);
        gm.mark_visited("node-2");
        assert!(gm.has_visited("node-2"));
    }

    #[test]
    fn gossip_message_mesh_roundtrip() {
        let routes = vec![make_route("d1", 0.9)];
        let gm = GossipMessage::from_routes("node-1", &routes);
        let mesh_msg = gm.to_mesh_message("node-1", 4);
        assert!(mesh_msg.is_some());
        let mesh_msg = mesh_msg
            .as_ref()
            .unwrap_or_else(|| panic!("expected mesh message"));
        assert_eq!(mesh_msg.kind, MessageKind::Gossip);

        let parsed = GossipMessage::from_mesh_message(mesh_msg);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap_or_else(|| panic!("expected parsed gossip"));
        assert_eq!(parsed.sender, "node-1");
        assert_eq!(parsed.entry_count(), 1);
    }

    #[test]
    fn gossip_from_wrong_kind_returns_none() {
        let msg = MeshMessage::data("src", "dst", serde_json::json!(null), 10);
        assert!(GossipMessage::from_mesh_message(&msg).is_none());
    }

    #[test]
    fn gossip_loop_generate() {
        let mut gl = GossipLoop::new("node-1", GossipConfig::default());
        let routes = vec![make_route("d1", 0.9), make_route("d2", 0.8)];
        let gm = gl.generate_gossip(&routes);
        assert_eq!(gl.rounds_completed, 1);
        assert_eq!(gl.entries_gossiped, 2);
        assert_eq!(gm.entry_count(), 2);
    }

    #[test]
    fn gossip_loop_respects_max_entries() {
        let mut config = GossipConfig::default();
        config.max_entries_per_message = 1;
        let mut gl = GossipLoop::new("node-1", config);
        let routes = vec![make_route("d1", 0.9), make_route("d2", 0.8)];
        let gm = gl.generate_gossip(&routes);
        assert_eq!(gm.entry_count(), 1); // capped
    }

    #[test]
    fn gossip_loop_process_filters_self() {
        let mut gl = GossipLoop::new("node-1", GossipConfig::default());
        let routes = vec![make_route("d1", 0.9)];
        let gm = GossipMessage::from_routes("node-1", &routes);
        let entries = gl.process_gossip(&gm);
        assert!(entries.is_empty()); // from self, filtered
        assert_eq!(gl.messages_received, 0);
    }

    #[test]
    fn gossip_loop_process_filters_own_destination() {
        let mut gl = GossipLoop::new("node-1", GossipConfig::default());
        let routes = vec![make_route("node-1", 0.9), make_route("d2", 0.8)];
        let gm = GossipMessage::from_routes("node-2", &routes);
        let entries = gl.process_gossip(&gm);
        // Should filter out the route to ourselves
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].destination, "d2");
    }

    #[test]
    fn gossip_loop_start_stop() {
        let mut gl = GossipLoop::new("node-1", GossipConfig::default());
        assert!(!gl.running);
        gl.start();
        assert!(gl.running);
        gl.stop();
        assert!(!gl.running);
    }

    #[test]
    fn gossip_config_default() {
        let cfg = GossipConfig::default();
        assert_eq!(cfg.gossip_interval, Duration::from_secs(10));
        assert_eq!(cfg.max_entries_per_message, 20);
        assert_eq!(cfg.gossip_ttl, 4);
        assert_eq!(cfg.max_fan_out, 3);
    }

    #[test]
    fn gossip_route_entry_fields() {
        let entry = GossipRouteEntry {
            destination: "dest-1".to_string(),
            quality: RouteQuality::new(100.0, 0.95, 2),
            hop_count: 2,
        };
        assert_eq!(entry.destination, "dest-1");
        assert_eq!(entry.hop_count, 2);
    }
}
