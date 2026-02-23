//! Simplified Universal Force Field (UFF) for molecular visualization.
//!
//! This module provides a UFF-like implementation suitable for interactive
//! energy minimization and basic molecular dynamics in a visualization context.
//! It is NOT a production molecular dynamics force field — accuracy is traded
//! for simplicity and speed to support real-time 3D visualization.
//!
//! ## Energy Terms
//!
//! - **Bond stretch**: harmonic E = 0.5 * k * (r - r0)^2
//! - **Angle bend**: harmonic E = 0.5 * kA * (theta - theta0)^2
//! - **Torsion**: cosine E = kT * (1 + cos(n*phi - phi0))
//! - **van der Waals**: Lennard-Jones 12-6 E = 4*eps * [(sigma/r)^12 - (sigma/r)^6]
//! - **Electrostatics**: Coulomb E = q1*q2 / (4*pi*eps0 * dielectric * r)
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};
//! use nexcore_viz::force_field::{ForceFieldConfig, compute_energy};
//!
//! let mut mol = Molecule::new("Water");
//! mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(2, Element::H, [0.96, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(3, Element::H, [-0.24, 0.93, 0.0]));
//! mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
//! mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
//!
//! let config = ForceFieldConfig::default();
//! let energy = compute_energy(&mol, &config).ok();
//! assert!(energy.is_some());
//! ```

use crate::molecular::{Atom, Bond, Element, Molecule};
use serde::{Deserialize, Serialize};

// ============================================================================
// Error type
// ============================================================================

/// Errors that can occur during force field calculations.
#[derive(Debug, Clone, PartialEq)]
pub enum ForceFieldError {
    /// Atom index is out of bounds for the molecule.
    InvalidAtom { index: usize, n_atoms: usize },
    /// Bond references an invalid atom index.
    InvalidBond { bond_idx: usize, atom_idx: usize },
    /// The molecule has no atoms to compute energy for.
    EmptyMolecule,
}

impl std::fmt::Display for ForceFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAtom { index, n_atoms } => {
                write!(
                    f,
                    "atom index {index} out of bounds (molecule has {n_atoms} atoms)"
                )
            }
            Self::InvalidBond { bond_idx, atom_idx } => {
                write!(
                    f,
                    "bond {bond_idx} references invalid atom index {atom_idx}"
                )
            }
            Self::EmptyMolecule => write!(f, "molecule has no atoms"),
        }
    }
}

impl std::error::Error for ForceFieldError {}

// ============================================================================
// UFF parameters per atom type
// ============================================================================

/// Universal Force Field parameters for a single atom type.
///
/// Parameters are loosely based on UFF (Rappe et al. JACS 1992) but simplified
/// for visualization purposes. All distances in angstroms, energies in kcal/mol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ForceFieldParams {
    /// Equilibrium bond radius (Å) — used to compute ideal bond lengths.
    pub r0: f64,
    /// Equilibrium bond angle (radians).
    pub theta0: f64,
    /// Bond stretch force constant (kcal/mol/Å²).
    pub k_bond: f64,
    /// Angle bend force constant (kcal/mol/rad²).
    pub k_angle: f64,
    /// Torsion barrier (kcal/mol).
    pub k_torsion: f64,
    /// Lennard-Jones well depth epsilon (kcal/mol).
    pub epsilon: f64,
    /// Lennard-Jones sigma parameter (Å) — zero-energy distance.
    pub sigma: f64,
}

impl ForceFieldParams {
    /// Construct parameters directly.
    #[must_use]
    pub const fn new(
        r0: f64,
        theta0: f64,
        k_bond: f64,
        k_angle: f64,
        k_torsion: f64,
        epsilon: f64,
        sigma: f64,
    ) -> Self {
        Self {
            r0,
            theta0,
            k_bond,
            k_angle,
            k_torsion,
            epsilon,
            sigma,
        }
    }
}

