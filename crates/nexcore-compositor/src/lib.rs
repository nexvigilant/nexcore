// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore Compositor — Display Server
//!
//! The display server for NexCore OS across three form factors.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │            Compositor Modes                   │
//! │  Watch: SingleApp │ Phone: Stack │ Desktop: WM │
//! ├──────────────────────────────────────────────┤
//! │            Surface Manager                    │
//! │  Surface creation │ Z-order │ Focus tracking  │
//! ├──────────────────────────────────────────────┤
//! │            Rendering Backend                  │
//! │  Software (test) │ GPU (wgpu, future)         │
//! ├──────────────────────────────────────────────┤
//! │            PAL Display Trait                   │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Compositor Modes
//!
//! | Mode | Form Factor | Windows | Input Routing |
//! |------|-------------|---------|---------------|
//! | SingleApp | Watch | 1 full-screen | Direct to app |
//! | AppStack | Phone | N full-screen, swipe | Gesture-based |
//! | WindowManager | Desktop | N floating/tiled | Focus-based |
//!
//! ## Primitive Grounding
//!
//! | Component | Primitives | Role |
//! |-----------|------------|------|
//! | Surface | ∂ + λ | Bounded region at location |
//! | Z-order | σ + κ | Ordered comparison (front-to-back) |
//! | Focus | ∃ + ς | Active surface state |
//! | Compositing | Σ + μ | Sum of surfaces mapped to display |

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod composites;
pub mod compositor;
pub mod decoration;
pub mod grounding;
pub mod input;
pub mod mode;
pub mod prelude;
pub mod primitives;
pub mod render;
pub mod surface;
pub mod tiling;
pub mod transfer;

// Re-export main types
pub use compositor::Compositor;
pub use mode::CompositorMode;
pub use surface::{Surface, SurfaceId};

// Re-export new modules
pub use decoration::{DecorationFrame, DecorationRenderer, DecorationTheme};
pub use input::{DecorationZone, GlobalAction, InputRouter, InputTarget, ResizeEdge};
pub use render::{FrameStats, RenderCommand, RenderPipeline};
pub use tiling::{SplitDirection, SplitNode, TilingEngine, TilingLayout};
