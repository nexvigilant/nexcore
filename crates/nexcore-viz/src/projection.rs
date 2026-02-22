//! Higher-dimensional projection engine for nD-to-3D visualization.
//!
//! Provides stereographic, perspective, and orthographic projections from
//! arbitrary dimensions down to 3D, plus 4D rotation matrices and parametric
//! surface generators for tesseracts, hyperspheres, and Klein bottles.
//!
//! Primitive formula: projection = μ(nD → 3D) × N(focal_distance) + σ(rotation_sequence)
//!
//! # Quick Start
//!
//! ```rust
//! use nexcore_viz::projection::{ProjectionMethod, project_points, generate_tesseract};
//!
//! // Perspective-project a 4D point down to 3D.
//! let pts = vec![vec![1.0_f64, 0.0, 0.0, 2.0]];
//! let method = ProjectionMethod::Perspective { focal_distance: 3.0 };
//! if let Ok(projected) = project_points(&pts, &method) {
//!     assert_eq!(projected.len(), 1);
//! }
//!
//! // Generate a tesseract wireframe (16 vertices, 32 edges).
//! let mesh = generate_tesseract();
//! assert_eq!(mesh.vertices.len(), 16);
//! assert_eq!(mesh.edges.len(), 32);
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Error Type ───────────────────────────────────────────────────────────────

/// Errors produced by projection operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectionError {
    /// The input point has fewer than 3 dimensions, making 3D output impossible.
    DimensionTooLow(usize),
    /// One or more axis indices are out of bounds or otherwise invalid.
    InvalidAxes(String),
    /// The input slice or point list was empty.
    EmptyInput,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionTooLow(n) => {
                write!(f, "dimension too low: need at least 3, got {n}")
            }
            Self::InvalidAxes(msg) => write!(f, "invalid axes: {msg}"),
            Self::EmptyInput => write!(f, "empty input: no points to project"),
        }
    }
}

impl std::error::Error for ProjectionError {}

// ─── Core Types ───────────────────────────────────────────────────────────────

/// The algorithm used to reduce nD coordinates to 3D.
///
/// All three variants accept points of any dimension >= 3.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::{ProjectionMethod, project_points};
///
/// let pts = vec![vec![1.0_f64, 2.0, 3.0, 4.0]];
/// let m = ProjectionMethod::Perspective { focal_distance: 5.0 };
/// if let Ok(out) = project_points(&pts, &m) {
///     assert_eq!(out.len(), 1);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectionMethod {
    /// Stereographic projection from the north pole (last-axis pole) onto the
    /// complementary hyperplane.  Applied recursively until 3D is reached.
    Stereographic,
    /// Perspective (pinhole) projection.  The last coordinate is used as the
    /// depth axis; all others are scaled by `focal_distance / (focal_distance + depth)`.
    Perspective {
        /// Distance from the viewer to the projection plane.  Must be positive.
        focal_distance: f64,
    },
    /// Orthographic projection: select exactly three named axes and discard the
    /// rest.  No scaling is applied.
    Orthographic {
        /// Indices of the three axes to retain, in output order [x, y, z].
        axes: [usize; 3],
    },
}

/// The 3D result of projecting a single nD point.
///
/// `depth` carries the original "distance" information for use in depth
/// sorting or colour mapping; its interpretation depends on the projection
/// method used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedPoint {
    /// 3D Cartesian position after projection.
    pub position: [f64; 3],
    /// Depth value for sorting / colouring (method-dependent).
    pub depth: f64,
    /// Number of dimensions the source point had.
    pub original_dim: usize,
}

/// A projected wireframe mesh consisting of vertices and edges.
///
/// Edge indices refer into the `vertices` slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedMesh {
    /// Projected vertices.
    pub vertices: Vec<ProjectedPoint>,
    /// Pairs of vertex indices forming the wireframe edges.
    pub edges: Vec<(usize, usize)>,
}

// ─── 4D Rotation Matrix ───────────────────────────────────────────────────────

/// A 4x4 orthogonal rotation matrix for transforming 4D points.
///
/// All six independent 4D rotation planes (XY, XZ, XW, YZ, YW, ZW) are
/// provided as named constructors.  Matrices compose via [`Rotation4D::compose`].
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::Rotation4D;
/// use std::f64::consts::FRAC_PI_2;
///
/// // A 90 degree rotation in the XY plane maps (1,0,0,0) to (0,1,0,0).
/// let r = Rotation4D::xy(FRAC_PI_2);
/// let p = r.apply(&[1.0, 0.0, 0.0, 0.0]);
/// assert!(p[0].abs() < 1e-10);
/// assert!((p[1] - 1.0).abs() < 1e-10);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation4D {
    /// Row-major 4x4 matrix entries.
    pub matrix: [[f64; 4]; 4],
}

impl Rotation4D {
    /// Returns the 4x4 identity rotation (no rotation).
    #[must_use]
    pub fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Builds a rotation by `angle` radians in the **XY plane** (axes 0 and 1).
    #[must_use]
    pub fn xy(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::identity();
        m.matrix[0][0] = c;
        m.matrix[0][1] = -s;
        m.matrix[1][0] = s;
        m.matrix[1][1] = c;
        m
    }

