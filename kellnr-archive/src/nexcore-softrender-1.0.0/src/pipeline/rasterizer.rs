//! Rasterizer: triangle fill via edge functions
//!
//! The core algorithm. For each pixel in the triangle's bounding box,
//! evaluate three half-plane edge functions. All non-negative? Pixel is inside.
//! The barycentric coordinates fall out of the edge function values for free.

use super::fragment::{Fragment, shade_fragment};
use super::framebuffer::Framebuffer;
use crate::geometry::{Mesh, Triangle, Vertex};
use crate::math::{Color, Mat3, Vec2};

/// Edge function: positive if point P is on the left side of edge V0→V1
/// Returns twice the signed area of triangle (V0, V1, P)
#[inline]
fn edge(v0: Vec2, v1: Vec2, p: Vec2) -> f64 {
    (p.x - v0.x) * (v1.y - v0.y) - (p.y - v0.y) * (v1.x - v0.x)
}

/// Rasterize a single triangle to fragments
pub fn rasterize_triangle(
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    width: u32,
    height: u32,
) -> Vec<Fragment> {
    let p0 = Vec2::new(v0.position.x, v0.position.y);
    let p1 = Vec2::new(v1.position.x, v1.position.y);
    let p2 = Vec2::new(v2.position.x, v2.position.y);

    // Bounding box (clamped to framebuffer)
    let min_x = p0.x.min(p1.x).min(p2.x).max(0.0) as u32;
    let min_y = p0.y.min(p1.y).min(p2.y).max(0.0) as u32;
    let max_x = (p0.x.max(p1.x).max(p2.x).ceil() as u32).min(width.saturating_sub(1));
    let max_y = (p0.y.max(p1.y).max(p2.y).ceil() as u32).min(height.saturating_sub(1));

    // Total signed area (for barycentric normalization)
    let area = edge(p0, p1, p2);
    if area.abs() < f64::EPSILON {
        return Vec::new(); // degenerate triangle
    }
    let inv_area = 1.0 / area;

    let mut fragments = Vec::new();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec2::new(x as f64 + 0.5, y as f64 + 0.5); // pixel center

            let w0 = edge(p1, p2, p) * inv_area;
            let w1 = edge(p2, p0, p) * inv_area;
            let w2 = edge(p0, p1, p) * inv_area;

            // Inside test: all barycentric coords >= 0
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                fragments.push(shade_fragment(x, y, v0, v1, v2, w0, w1, w2));
            }
        }
    }

    fragments
}

/// Rasterize a triangle directly to framebuffer (more efficient — no fragment allocation)
pub fn rasterize_triangle_to_fb(fb: &mut Framebuffer, v0: &Vertex, v1: &Vertex, v2: &Vertex) {
    let p0 = Vec2::new(v0.position.x, v0.position.y);
    let p1 = Vec2::new(v1.position.x, v1.position.y);
    let p2 = Vec2::new(v2.position.x, v2.position.y);

    let min_x = p0.x.min(p1.x).min(p2.x).max(0.0) as u32;
    let min_y = p0.y.min(p1.y).min(p2.y).max(0.0) as u32;
    let max_x = (p0.x.max(p1.x).max(p2.x).ceil() as u32).min(fb.width.saturating_sub(1));
    let max_y = (p0.y.max(p1.y).max(p2.y).ceil() as u32).min(fb.height.saturating_sub(1));

    let area = edge(p0, p1, p2);
    if area.abs() < f64::EPSILON {
        return;
    }
    let inv_area = 1.0 / area;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec2::new(x as f64 + 0.5, y as f64 + 0.5);

            let w0 = edge(p1, p2, p) * inv_area;
            let w1 = edge(p2, p0, p) * inv_area;
            let w2 = edge(p0, p1, p) * inv_area;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let color = Color::rgba(
                    v0.color.r * w0 + v1.color.r * w1 + v2.color.r * w2,
                    v0.color.g * w0 + v1.color.g * w1 + v2.color.g * w2,
                    v0.color.b * w0 + v1.color.b * w1 + v2.color.b * w2,
                    v0.color.a * w0 + v1.color.a * w1 + v2.color.a * w2,
                );
                fb.blend_pixel(x, y, color);
            }
        }
    }
}

/// Apply a 2D transform to vertex positions, then rasterize to framebuffer
pub fn rasterize_triangle_transformed(
    fb: &mut Framebuffer,
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    transform: &Mat3,
) {
    let tp0 = transform.transform_point(v0.position.xy());
    let tp1 = transform.transform_point(v1.position.xy());
    let tp2 = transform.transform_point(v2.position.xy());

    let mut tv0 = *v0;
    let mut tv1 = *v1;
    let mut tv2 = *v2;
    tv0.position.x = tp0.x;
    tv0.position.y = tp0.y;
    tv1.position.x = tp1.x;
    tv1.position.y = tp1.y;
    tv2.position.x = tp2.x;
    tv2.position.y = tp2.y;

    rasterize_triangle_to_fb(fb, &tv0, &tv1, &tv2);
}

