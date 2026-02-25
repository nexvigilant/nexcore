//! Viewport: maps world coordinates to screen pixels
//!
//! Combines projection + viewport transform into a single pipeline.

use crate::math::{Mat3, Mat4, Vec2, Vec3};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    /// 3D projection (ortho or perspective)
    pub projection: Mat4,
    /// 2D screen transform (NDC → pixels)
    pub screen: Mat3,
}

impl Viewport {
    /// 2D orthographic viewport: world coords map directly to pixels
    /// Origin at top-left, Y increases downward (screen convention)
    pub fn screen_2d(width: u32, height: u32) -> Self {
        let w = width as f64;
        let h = height as f64;
        Self {
            width,
            height,
            projection: Mat4::IDENTITY,
            screen: crate::math::transform::viewport(w, h),
        }
    }

    /// 3D orthographic viewport
    pub fn ortho_3d(width: u32, height: u32, near: f64, far: f64) -> Self {
        let w = width as f64;
        let h = height as f64;
        Self {
            width,
            height,
            projection: crate::math::transform::ortho(
                -w / 2.0,
                w / 2.0,
                -h / 2.0,
                h / 2.0,
                near,
                far,
            ),
            screen: crate::math::transform::viewport(w, h),
        }
    }

    /// 3D perspective viewport
    pub fn perspective_3d(width: u32, height: u32, fov_y: f64, near: f64, far: f64) -> Self {
        let w = width as f64;
        let h = height as f64;
        Self {
            width,
            height,
            projection: crate::math::transform::perspective(fov_y, w / h, near, far),
            screen: crate::math::transform::viewport(w, h),
        }
    }

    /// Project a 3D point to screen pixel coordinates
    pub fn project(&self, world: Vec3) -> Vec2 {
        let clip = self.projection.transform_point(world);
        self.screen.transform_point(Vec2::new(clip.x, clip.y))
    }

    /// Check if a screen-space point is within viewport bounds
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= 0.0 && p.x < self.width as f64 && p.y >= 0.0 && p.y < self.height as f64
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_2d_center() {
        let vp = Viewport::screen_2d(800, 600);
        let p = vp.project(Vec3::ZERO);
        // NDC (0,0) with identity projection → screen center
        assert!((p.x - 400.0).abs() < 1.0);
        assert!((p.y - 300.0).abs() < 1.0);
    }

    #[test]
    fn contains_bounds() {
        let vp = Viewport::screen_2d(100, 100);
        assert!(vp.contains(Vec2::new(50.0, 50.0)));
        assert!(vp.contains(Vec2::new(0.0, 0.0)));
        assert!(!vp.contains(Vec2::new(100.0, 100.0)));
        assert!(!vp.contains(Vec2::new(-1.0, 50.0)));
    }
}
