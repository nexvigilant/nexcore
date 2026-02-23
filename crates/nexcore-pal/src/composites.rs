// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from `nexcore-pal` primitives.
//!
//! ## PAL Composite Structure
//!
//! The PAL composes hardware subsystems into the top-level `Platform` trait.
//! Each concrete platform (e.g., `LinuxPlatform`) is a T3 composite binding
//! all subsystems together.
//!
//! Tier analysis:
//! - `Resolution`: T2-P (N + ∂)
//! - `InputEvent`: T2-C (σ + ∃ + μ)
//! - `PowerState`: T2-P (ς + N)
//! - `Platform` impl: T3 (Σ of all subsystems)
//!
//! Composites will be added as the crate evolves and cross-subsystem
//! aggregate types are identified.

// Currently empty — composites will be added as the crate evolves.
