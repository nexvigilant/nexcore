//! # Capability 29: Communications Act (Inter-Agent Protocols)
//!
//! Implementation of the Communications Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Inter-Agent Communication" and "Protocol Standards" of the Union.
//!
//! Matches 1:1 to the US Federal Communications Commission (FCC) mandate
//! to regulate interstate and international communications.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │            COMMUNICATIONS ACT (CAP-029)                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  PROTOCOL LAYER                                              │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
//! │  │  MCP    │  │JSON-RPC │  │ Events  │                      │
//! │  │Protocol │  │Protocol │  │Protocol │                      │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                      │
//! │       │            │            │                            │
//! │       ▼            ▼            ▼                            │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          MESSAGE ROUTING ENGINE             │            │
//! │  │  • Channel selection                        │            │
//! │  │  • Priority queuing                         │            │
//! │  │  • Delivery guarantees                      │            │
//! │  └────────────────────┬────────────────────────┘            │
//! │                       │                                      │
//! │                       ▼                                      │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          TRANSMISSION QUALITY               │            │
//! │  │  • Signal clarity metrics                   │            │
//! │  │  • Bandwidth allocation                     │            │
//! │  │  • Latency tracking                         │            │
//! │  └─────────────────────────────────────────────┘            │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// T1 PRIMITIVES (Universal)
// ============================================================================

/// T1: ProtocolType - Communication protocol standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolType {
    /// Model Context Protocol (Claude Code tools).
    Mcp,
    /// JSON-RPC 2.0 (standard API calls).
    JsonRpc,
    /// Event-driven (pub/sub, webhooks).
    Event,
    /// Direct function call (in-process).
    Direct,
    /// REST HTTP (external services).
    Rest,
}

impl ProtocolType {
    /// Get typical latency in milliseconds.
    pub fn typical_latency_ms(&self) -> u32 {
        match self {
            Self::Direct => 1,
            Self::Mcp => 50,
            Self::JsonRpc => 100,
            Self::Event => 10,
            Self::Rest => 200,
        }
    }

    /// Get reliability score (0.0-1.0).
    pub fn reliability(&self) -> f64 {
        match self {
            Self::Direct => 1.0,
            Self::Mcp => 0.99,
            Self::JsonRpc => 0.95,
            Self::Event => 0.90,
            Self::Rest => 0.85,
        }
    }
}

/// T1: ChannelType - Message delivery semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Point-to-point, guaranteed delivery.
    Unicast,
    /// One-to-many, at-most-once.
    Broadcast,
    /// Request-response pattern.
    RequestReply,
    /// Fire-and-forget.
    FireForget,
}

/// T1: DeliveryGuarantee - Message delivery semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryGuarantee {
    /// No guarantee.
    BestEffort,
    /// At least once (may duplicate).
    AtLeastOnce,
    /// At most once (may lose).
    AtMostOnce,
    /// Exactly once (requires state).
    ExactlyOnce,
}

// ============================================================================
// T2-P PRIMITIVES (Cross-Domain)
// ============================================================================

/// T2-P: SignalClarity - The quantified quality of transmission.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SignalClarity(pub f64);

impl SignalClarity {
    /// Create new clarity (clamped 0.0-1.0).
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Is clarity acceptable (>0.9)?
    pub fn is_acceptable(&self) -> bool {
        self.0 >= 0.9
    }

    /// Inner value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// T2-P: Bandwidth - Tokens per second capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Bandwidth(pub u64);

impl Bandwidth {
    /// Create new bandwidth.
    pub fn new(tokens_per_sec: u64) -> Self {
        Self(tokens_per_sec)
    }

    /// High bandwidth (>10K tokens/sec).
    pub fn is_high(&self) -> bool {
        self.0 >= 10_000
    }

    /// Inner value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// T2-P: Latency - Round-trip time in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Latency(pub u32);

impl Latency {
    /// Create new latency.
    pub fn new(ms: u32) -> Self {
        Self(ms)
    }

    /// Is latency low (<100ms)?
    pub fn is_low(&self) -> bool {
        self.0 < 100
    }

