//! Node: the central mesh participant.
//!
//! ## Primitive Foundation
//! - `NodeState`: T1, ς (State)
//! - `MeshMessage`: T2-C, σ (Sequence) + λ (Location) + μ (Mapping) + → (Causality)
//! - `Node`: T3, ς (State) — owns neighbors, routing, and message channels

use crate::error::MeshError;
use crate::neighbor::NeighborRegistry;
use crate::routing::RoutingTable;
use crate::topology::Path;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Default maximum TTL for messages.
pub const DEFAULT_MAX_TTL: u8 = 16;

/// Default neighbor capacity.
pub const DEFAULT_NEIGHBOR_CAPACITY: usize = 64;

/// Default channel buffer size.
pub const DEFAULT_CHANNEL_BUFFER: usize = 256;

// ============================================================================
// NodeState — Lifecycle state of a mesh node
// ============================================================================

/// Lifecycle state of a mesh node.
///
/// Tier: T1 | Dominant: ς (State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is initializing (not yet joined the mesh)
    Initializing,
    /// Node is discovering neighbors
    Discovering,
    /// Node is fully operational
    Active,
    /// Node is gracefully leaving the mesh
    Draining,
    /// Node has shut down
    Shutdown,
}

impl NodeState {
    /// Whether the node can send messages in this state.
    pub fn can_send(&self) -> bool {
        matches!(self, Self::Active | Self::Draining)
    }

    /// Whether the node can accept new neighbors in this state.
    pub fn can_accept_neighbors(&self) -> bool {
        matches!(self, Self::Discovering | Self::Active)
    }

    /// Whether the node is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Shutdown)
    }
}

// ============================================================================
// MessageKind — Types of mesh messages
// ============================================================================

/// Discriminator for mesh message types.
///
/// Tier: T2-P | Dominant: Σ (Sum)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    /// Direct unicast data message
    Data,
    /// Discovery announcement
    Discovery,
    /// Gossip route update
    Gossip,
    /// Heartbeat ping
    Heartbeat,
    /// Heartbeat response
    HeartbeatAck,
}

// ============================================================================
// MeshMessage — A message traversing the mesh
// ============================================================================

/// A message that traverses the mesh network via multi-hop relay.
///
/// Tier: T2-C | Dominant: σ (Sequence)
///
/// Each message carries its path (σ), source/destination (λ),
/// payload mapping (μ), and propagation direction (→).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMessage {
    /// Unique message ID
    pub id: String,
    /// Kind of message
    pub kind: MessageKind,
    /// Source node ID
    pub source: String,
    /// Destination node ID (empty for broadcast)
    pub destination: String,
    /// Path taken so far
    pub path: Path,
    /// Payload (JSON-serializable)
    pub payload: serde_json::Value,
}

impl MeshMessage {
    /// Create a new data message.
    pub fn data(
        source: impl Into<String>,
        destination: impl Into<String>,
        payload: serde_json::Value,
        max_ttl: u8,
    ) -> Self {
        let src = source.into();
        Self {
            id: Uuid::new_v4().to_string(),
            kind: MessageKind::Data,
            source: src.clone(),
            destination: destination.into(),
            path: Path::new(src, max_ttl),
            payload,
        }
    }

    /// Create a broadcast message (empty destination).
    pub fn broadcast(
        source: impl Into<String>,
        kind: MessageKind,
        payload: serde_json::Value,
        max_ttl: u8,
    ) -> Self {
        let src = source.into();
        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            source: src.clone(),
            destination: String::new(),
            path: Path::new(src, max_ttl),
            payload,
        }
    }

    /// Whether this message is a broadcast (no specific destination).
    pub fn is_broadcast(&self) -> bool {
        self.destination.is_empty()
    }

    /// Whether the message has reached its destination.
    pub fn is_at_destination(&self, current_node: &str) -> bool {
        !self.is_broadcast() && self.destination == current_node
    }

    /// Record a relay hop through the given node.
    pub fn relay_through(&mut self, node_id: impl Into<String>) -> bool {
        self.path.add_hop(node_id)
    }

    /// Check if this message has already visited the given node.
    pub fn has_visited(&self, node_id: &str) -> bool {
        self.path.hops.iter().any(|h| h == node_id)
    }

    /// Remaining TTL.
    pub fn remaining_ttl(&self) -> u8 {
        self.path.remaining_ttl()
    }
}