/// Rasterize an entire mesh to framebuffer
pub fn rasterize_mesh(fb: &mut Framebuffer, mesh: &Mesh) {
    for tri in &mesh.triangles {
        rasterize_triangle_to_fb(fb, &tri.v0, &tri.v1, &tri.v2);
    }
}

/// Rasterize mesh with a 2D transform applied
pub fn rasterize_mesh_transformed(fb: &mut Framebuffer, mesh: &Mesh, transform: &Mat3) {
    for tri in &mesh.triangles {
        rasterize_triangle_transformed(fb, &tri.v0, &tri.v1, &tri.v2, transform);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Vertex;
    use crate::math::Color;

    #[test]
    fn edge_function_sign() {
        // edge(v0,v1,p) = (p.x-v0.x)*(v1.y-v0.y) - (p.y-v0.y)*(v1.x-v0.x)
        // For v0=(0,0) v1=(10,0): = p.x*0 - p.y*10 = -10*p.y
        // Positive when p.y < 0 (below the rightward edge in math coords)
        let v0 = Vec2::new(0.0, 0.0);
        let v1 = Vec2::new(10.0, 0.0);
        let p_below = Vec2::new(5.0, -5.0);
        assert!(edge(v0, v1, p_below) > 0.0);
        let p_above = Vec2::new(5.0, 5.0);
        assert!(edge(v0, v1, p_above) < 0.0);
    }

    #[test]
    fn rasterize_small_triangle() {
        let v0 = Vertex::colored(10.0, 10.0, Color::RED);
        let v1 = Vertex::colored(30.0, 10.0, Color::GREEN);
        let v2 = Vertex::colored(20.0, 30.0, Color::BLUE);

        let fragments = rasterize_triangle(&v0, &v1, &v2, 100, 100);
        assert!(!fragments.is_empty());

        // All fragments should be within bounding box
        for f in &fragments {
            assert!(f.x >= 10 && f.x <= 30);
            assert!(f.y >= 10 && f.y <= 30);
        }
    }

    #[test]
    fn rasterize_degenerate_triangle() {
        let v0 = Vertex::colored(10.0, 10.0, Color::RED);
        let v1 = Vertex::colored(20.0, 20.0, Color::GREEN);
        let v2 = Vertex::colored(30.0, 30.0, Color::BLUE); // collinear

        let fragments = rasterize_triangle(&v0, &v1, &v2, 100, 100);
        assert!(fragments.is_empty());
    }

    #[test]
    fn rasterize_to_framebuffer() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::BLACK);

        let v0 = Vertex::colored(10.0, 10.0, Color::RED);
        let v1 = Vertex::colored(50.0, 10.0, Color::RED);
        let v2 = Vertex::colored(30.0, 50.0, Color::RED);

        rasterize_triangle_to_fb(&mut fb, &v0, &v1, &v2);

        // Center of triangle should be red-ish
        let c = fb.get_pixel(30, 25);
        assert!(c.r > 0.5);
    }

    #[test]
    fn rasterize_mesh_rect() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::BLACK);

        let mesh = crate::geometry::shapes::rect(10.0, 10.0, 30.0, 20.0, Color::GREEN);
        rasterize_mesh(&mut fb, &mesh);

        let inside = fb.get_pixel(25, 20);
        assert!(inside.g > 0.5);

        let outside = fb.get_pixel(0, 0);
        assert!(outside.g < 0.1);
    }

    #[test]
    fn rasterize_with_transform() {
        let mut fb = Framebuffer::new(200, 200);
        fb.clear(Color::BLACK);

        let mesh = crate::geometry::shapes::rect(0.0, 0.0, 20.0, 20.0, Color::BLUE);
        let t = crate::math::transform::translate_2d(50.0, 50.0);
        rasterize_mesh_transformed(&mut fb, &mesh, &t);

        // Translated position should have blue
        let c = fb.get_pixel(60, 60);
        assert!(c.b > 0.5);

        // Original position should be empty
        let c2 = fb.get_pixel(10, 10);
        assert!(c2.b < 0.1);
    }

    #[test]
    fn barycentric_color_interpolation() {
        let v0 = Vertex::colored(0.0, 0.0, Color::RED);
        let v1 = Vertex::colored(100.0, 0.0, Color::GREEN);
        let v2 = Vertex::colored(50.0, 100.0, Color::BLUE);

        let fragments = rasterize_triangle(&v0, &v1, &v2, 200, 200);
        assert!(!fragments.is_empty());

        // Find a fragment near the center — should be a blend
        let center_frags: Vec<_> = fragments
            .iter()
            .filter(|f| f.x >= 45 && f.x <= 55 && f.y >= 30 && f.y <= 40)
            .collect();
        assert!(!center_frags.is_empty());

        // Center should have contributions from all three vertex colors
        for f in center_frags {
            assert!(f.color.r > 0.05, "expected red > 0.05, got {}", f.color.r);
            assert!(f.color.g > 0.05, "expected green > 0.05, got {}", f.color.g);
            assert!(f.color.b > 0.05, "expected blue > 0.05, got {}", f.color.b);
        }
    }
}
