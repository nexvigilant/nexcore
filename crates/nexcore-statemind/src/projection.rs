//! Stage 5: 3D Feature Projection.
//!
//! Maps composition and spectral features to a 3D coordinate:
//! (Shannon entropy, GC ratio, spectral entropy).
//!
//! Tier: T2-P | Dominant: λ (Location) — spatial embedding.

use crate::composition::Composition;
use crate::spectral::SpectralProfile;
use serde::{Deserialize, Serialize};

/// A point in 3D statemind space.
///
/// Axes:
/// - x: Shannon entropy (information content) [0.0, 2.0]
/// - y: GC ratio (structural stability) [0.0, 1.0]
/// - z: Spectral entropy (harmonic complexity) [0.0, ∞)
///
/// Tier: T2-P | Grounds to: λ (Location) + N (Quantity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point3D {
    /// Shannon entropy — information axis.
    pub x: f64,
    /// GC ratio — stability axis.
    pub y: f64,
    /// Spectral entropy — complexity axis.
    pub z: f64,
}

impl Point3D {
    /// Create a point from explicit coordinates.
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Origin point (0, 0, 0).
    #[must_use]
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Euclidean distance to another point.
    #[must_use]
    pub fn distance(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    /// Project from analysis features into statemind space.
    ///
    /// x = Shannon entropy (information content)
    /// y = GC ratio (structural stability)
    /// z = spectral entropy (harmonic complexity)
    #[must_use]
    pub fn from_features(composition: &Composition, spectral: &SpectralProfile) -> Self {
        Self {
            x: composition.shannon_entropy,
            y: composition.gc_ratio,
            z: spectral.spectral_entropy,
        }
    }

    /// Magnitude (distance from origin).
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Midpoint between two points.
    #[must_use]
    pub fn midpoint(&self, other: &Self) -> Self {
        Self {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
            z: (self.z + other.z) / 2.0,
        }
    }
}

impl std::fmt::Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_symmetric() {
        let a = Point3D::new(1.0, 2.0, 3.0);
        let b = Point3D::new(4.0, 5.0, 6.0);
        assert!((a.distance(&b) - b.distance(&a)).abs() < 1e-10);
    }

    #[test]
    fn self_distance_zero() {
        let p = Point3D::new(1.5, 0.4, 3.2);
        assert!(p.distance(&p) < 1e-10);
    }

    #[test]
    fn origin_magnitude_zero() {
        assert!(Point3D::origin().magnitude() < 1e-10);
    }

    #[test]
    fn known_distance() {
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(3.0, 4.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn midpoint_equidistant() {
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(4.0, 6.0, 8.0);
        let m = a.midpoint(&b);
        assert!((a.distance(&m) - b.distance(&m)).abs() < 1e-10);
    }
}
