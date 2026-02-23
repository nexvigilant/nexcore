//! # Calabi-Yau Manifold Renderer
//!
//! Parametric surface evaluator and mesh generator for Calabi-Yau manifold
//! visualization, targeting string theory "Phase 5 Eyes" of SOP-DEV-003.
//!
//! ## Mathematical Background
//!
//! A Calabi-Yau manifold is a complex Kähler manifold with a vanishing first
//! Chern class (trivial canonical bundle). The Fermat quintic hypersurface in CP⁴,
//! defined by `z₀⁵ + z₁⁵ + z₂⁵ + z₃⁵ + z₄⁵ = 0`, is the prototypical compact
//! Calabi-Yau 3-fold appearing in string compactification.
//!
//! For 2D cross-sectional slices (suitable for real-time rendering), we work
//! with the complex curve:
//!
//! ```text
//! z₁ⁿ + z₂ⁿ = 1
//! ```
//!
//! Parametrized as:
//!
//! ```text
//! z₁ = exp(i·2πk₁/n) · cos(θ)^(2/n) · exp(i·φ₁)
//! z₂ = exp(i·2πk₂/n) · sin(θ)^(2/n) · exp(i·φ₂)
//! ```
//!
//! where `θ ∈ [0, π/2]`, `φ₁, φ₂ ∈ [0, 2π/n]`.
//!
//! The integer pairs `(k₁, k₂)` select distinct topological patches of the surface.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::manifold::{CalabiYauConfig, generate_manifold_mesh};
//!
//! let config = CalabiYauConfig::default();
//! if let Ok(mesh) = generate_manifold_mesh(&config) {
//!     assert!(!mesh.vertices.is_empty());
//!     assert!(!mesh.triangles.is_empty());
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, TAU};
use std::fmt;

// ─────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────

/// Errors that can occur during Calabi-Yau manifold computation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ManifoldError {
    /// The polynomial degree `n` must be ≥ 2.
    InvalidDegree,
    /// The grid resolution must be ≥ 2.
    InvalidResolution,
    /// Mesh generation produced no geometry.
    EmptyMesh,
    /// A numerical computation step failed.
    ComputationFailed(String),
}

impl fmt::Display for ManifoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDegree => write!(f, "invalid degree: manifold degree must be at least 2"),
            Self::InvalidResolution => {
                write!(f, "invalid resolution: grid resolution must be at least 2")
            }
            Self::EmptyMesh => write!(f, "empty mesh: surface evaluation produced no geometry"),
            Self::ComputationFailed(msg) => write!(f, "computation failed: {msg}"),
        }
    }
}

impl std::error::Error for ManifoldError {}

// ─────────────────────────────────────────────────────────────────
// Complex numbers
// ─────────────────────────────────────────────────────────────────

/// A complex number `re + i·im` with 64-bit floating-point components.
///
/// All arithmetic operations are closed: they accept and return `Complex`,
/// enabling method chaining.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::Complex;
///
/// let z = Complex { re: 3.0, im: 4.0 };
/// assert!((z.norm() - 5.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Complex {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

impl Complex {
    /// Constructs a complex number from its real and imaginary parts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(1.0, -2.0);
    /// assert_eq!(z.re, 1.0);
    /// assert_eq!(z.im, -2.0);
    /// ```
    #[must_use]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Constructs a complex number from polar form `r·exp(i·θ)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    /// use std::f64::consts::PI;
    ///
    /// let z = Complex::from_polar(1.0, PI);
    /// assert!((z.re - (-1.0)).abs() < 1e-12);
    /// assert!(z.im.abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Returns the squared modulus `|z|²`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(3.0, 4.0);
    /// assert_eq!(z.norm_sq(), 25.0);
    /// ```
    #[must_use]
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Returns the modulus `|z|`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(3.0, 4.0);
    /// assert!((z.norm() - 5.0).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn norm(self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Returns the complex conjugate `re - i·im`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(3.0, 4.0);
    /// let conj = z.conj();
    /// assert_eq!(conj, Complex::new(3.0, -4.0));
    /// ```
    #[must_use]
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Scales the complex number by a real scalar.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(2.0, 3.0).scale(2.0);
    /// assert_eq!(z, Complex::new(4.0, 6.0));
    /// ```
    #[must_use]
    pub fn scale(self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    /// Adds two complex numbers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(1.0, 2.0).add_vector(Complex::new(3.0, 4.0));
    /// assert_eq!(z, Complex::new(4.0, 6.0));
    /// ```
    #[must_use]
    pub fn add_vector(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }

    /// Multiplies two complex numbers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(1.0, 2.0).scale_by(Complex::new(3.0, 4.0));
    /// // (1+2i)(3+4i) = 3+4i+6i-8 = -5+10i
    /// assert_eq!(z, Complex::new(-5.0, 10.0));
    /// ```
    #[must_use]
    pub fn scale_by(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }

    /// Raises the complex number to a non-negative integer power via repeated
    /// multiplication.
    ///
    /// `z.pow_n(0)` returns `1 + 0i` (the multiplicative identity).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::manifold::Complex;
    ///
    /// let z = Complex::new(0.0, 1.0); // i
    /// let z4 = z.pow_n(4);            // i⁴ = 1
    /// assert!((z4.re - 1.0).abs() < 1e-12);
    /// assert!(z4.im.abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn pow_n(self, n: u32) -> Self {
        let mut acc = Self::new(1.0, 0.0);
        for _ in 0..n {
            acc = acc.scale_by(self);
        }
        acc
    }
}

// ─────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────