/// Return UFF-like parameters for the given element.
///
/// Values are simplified from Rappe et al. (1992) UFF for visualization.
/// The equilibrium bond length for a pair is r0_a + r0_b (combining rule).
#[must_use]
pub fn default_params(element: &Element) -> ForceFieldParams {
    // theta0 in radians: tetrahedral ~109.5 deg, trigonal ~120 deg
    let tetrahedral = 109.471_f64.to_radians();
    let trigonal = 120.0_f64.to_radians();
    let linear = 180.0_f64.to_radians();

    match element {
        Element::H => ForceFieldParams::new(0.354, linear, 700.0, 0.0, 0.0, 0.044, 2.886),
        Element::He => ForceFieldParams::new(0.849, linear, 0.0, 0.0, 0.0, 0.056, 2.362),
        Element::C => ForceFieldParams::new(0.757, tetrahedral, 700.0, 100.0, 2.119, 0.105, 3.851),
        Element::N => ForceFieldParams::new(0.700, tetrahedral, 700.0, 100.0, 0.450, 0.069, 3.660),
        Element::O => ForceFieldParams::new(0.658, tetrahedral, 700.0, 100.0, 0.018, 0.060, 3.500),
        Element::F => ForceFieldParams::new(0.668, tetrahedral, 700.0, 100.0, 0.0, 0.050, 3.364),
        Element::Na => ForceFieldParams::new(1.411, linear, 150.0, 30.0, 0.0, 0.030, 5.540),
        Element::Mg => ForceFieldParams::new(1.341, tetrahedral, 250.0, 50.0, 0.0, 0.111, 4.936),
        Element::P => ForceFieldParams::new(1.117, tetrahedral, 500.0, 75.0, 1.200, 0.305, 4.147),
        Element::S => ForceFieldParams::new(1.167, tetrahedral, 500.0, 75.0, 0.484, 0.274, 4.035),
        Element::Cl => ForceFieldParams::new(1.044, linear, 500.0, 75.0, 0.0, 0.227, 3.947),
        Element::K => ForceFieldParams::new(1.672, linear, 80.0, 20.0, 0.0, 0.035, 6.188),
        Element::Ca => ForceFieldParams::new(1.562, linear, 200.0, 40.0, 0.0, 0.238, 5.581),
        Element::Fe => ForceFieldParams::new(1.285, tetrahedral, 350.0, 60.0, 0.0, 0.013, 4.886),
        Element::Zn => ForceFieldParams::new(1.225, tetrahedral, 350.0, 60.0, 0.0, 0.124, 4.541),
        Element::Br => ForceFieldParams::new(1.141, linear, 450.0, 70.0, 0.0, 0.251, 4.189),
        Element::I => ForceFieldParams::new(1.360, linear, 380.0, 60.0, 0.0, 0.339, 4.750),
        Element::Se => ForceFieldParams::new(1.224, tetrahedral, 430.0, 65.0, 0.416, 0.291, 4.205),
        Element::Other => {
            ForceFieldParams::new(trigonal, tetrahedral, 300.0, 50.0, 0.0, 0.100, 3.800)
        }
    }
}

// ============================================================================
// Force field configuration
// ============================================================================

/// Global configuration for the force field calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceFieldConfig {
    /// Non-bonded interaction cutoff distance in angstroms.
    pub cutoff: f64,
    /// Dielectric constant for electrostatic damping.
    pub dielectric: f64,
    /// Whether to include electrostatic terms.
    pub use_electrostatics: bool,
    /// Step size for numerical gradient (finite differences), in angstroms.
    pub gradient_step: f64,
}

impl Default for ForceFieldConfig {
    fn default() -> Self {
        Self {
            cutoff: 10.0,
            dielectric: 1.0,
            use_electrostatics: true,
            gradient_step: 1e-4,
        }
    }
}

// ============================================================================
// Energy components
// ============================================================================

/// Breakdown of the molecular potential energy into individual contributions.
///
/// All energies are in kcal/mol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct EnergyComponents {
    /// Harmonic bond stretch energy sum.
    pub bond_stretch: f64,
    /// Harmonic angle bend energy sum.
    pub angle_bend: f64,
    /// Cosine torsion energy sum.
    pub torsion: f64,
    /// Lennard-Jones 12-6 van der Waals energy.
    pub vdw: f64,
    /// Coulombic electrostatic energy.
    pub electrostatic: f64,
    /// Total: sum of all contributions.
    pub total: f64,
}

