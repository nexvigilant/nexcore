//! Vector types: Vec2, Vec3, Vec4 (homogeneous coordinates)
//!
//! Pure math. Every field is f64 for precision.
//! Vec4 uses homogeneous coordinates: actual position = (x/w, y/w, z/w).

use core::ops::{Add, Div, Mul, Neg, Sub};

// ============================================================================
// Vec2
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// 2D cross product (returns scalar z-component)
    pub fn cross(self, rhs: Self) -> f64 {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn length_sq(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < f64::EPSILON {
            Self::ZERO
        } else {
            self * (1.0 / len)
        }
    }

    /// Perpendicular vector (90° counter-clockwise)
    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s)
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        Vec2::new(self * v.x, self * v.y)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

// ============================================================================
// Vec3
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn from_vec2(v: Vec2, z: f64) -> Self {
        Self::new(v.x, v.y, z)
    }

    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn length_sq(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < f64::EPSILON {
            Self::ZERO
        } else {
            self * (1.0 / len)
        }
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        Vec3::new(self * v.x, self * v.y, self * v.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

// ============================================================================
// Vec4 — Homogeneous coordinates
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vec4 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    /// Create homogeneous point (w=1)
    pub fn point(x: f64, y: f64, z: f64) -> Self {
        Self::new(x, y, z, 1.0)
    }

    /// Create homogeneous direction (w=0)
    pub fn direction(x: f64, y: f64, z: f64) -> Self {
        Self::new(x, y, z, 0.0)
    }

    pub fn from_vec3(v: Vec3, w: f64) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    /// Perspective divide: (x/w, y/w, z/w)
    pub fn to_vec3(self) -> Vec3 {
        if self.w.abs() < f64::EPSILON {
            Vec3::new(self.x, self.y, self.z)
        } else {
            Vec3::new(self.x / self.w, self.y / self.w, self.z / self.w)
        }
    }

    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

impl Add for Vec4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Sub for Vec4 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl Mul<f64> for Vec4 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s, self.w * s)
    }
}

impl Div<f64> for Vec4 {
    type Output = Self;
    fn div(self, s: f64) -> Self {
        Self::new(self.x / s, self.y / s, self.z / s, self.w / s)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_basic_ops() {
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(1.0, 2.0);
        let sum = a + b;
        assert!((sum.x - 4.0).abs() < f64::EPSILON);
        assert!((sum.y - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec2_length() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn vec2_normalize() {
        let v = Vec2::new(3.0, 4.0).normalize();
        assert!((v.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vec2_cross() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert!((a.cross(b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec2_perp() {
        let v = Vec2::new(1.0, 0.0);
        let p = v.perp();
        assert!((p.x - 0.0).abs() < f64::EPSILON);
        assert!((p.y - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec3_cross_product() {
        let x = Vec3::X;
        let y = Vec3::Y;
        let z = x.cross(y);
        assert!((z.x - 0.0).abs() < f64::EPSILON);
        assert!((z.y - 0.0).abs() < f64::EPSILON);
        assert!((z.z - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec3_dot() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert!((a.dot(b) - 32.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec4_perspective_divide() {
        let v = Vec4::new(4.0, 6.0, 8.0, 2.0);
        let p = v.to_vec3();
        assert!((p.x - 2.0).abs() < f64::EPSILON);
        assert!((p.y - 3.0).abs() < f64::EPSILON);
        assert!((p.z - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec2_lerp() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 10.0);
        let mid = a.lerp(b, 0.5);
        assert!((mid.x - 5.0).abs() < f64::EPSILON);
        assert!((mid.y - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vec3_normalize_zero() {
        let v = Vec3::ZERO.normalize();
        assert!((v.length()).abs() < f64::EPSILON);
    }
}
