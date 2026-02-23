// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from this crate's primitives.
//!
//! Composites combine two or more of the core networking abstractions
//! (`Interface`, `Connection`, `DnsResolver`, `Firewall`, `RoutingTable`,
//! `NetworkMonitor`, `CertStore`) into higher-level structures that span
//! multiple modules.
//!
//! ## Design Philosophy
//!
//! Each composite should satisfy the T2-C (cross-domain composite) tier:
//! it must reference at least two distinct primitive symbols from the module
//! header (e.g. `Σ + ∂ + ς`) and must not duplicate logic already present
//! in the constituent modules.
//!
//! ## Status
//!
//! Currently empty — composites will be added as the crate evolves.

// No items yet. Composites are added here as the networking stack grows.