    /// Inner value.
    pub fn value(&self) -> u32 {
        self.0
    }
}

// ============================================================================
// T2-C COMPOSITES (Cross-Domain)
// ============================================================================

/// T2-C: ProtocolAssignment - A formal communication standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolAssignment {
    /// The protocol type.
    pub protocol: ProtocolType,
    /// The channel type.
    pub channel: ChannelType,
    /// Delivery guarantee.
    pub guarantee: DeliveryGuarantee,
    /// Minimum clarity threshold.
    pub min_clarity: SignalClarity,
    /// Allocated bandwidth.
    pub bandwidth: Bandwidth,
    /// Maximum allowed latency.
    pub max_latency: Latency,
}

impl Default for ProtocolAssignment {
    fn default() -> Self {
        Self {
            protocol: ProtocolType::Mcp,
            channel: ChannelType::RequestReply,
            guarantee: DeliveryGuarantee::AtLeastOnce,
            min_clarity: SignalClarity::new(0.95),
            bandwidth: Bandwidth::new(5_000),
            max_latency: Latency::new(500),
        }
    }
}

/// T2-C: Message - Inter-agent message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID.
    pub id: String,
    /// Sender agent ID.
    pub from: String,
    /// Recipient agent ID.
    pub to: String,
    /// Protocol used.
    pub protocol: ProtocolType,
    /// Message payload (JSON).
    pub payload: String,
    /// Timestamp (Unix ms).
    pub timestamp: i64,
    /// Time-to-live in seconds.
    pub ttl: u32,
}

/// T2-C: TransmissionMetrics - Quality metrics for a transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionMetrics {
    /// Actual latency observed.
    pub latency: Latency,
    /// Signal clarity achieved.
    pub clarity: SignalClarity,
    /// Message size in bytes.
    pub size_bytes: usize,
    /// Was delivery successful?
    pub delivered: bool,
    /// Retry count.
    pub retries: u32,
}

/// T2-C: ChannelStatus - Status of a communication channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatus {
    /// Channel identifier.
    pub channel_id: String,
    /// Current bandwidth utilization.
    pub utilization: f64,
    /// Messages in queue.
    pub queue_depth: u32,
    /// Is channel healthy?
    pub healthy: bool,
    /// Average latency.
    pub avg_latency: Latency,
}

// ============================================================================
// T3 DOMAIN-SPECIFIC (CommunicationsAct)
// ============================================================================

/// T3: CommunicationsAct - Capability 29 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationsAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the communications engine is active.
    pub protocols_active: bool,
    /// Protocol assignments by task type.
    assignments: HashMap<String, ProtocolAssignment>,
    /// Active channels.
    channels: HashMap<String, ChannelStatus>,
    /// Message history (last N messages).
    message_log: Vec<Message>,
    /// Log size limit.
    log_limit: usize,
}

impl Default for CommunicationsAct {
    fn default() -> Self {
        Self::new()
    }
}

impl CommunicationsAct {
    /// Creates a new instance.
    pub fn new() -> Self {
        let mut act = Self {
            id: "CAP-029".into(),
            protocols_active: true,
            assignments: HashMap::new(),
            channels: HashMap::new(),
            message_log: Vec::new(),
            log_limit: 100,
        };
        act.register_default_protocols();
        act
    }

    /// Register default protocol assignments.
    fn register_default_protocols(&mut self) {
        // MCP for tool calls
        self.assignments.insert(
            "tool_call".into(),
            ProtocolAssignment {
                protocol: ProtocolType::Mcp,
                channel: ChannelType::RequestReply,
                guarantee: DeliveryGuarantee::ExactlyOnce,
                min_clarity: SignalClarity::new(0.99),
                bandwidth: Bandwidth::new(10_000),
                max_latency: Latency::new(1000),
            },
        );

        // Events for notifications
        self.assignments.insert(
            "notification".into(),
            ProtocolAssignment {
                protocol: ProtocolType::Event,
                channel: ChannelType::Broadcast,
                guarantee: DeliveryGuarantee::AtMostOnce,
                min_clarity: SignalClarity::new(0.9),
                bandwidth: Bandwidth::new(1_000),
                max_latency: Latency::new(100),
            },
        );

        // Direct for in-process
        self.assignments.insert(
            "internal".into(),
            ProtocolAssignment {
                protocol: ProtocolType::Direct,
                channel: ChannelType::Unicast,
                guarantee: DeliveryGuarantee::ExactlyOnce,
                min_clarity: SignalClarity::new(1.0),
                bandwidth: Bandwidth::new(100_000),
                max_latency: Latency::new(10),
            },
        );
    }

