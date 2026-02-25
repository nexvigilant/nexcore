//! nexcore-softrender: Pure-Rust software rasterizer
//!
//! First-principles rendering from math to pixels. Zero dependencies.
//!
//! # Pipeline
//! ```text
//! Geometry (vertices) → Transform (matrices) → Rasterize (edge functions) → Framebuffer (pixels)
//! ```
//!
//! # Architecture
//! - `math` — Vec2/3/4, Mat3/4, transforms, color (pure computation)
//! - `geometry` — Vertex, Triangle, Mesh, shape generators
//! - `pipeline` — Viewport, rasterizer (edge functions), fragment shading, framebuffer

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "Software rendering requires frequent pixel/math conversions and low-level indexing for performance"
)]
#![allow(
    clippy::too_many_arguments,
    clippy::many_single_char_names,
    reason = "Rendering pipelines and math functions often have many parameters (x, y, z, w, r, g, b, a)"
)]

pub mod geometry;
pub mod math;
pub mod pipeline;

// Top-level re-exports for ergonomic use
pub use geometry::{Mesh, Triangle, Vertex};
pub use math::{Color, Mat3, Mat4, Vec2, Vec3, Vec4};
pub use pipeline::{Framebuffer, Viewport};
