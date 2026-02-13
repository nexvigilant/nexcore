//! # Spatial Bridge: nexcore-compilation-space → stem-math
//!
//! Formalizes the 7-dimensional compilation space as a spatial structure:
//! - `COMPILATION_DIMENSION` = 7 (number of axes)
//! - `AbstractionMetric` measures level distance (|a - b| levels)
//! - `TemporalMetric` measures revision distance
//! - `ReflectionMetric` measures meta-depth distance
//! - `CompilationAxisMetric` counts differing axes (Hamming-like)
//! - Transform direction maps to `Orientation` (Up=Positive, Down=Negative)
//!
//! ## Primitive Foundation
//!
//! The compilation space is inherently spatial:
//! - 7 orthogonal axes define the full space (Dimension)
//! - Each axis has a distance function (Metric)
//! - Transforms have direction along axes (Orientation)
//! - Compilation = movement through a 7D metric space
//!
//! ## Architecture Decision
//!
//! Four `Metric` implementations formalize the four axes that have
//! natural numeric distance (abstraction, temporal, reflection, axes-count).
//! `Direction` maps to `Orientation` for directional transforms.

use stem_math::spatial::{Dimension, Distance, Metric, Orientation};

use crate::axis::{AbstractionLevel, Direction, ReflectionDepth, TemporalCoord};
use crate::point::CompilationPoint;

// ============================================================================
// Dimension constants
// ============================================================================

/// The compilation space has 7 orthogonal axes.
///
/// Tier: T1 (N Quantity — count of independent dimensions)
pub const COMPILATION_DIMENSION: Dimension = Dimension::new(7);

/// The abstraction axis has 8 discrete levels (Execution..Intent).
///
/// Tier: T1 (N Quantity — count of abstraction levels)
pub const ABSTRACTION_LEVELS: Dimension = Dimension::new(8);

// ============================================================================
// AbstractionMetric: Distance between abstraction levels
// ============================================================================

/// Metric over `AbstractionLevel` values.
///
/// Distance = |level_a - level_b| (integer levels). This is a valid metric
/// on the ordered set {Execution, Binary, IR, AST, Token, Source, Spec, Intent}.
///
/// Use case: Measuring how "far" a compilation step moves in the abstraction
/// hierarchy — compiling Source→Binary is distance 4.
///
/// Tier: T2-P (σ Sequence + κ Comparison)
pub struct AbstractionMetric;

impl Metric for AbstractionMetric {
    type Element = AbstractionLevel;

    fn distance(&self, a: &AbstractionLevel, b: &AbstractionLevel) -> Distance {
        Distance::new(a.distance(b) as f64)
    }
}

// ============================================================================
// TemporalMetric: Distance between temporal coordinates
// ============================================================================

/// Metric over `TemporalCoord` values.
///
/// Distance = |revision_a - revision_b|. Measures how many revisions apart
/// two artifacts are in the version timeline.
///
/// Use case: Measuring version drift — how far apart two code snapshots are.
///
/// Tier: T2-P (ν Frequency + N Quantity)
pub struct TemporalMetric;

impl Metric for TemporalMetric {
    type Element = TemporalCoord;

    fn distance(&self, a: &TemporalCoord, b: &TemporalCoord) -> Distance {
        Distance::new(a.distance(b) as f64)
    }
}

// ============================================================================
// ReflectionMetric: Distance between reflection depths
// ============================================================================

/// Metric over `ReflectionDepth` values.
///
/// Distance = |depth_a - depth_b|. Measures how many meta-levels apart
/// two artifacts are (code vs code-about-code vs code-about-code-about-code).
///
/// Use case: Measuring "strangeness" of a loop — ground→meta is distance 1.
///
/// Tier: T2-P (ρ Recursion + N Quantity)
pub struct ReflectionMetric;

impl Metric for ReflectionMetric {
    type Element = ReflectionDepth;

    fn distance(&self, a: &ReflectionDepth, b: &ReflectionDepth) -> Distance {
        Distance::new(a.distance(b) as f64)
    }
}

// ============================================================================
// CompilationAxisMetric: Hamming-like distance between compilation points
// ============================================================================

/// Metric over `CompilationPoint` values.
///
/// Distance = number of differing axes (0..7). This is a Hamming-like metric
/// on the 7-dimensional space: two points are "close" when they differ on
/// few axes and "far" when they differ on many.
///
/// Use case: Measuring transform complexity — a simple rename changes 0 axes,
/// transpilation changes 1 (Language), compilation changes 2+ (Abstraction +
/// Dimensionality + possibly Evaluation).
///
/// Tier: T2-C (σ + μ + ν + ∂ + ρ + Σ — all axis primitives)
pub struct CompilationAxisMetric;

impl Metric for CompilationAxisMetric {
    type Element = CompilationPoint;

    fn distance(&self, a: &CompilationPoint, b: &CompilationPoint) -> Distance {
        Distance::new(a.axis_distance(b) as f64)
    }
}

// ============================================================================
// Direction → Orientation mapping
// ============================================================================

