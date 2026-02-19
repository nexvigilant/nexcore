//! # NexVigilant Core — cloud
//!
//! Cloud computing primitives grounded to Lex Primitiva.
//!
//! ## Type Inventory (35 types)
//!
//! | Tier | Count | Description |
//! |------|-------|-------------|
//! | T1   | 6     | Universal primitives (Identity, Threshold, FeedbackLoop, Idempotency, Immutability, Convergence) |
//! | T2-P | 14    | Cross-domain primitives (Compute, Storage, NetworkLink, IsolationBoundary, Permission, ResourcePool, Metering, Replication, Routing, Lease, Encryption, Queue, HealthCheck, Elasticity) |
//! | T2-C | 10    | Composites (VirtualMachine, LoadBalancer, AutoScaling, Iam, EventualConsistency, Tenancy, PayPerUse, ReservedCapacity, SpotPricing, SecretsManagement) |
//! | T3   | 5     | Domain-specific (Container, Iaas, Paas, Saas, Serverless) |
//!
//! ## Grounding Table
//!
//! | Type | Lex Primitiva | Dominant | Confidence |
//! |------|--------------|----------|------------|
//! | Identity | ∃ | ∃ | 1.00 |
//! | Threshold | κ + ∂ | κ | 0.95 |
//! | FeedbackLoop | → + ρ | → | 0.92 |
//! | Idempotency | ρ + ς | ρ | 0.90 |
//! | Immutability | ∝ + π | ∝ | 0.95 |
//! | Convergence | → + σ | → | 0.90 |
//! | Compute | N + ν | N | 0.92 |
//! | Storage | π + N | π | 0.90 |
//! | NetworkLink | → + λ | → | 0.92 |
//! | IsolationBoundary | ∂ | ∂ | 1.00 |
//! | Permission | μ + ∂ | μ | 0.88 |
//! | ResourcePool | N + Σ | N | 0.90 |
//! | Metering | N + ν | N | 0.88 |
//! | Replication | ρ + π | ρ | 0.90 |
//! | Routing | μ + → | μ | 0.88 |
//! | Lease | ς + σ | ς | 0.85 |
//! | Encryption | ∂ + μ | ∂ | 0.85 |
//! | Queue | σ + ς | σ | 0.90 |
//! | HealthCheck | ∃ + ν | ∃ | 0.88 |
//! | Elasticity | N + ρ + ς | N | 0.85 |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_cloud::prelude::*;
//!
//! // Create a virtual machine
//! let vm = VirtualMachine::new(4.0, 3.0, 16.0, "user-1", 3600.0);
//! assert!(vm.is_active());
//!
//! // Check tier classification
//! assert!(matches!(VirtualMachine::tier(), Tier::T2Composite | Tier::T3DomainSpecific));
//!
//! // Get primitive composition
//! let comp = VirtualMachine::primitive_composition();
//! assert!(comp.dominant == Some(LexPrimitiva::Quantity));
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod composites;
pub mod grounding;
pub mod prelude;
pub mod primitives;
pub mod service_models;
pub mod transfer;

// Re-export at crate root for convenience
pub use composites::*;
pub use primitives::*;
pub use service_models::*;
