//! Vertex: position + color + UV coordinates
//!
//! The fundamental unit of geometry. Every shape decomposes to vertices.

use crate::math::{Color, Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Color,
    pub uv: Vec2,
}

impl Vertex {
    pub fn new(position: Vec3, color: Color, uv: Vec2) -> Self {
        Self {
            position,
            color,
            uv,
        }
    }

    /// Simple vertex with position and color, UV defaults to (0,0)
    pub fn colored(x: f64, y: f64, color: Color) -> Self {
        Self {
            position: Vec3::new(x, y, 0.0),
            color,
            uv: Vec2::ZERO,
        }
    }

    /// Vertex at 2D position with white color
    pub fn pos2d(x: f64, y: f64) -> Self {
        Self::colored(x, y, Color::WHITE)
    }

    /// Barycentric interpolation of three vertices
    pub fn interpolate(v0: &Self, v1: &Self, v2: &Self, w0: f64, w1: f64, w2: f64) -> Self {
        Self {
            position: Vec3::new(
                v0.position.x * w0 + v1.position.x * w1 + v2.position.x * w2,
                v0.position.y * w0 + v1.position.y * w1 + v2.position.y * w2,
                v0.position.z * w0 + v1.position.z * w1 + v2.position.z * w2,
            ),
            color: Color::rgba(
                v0.color.r * w0 + v1.color.r * w1 + v2.color.r * w2,
                v0.color.g * w0 + v1.color.g * w1 + v2.color.g * w2,
                v0.color.b * w0 + v1.color.b * w1 + v2.color.b * w2,
                v0.color.a * w0 + v1.color.a * w1 + v2.color.a * w2,
            ),
            uv: Vec2::new(
                v0.uv.x * w0 + v1.uv.x * w1 + v2.uv.x * w2,
                v0.uv.y * w0 + v1.uv.y * w1 + v2.uv.y * w2,
            ),
        }
    }
}