/// Method used to embed the 4D complex surface into 3D Euclidean space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum ProjectionMethod {
    /// Stereographic projection from C² ≅ R⁴ to S³ ⊂ R³.
    #[default]
    Stereographic,
    /// Orthographic projection: `(Re z₁, Re z₂, Im z₁)`.
    Orthographic,
}

/// Configuration for Calabi-Yau manifold mesh generation.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, ProjectionMethod};
///
/// let config = CalabiYauConfig {
///     degree: 5,
///     resolution: 30,
///     alpha: 1.0,
///     k1: 0,
///     k2: 1,
///     scale: 2.0,
///     projection_method: ProjectionMethod::Orthographic,
/// };
/// assert_eq!(config.degree, 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalabiYauConfig {
    /// Degree `n` of the Fermat polynomial `z₁ⁿ + z₂ⁿ = 1`.
    /// Must be ≥ 2. Default: 5 (the quintic).
    pub degree: u32,
    /// Number of sample points per parameter axis.
    /// Total vertices = `resolution²`. Must be ≥ 2. Default: 50.
    pub resolution: usize,
    /// Deformation (shape) parameter `α`. Default: 1.0.
    pub alpha: f64,
    /// First surface-selector integer `k₁ ∈ {0, …, n−1}`. Default: 0.
    pub k1: u32,
    /// Second surface-selector integer `k₂ ∈ {0, …, n−1}`. Default: 1.
    pub k2: u32,
    /// Uniform scale applied to projected coordinates. Default: 2.0.
    pub scale: f64,
    /// Projection method for embedding in R³. Default: `Stereographic`.
    pub projection_method: ProjectionMethod,
}

impl Default for CalabiYauConfig {
    fn default() -> Self {
        Self {
            degree: 5,
            resolution: 50,
            alpha: 1.0,
            k1: 0,
            k2: 1,
            scale: 2.0,
            projection_method: ProjectionMethod::Stereographic,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Mesh types
// ─────────────────────────────────────────────────────────────────

/// A single vertex on the Calabi-Yau surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SurfaceVertex {
    /// Position in R³.
    pub position: [f64; 3],
    /// Unit surface normal at this vertex.
    pub normal: [f64; 3],
    /// UV texture coordinates, both in `[0, 1]`.
    pub uv: [f64; 2],
    /// Estimated Gaussian curvature at this vertex.
    pub curvature: f64,
}

/// An indexed triangle referencing three vertices by index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SurfaceTriangle {
    /// Counter-clockwise vertex indices into the parent mesh's vertex array.
    pub indices: [usize; 3],
}

/// A complete triangulated mesh of one Calabi-Yau surface patch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifoldMesh {
    /// All surface vertices.
    pub vertices: Vec<SurfaceVertex>,
    /// All triangles.
    pub triangles: Vec<SurfaceTriangle>,
    /// Topological genus of the surface.
    pub genus: usize,
    /// Polynomial degree used to generate this mesh.
    pub degree: u32,
}

// ─────────────────────────────────────────────────────────────────
// Core math helpers
// ─────────────────────────────────────────────────────────────────

/// Raises a complex number to a non-negative integer power.
///
/// This is equivalent to `z.pow_n(n)` and is provided as a free function for
/// ergonomic use in surface evaluation pipelines.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{Complex, complex_power};
///
/// let z = Complex::new(1.0, 0.0);
/// let result = complex_power(z, 5);
/// assert!((result.re - 1.0).abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn complex_power(z: Complex, n: u32) -> Complex {
    z.pow_n(n)
}

// ─────────────────────────────────────────────────────────────────
// Surface parametrization helpers
// ─────────────────────────────────────────────────────────────────

/// Returns the complex coordinates `(z₁, z₂)` for a given parameter triple
/// `(theta, phi1, phi2)` under the supplied configuration.
///
/// The Fermat surface patch is:
/// ```text
/// z₁ = exp(i·2π·k₁/n) · cos(θ)^(2/n) · exp(i·φ₁·α)
/// z₂ = exp(i·2π·k₂/n) · sin(θ)^(2/n) · exp(i·φ₂·α)
/// ```
fn surface_coords(
    theta: f64,
    phi1: f64,
    phi2: f64,
    config: &CalabiYauConfig,
) -> (Complex, Complex) {
    let n = config.degree as f64;
    let exp = 2.0 / n;

    // Phase rotations from k selectors
    let phase1 = TAU * (config.k1 as f64) / n;
    let phase2 = TAU * (config.k2 as f64) / n;

    // Magnitudes (cos/sin raised to 2/n power, clamped to avoid NaN at boundaries)
    let mag1 = theta.cos().abs().powf(exp);
    let mag2 = theta.sin().abs().powf(exp);

    // z₁ = phase_rot₁ · mag1 · exp(i·φ₁·α)
    let rot1 = Complex::from_polar(1.0, phase1);
    let inner1 = Complex::from_polar(mag1, phi1 * config.alpha);
    let z1 = rot1.scale_by(inner1);

    // z₂ = phase_rot₂ · mag2 · exp(i·φ₂·α)
    let rot2 = Complex::from_polar(1.0, phase2);
    let inner2 = Complex::from_polar(mag2, phi2 * config.alpha);
    let z2 = rot2.scale_by(inner2);

    (z1, z2)
}

// ─────────────────────────────────────────────────────────────────
// Projection
// ─────────────────────────────────────────────────────────────────

