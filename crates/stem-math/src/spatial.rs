//! # Spatial Mathematics: Geometric & Topological Primitives
//!
//! Implements cross-domain T2-P and T2-C primitives derived from the
//! primitive extraction of "What is Space?" — 6 concepts absent from
//! algebraic stem-math, grounded to Lex Primitiva.
//!
//! ## Space Decomposition Origin
//!
//! The concept "space" decomposes to an irreducible core of 4 T1 + 1 T2-P:
//! ```text
//! Space := lambda(Location) + partial(Boundary) + exists(Existence) + empty(Void) + Dimension(T2-P)
//! ```
//!
//! This module provides the Rust manifestations of the spatial composites
//! built atop that core.
//!
//! ## Cross-Domain Transfer
//!
//! | Spatial Math | PV Signals | Software | Economics |
//! |-------------|------------|----------|-----------|
//! | Distance | Signal divergence from baseline | Edit distance, similarity | Market spread |
//! | Dimension | Independent risk factors | Feature dimensions | Market dimensions |
//! | Metric | Disproportionality measure | Code similarity metric | Price distance |
//! | Embedding | MedDRA → SMQ mapping | Subtype relationship | Index composition |
//! | Neighborhood | Signal cluster radius | Fuzzy match threshold | Price band |
//! | Orientation | Causality direction | Endianness, graph direction | Bull/bear polarity |

use serde::{Deserialize, Serialize};
use stem_core::Confidence;

// ============================================================================
// Core Spatial Types
// ============================================================================

/// Tier: T2-P (N Quantity + kappa Comparison)
///
/// A non-negative measure of separation between two elements.
/// The atomic unit of spatial reasoning — you cannot reason about
/// space without some concept of "how far apart."
///
/// # Cross-Domain Transfer
/// - PV: Disproportionality score (how far from expected?)
/// - Software: Edit distance, Hamming distance
/// - Economics: Price spread, market gap
/// - Physics: Proper length, geodesic distance
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Distance(f64);

impl Distance {
    /// Create a new distance (clamped to non-negative)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero distance (identity of indiscernibles)
    pub const ZERO: Self = Self(0.0);

    /// Check if two distances are approximately equal (within tolerance)
    #[must_use]
    pub fn approx_eq(&self, other: &Self, tolerance: f64) -> bool {
        (self.0 - other.0).abs() < tolerance
    }

    /// Check triangle inequality: d(a,c) <= d(a,b) + d(b,c)
    ///
    /// Given three distances forming a triangle, verify the metric axiom.
    /// Returns `true` if the triangle inequality holds.
    #[must_use]
    pub fn triangle_valid(ab: Distance, bc: Distance, ac: Distance) -> bool {
        ac.0 <= ab.0 + bc.0 + f64::EPSILON
    }
}

impl Default for Distance {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Distance {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

/// Tier: T2-P (independent, not reducible to single T1)
///
/// The count of independent axes of variation in a space.
/// A space with N dimensions has N independent parameters.
///
/// # Cross-Domain Transfer
/// - PV: Number of independent risk factors (dose, duration, genetics...)
/// - Software: Feature vector dimensionality, array rank
/// - Economics: Number of independent market variables
/// - Physics: Spatial dimensions (3), spacetime (4), string theory (10/11)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Dimension(u32);

impl Dimension {
    /// Create a new dimension count (minimum 0)
    #[must_use]
    pub const fn new(n: u32) -> Self {
        Self(n)
    }

    /// Get the dimension count
    #[must_use]
    pub fn rank(&self) -> u32 {
        self.0
    }

    /// Scalar (0-dimensional)
    pub const SCALAR: Self = Self(0);

    /// Line (1-dimensional)
    pub const LINE: Self = Self(1);

    /// Plane (2-dimensional)
    pub const PLANE: Self = Self(2);

    /// Euclidean space (3-dimensional)
    pub const SPACE_3D: Self = Self(3);

    /// Spacetime (4-dimensional)
    pub const SPACETIME: Self = Self(4);

