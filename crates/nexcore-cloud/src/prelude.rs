//! # Prelude
//!
//! Convenient re-exports for `use nexcore_cloud::prelude::*`.

// T1 Universal
pub use crate::primitives::{
    Convergence, FeedbackLoop, Idempotency, Identity, Immutability, Threshold,
};

// T2-P Cross-Domain
pub use crate::primitives::{
    Compute, Elasticity, Encryption, HealthCheck, IsolationBoundary, Lease, Metering, NetworkLink,
    Permission, Queue, Replication, ResourcePool, Routing, Storage,
};

// T2-C Composites
pub use crate::composites::{
    AutoScaling, EventualConsistency, Iam, LoadBalancer, PayPerUse, ReservedCapacity,
    SecretsManagement, SpotPricing, Tenancy, VirtualMachine,
};

// T3 Domain-Specific
pub use crate::service_models::{Container, Iaas, Paas, Saas, Serverless};

// Grounding (re-export trait)
pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
pub use nexcore_lex_primitiva::tier::Tier;

// Transfer
pub use crate::transfer::{
    TransferMapping, transfer_confidence, transfer_mappings, transfers_for_type,
};
