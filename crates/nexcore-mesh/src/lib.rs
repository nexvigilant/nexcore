//! # nexcore-mesh — Runtime Mesh Networking
//!
//! Routed multi-hop adaptive networking with discovery, gossip, and resilience.
//!
//! ## Primitive Foundation (14/16 T1 coverage)
//!
//! | Primitive | Role in Mesh |
//! |-----------|-------------|
//! | λ Location | Node addresses, route destinations |
//! | μ Mapping | Routing tables, neighbor registries |
//! | σ Sequence | Hop paths, message relay chains |
//! | ∂ Boundary | TTL limits, circuit breakers, error boundaries |
//! | ρ Recursion | Multi-hop relay, self-correcting resilience |
//! | ν Frequency | Heartbeats, discovery/gossip intervals |
//! | ∃ Existence | Node liveness, neighbor verification |
//! | ς State | Node lifecycle, circuit breaker state |
//! | κ Comparison | Route quality scoring, best-path selection |
//! | → Causality | Message propagation, causal relay chains |
//! | π Persistence | Route caching, mesh snapshots |
//! | Σ Sum | Gradient aggregation, chemotactic routing |
//! | N Quantity | Route quality metrics, gradient concentration |
//! | ∅ Void | Error boundaries, null handling |
//!
//! ## Transfer Primitive Reuse
//!
//! - [`nexcore_primitives::transfer::TopologicalAddress`] — hierarchical mesh addressing
//! - [`nexcore_primitives::transfer::CircuitBreaker`] — per-neighbor link protection
//! - [`nexcore_primitives::transfer::Homeostasis`] — target neighbor count maintenance
//! - [`nexcore_primitives::transfer::FeedbackLoop`] — discovery rate adjustment
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │                 Node (T3: ς)            │
//! │  ┌─────────────┐  ┌──────────────────┐  │
//! │  │ Neighbors    │  │  RoutingTable    │  │
//! │  │ (T3: μ)      │  │  (T3: μ)         │  │
//! │  │ DashMap +    │  │  DashMap +        │  │
//! │  │ CircuitBkr   │  │  quality-sorted   │  │
//! │  └─────────────┘  └──────────────────┘  │
//! │  ┌─────────────┐  ┌─────────────────┐   │
//! │  │ Discovery    │  │ Gossip          │   │
//! │  │ (T3: ν)      │  │ (T3: ν)         │   │
//! │  └─────────────┘  └─────────────────┘   │
//! │  ┌──────────────────────────────────┐   │
//! │  │ Resilience (T3: ρ)               │   │
//! │  │ Homeostasis + FeedbackLoop       │   │
//! │  └──────────────────────────────────┘   │
//! └─────────────────────────────────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod chemotaxis;
pub mod discovery;
pub mod error;
pub mod gossip;
pub mod grounding;
pub mod neighbor;
pub mod node;
pub mod persistence;
pub mod resilience;
pub mod routing;
pub mod runtime;
pub mod security;
pub mod topology;

// Re-export core types at crate root
pub use chemotaxis::{ChemotacticRouteSelector, ChemotacticRouter, MeshGradientConfig};
pub use discovery::{DiscoveryAction, DiscoveryConfig, DiscoveryLoop, DiscoveryMessage};
pub use error::MeshError;
pub use gossip::{GossipConfig, GossipLoop, GossipMessage, GossipRouteEntry};
pub use neighbor::{Neighbor, NeighborRegistry};
pub use node::{MeshConfig, MeshMessage, MessageAction, MessageKind, Node, NodeState};
pub use persistence::{MeshSnapshot, PersistenceConfig, SnapshotError, SnapshotStore};
pub use resilience::{ResilienceAction, ResilienceConfig, ResilienceLoop, ResilienceState};
pub use routing::{Route, RoutingTable};
pub use runtime::{MeshEvent, MeshHandle, MeshRuntime};
pub use security::{
    CertValidationMode, PeerIdentity, SecurityGate, SecurityPolicy, SecurityVerdict, TlsTier,
};
pub use topology::{AddressExt, Path, RouteQuality};

// Re-export transfer primitives used by this crate
pub use nexcore_primitives::transfer::{
    BreakerState, CircuitBreaker, FeedbackLoop, Homeostasis, TopologicalAddress,
};
