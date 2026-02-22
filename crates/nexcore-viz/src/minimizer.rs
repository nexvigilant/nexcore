//! Energy minimization algorithms for molecular geometry optimization.
//!
//! Provides steepest descent and Fletcher-Reeves conjugate gradient minimizers
//! that use the UFF force field from [`crate::force_field`] to drive a molecule
//! toward a local energy minimum. Designed for interactive visualization use
//! (fast convergence, not production-quality MD pre-processing).
//!
//! ## Algorithms
//!
//! - **Steepest descent** (`steepest_descent`): moves along the negative gradient.
//!   Simple and robust; converges slowly near the minimum.
//! - **Conjugate gradient** (`conjugate_gradient`): Fletcher-Reeves CG method.
//!   Much faster convergence than steepest descent for well-behaved surfaces.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};
//! use nexcore_viz::force_field::ForceFieldConfig;
//! use nexcore_viz::minimizer::{MinimizerConfig, steepest_descent};
//!
//! let mut mol = Molecule::new("Water");
//! mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(2, Element::H, [1.2, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(3, Element::H, [-0.3, 1.1, 0.0]));
//! mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
//! mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
//!
//! let min_config = MinimizerConfig::default();
//! let ff_config = ForceFieldConfig::default();
//! // Result is Ok on convergence or Err(MaxIterations) if limit reached
//! let _result = steepest_descent(&mut mol, &min_config, &ff_config);
//! ```

use crate::force_field::{
    ForceFieldConfig, ForceFieldError, compute_energy, compute_forces,
};
use crate::molecular::Molecule;
use serde::{Deserialize, Serialize};

// ============================================================================
// Error type
// ============================================================================

/// Errors that can occur during energy minimization.
#[derive(Debug, Clone, PartialEq)]
pub enum MinimizerError {
    /// Reached maximum iterations without meeting the force tolerance.
    MaxIterations { iterations: usize, force_rms: f64 },
    /// An error propagated from the force field layer.
    ForceField(ForceFieldError),
    /// The minimization procedure failed to make progress (flat gradient, NaN).
    Convergence(String),
}

impl std::fmt::Display for MinimizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxIterations { iterations, force_rms } => write!(
                f,
                "maximum iterations ({iterations}) reached; final force RMS = {force_rms}"
            ),
            Self::ForceField(e) => write!(f, "force field error: {e}"),
            Self::Convergence(msg) => write!(f, "convergence failure: {msg}"),
        }
    }
}

impl std::error::Error for MinimizerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ForceField(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ForceFieldError> for MinimizerError {
    fn from(e: ForceFieldError) -> Self {
        Self::ForceField(e)
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for energy minimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimizerConfig {
    /// Maximum number of optimization iterations.
    pub max_iterations: usize,
    /// Convergence criterion: RMS force on all atoms (kcal/mol/Å).
    pub force_tolerance: f64,
    /// Initial step size for position updates (Å).
    pub step_size: f64,
    /// Whether to use backtracking line search to find an optimal step.
    pub line_search: bool,
}

impl Default for MinimizerConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            force_tolerance: 0.1,
            step_size: 0.01,
            line_search: true,
        }
    }
}

// ============================================================================
// Result type
// ============================================================================

/// Outcome of an energy minimization run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimizationResult {
    /// Final total potential energy (kcal/mol).
    pub final_energy: f64,
    /// Number of iterations performed.
    pub iterations: u32,
    /// Whether the minimization converged within the force tolerance.
    pub converged: bool,
    /// Total energy recorded at each iteration.
    pub energy_history: Vec<f64>,
    /// Final RMS force on all atoms (kcal/mol/Å).
    pub force_rms: f64,
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Compute the RMS force magnitude across all atoms.
fn rms_force(forces: &[[f64; 3]]) -> f64 {
    if forces.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = forces.iter().flat_map(|f| f.iter()).map(|x| x * x).sum();
    (sum_sq / (forces.len() * 3) as f64).sqrt()
}

/// Apply a displacement to all atom positions: pos += step * direction.
fn apply_displacement(mol: &mut Molecule, step: f64, direction: &[[f64; 3]]) {
    for (atom, dir) in mol.atoms.iter_mut().zip(direction.iter()) {
        atom.position[0] += step * dir[0];
        atom.position[1] += step * dir[1];
        atom.position[2] += step * dir[2];
    }
}

/// Dot product of two flat force/direction arrays.
fn dot_forces(a: &[[f64; 3]], b: &[[f64; 3]]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(ai, bi)| ai[0] * bi[0] + ai[1] * bi[1] + ai[2] * bi[2])
        .sum()
}

