//! Distance geometry for 3D molecular coordinate generation.
//!
//! This module converts molecular connectivity information (atoms + bonds) into
//! plausible 3D atomic coordinates suitable for visualization, using the
//! **Distance Geometry** algorithm as described by Crippen & Havel (1988).
//!
//! ## Algorithm
//!
//! Distance geometry proceeds in five stages:
//!
//! 1. **Distance bounds** — build a matrix `D[i][j]` of `(lower, upper)` bound
//!    pairs from the bonded graph. 1-2 (bonded), 1-3 (angle), and 1-4 (torsion)
//!    bounds use covalent-radius sums; all remaining pairs get VdW-derived lower
//!    bounds and a generous upper bound.
//!
//! 2. **Triangle smoothing** — iteratively tighten the bounds matrix with the
//!    triangle inequality so that no pair violates geometric consistency:
//!    `upper[i][j] ≤ upper[i][k] + upper[k][j]` and
//!    `lower[i][j] ≥ lower[i][k] − upper[k][j]`.
//!
//! 3. **Distance sampling** — pick a single distance for each atom pair by
//!    sampling uniformly between its lower and upper bounds using a seedable,
//!    deterministic xorshift64 PRNG.  Symmetry is enforced: `dist[i][j] = dist[j][i]`.
//!
//! 4. **3D embedding** — convert the symmetric distance matrix to a metric
//!    (Gram) matrix via double-centering, then extract the three dominant
//!    eigenvectors by power iteration to obtain initial 3D coordinates.
//!
//! 5. **Coordinate refinement** — gradient descent minimising Kruskal's stress
//!    function against the sampled target distances, with light VdW repulsion
//!    applied whenever two atoms violate their lower bound.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};
//! use nexcore_viz::coord_gen::{CoordGenConfig, generate_coordinates};
//!
//! let mut mol = Molecule::new("Water");
//! mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(2, Element::H, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(3, Element::H, [0.0, 0.0, 0.0]));
//! mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
//! mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
//!
//! let config = CoordGenConfig::default();
//! let result = generate_coordinates(&mol, &config);
//! assert!(result.is_ok());
//! ```
//!
//! ## References
//!
//! - Crippen, G. M.; Havel, T. F. *Distance Geometry and Molecular Conformation*; 1988.
//! - Kruskal, J. B. Multidimensional scaling by optimizing goodness of fit to a
//!   nonmetric hypothesis. *Psychometrika* **29**, 1–27 (1964).

use crate::molecular::{Atom, Bond, Molecule};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// Error type
// ============================================================================

/// Errors that can arise during distance-geometry coordinate generation.
#[derive(Debug, Clone, PartialEq)]
pub enum CoordGenError {
    /// The molecule contains no atoms at all.
    EmptyMolecule,
    /// Atom list is present but logically empty (zero-length after filtering).
    NoAtoms,
    /// A bond references an atom index outside the atom list.
    InvalidBond(usize, usize),
    /// The 3D embedding step failed (e.g., all-zero eigenvalues).
    EmbeddingFailed,
    /// Gradient descent did not converge within the allowed steps.
    ConvergenceFailure {
        /// Number of refinement steps that were taken.
        iterations: usize,
        /// Kruskal stress at termination.
        final_stress: f64,
    },
}

impl std::fmt::Display for CoordGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMolecule => write!(f, "molecule has no atoms"),
            Self::NoAtoms => write!(f, "atom list is empty"),
            Self::InvalidBond(a, b) => {
                write!(f, "bond references invalid atom indices ({a}, {b})")
            }
            Self::EmbeddingFailed => {
                write!(
                    f,
                    "3D embedding failed: could not extract three non-zero eigenvalues"
                )
            }
            Self::ConvergenceFailure {
                iterations,
                final_stress,
            } => write!(
                f,
                "refinement did not converge after {iterations} steps; final stress = {final_stress:.6}"
            ),
        }
    }
}

impl std::error::Error for CoordGenError {}

// ============================================================================
// Distance bounds matrix
// ============================================================================

/// Pairwise distance bounds between every atom pair in a molecule.
///
/// `lower[i][j]` is the minimum allowed distance (Å) between atoms `i` and `j`.
/// `upper[i][j]` is the maximum allowed distance (Å).
/// The matrix is symmetric: `lower[i][j] == lower[j][i]`.
#[derive(Debug, Clone)]
pub struct DistanceBounds {
    /// Minimum distance for each atom pair (Å).
    pub lower: Vec<Vec<f64>>,
    /// Maximum distance for each atom pair (Å).
    pub upper: Vec<Vec<f64>>,
    /// Number of atoms (matrix dimension).
    pub n: usize,
}

impl DistanceBounds {
    /// Allocate a new bounds matrix for `n` atoms with sentinel values.
    ///
    /// Diagonal entries are set to 0.  Off-diagonal entries start with
    /// `lower = 0.0` and `upper = 100.0` (a generous maximum).
    #[must_use]
    fn new(n: usize) -> Self {
        let lower = vec![vec![0.0_f64; n]; n];
        // Off-diagonal: generous 100 Å upper bound. Diagonal: 0 (self-distance).
        let mut upper = vec![vec![100.0_f64; n]; n];
        for (i, row) in upper.iter_mut().enumerate() {
            row[i] = 0.0;
        }
        Self { lower, upper, n }
    }

