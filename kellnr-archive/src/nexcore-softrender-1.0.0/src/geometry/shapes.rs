//! Shape generators: rect, circle, line, rounded_rect
//!
//! Each function returns a Mesh of triangles. All shapes are flat (z=0).

use super::mesh::{Mesh, Triangle};
use super::vertex::Vertex;
use crate::math::Color;

/// Axis-aligned rectangle: 2 triangles
pub fn rect(x: f64, y: f64, w: f64, h: f64, color: Color) -> Mesh {
    let tl = Vertex::colored(x, y, color);
    let tr = Vertex::colored(x + w, y, color);
    let bl = Vertex::colored(x, y + h, color);
    let br = Vertex::colored(x + w, y + h, color);

    Mesh {
        triangles: vec![Triangle::new(tl, tr, bl), Triangle::new(tr, br, bl)],
    }
}

/// Rectangle with per-corner colors (gradient)
pub fn rect_gradient(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    tl_color: Color,
    tr_color: Color,
    bl_color: Color,
    br_color: Color,
) -> Mesh {
    let tl = Vertex::colored(x, y, tl_color);
    let tr = Vertex::colored(x + w, y, tr_color);
    let bl = Vertex::colored(x, y + h, bl_color);
    let br = Vertex::colored(x + w, y + h, br_color);

    Mesh {
        triangles: vec![Triangle::new(tl, tr, bl), Triangle::new(tr, br, bl)],
    }
}

/// Circle approximated as triangle fan (center + N segments)
pub fn circle(cx: f64, cy: f64, radius: f64, segments: u32, color: Color) -> Mesh {
    let segments = segments.max(3);
    let center = Vertex::colored(cx, cy, color);
    let mut triangles = Vec::with_capacity(segments as usize);

    for i in 0..segments {
        let a0 = 2.0 * core::f64::consts::PI * (i as f64) / (segments as f64);
        let a1 = 2.0 * core::f64::consts::PI * ((i + 1) as f64) / (segments as f64);

        let v0 = Vertex::colored(cx + radius * a0.cos(), cy + radius * a0.sin(), color);
        let v1 = Vertex::colored(cx + radius * a1.cos(), cy + radius * a1.sin(), color);

        triangles.push(Triangle::new(center, v0, v1));
    }

    Mesh { triangles }
}

/// Line segment with thickness (rendered as a rect oriented along the line)
pub fn line(x0: f64, y0: f64, x1: f64, y1: f64, thickness: f64, color: Color) -> Mesh {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < f64::EPSILON {
        return Mesh::new();
    }

    // Perpendicular unit vector × half-thickness
    let nx = -dy / len * thickness * 0.5;
    let ny = dx / len * thickness * 0.5;

    let v0 = Vertex::colored(x0 + nx, y0 + ny, color);
    let v1 = Vertex::colored(x0 - nx, y0 - ny, color);
    let v2 = Vertex::colored(x1 + nx, y1 + ny, color);
    let v3 = Vertex::colored(x1 - nx, y1 - ny, color);

    Mesh {
        triangles: vec![Triangle::new(v0, v2, v1), Triangle::new(v2, v3, v1)],
    }
}

/// Rounded rectangle: 1 inner rect + 4 side rects + 4 corner arcs
pub fn rounded_rect(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    radius: f64,
    color: Color,
    corner_segments: u32,
) -> Mesh {
    let r = radius.min(w / 2.0).min(h / 2.0);
    let segs = corner_segments.max(2);
    let mut mesh = Mesh::new();

    // Inner cross (horizontal + vertical rects avoiding corners)
    mesh.extend(&rect(x + r, y, w - 2.0 * r, h, color));
    mesh.extend(&rect(x, y + r, r, h - 2.0 * r, color));
    mesh.extend(&rect(x + w - r, y + r, r, h - 2.0 * r, color));

    // Four corner arcs
    let corners = [
        (
            x + r,
            y + r,
            core::f64::consts::PI,
            core::f64::consts::FRAC_PI_2 * 3.0,
        ), // top-left
        (
            x + w - r,
            y + r,
            core::f64::consts::FRAC_PI_2 * 3.0,
            2.0 * core::f64::consts::PI,
        ), // top-right
        (
            x + r,
            y + h - r,
            core::f64::consts::FRAC_PI_2,
            core::f64::consts::PI,
        ), // bottom-left
        (x + w - r, y + h - r, 0.0, core::f64::consts::FRAC_PI_2), // bottom-right
    ];

    for &(cx, cy, start_angle, end_angle) in &corners {
        let center = Vertex::colored(cx, cy, color);
        for i in 0..segs {
            let t0 = i as f64 / segs as f64;
            let t1 = (i + 1) as f64 / segs as f64;
            let a0 = start_angle + (end_angle - start_angle) * t0;
            let a1 = start_angle + (end_angle - start_angle) * t1;

            let v0 = Vertex::colored(cx + r * a0.cos(), cy + r * a0.sin(), color);
            let v1 = Vertex::colored(cx + r * a1.cos(), cy + r * a1.sin(), color);

            mesh.push(Triangle::new(center, v0, v1));
        }
    }

    mesh
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_has_2_triangles() {
        let m = rect(0.0, 0.0, 100.0, 50.0, Color::RED);
        assert_eq!(m.triangle_count(), 2);
    }

    #[test]
    fn circle_segment_count() {
        let m = circle(0.0, 0.0, 10.0, 32, Color::GREEN);
        assert_eq!(m.triangle_count(), 32);
    }

    #[test]
    fn circle_minimum_segments() {
        let m = circle(0.0, 0.0, 10.0, 1, Color::GREEN);
        assert_eq!(m.triangle_count(), 3); // clamped to min 3
    }

    #[test]
    fn line_zero_length_empty() {
        let m = line(5.0, 5.0, 5.0, 5.0, 2.0, Color::WHITE);
        assert_eq!(m.triangle_count(), 0);
    }

    #[test]
    fn line_has_2_triangles() {
        let m = line(0.0, 0.0, 100.0, 0.0, 2.0, Color::WHITE);
        assert_eq!(m.triangle_count(), 2);
    }

    #[test]
    fn rounded_rect_has_triangles() {
        let m = rounded_rect(0.0, 0.0, 200.0, 100.0, 10.0, Color::BLUE, 4);
        // 3 rects (6 tris) + 4 corners × 4 segments = 22
        assert!(m.triangle_count() >= 22);
    }
}
