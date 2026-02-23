// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from `nexcore-shell` primitives.
//!
//! ## Shell Composite Structure
//!
//! The shell's top-level `Shell` struct composes:
//! - `Compositor` — display output (from nexcore-compositor)
//! - `ShellLayout` — screen region definitions (∂ + λ)
//! - `AppRegistry` — app lifecycle tracking (∃ + ς)
//! - `InputProcessor` — input → action mapping (μ + σ)
//! - `NotificationManager` — priority queue (σ + κ)
//!
//! | Type | Tier | Primitives |
//! |------|------|-----------|
//! | `ShellState` | T2-P | ς + ∂ |
//! | `AppState` | T1 | ς |
//! | `AppId` | T2-P | ∃ |
//! | `App` | T2-C | ∃ + ς + σ |
//! | `Shell` | T3 | μ + σ + ς + ∂ |
//!
//! Additional composite aliases will be added as shell cross-cutting
//! concerns are identified.

// Currently empty — composites will be added as the crate evolves.
