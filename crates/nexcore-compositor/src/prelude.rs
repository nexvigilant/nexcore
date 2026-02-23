// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Compositor Prelude
//!
//! Convenience re-exports of the most-used types from `nexcore-compositor`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_compositor::prelude::*;
//! ```
//!
//! Brings into scope the main compositor, all surface management types,
//! render pipeline, tiling engine, and the Lex Primitiva grounding infrastructure.

// Core compositor
pub use crate::compositor::{Compositor, CompositorState};

// Compositor mode
pub use crate::mode::CompositorMode;

// Surface management
pub use crate::surface::{Rect, Surface, SurfaceId, Visibility};

// Rendering
pub use crate::render::{FrameStats, RenderCommand, RenderPipeline};

// Tiling
pub use crate::tiling::{SplitDirection, SplitNode, TilingEngine, TilingLayout};

// Decoration
pub use crate::decoration::{DecorationFrame, DecorationRenderer, DecorationTheme};

// Input routing
pub use crate::input::{DecorationZone, GlobalAction, InputRouter, InputTarget, ResizeEdge};

// Grounding
pub use crate::primitives::{GroundsTo, LexPrimitiva, PrimitiveComposition, Tier};