/// Stereographic projection from C² to R³.
///
/// Maps `(z₁, z₂) ∈ C²` using the four real coordinates
/// `(x, y, u, v) = (Re z₁, Im z₁, Re z₂, Im z₂)` via the formula:
///
/// ```text
/// denom = 1 + x² + y² + u² + v²
/// X = 2x / denom
/// Y = 2u / denom
/// Z = 2v / denom
/// ```
///
/// The denominator is clamped to a minimum of `1e-10` to prevent division by
/// zero at the projection pole.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{Complex, project_stereographic};
///
/// let origin = Complex::new(0.0, 0.0);
/// let p = project_stereographic(origin, origin);
/// assert_eq!(p, [0.0, 0.0, 0.0]);
/// ```
#[must_use]
pub fn project_stereographic(z1: Complex, z2: Complex) -> [f64; 3] {
    let (x, y, u, v) = (z1.re, z1.im, z2.re, z2.im);
    let denom = (1.0 + x * x + y * y + u * u + v * v).max(1e-10);
    [2.0 * x / denom, 2.0 * u / denom, 2.0 * v / denom]
}

/// Orthographic projection from C² to R³.
///
/// Simply reads three of the four real Cartesian coordinates:
///
/// ```text
/// (Re z₁, Re z₂, Im z₁)
/// ```
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{Complex, project_orthographic};
///
/// let z1 = Complex::new(1.0, 2.0);
/// let z2 = Complex::new(3.0, 4.0);
/// let p = project_orthographic(z1, z2);
/// assert_eq!(p, [1.0, 3.0, 2.0]);
/// ```
#[must_use]
pub fn project_orthographic(z1: Complex, z2: Complex) -> [f64; 3] {
    [z1.re, z2.re, z1.im]
}

// ─────────────────────────────────────────────────────────────────
// Surface evaluation
// ─────────────────────────────────────────────────────────────────

/// Evaluates a single point on the Calabi-Yau surface and projects it to R³.
///
/// Parameters:
/// - `theta  ∈ [0, π/2]` — polar angle mixing z₁/z₂ magnitudes.
/// - `phi1   ∈ [0, 2π/n]` — phase of z₁.
/// - `phi2   ∈ [0, 2π/n]` — phase of z₂.
///
/// Returns an `[x, y, z]` triple scaled by `config.scale`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, evaluate_surface_point};
/// use std::f64::consts::FRAC_PI_4;
///
/// let config = CalabiYauConfig::default();
/// let p = evaluate_surface_point(FRAC_PI_4, 0.0, 0.0, &config);
/// assert!(p.iter().all(|v| v.is_finite()));
/// ```
#[must_use]
pub fn evaluate_surface_point(
    theta: f64,
    phi1: f64,
    phi2: f64,
    config: &CalabiYauConfig,
) -> [f64; 3] {
    let (z1, z2) = surface_coords(theta, phi1, phi2, config);
    let raw = match config.projection_method {
        ProjectionMethod::Stereographic => project_stereographic(z1, z2),
        ProjectionMethod::Orthographic => project_orthographic(z1, z2),
    };
    [
        raw[0] * config.scale,
        raw[1] * config.scale,
        raw[2] * config.scale,
    ]
}

// ─────────────────────────────────────────────────────────────────
// Normals and curvature
// ─────────────────────────────────────────────────────────────────

/// Normalizes a 3-vector, returning `[0, 0, 1]` if the input has zero length.
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Cross product of two 3-vectors.
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Subtracts two 3-vectors: `a - b`.
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Computes the surface normal at `(theta, phi1, phi2)` by numerical
/// differentiation, using a central-difference scheme where possible.
///
/// Returns an approximately unit-length 3-vector. If the partial derivatives are
/// parallel (degenerate point), returns `[0.0, 0.0, 1.0]` as a fallback.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, compute_surface_normal};
/// use std::f64::consts::FRAC_PI_4;
///
/// let config = CalabiYauConfig::default();
/// let n = compute_surface_normal(FRAC_PI_4, 0.5, 0.5, &config);
/// let length = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
/// // Normal should be approximately unit-length (allow some slack from numerics)
/// assert!((length - 1.0).abs() < 0.1);
/// ```
#[must_use]
pub fn compute_surface_normal(
    theta: f64,
    phi1: f64,
    phi2: f64,
    config: &CalabiYauConfig,
) -> [f64; 3] {
    let h = 1e-4_f64;
    // ∂/∂theta  (clamp to avoid boundary issues)
    let dp_theta = {
        let t_hi = (theta + h).min(FRAC_PI_2 - h);
        let t_lo = (theta - h).max(h);
        let hi = evaluate_surface_point(t_hi, phi1, phi2, config);
        let lo = evaluate_surface_point(t_lo, phi1, phi2, config);
        sub3(hi, lo)
    };
    // ∂/∂phi1
    let dp_phi1 = {
        let hi = evaluate_surface_point(theta, phi1 + h, phi2, config);
        let lo = evaluate_surface_point(theta, phi1 - h, phi2, config);
        sub3(hi, lo)
    };
    normalize(cross(dp_theta, dp_phi1))
}

