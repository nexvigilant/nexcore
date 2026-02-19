//! Transform factory functions
//!
//! Every visual operation is a matrix. Compose them via multiplication.
//! M_final = M_last * ... * M_first (applied right to left).

use super::mat::{Mat3, Mat4};

// ============================================================================
// 2D Transforms (Mat3)
// ============================================================================

/// 2D translation
pub fn translate_2d(tx: f64, ty: f64) -> Mat3 {
    Mat3::new([[1.0, 0.0, tx], [0.0, 1.0, ty], [0.0, 0.0, 1.0]])
}

/// 2D rotation (counter-clockwise, radians)
pub fn rotate_2d(angle: f64) -> Mat3 {
    let c = angle.cos();
    let s = angle.sin();
    Mat3::new([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
}

/// 2D non-uniform scale
pub fn scale_2d(sx: f64, sy: f64) -> Mat3 {
    Mat3::new([[sx, 0.0, 0.0], [0.0, sy, 0.0], [0.0, 0.0, 1.0]])
}

/// 2D uniform scale
pub fn scale_2d_uniform(s: f64) -> Mat3 {
    scale_2d(s, s)
}

/// 2D shear
pub fn shear_2d(shx: f64, shy: f64) -> Mat3 {
    Mat3::new([[1.0, shx, 0.0], [shy, 1.0, 0.0], [0.0, 0.0, 1.0]])
}

/// Compose two 2D transforms: result = a * b (b applied first)
pub fn compose_2d(a: &Mat3, b: &Mat3) -> Mat3 {
    a.mul(b)
}

// ============================================================================
// 3D Transforms (Mat4)
// ============================================================================

/// 3D translation
pub fn translate_3d(tx: f64, ty: f64, tz: f64) -> Mat4 {
    Mat4::new([
        [1.0, 0.0, 0.0, tx],
        [0.0, 1.0, 0.0, ty],
        [0.0, 0.0, 1.0, tz],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// 3D non-uniform scale
pub fn scale_3d(sx: f64, sy: f64, sz: f64) -> Mat4 {
    Mat4::new([
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Rotation around X axis
pub fn rotate_x(angle: f64) -> Mat4 {
    let c = angle.cos();
    let s = angle.sin();
    Mat4::new([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Rotation around Y axis
pub fn rotate_y(angle: f64) -> Mat4 {
    let c = angle.cos();
    let s = angle.sin();
    Mat4::new([
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Rotation around Z axis
pub fn rotate_z(angle: f64) -> Mat4 {
    let c = angle.cos();
    let s = angle.sin();
    Mat4::new([
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Orthographic projection: maps [l,r]×[b,t]×[n,f] → [-1,1]³
pub fn ortho(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) -> Mat4 {
    let rl = right - left;
    let tb = top - bottom;
    let fn_ = far - near;
    Mat4::new([
        [2.0 / rl, 0.0, 0.0, -(right + left) / rl],
        [0.0, 2.0 / tb, 0.0, -(top + bottom) / tb],
        [0.0, 0.0, -2.0 / fn_, -(far + near) / fn_],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Perspective projection (fov_y in radians, aspect = width/height)
pub fn perspective(fov_y: f64, aspect: f64, near: f64, far: f64) -> Mat4 {
    let f = 1.0 / (fov_y / 2.0).tan();
    let nf = near - far;
    Mat4::new([
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / nf, 2.0 * far * near / nf],
        [0.0, 0.0, -1.0, 0.0],
    ])
}

/// Screen-space transform: NDC [-1,1]² → pixel [0,w)×[0,h)
pub fn viewport(width: f64, height: f64) -> Mat3 {
    let hw = width / 2.0;
    let hh = height / 2.0;
    Mat3::new([
        [hw, 0.0, hw],
        [0.0, -hh, hh], // flip Y: screen Y goes down
        [0.0, 0.0, 1.0],
    ])
}

/// Compose two 3D transforms: result = a * b (b applied first)
pub fn compose_3d(a: &Mat4, b: &Mat4) -> Mat4 {
    a.mul(b)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::vec::Vec2;
    use super::*;

    #[test]
    fn translate_2d_moves_point() {
        let m = translate_2d(10.0, 20.0);
        let p = m.transform_point(Vec2::new(1.0, 2.0));
        assert!((p.x - 11.0).abs() < 1e-10);
        assert!((p.y - 22.0).abs() < 1e-10);
    }

    #[test]
    fn rotate_2d_90_degrees() {
        let m = rotate_2d(core::f64::consts::FRAC_PI_2);
        let p = m.transform_point(Vec2::new(1.0, 0.0));
        assert!((p.x - 0.0).abs() < 1e-10);
        assert!((p.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scale_2d_doubles() {
        let m = scale_2d(2.0, 3.0);
        let p = m.transform_point(Vec2::new(5.0, 7.0));
        assert!((p.x - 10.0).abs() < 1e-10);
        assert!((p.y - 21.0).abs() < 1e-10);
    }

    #[test]
    fn compose_translate_then_rotate() {
        // First translate (1,0), then rotate 90°
        let t = translate_2d(1.0, 0.0);
        let r = rotate_2d(core::f64::consts::FRAC_PI_2);
        let m = compose_2d(&r, &t); // r(t(v))
        let p = m.transform_point(Vec2::ZERO);
        // (0,0) → translate → (1,0) → rotate 90° → (0,1)
        assert!((p.x - 0.0).abs() < 1e-10);
        assert!((p.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn viewport_maps_ndc_to_pixels() {
        let v = viewport(800.0, 600.0);
        // NDC (0,0) → center of screen
        let center = v.transform_point(Vec2::ZERO);
        assert!((center.x - 400.0).abs() < 1e-10);
        assert!((center.y - 300.0).abs() < 1e-10);
        // NDC (-1, 1) → top-left (0, 0)
        let tl = v.transform_point(Vec2::new(-1.0, 1.0));
        assert!((tl.x - 0.0).abs() < 1e-10);
        assert!((tl.y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn translate_3d_moves_point() {
        let m = translate_3d(1.0, 2.0, 3.0);
        let p = super::super::vec::Vec3::new(10.0, 20.0, 30.0);
        let result = m.transform_point(p);
        assert!((result.x - 11.0).abs() < 1e-10);
        assert!((result.y - 22.0).abs() < 1e-10);
        assert!((result.z - 33.0).abs() < 1e-10);
    }

    #[test]
    fn rotate_z_90_degrees() {
        let m = rotate_z(core::f64::consts::FRAC_PI_2);
        let p = super::super::vec::Vec3::new(1.0, 0.0, 0.0);
        let result = m.transform_point(p);
        assert!((result.x - 0.0).abs() < 1e-10);
        assert!((result.y - 1.0).abs() < 1e-10);
    }
}