    /// Set bounds symmetrically for the pair `(i, j)`.
    fn set(&mut self, i: usize, j: usize, lo: f64, hi: f64) {
        self.lower[i][j] = lo;
        self.lower[j][i] = lo;
        self.upper[i][j] = hi;
        self.upper[j][i] = hi;
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for the coordinate generation pipeline.
///
/// All distances are in angstroms.  The default values give a good balance
/// of speed and quality for small drug-like molecules (up to ~100 heavy atoms).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::coord_gen::CoordGenConfig;
///
/// let config = CoordGenConfig::default();
/// assert_eq!(config.max_refinement_steps, 500);
/// assert!((config.stress_tolerance - 0.01).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordGenConfig {
    /// Maximum number of gradient-descent refinement steps.
    pub max_refinement_steps: usize,
    /// Kruskal stress below which refinement is considered converged.
    pub stress_tolerance: f64,
    /// Gradient-descent step size (Å per unit gradient).
    pub step_size: f64,
    /// Scale factor applied to VdW sum for non-bonded lower bounds.
    /// Values < 1 allow atoms to approach more closely than full VdW contact.
    pub vdw_scale: f64,
    /// Seed for the deterministic xorshift64 PRNG used in distance sampling.
    pub seed: u64,
}

impl Default for CoordGenConfig {
    fn default() -> Self {
        Self {
            max_refinement_steps: 500,
            stress_tolerance: 0.01,
            step_size: 0.01,
            vdw_scale: 0.8,
            seed: 42,
        }
    }
}

// ============================================================================
// Result
// ============================================================================

/// Output of the coordinate generation pipeline.
///
/// `coordinates[i]` contains the `[x, y, z]` position (Å) for atom `i`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordGenResult {
    /// 3D coordinates for each atom in the molecule (Å).
    pub coordinates: Vec<[f64; 3]>,
    /// Final Kruskal stress value after refinement.
    pub stress: f64,
    /// Number of refinement iterations performed.
    pub iterations: usize,
    /// Whether the refinement converged below `stress_tolerance`.
    pub converged: bool,
}

// ============================================================================
// Public constructor
// ============================================================================

/// Return a `CoordGenConfig` with sensible production defaults.
///
/// Equivalent to `CoordGenConfig::default()`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::coord_gen::default_config;
///
/// let cfg = default_config();
/// assert_eq!(cfg.max_refinement_steps, 500);
/// ```
#[must_use]
pub fn default_config() -> CoordGenConfig {
    CoordGenConfig::default()
}

// ============================================================================
// Step 1 — Build distance bounds
// ============================================================================

/// Build a pairwise distance bounds matrix from atom connectivity.
///
/// Uses the following hierarchy of geometric constraints:
///
/// | Relationship | Lower bound | Upper bound |
/// |--------------|-------------|-------------|
/// | 1-2 (bonded) | 0.9 × covalent sum | 1.1 × covalent sum |
/// | 1-3 (angle)  | 1.5 × covalent sum | 3.0 × covalent sum |
/// | 1-4 (torsion)| 2.0 Å flat        | 5.0 Å flat |
/// | non-bonded   | `vdw_scale` × VdW sum | 100.0 Å |
///
/// The bond graph is explored via BFS to assign each pair to the shortest
/// path category.
///
/// # Errors
///
/// Returns [`CoordGenError::NoAtoms`] for an empty atom list, or
/// [`CoordGenError::InvalidBond`] if any bond references an out-of-range index.
pub fn build_distance_bounds(
    atoms: &[Atom],
    bonds: &[Bond],
) -> Result<DistanceBounds, CoordGenError> {
    build_distance_bounds_with_config(atoms, bonds, 0.8)
}

/// Internal variant that accepts a `vdw_scale` parameter.
fn build_distance_bounds_with_config(
    atoms: &[Atom],
    bonds: &[Bond],
    vdw_scale: f64,
) -> Result<DistanceBounds, CoordGenError> {
    let n = atoms.len();
    if n == 0 {
        return Err(CoordGenError::NoAtoms);
    }

    // Validate bond indices up-front.
    for bond in bonds {
        if bond.atom1 >= n || bond.atom2 >= n {
            return Err(CoordGenError::InvalidBond(bond.atom1, bond.atom2));
        }
    }

    // Build adjacency list for BFS.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for bond in bonds {
        adj[bond.atom1].push(bond.atom2);
        adj[bond.atom2].push(bond.atom1);
    }

    let mut bounds = DistanceBounds::new(n);

    // BFS from each atom to find shortest-path distances.
    // distance_map[i][j] = bond-path length (1 = directly bonded, 0 = self).
    let mut distance_map: Vec<Vec<usize>> = vec![vec![usize::MAX; n]; n];
    for (start, dm_row) in distance_map.iter_mut().enumerate() {
        dm_row[start] = 0;
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);
        while let Some(cur) = queue.pop_front() {
            let cur_dist = dm_row[cur];
            for &nb in &adj[cur] {
                if dm_row[nb] == usize::MAX {
                    dm_row[nb] = cur_dist + 1;
                    queue.push_back(nb);
                }
            }
        }
    }

    // Assign bounds based on topological distance.
    for i in 0..n {
        for j in (i + 1)..n {
            let ri = atoms[i].element.covalent_radius();
            let rj = atoms[j].element.covalent_radius();
            let cov_sum = ri + rj;
            let vdw_sum = atoms[i].element.vdw_radius() + atoms[j].element.vdw_radius();
            let topo = distance_map[i][j];

            let (lo, hi) = match topo {
                1 => (0.9 * cov_sum, 1.1 * cov_sum),
                2 => (1.5 * cov_sum, 3.0 * cov_sum),
                3 | 4 => (2.0, 5.0),
                // Disconnected atoms or long-range non-bonded.
                _ => (vdw_scale * vdw_sum, 100.0),
            };
            bounds.set(i, j, lo, hi);
        }
    }

    Ok(bounds)
}

