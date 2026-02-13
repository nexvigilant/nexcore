//! Fragment processing: per-pixel color computation
//!
//! After rasterization determines which pixels are inside a triangle,
//! the fragment stage computes the final color for each pixel using
//! barycentric interpolation of vertex attributes.

use crate::geometry::Vertex;
use crate::math::Color;

/// Fragment: a pixel candidate with interpolated attributes
#[derive(Debug, Clone, Copy)]
pub struct Fragment {
    pub x: u32,
    pub y: u32,
    pub color: Color,
    pub depth: f64,
}

/// Compute fragment color from barycentric weights
pub fn shade_fragment(
    x: u32,
    y: u32,
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    w0: f64,
    w1: f64,
    w2: f64,
) -> Fragment {
    let interp = Vertex::interpolate(v0, v1, v2, w0, w1, w2);
    Fragment {
        x,
        y,
        color: interp.color.clamp(),
        depth: interp.position.z,
    }
}

/// Flat shading: all pixels get the same color (v0's color)
pub fn shade_flat(x: u32, y: u32, v0: &Vertex) -> Fragment {
    Fragment {
        x,
        y,
        color: v0.color,
        depth: v0.position.z,
    }
}
