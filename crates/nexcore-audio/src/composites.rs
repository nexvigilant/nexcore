// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from this crate's primitives.
//!
//! Composites combine two or more of the core audio abstractions
//! (`AudioBuffer`, `AudioDevice`, `AudioStream`, `Mixer`, `SampleFormat`,
//! `CodecId`) into higher-level structures that span multiple modules.
//!
//! ## Design Philosophy
//!
//! Each composite should satisfy the T2-C (cross-domain composite) tier:
//! it must reference at least two distinct primitive symbols from the module
//! header (e.g. `σ + ∂ + Σ`) and must not duplicate logic already present
//! in the constituent modules.
//!
//! Typical composites for this crate include:
//!
//! - **`AudioPipeline`**: links a `Device` → `Stream` → `Mixer` into a single
//!   managed entity with unified start/stop lifecycle.
//! - **`TranscodingSession`**: pairs a source `CodecId` with a target `CodecId`
//!   and an intermediate `AudioBuffer` for format conversion.
//!
//! ## Status
//!
//! Currently empty — composites will be added as the crate evolves.

// No items yet. Composites are added here as the audio stack grows.
