// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from `nexcore-os` primitives.
//!
//! ## OS Composite Structure
//!
//! The OS composes multiple domain subsystems (PAL, STOS, Cytokine IPC,
//! Clearance, Vault, Brain) into the top-level `NexCoreOs` kernel struct.
//! Each sub-domain is a T2-C or T3 type; the kernel itself is T3.
//!
//! | Type | Tier | Primitives |
//! |------|------|-----------|
//! | `BootPhase` | T2-P | σ + ς |
//! | `ServiceState` | T1 | ς |
//! | `ServiceId` | T2-P | ∃ + N |
//! | `SecurityLevel` | T2-P | ς + κ |
//! | `VaultState` | T1 | ς |
//! | `NexCoreOs` | T3 | Σ of all above |
//!
//! Composites will be added as the crate evolves and cross-subsystem
//! aggregate types (e.g., `SystemHealthSnapshot`) are identified.

// Currently empty — composites will be added as the crate evolves.