// ============================================================================
// MeshConfig — Node configuration
// ============================================================================

/// Configuration for a mesh node.
///
/// Tier: T2-C | Dominant: ∂ (Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Maximum TTL for outgoing messages
    pub max_ttl: u8,
    /// Maximum neighbor capacity
    pub max_neighbors: usize,
    /// Discovery announce interval in milliseconds
    pub discovery_interval_ms: u64,
    /// Gossip propagation interval in milliseconds
    pub gossip_interval_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Maximum message payload size in bytes
    pub max_payload_bytes: usize,
    /// Target neighbor count for homeostasis
    pub target_neighbors: usize,
    /// Neighbor tolerance band for homeostasis
    pub neighbor_tolerance: f64,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            max_ttl: DEFAULT_MAX_TTL,
            max_neighbors: DEFAULT_NEIGHBOR_CAPACITY,
            discovery_interval_ms: 5000,
            gossip_interval_ms: 10000,
            heartbeat_interval_ms: 3000,
            max_payload_bytes: 65536,
            target_neighbors: 8,
            neighbor_tolerance: 2.0,
        }
    }
}

impl MeshConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), MeshError> {
        if self.max_ttl == 0 {
            return Err(MeshError::InvalidConfig("max_ttl must be > 0".to_string()));
        }
        if self.max_neighbors == 0 {
            return Err(MeshError::InvalidConfig(
                "max_neighbors must be > 0".to_string(),
            ));
        }
        if self.target_neighbors > self.max_neighbors {
            return Err(MeshError::InvalidConfig(
                "target_neighbors must be <= max_neighbors".to_string(),
            ));
        }
        Ok(())
    }
}

// ============================================================================
// Node — The central mesh participant
// ============================================================================

/// A mesh network node that manages neighbors, routing, and message relay.
///
/// Tier: T3 | Dominant: ς (State)
///
/// Owns:
/// - NeighborRegistry (neighbor management with circuit breakers)
/// - RoutingTable (quality-sorted multi-path routing)
/// - Message channels (tokio mpsc for async relay)
pub struct Node {
    /// Unique node identifier
    pub id: String,
    /// Current lifecycle state
    pub state: NodeState,
    /// Configuration
    pub config: MeshConfig,
    /// Neighbor registry
    pub neighbors: NeighborRegistry,
    /// Routing table
    pub routing: RoutingTable,
    /// Inbound message sender (clone to give to other components)
    inbound_tx: mpsc::Sender<MeshMessage>,
    /// Inbound message receiver
    inbound_rx: Option<mpsc::Receiver<MeshMessage>>,
    /// Outbound message sender
    outbound_tx: mpsc::Sender<MeshMessage>,
    /// Outbound message receiver
    outbound_rx: Option<mpsc::Receiver<MeshMessage>>,
    /// Messages processed count
    pub messages_processed: u64,
    /// Messages relayed count
    pub messages_relayed: u64,
}