// ============================================================================
// Individual energy term functions
// ============================================================================

/// Compute the harmonic bond stretch energy for a single bond.
///
/// E = 0.5 * k_eff * (r - r0_eff)^2
///
/// The effective equilibrium length is the sum of the two atoms' r0 values
/// (UFF combining rule). The effective force constant is the geometric mean.
#[must_use]
pub fn bond_energy(mol: &Molecule, bond: &Bond, params: &[ForceFieldParams]) -> f64 {
    let a1 = match mol.atoms.get(bond.atom1) {
        Some(a) => a,
        None => return 0.0,
    };
    let a2 = match mol.atoms.get(bond.atom2) {
        Some(a) => a,
        None => return 0.0,
    };

    let p1 = get_params(params, bond.atom1, &a1.element);
    let p2 = get_params(params, bond.atom2, &a2.element);

    // UFF combining rules
    let r0_eff = p1.r0 + p2.r0;
    let k_eff = (p1.k_bond * p2.k_bond).sqrt();

    let r = atom_distance(a1, a2);
    0.5 * k_eff * (r - r0_eff).powi(2)
}

/// Compute the harmonic angle bend energy for the angle formed by atoms i-j-k.
///
/// E = 0.5 * kA_eff * (theta - theta0_eff)^2
///
/// Atom j is the central atom.
#[must_use]
pub fn angle_energy(
    mol: &Molecule,
    i: usize,
    j: usize,
    k: usize,
    params: &[ForceFieldParams],
) -> f64 {
    let ai = match mol.atoms.get(i) {
        Some(a) => a,
        None => return 0.0,
    };
    let aj = match mol.atoms.get(j) {
        Some(a) => a,
        None => return 0.0,
    };
    let ak = match mol.atoms.get(k) {
        Some(a) => a,
        None => return 0.0,
    };

    // Vectors from central atom j to terminal atoms i and k
    let v1 = vec3_sub(ai.position, aj.position);
    let v2 = vec3_sub(ak.position, aj.position);
    let n1 = vec3_norm(v1);
    let n2 = vec3_norm(v2);

    if n1 < 1e-10 || n2 < 1e-10 {
        return 0.0;
    }

    let cos_theta = (vec3_dot(v1, v2) / (n1 * n2)).clamp(-1.0, 1.0);
    let theta = cos_theta.acos();

    let pj = get_params(params, j, &aj.element);
    let theta0 = pj.theta0;
    let k_angle = pj.k_angle;

    0.5 * k_angle * (theta - theta0).powi(2)
}

/// Compute the cosine torsion energy for the dihedral angle i-j-k-l.
///
/// E = kT * (1 - cos(2*phi))
///
/// Uses the central bond j-k and a simplified two-fold cosine potential.
#[must_use]
pub fn torsion_energy(
    mol: &Molecule,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    params: &[ForceFieldParams],
) -> f64 {
    let ai = match mol.atoms.get(i) {
        Some(a) => a,
        None => return 0.0,
    };
    let aj = match mol.atoms.get(j) {
        Some(a) => a,
        None => return 0.0,
    };
    let ak = match mol.atoms.get(k) {
        Some(a) => a,
        None => return 0.0,
    };
    let al = match mol.atoms.get(l) {
        Some(a) => a,
        None => return 0.0,
    };

    // Dihedral angle via cross products (IUPAC convention)
    let b1 = vec3_sub(aj.position, ai.position);
    let b2 = vec3_sub(ak.position, aj.position);
    let b3 = vec3_sub(al.position, ak.position);

    let n1 = vec3_cross(b1, b2);
    let n2 = vec3_cross(b2, b3);

    let nn1 = vec3_norm(n1);
    let nn2 = vec3_norm(n2);

    if nn1 < 1e-10 || nn2 < 1e-10 {
        return 0.0;
    }

    let cos_phi = (vec3_dot(n1, n2) / (nn1 * nn2)).clamp(-1.0, 1.0);
    let phi = cos_phi.acos();

    // Use geometric mean of j and k torsion barriers
    let pj = get_params(params, j, &aj.element);
    let pk = get_params(params, k, &ak.element);
    let k_t = (pj.k_torsion * pk.k_torsion).sqrt();

    // Two-fold cosine: E = kT * (1 - cos(2*phi))
    k_t * (1.0 - (2.0 * phi).cos())
}