    /// Builds a rotation by `angle` radians in the **XZ plane** (axes 0 and 2).
    #[must_use]
    pub fn xz(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::identity();
        m.matrix[0][0] = c;
        m.matrix[0][2] = -s;
        m.matrix[2][0] = s;
        m.matrix[2][2] = c;
        m
    }

    /// Builds a rotation by `angle` radians in the **XW plane** (axes 0 and 3).
    #[must_use]
    pub fn xw(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::identity();
        m.matrix[0][0] = c;
        m.matrix[0][3] = -s;
        m.matrix[3][0] = s;
        m.matrix[3][3] = c;
        m
    }

    /// Builds a rotation by `angle` radians in the **YZ plane** (axes 1 and 2).
    #[must_use]
    pub fn yz(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::identity();
        m.matrix[1][1] = c;
        m.matrix[1][2] = -s;
        m.matrix[2][1] = s;
        m.matrix[2][2] = c;
        m
    }

    /// Builds a rotation by `angle` radians in the **YW plane** (axes 1 and 3).
    #[must_use]
    pub fn yw(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::identity();
        m.matrix[1][1] = c;
        m.matrix[1][3] = -s;
        m.matrix[3][1] = s;
        m.matrix[3][3] = c;
        m
    }

    /// Builds a rotation by `angle` radians in the **ZW plane** (axes 2 and 3).
    #[must_use]
    pub fn zw(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::identity();
        m.matrix[2][2] = c;
        m.matrix[2][3] = -s;
        m.matrix[3][2] = s;
        m.matrix[3][3] = c;
        m
    }

    /// Applies this rotation to a 4D point, returning the rotated point.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::projection::Rotation4D;
    ///
    /// let r = Rotation4D::identity();
    /// let p = [1.0_f64, 2.0, 3.0, 4.0];
    /// assert_eq!(r.apply(&p), p);
    /// ```
    #[must_use]
    pub fn apply(&self, point: &[f64; 4]) -> [f64; 4] {
        let mut out = [0.0_f64; 4];
        for (i, out_i) in out.iter_mut().enumerate() {
            for (j, &pj) in point.iter().enumerate() {
                *out_i += self.matrix[i][j] * pj;
            }
        }
        out
    }

    /// Returns the composition `self * other` (applies `other` first, then `self`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::projection::Rotation4D;
    /// use std::f64::consts::FRAC_PI_2;
    ///
    /// // Two 90 degree XY rotations compose to a 180 degree rotation.
    /// let r90  = Rotation4D::xy(FRAC_PI_2);
    /// let r180 = r90.compose(&r90);
    /// let p = r180.apply(&[1.0, 0.0, 0.0, 0.0]);
    /// assert!((p[0] + 1.0).abs() < 1e-10);
    /// assert!(p[1].abs()         < 1e-10);
    /// ```
    #[must_use]
    pub fn compose(&self, other: &Rotation4D) -> Rotation4D {
        let mut m = [[0.0_f64; 4]; 4];
        for (i, m_row) in m.iter_mut().enumerate() {
            for (j, m_ij) in m_row.iter_mut().enumerate() {
                for k in 0..4 {
                    *m_ij += self.matrix[i][k] * other.matrix[k][j];
                }
            }
        }
        Rotation4D { matrix: m }
    }
}

// ─── Parametric Surface Enum ──────────────────────────────────────────────────

/// Built-in parametric 4D shapes that can be generated and projected.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::{ParametricSurface, ProjectionMethod, project_surface};
///
/// if let Ok(mesh) = project_surface(
///     &ParametricSurface::Tesseract,
///     8,
///     &ProjectionMethod::Perspective { focal_distance: 3.0 },
/// ) {
///     assert_eq!(mesh.vertices.len(), 16);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParametricSurface {
    /// 4D hypercube with vertices at (+-1, +-1, +-1, +-1).
    Tesseract,
    /// 3-sphere (4D hypersphere) of given radius.
    Hypersphere {
        /// Sphere radius (must be positive; clamped to 1e-6 if not).
        radius: f64,
    },
    /// 4D immersion of the Klein bottle.
    KleinBottle {
        /// Overall scale factor.
        scale: f64,
    },
    /// Clifford torus embedded in 4D.
    CliffordTorus {
        /// Radius in the XY circle.
        r1: f64,
        /// Radius in the ZW circle.
        r2: f64,
    },
    /// User-supplied high-dimensional point cloud with no built-in generator.
    Custom {
        /// Number of dimensions in the custom data.
        dimensions: usize,
    },
}

// ─── Projection Functions ─────────────────────────────────────────────────────