impl Node {
    /// Create a new mesh node with the given ID and configuration.
    pub fn new(id: impl Into<String>, config: MeshConfig) -> Self {
        let (inbound_tx, inbound_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER);
        let (outbound_tx, outbound_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER);
        let neighbors = NeighborRegistry::new(config.max_neighbors);
        let routing = RoutingTable::with_defaults();

        Self {
            id: id.into(),
            state: NodeState::Initializing,
            config,
            neighbors,
            routing,
            inbound_tx,
            inbound_rx: Some(inbound_rx),
            outbound_tx,
            outbound_rx: Some(outbound_rx),
            messages_processed: 0,
            messages_relayed: 0,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults(id: impl Into<String>) -> Self {
        Self::new(id, MeshConfig::default())
    }

    /// Get a clone of the inbound message sender (for external components to send messages).
    pub fn inbound_sender(&self) -> mpsc::Sender<MeshMessage> {
        self.inbound_tx.clone()
    }

    /// Get a clone of the outbound message sender.
    pub fn outbound_sender(&self) -> mpsc::Sender<MeshMessage> {
        self.outbound_tx.clone()
    }

    /// Take the inbound receiver (can only be called once).
    pub fn take_inbound_rx(&mut self) -> Option<mpsc::Receiver<MeshMessage>> {
        self.inbound_rx.take()
    }

    /// Take the outbound receiver (can only be called once).
    pub fn take_outbound_rx(&mut self) -> Option<mpsc::Receiver<MeshMessage>> {
        self.outbound_rx.take()
    }

    /// Transition to a new state.
    pub fn transition(&mut self, new_state: NodeState) {
        tracing::info!(
            node_id = %self.id,
            from = ?self.state,
            to = ?new_state,
            "node state transition"
        );
        self.state = new_state;
    }

    /// Send a data message to a destination.
    pub fn send_message(
        &self,
        destination: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<MeshMessage, MeshError> {
        if !self.state.can_send() {
            return Err(MeshError::InvalidConfig(format!(
                "cannot send in state {:?}",
                self.state
            )));
        }
        let dest = destination.into();
        let msg = MeshMessage::data(&self.id, &dest, payload, self.config.max_ttl);
        Ok(msg)
    }

    /// Process an inbound message: deliver locally or relay.
    pub fn process_message(&mut self, msg: &MeshMessage) -> MessageAction {
        self.messages_processed += 1;

        if msg.is_at_destination(&self.id) {
            return MessageAction::DeliverLocal;
        }

        if msg.path.is_expired() {
            return MessageAction::Drop("TTL expired".to_string());
        }

        if msg.has_visited(&self.id) && !msg.is_broadcast() {
            return MessageAction::Drop("loop detected".to_string());
        }

        // Find next hop
        if msg.is_broadcast() {
            return MessageAction::Broadcast;
        }

        match self.routing.best_route(&msg.destination) {
            Some(route) if self.neighbors.contains(&route.next_hop) => {
                self.messages_relayed += 1;
                MessageAction::RelayTo(route.next_hop)
            }
            _ => MessageAction::Drop(format!("no route to {}", msg.destination)),
        }
    }
}

/// Action to take after processing an inbound message.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageAction {
    /// Deliver the message to the local application
    DeliverLocal,
    /// Relay the message to the specified next-hop node
    RelayTo(String),
    /// Broadcast to all neighbors
    Broadcast,
    /// Drop the message (with reason)
    Drop(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- NodeState tests ----------

    #[test]
    fn node_state_can_send() {
        assert!(!NodeState::Initializing.can_send());
        assert!(!NodeState::Discovering.can_send());
        assert!(NodeState::Active.can_send());
        assert!(NodeState::Draining.can_send());
        assert!(!NodeState::Shutdown.can_send());
    }

    #[test]
    fn node_state_can_accept_neighbors() {
        assert!(!NodeState::Initializing.can_accept_neighbors());
        assert!(NodeState::Discovering.can_accept_neighbors());
        assert!(NodeState::Active.can_accept_neighbors());
        assert!(!NodeState::Draining.can_accept_neighbors());
    }

    #[test]
    fn node_state_is_terminal() {
        assert!(!NodeState::Active.is_terminal());
        assert!(NodeState::Shutdown.is_terminal());
    }

    // ---------- MeshMessage tests ----------

    #[test]
    fn message_data_has_source_and_dest() {
        let msg = MeshMessage::data("src", "dst", serde_json::json!({"key": "val"}), 16);
        assert_eq!(msg.source, "src");
        assert_eq!(msg.destination, "dst");
        assert_eq!(msg.kind, MessageKind::Data);
        assert!(!msg.is_broadcast());
    }

    #[test]
    fn message_broadcast_empty_destination() {
        let msg = MeshMessage::broadcast("src", MessageKind::Discovery, serde_json::json!(null), 8);
        assert!(msg.is_broadcast());
        assert!(msg.destination.is_empty());
    }

    #[test]
    fn message_relay_through() {
        let mut msg = MeshMessage::data("src", "dst", serde_json::json!(null), 3);
        assert!(msg.relay_through("hop1"));
        assert!(msg.relay_through("hop2"));
        assert!(msg.relay_through("hop3"));
        assert!(!msg.relay_through("hop4")); // TTL expired after 3 hops
    }

    #[test]
    fn message_has_visited() {
        let mut msg = MeshMessage::data("src", "dst", serde_json::json!(null), 10);
        assert!(msg.has_visited("src"));
        assert!(!msg.has_visited("hop1"));
        assert!(msg.relay_through("hop1"));
        assert!(msg.has_visited("hop1"));
    }

    #[test]
    fn message_is_at_destination() {
        let msg = MeshMessage::data("src", "dst", serde_json::json!(null), 10);
        assert!(msg.is_at_destination("dst"));
        assert!(!msg.is_at_destination("src"));
    }

    #[test]
    fn message_remaining_ttl() {
        let mut msg = MeshMessage::data("src", "dst", serde_json::json!(null), 5);
        assert_eq!(msg.remaining_ttl(), 5);
        assert!(msg.relay_through("a"));
        assert_eq!(msg.remaining_ttl(), 4);
    }

    #[test]
    fn message_unique_ids() {
        let a = MeshMessage::data("s", "d", serde_json::json!(null), 10);
        let b = MeshMessage::data("s", "d", serde_json::json!(null), 10);
        assert_ne!(a.id, b.id);
    }

    // ---------- MeshConfig tests ----------

    #[test]
    fn config_default_valid() {
        let cfg = MeshConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn config_zero_ttl_invalid() {
        let mut cfg = MeshConfig::default();
        cfg.max_ttl = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn config_zero_neighbors_invalid() {
        let mut cfg = MeshConfig::default();
        cfg.max_neighbors = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn config_target_exceeds_max_invalid() {
        let mut cfg = MeshConfig::default();
        cfg.target_neighbors = 100;
        cfg.max_neighbors = 10;
        assert!(cfg.validate().is_err());
    }

    // ---------- Node tests ----------

    #[test]
    fn node_new_initializing() {
        let node = Node::with_defaults("node-1");
        assert_eq!(node.id, "node-1");
        assert_eq!(node.state, NodeState::Initializing);
        assert_eq!(node.messages_processed, 0);
    }

    #[test]
    fn node_transition() {
        let mut node = Node::with_defaults("node-1");
        node.transition(NodeState::Discovering);
        assert_eq!(node.state, NodeState::Discovering);
        node.transition(NodeState::Active);
        assert_eq!(node.state, NodeState::Active);
    }

    #[test]
    fn node_send_requires_active() {
        let node = Node::with_defaults("node-1");
        let result = node.send_message("dest", serde_json::json!(null));
        assert!(result.is_err()); // still Initializing
    }

    #[test]
    fn node_send_active_ok() {
        let mut node = Node::with_defaults("node-1");
        node.transition(NodeState::Active);
        let result = node.send_message("dest", serde_json::json!({"hello": "world"}));
        assert!(result.is_ok());
        let msg = result.ok();
        assert!(msg.is_some());
        assert_eq!(msg.as_ref().map(|m| m.source.as_str()), Some("node-1"));
    }

    #[test]
    fn node_process_deliver_local() {
        let mut node = Node::with_defaults("node-1");
        node.transition(NodeState::Active);
        let msg = MeshMessage::data("src", "node-1", serde_json::json!(null), 10);
        let action = node.process_message(&msg);
        assert_eq!(action, MessageAction::DeliverLocal);
        assert_eq!(node.messages_processed, 1);
    }

    #[test]
    fn node_process_drop_expired() {
        let mut node = Node::with_defaults("relay");
        node.transition(NodeState::Active);
        let mut msg = MeshMessage::data("src", "dst", serde_json::json!(null), 1);
        assert!(msg.relay_through("relay")); // uses up TTL
        let action = node.process_message(&msg);
        assert!(matches!(action, MessageAction::Drop(_)));
    }

    #[test]
    fn node_process_drop_no_route() {
        let mut node = Node::with_defaults("relay");
        node.transition(NodeState::Active);
        let msg = MeshMessage::data("src", "unknown-dest", serde_json::json!(null), 10);
        let action = node.process_message(&msg);
        assert!(matches!(action, MessageAction::Drop(_)));
    }

    #[test]
    fn node_process_broadcast() {
        let mut node = Node::with_defaults("node-1");
        node.transition(NodeState::Active);
        let msg = MeshMessage::broadcast("src", MessageKind::Discovery, serde_json::json!(null), 8);
        let action = node.process_message(&msg);
        assert_eq!(action, MessageAction::Broadcast);
    }

    #[test]
    fn node_take_receivers_once() {
        let mut node = Node::with_defaults("node-1");
        assert!(node.take_inbound_rx().is_some());
        assert!(node.take_inbound_rx().is_none()); // already taken
        assert!(node.take_outbound_rx().is_some());
        assert!(node.take_outbound_rx().is_none());
    }
}
