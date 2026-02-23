//! Async runtime: event loop wiring discovery, gossip, resilience, and message relay.
//!
//! ## Primitive Foundation
//! - `MeshRuntime`: ρ (Recursion) + σ (Sequence) — self-correcting event loop
//! - `MeshHandle`: μ (Mapping) + → (Causality) — external control interface
//! - `MeshEvent`: Σ (Sum) — observable runtime events

use crate::discovery::{DiscoveryAction, DiscoveryConfig, DiscoveryLoop, DiscoveryMessage};
use crate::error::MeshError;
use crate::gossip::{GossipConfig, GossipLoop, GossipMessage};
use crate::neighbor::{Neighbor, NeighborRegistry};
use crate::node::{MeshConfig, MeshMessage, MessageKind, Node, NodeState};
use crate::resilience::{ResilienceAction, ResilienceConfig, ResilienceLoop};
use crate::routing::{Route, RoutingTable};
use crate::topology::{Path, RouteQuality};
use tokio::sync::{mpsc, watch};
use tokio::time::{Duration, interval};
use tracing::Instrument;

/// Default event channel buffer size.
pub const DEFAULT_EVENT_BUFFER: usize = 128;

/// Default delivery channel buffer size.
pub const DEFAULT_DELIVERY_BUFFER: usize = 64;

/// Minimum discovery rounds before transitioning to Active.
const MIN_DISCOVERY_ROUNDS: u64 = 2;

// ============================================================================
// MeshEvent — Observable events from the runtime
// ============================================================================

/// Events emitted by the mesh runtime for observability.
///
/// Tier: T2-P | Dominant: Σ (Sum)
#[derive(Debug, Clone)]
pub enum MeshEvent {
    /// A message was delivered to the local application
    MessageDelivered { msg_id: String, source: String },
    /// A message was relayed to a neighbor
    MessageRelayed { msg_id: String, next_hop: String },
    /// A message was dropped
    MessageDropped { msg_id: String, reason: String },
    /// A new neighbor was added via discovery
    NeighborAdded { node_id: String },
    /// Discovery announce sent
    DiscoveryAnnounce,
    /// Gossip round completed
    GossipRound { entries_shared: usize },
    /// Resilience action taken
    ResilienceTick { action: ResilienceAction },
    /// Heartbeat sent to neighbor
    HeartbeatSent { neighbor_id: String },
    /// Heartbeat acknowledged by neighbor
    HeartbeatAcked { neighbor_id: String },
    /// Node state transition
    StateTransition { from: NodeState, to: NodeState },
    /// Runtime shutdown complete
    Shutdown,
}

// ============================================================================
// MeshHandle — External interface to a running runtime
// ============================================================================

/// Handle to a running mesh runtime.
///
/// Tier: T2-C | Dominant: μ (Mapping)
///
/// Thread-safe interface providing:
/// - Message injection into the mesh
/// - Delivery channel for locally-addressed messages
/// - Outbound channel for transport layer
/// - Event channel for observability
/// - Shared read access to neighbors and routing
pub struct MeshHandle {
    /// Node ID
    pub node_id: String,
    /// Send messages into this node
    inbound_tx: mpsc::Sender<MeshMessage>,
    /// Receive messages delivered to this node
    delivery_rx: Option<mpsc::Receiver<MeshMessage>>,
    /// Receive messages to forward to other nodes (transport layer)
    outbound_rx: Option<mpsc::Receiver<MeshMessage>>,
    /// Receive runtime events
    event_rx: Option<mpsc::Receiver<MeshEvent>>,
    /// Shutdown signal
    shutdown_tx: watch::Sender<bool>,
    /// Shared neighbor registry (read access via Arc<DashMap>)
    pub neighbors: NeighborRegistry,
    /// Shared routing table (read access via Arc<DashMap>)
    pub routing: RoutingTable,
    /// Node configuration
    pub config: MeshConfig,
}

