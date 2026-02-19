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
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod geometry;
pub mod math;
pub mod pipeline;

// Top-level re-exports for ergonomic use
pub use geometry::{Mesh, Triangle, Vertex};
pub use math::{Color, Mat3, Mat4, Vec2, Vec3, Vec4};
pub use pipeline::{Framebuffer, Viewport};