// ============================================================================
// Line search
// ============================================================================

/// Backtracking Armijo line search along a search direction.
///
/// Starts from a default step and halves it until the energy decreases by at
/// least the Armijo sufficient-decrease condition, or the step becomes tiny.
///
/// Returns the accepted step size, or [`MinimizerError::Convergence`] if no
/// decrease can be found.
///
/// # Errors
///
/// Returns [`MinimizerError::ForceField`] on energy computation failure, or
/// [`MinimizerError::Convergence`] if no acceptable step is found.
pub fn line_search(
    mol: &mut Molecule,
    direction: &[[f64; 3]],
    ff_config: &ForceFieldConfig,
) -> Result<f64, MinimizerError> {
    const MAX_BACKTRACKS: usize = 30;
    const ARMIJO_C: f64 = 1e-4;
    const CONTRACTION: f64 = 0.5;
    const MIN_STEP: f64 = 1e-12;

    let e0 = compute_energy(mol, ff_config)?.total;
    let forces0 = compute_forces(mol, ff_config)?;
    // Directional derivative: forces = -grad; direction is along forces.
    let slope = -dot_forces(&forces0, direction);

    let mut step = 0.1;

    for _ in 0..MAX_BACKTRACKS {
        // Temporarily apply displacement, measure energy, then undo.
        apply_displacement(mol, step, direction);
        let e_new = compute_energy(mol, ff_config)?.total;
        apply_displacement(mol, -step, direction);

        // Armijo sufficient decrease: E(x + step*d) <= E(x) + c * step * slope
        if e_new <= e0 + ARMIJO_C * step * slope || e_new < e0 {
            return Ok(step);
        }

        step *= CONTRACTION;

        if step < MIN_STEP {
            return Ok(step);
        }
    }

    Err(MinimizerError::Convergence(
        "line search exhausted backtrack budget without finding a descent step".to_string(),
    ))
}

// ============================================================================
// Steepest descent
// ============================================================================

/// Minimize molecular energy using the steepest descent method.
///
/// Each iteration moves atoms along the negative energy gradient (the force
/// direction). If `config.line_search` is enabled, an Armijo backtracking line
/// search determines the optimal step size; otherwise `config.step_size` is
/// used with simple adaptive scaling.
///
/// On convergence returns `Ok(MinimizationResult { converged: true, .. })`.
/// If the maximum iteration count is reached before convergence, returns
/// `Err(MinimizerError::MaxIterations)` — the molecule positions are still
/// updated in-place, so callers can continue from the partially minimized state.
///
/// # Errors
///
/// Returns [`MinimizerError::MaxIterations`] if the force tolerance is not met
/// within `config.max_iterations` steps, [`MinimizerError::Convergence`] if the
/// step collapses, or propagates [`MinimizerError::ForceField`] on force field errors.
pub fn steepest_descent(
    mol: &mut Molecule,
    config: &MinimizerConfig,
    ff_config: &ForceFieldConfig,
) -> Result<MinimizationResult, MinimizerError> {
    let mut energy_history = Vec::with_capacity(config.max_iterations.min(1000));
    let mut step = config.step_size;
    let mut prev_energy = compute_energy(mol, ff_config)?.total;
    energy_history.push(prev_energy);

    for iter in 0..config.max_iterations {
        let forces = compute_forces(mol, ff_config)?;
        let frms = rms_force(&forces);

        if frms < config.force_tolerance {
            let final_energy = compute_energy(mol, ff_config)?.total;
            return Ok(MinimizationResult {
                final_energy,
                iterations: iter as u32,
                converged: true,
                energy_history,
                force_rms: frms,
            });
        }

        // Steepest descent direction is the force vector (= negative gradient).
        let direction: Vec<[f64; 3]> = forces.clone();

        let accepted_step = if config.line_search {
            // Fall back to a small fixed step if line search fails.
            line_search(mol, &direction, ff_config).unwrap_or(step * 0.1)
        } else {
            step
        };

        apply_displacement(mol, accepted_step, &direction);

        let new_energy = compute_energy(mol, ff_config)?.total;

        // Without line search: revert and shrink step if energy increased.
        if !config.line_search && new_energy > prev_energy {
            apply_displacement(mol, -accepted_step, &direction);
            step *= 0.5;
            if step < 1e-12 {
                return Err(MinimizerError::Convergence(
                    "step size collapsed to zero in steepest descent".to_string(),
                ));
            }
            continue;
        }

        // Gently accelerate when consistently making progress.
        if !config.line_search && new_energy < prev_energy {
            step *= 1.05;
        }

        prev_energy = new_energy;
        energy_history.push(new_energy);
    }

    // Maximum iterations reached — compute final state for the error payload.
    let forces = compute_forces(mol, ff_config)?;
    let frms = rms_force(&forces);

    Err(MinimizerError::MaxIterations {
        iterations: config.max_iterations,
        force_rms: frms,
    })
}