    /// Check if this is a subspace of another (fewer or equal dimensions)
    #[must_use]
    pub fn is_subspace_of(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    /// Product dimension: dim(A x B) = dim(A) + dim(B)
    #[must_use]
    pub fn product(&self, other: &Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::SCALAR
    }
}

/// Tier: T2-C (lambda Location + partial Boundary + exists Existence)
///
/// A region around a point defined by a radius — the set of all elements
/// within a given distance. Fundamental to continuity and convergence.
///
/// # Cross-Domain Transfer
/// - PV: Signal cluster — all drugs within a therapeutic class
/// - Software: Fuzzy match — strings within edit distance N
/// - Economics: Price band — values within tolerance of target
/// - Physics: Event horizon, interaction radius
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Neighborhood {
    /// The radius defining the neighborhood
    pub radius: Distance,
    /// Whether the boundary is included (closed) or excluded (open)
    pub open: bool,
}

impl Neighborhood {
    /// Create an open neighborhood (boundary excluded)
    #[must_use]
    pub fn open(radius: Distance) -> Self {
        Self { radius, open: true }
    }

    /// Create a closed neighborhood (boundary included)
    #[must_use]
    pub fn closed(radius: Distance) -> Self {
        Self {
            radius,
            open: false,
        }
    }

    /// Check if a distance is within this neighborhood
    #[must_use]
    pub fn contains(&self, distance: Distance) -> bool {
        if self.open {
            distance.0 < self.radius.0
        } else {
            distance.0 <= self.radius.0
        }
    }

    /// Shrink the neighborhood by a factor (0.0 to 1.0)
    #[must_use]
    pub fn shrink(&self, factor: f64) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            radius: Distance::new(self.radius.0 * factor),
            open: self.open,
        }
    }
}

/// Tier: T2-C (kappa Comparison + sigma Sequence + partial Boundary)
///
/// A binary distinction between equivalent arrangements — left/right,
/// clockwise/counterclockwise, positive/negative.
///
/// # Cross-Domain Transfer
/// - PV: Causality direction (drug → ADR vs reverse causation)
/// - Software: Endianness, graph edge direction, sort order
/// - Chemistry: Enantiomers (R/S chirality)
/// - Economics: Bull/bear polarity, long/short position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Orientation {
    /// Positive orientation (right-handed, clockwise, forward)
    Positive,
    /// Negative orientation (left-handed, counterclockwise, reverse)
    Negative,
    /// No inherent orientation (achiral, undirected)
    #[default]
    Unoriented,
}

impl Orientation {
    /// Reverse the orientation
    #[must_use]
    pub fn reverse(&self) -> Self {
        match self {
            Orientation::Positive => Orientation::Negative,
            Orientation::Negative => Orientation::Positive,
            Orientation::Unoriented => Orientation::Unoriented,
        }
    }

    /// Check if oriented (has a definite direction)
    #[must_use]
    pub fn is_oriented(&self) -> bool {
        !matches!(self, Orientation::Unoriented)
    }

    /// Compose two orientations (negative * negative = positive)
    #[must_use]
    pub fn compose(&self, other: &Self) -> Self {
        match (self, other) {
            (Orientation::Unoriented, _) | (_, Orientation::Unoriented) => Orientation::Unoriented,
            (Orientation::Positive, Orientation::Positive) => Orientation::Positive,
            (Orientation::Negative, Orientation::Negative) => Orientation::Positive,
            _ => Orientation::Negative,
        }
    }
}

/// Tier: T2-C (N Quantity + Dimension + partial Boundary)
///
/// The measure of a space's magnitude along its dimensions —
/// length for 1D, area for 2D, volume for 3D. Extension is
/// what distinguishes a space from a point.
///
/// # Cross-Domain Transfer
/// - PV: Reporting window size, population coverage
/// - Software: Buffer capacity, address range
/// - Economics: Market cap, portfolio breadth
/// - Physics: Spatial extent, cross-section area
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Extension {
    /// The magnitude of extent
    pub magnitude: f64,
    /// The dimension of the extension
    pub dimension: Dimension,
}

impl Extension {
    /// Create a new extension
    #[must_use]
    pub fn new(magnitude: f64, dimension: Dimension) -> Self {
        Self {
            magnitude: magnitude.max(0.0),
            dimension,
        }
    }

    /// A point (zero extension)
    pub const POINT: Self = Self {
        magnitude: 0.0,
        dimension: Dimension::SCALAR,
    };

    /// Check if this has zero extent (is a point)
    #[must_use]
    pub fn is_point(&self) -> bool {
        self.magnitude < f64::EPSILON
    }