/// Estimates the Gaussian curvature at `(theta, phi1, phi2)` via a
/// discrete Laplacian approximation using finite differences.
///
/// The estimate is computed as the average magnitude of the second-order
/// difference of position in the `theta` and `phi1` directions. It is a coarse
/// but numerically stable approximation suitable for real-time visualization.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, estimate_curvature};
/// use std::f64::consts::FRAC_PI_4;
///
/// let config = CalabiYauConfig::default();
/// let k = estimate_curvature(FRAC_PI_4, 0.5, 0.5, &config);
/// assert!(k.is_finite());
/// ```
#[must_use]
pub fn estimate_curvature(theta: f64, phi1: f64, phi2: f64, config: &CalabiYauConfig) -> f64 {
    let h = 1e-3_f64;
    let center = evaluate_surface_point(theta, phi1, phi2, config);
    let t_hi = (theta + h).min(FRAC_PI_2 - h);
    let t_lo = (theta - h).max(h);
    let p_theta_hi = evaluate_surface_point(t_hi, phi1, phi2, config);
    let p_theta_lo = evaluate_surface_point(t_lo, phi1, phi2, config);
    let p_phi1_hi = evaluate_surface_point(theta, phi1 + h, phi2, config);
    let p_phi1_lo = evaluate_surface_point(theta, phi1 - h, phi2, config);

    // Discrete Laplacian ≈ ‖(p_hi + p_lo - 2·center) / h²‖  for each direction
    let laplacian_theta: f64 = (0..3)
        .map(|i| (p_theta_hi[i] + p_theta_lo[i] - 2.0 * center[i]) / (h * h))
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt();

    let laplacian_phi: f64 = (0..3)
        .map(|i| (p_phi1_hi[i] + p_phi1_lo[i] - 2.0 * center[i]) / (h * h))
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt();

    // Average of the two principal curvature magnitudes
    (laplacian_theta + laplacian_phi) * 0.5
}

// ─────────────────────────────────────────────────────────────────
// Mesh generation
// ─────────────────────────────────────────────────────────────────

/// Generates a full triangulated mesh for one Calabi-Yau surface patch.
///
/// Samples the parameter space `(theta, phi1)` on a uniform `resolution × resolution`
/// grid (phi2 is fixed at its midpoint for a 2D cross-section). Triangulates
/// the resulting quad grid into two triangles per quad.
///
/// # Errors
///
/// Returns:
/// - [`ManifoldError::InvalidDegree`] if `config.degree < 2`.
/// - [`ManifoldError::InvalidResolution`] if `config.resolution < 2`.
/// - [`ManifoldError::EmptyMesh`] if evaluation produces no geometry.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, generate_manifold_mesh};
///
/// let config = CalabiYauConfig { resolution: 10, ..CalabiYauConfig::default() };
/// if let Ok(mesh) = generate_manifold_mesh(&config) {
///     assert_eq!(mesh.vertices.len(), 100);
/// }
/// ```
pub fn generate_manifold_mesh(config: &CalabiYauConfig) -> Result<ManifoldMesh, ManifoldError> {
    if config.degree < 2 {
        return Err(ManifoldError::InvalidDegree);
    }
    if config.resolution < 2 {
        return Err(ManifoldError::InvalidResolution);
    }

    let res = config.resolution;
    let n = config.degree as f64;

    // Parameter ranges
    let theta_range = FRAC_PI_2; // [0, π/2]
    let phi_range = TAU / n; // [0, 2π/n]
    // phi2 fixed at midpoint for 2D cross-section
    let phi2_fixed = phi_range * 0.5;

    let mut vertices: Vec<SurfaceVertex> = Vec::with_capacity(res * res);

    for j in 0..res {
        for i in 0..res {
            let u_t = i as f64 / (res - 1) as f64; // ∈ [0, 1]
            let u_p = j as f64 / (res - 1) as f64; // ∈ [0, 1]

            // Clamp theta away from exact 0 and π/2 to avoid 0^(2/n) edge cases
            let theta = (u_t * theta_range).clamp(1e-6, theta_range - 1e-6);
            let phi1 = u_p * phi_range;

            let position = evaluate_surface_point(theta, phi1, phi2_fixed, config);
            let normal = compute_surface_normal(theta, phi1, phi2_fixed, config);
            let curvature = estimate_curvature(theta, phi1, phi2_fixed, config);

            vertices.push(SurfaceVertex {
                position,
                normal,
                uv: [u_t, u_p],
                curvature,
            });
        }
    }

    if vertices.is_empty() {
        return Err(ManifoldError::EmptyMesh);
    }

    // Triangulate: each (i, j) quad → 2 triangles
    // Vertex index: v(i, j) = j * res + i
    let mut triangles: Vec<SurfaceTriangle> = Vec::with_capacity((res - 1) * (res - 1) * 2);
    for j in 0..(res - 1) {
        for i in 0..(res - 1) {
            let v00 = j * res + i;
            let v10 = j * res + i + 1;
            let v01 = (j + 1) * res + i;
            let v11 = (j + 1) * res + i + 1;
            triangles.push(SurfaceTriangle {
                indices: [v00, v10, v11],
            });
            triangles.push(SurfaceTriangle {
                indices: [v00, v11, v01],
            });
        }
    }

    let genus = compute_genus(config.degree);

    Ok(ManifoldMesh {
        vertices,
        triangles,
        genus,
        degree: config.degree,
    })
}

// ─────────────────────────────────────────────────────────────────
// Topology
// ─────────────────────────────────────────────────────────────────

/// Computes the topological genus of a Fermat hypersurface of degree `n`.
///
/// For the Fermat quintic 3-fold (complex dimension 3), the Euler characteristic
/// is `−200`, which determines the Hodge numbers. As a visualization hint the
/// formula `(n−1)(n−2)(n−3)/6` is used (analogous to the Riemann-Hurwitz
/// formula applied to the 3-fold). For `n ≤ 2` it returns `0`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::compute_genus;
///
/// // Quintic 3-fold: (5-1)(5-2)(5-3)/6 = 4*3*2/6 = 4
/// assert_eq!(compute_genus(5), 4);
/// // Cubic: (3-1)(3-2)(3-3)/6 = 0
/// assert_eq!(compute_genus(3), 0);
/// ```
#[must_use]
pub fn compute_genus(degree: u32) -> usize {
    if degree < 2 {
        return 0;
    }
    let n = degree as usize;
    (n.saturating_sub(1)) * (n.saturating_sub(2)) * (n.saturating_sub(3)) / 6
}