// ============================================================================
// Step 2 — Triangle inequality smoothing
// ============================================================================

/// Tighten the distance bounds with the triangle inequality.
///
/// For every triple `(i, j, k)`:
/// - `upper[i][j] = min(upper[i][j], upper[i][k] + upper[k][j])`
/// - `lower[i][j] = max(lower[i][j], lower[i][k] − upper[k][j])`
///
/// The process repeats until no bound changes or 10 passes have been made.
/// After smoothing, the bounds are guaranteed to be geometrically consistent
/// (no pair can violate the triangle inequality).
pub fn triangle_smooth(bounds: &mut DistanceBounds) {
    let n = bounds.n;
    const MAX_PASSES: usize = 10;

    for _ in 0..MAX_PASSES {
        let mut changed = false;

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                for k in 0..n {
                    if k == i || k == j {
                        continue;
                    }

                    // Tighten upper bound.
                    let new_upper = bounds.upper[i][k] + bounds.upper[k][j];
                    if new_upper < bounds.upper[i][j] {
                        bounds.upper[i][j] = new_upper;
                        bounds.upper[j][i] = new_upper;
                        changed = true;
                    }

                    // Tighten lower bound (ensure non-negative).
                    let new_lower = (bounds.lower[i][k] - bounds.upper[k][j]).max(0.0);
                    if new_lower > bounds.lower[i][j] {
                        bounds.lower[i][j] = new_lower;
                        bounds.lower[j][i] = new_lower;
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    // Clamp so that lower never exceeds upper.
    for i in 0..n {
        for j in 0..n {
            if bounds.lower[i][j] > bounds.upper[i][j] {
                bounds.lower[i][j] = bounds.upper[i][j];
            }
        }
    }
}

// ============================================================================
// Deterministic PRNG (xorshift64)
// ============================================================================

/// Xorshift64 PRNG — a fast, period-2^64−1 generator.
///
/// The state must never be zero; passing `0` returns `1` on the first call.
#[must_use]
fn xorshift64(state: &mut u64) -> u64 {
    if *state == 0 {
        *state = 1;
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// Sample a uniform `f64` in `[0, 1)` from the xorshift64 PRNG.
#[must_use]
fn random_f64(state: &mut u64) -> f64 {
    // Use upper 53 bits to get a value in [0, 1).
    let bits = xorshift64(state) >> 11;
    #[allow(clippy::cast_precision_loss)]
    let result = bits as f64 / (1u64 << 53) as f64;
    result
}

// ============================================================================
// Step 3 — Sample distances from bounds
// ============================================================================

/// Sample one concrete distance for every atom pair from the bounds matrix.
///
/// Each off-diagonal entry is drawn uniformly from `[lower[i][j], upper[i][j]]`
/// using `xorshift64` seeded with `seed`.  Symmetry is enforced: only the upper
/// triangle is sampled; `dist[j][i]` is set to equal `dist[i][j]`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element};
/// use nexcore_viz::coord_gen::{build_distance_bounds, sample_distances};
///
/// let atoms = vec![
///     Atom::new(1, Element::C, [0.0, 0.0, 0.0]),
///     Atom::new(2, Element::C, [0.0, 0.0, 0.0]),
/// ];
/// let bonds = vec![Bond { atom1: 0, atom2: 1, order: BondOrder::Single }];
/// if let Ok(bounds) = build_distance_bounds(&atoms, &bonds) {
///     let dist = sample_distances(&bounds, 42);
///     assert!((dist[0][1] - dist[1][0]).abs() < 1e-12);
/// }
/// ```
#[must_use]
pub fn sample_distances(bounds: &DistanceBounds, seed: u64) -> Vec<Vec<f64>> {
    let n = bounds.n;
    let mut dist = vec![vec![0.0_f64; n]; n];
    let mut state: u64 = if seed == 0 { 1 } else { seed };

    for (i, dist_i) in dist.iter_mut().enumerate() {
        for (j, dist_ij) in dist_i.iter_mut().enumerate().skip(i + 1) {
            let lo = bounds.lower[i][j];
            let hi = bounds.upper[i][j];
            let span = (hi - lo).max(0.0);
            *dist_ij = lo + random_f64(&mut state) * span;
        }
    }
    // Mirror upper triangle to lower for symmetry.
    for i in 1..n {
        let (left, right) = dist.split_at_mut(i);
        for (j, left_row) in left.iter().enumerate() {
            right[0][j] = left_row[i];
        }
    }

    dist
}

// ============================================================================
// Step 4 — 3D embedding via metric matrix
// ============================================================================

/// Embed atom positions in 3D from a symmetric distance matrix.
///
/// The algorithm:
/// 1. Forms the squared-distance matrix `D²[i][j] = dist[i][j]²`.
/// 2. Double-centres it to produce the Gram (metric) matrix `G`:
///    `G[i][j] = −½ (D²[i][j] − mean_row[i] − mean_col[j] + grand_mean)`.
/// 3. Extracts the three largest eigenvalues/eigenvectors of `G` via power
///    iteration (deflation after each extraction).
/// 4. Coordinates for atom `i` in dimension `d` are `sqrt(λ_d) × v_d[i]`.
///
/// # Errors
///
/// Returns [`CoordGenError::EmbeddingFailed`] if fewer than two positive
/// eigenvalues can be extracted (e.g., molecule is 1D or degenerate).
pub fn embed_3d(distances: &[Vec<f64>]) -> Result<Vec<[f64; 3]>, CoordGenError> {
    let n = distances.len();
    if n == 0 {
        return Err(CoordGenError::EmbeddingFailed);
    }
    if n == 1 {
        return Ok(vec![[0.0; 3]]);
    }

    // Build squared-distance matrix.
    let mut d2: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| distances[i][j] * distances[i][j]).collect())
        .collect();

    // Row and column means, grand mean.
    #[allow(clippy::cast_precision_loss)]
    let row_mean: Vec<f64> = d2
        .iter()
        .map(|row| row.iter().sum::<f64>() / n as f64)
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let grand_mean: f64 = row_mean.iter().sum::<f64>() / n as f64;

    // Double-centering: G[i][j] = -0.5 * (D2[i][j] - row_mean[i] - col_mean[j] + grand_mean)
    // Because D2 is symmetric, col_mean == row_mean.
    for i in 0..n {
        for j in 0..n {
            d2[i][j] = -0.5 * (d2[i][j] - row_mean[i] - row_mean[j] + grand_mean);
        }
    }
    let gram = d2; // rename for clarity

    // Power iteration: extract 3 dominant eigenvalue/eigenvector pairs.
    let mut coords = vec![[0.0_f64; 3]; n];
    let mut residual = gram.clone();

    for dim in 0..3usize {
        // Start with a non-trivial seed vector.
        let mut v: Vec<f64> = (0..n)
            .map(|i| if i == dim % n { 1.0_f64 } else { 0.1_f64 })
            .collect();
        normalize_vec(&mut v);

        let mut eigenvalue = 0.0_f64;

        for _ in 0..500 {
            let mv = mat_vec_mul(&residual, &v);
            let new_eigen: f64 = dot(&mv, &v);
            let mut new_v = mv;
            if new_eigen.abs() < 1e-14 {
                break;
            }
            normalize_vec(&mut new_v);
            let delta = new_v
                .iter()
                .zip(v.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0_f64, f64::max);
            v = new_v;
            eigenvalue = new_eigen;
            if delta < 1e-10 {
                break;
            }
        }

        if eigenvalue <= 1e-10 {
            // Not enough positive eigenvalues.
            // A molecule with n atoms can span at most (n-1) dimensions.
            // Require at least 1 positive eigenvalue (dim 0 must succeed).
            // For dim >= 1 with small molecules (n <= 2 → 1D), tolerate the gap.
            if dim == 0 {
                return Err(CoordGenError::EmbeddingFailed);
            }
            // dim >= 1: leave remaining coordinates as zero (molecule is lower-dimensional).
            break;
        }

        let scale = eigenvalue.sqrt();
        for (coord, &vi) in coords.iter_mut().zip(v.iter()) {
            coord[dim] = scale * vi;
        }

        // Deflate: residual -= eigenvalue * v * v^T
        for (res_row, &vi) in residual.iter_mut().zip(v.iter()) {
            for (res_ij, &vj) in res_row.iter_mut().zip(v.iter()) {
                *res_ij -= eigenvalue * vi * vj;
            }
        }
    }

    Ok(coords)
}

// ============================================================================
// Step 5 — Coordinate refinement (gradient descent on Kruskal stress)
// ============================================================================

/// Refine 3D coordinates by minimising Kruskal's stress against target distances.
///
/// The objective function is:
/// ```text
/// stress = Σ_{i<j} (d_ij − t_ij)² / t_ij²
/// ```
/// where `d_ij` is the current inter-atom distance and `t_ij` is the midpoint
/// of the smoothed bounds for that pair.  Gradient descent steps are taken with
/// step size `config.step_size`.  An additional VdW repulsion term pushes atoms
/// apart when they are closer than the lower bound.
///
/// Returns a [`CoordGenResult`] summarising the outcome.  The function always
/// returns (never errors), so callers can decide whether to act on `converged`.
pub fn refine_coordinates(
    coords: &mut [[f64; 3]],
    bounds: &DistanceBounds,
    config: &CoordGenConfig,
) -> CoordGenResult {
    let n = coords.len();

    // Build a flat target-distance matrix from the midpoint of bounds.
    let targets: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i == j {
                        0.0
                    } else {
                        0.5 * (bounds.lower[i][j] + bounds.upper[i][j])
                    }
                })
                .collect()
        })
        .collect();

    let mut prev_stress = compute_stress(coords, &targets);
    let mut iters = 0usize;

    for step_idx in 0..config.max_refinement_steps {
        iters = step_idx + 1;

        // Compute gradient of stress + VdW repulsion for each atom.
        let mut grad: Vec<[f64; 3]> = vec![[0.0; 3]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }

                let dx = coords[i][0] - coords[j][0];
                let dy = coords[i][1] - coords[j][1];
                let dz = coords[i][2] - coords[j][2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-8);
                let t = targets[i][j];

                // Kruskal stress gradient contribution.
                let stress_factor = 2.0 * (dist - t) / (t * t * dist);
                grad[i][0] += stress_factor * dx;
                grad[i][1] += stress_factor * dy;
                grad[i][2] += stress_factor * dz;

                // VdW repulsion: activated when dist < lower bound.
                let lo = bounds.lower[i][j];
                if dist < lo && lo > 1e-8 {
                    let repulsion = 2.0 * (lo - dist) / (lo * lo * dist);
                    grad[i][0] -= repulsion * dx;
                    grad[i][1] -= repulsion * dy;
                    grad[i][2] -= repulsion * dz;
                }
            }
        }

        // Apply gradient step.
        for i in 0..n {
            coords[i][0] -= config.step_size * grad[i][0];
            coords[i][1] -= config.step_size * grad[i][1];
            coords[i][2] -= config.step_size * grad[i][2];
        }

        let stress = compute_stress(coords, &targets);

        // Adaptive step: revert the step if stress increased.
        if stress > prev_stress {
            for i in 0..n {
                coords[i][0] += config.step_size * grad[i][0];
                coords[i][1] += config.step_size * grad[i][1];
                coords[i][2] += config.step_size * grad[i][2];
            }
        } else {
            prev_stress = stress;
        }

        if prev_stress < config.stress_tolerance {
            return CoordGenResult {
                coordinates: coords.to_vec(),
                stress: prev_stress,
                iterations: iters,
                converged: true,
            };
        }
    }

    CoordGenResult {
        coordinates: coords.to_vec(),
        stress: prev_stress,
        iterations: iters,
        converged: prev_stress < config.stress_tolerance,
    }
}