impl MeshHandle {
    /// Send a data message through the mesh.
    pub async fn send(
        &self,
        destination: &str,
        payload: serde_json::Value,
    ) -> Result<(), MeshError> {
        let msg = MeshMessage::data(&self.node_id, destination, payload, self.config.max_ttl);
        self.inbound_tx
            .send(msg)
            .await
            .map_err(|_| MeshError::ChannelError("inbound channel closed".to_string()))
    }

    /// Inject a raw MeshMessage into this node.
    pub async fn inject(&self, msg: MeshMessage) -> Result<(), MeshError> {
        self.inbound_tx
            .send(msg)
            .await
            .map_err(|_| MeshError::ChannelError("inbound channel closed".to_string()))
    }

    /// Get a clone of the inbound sender (for wiring to other nodes).
    pub fn inbound_sender(&self) -> mpsc::Sender<MeshMessage> {
        self.inbound_tx.clone()
    }

    /// Take the delivery receiver (messages addressed to this node).
    pub fn take_delivery_rx(&mut self) -> Option<mpsc::Receiver<MeshMessage>> {
        self.delivery_rx.take()
    }

    /// Take the outbound receiver (messages to forward to other nodes).
    pub fn take_outbound_rx(&mut self) -> Option<mpsc::Receiver<MeshMessage>> {
        self.outbound_rx.take()
    }

    /// Take the event receiver (runtime events).
    pub fn take_event_rx(&mut self) -> Option<mpsc::Receiver<MeshEvent>> {
        self.event_rx.take()
    }

    /// Trigger graceful shutdown.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Check if the runtime is still running.
    pub fn is_running(&self) -> bool {
        !*self.shutdown_tx.borrow()
    }

    /// Number of known neighbors.
    pub fn neighbor_count(&self) -> usize {
        self.neighbors.len()
    }

    /// Number of reachable neighbors.
    pub fn reachable_count(&self) -> usize {
        self.neighbors.reachable_ids().len()
    }

    /// Number of known route destinations.
    pub fn destination_count(&self) -> usize {
        self.routing.destination_count()
    }
}

// ============================================================================
// MeshRuntime — The async event loop
// ============================================================================

/// Async mesh runtime orchestrating discovery, gossip, resilience, and relay.
///
/// Tier: T3 | Dominant: ρ (Recursion)
///
/// Main event loop:
/// 1. Process inbound messages (deliver/relay/broadcast/drop)
/// 2. Periodic discovery announces
/// 3. Periodic gossip rounds
/// 4. Periodic heartbeats
/// 5. Periodic resilience ticks
/// 6. Graceful shutdown via watch channel
pub struct MeshRuntime {
    node_id: String,
    config: MeshConfig,
    state: NodeState,
    neighbors: NeighborRegistry,
    routing: RoutingTable,
    discovery: DiscoveryLoop,
    gossip: GossipLoop,
    resilience: ResilienceLoop,
    inbound_rx: mpsc::Receiver<MeshMessage>,
    outbound_tx: mpsc::Sender<MeshMessage>,
    delivery_tx: mpsc::Sender<MeshMessage>,
    event_tx: mpsc::Sender<MeshEvent>,
    shutdown_rx: watch::Receiver<bool>,
    messages_processed: u64,
    messages_relayed: u64,
}