/// Compute the Lennard-Jones 12-6 van der Waals energy between two atoms.
///
/// E = 4 * eps_eff * [(sigma_eff/r)^12 - (sigma_eff/r)^6]
///
/// Combining rules: eps_eff = sqrt(eps1 * eps2), sigma_eff = (sigma1 + sigma2) / 2
#[must_use]
pub fn vdw_energy(a1: &Atom, a2: &Atom, params: &[ForceFieldParams]) -> f64 {
    let p1 = get_params(params, a1.id as usize, &a1.element);
    let p2 = get_params(params, a2.id as usize, &a2.element);

    let eps_eff = (p1.epsilon * p2.epsilon).sqrt();
    let sigma_eff = (p1.sigma + p2.sigma) * 0.5;

    let r = atom_distance(a1, a2);
    if r < 0.5 {
        // Avoid division by zero / extreme repulsion at very short range
        return 1000.0;
    }

    let sr6 = (sigma_eff / r).powi(6);
    4.0 * eps_eff * (sr6 * sr6 - sr6)
}

// ============================================================================
// Full energy computation
// ============================================================================

/// Compute the total potential energy for a molecule.
///
/// Returns an [`EnergyComponents`] breakdown covering bond stretch, angle bend,
/// torsion, van der Waals, and electrostatic contributions.
///
/// # Errors
///
/// Returns [`ForceFieldError::EmptyMolecule`] if the molecule has no atoms, or
/// [`ForceFieldError::InvalidBond`] if any bond references an out-of-bounds atom.
pub fn compute_energy(
    mol: &Molecule,
    config: &ForceFieldConfig,
) -> Result<EnergyComponents, ForceFieldError> {
    if mol.atoms.is_empty() {
        return Err(ForceFieldError::EmptyMolecule);
    }

    // Validate bond indices upfront
    for (bi, bond) in mol.bonds.iter().enumerate() {
        if bond.atom1 >= mol.atoms.len() {
            return Err(ForceFieldError::InvalidBond {
                bond_idx: bi,
                atom_idx: bond.atom1,
            });
        }
        if bond.atom2 >= mol.atoms.len() {
            return Err(ForceFieldError::InvalidBond {
                bond_idx: bi,
                atom_idx: bond.atom2,
            });
        }
    }

    // Build per-atom parameter array (indexed by position in mol.atoms)
    let params: Vec<ForceFieldParams> = mol
        .atoms
        .iter()
        .map(|a| default_params(&a.element))
        .collect();

    let mut components = EnergyComponents::default();

    // Bond stretch
    for bond in &mol.bonds {
        components.bond_stretch += bond_energy(mol, bond, &params);
    }

    // Angle bend — enumerate all i-j-k triples where j is central.
    let angles = enumerate_angles(mol);
    for &(i, j, k) in &angles {
        components.angle_bend += angle_energy(mol, i, j, k, &params);
    }

    // Torsion
    let torsions = enumerate_torsions(mol);
    for &(i, j, k, l) in &torsions {
        components.torsion += torsion_energy(mol, i, j, k, l, &params);
    }

    // Non-bonded: VdW + electrostatics
    let bonded_pairs = bonded_pair_set(mol);
    let n = mol.atoms.len();
    for ai in 0..n {
        for bi in (ai + 1)..n {
            // Skip 1-2 and 1-3 pairs (directly bonded or angle-related)
            if bonded_pairs.contains(&(ai, bi)) {
                continue;
            }

            let a1 = &mol.atoms[ai];
            let a2 = &mol.atoms[bi];
            let r = atom_distance(a1, a2);

            if r > config.cutoff {
                continue;
            }

            // VdW
            let p1 = params[ai];
            let p2 = params[bi];
            let eps_eff = (p1.epsilon * p2.epsilon).sqrt();
            let sigma_eff = (p1.sigma + p2.sigma) * 0.5;

            if r < 0.5 {
                components.vdw += 1000.0;
            } else {
                let sr6 = (sigma_eff / r).powi(6);
                components.vdw += 4.0 * eps_eff * (sr6 * sr6 - sr6);
            }

            // Electrostatics (Coulomb, kcal/mol with charge in elementary units)
            if config.use_electrostatics {
                // Coulomb constant in kcal*Å/(mol*e²) ≈ 332.06
                const COULOMB_K: f64 = 332.06;
                let q1 = f64::from(a1.charge);
                let q2 = f64::from(a2.charge);
                if r > 1e-10 {
                    components.electrostatic += COULOMB_K * q1 * q2 / (config.dielectric * r);
                }
            }
        }
    }

    components.total = components.bond_stretch
        + components.angle_bend
        + components.torsion
        + components.vdw
        + components.electrostatic;

    Ok(components)
}