/// Stereographic projection of an nD point to 3D.
///
/// Projects from the north pole (unit vector along `pole_index`) onto the
/// complementary hyperplane, reducing dimension by 1.  Applied recursively
/// until 3D is reached.
///
/// Points at or very close to the pole are clamped to scale 1e6 rather than
/// producing infinity.
///
/// # Errors
///
/// Returns [`ProjectionError::DimensionTooLow`] if `point.len() < 3`.
/// Returns [`ProjectionError::EmptyInput`] if `point` is empty.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::project_stereographic;
///
/// // A 4D point on the equatorial hyperplane (w=0) projects with scale 1.
/// if let Ok(p) = project_stereographic(&[0.5, 0.5, 0.5, 0.0], 3) {
///     assert_eq!(p.original_dim, 4);
///     assert!((p.position[0] - 0.5).abs() < 1e-9);
/// }
/// ```
pub fn project_stereographic(
    point: &[f64],
    pole_index: usize,
) -> Result<ProjectedPoint, ProjectionError> {
    if point.is_empty() {
        return Err(ProjectionError::EmptyInput);
    }
    let n = point.len();
    if n < 3 {
        return Err(ProjectionError::DimensionTooLow(n));
    }

    let original_dim = n;

    // Iteratively reduce dimension from n down to 3.
    let mut current: Vec<f64> = point.to_vec();

    while current.len() > 3 {
        let dim = current.len();
        // Clamp pole index to the last valid index of the current slice.
        let pole_idx = pole_index.min(dim - 1);
        let pole_val = current.get(pole_idx).copied().unwrap_or(0.0);

        // Standard stereographic from north pole at coordinate=1:
        //   x'_i = x_i / (1 - x_pole)  for i != pole_idx
        let denom = 1.0 - pole_val;
        let scale = if denom.abs() < 1e-12 {
            1e6_f64 // point is at the pole — clamp to large finite value
        } else {
            1.0 / denom
        };

        let next: Vec<f64> = current
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| if i == pole_idx { None } else { Some(v * scale) })
            .collect();

        current = next;
    }

    // current.len() == 3 here.
    let x = current.first().copied().unwrap_or(0.0);
    let y = current.get(1).copied().unwrap_or(0.0);
    let z = current.get(2).copied().unwrap_or(0.0);

    // Depth: mean of all coordinates beyond index 2 in the original point.
    let depth = if original_dim > 3 {
        let extra_sum: f64 = point.iter().skip(3).copied().sum();
        extra_sum / (original_dim - 3) as f64
    } else {
        0.0
    };

    Ok(ProjectedPoint {
        position: [x, y, z],
        depth,
        original_dim,
    })
}

/// Perspective (pinhole) projection of an nD point to 3D.
///
/// Uses the **last** coordinate as the depth axis.  Coordinates at indices
/// 0, 1, 2 are scaled by `focal / (focal + depth)`.
///
/// # Errors
///
/// Returns [`ProjectionError::DimensionTooLow`] if `point.len() < 3`.
/// Returns [`ProjectionError::EmptyInput`] if `point` is empty.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::project_perspective;
///
/// // A point at the origin projects to the origin.
/// if let Ok(p) = project_perspective(&[0.0, 0.0, 0.0, 0.0], 3.0) {
///     assert!(p.position[0].abs() < 1e-10);
/// }
/// ```
pub fn project_perspective(
    point: &[f64],
    focal: f64,
) -> Result<ProjectedPoint, ProjectionError> {
    if point.is_empty() {
        return Err(ProjectionError::EmptyInput);
    }
    let n = point.len();
    if n < 3 {
        return Err(ProjectionError::DimensionTooLow(n));
    }

    let depth = if n > 3 {
        point.get(n - 1).copied().unwrap_or(0.0)
    } else {
        0.0
    };

    let denom = focal + depth;
    let scale = if denom.abs() < 1e-12 { 1.0 } else { focal / denom };

    let x = point.first().copied().unwrap_or(0.0) * scale;
    let y = point.get(1).copied().unwrap_or(0.0) * scale;
    let z = point.get(2).copied().unwrap_or(0.0) * scale;

    Ok(ProjectedPoint {
        position: [x, y, z],
        depth,
        original_dim: n,
    })
}

/// Orthographic projection: selects three named axis coordinates from an nD point.
///
/// No perspective scaling is applied; depth is always `0.0`.
///
/// # Errors
///
/// Returns [`ProjectionError::DimensionTooLow`] if `point.len() < 3`.
/// Returns [`ProjectionError::EmptyInput`] if `point` is empty.
/// Returns [`ProjectionError::InvalidAxes`] if any axis index is out of bounds.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::project_orthographic;
///
/// // From a 5D point extract axes 0, 2, 4.
/// if let Ok(p) = project_orthographic(&[10.0, 20.0, 30.0, 40.0, 50.0], &[0, 2, 4]) {
///     assert!((p.position[0] - 10.0).abs() < 1e-10);
///     assert!((p.position[1] - 30.0).abs() < 1e-10);
///     assert!((p.position[2] - 50.0).abs() < 1e-10);
/// }
/// ```
pub fn project_orthographic(
    point: &[f64],
    axes: &[usize; 3],
) -> Result<ProjectedPoint, ProjectionError> {
    if point.is_empty() {
        return Err(ProjectionError::EmptyInput);
    }
    let n = point.len();
    if n < 3 {
        return Err(ProjectionError::DimensionTooLow(n));
    }

    for &ax in axes.iter() {
        if ax >= n {
            return Err(ProjectionError::InvalidAxes(format!(
                "axis {ax} is out of bounds for a {n}-dimensional point"
            )));
        }
    }

    let x = point.get(axes[0]).copied().unwrap_or(0.0);
    let y = point.get(axes[1]).copied().unwrap_or(0.0);
    let z = point.get(axes[2]).copied().unwrap_or(0.0);

    Ok(ProjectedPoint {
        position: [x, y, z],
        depth: 0.0,
        original_dim: n,
    })
}