// ─────────────────────────────────────────────────────────────────
// Mesh analysis
// ─────────────────────────────────────────────────────────────────

/// Returns the total surface area of the mesh, summed over all triangles.
///
/// Each triangle's area is computed as half the magnitude of its edge cross product.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, generate_manifold_mesh, mesh_surface_area};
///
/// let config = CalabiYauConfig { resolution: 10, ..CalabiYauConfig::default() };
/// if let Ok(mesh) = generate_manifold_mesh(&config) {
///     let area = mesh_surface_area(&mesh);
///     assert!(area > 0.0);
/// }
/// ```
#[must_use]
pub fn mesh_surface_area(mesh: &ManifoldMesh) -> f64 {
    mesh.triangles.iter().fold(0.0, |acc, tri| {
        let [i0, i1, i2] = tri.indices;
        // Guard against out-of-range indices (should never occur for well-formed meshes)
        let (Some(v0), Some(v1), Some(v2)) = (
            mesh.vertices.get(i0),
            mesh.vertices.get(i1),
            mesh.vertices.get(i2),
        ) else {
            return acc;
        };
        let p0 = v0.position;
        let p1 = v1.position;
        let p2 = v2.position;
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        let c = cross(e1, e2);
        let area = 0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        acc + area
    })
}

/// Returns the axis-aligned bounding box (AABB) of the mesh as `(min, max)`.
///
/// Each component of `min` is ≤ the corresponding component of `max`.
/// Returns `([0.0; 3], [0.0; 3])` if the mesh is empty.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::manifold::{CalabiYauConfig, generate_manifold_mesh, mesh_bounding_box};
///
/// let config = CalabiYauConfig { resolution: 10, ..CalabiYauConfig::default() };
/// if let Ok(mesh) = generate_manifold_mesh(&config) {
///     let (min, max) = mesh_bounding_box(&mesh);
///     for i in 0..3 {
///         assert!(min[i] <= max[i]);
///     }
/// }
/// ```
#[must_use]
pub fn mesh_bounding_box(mesh: &ManifoldMesh) -> ([f64; 3], [f64; 3]) {
    if mesh.vertices.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for v in &mesh.vertices {
        for i in 0..3 {
            if v.position[i] < min[i] {
                min[i] = v.position[i];
            }
            if v.position[i] > max[i] {
                max[i] = v.position[i];
            }
        }
    }
    (min, max)
}

