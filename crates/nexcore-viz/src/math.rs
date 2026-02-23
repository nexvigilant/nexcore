//! # Internal Mathematics Module
//!
//! Consolidated copies of mathematical primitives (vectors, matrices, colors)
//! from across the NexVigilant ecosystem. Copied here to ensure `nexcore-viz`
//! remains a zero-dependency crate while maintaining high-fidelity types.
//!
//! Grounded: μ (Mapping) space transforms, κ (Comparison) bounds, ∂ (Boundary) limits.

use core::ops::{Add, Mul, Sub};
use serde::{Deserialize, Serialize};

// ============================================================================
// Vectors
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn length_sq(self) -> f64 {
        self.x * self.x + self.y * self.y
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
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, r: Self) -> Self {
        Self::new(self.x + r.x, self.y + r.y)
    }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, r: Self) -> Self {
        Self::new(self.x - r.x, self.y - r.y)
    }
}
impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, r: Self) -> f64 {
        self.x * r.x + self.y * r.y + self.z * r.z
    }
    pub fn cross(self, r: Self) -> Self {
        Self::new(
            self.y * r.z - self.z * r.y,
            self.z * r.x - self.x * r.z,
            self.x * r.y - self.y * r.x,
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
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, r: Self) -> Self {
        Self::new(self.x + r.x, self.y + r.y, self.z + r.z)
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, r: Self) -> Self {
        Self::new(self.x - r.x, self.y - r.y, self.z - r.z)
    }
}
impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

// ============================================================================
// Matrices
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat4 {
    pub m: [[f64; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn transform_point(&self, v: Vec3) -> Vec3 {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z + self.m[0][3];
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z + self.m[1][3];
        let z = self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z + self.m[2][3];
        let w = self.m[3][0] * v.x + self.m[3][1] * v.y + self.m[3][2] * v.z + self.m[3][3];
        if w.abs() < 1e-10 {
            Vec3::new(x, y, z)
        } else {
            Vec3::new(x / w, y / w, z / w)
        }
    }
}

// ============================================================================
// Color
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
    pub fn to_array(&self) -> [f32; 4] {
        [self.r as f32, self.g as f32, self.b as f32, self.a as f32]
    }
}

// ============================================================================
// Bounded Value
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounded<T> {
    pub value: T,
    pub lower: Option<T>,
    pub upper: Option<T>,
}

impl<T: PartialOrd + Copy> Bounded<T> {
    pub fn new(value: T, lower: Option<T>, upper: Option<T>) -> Self {
        Self {
            value,
            lower,
            upper,
        }
    }
    pub fn clamp(&self) -> T {
        let mut v = self.value;
        if let Some(l) = self.lower {
            if v < l {
                v = l;
            }
        }
        if let Some(u) = self.upper {
            if v > u {
                v = u;
            }
        }
        v
    }
}