/// Projects a batch of nD points using the chosen [`ProjectionMethod`].
///
/// # Errors
///
/// Returns [`ProjectionError::EmptyInput`] if `points` is empty.
/// Propagates errors from the individual projection functions.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::{ProjectionMethod, project_points};
///
/// let pts = vec![
///     vec![1.0_f64, 0.0, 0.0, 1.0],
///     vec![0.0_f64, 1.0, 0.0, 2.0],
/// ];
/// if let Ok(out) = project_points(&pts, &ProjectionMethod::Perspective { focal_distance: 4.0 }) {
///     assert_eq!(out.len(), 2);
/// }
/// ```
pub fn project_points(
    points: &[Vec<f64>],
    method: &ProjectionMethod,
) -> Result<Vec<ProjectedPoint>, ProjectionError> {
    if points.is_empty() {
        return Err(ProjectionError::EmptyInput);
    }

    points
        .iter()
        .map(|p| match method {
            ProjectionMethod::Stereographic => {
                project_stereographic(p, p.len().saturating_sub(1))
            }
            ProjectionMethod::Perspective { focal_distance } => {
                project_perspective(p, *focal_distance)
            }
            ProjectionMethod::Orthographic { axes } => project_orthographic(p, axes),
        })
        .collect()
}

// ─── Geometry Generators ──────────────────────────────────────────────────────

/// Generates a 4D hypercube (tesseract) wireframe and projects it to 3D.
///
/// The 16 vertices are all (+-1, +-1, +-1, +-1) combinations.  Two vertices
/// share an edge when they differ in exactly one coordinate — giving 32 edges.
/// Projection uses perspective with `focal_distance = 3.0`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::generate_tesseract;
///
/// let mesh = generate_tesseract();
/// assert_eq!(mesh.vertices.len(), 16);
/// assert_eq!(mesh.edges.len(), 32);
/// ```
#[must_use]
pub fn generate_tesseract() -> ProjectedMesh {
    // All 16 vertices: every combination of (+-1)^4.
    let raw: Vec<[f64; 4]> = (0_u8..16)
        .map(|bits| {
            [
                if bits & 1 != 0 { 1.0 } else { -1.0 },
                if bits & 2 != 0 { 1.0 } else { -1.0 },
                if bits & 4 != 0 { 1.0 } else { -1.0 },
                if bits & 8 != 0 { 1.0 } else { -1.0 },
            ]
        })
        .collect();

    // Edges: pairs that differ in exactly one bit.
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for i in 0..16_usize {
        for j in (i + 1)..16_usize {
            if (i ^ j).count_ones() == 1 {
                edges.push((i, j));
            }
        }
    }

    let focal = 3.0_f64;
    let vertices: Vec<ProjectedPoint> = raw
        .iter()
        .map(|p| {
            let depth = p[3];
            let denom = focal + depth;
            let scale = if denom.abs() < 1e-12 { 1.0 } else { focal / denom };
            ProjectedPoint {
                position: [p[0] * scale, p[1] * scale, p[2] * scale],
                depth,
                original_dim: 4,
            }
        })
        .collect();

    ProjectedMesh { vertices, edges }
}

/// Generates a 3-sphere (4D hypersphere) of the given `radius` and projects to 3D.
///
/// Points are sampled using three angular parameters:
/// `theta1` in [0, pi], `theta2` in [0, pi], `phi` in [0, 2*pi).
///
/// `segments` controls angular resolution (minimum 2).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::generate_hypersphere;
///
/// let mesh = generate_hypersphere(1.0, 6);
/// assert!(!mesh.vertices.is_empty());
/// ```
#[must_use]
pub fn generate_hypersphere(radius: f64, segments: usize) -> ProjectedMesh {
    let r = radius.max(1e-6);
    let seg = segments.max(2);
    let focal = 3.0_f64;

    // (seg+1) latitudes along t1, (seg+1) latitudes along t2, (2*seg) longitudes.
    let n1 = seg + 1;
    let n2 = seg + 1;
    let n_phi = 2 * seg;

    let mut raw4: Vec<[f64; 4]> = Vec::with_capacity(n1 * n2 * n_phi);

    for i1 in 0..n1 {
        let t1 = std::f64::consts::PI * (i1 as f64) / ((n1 - 1) as f64);
        for i2 in 0..n2 {
            let t2 = std::f64::consts::PI * (i2 as f64) / ((n2 - 1) as f64);
            for ip in 0..n_phi {
                let phi = 2.0 * std::f64::consts::PI * (ip as f64) / (n_phi as f64);
                let x = r * t1.sin() * t2.sin() * phi.cos();
                let y = r * t1.sin() * t2.sin() * phi.sin();
                let z = r * t1.sin() * t2.cos();
                let w = r * t1.cos();
                raw4.push([x, y, z, w]);
            }
        }
    }

    // Grid connectivity in all three angular directions.
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let stride_t2 = n_phi;
    let stride_t1 = n2 * n_phi;

    for i1 in 0..n1 {
        for i2 in 0..n2 {
            for ip in 0..n_phi {
                let idx = i1 * stride_t1 + i2 * stride_t2 + ip;
                // Along phi (wrapping).
                let ip_next = (ip + 1) % n_phi;
                edges.push((idx, i1 * stride_t1 + i2 * stride_t2 + ip_next));
                // Along t2.
                if i2 + 1 < n2 {
                    edges.push((idx, i1 * stride_t1 + (i2 + 1) * stride_t2 + ip));
                }
                // Along t1.
                if i1 + 1 < n1 {
                    edges.push((idx, (i1 + 1) * stride_t1 + i2 * stride_t2 + ip));
                }
            }
        }
    }

    let vertices: Vec<ProjectedPoint> = raw4
        .iter()
        .map(|p| {
            let depth = p[3];
            let denom = focal + depth;
            let scale = if denom.abs() < 1e-12 { 1.0 } else { focal / denom };
            ProjectedPoint {
                position: [p[0] * scale, p[1] * scale, p[2] * scale],
                depth,
                original_dim: 4,
            }
        })
        .collect();

    ProjectedMesh { vertices, edges }
}