    /// Get protocol assignment for a task type.
    pub fn get_protocol(&self, task_type: &str) -> ProtocolAssignment {
        self.assignments.get(task_type).cloned().unwrap_or_default()
    }

    /// Assign a protocol for a task type.
    pub fn assign_protocol(
        &mut self,
        task_type: &str,
        assignment: ProtocolAssignment,
    ) -> Measured<ProtocolAssignment> {
        self.assignments
            .insert(task_type.to_string(), assignment.clone());
        let confidence = assignment.protocol.reliability();
        Measured::uncertain(assignment, Confidence::new(confidence))
    }

    /// Route a message through the appropriate channel.
    pub fn route_message(&mut self, message: Message) -> Measured<TransmissionMetrics> {
        let protocol = message.protocol;

        // Log the message
        self.message_log.push(message.clone());
        if self.message_log.len() > self.log_limit {
            self.message_log.remove(0);
        }

        // Simulate transmission metrics
        let metrics = TransmissionMetrics {
            latency: Latency::new(protocol.typical_latency_ms()),
            clarity: SignalClarity::new(protocol.reliability()),
            size_bytes: message.payload.len(),
            delivered: true,
            retries: 0,
        };

        Measured::uncertain(metrics, Confidence::new(protocol.reliability()))
    }

    /// Get channel status.
    pub fn get_channel_status(&self, channel_id: &str) -> Option<&ChannelStatus> {
        self.channels.get(channel_id)
    }

    /// Register a channel.
    pub fn register_channel(&mut self, status: ChannelStatus) {
        self.channels.insert(status.channel_id.clone(), status);
    }

    /// Get recent message log.
    pub fn get_message_log(&self, limit: usize) -> Vec<&Message> {
        self.message_log.iter().rev().take(limit).collect()
    }

    /// Recommend protocol for use case.
    pub fn recommend_protocol(
        &self,
        needs_guarantee: bool,
        low_latency: bool,
        is_broadcast: bool,
    ) -> ProtocolType {
        match (needs_guarantee, low_latency, is_broadcast) {
            (true, true, false) => ProtocolType::Direct,
            (true, false, false) => ProtocolType::Mcp,
            (false, true, true) => ProtocolType::Event,
            (false, false, true) => ProtocolType::Event,
            (true, _, true) => ProtocolType::JsonRpc,
            _ => ProtocolType::Mcp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_latency() {
        assert!(ProtocolType::Direct.typical_latency_ms() < ProtocolType::Mcp.typical_latency_ms());
        assert!(ProtocolType::Mcp.typical_latency_ms() < ProtocolType::Rest.typical_latency_ms());
    }

    #[test]
    fn test_signal_clarity() {
        let good = SignalClarity::new(0.95);
        assert!(good.is_acceptable());

        let poor = SignalClarity::new(0.8);
        assert!(!poor.is_acceptable());
    }

    #[test]
    fn test_protocol_assignment() {
        let comm = CommunicationsAct::new();

        let tool_proto = comm.get_protocol("tool_call");
        assert_eq!(tool_proto.protocol, ProtocolType::Mcp);
        assert_eq!(tool_proto.guarantee, DeliveryGuarantee::ExactlyOnce);
    }

    #[test]
    fn test_message_routing() {
        let mut comm = CommunicationsAct::new();

        let msg = Message {
            id: "msg-1".into(),
            from: "agent-a".into(),
            to: "agent-b".into(),
            protocol: ProtocolType::Mcp,
            payload: r#"{"action":"test"}"#.into(),
            timestamp: nexcore_chrono::DateTime::now().timestamp_millis(),
            ttl: 60,
        };

        let result = comm.route_message(msg);
        assert!(result.value.delivered);
        assert!(result.value.clarity.is_acceptable());
    }

    #[test]
    fn test_protocol_recommendation() {
        let comm = CommunicationsAct::new();

        // Needs guarantee + low latency = Direct
        assert_eq!(
            comm.recommend_protocol(true, true, false),
            ProtocolType::Direct
        );

        // Broadcast + no guarantee = Event
        assert_eq!(
            comm.recommend_protocol(false, true, true),
            ProtocolType::Event
        );
    }
}