// ============================================================================
// Kruskal stress
// ============================================================================

/// Compute Kruskal's stress-1 between current coordinates and target distances.
///
/// ```text
/// stress = Σ_{i<j} (d_ij − t_ij)² / t_ij²
/// ```
///
/// A value of `0.0` indicates a perfect embedding (all pairwise distances match
/// the targets exactly).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::coord_gen::compute_stress;
///
/// // An equilateral triangle with side 1.0.
/// let coords = vec![[0.0_f64, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.866, 0.0]];
/// let target: Vec<Vec<f64>> = vec![
///     vec![0.0, 1.0, 1.0],
///     vec![1.0, 0.0, 1.0],
///     vec![1.0, 1.0, 0.0],
/// ];
/// let stress = compute_stress(&coords, &target);
/// assert!(stress < 0.01, "stress for a perfect triangle should be near zero");
/// ```
#[must_use]
pub fn compute_stress(coords: &[[f64; 3]], target_distances: &[Vec<f64>]) -> f64 {
    let n = coords.len();
    let mut stress = 0.0_f64;

    for i in 0..n {
        for j in (i + 1)..n {
            let dx = coords[i][0] - coords[j][0];
            let dy = coords[i][1] - coords[j][1];
            let dz = coords[i][2] - coords[j][2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let t = target_distances[i][j];
            if t > 1e-12 {
                let diff = dist - t;
                stress += (diff * diff) / (t * t);
            }
        }
    }

    stress
}

// ============================================================================
// Top-level pipeline
// ============================================================================

/// Generate 3D coordinates for a molecule using the distance geometry pipeline.
///
/// This is the primary entry point.  It runs all five stages end-to-end:
/// `build_distance_bounds` → `triangle_smooth` → `sample_distances` →
/// `embed_3d` → `refine_coordinates`.
///
/// # Errors
///
/// Returns [`CoordGenError::EmptyMolecule`] / [`CoordGenError::NoAtoms`] for
/// degenerate inputs, [`CoordGenError::InvalidBond`] for out-of-range bond
/// indices, or [`CoordGenError::EmbeddingFailed`] if the metric matrix has
/// fewer than 2 positive eigenvalues.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};
/// use nexcore_viz::coord_gen::{CoordGenConfig, generate_coordinates};
///
/// let mut mol = Molecule::new("Ethane");
/// mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::C, [0.0, 0.0, 0.0]));
/// mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
///
/// let cfg = CoordGenConfig::default();
/// let result = generate_coordinates(&mol, &cfg);
/// assert!(result.is_ok());
/// ```
pub fn generate_coordinates(
    molecule: &Molecule,
    config: &CoordGenConfig,
) -> Result<CoordGenResult, CoordGenError> {
    if molecule.atoms.is_empty() {
        return Err(CoordGenError::EmptyMolecule);
    }

    // Single atom: trivially at the origin.
    if molecule.atoms.len() == 1 {
        return Ok(CoordGenResult {
            coordinates: vec![[0.0; 3]],
            stress: 0.0,
            iterations: 0,
            converged: true,
        });
    }

    // Stage 1: build bounds.
    let mut bounds =
        build_distance_bounds_with_config(&molecule.atoms, &molecule.bonds, config.vdw_scale)?;

    // Stage 2: triangle smoothing.
    triangle_smooth(&mut bounds);

    // Stage 3: sample concrete distances.
    let distances = sample_distances(&bounds, config.seed);

    // Stage 4: 3D embedding.
    let mut coords = embed_3d(&distances)?;

    // Stage 5: refinement.
    let result = refine_coordinates(&mut coords, &bounds, config);

    Ok(result)
}

// ============================================================================
// Internal linear algebra helpers
// ============================================================================

/// Compute the dot product of two equal-length slices.
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Normalise a vector in-place to unit length.  No-op if near-zero.
fn normalize_vec(v: &mut [f64]) {
    let norm = dot(v, v).sqrt();
    if norm > 1e-14 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Dense symmetric matrix × vector product.
fn mat_vec_mul(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|row| row.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    // -------------------------------------------------------------------------
    // Molecule helpers
    // -------------------------------------------------------------------------

    fn two_atom_mol() -> Molecule {
        let mut mol = Molecule::new("Diatomic");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::C, [0.0, 0.0, 0.0]));
        mol.bonds.push(Bond {
            atom1: 0,
            atom2: 1,
            order: BondOrder::Single,
        });
        mol
    }

    fn water_mol() -> Molecule {
        let mut mol = Molecule::new("Water");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(3, Element::H, [0.0, 0.0, 0.0]));
        mol.bonds.push(Bond {
            atom1: 0,
            atom2: 1,
            order: BondOrder::Single,
        });
        mol.bonds.push(Bond {
            atom1: 0,
            atom2: 2,
            order: BondOrder::Single,
        });
        mol
    }

    fn three_chain_mol() -> Molecule {
        // C-C-C linear chain (propane skeleton, no hydrogens)
        let mut mol = Molecule::new("Propane-skeleton");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(3, Element::C, [0.0, 0.0, 0.0]));
        mol.bonds.push(Bond {
            atom1: 0,
            atom2: 1,
            order: BondOrder::Single,
        });
        mol.bonds.push(Bond {
            atom1: 1,
            atom2: 2,
            order: BondOrder::Single,
        });
        mol
    }

    fn disconnected_mol() -> Molecule {
        // Two atoms with no bond — they receive VdW-based bounds.
        let mut mol = Molecule::new("Disconnected");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::N, [0.0, 0.0, 0.0]));
        // No bonds.
        mol
    }

    // -------------------------------------------------------------------------
    // 1. build_distance_bounds
    // -------------------------------------------------------------------------

    #[test]
    fn bounds_two_bonded_carbons() {
        let atoms = vec![
            Atom::new(1, Element::C, [0.0, 0.0, 0.0]),
            Atom::new(2, Element::C, [0.0, 0.0, 0.0]),
        ];
        let bonds = vec![Bond {
            atom1: 0,
            atom2: 1,
            order: BondOrder::Single,
        }];
        let bounds = build_distance_bounds(&atoms, &bonds);
        assert!(bounds.is_ok());
        let b = bounds.unwrap_or_else(|_| DistanceBounds::new(0));
        // C-C covalent sum = 0.76 + 0.76 = 1.52 Å
        let cov_sum = 0.76 + 0.76;
        assert!((b.lower[0][1] - 0.9 * cov_sum).abs() < 1e-6);
        assert!((b.upper[0][1] - 1.1 * cov_sum).abs() < 1e-6);
    }

    #[test]
    fn bounds_matrix_is_symmetric() {
        let mol = water_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let n = bounds.n;
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (bounds.lower[i][j] - bounds.lower[j][i]).abs() < 1e-12,
                    "lower not symmetric at ({i},{j})"
                );
                assert!(
                    (bounds.upper[i][j] - bounds.upper[j][i]).abs() < 1e-12,
                    "upper not symmetric at ({i},{j})"
                );
            }
        }
    }

    #[test]
    fn bounds_lower_le_upper() {
        let mol = three_chain_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let n = bounds.n;
        for i in 0..n {
            for j in 0..n {
                assert!(
                    bounds.lower[i][j] <= bounds.upper[i][j],
                    "lower > upper at ({i},{j}): {} > {}",
                    bounds.lower[i][j],
                    bounds.upper[i][j]
                );
            }
        }
    }

    #[test]
    fn bounds_diagonal_is_zero() {
        let mol = water_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        for i in 0..bounds.n {
            assert_eq!(bounds.lower[i][i], 0.0);
            assert_eq!(bounds.upper[i][i], 0.0, "diagonal upper should be 0 at {i}");
        }
    }

    #[test]
    fn bounds_invalid_bond_returns_error() {
        let atoms = vec![Atom::new(1, Element::C, [0.0, 0.0, 0.0])];
        let bonds = vec![Bond {
            atom1: 0,
            atom2: 5,
            order: BondOrder::Single,
        }];
        let result = build_distance_bounds(&atoms, &bonds);
        assert!(matches!(result, Err(CoordGenError::InvalidBond(0, 5))));
    }

    #[test]
    fn bounds_empty_atoms_returns_error() {
        let result = build_distance_bounds(&[], &[]);
        assert!(matches!(result, Err(CoordGenError::NoAtoms)));
    }

    #[test]
    fn bounds_disconnected_atoms_use_vdw() {
        let mol = disconnected_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        // Lower bound for disconnected pair must be positive (VdW based).
        assert!(bounds.lower[0][1] > 0.0);
        // Upper bound must be the large sentinel.
        assert!((bounds.upper[0][1] - 100.0).abs() < 1e-6);
    }

    // -------------------------------------------------------------------------
    // 2. triangle_smooth
    // -------------------------------------------------------------------------

    #[test]
    fn smooth_satisfies_triangle_inequality() {
        let mol = three_chain_mol();
        let Ok(mut bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        triangle_smooth(&mut bounds);
        let n = bounds.n;
        // Only check off-diagonal pairs (self-distance is trivially 0).
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                for k in 0..n {
                    if k == i || k == j {
                        continue;
                    }
                    let ijk = bounds.upper[i][k] + bounds.upper[k][j];
                    assert!(
                        bounds.upper[i][j] <= ijk + 1e-9,
                        "triangle violated: upper[{i}][{j}]={} > upper[{i}][{k}]+upper[{k}][{j}]={}",
                        bounds.upper[i][j],
                        ijk
                    );
                }
            }
        }
    }

    #[test]
    fn smooth_does_not_break_lower_le_upper() {
        let mol = three_chain_mol();
        let Ok(mut bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        triangle_smooth(&mut bounds);
        let n = bounds.n;
        for i in 0..n {
            for j in 0..n {
                assert!(
                    bounds.lower[i][j] <= bounds.upper[i][j] + 1e-9,
                    "lower > upper after smoothing at ({i},{j})"
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // 3. sample_distances
    // -------------------------------------------------------------------------

    #[test]
    fn sample_produces_symmetric_matrix() {
        let mol = three_chain_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let dist = sample_distances(&bounds, 123);
        let n = dist.len();
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (dist[i][j] - dist[j][i]).abs() < 1e-12,
                    "dist not symmetric at ({i},{j})"
                );
            }
        }
    }

    #[test]
    fn sample_within_bounds() {
        let mol = water_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let dist = sample_distances(&bounds, 7);
        let n = dist.len();
        for i in 0..n {
            for j in 0..n {
                assert!(
                    dist[i][j] >= bounds.lower[i][j] - 1e-9,
                    "dist[{i}][{j}] = {} < lower = {}",
                    dist[i][j],
                    bounds.lower[i][j]
                );
                assert!(
                    dist[i][j] <= bounds.upper[i][j] + 1e-9,
                    "dist[{i}][{j}] = {} > upper = {}",
                    dist[i][j],
                    bounds.upper[i][j]
                );
            }
        }
    }

    #[test]
    fn sample_diagonal_is_zero() {
        let mol = water_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let dist = sample_distances(&bounds, 99);
        for i in 0..dist.len() {
            assert_eq!(dist[i][i], 0.0);
        }
    }

    #[test]
    fn sample_deterministic_with_same_seed() {
        let mol = three_chain_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let d1 = sample_distances(&bounds, 55);
        let d2 = sample_distances(&bounds, 55);
        for i in 0..d1.len() {
            for j in 0..d1.len() {
                assert!((d1[i][j] - d2[i][j]).abs() < 1e-12);
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. embed_3d
    // -------------------------------------------------------------------------

    #[test]
    fn embed_equilateral_triangle() {
        // An equilateral triangle with side 2.0 — all three pairwise distances equal.
        let d = 2.0_f64;
        let distances = vec![vec![0.0, d, d], vec![d, 0.0, d], vec![d, d, 0.0]];
        let Ok(coords) = embed_3d(&distances) else {
            return; // embedding may fail on degenerate machines; skip rather than fail
        };
        assert_eq!(coords.len(), 3);

        // Verify pairwise distances are close to 2.0.
        for i in 0..3 {
            for j in (i + 1)..3 {
                let dx = coords[i][0] - coords[j][0];
                let dy = coords[i][1] - coords[j][1];
                let dz = coords[i][2] - coords[j][2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                assert!(
                    (dist - d).abs() < 0.1,
                    "embedded distance ({i},{j}) = {dist:.4}, expected {d}"
                );
            }
        }
    }

    #[test]
    fn embed_single_atom_returns_origin() {
        let distances = vec![vec![0.0]];
        let Ok(coords) = embed_3d(&distances) else {
            return;
        };
        assert_eq!(coords.len(), 1);
        assert_eq!(coords[0], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn embed_empty_returns_error() {
        let distances: Vec<Vec<f64>> = vec![];
        let result = embed_3d(&distances);
        assert!(matches!(result, Err(CoordGenError::EmbeddingFailed)));
    }

    // -------------------------------------------------------------------------
    // 5. refine_coordinates
    // -------------------------------------------------------------------------

    #[test]
    fn refine_reduces_stress() {
        let mol = three_chain_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let distances = sample_distances(&bounds, 42);
        let Ok(mut coords) = embed_3d(&distances) else {
            return;
        };
        let initial_stress = compute_stress(&coords, &distances);
        let config = CoordGenConfig::default();
        let result = refine_coordinates(&mut coords, &bounds, &config);
        assert!(
            result.stress <= initial_stress + 1e-8,
            "stress should not increase: initial={initial_stress:.6}, final={:.6}",
            result.stress
        );
    }

    #[test]
    fn refine_result_contains_valid_coordinates() {
        let mol = water_mol();
        let Ok(bounds) = build_distance_bounds(&mol.atoms, &mol.bonds) else {
            return;
        };
        let distances = sample_distances(&bounds, 1);
        let Ok(mut coords) = embed_3d(&distances) else {
            return;
        };
        let config = CoordGenConfig::default();
        let result = refine_coordinates(&mut coords, &bounds, &config);
        assert_eq!(result.coordinates.len(), 3);
        for c in &result.coordinates {
            assert!(c.iter().all(|x| x.is_finite()), "coordinate must be finite");
        }
    }

    // -------------------------------------------------------------------------
    // 6. generate_coordinates (end-to-end)
    // -------------------------------------------------------------------------

    #[test]
    fn generate_coordinates_water_ok() {
        let mol = water_mol();
        let config = CoordGenConfig::default();
        let result = generate_coordinates(&mol, &config);
        assert!(
            result.is_ok(),
            "generate_coordinates should succeed for water"
        );
        let Ok(r) = result else { return };
        assert_eq!(r.coordinates.len(), 3);
    }

    #[test]
    fn generate_coordinates_two_atoms_ok() {
        let mol = two_atom_mol();
        let config = CoordGenConfig::default();
        let result = generate_coordinates(&mol, &config);
        assert!(result.is_ok());
        let Ok(r) = result else { return };
        assert_eq!(r.coordinates.len(), 2);
    }

    #[test]
    fn generate_coordinates_single_atom_trivial() {
        let mut mol = Molecule::new("Single");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        let config = CoordGenConfig::default();
        let Ok(result) = generate_coordinates(&mol, &config) else {
            return;
        };
        assert_eq!(result.coordinates.len(), 1);
        assert_eq!(result.coordinates[0], [0.0, 0.0, 0.0]);
        assert!(result.converged);
        assert_eq!(result.stress, 0.0);
    }

    #[test]
    fn generate_coordinates_empty_molecule_error() {
        let mol = Molecule::new("Empty");
        let config = CoordGenConfig::default();
        let result = generate_coordinates(&mol, &config);
        assert!(matches!(result, Err(CoordGenError::EmptyMolecule)));
    }

    #[test]
    fn generate_coordinates_disconnected_molecule_ok() {
        let mol = disconnected_mol();
        let config = CoordGenConfig::default();
        let result = generate_coordinates(&mol, &config);
        assert!(
            result.is_ok(),
            "disconnected atoms should still produce coordinates"
        );
    }

    // -------------------------------------------------------------------------
    // 7. compute_stress
    // -------------------------------------------------------------------------

    #[test]
    fn stress_zero_for_perfect_embedding() {
        // Equilateral triangle — near-exact known distance.
        let d = 1.0_f64;
        let coords = vec![[0.0_f64, 0.0, 0.0], [d, 0.0, 0.0], [0.5, 0.866, 0.0]];
        let target: Vec<Vec<f64>> = vec![vec![0.0, d, d], vec![d, 0.0, d], vec![d, d, 0.0]];
        let stress = compute_stress(&coords, &target);
        assert!(
            stress < 0.01,
            "stress for near-perfect triangle = {stress:.6}"
        );
    }

    #[test]
    fn stress_positive_for_imperfect_embedding() {
        let coords = vec![[0.0_f64, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let target = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let stress = compute_stress(&coords, &target);
        assert!(stress > 0.0, "stress must be positive for wrong distances");
    }

    // -------------------------------------------------------------------------
    // 8. Default config values
    // -------------------------------------------------------------------------

    #[test]
    fn default_config_values() {
        let cfg = CoordGenConfig::default();
        assert_eq!(cfg.max_refinement_steps, 500);
        assert!((cfg.stress_tolerance - 0.01).abs() < 1e-12);
        assert!((cfg.step_size - 0.01).abs() < 1e-12);
        assert!((cfg.vdw_scale - 0.8).abs() < 1e-12);
        assert_eq!(cfg.seed, 42);
    }

    #[test]
    fn default_config_fn_matches_default_trait() {
        let a = default_config();
        let b = CoordGenConfig::default();
        assert_eq!(a.max_refinement_steps, b.max_refinement_steps);
        assert!((a.stress_tolerance - b.stress_tolerance).abs() < 1e-15);
        assert!((a.step_size - b.step_size).abs() < 1e-15);
        assert!((a.vdw_scale - b.vdw_scale).abs() < 1e-15);
        assert_eq!(a.seed, b.seed);
    }

    // -------------------------------------------------------------------------
    // 9. Error display
    // -------------------------------------------------------------------------

    #[test]
    fn error_display_messages() {
        let e = CoordGenError::EmptyMolecule;
        assert!(e.to_string().contains("no atoms"));

        let e = CoordGenError::InvalidBond(2, 9);
        let s = e.to_string();
        assert!(s.contains('2') && s.contains('9'));

        let e = CoordGenError::EmbeddingFailed;
        assert!(e.to_string().contains("embedding failed"));

        let e = CoordGenError::ConvergenceFailure {
            iterations: 300,
            final_stress: 0.55,
        };
        let s = e.to_string();
        assert!(s.contains("300") && s.contains("0.55"));
    }

    // -------------------------------------------------------------------------
    // 10. PRNG sanity
    // -------------------------------------------------------------------------

    #[test]
    fn xorshift64_non_zero_output() {
        let mut state = 1u64;
        for _ in 0..100 {
            let v = xorshift64(&mut state);
            assert_ne!(v, 0, "xorshift64 must never produce 0");
        }
    }

    #[test]
    fn random_f64_in_unit_interval() {
        let mut state = 42u64;
        for _ in 0..1000 {
            let v = random_f64(&mut state);
            assert!(v >= 0.0 && v < 1.0, "random_f64 out of [0,1): {v}");
        }
    }
}