/// Generates a 4D immersion of the Klein bottle and projects it to 3D.
///
/// The parametric equations are:
///
/// ```text
/// x = (a + b*cos(v))*cos(u)
/// y = (a + b*cos(v))*sin(u)
/// z = b*sin(v)*cos(u/2)
/// w = b*sin(v)*sin(u/2)
/// ```
///
/// where `a = scale`, `b = scale / 3`, `u` in `[0, 2*pi)`, `v` in `[0, 2*pi)`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::generate_klein_bottle;
///
/// let mesh = generate_klein_bottle(1.0, 8, 8);
/// assert!(!mesh.vertices.is_empty());
/// assert!(!mesh.edges.is_empty());
/// ```
#[must_use]
pub fn generate_klein_bottle(scale: f64, u_segments: usize, v_segments: usize) -> ProjectedMesh {
    let a = scale.max(1e-6);
    let b = a / 3.0;
    let nu = u_segments.max(2);
    let nv = v_segments.max(2);
    let focal = 3.0_f64;
    let tau = 2.0 * std::f64::consts::PI;

    let mut raw4: Vec<[f64; 4]> = Vec::with_capacity(nu * nv);

    for iu in 0..nu {
        let u = tau * (iu as f64) / (nu as f64);
        for iv in 0..nv {
            let v = tau * (iv as f64) / (nv as f64);
            let x = (a + b * v.cos()) * u.cos();
            let y = (a + b * v.cos()) * u.sin();
            let z = b * v.sin() * (u / 2.0).cos();
            let w = b * v.sin() * (u / 2.0).sin();
            raw4.push([x, y, z, w]);
        }
    }

    // Grid edges (wrapping in both u and v).
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for iu in 0..nu {
        for iv in 0..nv {
            let idx = iu * nv + iv;
            let iu_next = (iu + 1) % nu;
            let iv_next = (iv + 1) % nv;
            edges.push((idx, iu * nv + iv_next));
            edges.push((idx, iu_next * nv + iv));
        }
    }

    let vertices: Vec<ProjectedPoint> = raw4
        .iter()
        .map(|p| {
            let depth = p[3];
            let denom = focal + depth;
            let scale_f = if denom.abs() < 1e-12 { 1.0 } else { focal / denom };
            ProjectedPoint {
                position: [p[0] * scale_f, p[1] * scale_f, p[2] * scale_f],
                depth,
                original_dim: 4,
            }
        })
        .collect();

    ProjectedMesh { vertices, edges }
}

/// Generates a Clifford torus embedded in 4D and projects it to 3D.
///
/// Parametric equations:
///
/// ```text
/// x = r1*cos(u),  y = r1*sin(u),  z = r2*cos(v),  w = r2*sin(v)
/// ```
///
/// where `u, v` are in `[0, 2*pi)`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::generate_clifford_torus;
///
/// let mesh = generate_clifford_torus(1.0, 0.5, 16);
/// assert_eq!(mesh.vertices.len(), 16 * 16);
/// ```
#[must_use]
pub fn generate_clifford_torus(r1: f64, r2: f64, segments: usize) -> ProjectedMesh {
    let seg = segments.max(2);
    let focal = 3.0_f64;
    let tau = 2.0 * std::f64::consts::PI;

    let mut raw4: Vec<[f64; 4]> = Vec::with_capacity(seg * seg);

    for iu in 0..seg {
        let u = tau * (iu as f64) / (seg as f64);
        for iv in 0..seg {
            let v = tau * (iv as f64) / (seg as f64);
            raw4.push([r1 * u.cos(), r1 * u.sin(), r2 * v.cos(), r2 * v.sin()]);
        }
    }

    let mut edges: Vec<(usize, usize)> = Vec::new();
    for iu in 0..seg {
        for iv in 0..seg {
            let idx = iu * seg + iv;
            let iu_next = (iu + 1) % seg;
            let iv_next = (iv + 1) % seg;
            edges.push((idx, iu * seg + iv_next));
            edges.push((idx, iu_next * seg + iv));
        }
    }

    let vertices: Vec<ProjectedPoint> = raw4
        .iter()
        .map(|p| {
            let depth = p[3];
            let denom = focal + depth;
            let scale = if denom.abs() < 1e-12 { 1.0 } else { focal / denom };
            ProjectedPoint {
                position: [p[0] * scale, p[1] * scale, p[2] * scale],
                depth,
                original_dim: 4,
            }
        })
        .collect();

    ProjectedMesh { vertices, edges }
}

