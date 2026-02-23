// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from `nexcore-compositor` primitives.
//!
//! ## Compositor Composite Structure
//!
//! The compositor's top-level `Compositor` struct composes:
//! - `SurfaceManager` — ordered collection of surfaces (σ + ∂)
//! - `RenderPipeline` — command-based rendering pipeline (μ + σ)
//! - `TilingEngine` — BSP window layout (ρ + ∂ + μ)
//! - `InputRouter` — focus-based input dispatch (μ + ∃)
//! - `DecorationRenderer` — window chrome generator (∂ + λ + μ)
//!
//! | Type | Tier | Primitives |
//! |------|------|-----------|
//! | `Rect` | T2-P | ∂ + λ |
//! | `SurfaceId` | T2-P | ∃ + N |
//! | `Surface` | T2-C | ∂ + λ + ∃ + π |
//! | `CompositorMode` | T2-P | κ + ∂ |
//! | `Compositor` | T3 | Σ + μ + σ + ς |
//!
//! Additional composite aliases will be added as display-server
//! cross-cutting concerns are identified.

// Currently empty — composites will be added as the crate evolves.