impl MeshRuntime {
    /// Build a runtime and handle from a Node.
    ///
    /// Consumes the Node's channel receivers. Returns `(runtime, handle)`.
    /// Spawn `runtime.run()` as a tokio task, use `handle` for interaction.
    pub fn build(mut node: Node) -> (Self, MeshHandle) {
        let inbound_rx = node.take_inbound_rx().unwrap_or_else(|| {
            let (_tx, rx) = mpsc::channel(1);
            rx
        });
        let outbound_rx = node.take_outbound_rx().unwrap_or_else(|| {
            let (_tx, rx) = mpsc::channel(1);
            rx
        });

        let (delivery_tx, delivery_rx) = mpsc::channel(DEFAULT_DELIVERY_BUFFER);
        let (event_tx, event_rx) = mpsc::channel(DEFAULT_EVENT_BUFFER);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let discovery_config = DiscoveryConfig {
            announce_interval: Duration::from_millis(node.config.discovery_interval_ms),
            ..DiscoveryConfig::default()
        };
        let gossip_config = GossipConfig {
            gossip_interval: Duration::from_millis(node.config.gossip_interval_ms),
            ..GossipConfig::default()
        };
        let resilience_config = ResilienceConfig {
            target_neighbors: node.config.target_neighbors,
            neighbor_tolerance: node.config.neighbor_tolerance,
            heartbeat_interval: Duration::from_millis(node.config.heartbeat_interval_ms),
            ..ResilienceConfig::default()
        };

        let runtime = Self {
            node_id: node.id.clone(),
            config: node.config.clone(),
            state: NodeState::Initializing,
            neighbors: node.neighbors.clone(),
            routing: node.routing.clone(),
            discovery: DiscoveryLoop::new(&node.id, discovery_config),
            gossip: GossipLoop::new(&node.id, gossip_config),
            resilience: ResilienceLoop::new(&node.id, resilience_config),
            inbound_rx,
            outbound_tx: node.outbound_sender(),
            delivery_tx,
            event_tx,
            shutdown_rx,
            messages_processed: 0,
            messages_relayed: 0,
        };

        let handle = MeshHandle {
            node_id: node.id.clone(),
            inbound_tx: node.inbound_sender(),
            delivery_rx: Some(delivery_rx),
            outbound_rx: Some(outbound_rx),
            event_rx: Some(event_rx),
            shutdown_tx,
            neighbors: node.neighbors.clone(),
            routing: node.routing.clone(),
            config: node.config,
        };

        (runtime, handle)
    }

    /// Run the mesh runtime (consumes self).
    pub async fn run(mut self) {
        let span = tracing::info_span!("mesh_runtime", node_id = %self.node_id);
        self.run_loop().instrument(span).await;
    }