    /// Scale the extension uniformly
    #[must_use]
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            magnitude: (self.magnitude * factor).max(0.0),
            dimension: self.dimension,
        }
    }
}

impl Default for Extension {
    fn default() -> Self {
        Self::POINT
    }
}

// ============================================================================
// Measured Spatial Types
// ============================================================================

/// A distance measurement with confidence (Codex IX: MEASURE)
///
/// Tier: T2-C (N + kappa + mu + Confidence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredDistance {
    /// The measured distance
    pub value: Distance,
    /// Confidence in the measurement
    pub confidence: Confidence,
}

impl MeasuredDistance {
    /// Create new measured distance
    pub fn new(value: Distance, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

/// A dimension measurement with confidence
///
/// Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredDimension {
    /// The estimated dimension
    pub value: Dimension,
    /// Confidence in the estimate (e.g., fractal dimension estimation)
    pub confidence: Confidence,
}

impl MeasuredDimension {
    /// Create new measured dimension
    pub fn new(value: Dimension, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

// ============================================================================
// Spatial Traits (T2-P / T2-C)
// ============================================================================

/// Tier: T2-C (N Quantity + mu Mapping + kappa Comparison)
///
/// A distance function satisfying metric axioms:
/// 1. d(x,y) >= 0 (non-negativity)
/// 2. d(x,y) = 0 iff x = y (identity of indiscernibles)
/// 3. d(x,y) = d(y,x) (symmetry)
/// 4. d(x,z) <= d(x,y) + d(y,z) (triangle inequality)
///
/// # Cross-Domain Transfer
/// - PV: Disproportionality as metric (PRR as "distance from expected")
/// - Software: Levenshtein distance, cosine distance
/// - Economics: Price distance, market divergence measure
pub trait Metric {
    /// Element type in the metric space
    type Element;

    /// Compute the distance between two elements
    fn distance(&self, a: &Self::Element, b: &Self::Element) -> Distance;

    /// Check if two elements are within a neighborhood
    fn within(&self, a: &Self::Element, b: &Self::Element, neighborhood: &Neighborhood) -> bool {
        neighborhood.contains(self.distance(a, b))
    }

    /// Verify metric symmetry: d(a,b) = d(b,a)
    fn is_symmetric(&self, a: &Self::Element, b: &Self::Element, tolerance: f64) -> bool {
        self.distance(a, b)
            .approx_eq(&self.distance(b, a), tolerance)
    }
}

/// Tier: T2-C (mu Mapping + kappa Comparison + pi Persistence)
///
/// A structure-preserving injection from one space into another.
/// The embedded space retains its intrinsic properties within the
/// larger space.
///
/// # Cross-Domain Transfer
/// - PV: MedDRA PT → SMQ mapping (terms embedded in queries)
/// - Software: Subtype relationship, word2vec embedding
/// - Economics: Stock → index composition
/// - Math: R^2 embedded in R^3
pub trait Embed {
    /// Source space element type
    type Source;
    /// Target space element type
    type Target;

    /// Embed a source element into the target space
    fn embed(&self, source: &Self::Source) -> Self::Target;

    /// Check if a target element is in the image of the embedding
    fn in_image(&self, target: &Self::Target) -> bool;

    /// Embedding dimension (dimension of source within target)
    fn codimension(&self) -> Dimension;
}

/// Tier: T2-C (kappa Comparison + sigma Sequence + partial Boundary)
///
/// Detect the orientation (handedness) of an arrangement.
/// Returns whether a structure has positive, negative, or no orientation.
///
/// # Cross-Domain Transfer
/// - PV: Causality direction assessment
/// - Software: DAG edge direction, sort stability
/// - Chemistry: Chirality detection (R/S configuration)
pub trait Orient {
    /// Element type whose orientation is being assessed
    type Element;

    /// Determine the orientation of an element
    fn orientation(&self, element: &Self::Element) -> Orientation;

    /// Check if two elements have the same orientation
    fn same_orientation(&self, a: &Self::Element, b: &Self::Element) -> bool {
        self.orientation(a) == self.orientation(b)
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors in spatial operations
#[derive(Debug, nexcore_error::Error)]
pub enum SpatialError {
    /// Triangle inequality violated
    #[error("triangle inequality violated: d(a,c)={ac} > d(a,b)={ab} + d(b,c)={bc}")]
    TriangleViolation {
        /// d(a,b)
        ab: f64,
        /// d(b,c)
        bc: f64,
        /// d(a,c)
        ac: f64,
    },

    /// Dimension mismatch
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Expected dimension
        expected: u32,
        /// Actual dimension
        actual: u32,
    },

    /// Embedding failed
    #[error("embedding failed: {0}")]
    EmbeddingFailed(String),

    /// Metric undefined for given elements
    #[error("metric undefined for given elements")]
    MetricUndefined,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Distance tests =====

    #[test]
    fn distance_is_non_negative() {
        let d = Distance::new(-5.0);
        assert!((d.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_zero_is_identity() {
        assert!((Distance::ZERO.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_addition() {
        let a = Distance::new(3.0);
        let b = Distance::new(4.0);
        assert!(((a + b).value() - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_approx_eq() {
        let a = Distance::new(1.0);
        let b = Distance::new(1.001);
        assert!(a.approx_eq(&b, 0.01));
        assert!(!a.approx_eq(&b, 0.0001));
    }

    #[test]
    fn triangle_inequality_valid() {
        let ab = Distance::new(3.0);
        let bc = Distance::new(4.0);
        let ac = Distance::new(5.0);
        assert!(Distance::triangle_valid(ab, bc, ac));
    }

    #[test]
    fn triangle_inequality_violated() {
        let ab = Distance::new(1.0);
        let bc = Distance::new(1.0);
        let ac = Distance::new(3.0);
        assert!(!Distance::triangle_valid(ab, bc, ac));
    }

    // ===== Dimension tests =====

    #[test]
    fn dimension_constants() {
        assert_eq!(Dimension::SCALAR.rank(), 0);
        assert_eq!(Dimension::LINE.rank(), 1);
        assert_eq!(Dimension::PLANE.rank(), 2);
        assert_eq!(Dimension::SPACE_3D.rank(), 3);
        assert_eq!(Dimension::SPACETIME.rank(), 4);
    }

    #[test]
    fn dimension_subspace() {
        assert!(Dimension::PLANE.is_subspace_of(&Dimension::SPACE_3D));
        assert!(!Dimension::SPACE_3D.is_subspace_of(&Dimension::PLANE));
        assert!(Dimension::LINE.is_subspace_of(&Dimension::LINE)); // reflexive
    }

    #[test]
    fn dimension_product() {
        let d = Dimension::PLANE.product(&Dimension::SPACE_3D);
        assert_eq!(d.rank(), 5);
    }

    // ===== Neighborhood tests =====

    #[test]
    fn open_neighborhood_excludes_boundary() {
        let n = Neighborhood::open(Distance::new(1.0));
        assert!(n.contains(Distance::new(0.5)));
        assert!(!n.contains(Distance::new(1.0))); // boundary excluded
        assert!(!n.contains(Distance::new(1.5)));
    }

    #[test]
    fn closed_neighborhood_includes_boundary() {
        let n = Neighborhood::closed(Distance::new(1.0));
        assert!(n.contains(Distance::new(0.5)));
        assert!(n.contains(Distance::new(1.0))); // boundary included
        assert!(!n.contains(Distance::new(1.5)));
    }

    #[test]
    fn neighborhood_shrink() {
        let n = Neighborhood::open(Distance::new(10.0));
        let shrunk = n.shrink(0.5);
        assert!((shrunk.radius.value() - 5.0).abs() < f64::EPSILON);
        assert!(shrunk.open);
    }

    // ===== Orientation tests =====

    #[test]
    fn orientation_reverse() {
        assert_eq!(Orientation::Positive.reverse(), Orientation::Negative);
        assert_eq!(Orientation::Negative.reverse(), Orientation::Positive);
        assert_eq!(Orientation::Unoriented.reverse(), Orientation::Unoriented);
    }

    #[test]
    fn orientation_compose() {
        // neg * neg = pos (like multiplying signs)
        assert_eq!(
            Orientation::Negative.compose(&Orientation::Negative),
            Orientation::Positive
        );
        assert_eq!(
            Orientation::Positive.compose(&Orientation::Negative),
            Orientation::Negative
        );
        assert_eq!(
            Orientation::Positive.compose(&Orientation::Positive),
            Orientation::Positive
        );
    }

    #[test]
    fn orientation_unoriented_absorbs() {
        assert_eq!(
            Orientation::Positive.compose(&Orientation::Unoriented),
            Orientation::Unoriented
        );
    }

    #[test]
    fn orientation_is_oriented() {
        assert!(Orientation::Positive.is_oriented());
        assert!(Orientation::Negative.is_oriented());
        assert!(!Orientation::Unoriented.is_oriented());
    }

    // ===== Extension tests =====

    #[test]
    fn extension_point_is_zero() {
        assert!(Extension::POINT.is_point());
        assert_eq!(Extension::POINT.dimension, Dimension::SCALAR);
    }

    #[test]
    fn extension_scale() {
        let ext = Extension::new(10.0, Dimension::SPACE_3D);
        let scaled = ext.scale(0.5);
        assert!((scaled.magnitude - 5.0).abs() < f64::EPSILON);
        assert_eq!(scaled.dimension, Dimension::SPACE_3D);
    }

    #[test]
    fn extension_negative_clamped() {
        let ext = Extension::new(-5.0, Dimension::LINE);
        assert!((ext.magnitude - 0.0).abs() < f64::EPSILON);
    }

    // ===== Metric trait test (concrete impl) =====

    struct EuclideanMetric;

    impl Metric for EuclideanMetric {
        type Element = f64;

        fn distance(&self, a: &f64, b: &f64) -> Distance {
            Distance::new((a - b).abs())
        }
    }

    #[test]
    fn euclidean_metric_distance() {
        let m = EuclideanMetric;
        let d = m.distance(&3.0, &7.0);
        assert!((d.value() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn euclidean_metric_symmetry() {
        let m = EuclideanMetric;
        assert!(m.is_symmetric(&3.0, &7.0, f64::EPSILON));
    }

    #[test]
    fn euclidean_metric_within_neighborhood() {
        let m = EuclideanMetric;
        let n = Neighborhood::closed(Distance::new(5.0));
        assert!(m.within(&3.0, &7.0, &n)); // d=4 < 5
        assert!(!m.within(&0.0, &10.0, &n)); // d=10 > 5
    }

    // ===== Measured types =====

    #[test]
    fn measured_distance_confidence() {
        let md = MeasuredDistance::new(Distance::new(4.0), Confidence::new(0.9));
        assert!((md.value.value() - 4.0).abs() < f64::EPSILON);
        assert!((md.confidence.value() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn measured_dimension_confidence() {
        let md = MeasuredDimension::new(Dimension::SPACE_3D, Confidence::new(0.85));
        assert_eq!(md.value.rank(), 3);
        assert!((md.confidence.value() - 0.85).abs() < f64::EPSILON);
    }

    // ===== Embed trait test (concrete impl) =====

    struct LineIntoPlane;

    impl Embed for LineIntoPlane {
        type Source = f64;
        type Target = (f64, f64);

        fn embed(&self, source: &f64) -> (f64, f64) {
            (*source, 0.0) // Embed on x-axis
        }

        fn in_image(&self, target: &(f64, f64)) -> bool {
            target.1.abs() < f64::EPSILON // y = 0
        }

        fn codimension(&self) -> Dimension {
            Dimension::new(1) // 2D - 1D = codimension 1
        }
    }

    #[test]
    fn line_embeds_in_plane() {
        let e = LineIntoPlane;
        assert_eq!(e.embed(&5.0), (5.0, 0.0));
        assert!(e.in_image(&(3.0, 0.0)));
        assert!(!e.in_image(&(3.0, 1.0)));
        assert_eq!(e.codimension().rank(), 1);
    }

    // ===== SpatialError tests =====

    #[test]
    fn spatial_error_display() {
        let err = SpatialError::DimensionMismatch {
            expected: 3,
            actual: 2,
        };
        let msg = format!("{err}");
        assert!(msg.contains("dimension mismatch"));
        assert!(msg.contains("3"));
        assert!(msg.contains("2"));
    }

    #[test]
    fn triangle_violation_display() {
        let err = SpatialError::TriangleViolation {
            ab: 1.0,
            bc: 1.0,
            ac: 3.0,
        };
        let msg = format!("{err}");
        assert!(msg.contains("triangle inequality"));
    }
}