// ============================================================================
// Force computation
// ============================================================================

/// Compute per-atom force vectors via numerical gradient: F_i = -dE/dr_i.
///
/// Uses central finite differences with step size from [`ForceFieldConfig::gradient_step`].
/// Returns a `Vec<[f64; 3]>` with one force vector (Fx, Fy, Fz) per atom,
/// in kcal/mol/Å.
///
/// # Errors
///
/// Propagates [`ForceFieldError`] from [`compute_energy`].
pub fn compute_forces(
    mol: &Molecule,
    config: &ForceFieldConfig,
) -> Result<Vec<[f64; 3]>, ForceFieldError> {
    if mol.atoms.is_empty() {
        return Err(ForceFieldError::EmptyMolecule);
    }

    let n = mol.atoms.len();
    let h = config.gradient_step;
    let mut forces = vec![[0.0_f64; 3]; n];

    // Mutable clone to perturb positions
    let mut mol_scratch = mol.clone();

    for (atom_idx, force_row) in forces.iter_mut().enumerate() {
        for (dim, force_val) in force_row.iter_mut().enumerate() {
            // Forward displacement
            mol_scratch.atoms[atom_idx].position[dim] += h;
            let e_fwd = compute_energy(&mol_scratch, config)?.total;

            // Backward displacement
            mol_scratch.atoms[atom_idx].position[dim] -= 2.0 * h;
            let e_bwd = compute_energy(&mol_scratch, config)?.total;

            // Restore
            mol_scratch.atoms[atom_idx].position[dim] += h;

            // Central difference: force = -dE/dr
            *force_val = -(e_fwd - e_bwd) / (2.0 * h);
        }
    }

    Ok(forces)
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Retrieve parameters from a pre-built slice; fall back to element default.
#[inline]
fn get_params(params: &[ForceFieldParams], idx: usize, element: &Element) -> ForceFieldParams {
    params
        .get(idx)
        .copied()
        .unwrap_or_else(|| default_params(element))
}

/// Euclidean distance between two atoms.
#[inline]
#[must_use]
pub fn atom_distance(a: &Atom, b: &Atom) -> f64 {
    let dx = a.position[0] - b.position[0];
    let dy = a.position[1] - b.position[1];
    let dz = a.position[2] - b.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn vec3_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_norm(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[inline]
fn vec3_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Enumerate all unique angle triples (i, j, k) where j is the central atom.
pub fn enumerate_angles(mol: &Molecule) -> Vec<(usize, usize, usize)> {
    let mut angles = Vec::new();
    let n = mol.atoms.len();

    for j in 0..n {
        let neighbors = mol.bonded_to(j);
        let m = neighbors.len();
        for a in 0..m {
            for b in (a + 1)..m {
                let i = neighbors[a];
                let k = neighbors[b];
                if i < k {
                    angles.push((i, j, k));
                } else {
                    angles.push((k, j, i));
                }
            }
        }
    }
    angles
}

/// Enumerate all unique torsion quadruples (i, j, k, l) along each bond.
pub fn enumerate_torsions(mol: &Molecule) -> Vec<(usize, usize, usize, usize)> {
    let mut torsions = Vec::new();

    for bond in &mol.bonds {
        let j = bond.atom1;
        let k = bond.atom2;

        let j_neighbors = mol.bonded_to(j);
        let k_neighbors = mol.bonded_to(k);

        for &i in &j_neighbors {
            if i == k {
                continue;
            }
            for &l in &k_neighbors {
                if l == j || l == i {
                    continue;
                }
                let quad = if i < l { (i, j, k, l) } else { (l, k, j, i) };
                if !torsions.contains(&quad) {
                    torsions.push(quad);
                }
            }
        }
    }
    torsions
}

/// Build a set of directly bonded pairs and 1-3 pairs (angle-related).
///
/// Used to exclude these from non-bonded calculations.
pub fn bonded_pair_set(mol: &Molecule) -> std::collections::HashSet<(usize, usize)> {
    let mut set = std::collections::HashSet::new();

    // 1-2 pairs (direct bonds)
    for bond in &mol.bonds {
        let (a, b) = (bond.atom1.min(bond.atom2), bond.atom1.max(bond.atom2));
        set.insert((a, b));
    }

    // 1-3 pairs (angle-related: atoms share a common bonded neighbour)
    let angles = enumerate_angles(mol);
    for (i, _j, k) in angles {
        let (a, b) = (i.min(k), i.max(k));
        set.insert((a, b));
    }

    set
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    /// Build a simple water molecule (O-H-H).
    fn water() -> Molecule {
        let mut mol = Molecule::new("Water");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [0.96, 0.0, 0.0]));
        mol.atoms.push(Atom::new(3, Element::H, [-0.24, 0.93, 0.0]));
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

    /// Build methane: C at origin, 4 H atoms in approximate tetrahedral geometry.
    fn methane() -> Molecule {
        let mut mol = Molecule::new("Methane");
        let h = 0.629_f64; // ~1.089 Å / sqrt(3)
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [h, h, h]));
        mol.atoms.push(Atom::new(3, Element::H, [-h, -h, h]));
        mol.atoms.push(Atom::new(4, Element::H, [-h, h, -h]));
        mol.atoms.push(Atom::new(5, Element::H, [h, -h, -h]));
        for i in 1..5 {
            mol.bonds.push(Bond {
                atom1: 0,
                atom2: i,
                order: BondOrder::Single,
            });
        }
        mol
    }

    #[test]
    fn default_params_carbon() {
        let p = default_params(&Element::C);
        assert!((p.r0 - 0.757).abs() < 1e-3);
        assert!(p.epsilon > 0.0);
        assert!(p.sigma > 0.0);
    }

    #[test]
    fn empty_molecule_returns_error() {
        let mol = Molecule::new("Empty");
        let config = ForceFieldConfig::default();
        let result = compute_energy(&mol, &config);
        assert!(matches!(result, Err(ForceFieldError::EmptyMolecule)));
    }

    #[test]
    fn bond_energy_positive_for_stretched_bond() {
        let mut mol = Molecule::new("HH-stretched");
        mol.atoms.push(Atom::new(1, Element::H, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [5.0, 0.0, 0.0])); // far apart
        mol.bonds.push(Bond {
            atom1: 0,
            atom2: 1,
            order: BondOrder::Single,
        });
        let params: Vec<_> = mol
            .atoms
            .iter()
            .map(|a| default_params(&a.element))
            .collect();
        let e = bond_energy(&mol, &mol.bonds[0], &params);
        assert!(e > 0.0, "stretched bond should have positive energy: {e}");
    }

    #[test]
    fn compute_energy_water_is_finite() {
        let mol = water();
        let config = ForceFieldConfig::default();
        let result = compute_energy(&mol, &config);
        assert!(result.is_ok());
        let e = result.unwrap_or_default();
        assert!(
            e.total.is_finite(),
            "energy should be finite, got {}",
            e.total
        );
        assert!(e.bond_stretch >= 0.0, "bond stretch must be non-negative");
        assert!(e.angle_bend >= 0.0, "angle bend must be non-negative");
    }

    #[test]
    fn compute_energy_methane_has_angles() {
        let mol = methane();
        let config = ForceFieldConfig::default();
        let e = compute_energy(&mol, &config).unwrap_or_default();
        assert!(e.angle_bend >= 0.0);
        assert!(e.bond_stretch >= 0.0);
    }

    #[test]
    fn vdw_energy_repulsive_at_short_range() {
        let a1 = Atom::new(1, Element::C, [0.0, 0.0, 0.0]);
        let a2 = Atom::new(2, Element::C, [0.1, 0.0, 0.0]); // very close
        let params = vec![default_params(&Element::C), default_params(&Element::C)];
        let e = vdw_energy(&a1, &a2, &params);
        assert!(e > 0.0, "VdW should be repulsive at close range: {e}");
    }

    #[test]
    fn compute_forces_length_matches_atoms() {
        let mol = water();
        let config = ForceFieldConfig::default();
        let forces = compute_forces(&mol, &config);
        assert!(forces.is_ok());
        let f = forces.unwrap_or_default();
        assert_eq!(f.len(), mol.atoms.len());
    }

    #[test]
    fn forces_are_finite() {
        let mol = methane();
        let config = ForceFieldConfig::default();
        let forces = compute_forces(&mol, &config).unwrap_or_default();
        for (i, force) in forces.iter().enumerate() {
            assert!(
                force.iter().all(|x| x.is_finite()),
                "force on atom {i} is not finite: {force:?}"
            );
        }
    }

    #[test]
    fn invalid_bond_returns_error() {
        let mut mol = Molecule::new("Bad");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        // Bond references atom index 99 which doesn't exist
        mol.bonds.push(Bond {
            atom1: 0,
            atom2: 99,
            order: BondOrder::Single,
        });
        let config = ForceFieldConfig::default();
        let result = compute_energy(&mol, &config);
        assert!(matches!(result, Err(ForceFieldError::InvalidBond { .. })));
    }

    #[test]
    fn angle_energy_near_zero_at_equilibrium() {
        // H-O-H at the oxygen's equilibrium angle
        let p_o = default_params(&Element::O);
        let theta0 = p_o.theta0;
        let mut mol = Molecule::new("WaterEquil");
        mol.atoms.push(Atom::new(1, Element::H, [0.96, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::O, [0.0, 0.0, 0.0]));
        let x2 = 0.96 * theta0.cos();
        let y2 = 0.96 * theta0.sin();
        mol.atoms.push(Atom::new(3, Element::H, [x2, y2, 0.0]));
        let params: Vec<_> = mol
            .atoms
            .iter()
            .map(|a| default_params(&a.element))
            .collect();
        let e = angle_energy(&mol, 0, 1, 2, &params);
        // Should be near zero (atoms placed at equilibrium angle)
        assert!(
            e < 1e-6,
            "angle energy at equilibrium should be ~0, got {e}"
        );
    }

    #[test]
    fn torsion_energy_non_negative() {
        // H-C-C-H dihedral
        let mut mol = Molecule::new("Ethane-torsion");
        mol.atoms.push(Atom::new(1, Element::H, [0.0, 0.0, 1.0]));
        mol.atoms.push(Atom::new(2, Element::C, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(3, Element::C, [1.5, 0.0, 0.0]));
        mol.atoms.push(Atom::new(4, Element::H, [1.5, 1.0, 0.0]));
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
        mol.bonds.push(Bond {
            atom1: 2,
            atom2: 3,
            order: BondOrder::Single,
        });
        let params: Vec<_> = mol
            .atoms
            .iter()
            .map(|a| default_params(&a.element))
            .collect();
        let e = torsion_energy(&mol, 0, 1, 2, 3, &params);
        assert!(e >= 0.0, "torsion energy must be non-negative: {e}");
    }
}