    /// Inner event loop — separated to keep nesting under complexity gate.
    async fn run_loop(&mut self) {
        self.transition(NodeState::Discovering);
        self.discovery.start();
        self.gossip.start();
        self.resilience.start();

        let disc_ms = self.config.discovery_interval_ms;
        let gossip_ms = self.config.gossip_interval_ms;
        let hb_ms = self.config.heartbeat_interval_ms;

        let mut discovery_tick = interval(Duration::from_millis(disc_ms));
        let mut gossip_tick = interval(Duration::from_millis(gossip_ms));
        let mut heartbeat_tick = interval(Duration::from_millis(hb_ms));
        let mut resilience_tick = interval(Duration::from_millis(hb_ms.saturating_mul(2)));

        let mut discovery_rounds = 0u64;

        loop {
            tokio::select! {
                msg = self.inbound_rx.recv() => {
                    match msg {
                        Some(m) => self.handle_inbound(m).await,
                        None => break,
                    }
                }
                _ = discovery_tick.tick() => {
                    self.on_discovery_tick();
                    discovery_rounds += 1;
                    self.maybe_activate(discovery_rounds);
                }
                _ = gossip_tick.tick() => {
                    self.on_gossip_tick();
                }
                _ = heartbeat_tick.tick() => {
                    self.on_heartbeat_tick();
                }
                _ = resilience_tick.tick() => {
                    self.on_resilience_tick();
                }
                result = self.shutdown_rx.changed() => {
                    if result.is_err() || *self.shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }

        self.shutdown_sequence();
    }

    // --- State Management ---

    fn transition(&mut self, new_state: NodeState) {
        let from = self.state;
        self.state = new_state;
        tracing::info!(from = ?from, to = ?new_state, "state transition");
        self.emit(MeshEvent::StateTransition {
            from,
            to: new_state,
        });
    }

    fn maybe_activate(&mut self, discovery_rounds: u64) {
        if self.state == NodeState::Discovering && discovery_rounds >= MIN_DISCOVERY_ROUNDS {
            self.transition(NodeState::Active);
        }
    }

    fn emit(&self, event: MeshEvent) {
        let _ = self.event_tx.try_send(event);
    }

    fn shutdown_sequence(&mut self) {
        self.transition(NodeState::Draining);
        self.discovery.stop();
        self.gossip.stop();
        self.resilience.stop();
        self.transition(NodeState::Shutdown);
        self.emit(MeshEvent::Shutdown);

        tracing::info!(
            processed = self.messages_processed,
            relayed = self.messages_relayed,
            neighbors = self.neighbors.len(),
            routes = self.routing.total_routes(),
            "mesh runtime shutdown complete"
        );
    }

    // --- Inbound Message Handling ---

    async fn handle_inbound(&mut self, msg: MeshMessage) {
        self.messages_processed += 1;

        match msg.kind {
            MessageKind::Heartbeat => {
                self.on_heartbeat_received(&msg);
                return;
            }
            MessageKind::HeartbeatAck => {
                self.on_heartbeat_ack(&msg);
                return;
            }
            MessageKind::Discovery => {
                self.on_discovery_received(&msg);
                return;
            }
            MessageKind::Gossip => {
                self.on_gossip_received(&msg);
                return;
            }
            MessageKind::Data => {}
        }

        self.route_data_message(msg);
    }

    fn route_data_message(&mut self, msg: MeshMessage) {
        if msg.is_at_destination(&self.node_id) {
            let _ = self.delivery_tx.try_send(msg.clone());
            self.emit(MeshEvent::MessageDelivered {
                msg_id: msg.id.clone(),
                source: msg.source.clone(),
            });
            return;
        }

        // Originating message — skip loop check (source is already in path)
        if msg.source == self.node_id {
            self.route_outbound(msg);
            return;
        }

        // Relay from another node
        if msg.path.is_expired() {
            self.emit(MeshEvent::MessageDropped {
                msg_id: msg.id.clone(),
                reason: "TTL expired".to_string(),
            });
            return;
        }

        if msg.has_visited(&self.node_id) {
            self.emit(MeshEvent::MessageDropped {
                msg_id: msg.id.clone(),
                reason: "loop detected".to_string(),
            });
            return;
        }

        self.relay_outbound(msg);
    }

    /// Route an originating message (we are the source).
    fn route_outbound(&mut self, msg: MeshMessage) {
        match self.routing.best_route(&msg.destination) {
            Some(route) if self.neighbors.contains(&route.next_hop) => {
                let next_hop = route.next_hop.clone();
                let _ = self.outbound_tx.try_send(msg.clone());
                self.emit(MeshEvent::MessageRelayed {
                    msg_id: msg.id.clone(),
                    next_hop,
                });
            }
            _ => {
                self.emit(MeshEvent::MessageDropped {
                    msg_id: msg.id.clone(),
                    reason: format!("no route to {}", msg.destination),
                });
            }
        }
    }

    /// Relay a message from another node (add self to path).
    fn relay_outbound(&mut self, mut msg: MeshMessage) {
        let dest = msg.destination.clone();
        match self.routing.best_route(&dest) {
            Some(route) if self.neighbors.contains(&route.next_hop) => {
                if msg.relay_through(&self.node_id) {
                    self.messages_relayed += 1;
                    let next_hop = route.next_hop.clone();
                    let _ = self.outbound_tx.try_send(msg.clone());
                    self.emit(MeshEvent::MessageRelayed {
                        msg_id: msg.id.clone(),
                        next_hop,
                    });
                }
            }
            _ => {
                self.emit(MeshEvent::MessageDropped {
                    msg_id: msg.id.clone(),
                    reason: format!("no route to {dest}"),
                });
            }
        }
    }

    // --- Discovery ---

    fn on_discovery_tick(&mut self) {
        let neighbor_count = self.neighbors.len();
        let capacity = self.config.max_neighbors.saturating_sub(neighbor_count);
        let announce = self.discovery.generate_announce(neighbor_count, capacity);
        let ttl = self.discovery.config.discovery_ttl;

        if let Some(mesh_msg) = announce.to_mesh_message(&self.node_id, ttl) {
            let _ = self.outbound_tx.try_send(mesh_msg);
        }
        self.emit(MeshEvent::DiscoveryAnnounce);
    }

    fn on_discovery_received(&mut self, msg: &MeshMessage) {
        let disc_msg = match DiscoveryMessage::from_mesh_message(msg) {
            Some(m) => m,
            None => return,
        };

        let action = self.discovery.process_message(&disc_msg);
        match action {
            DiscoveryAction::RespondTo(node_id) => {
                self.add_discovered_neighbor(&node_id);
                let quality = RouteQuality::new(50.0, 0.8, 1);
                let response = DiscoveryMessage::response(&self.node_id, quality);
                if let Some(mesh_msg) = response.to_mesh_message(&self.node_id, 1) {
                    let _ = self.outbound_tx.try_send(mesh_msg);
                }
            }
            DiscoveryAction::AddNeighbor(node_id) => {
                self.add_discovered_neighbor(&node_id);
            }
            DiscoveryAction::Ignore => {}
        }
    }

    fn add_discovered_neighbor(&mut self, node_id: &str) {
        if self.neighbors.contains(node_id) {
            return;
        }
        let quality = RouteQuality::new(50.0, 0.8, 1);
        let neighbor = Neighbor::new(node_id, quality.clone());
        if self.neighbors.add(neighbor).is_ok() {
            let path = Path::new(&self.node_id, self.config.max_ttl);
            let route = Route::new(node_id, node_id, path, quality);
            self.routing.upsert(route);
            self.emit(MeshEvent::NeighborAdded {
                node_id: node_id.to_string(),
            });
            tracing::info!(neighbor = node_id, "discovered neighbor");
        }
    }

    // --- Gossip ---

    fn on_gossip_tick(&mut self) {
        if self.neighbors.is_empty() {
            return;
        }

        let destinations = self.routing.destinations();
        let routes: Vec<Route> = destinations
            .iter()
            .filter_map(|d| self.routing.best_route(d))
            .collect();

        if routes.is_empty() {
            return;
        }

        let gossip = self.gossip.generate_gossip(&routes);
        let ttl = self.gossip.config.gossip_ttl;
        if let Some(mesh_msg) = gossip.to_mesh_message(&self.node_id, ttl) {
            let _ = self.outbound_tx.try_send(mesh_msg);
        }
        self.emit(MeshEvent::GossipRound {
            entries_shared: routes.len(),
        });
    }

    fn on_gossip_received(&mut self, msg: &MeshMessage) {
        let gossip_msg = match GossipMessage::from_mesh_message(msg) {
            Some(m) => m,
            None => return,
        };

        let learned = self.gossip.process_gossip(&gossip_msg);
        for entry in &learned {
            let path = {
                let mut p = Path::new(&self.node_id, self.config.max_ttl);
                let _ = p.add_hop(&gossip_msg.sender);
                p
            };
            let quality = RouteQuality::new(
                entry.quality.latency_ms * 1.2,
                entry.quality.reliability * 0.95,
                entry.hop_count + 1,
            );
            let route = Route::new(&entry.destination, &gossip_msg.sender, path, quality)
                .with_source("gossip");
            self.routing.upsert(route);
        }
    }

    // --- Heartbeat ---

    fn on_heartbeat_tick(&mut self) {
        let reachable = self.neighbors.reachable_ids();
        for neighbor_id in &reachable {
            let msg = MeshMessage {
                id: nexcore_id::NexId::v4().to_string(),
                kind: MessageKind::Heartbeat,
                source: self.node_id.clone(),
                destination: neighbor_id.clone(),
                path: Path::new(&self.node_id, 1),
                payload: serde_json::json!(null),
            };
            let _ = self.outbound_tx.try_send(msg);
            self.resilience.record_heartbeat_sent();
            self.emit(MeshEvent::HeartbeatSent {
                neighbor_id: neighbor_id.clone(),
            });
        }
    }

    fn on_heartbeat_received(&mut self, msg: &MeshMessage) {
        self.neighbors.record_success(&msg.source);
        let ack = MeshMessage {
            id: nexcore_id::NexId::v4().to_string(),
            kind: MessageKind::HeartbeatAck,
            source: self.node_id.clone(),
            destination: msg.source.clone(),
            path: Path::new(&self.node_id, 1),
            payload: serde_json::json!(null),
        };
        let _ = self.outbound_tx.try_send(ack);
    }

    fn on_heartbeat_ack(&mut self, msg: &MeshMessage) {
        self.neighbors.record_success(&msg.source);
        self.resilience.record_heartbeat_received();
        self.emit(MeshEvent::HeartbeatAcked {
            neighbor_id: msg.source.clone(),
        });
    }

    // --- Resilience ---

    fn on_resilience_tick(&mut self) {
        let reachable = self.neighbors.reachable_ids().len();
        let unreachable = self.neighbors.len().saturating_sub(reachable);

        let action = self.resilience.tick(reachable, unreachable);
        self.emit(MeshEvent::ResilienceTick {
            action: action.clone(),
        });

        match action {
            ResilienceAction::ResetBreakers => {
                self.neighbors.attempt_reset_all();
                tracing::info!("resilience: reset circuit breakers");
            }
            ResilienceAction::AccelerateDiscovery { urgency } => {
                tracing::info!(urgency, "resilience: accelerate discovery");
            }
            ResilienceAction::DecelerateDiscovery => {
                tracing::info!("resilience: decelerate discovery");
            }
            ResilienceAction::Nominal => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    fn quick_config() -> MeshConfig {
        MeshConfig {
            discovery_interval_ms: 50,
            gossip_interval_ms: 100,
            heartbeat_interval_ms: 50,
            target_neighbors: 2,
            neighbor_tolerance: 1.0,
            ..MeshConfig::default()
        }
    }

    #[tokio::test]
    async fn runtime_build_creates_handle() {
        let node = Node::new("test-1", quick_config());
        let (_runtime, handle) = MeshRuntime::build(node);
        assert_eq!(handle.node_id, "test-1");
        assert!(handle.is_running());
        assert_eq!(handle.neighbor_count(), 0);
        assert_eq!(handle.destination_count(), 0);
    }

    #[tokio::test]
    async fn runtime_starts_and_shuts_down() {
        let node = Node::new("test-1", quick_config());
        let (runtime, handle) = MeshRuntime::build(node);

        let task = tokio::spawn(runtime.run());

        tokio::time::sleep(Duration::from_millis(100)).await;

        handle.shutdown();
        let result = timeout(Duration::from_secs(2), task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn runtime_delivers_local_message() {
        let node = Node::new("node-a", quick_config());
        let (runtime, mut handle) = MeshRuntime::build(node);

        let mut delivery = match handle.take_delivery_rx() {
            Some(rx) => rx,
            None => return,
        };

        let task = tokio::spawn(runtime.run());

        // Message addressed to ourselves from external source
        let msg = MeshMessage::data("external", "node-a", serde_json::json!({"test": true}), 10);
        let _ = handle.inject(msg).await;

        let delivered = timeout(Duration::from_secs(1), delivery.recv()).await;
        assert!(delivered.is_ok());
        let delivered = delivered.ok().flatten();
        assert!(delivered.is_some());

        if let Some(m) = &delivered {
            assert_eq!(m.destination, "node-a");
            assert_eq!(m.source, "external");
        }

        handle.shutdown();
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn runtime_state_transitions() {
        let node = Node::new("n1", quick_config());
        let (runtime, mut handle) = MeshRuntime::build(node);

        let mut events = match handle.take_event_rx() {
            Some(rx) => rx,
            None => return,
        };

        let task = tokio::spawn(runtime.run());

        // Wait for discovery rounds to trigger Active
        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut transitions = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(50), events.recv()).await {
            if let MeshEvent::StateTransition { from, to } = event {
                transitions.push((from, to));
            }
        }

        // Should have: Initializing→Discovering, Discovering→Active
        assert!(transitions.len() >= 2);
        assert_eq!(
            transitions[0],
            (NodeState::Initializing, NodeState::Discovering)
        );
        assert_eq!(transitions[1], (NodeState::Discovering, NodeState::Active));

        handle.shutdown();
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn runtime_emits_discovery_announces() {
        let node = Node::new("n1", quick_config());
        let (runtime, mut handle) = MeshRuntime::build(node);

        let mut events = match handle.take_event_rx() {
            Some(rx) => rx,
            None => return,
        };

        let task = tokio::spawn(runtime.run());

        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut announce_count = 0;
        while let Ok(Some(event)) = timeout(Duration::from_millis(50), events.recv()).await {
            if matches!(event, MeshEvent::DiscoveryAnnounce) {
                announce_count += 1;
            }
        }
        assert!(announce_count > 0);

        handle.shutdown();
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn runtime_drops_no_route() {
        let node = Node::new("n1", quick_config());
        let (runtime, mut handle) = MeshRuntime::build(node);

        let mut events = match handle.take_event_rx() {
            Some(rx) => rx,
            None => return,
        };

        let task = tokio::spawn(runtime.run());
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send to unknown destination
        let _ = handle.send("nonexistent", serde_json::json!(null)).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut dropped = false;
        while let Ok(Some(event)) = timeout(Duration::from_millis(50), events.recv()).await {
            if let MeshEvent::MessageDropped { reason, .. } = &event {
                if reason.contains("no route") {
                    dropped = true;
                }
            }
        }
        assert!(dropped);

        handle.shutdown();
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn two_node_message_delivery() {
        let config = quick_config();

        let node_a = Node::new("A", config.clone());
        let node_b = Node::new("B", config);

        let (runtime_a, mut handle_a) = MeshRuntime::build(node_a);
        let (runtime_b, mut handle_b) = MeshRuntime::build(node_b);

        // Wire transport: A↔B
        let mut a_out = match handle_a.take_outbound_rx() {
            Some(rx) => rx,
            None => return,
        };
        let mut b_out = match handle_b.take_outbound_rx() {
            Some(rx) => rx,
            None => return,
        };
        let a_in = handle_a.inbound_sender();
        let b_in = handle_b.inbound_sender();

        tokio::spawn(async move {
            while let Some(msg) = a_out.recv().await {
                let _ = b_in.send(msg).await;
            }
        });
        tokio::spawn(async move {
            while let Some(msg) = b_out.recv().await {
                let _ = a_in.send(msg).await;
            }
        });

        // Pre-configure neighbors and routes
        let quality = RouteQuality::new(10.0, 0.95, 1);
        let _ = handle_a.neighbors.add(Neighbor::new("B", quality.clone()));
        let _ = handle_b.neighbors.add(Neighbor::new("A", quality.clone()));
        handle_a
            .routing
            .upsert(Route::new("B", "B", Path::new("A", 16), quality.clone()));
        handle_b
            .routing
            .upsert(Route::new("A", "A", Path::new("B", 16), quality));

        // Start runtimes
        let task_a = tokio::spawn(runtime_a.run());
        let task_b = tokio::spawn(runtime_b.run());

        let mut b_delivery = match handle_b.take_delivery_rx() {
            Some(rx) => rx,
            None => return,
        };

        // Wait for runtimes to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send from A to B
        let _ = handle_a
            .send("B", serde_json::json!({"hello": "mesh"}))
            .await;

        let delivered = timeout(Duration::from_secs(2), b_delivery.recv()).await;
        assert!(delivered.is_ok());

        if let Ok(Some(msg)) = delivered {
            assert_eq!(msg.source, "A");
            assert_eq!(msg.destination, "B");
        }

        handle_a.shutdown();
        handle_b.shutdown();
        let _ = timeout(Duration::from_secs(1), task_a).await;
        let _ = timeout(Duration::from_secs(1), task_b).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn three_node_relay() {
        let config = quick_config();

        let node_a = Node::new("A", config.clone());
        let node_b = Node::new("B", config.clone());
        let node_c = Node::new("C", config);

        let (runtime_a, mut handle_a) = MeshRuntime::build(node_a);
        let (runtime_b, mut handle_b) = MeshRuntime::build(node_b);
        let (runtime_c, mut handle_c) = MeshRuntime::build(node_c);

        // Linear topology: A — B — C
        let mut a_out = match handle_a.take_outbound_rx() {
            Some(rx) => rx,
            None => return,
        };
        let mut b_out = match handle_b.take_outbound_rx() {
            Some(rx) => rx,
            None => return,
        };
        let mut c_out = match handle_c.take_outbound_rx() {
            Some(rx) => rx,
            None => return,
        };

        let a_in = handle_a.inbound_sender();
        let b_in_from_a = handle_b.inbound_sender();
        let b_in_from_c = handle_b.inbound_sender();
        let c_in = handle_c.inbound_sender();

        // A.out → B.in
        tokio::spawn(async move {
            while let Some(msg) = a_out.recv().await {
                let _ = b_in_from_a.send(msg).await;
            }
        });
        // B.out → A.in + C.in (B forwards to both neighbors)
        tokio::spawn(async move {
            while let Some(msg) = b_out.recv().await {
                let _ = a_in.send(msg.clone()).await;
                let _ = c_in.send(msg).await;
            }
        });
        // C.out → B.in
        tokio::spawn(async move {
            while let Some(msg) = c_out.recv().await {
                let _ = b_in_from_c.send(msg).await;
            }
        });

        // Configure topology
        let q = RouteQuality::new(10.0, 0.95, 1);

        // A: neighbor B, route to B (direct), route to C (via B)
        let _ = handle_a.neighbors.add(Neighbor::new("B", q.clone()));
        handle_a
            .routing
            .upsert(Route::new("B", "B", Path::new("A", 16), q.clone()));
        handle_a
            .routing
            .upsert(Route::new("C", "B", Path::new("A", 16), q.clone()));

        // B: neighbors A and C
        let _ = handle_b.neighbors.add(Neighbor::new("A", q.clone()));
        let _ = handle_b.neighbors.add(Neighbor::new("C", q.clone()));
        handle_b
            .routing
            .upsert(Route::new("A", "A", Path::new("B", 16), q.clone()));
        handle_b
            .routing
            .upsert(Route::new("C", "C", Path::new("B", 16), q.clone()));

        // C: neighbor B, route to A (via B)
        let _ = handle_c.neighbors.add(Neighbor::new("B", q.clone()));
        handle_c
            .routing
            .upsert(Route::new("B", "B", Path::new("C", 16), q.clone()));
        handle_c
            .routing
            .upsert(Route::new("A", "B", Path::new("C", 16), q));

        // Start runtimes
        let task_a = tokio::spawn(runtime_a.run());
        let task_b = tokio::spawn(runtime_b.run());
        let task_c = tokio::spawn(runtime_c.run());

        let mut c_delivery = match handle_c.take_delivery_rx() {
            Some(rx) => rx,
            None => return,
        };

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send from A to C (must relay through B)
        let _ = handle_a
            .send("C", serde_json::json!({"routed": "through-B"}))
            .await;

        let delivered = timeout(Duration::from_secs(2), c_delivery.recv()).await;
        assert!(delivered.is_ok());

        if let Ok(Some(msg)) = delivered {
            assert_eq!(msg.source, "A");
            assert_eq!(msg.destination, "C");
            // Path should show the relay: [A, B]
            assert!(msg.path.hops.len() >= 2);
        }

        handle_a.shutdown();
        handle_b.shutdown();
        handle_c.shutdown();
        let _ = timeout(Duration::from_secs(1), task_a).await;
        let _ = timeout(Duration::from_secs(1), task_b).await;
        let _ = timeout(Duration::from_secs(1), task_c).await;
    }

    #[tokio::test]
    async fn handle_send_receive_roundtrip() {
        let node = Node::new("echo", quick_config());
        let (runtime, mut handle) = MeshRuntime::build(node);

        let mut delivery = match handle.take_delivery_rx() {
            Some(rx) => rx,
            None => return,
        };

        let task = tokio::spawn(runtime.run());
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send message to self
        let msg = MeshMessage::data("external", "echo", serde_json::json!(42), 10);
        let _ = handle.inject(msg).await;

        let received = timeout(Duration::from_secs(1), delivery.recv()).await;
        assert!(received.is_ok());

        handle.shutdown();
        let _ = timeout(Duration::from_secs(1), task).await;
    }
}
