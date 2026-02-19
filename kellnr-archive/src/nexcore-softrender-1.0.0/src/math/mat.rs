//! Matrix types: Mat3 (2D transforms), Mat4 (3D transforms)
//!
//! Row-major storage. Multiplication is M × v (matrix on left).
//! Column vectors: v' = M * v. Composition: M_final = M_last * ... * M_first.

use super::vec::{Vec2, Vec3, Vec4};

// ============================================================================
// Mat3 — 2D homogeneous transforms
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    /// Row-major: m[row][col]
    pub m: [[f64; 3]; 3],
}

impl Mat3 {
    pub const IDENTITY: Self = Self {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    pub const ZERO: Self = Self { m: [[0.0; 3]; 3] };

    pub const fn new(m: [[f64; 3]; 3]) -> Self {
        Self { m }
    }

    pub fn from_cols(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Self {
            m: [[c0.x, c1.x, c2.x], [c0.y, c1.y, c2.y], [c0.z, c1.z, c2.z]],
        }
    }

    /// Matrix × vector (homogeneous 2D: Vec2 → Vec2 via w=1)
    pub fn transform_point(&self, v: Vec2) -> Vec2 {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2];
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2];
        let w = self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2];
        if w.abs() < f64::EPSILON {
            Vec2::new(x, y)
        } else {
            Vec2::new(x / w, y / w)
        }
    }

    /// Transform direction (ignores translation)
    pub fn transform_dir(&self, v: Vec2) -> Vec2 {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y;
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y;
        Vec2::new(x, y)
    }

    /// Matrix × matrix
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut out = Self::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    out.m[i][j] += self.m[i][k] * rhs.m[k][j];
                }
            }
        }
        out
    }

    pub fn transpose(&self) -> Self {
        Self::new([
            [self.m[0][0], self.m[1][0], self.m[2][0]],
            [self.m[0][1], self.m[1][1], self.m[2][1]],
            [self.m[0][2], self.m[1][2], self.m[2][2]],
        ])
    }

    pub fn determinant(&self) -> f64 {
        self.m[0][0] * (self.m[1][1] * self.m[2][2] - self.m[1][2] * self.m[2][1])
            - self.m[0][1] * (self.m[1][0] * self.m[2][2] - self.m[1][2] * self.m[2][0])
            + self.m[0][2] * (self.m[1][0] * self.m[2][1] - self.m[1][1] * self.m[2][0])
    }

    /// Inverse via cofactor matrix. Returns None if singular.
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f64::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;

        Some(Self::new([
            [
                (self.m[1][1] * self.m[2][2] - self.m[1][2] * self.m[2][1]) * inv_det,
                (self.m[0][2] * self.m[2][1] - self.m[0][1] * self.m[2][2]) * inv_det,
                (self.m[0][1] * self.m[1][2] - self.m[0][2] * self.m[1][1]) * inv_det,
            ],
            [
                (self.m[1][2] * self.m[2][0] - self.m[1][0] * self.m[2][2]) * inv_det,
                (self.m[0][0] * self.m[2][2] - self.m[0][2] * self.m[2][0]) * inv_det,
                (self.m[0][2] * self.m[1][0] - self.m[0][0] * self.m[1][2]) * inv_det,
            ],
            [
                (self.m[1][0] * self.m[2][1] - self.m[1][1] * self.m[2][0]) * inv_det,
                (self.m[0][1] * self.m[2][0] - self.m[0][0] * self.m[2][1]) * inv_det,
                (self.m[0][0] * self.m[1][1] - self.m[0][1] * self.m[1][0]) * inv_det,
            ],
        ]))
    }
}

// ============================================================================
// Mat4 — 3D homogeneous transforms
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Row-major: m[row][col]
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

    pub const ZERO: Self = Self { m: [[0.0; 4]; 4] };

    pub const fn new(m: [[f64; 4]; 4]) -> Self {
        Self { m }
    }

    /// Matrix × Vec4
    pub fn transform(&self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z + self.m[0][3] * v.w,
            self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z + self.m[1][3] * v.w,
            self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z + self.m[2][3] * v.w,
            self.m[3][0] * v.x + self.m[3][1] * v.y + self.m[3][2] * v.z + self.m[3][3] * v.w,
        )
    }

    /// Transform a 3D point (w=1, perspective divide on output)
    pub fn transform_point(&self, v: Vec3) -> Vec3 {
        self.transform(Vec4::point(v.x, v.y, v.z)).to_vec3()
    }

    /// Transform a direction (w=0, no translation)
    pub fn transform_dir(&self, v: Vec3) -> Vec3 {
        self.transform(Vec4::direction(v.x, v.y, v.z)).xyz()
    }

    /// Matrix × matrix
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut out = Self::ZERO;
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    out.m[i][j] += self.m[i][k] * rhs.m[k][j];
                }
            }
        }
        out
    }

    pub fn transpose(&self) -> Self {
        let mut out = Self::ZERO;
        for i in 0..4 {
            for j in 0..4 {
                out.m[i][j] = self.m[j][i];
            }
        }
        out
    }

    /// Extract the upper-left 3×3 submatrix
    pub fn to_mat3(&self) -> Mat3 {
        Mat3::new([
            [self.m[0][0], self.m[0][1], self.m[0][2]],
            [self.m[1][0], self.m[1][1], self.m[1][2]],
            [self.m[2][0], self.m[2][1], self.m[2][2]],
        ])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat3_identity_transform() {
        let p = Vec2::new(5.0, 7.0);
        let result = Mat3::IDENTITY.transform_point(p);
        assert!((result.x - 5.0).abs() < f64::EPSILON);
        assert!((result.y - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mat3_multiply_identity() {
        let m = Mat3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let result = m.mul(&Mat3::IDENTITY);
        assert_eq!(result, m);
    }

    #[test]
    fn mat3_determinant() {
        let m = Mat3::new([[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [5.0, 6.0, 0.0]]);
        let det = m.determinant();
        assert!((det - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mat3_inverse_roundtrip() {
        let m = Mat3::new([[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [5.0, 6.0, 0.0]]);
        if let Some(inv) = m.inverse() {
            let product = m.mul(&inv);
            for i in 0..3 {
                for j in 0..3 {
                    let expected = if i == j { 1.0 } else { 0.0 };
                    assert!(
                        (product.m[i][j] - expected).abs() < 1e-10,
                        "m[{i}][{j}] = {} (expected {expected})",
                        product.m[i][j]
                    );
                }
            }
        }
    }

    #[test]
    fn mat3_singular_no_inverse() {
        let m = Mat3::new([[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [1.0, 1.0, 1.0]]);
        assert!(m.inverse().is_none());
    }

    #[test]
    fn mat4_identity_transform() {
        let p = Vec3::new(1.0, 2.0, 3.0);
        let result = Mat4::IDENTITY.transform_point(p);
        assert!((result.x - 1.0).abs() < f64::EPSILON);
        assert!((result.y - 2.0).abs() < f64::EPSILON);
        assert!((result.z - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mat4_multiply_identity() {
        let result = Mat4::IDENTITY.mul(&Mat4::IDENTITY);
        assert_eq!(result, Mat4::IDENTITY);
    }

    #[test]
    fn mat4_transpose() {
        let m = Mat4::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        let t = m.transpose();
        assert!((t.m[0][1] - 5.0).abs() < f64::EPSILON);
        assert!((t.m[1][0] - 2.0).abs() < f64::EPSILON);
    }
}