/// Applies a [`Rotation4D`] matrix to every point in the slice, in-place.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::{Rotation4D, rotate_4d};
///
/// let mut pts = [[1.0_f64, 0.0, 0.0, 0.0]];
/// rotate_4d(&mut pts, &Rotation4D::identity());
/// assert!((pts[0][0] - 1.0).abs() < 1e-10);
/// ```
pub fn rotate_4d(points: &mut [[f64; 4]], rotation: &Rotation4D) {
    for p in points.iter_mut() {
        *p = rotation.apply(p);
    }
}

/// Generates and projects a [`ParametricSurface`] using the given method.
///
/// The `segments` parameter controls angular or grid resolution for smooth
/// surfaces; it is ignored for `Tesseract` and `Custom`.
///
/// # Errors
///
/// Returns [`ProjectionError::InvalidAxes`] if an `Orthographic` axis is out
/// of range for the generated 4D data.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::projection::{ParametricSurface, ProjectionMethod, project_surface};
///
/// if let Ok(mesh) = project_surface(
///     &ParametricSurface::CliffordTorus { r1: 1.0, r2: 1.0 },
///     8,
///     &ProjectionMethod::Perspective { focal_distance: 3.0 },
/// ) {
///     assert!(!mesh.vertices.is_empty());
/// }
/// ```
pub fn project_surface(
    surface: &ParametricSurface,
    segments: usize,
    method: &ProjectionMethod,
) -> Result<ProjectedMesh, ProjectionError> {
    match surface {
        ParametricSurface::Tesseract => {
            let base = generate_tesseract();
            reproject_mesh(base, method)
        }
        ParametricSurface::Hypersphere { radius } => {
            let base = generate_hypersphere(*radius, segments);
            reproject_mesh(base, method)
        }
        ParametricSurface::KleinBottle { scale } => {
            let base = generate_klein_bottle(*scale, segments, segments);
            reproject_mesh(base, method)
        }
        ParametricSurface::CliffordTorus { r1, r2 } => {
            let base = generate_clifford_torus(*r1, *r2, segments);
            reproject_mesh(base, method)
        }
        ParametricSurface::Custom { .. } => {
            // No built-in point generator; return an empty mesh.
            Ok(ProjectedMesh {
                vertices: vec![],
                edges: vec![],
            })
        }
    }
}

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// Re-projects a mesh that was generated with `focal = 3.0` perspective, using
/// a different projection method if requested.
///
/// For `Perspective` we recover the original XYZ by inverting the focal=3
/// scale, then re-apply the user's focal distance.  For `Stereographic` and
/// `Orthographic` we reconstruct a 4D vector `[x, y, z, depth]` and call the
/// standard projection path.
fn reproject_mesh(
    base: ProjectedMesh,
    method: &ProjectionMethod,
) -> Result<ProjectedMesh, ProjectionError> {
    /// Recover the pre-perspective-projection XYZ (original focal = 3.0).
    fn recover_xyz(v: &ProjectedPoint) -> [f64; 3] {
        let old_denom = 3.0 + v.depth;
        let old_scale = if old_denom.abs() < 1e-12 {
            1.0
        } else {
            3.0 / old_denom
        };
        if old_scale.abs() < 1e-12 {
            v.position
        } else {
            [
                v.position[0] / old_scale,
                v.position[1] / old_scale,
                v.position[2] / old_scale,
            ]
        }
    }

    match method {
        ProjectionMethod::Perspective { focal_distance } => {
            let vertices: Vec<ProjectedPoint> = base
                .vertices
                .iter()
                .map(|v| {
                    let [ox, oy, oz] = recover_xyz(v);
                    let depth = v.depth;
                    let denom = focal_distance + depth;
                    let scale = if denom.abs() < 1e-12 { 1.0 } else { focal_distance / denom };
                    ProjectedPoint {
                        position: [ox * scale, oy * scale, oz * scale],
                        depth,
                        original_dim: v.original_dim,
                    }
                })
                .collect();
            Ok(ProjectedMesh { vertices, edges: base.edges })
        }
        ProjectionMethod::Stereographic => {
            // Reconstruct 4D as [ox, oy, oz, depth] then call project_stereographic.
            let result: Result<Vec<ProjectedPoint>, ProjectionError> = base
                .vertices
                .iter()
                .map(|v| {
                    let [ox, oy, oz] = recover_xyz(v);
                    let point4 = [ox, oy, oz, v.depth];
                    project_stereographic(&point4, 3)
                })
                .collect();
            Ok(ProjectedMesh { vertices: result?, edges: base.edges })
        }
        ProjectionMethod::Orthographic { axes } => {
            // Validate all axes are within 4D bounds.
            for &ax in axes.iter() {
                if ax >= 4 {
                    return Err(ProjectionError::InvalidAxes(format!(
                        "axis {ax} out of bounds for 4D points"
                    )));
                }
            }
            let result: Result<Vec<ProjectedPoint>, ProjectionError> = base
                .vertices
                .iter()
                .map(|v| {
                    let [ox, oy, oz] = recover_xyz(v);
                    let point4 = [ox, oy, oz, v.depth];
                    project_orthographic(&point4, axes)
                })
                .collect();
            Ok(ProjectedMesh { vertices: result?, edges: base.edges })
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    const EPS: f64 = 1e-6;

    // ── 1. stereographic_3d_point ─────────────────────────────────────────────

    #[test]
    fn stereographic_3d_point() {
        // A 4D point on the equatorial hyperplane (w=0) has denom = 1-0 = 1,
        // so the stereographic scale is 1 and the projected position equals
        // the original (x, y, z).
        let result = project_stereographic(&[0.5, 0.3, 0.2, 0.0], 3);
        match result {
            Ok(p) => {
                assert_eq!(p.original_dim, 4);
                assert!((p.position[0] - 0.5).abs() < EPS, "x mismatch");
                assert!((p.position[1] - 0.3).abs() < EPS, "y mismatch");
                assert!((p.position[2] - 0.2).abs() < EPS, "z mismatch");
            }
            Err(e) => assert!(false, "unexpected error: {e}"),
        }
    }

    // ── 2. perspective_origin ─────────────────────────────────────────────────

    #[test]
    fn perspective_origin() {
        // Zero x/y/z coordinates project to zero regardless of focal distance.
        let result = project_perspective(&[0.0, 0.0, 0.0, 0.0], 3.0);
        match result {
            Ok(p) => {
                assert!((p.position[0]).abs() < EPS);
                assert!((p.position[1]).abs() < EPS);
                assert!((p.position[2]).abs() < EPS);
            }
            Err(e) => assert!(false, "unexpected error: {e}"),
        }
    }

    // ── 3. perspective_depth_scaling ──────────────────────────────────────────

    #[test]
    fn perspective_depth_scaling() {
        // depth=0, focal=3 → scale = 3/3 = 1.0  → x projects to 1.0
        // depth=3, focal=3 → scale = 3/6 = 0.5  → x projects to 0.5
        let near = project_perspective(&[1.0, 0.0, 0.0, 0.0], 3.0);
        let far  = project_perspective(&[1.0, 0.0, 0.0, 3.0], 3.0);
        match (near, far) {
            (Ok(n), Ok(f)) => {
                assert!(
                    n.position[0] > f.position[0],
                    "nearer point should appear larger: {} vs {}",
                    n.position[0],
                    f.position[0]
                );
                assert!((n.position[0] - 1.0).abs() < EPS, "near x");
                assert!((f.position[0] - 0.5).abs() < EPS, "far x");
            }
            (Err(e), _) | (_, Err(e)) => assert!(false, "unexpected error: {e}"),
        }
    }

    // ── 4. orthographic_selects_axes ─────────────────────────────────────────

    #[test]
    fn orthographic_selects_axes() {
        let result = project_orthographic(&[10.0, 20.0, 30.0, 40.0, 50.0], &[0, 2, 4]);
        match result {
            Ok(p) => {
                assert!((p.position[0] - 10.0).abs() < EPS, "axis 0");
                assert!((p.position[1] - 30.0).abs() < EPS, "axis 2");
                assert!((p.position[2] - 50.0).abs() < EPS, "axis 4");
                assert_eq!(p.original_dim, 5);
                assert!((p.depth).abs() < EPS, "orthographic depth is always 0");
            }
            Err(e) => assert!(false, "unexpected error: {e}"),
        }
    }

    // ── 5. orthographic_invalid_axes ─────────────────────────────────────────

    #[test]
    fn orthographic_invalid_axes() {
        let result = project_orthographic(&[1.0, 2.0, 3.0, 4.0, 5.0], &[0, 1, 10]);
        assert!(
            matches!(result, Err(ProjectionError::InvalidAxes(_))),
            "expected InvalidAxes error, got: {result:?}"
        );
    }

    // ── 6. rotation_identity ──────────────────────────────────────────────────

    #[test]
    fn rotation_identity() {
        let r = Rotation4D::identity();
        let p = [1.0_f64, 2.0, 3.0, 4.0];
        let out = r.apply(&p);
        for i in 0..4 {
            assert!(
                (out[i] - p[i]).abs() < EPS,
                "identity should preserve component {i}: got {}",
                out[i]
            );
        }
    }

    // ── 7. rotation_xy_90 ─────────────────────────────────────────────────────

    #[test]
    fn rotation_xy_90() {
        // 90 degree XY rotation: (1,0,0,0) -> (0,1,0,0).
        let r = Rotation4D::xy(FRAC_PI_2);
        let out = r.apply(&[1.0, 0.0, 0.0, 0.0]);
        assert!((out[0]).abs() < EPS,       "x should be ~0, got {}", out[0]);
        assert!((out[1] - 1.0).abs() < EPS, "y should be ~1, got {}", out[1]);
        assert!((out[2]).abs() < EPS,       "z unchanged, got {}", out[2]);
        assert!((out[3]).abs() < EPS,       "w unchanged, got {}", out[3]);
    }

    // ── 8. rotation_compose ───────────────────────────────────────────────────

    #[test]
    fn rotation_compose() {
        // Two 90-degree XY rotations must equal a 180-degree rotation.
        let r90  = Rotation4D::xy(FRAC_PI_2);
        let r180 = r90.compose(&r90);
        let out  = r180.apply(&[1.0, 0.0, 0.0, 0.0]);
        assert!((out[0] + 1.0).abs() < EPS, "x should be -1, got {}", out[0]);
        assert!((out[1]).abs()       < EPS, "y should be  0, got {}", out[1]);

        // Cross-check against a direct PI rotation.
        let r_pi    = Rotation4D::xy(PI);
        let ref_out = r_pi.apply(&[1.0, 0.0, 0.0, 0.0]);
        assert!((out[0] - ref_out[0]).abs() < EPS);
        assert!((out[1] - ref_out[1]).abs() < EPS);
    }

    // ── 9. tesseract_vertices ─────────────────────────────────────────────────

    #[test]
    fn tesseract_vertices() {
        let mesh = generate_tesseract();
        assert_eq!(mesh.vertices.len(), 16, "tesseract must have exactly 16 vertices");
    }

    // ── 10. tesseract_edges ───────────────────────────────────────────────────

    #[test]
    fn tesseract_edges() {
        let mesh = generate_tesseract();
        assert_eq!(mesh.edges.len(), 32, "tesseract must have exactly 32 edges");
    }

    // ── 11. hypersphere_on_sphere ─────────────────────────────────────────────

    #[test]
    fn hypersphere_on_sphere() {
        // Verify the raw 4D parametric formula places every point exactly on
        // the 3-sphere of the given radius.
        let radius = 2.0_f64;
        let seg = 4_usize;
        let n1    = seg + 1;
        let n2    = seg + 1;
        let n_phi = 2 * seg;

        for i1 in 0..n1 {
            let t1 = PI * (i1 as f64) / ((n1 - 1) as f64);
            for i2 in 0..n2 {
                let t2 = PI * (i2 as f64) / ((n2 - 1) as f64);
                for ip in 0..n_phi {
                    let phi = 2.0 * PI * (ip as f64) / (n_phi as f64);
                    let x = radius * t1.sin() * t2.sin() * phi.cos();
                    let y = radius * t1.sin() * t2.sin() * phi.sin();
                    let z = radius * t1.sin() * t2.cos();
                    let w = radius * t1.cos();
                    let norm = (x * x + y * y + z * z + w * w).sqrt();
                    assert!(
                        (norm - radius).abs() < EPS,
                        "point ({x:.4},{y:.4},{z:.4},{w:.4}) |p|={norm:.6} != radius={radius}"
                    );
                }
            }
        }
    }

    // ── 12. empty_input_error ─────────────────────────────────────────────────

    #[test]
    fn empty_input_error() {
        let result =
            project_points(&[], &ProjectionMethod::Perspective { focal_distance: 3.0 });
        assert!(
            matches!(result, Err(ProjectionError::EmptyInput)),
            "empty input should return EmptyInput, got: {result:?}"
        );
    }

    // ── Additional: dimension_too_low for stereographic ───────────────────────

    #[test]
    fn stereographic_dimension_too_low() {
        let result = project_stereographic(&[1.0, 0.0], 1);
        assert!(
            matches!(result, Err(ProjectionError::DimensionTooLow(2))),
            "2D input should return DimensionTooLow(2), got: {result:?}"
        );
    }

    // ── Additional: perspective depth value is last coord ─────────────────────

    #[test]
    fn perspective_depth_is_last_coord() {
        let result = project_perspective(&[1.0, 2.0, 3.0, 7.5], 5.0);
        match result {
            Ok(p) => assert!((p.depth - 7.5).abs() < EPS, "depth should equal last coordinate"),
            Err(e) => assert!(false, "unexpected error: {e}"),
        }
    }

    // ── Additional: Clifford torus vertex count ───────────────────────────────

    #[test]
    fn clifford_torus_vertex_count() {
        let seg = 16_usize;
        let mesh = generate_clifford_torus(1.0, 0.5, seg);
        assert_eq!(mesh.vertices.len(), seg * seg);
    }

    // ── Additional: rotation matrix is orthogonal (R * R^T = I) ──────────────

    #[test]
    fn rotation_matrix_orthogonal() {
        let r = Rotation4D::zw(0.7);
        for i in 0..4 {
            for j in 0..4 {
                let dot: f64 = (0..4).map(|k| r.matrix[i][k] * r.matrix[j][k]).sum();
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (dot - expected).abs() < EPS,
                    "R*R^T[{i}][{j}] = {dot:.8}, expected {expected}"
                );
            }
        }
    }
}