// ─────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

    // ── helpers ───────────────────────────────────────────────────

    /// Build a small mesh or return an empty sentinel.
    ///
    /// Callers must check `.vertices.is_empty()` when the result matters;
    /// most tests use resolution ≥ 2 with degree 5, which always succeeds.
    fn small_mesh(res: usize) -> ManifoldMesh {
        let config = CalabiYauConfig {
            resolution: res,
            ..CalabiYauConfig::default()
        };
        generate_manifold_mesh(&config).unwrap_or(ManifoldMesh {
            vertices: vec![],
            triangles: vec![],
            genus: 0,
            degree: 5,
        })
    }

    // ── Complex arithmetic ────────────────────────────────────────

    #[test]
    fn test_complex_add() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, -1.0);
        let c = a.add_vector(b);
        assert!((c.re - 4.0).abs() < 1e-12);
        assert!((c.im - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_complex_mul() {
        // (1+2i)(3+4i) = 3+4i+6i-8 = -5+10i
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);
        let c = a.scale_by(b);
        assert!((c.re - (-5.0)).abs() < 1e-12);
        assert!((c.im - 10.0).abs() < 1e-12);
    }

    #[test]
    fn test_complex_norm() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.norm() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_complex_norm_sq() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.norm_sq() - 25.0).abs() < 1e-12);
    }

    #[test]
    fn test_complex_conj() {
        let z = Complex::new(3.0, 4.0);
        let c = z.conj();
        assert_eq!(c, Complex::new(3.0, -4.0));
    }

    #[test]
    fn test_complex_conj_product_is_real() {
        // z * conj(z) should be purely real and equal to |z|²
        let z = Complex::new(3.0, 4.0);
        let p = z.scale_by(z.conj());
        assert!(p.im.abs() < 1e-12);
        assert!((p.re - 25.0).abs() < 1e-12);
    }

    #[test]
    fn test_complex_pow_n_zero() {
        let z = Complex::new(2.0, 3.0);
        let result = z.pow_n(0);
        assert_eq!(result, Complex::new(1.0, 0.0));
    }

    #[test]
    fn test_complex_pow_n_one() {
        let z = Complex::new(2.0, 3.0);
        let result = z.pow_n(1);
        assert_eq!(result, z);
    }

    #[test]
    fn test_complex_pow_n_i_fourth() {
        // i⁴ = 1
        let i = Complex::new(0.0, 1.0);
        let result = i.pow_n(4);
        assert!((result.re - 1.0).abs() < 1e-12);
        assert!(result.im.abs() < 1e-12);
    }

    #[test]
    fn test_complex_pow_n_i_squared() {
        // i² = -1
        let i = Complex::new(0.0, 1.0);
        let result = i.pow_n(2);
        assert!((result.re - (-1.0)).abs() < 1e-12);
        assert!(result.im.abs() < 1e-12);
    }

    #[test]
    fn test_complex_scale() {
        let z = Complex::new(2.0, 3.0).scale(3.0);
        assert_eq!(z, Complex::new(6.0, 9.0));
    }

    // ── Complex::from_polar ───────────────────────────────────────

    #[test]
    fn test_from_polar_unit_zero_angle() {
        let z = Complex::from_polar(1.0, 0.0);
        assert!((z.re - 1.0).abs() < 1e-12);
        assert!(z.im.abs() < 1e-12);
    }

    #[test]
    fn test_from_polar_pi() {
        let z = Complex::from_polar(1.0, PI);
        assert!((z.re - (-1.0)).abs() < 1e-12);
        assert!(z.im.abs() < 1e-12);
    }

    #[test]
    fn test_from_polar_half_pi() {
        let z = Complex::from_polar(2.0, FRAC_PI_2);
        assert!(z.re.abs() < 1e-12);
        assert!((z.im - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_from_polar_norm_preserved() {
        let r = 3.7_f64;
        let z = Complex::from_polar(r, 1.23);
        assert!((z.norm() - r).abs() < 1e-12);
    }

    // ── complex_power free function ───────────────────────────────

    #[test]
    fn test_complex_power_free_function() {
        let z = Complex::new(1.0, 0.0);
        let r = complex_power(z, 5);
        assert!((r.re - 1.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn test_complex_power_quintic() {
        // (cos(π/10) + i·sin(π/10))^5 = exp(i·π/2) = i
        let theta = PI / 10.0;
        let z = Complex::from_polar(1.0, theta);
        let r = complex_power(z, 5);
        // Should be close to exp(i·π/2) = 0 + 1i
        assert!(r.re.abs() < 1e-10, "Re={}", r.re);
        assert!((r.im - 1.0).abs() < 1e-10, "Im={}", r.im);
    }

    // ── Fermat equation verification ──────────────────────────────

    #[test]
    fn test_fermat_equation_satisfied() {
        // z₁ⁿ + z₂ⁿ = 1 holds when phi₁ = phi₂ = 0 (real-valued case):
        // z₁ = cos(θ)^(2/n), z₂ = sin(θ)^(2/n)  ⟹  z₁ⁿ + z₂ⁿ = cos²θ + sin²θ = 1
        let config = CalabiYauConfig {
            degree: 5,
            alpha: 1.0,
            k1: 0,
            k2: 0,
            ..CalabiYauConfig::default()
        };
        let n = config.degree;
        let theta = FRAC_PI_4;
        let phi1 = 0.0;
        let phi2 = 0.0;
        let (z1, z2) = surface_coords(theta, phi1, phi2, &config);
        let sum = complex_power(z1, n).add_vector(complex_power(z2, n));
        // Should be approximately 1 + 0i
        assert!((sum.re - 1.0).abs() < 1e-10, "Re(z1^n + z2^n) = {}", sum.re);
        assert!(sum.im.abs() < 1e-10, "Im(z1^n + z2^n) = {}", sum.im);
    }

    // ── evaluate_surface_point ────────────────────────────────────

    #[test]
    fn test_evaluate_surface_point_finite() {
        let config = CalabiYauConfig::default();
        let p = evaluate_surface_point(FRAC_PI_4, 0.3, 0.3, &config);
        assert!(
            p.iter().all(|v| v.is_finite()),
            "Expected finite coords: {p:?}"
        );
    }

    #[test]
    fn test_evaluate_surface_point_theta_near_zero_z2_small() {
        // At theta → 0, sin(theta) → 0, so |z2| → 0
        let config = CalabiYauConfig {
            projection_method: ProjectionMethod::Orthographic,
            ..CalabiYauConfig::default()
        };
        // theta very small → mag2 = sin(theta)^(2/n) ≈ 0
        let p_small_theta = evaluate_surface_point(1e-5, 0.0, 0.0, &config);
        let p_mid = evaluate_surface_point(FRAC_PI_4, 0.0, 0.0, &config);
        // Re(z2) component (index 1 in orthographic) should be smaller
        assert!(p_small_theta[1].abs() <= p_mid[1].abs() + 1e-10);
    }

    #[test]
    fn test_evaluate_surface_point_theta_near_half_pi_z1_small() {
        // At theta → π/2, cos(theta) → 0, so |z1| → 0
        let config = CalabiYauConfig {
            projection_method: ProjectionMethod::Orthographic,
            ..CalabiYauConfig::default()
        };
        let near_half_pi = FRAC_PI_2 - 1e-5;
        let p_large = evaluate_surface_point(near_half_pi, 0.0, 0.0, &config);
        let p_mid = evaluate_surface_point(FRAC_PI_4, 0.0, 0.0, &config);
        // Re(z1) (index 0) should approach 0
        assert!(p_large[0].abs() <= p_mid[0].abs() + 1e-6);
    }

    // ── compute_surface_normal ────────────────────────────────────

    #[test]
    fn test_compute_surface_normal_unit_length() {
        let config = CalabiYauConfig::default();
        let n = compute_surface_normal(FRAC_PI_4, 0.5, 0.5, &config);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 0.1, "Normal length: {len}");
    }

    #[test]
    fn test_compute_surface_normal_finite() {
        let config = CalabiYauConfig::default();
        let n = compute_surface_normal(0.3, 0.2, 0.2, &config);
        assert!(n.iter().all(|v| v.is_finite()));
    }

    // ── estimate_curvature ────────────────────────────────────────

    #[test]
    fn test_estimate_curvature_finite() {
        let config = CalabiYauConfig::default();
        let k = estimate_curvature(FRAC_PI_4, 0.5, 0.5, &config);
        assert!(k.is_finite(), "curvature should be finite, got {k}");
    }

    #[test]
    fn test_estimate_curvature_non_negative() {
        let config = CalabiYauConfig::default();
        let k = estimate_curvature(0.3, 0.2, 0.2, &config);
        assert!(
            k >= 0.0,
            "curvature estimate should be non-negative, got {k}"
        );
    }

    // ── generate_manifold_mesh ────────────────────────────────────

    #[test]
    fn test_generate_manifold_mesh_default() {
        let config = CalabiYauConfig::default();
        let result = generate_manifold_mesh(&config);
        assert!(
            result.is_ok(),
            "generate_manifold_mesh error: {:?}",
            result.err()
        );
        if let Ok(mesh) = result {
            assert!(!mesh.vertices.is_empty());
            assert!(!mesh.triangles.is_empty());
            assert_eq!(mesh.degree, 5);
        }
    }

    #[test]
    fn test_generate_manifold_mesh_vertex_count() {
        let mesh = small_mesh(10);
        assert_eq!(mesh.vertices.len(), 100);
    }

    #[test]
    fn test_generate_manifold_mesh_triangle_count() {
        let mesh = small_mesh(10);
        // (res-1)² * 2 triangles
        assert_eq!(mesh.triangles.len(), 9 * 9 * 2);
    }

    #[test]
    fn test_generate_manifold_mesh_resolution_zero_error() {
        let config = CalabiYauConfig {
            resolution: 0,
            ..CalabiYauConfig::default()
        };
        let result = generate_manifold_mesh(&config);
        assert_eq!(result, Err(ManifoldError::InvalidResolution));
    }

    #[test]
    fn test_generate_manifold_mesh_resolution_one_error() {
        let config = CalabiYauConfig {
            resolution: 1,
            ..CalabiYauConfig::default()
        };
        let result = generate_manifold_mesh(&config);
        assert_eq!(result, Err(ManifoldError::InvalidResolution));
    }

    #[test]
    fn test_generate_manifold_mesh_degree_zero_error() {
        let config = CalabiYauConfig {
            degree: 0,
            ..CalabiYauConfig::default()
        };
        let result = generate_manifold_mesh(&config);
        assert_eq!(result, Err(ManifoldError::InvalidDegree));
    }

    #[test]
    fn test_generate_manifold_mesh_degree_one_error() {
        let config = CalabiYauConfig {
            degree: 1,
            ..CalabiYauConfig::default()
        };
        let result = generate_manifold_mesh(&config);
        assert_eq!(result, Err(ManifoldError::InvalidDegree));
    }

    #[test]
    fn test_generate_manifold_mesh_all_vertices_finite() {
        let mesh = small_mesh(8);
        for v in &mesh.vertices {
            assert!(
                v.position.iter().all(|c| c.is_finite()),
                "non-finite vertex: {v:?}"
            );
            assert!(
                v.normal.iter().all(|c| c.is_finite()),
                "non-finite normal: {v:?}"
            );
        }
    }

    // ── compute_genus ─────────────────────────────────────────────

    #[test]
    fn test_compute_genus_quintic() {
        // (5-1)(5-2)(5-3)/6 = 4*3*2/6 = 4
        assert_eq!(compute_genus(5), 4);
    }

    #[test]
    fn test_compute_genus_cubic() {
        // (3-1)(3-2)(3-3)/6 = 2*1*0/6 = 0
        assert_eq!(compute_genus(3), 0);
    }

    #[test]
    fn test_compute_genus_quartic() {
        // (4-1)(4-2)(4-3)/6 = 3*2*1/6 = 1
        assert_eq!(compute_genus(4), 1);
    }

    #[test]
    fn test_compute_genus_zero_degree() {
        assert_eq!(compute_genus(0), 0);
    }

    // ── project_stereographic ─────────────────────────────────────

    #[test]
    fn test_project_stereographic_origin() {
        let z = Complex::new(0.0, 0.0);
        let p = project_stereographic(z, z);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_project_stereographic_bounded() {
        // Stereographic projection of a bounded point in C²
        let z1 = Complex::new(0.5, 0.0);
        let z2 = Complex::new(0.0, 0.5);
        let p = project_stereographic(z1, z2);
        for c in p {
            assert!(c.is_finite());
            assert!(c.abs() < 3.0, "coord out of expected range: {c}");
        }
    }

    // ── project_orthographic ──────────────────────────────────────

    #[test]
    fn test_project_orthographic_simple() {
        let z1 = Complex::new(1.0, 2.0);
        let z2 = Complex::new(3.0, 4.0);
        let p = project_orthographic(z1, z2);
        assert_eq!(p, [1.0, 3.0, 2.0]);
    }

    // ── mesh_surface_area ─────────────────────────────────────────

    #[test]
    fn test_mesh_surface_area_positive() {
        let mesh = small_mesh(8);
        let area = mesh_surface_area(&mesh);
        assert!(area > 0.0, "surface area should be positive, got {area}");
    }

    #[test]
    fn test_mesh_surface_area_empty_mesh() {
        let mesh = ManifoldMesh {
            vertices: vec![],
            triangles: vec![],
            genus: 0,
            degree: 5,
        };
        assert_eq!(mesh_surface_area(&mesh), 0.0);
    }

    // ── mesh_bounding_box ─────────────────────────────────────────

    #[test]
    fn test_mesh_bounding_box_min_le_max() {
        let mesh = small_mesh(8);
        let (min, max) = mesh_bounding_box(&mesh);
        for i in 0..3 {
            assert!(
                min[i] <= max[i],
                "axis {i}: min={} > max={}",
                min[i],
                max[i]
            );
        }
    }

    #[test]
    fn test_mesh_bounding_box_empty() {
        let mesh = ManifoldMesh {
            vertices: vec![],
            triangles: vec![],
            genus: 0,
            degree: 5,
        };
        let (min, max) = mesh_bounding_box(&mesh);
        assert_eq!(min, [0.0; 3]);
        assert_eq!(max, [0.0; 3]);
    }

    // ── Serde roundtrip ───────────────────────────────────────────

    #[test]
    fn test_serde_roundtrip_manifold_mesh() {
        let mesh = small_mesh(4);
        // Ensure all vertex values are finite before serializing
        for (i, v) in mesh.vertices.iter().enumerate() {
            for d in 0..3 {
                assert!(v.position[d].is_finite(), "vertex {i} pos[{d}] not finite");
                assert!(v.normal[d].is_finite(), "vertex {i} norm[{d}] not finite");
            }
            assert!(v.curvature.is_finite(), "vertex {i} curvature not finite");
        }
        let json = serde_json::to_string(&mesh);
        assert!(json.is_ok(), "serialize failed: {:?}", json.as_ref().err());
        let json = json.unwrap_or_default();
        let back: Result<ManifoldMesh, _> = serde_json::from_str(&json);
        assert!(
            back.is_ok(),
            "deserialize failed: {:?}",
            back.as_ref().err()
        );
        let back = back.unwrap_or(ManifoldMesh {
            vertices: vec![],
            triangles: vec![],
            genus: 0,
            degree: 0,
        });
        assert_eq!(mesh.vertices.len(), back.vertices.len(), "vertex count");
        assert_eq!(mesh.triangles.len(), back.triangles.len(), "triangle count");
        assert_eq!(mesh.genus, back.genus, "genus");
        assert_eq!(mesh.degree, back.degree, "degree");
    }

    #[test]
    fn test_serde_roundtrip_complex() {
        let z = Complex::new(1.234, -5.678);
        let json_result = serde_json::to_string(&z);
        assert!(
            json_result.is_ok(),
            "serialize failed: {:?}",
            json_result.err()
        );
        if let Ok(json) = json_result {
            let back_result = serde_json::from_str::<Complex>(&json);
            assert!(
                back_result.is_ok(),
                "deserialize failed: {:?}",
                back_result.err()
            );
            if let Ok(back) = back_result {
                assert_eq!(z, back);
            }
        }
    }

    // ── CalabiYauConfig defaults ──────────────────────────────────

    #[test]
    fn test_calabi_yau_config_default_values() {
        let cfg = CalabiYauConfig::default();
        assert_eq!(cfg.degree, 5);
        assert_eq!(cfg.resolution, 50);
        assert!((cfg.alpha - 1.0).abs() < 1e-12);
        assert_eq!(cfg.k1, 0);
        assert_eq!(cfg.k2, 1);
        assert!((cfg.scale - 2.0).abs() < 1e-12);
        assert_eq!(cfg.projection_method, ProjectionMethod::Stereographic);
    }

    // ── Different k1/k2 selectors produce different surfaces ──────

    #[test]
    fn test_different_k1_k2_produce_different_surfaces() {
        let config_a = CalabiYauConfig {
            k1: 0,
            k2: 0,
            resolution: 5,
            ..CalabiYauConfig::default()
        };
        let config_b = CalabiYauConfig {
            k1: 1,
            k2: 2,
            resolution: 5,
            ..CalabiYauConfig::default()
        };
        let res_a = generate_manifold_mesh(&config_a);
        let res_b = generate_manifold_mesh(&config_b);
        assert!(res_a.is_ok(), "mesh_a failed: {:?}", res_a.err());
        assert!(res_b.is_ok(), "mesh_b failed: {:?}", res_b.err());
        let (mesh_a, mesh_b) = match (res_a, res_b) {
            (Ok(a), Ok(b)) => (a, b),
            _ => return,
        };
        // At least one vertex position should differ
        let differs = mesh_a
            .vertices
            .iter()
            .zip(mesh_b.vertices.iter())
            .any(|(va, vb)| (0..3).any(|i| (va.position[i] - vb.position[i]).abs() > 1e-10));
        assert!(differs, "k1/k2 selectors should produce different surfaces");
    }

    // ── ManifoldError Display ─────────────────────────────────────

    #[test]
    fn test_manifold_error_display_invalid_degree() {
        let msg = ManifoldError::InvalidDegree.to_string();
        assert!(msg.contains("degree"), "message: {msg}");
    }

    #[test]
    fn test_manifold_error_display_invalid_resolution() {
        let msg = ManifoldError::InvalidResolution.to_string();
        assert!(msg.contains("resolution"), "message: {msg}");
    }

    #[test]
    fn test_manifold_error_display_empty_mesh() {
        let msg = ManifoldError::EmptyMesh.to_string();
        assert!(
            msg.contains("empty") || msg.contains("mesh"),
            "message: {msg}"
        );
    }

    #[test]
    fn test_manifold_error_display_computation_failed() {
        let msg = ManifoldError::ComputationFailed("overflow".to_string()).to_string();
        assert!(msg.contains("overflow"), "message: {msg}");
    }

    // ── Math constants guard ──────────────────────────────────────

    #[test]
    fn test_math_constants_consistent() {
        // Ensure the parameter ranges are consistent with the constants
        let n: f64 = 5.0;
        let phi_range = TAU / n;
        assert!((phi_range - 2.0 * PI / n).abs() < 1e-12);
        assert!((FRAC_PI_2 - PI / 2.0).abs() < 1e-12);
    }
}