// ============================================================================
// Conjugate gradient (Fletcher-Reeves)
// ============================================================================

/// Minimize molecular energy using the Fletcher-Reeves conjugate gradient method.
///
/// CG typically converges faster than steepest descent for smooth energy surfaces.
/// The search direction is updated each iteration using the Fletcher-Reeves beta
/// coefficient to incorporate curvature information. The direction is restarted
/// every `n_atoms` iterations or when beta is non-positive.
///
/// On convergence returns `Ok(MinimizationResult { converged: true, .. })`.
/// Returns `Err(MinimizerError::MaxIterations)` if the limit is reached.
///
/// # Errors
///
/// Returns [`MinimizerError::MaxIterations`] if the force tolerance is not met
/// within `config.max_iterations` steps, or propagates [`MinimizerError::ForceField`].
pub fn conjugate_gradient(
    mol: &mut Molecule,
    config: &MinimizerConfig,
    ff_config: &ForceFieldConfig,
) -> Result<MinimizationResult, MinimizerError> {
    let n = mol.atoms.len();
    let mut energy_history = Vec::with_capacity(config.max_iterations.min(1000));

    // Initial forces and search direction (steepest descent on first step).
    let mut forces = compute_forces(mol, ff_config)?;
    let mut direction: Vec<[f64; 3]> = forces.clone();
    let mut prev_grad_sq = dot_forces(&forces, &forces);

    let mut prev_energy = compute_energy(mol, ff_config)?.total;
    energy_history.push(prev_energy);

    for iter in 0..config.max_iterations {
        let frms = rms_force(&forces);

        if frms < config.force_tolerance {
            let final_energy = compute_energy(mol, ff_config)?.total;
            return Ok(MinimizationResult {
                final_energy,
                iterations: iter as u32,
                converged: true,
                energy_history,
                force_rms: frms,
            });
        }

        // Line search or fixed step along current CG direction.
        let step = if config.line_search {
            line_search(mol, &direction, ff_config).unwrap_or(config.step_size)
        } else {
            config.step_size
        };

        apply_displacement(mol, step, &direction);

        let new_energy = compute_energy(mol, ff_config)?.total;

        // Without line search: revert and restart as steepest descent if energy rose.
        if !config.line_search && new_energy > prev_energy {
            apply_displacement(mol, -step, &direction);
            forces = compute_forces(mol, ff_config)?;
            direction = forces.clone();
            prev_grad_sq = dot_forces(&forces, &forces);
            continue;
        }

        prev_energy = new_energy;
        energy_history.push(new_energy);

        // Recompute forces at the new position.
        let new_forces = compute_forces(mol, ff_config)?;
        let new_grad_sq = dot_forces(&new_forces, &new_forces);

        // Fletcher-Reeves beta coefficient.
        let beta = if prev_grad_sq.abs() < 1e-30 {
            0.0
        } else {
            new_grad_sq / prev_grad_sq
        };

        // Restart every n iterations or when beta is non-positive.
        let restart = (n > 0 && iter % n == n - 1) || beta <= 0.0;
        if restart {
            direction = new_forces.clone();
        } else {
            // d_new = f_new + beta * d_old
            direction = new_forces
                .iter()
                .zip(direction.iter())
                .map(|(&[fx, fy, fz], &[dx, dy, dz])| {
                    [fx + beta * dx, fy + beta * dy, fz + beta * dz]
                })
                .collect();
        }

        forces = new_forces;
        prev_grad_sq = new_grad_sq;
    }

    let frms = rms_force(&forces);

    Err(MinimizerError::MaxIterations {
        iterations: config.max_iterations,
        force_rms: frms,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::force_field::ForceFieldConfig;
    use crate::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    fn water_distorted() -> Molecule {
        let mut mol = Molecule::new("Water-distorted");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [1.5, 0.0, 0.0])); // stretched
        mol.atoms.push(Atom::new(3, Element::H, [0.0, 1.5, 0.0])); // stretched
        mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
        mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
        mol
    }

    fn methane_distorted() -> Molecule {
        let mut mol = Molecule::new("Methane-distorted");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [1.5, 0.0, 0.0]));
        mol.atoms.push(Atom::new(3, Element::H, [0.0, 1.5, 0.0]));
        mol.atoms.push(Atom::new(4, Element::H, [0.0, 0.0, 1.5]));
        mol.atoms.push(Atom::new(5, Element::H, [-1.0, -1.0, -1.0]));
        for i in 1..5 {
            mol.bonds.push(Bond { atom1: 0, atom2: i, order: BondOrder::Single });
        }
        mol
    }

    fn fast_config() -> MinimizerConfig {
        MinimizerConfig {
            max_iterations: 200,
            force_tolerance: 5.0, // lenient tolerance for unit tests
            step_size: 0.005,
            line_search: false, // skip line search for test speed
        }
    }

    /// Helper: get current energy of mol, returning f64::MAX on error.
    fn current_energy(mol: &Molecule) -> f64 {
        compute_energy(mol, &ForceFieldConfig::default())
            .map(|e| e.total)
            .unwrap_or(f64::MAX)
    }

    #[test]
    fn steepest_descent_reduces_energy() {
        let mut mol = water_distorted();
        let initial_energy = current_energy(&mol);
        let config = fast_config();
        let ff_config = ForceFieldConfig::default();

        match steepest_descent(&mut mol, &config, &ff_config) {
            Ok(_) | Err(MinimizerError::MaxIterations { .. }) => {}
            Err(e) => panic!("unexpected steepest descent error: {e}"),
        }

        let final_energy = current_energy(&mol);
        assert!(
            final_energy < initial_energy,
            "energy should decrease: {initial_energy:.4} -> {final_energy:.4}"
        );
    }

    #[test]
    fn steepest_descent_final_energy_finite() {
        let mut mol = water_distorted();
        let config = fast_config();
        let ff_config = ForceFieldConfig::default();

        match steepest_descent(&mut mol, &config, &ff_config) {
            Ok(_) | Err(MinimizerError::MaxIterations { .. }) => {}
            Err(e) => panic!("unexpected error: {e}"),
        }

        let e = current_energy(&mol);
        assert!(e.is_finite(), "post-SD energy must be finite, got {e}");
    }

    #[test]
    fn conjugate_gradient_reduces_energy() {
        let mut mol = water_distorted();
        let initial_energy = current_energy(&mol);
        // Use line search so CG reliably finds a descent step on the first iteration.
        let config = MinimizerConfig {
            max_iterations: 200,
            force_tolerance: 5.0,
            step_size: 0.01,
            line_search: true,
        };
        let ff_config = ForceFieldConfig::default();

        match conjugate_gradient(&mut mol, &config, &ff_config) {
            Ok(_) | Err(MinimizerError::MaxIterations { .. }) => {}
            Err(e) => panic!("unexpected CG error: {e}"),
        }

        let final_energy = current_energy(&mol);
        assert!(
            final_energy < initial_energy,
            "CG energy should decrease: {initial_energy:.4} -> {final_energy:.4}"
        );
    }

    #[test]
    fn conjugate_gradient_methane_reduces_energy() {
        let mut mol = methane_distorted();
        let initial_energy = current_energy(&mol);
        // Use line search so CG reliably finds a descent step on the first iteration.
        let config = MinimizerConfig {
            max_iterations: 100,
            force_tolerance: 10.0,
            step_size: 0.01,
            line_search: true,
        };
        let ff_config = ForceFieldConfig::default();

        match conjugate_gradient(&mut mol, &config, &ff_config) {
            Ok(_) | Err(MinimizerError::MaxIterations { .. }) => {}
            Err(e) => panic!("unexpected CG error on methane: {e}"),
        }

        let final_energy = current_energy(&mol);
        assert!(
            final_energy < initial_energy,
            "methane CG should reduce energy: {initial_energy:.4} -> {final_energy:.4}"
        );
    }

    #[test]
    fn line_search_returns_positive_step() {
        let mut mol = water_distorted();
        let ff_config = ForceFieldConfig::default();
        let forces = compute_forces(&mol, &ff_config).unwrap_or_default();
        match line_search(&mut mol, &forces, &ff_config) {
            Ok(s) => assert!(s > 0.0, "line search step must be positive, got {s}"),
            Err(e) => panic!("line search failed unexpectedly: {e}"),
        }
    }

    #[test]
    fn minimizer_error_display_contains_key_info() {
        let e = MinimizerError::MaxIterations { iterations: 500, force_rms: 0.42 };
        let s = e.to_string();
        assert!(s.contains("500"), "display should contain iteration count");
        assert!(s.contains("0.42"), "display should contain force RMS");
    }

    #[test]
    fn minimizer_config_default_values_sensible() {
        let c = MinimizerConfig::default();
        assert_eq!(c.max_iterations, 1000);
        assert!(c.force_tolerance > 0.0);
        assert!(c.step_size > 0.0);
        assert!(c.line_search);
    }
}