/// Map compilation direction to spatial orientation.
///
/// - `Down` (compile/lower) → `Negative` (reducing abstraction)
/// - `Up` (decompile/raise) → `Positive` (increasing abstraction)
/// - `Lateral` (transpile) → `Unoriented` (same-level movement)
///
/// Tier: T2-P (σ Sequence — directed axis movement)
pub fn direction_orientation(dir: &Direction) -> Orientation {
    match dir {
        Direction::Down => Orientation::Negative,
        Direction::Up => Orientation::Positive,
        Direction::Lateral => Orientation::Unoriented,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::LanguageId;

    // ===== Dimension tests =====

    #[test]
    fn compilation_space_is_7d() {
        assert_eq!(COMPILATION_DIMENSION.rank(), 7);
    }

    #[test]
    fn abstraction_has_8_levels() {
        assert_eq!(ABSTRACTION_LEVELS.rank(), 8);
    }

    // ===== AbstractionMetric axiom tests =====

    #[test]
    fn abstraction_metric_identity() {
        let m = AbstractionMetric;
        let a = AbstractionLevel::Source;
        assert!(m.distance(&a, &a).approx_eq(&Distance::ZERO, 1e-10));
    }

    #[test]
    fn abstraction_metric_symmetry() {
        let m = AbstractionMetric;
        let a = AbstractionLevel::Source;
        let b = AbstractionLevel::Ast;
        assert!(m.is_symmetric(&a, &b, 1e-10));
    }

    #[test]
    fn abstraction_metric_triangle() {
        let m = AbstractionMetric;
        let a = AbstractionLevel::Intent;
        let b = AbstractionLevel::Source;
        let c = AbstractionLevel::Binary;
        let d_ab = m.distance(&a, &b);
        let d_bc = m.distance(&b, &c);
        let d_ac = m.distance(&a, &c);
        assert!(Distance::triangle_valid(d_ab, d_bc, d_ac));
    }

    #[test]
    fn abstraction_source_to_ast_is_2() {
        let m = AbstractionMetric;
        let src = AbstractionLevel::Source;
        let ast = AbstractionLevel::Ast;
        // Source=5, Ast=3, |5-3|=2
        assert!(m.distance(&src, &ast).approx_eq(&Distance::new(2.0), 1e-10));
    }

    // ===== TemporalMetric axiom tests =====

    #[test]
    fn temporal_metric_identity() {
        let m = TemporalMetric;
        let a = TemporalCoord::new(5);
        assert!(m.distance(&a, &a).approx_eq(&Distance::ZERO, 1e-10));
    }

    #[test]
    fn temporal_metric_symmetry() {
        let m = TemporalMetric;
        let a = TemporalCoord::new(3);
        let b = TemporalCoord::new(10);
        assert!(m.is_symmetric(&a, &b, 1e-10));
    }

    #[test]
    fn temporal_metric_triangle() {
        let m = TemporalMetric;
        let a = TemporalCoord::new(1);
        let b = TemporalCoord::new(5);
        let c = TemporalCoord::new(12);
        let d_ab = m.distance(&a, &b);
        let d_bc = m.distance(&b, &c);
        let d_ac = m.distance(&a, &c);
        assert!(Distance::triangle_valid(d_ab, d_bc, d_ac));
    }

    // ===== ReflectionMetric =====

    #[test]
    fn reflection_metric_ground_to_meta() {
        let m = ReflectionMetric;
        let ground = ReflectionDepth::GROUND;
        let meta = ReflectionDepth::META;
        assert!(
            m.distance(&ground, &meta)
                .approx_eq(&Distance::new(1.0), 1e-10)
        );
    }

    #[test]
    fn reflection_metric_identity() {
        let m = ReflectionMetric;
        let a = ReflectionDepth::META;
        assert!(m.distance(&a, &a).approx_eq(&Distance::ZERO, 1e-10));
    }

    // ===== CompilationAxisMetric =====

    #[test]
    fn axis_metric_same_point() {
        let m = CompilationAxisMetric;
        let a = CompilationPoint::source(LanguageId::rust());
        let b = CompilationPoint::source(LanguageId::rust());
        assert!(m.distance(&a, &b).approx_eq(&Distance::ZERO, 1e-10));
    }

    #[test]
    fn axis_metric_language_change() {
        let m = CompilationAxisMetric;
        let a = CompilationPoint::source(LanguageId::rust());
        let b = CompilationPoint::source(LanguageId::javascript());
        // Only language axis differs → distance = 1
        assert!(m.distance(&a, &b).approx_eq(&Distance::new(1.0), 1e-10));
    }

    #[test]
    fn axis_metric_multi_axis_change() {
        let m = CompilationAxisMetric;
        let a = CompilationPoint::source(LanguageId::rust());
        let b = CompilationPoint::ast(LanguageId::javascript());
        // Abstraction + Language + Projection differ → distance >= 3
        assert!(m.distance(&a, &b).value() >= 3.0);
    }

    // ===== Direction → Orientation =====

    #[test]
    fn compile_is_negative() {
        assert_eq!(
            direction_orientation(&Direction::Down),
            Orientation::Negative
        );
    }

    #[test]
    fn decompile_is_positive() {
        assert_eq!(direction_orientation(&Direction::Up), Orientation::Positive);
    }

    #[test]
    fn transpile_is_unoriented() {
        assert_eq!(
            direction_orientation(&Direction::Lateral),
            Orientation::Unoriented
        );
    }
}
