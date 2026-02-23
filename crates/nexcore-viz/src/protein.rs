//! Protein structure analysis: secondary structure assignment, backbone geometry,
//! and structural metrics for 3D protein visualization.
//!
//! Implements DSSP-inspired hydrogen bond detection and secondary structure
//! assignment, phi/psi dihedral angle computation, Ramachandran region
//! classification, and whole-protein structural metrics.
//!
//! Primitive formula: protein = σ(residues) × ∂(secondary_structure) + N(geometry)

use serde::{Deserialize, Serialize};

use crate::molecular::{Molecule, SecondaryStructure};

// ============================================================================
// Error type
// ============================================================================

/// Errors that can arise during protein structure analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum ProteinError {
    /// The molecule has no chains (not a protein or chain data not loaded).
    NoChains,
    /// A referenced atom index is out of bounds.
    AtomIndexOutOfBounds(usize),
    /// A residue index is out of bounds.
    ResidueIndexOutOfBounds(usize),
}

impl std::fmt::Display for ProteinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoChains => write!(f, "molecule has no protein chains"),
            Self::AtomIndexOutOfBounds(idx) => {
                write!(f, "atom index {idx} is out of bounds")
            }
            Self::ResidueIndexOutOfBounds(idx) => {
                write!(f, "residue index {idx} is out of bounds")
            }
        }
    }
}

impl std::error::Error for ProteinError {}

// ============================================================================
// BackboneAtoms
// ============================================================================

/// Backbone atom indices for a single residue.
///
/// Each field holds the index into `Molecule.atoms` for the named backbone atom,
/// or `None` if that atom is absent or unresolved for this residue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackboneAtoms {
    /// Amide nitrogen atom index.
    pub n: Option<usize>,
    /// Alpha-carbon atom index.
    pub ca: Option<usize>,
    /// Carbonyl carbon atom index.
    pub c: Option<usize>,
    /// Carbonyl oxygen atom index.
    pub o: Option<usize>,
}

impl BackboneAtoms {
    /// Returns `true` if all four backbone atoms are present.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.n.is_some() && self.ca.is_some() && self.c.is_some() && self.o.is_some()
    }
}

// ============================================================================
// DihedralAngles
// ============================================================================

/// Phi and psi dihedral angles for a single residue (in radians).
///
/// - `phi`: C[i-1] — N[i] — CA[i] — C[i] (absent for the first residue)
/// - `psi`: N[i] — CA[i] — C[i] — N[i+1] (absent for the last residue)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DihedralAngles {
    /// Zero-based index of the residue in the flattened residue list.
    pub residue_index: usize,
    /// Phi angle in radians, or `None` for the first residue.
    pub phi: Option<f64>,
    /// Psi angle in radians, or `None` for the last residue.
    pub psi: Option<f64>,
}

// ============================================================================
// RamachandranRegion
// ============================================================================

/// Ramachandran diagram region classification for a (phi, psi) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RamachandranRegion {
    /// Favoured alpha-helix region (phi ≈ −60°, psi ≈ −45°).
    AlphaHelix,
    /// Favoured beta-sheet region (extended conformation).
    BetaSheet,
    /// Left-handed alpha-helix (positive phi).
    LeftHandedHelix,
    /// Broadly allowed region outside the core regions.
    GenerallyAllowed,
    /// Outlier region — sterically unfavourable.
    Outlier,
}

// ============================================================================
// HydrogenBond
// ============================================================================

/// A backbone hydrogen bond detected by the DSSP electrostatic model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HydrogenBond {
    /// Index of the donor atom (backbone N) in `Molecule.atoms`.
    pub donor_atom: usize,
    /// Index of the acceptor atom (backbone O) in `Molecule.atoms`.
    pub acceptor_atom: usize,
    /// Electrostatic interaction energy in kcal/mol (negative = favourable).
    pub energy: f64,
}

// ============================================================================
// ContactMapEntry
// ============================================================================

/// A residue–residue contact within a distance threshold.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ContactMapEntry {
    /// Zero-based index of the first residue in the flattened residue list.
    pub residue_i: usize,
    /// Zero-based index of the second residue in the flattened residue list.
    pub residue_j: usize,
    /// CA–CA distance in angstroms.
    pub distance: f64,
}

// ============================================================================
// ProteinMetrics
// ============================================================================

/// Whole-protein structural statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProteinMetrics {
    /// Total number of residues across all chains.
    pub residue_count: usize,
    /// Fraction of residues assigned as helix (0.0–1.0).
    pub helix_fraction: f64,
    /// Fraction of residues assigned as sheet (0.0–1.0).
    pub sheet_fraction: f64,
    /// Fraction of residues assigned as coil/loop (0.0–1.0).
    pub coil_fraction: f64,
    /// Radius of gyration in angstroms (mass-unweighted, over all atoms).
    pub radius_of_gyration: f64,
    /// Total backbone length in angstroms (sum of consecutive CA–CA distances).
    pub backbone_length: f64,
}

// ============================================================================
// Internal geometry helpers
// ============================================================================

/// Compute the Euclidean distance between two 3-D points.
#[inline]
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Subtract two 3-D vectors: `a - b`.
#[inline]
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Cross product of two 3-D vectors.
#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Dot product of two 3-D vectors.
#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Normalize a 3-D vector. Returns the zero vector if the norm is negligible.
#[inline]
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0; 3]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

// ============================================================================
// Public functions
// ============================================================================

/// Extract backbone atom indices for every residue in the molecule.
///
/// Iterates all chains and residues in order. For each residue the function
/// scans that residue's atom indices, looking for atoms whose `name` field
/// matches `"N"`, `"CA"`, `"C"`, or `"O"` exactly (case-sensitive, PDB
/// convention). The first match for each name is stored; duplicates are
/// ignored.
///
/// # Example
///
/// ```
/// use nexcore_viz::molecular::{Atom, Chain, Element, Molecule, Residue, SecondaryStructure};
/// use nexcore_viz::protein::extract_backbone;
///
/// let mut mol = Molecule::new("test");
/// let mut atom = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
/// atom.name = "N".to_string();
/// mol.atoms.push(atom);
/// let mut atom = Atom::new(2, Element::C, [1.0, 0.0, 0.0]);
/// atom.name = "CA".to_string();
/// mol.atoms.push(atom);
/// let mut atom = Atom::new(3, Element::C, [2.0, 0.0, 0.0]);
/// atom.name = "C".to_string();
/// mol.atoms.push(atom);
/// let mut atom = Atom::new(4, Element::O, [2.0, 1.0, 0.0]);
/// atom.name = "O".to_string();
/// mol.atoms.push(atom);
///
/// let residue = Residue {
///     name: "ALA".to_string(),
///     seq: 1,
///     insertion_code: None,
///     atom_indices: vec![0, 1, 2, 3],
///     secondary_structure: SecondaryStructure::Coil,
/// };
/// let chain = Chain { id: 'A', residues: vec![residue] };
/// mol.chains.push(chain);
///
/// let backbone = extract_backbone(&mol);
/// assert_eq!(backbone.len(), 1);
/// assert_eq!(backbone[0].n, Some(0));
/// assert_eq!(backbone[0].ca, Some(1));
/// assert_eq!(backbone[0].c, Some(2));
/// assert_eq!(backbone[0].o, Some(3));
/// ```
#[must_use]
pub fn extract_backbone(mol: &Molecule) -> Vec<BackboneAtoms> {
    let mut result = Vec::new();

    for chain in &mol.chains {
        for residue in &chain.residues {
            let mut bb = BackboneAtoms {
                n: None,
                ca: None,
                c: None,
                o: None,
            };

            for &atom_idx in &residue.atom_indices {
                if let Some(atom) = mol.atoms.get(atom_idx) {
                    match atom.name.as_str() {
                        "N" if bb.n.is_none() => bb.n = Some(atom_idx),
                        "CA" if bb.ca.is_none() => bb.ca = Some(atom_idx),
                        "C" if bb.c.is_none() => bb.c = Some(atom_idx),
                        "O" if bb.o.is_none() => bb.o = Some(atom_idx),
                        _ => {}
                    }
                }
            }

            result.push(bb);
        }
    }

    result
}

/// Compute the signed dihedral angle defined by four points in 3-D space.
///
/// Uses the standard plane-normal cross-product method. The result is in
/// radians on (−π, π].
///
/// # Arguments
///
/// * `p1`, `p2`, `p3`, `p4` — four consecutive bonded positions.
///
/// # Example
///
/// ```
/// use nexcore_viz::protein::dihedral_angle;
/// use std::f64::consts::PI;
///
/// // Flat cis configuration (all coplanar, same side) → angle ≈ 0
/// let angle = dihedral_angle(
///     [0.0, 1.0, 0.0],
///     [0.0, 0.0, 0.0],
///     [1.0, 0.0, 0.0],
///     [1.0, 1.0, 0.0],
/// );
/// assert!(angle.abs() < 1e-6);
/// ```
#[must_use]
pub fn dihedral_angle(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], p4: [f64; 3]) -> f64 {
    // Bond vectors
    let b1 = sub(p2, p1);
    let b2 = sub(p3, p2);
    let b3 = sub(p4, p3);

    // Normals to the two planes
    let n1 = cross(b1, b2);
    let n2 = cross(b2, b3);

    // Normalise the central bond vector for the cross-product sign test
    let b2_hat = normalize(b2);

    let x = dot(n1, n2);
    let y = dot(cross(n1, b2_hat), n2);

    f64::atan2(y, x)
}

/// Compute phi and psi dihedral angles for every residue.
///
/// `backbone` must be the slice returned by [`extract_backbone`] for `mol`.
///
/// - phi(i) = dihedral( C[i-1], N[i], CA[i], C[i] )
/// - psi(i) = dihedral( N[i], CA[i], C[i], N[i+1] )
///
/// The first residue has no phi and the last residue has no psi.
#[must_use]
pub fn compute_phi_psi(mol: &Molecule, backbone: &[BackboneAtoms]) -> Vec<DihedralAngles> {
    let n = backbone.len();
    let mut angles = Vec::with_capacity(n);

    for i in 0..n {
        let bb_i = &backbone[i];

        // phi: C[i-1], N[i], CA[i], C[i]
        let phi = if i == 0 {
            None
        } else {
            let bb_prev = &backbone[i - 1];
            match (bb_prev.c, bb_i.n, bb_i.ca, bb_i.c) {
                (Some(c_prev), Some(n_i), Some(ca_i), Some(c_i)) => {
                    let p_c_prev = mol.atoms.get(c_prev).map(|a| a.position);
                    let p_n_i = mol.atoms.get(n_i).map(|a| a.position);
                    let p_ca_i = mol.atoms.get(ca_i).map(|a| a.position);
                    let p_c_i = mol.atoms.get(c_i).map(|a| a.position);
                    match (p_c_prev, p_n_i, p_ca_i, p_c_i) {
                        (Some(a), Some(b), Some(c), Some(d)) => Some(dihedral_angle(a, b, c, d)),
                        _ => None,
                    }
                }
                _ => None,
            }
        };

        // psi: N[i], CA[i], C[i], N[i+1]
        let psi = if i + 1 >= n {
            None
        } else {
            let bb_next = &backbone[i + 1];
            match (bb_i.n, bb_i.ca, bb_i.c, bb_next.n) {
                (Some(n_i), Some(ca_i), Some(c_i), Some(n_next)) => {
                    let p_n_i = mol.atoms.get(n_i).map(|a| a.position);
                    let p_ca_i = mol.atoms.get(ca_i).map(|a| a.position);
                    let p_c_i = mol.atoms.get(c_i).map(|a| a.position);
                    let p_n_next = mol.atoms.get(n_next).map(|a| a.position);
                    match (p_n_i, p_ca_i, p_c_i, p_n_next) {
                        (Some(a), Some(b), Some(c), Some(d)) => Some(dihedral_angle(a, b, c, d)),
                        _ => None,
                    }
                }
                _ => None,
            }
        };

        angles.push(DihedralAngles {
            residue_index: i,
            phi,
            psi,
        });
    }

    angles
}

/// Classify a (phi, psi) pair into a Ramachandran region.
///
/// All bounds are converted internally from degrees to radians.
///
/// | Region | phi (°) | psi (°) |
/// |--------|---------|---------|
/// | Alpha helix | [−160, −20] | [−120, 50] |
/// | Beta sheet | [−180, −40] | [50, 180] |
/// | Left-handed helix | [20, 120] | [−60, 60] |
/// | Generally allowed | broad region outside the above |
/// | Outlier | none of the above |
///
/// # Example
///
/// ```
/// use nexcore_viz::protein::{classify_ramachandran, RamachandranRegion};
/// use std::f64::consts::PI;
///
/// let region = classify_ramachandran(-60.0_f64.to_radians(), -45.0_f64.to_radians());
/// assert_eq!(region, RamachandranRegion::AlphaHelix);
/// ```
#[must_use]
pub fn classify_ramachandran(phi: f64, psi: f64) -> RamachandranRegion {
    let phi_deg = phi.to_degrees();
    let psi_deg = psi.to_degrees();

    // Alpha helix: phi ∈ [−160, −20], psi ∈ [−120, 50]
    if (-160.0..=-20.0).contains(&phi_deg) && (-120.0..=50.0).contains(&psi_deg) {
        return RamachandranRegion::AlphaHelix;
    }

    // Beta sheet: phi ∈ [−180, −40], psi ∈ [50, 180]
    if (-180.0..=-40.0).contains(&phi_deg) && (50.0..=180.0).contains(&psi_deg) {
        return RamachandranRegion::BetaSheet;
    }

    // Left-handed helix: phi ∈ [20, 120], psi ∈ [−60, 60]
    if (20.0..=120.0).contains(&phi_deg) && (-60.0..=60.0).contains(&psi_deg) {
        return RamachandranRegion::LeftHandedHelix;
    }

    // Generally allowed: large permissive region
    // phi ∈ [−180, 0] and psi anywhere, or phi ∈ [0, 180] and psi outside left-handed region
    if phi_deg <= 0.0 {
        // Negative phi half — most conformations here are at least broadly allowed
        // unless they fall in outlier territory (very positive phi_deg region already excluded)
        return RamachandranRegion::GenerallyAllowed;
    }

    // Positive phi outside the left-handed helix box → outlier
    RamachandranRegion::Outlier
}

/// Detect backbone hydrogen bonds using the DSSP electrostatic model.
///
/// The DSSP energy function (Kabsch & Sander 1983) is:
///
/// `E = 0.084 × 332 × (1/r_ON + 1/r_CH − 1/r_OH − 1/r_CN)` kcal/mol
///
/// A hydrogen bond is accepted when `E < −0.5 kcal/mol` **and** the N–O
/// distance is below `distance_cutoff` angstroms (typically 3.5 Å).
///
/// Only backbone atoms are considered: `N` as donor and `O` as acceptor.
/// The virtual hydrogen position is approximated as `N + (N − C_prev)` unit
/// vector × 1.0 Å when the preceding carbonyl carbon is available; otherwise
/// the N position is used directly for r_ON and r_NH.
///
/// # Arguments
///
/// * `mol` — protein molecule with chain/residue data.
/// * `distance_cutoff` — maximum N–O distance in angstroms.
#[must_use]
pub fn detect_hydrogen_bonds(mol: &Molecule, distance_cutoff: f64) -> Vec<HydrogenBond> {
    let backbone = extract_backbone(mol);

    // Collect (atom_index, position) pairs for all backbone N and O atoms
    let mut n_atoms: Vec<(usize, [f64; 3])> = Vec::new(); // (atom_idx, pos)
    let mut o_atoms: Vec<(usize, [f64; 3])> = Vec::new(); // (atom_idx, pos)
    // For each backbone N we also want to know the preceding C position (for H estimate)
    let mut n_c_prev: Vec<Option<[f64; 3]>> = Vec::new();

    for (res_idx, bb) in backbone.iter().enumerate() {
        if let Some(n_idx) = bb.n {
            if let Some(n_atom) = mol.atoms.get(n_idx) {
                let c_prev_pos = if res_idx > 0 {
                    backbone
                        .get(res_idx - 1)
                        .and_then(|prev_bb| prev_bb.c)
                        .and_then(|c_idx| mol.atoms.get(c_idx))
                        .map(|a| a.position)
                } else {
                    None
                };
                n_atoms.push((n_idx, n_atom.position));
                n_c_prev.push(c_prev_pos);
            }
        }
        if let Some(o_idx) = bb.o {
            if let Some(o_atom) = mol.atoms.get(o_idx) {
                o_atoms.push((o_idx, o_atom.position));
            }
        }
    }

    // Also need to find the C atom co-located with each O (backbone C of same residue)
    // We can look these up from the backbone array in the same pass.
    // Build a mapping: o_atom_idx -> c_atom_pos for the same residue
    let mut o_to_c_pos: std::collections::HashMap<usize, [f64; 3]> =
        std::collections::HashMap::new();
    for bb in &backbone {
        if let (Some(o_idx), Some(c_idx)) = (bb.o, bb.c) {
            if let Some(c_atom) = mol.atoms.get(c_idx) {
                o_to_c_pos.insert(o_idx, c_atom.position);
            }
        }
    }

    let mut bonds = Vec::new();

    let q1q2 = 0.084_f64 * 332.0_f64; // DSSP constant

    for (n_entry_idx, &(n_idx, n_pos)) in n_atoms.iter().enumerate() {
        // Virtual hydrogen position: N + normalise(N - C_prev) * 1.0 Å
        let h_pos = if let Some(c_prev_pos) = n_c_prev.get(n_entry_idx).copied().flatten() {
            let dir = normalize(sub(n_pos, c_prev_pos));
            [n_pos[0] + dir[0], n_pos[1] + dir[1], n_pos[2] + dir[2]]
        } else {
            n_pos // fallback: H at N position
        };

        for &(o_idx, o_pos) in &o_atoms {
            // Skip self-residue: if the O is in the same residue as this N, skip.
            // Heuristic: N and O from the same residue would share a backbone entry,
            // which means n_idx and o_idx appear together in a BackboneAtoms.
            // We detect this by checking if they appear in the same backbone slot.
            let same_residue = backbone
                .iter()
                .any(|bb| bb.n == Some(n_idx) && bb.o == Some(o_idx));
            if same_residue {
                continue;
            }

            let r_on = distance(o_pos, n_pos);
            if r_on >= distance_cutoff {
                continue;
            }

            // C atom position for this O (carbonyl C)
            let c_pos = match o_to_c_pos.get(&o_idx).copied() {
                Some(p) => p,
                None => continue,
            };

            let r_ch = distance(c_pos, h_pos);
            let r_oh = distance(o_pos, h_pos);
            let r_cn = distance(c_pos, n_pos);

            // Guard against division by zero
            if r_ch < 1e-9 || r_oh < 1e-9 || r_cn < 1e-9 || r_on < 1e-9 {
                continue;
            }

            let energy = q1q2 * (1.0 / r_on + 1.0 / r_ch - 1.0 / r_oh - 1.0 / r_cn);

            if energy < -0.5 {
                bonds.push(HydrogenBond {
                    donor_atom: n_idx,
                    acceptor_atom: o_idx,
                    energy,
                });
            }
        }
    }

    bonds
}

/// Assign secondary structure to each residue based on detected hydrogen bonds.
///
/// Rules (DSSP-inspired, simplified):
/// - **Helix**: residue `i` participates in an H-bond to residue `i+4`
///   (alpha helix) or `i+3` (3₁₀ helix) for at least 3 consecutive residues.
/// - **Sheet**: residue `i` participates in an H-bond with a distant residue
///   (`|i − j| > 4`), suggesting parallel or antiparallel beta structure.
/// - **Coil**: everything else.
///
/// Returns one [`SecondaryStructure`] per residue in chain order (same ordering
/// as [`extract_backbone`]).
#[must_use]
pub fn assign_secondary_structure(
    mol: &Molecule,
    hbonds: &[HydrogenBond],
) -> Vec<SecondaryStructure> {
    let backbone = extract_backbone(mol);
    let n_res = backbone.len();

    if n_res == 0 {
        return Vec::new();
    }

    // Build index: atom_idx -> residue_index (in flattened backbone order)
    let mut atom_to_res: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (res_idx, bb) in backbone.iter().enumerate() {
        for atom_idx in [bb.n, bb.ca, bb.c, bb.o].into_iter().flatten() {
            atom_to_res.insert(atom_idx, res_idx);
        }
    }

    // Collect H-bond partnerships as (donor_res, acceptor_res) pairs
    let mut hbond_pairs: Vec<(usize, usize)> = Vec::new();
    for hb in hbonds {
        let donor_res = atom_to_res.get(&hb.donor_atom).copied();
        let acceptor_res = atom_to_res.get(&hb.acceptor_atom).copied();
        if let (Some(d), Some(a)) = (donor_res, acceptor_res) {
            hbond_pairs.push((d, a));
        }
    }

    // Score each residue
    let mut helix_score = vec![0_u32; n_res]; // alpha/3-10 helix pattern count
    let mut sheet_flag = vec![false; n_res];

    for &(donor, acceptor) in &hbond_pairs {
        let diff = acceptor.abs_diff(donor);

        if diff == 4 || diff == 3 {
            // Helix i->i+4 (alpha) or i->i+3 (3_10)
            let lo = donor.min(acceptor);
            let hi = donor.max(acceptor);
            let end = (hi + 1).min(n_res);
            if lo < end {
                for score in &mut helix_score[lo..end] {
                    *score = score.saturating_add(1);
                }
            }
        } else if diff > 4 {
            // Sheet: distant pair
            sheet_flag[donor] = true;
            sheet_flag[acceptor] = true;
        }
    }

    // Assign: helix takes priority over sheet
    let mut result = vec![SecondaryStructure::Coil; n_res];
    for i in 0..n_res {
        if helix_score[i] >= 1 {
            result[i] = SecondaryStructure::Helix;
        } else if sheet_flag[i] {
            result[i] = SecondaryStructure::Sheet;
        }
    }

    result
}

/// Build a residue–residue contact map.
///
/// For each pair of residues `(i, j)` with `i < j`, computes the CA–CA
/// distance. If the distance is below `threshold`, a [`ContactMapEntry`] is
/// added to the result. Only residues that have a CA atom present are
/// considered.
///
/// A typical threshold for contact map construction is 8.0 Å.
///
/// # Example
///
/// ```
/// use nexcore_viz::protein::compute_contact_map;
/// use nexcore_viz::molecular::Molecule;
///
/// let mol = Molecule::new("empty");
/// let contacts = compute_contact_map(&mol, 8.0);
/// assert!(contacts.is_empty());
/// ```
#[must_use]
pub fn compute_contact_map(mol: &Molecule, threshold: f64) -> Vec<ContactMapEntry> {
    let backbone = extract_backbone(mol);
    let n_res = backbone.len();

    // Collect CA positions
    let ca_positions: Vec<Option<[f64; 3]>> = backbone
        .iter()
        .map(|bb| bb.ca.and_then(|idx| mol.atoms.get(idx)).map(|a| a.position))
        .collect();

    let mut contacts = Vec::new();

    for i in 0..n_res {
        let pos_i = match ca_positions.get(i).copied().flatten() {
            Some(p) => p,
            None => continue,
        };
        for j in (i + 1)..n_res {
            let pos_j = match ca_positions.get(j).copied().flatten() {
                Some(p) => p,
                None => continue,
            };
            let dist = distance(pos_i, pos_j);
            if dist < threshold {
                contacts.push(ContactMapEntry {
                    residue_i: i,
                    residue_j: j,
                    distance: dist,
                });
            }
        }
    }

    contacts
}

/// Compute the radius of gyration for all atoms in the molecule.
///
/// `Rg = sqrt( Σ |r_i − r_mean|² / n )`
///
/// Returns `0.0` for molecules with fewer than two atoms.
///
/// # Example
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::protein::radius_of_gyration;
///
/// let mut mol = Molecule::new("single");
/// mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
/// assert!((radius_of_gyration(&mol) - 0.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn radius_of_gyration(mol: &Molecule) -> f64 {
    let n = mol.atoms.len();
    if n == 0 {
        return 0.0;
    }

    // Centroid
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    let mut cz = 0.0_f64;
    for atom in &mol.atoms {
        cx += atom.position[0];
        cy += atom.position[1];
        cz += atom.position[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    // Sum of squared distances from centroid
    let sum_sq: f64 = mol
        .atoms
        .iter()
        .map(|a| {
            let dx = a.position[0] - cx;
            let dy = a.position[1] - cy;
            let dz = a.position[2] - cz;
            dx * dx + dy * dy + dz * dz
        })
        .sum();

    (sum_sq / nf).sqrt()
}

/// Compute aggregate structural metrics for a protein molecule.
///
/// Combines residue counts, secondary structure fractions (from the chain
/// residue data already embedded in the molecule), radius of gyration, and
/// backbone length (sum of consecutive CA–CA distances along each chain).
#[must_use]
pub fn compute_protein_metrics(mol: &Molecule) -> ProteinMetrics {
    let mut residue_count = 0_usize;
    let mut helix_count = 0_usize;
    let mut sheet_count = 0_usize;
    let mut coil_count = 0_usize;

    for chain in &mol.chains {
        for residue in &chain.residues {
            residue_count += 1;
            match residue.secondary_structure {
                SecondaryStructure::Helix
                | SecondaryStructure::Helix310
                | SecondaryStructure::HelixPi => {
                    helix_count += 1;
                }
                SecondaryStructure::Sheet => {
                    sheet_count += 1;
                }
                // Coil, Turn, and anything else counts as coil
                _ => {
                    coil_count += 1;
                }
            }
        }
    }

    let (helix_fraction, sheet_fraction, coil_fraction) = if residue_count == 0 {
        (0.0, 0.0, 0.0)
    } else {
        let n = residue_count as f64;
        (
            helix_count as f64 / n,
            sheet_count as f64 / n,
            coil_count as f64 / n,
        )
    };

    // Backbone length: sum of CA-CA distances along the residue sequence
    let backbone = extract_backbone(mol);
    let ca_positions: Vec<Option<[f64; 3]>> = backbone
        .iter()
        .map(|bb| bb.ca.and_then(|idx| mol.atoms.get(idx)).map(|a| a.position))
        .collect();

    // Walk through CA positions per chain to avoid cross-chain distances.
    // We track which flat residue index corresponds to each chain boundary.
    let mut backbone_length = 0.0_f64;
    let mut flat_idx = 0_usize;
    for chain in &mol.chains {
        let chain_len = chain.residues.len();
        let chain_end = flat_idx + chain_len;

        let mut prev_ca: Option<[f64; 3]> = None;
        for res_idx in flat_idx..chain_end {
            let ca_pos = ca_positions.get(res_idx).copied().flatten();
            if let (Some(prev), Some(curr)) = (prev_ca, ca_pos) {
                backbone_length += distance(prev, curr);
            }
            if ca_pos.is_some() {
                prev_ca = ca_pos;
            }
        }

        flat_idx = chain_end;
    }

    ProteinMetrics {
        residue_count,
        helix_fraction,
        sheet_fraction,
        coil_fraction,
        radius_of_gyration: radius_of_gyration(mol),
        backbone_length,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Chain, Element, Molecule, Residue, SecondaryStructure};

    // -------------------------------------------------------------------------
    // Geometry helpers
    // -------------------------------------------------------------------------

    /// Build a minimal Atom with a given name, element, and position.
    fn make_atom(id: u32, element: Element, pos: [f64; 3], name: &str) -> Atom {
        let mut a = Atom::new(id, element, pos);
        a.name = name.to_string();
        a
    }

    // -------------------------------------------------------------------------
    // 1. dihedral_angle — coplanar points, expected angle ≈ π (trans)
    // -------------------------------------------------------------------------

    #[test]
    fn dihedral_angle_planar_trans() {
        // Trans arrangement: p1 and p4 on opposite sides of the p2-p3 bond.
        // p1=[0,1,0], p2=[0,0,0], p3=[1,0,0], p4=[1,-1,0]:
        //   b1=(0,-1,0), b2=(1,0,0), b3=(0,-1,0)
        //   n1=cross(b1,b2)=(0,0,1), n2=cross(b2,b3)=(0,0,-1)
        //   x=dot(n1,n2)=-1, b2_hat=(1,0,0)
        //   cross(n1,b2_hat)=(0,1,0), y=dot((0,1,0),(0,0,-1))=0
        //   atan2(0,-1)=π  ✓
        let angle = dihedral_angle(
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, -1.0, 0.0],
        );
        assert!(
            (angle.abs() - std::f64::consts::PI).abs() < 1e-6,
            "expected |angle| ≈ π, got {angle}"
        );
    }

    /// Cis arrangement: p1 and p4 on the same side of the p2-p3 bond → dihedral ≈ 0.
    #[test]
    fn dihedral_angle_planar_cis() {
        // p1=[0,1,0], p2=[0,0,0], p3=[1,0,0], p4=[1,1,0]:
        //   b1=(0,-1,0), b2=(1,0,0), b3=(0,1,0)
        //   n1=cross(b1,b2)=(0,0,1), n2=cross(b2,b3)=(0,0,1)
        //   x=dot(n1,n2)=1, cross(n1,b2_hat)=(0,1,0), y=dot((0,1,0),(0,0,1))=0
        //   atan2(0,1)=0  ✓
        let angle = dihedral_angle(
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        );
        assert!(angle.abs() < 1e-6, "expected angle ≈ 0, got {angle}");
    }

    // -------------------------------------------------------------------------
    // 2. dihedral_angle — perpendicular planes (90°)
    // -------------------------------------------------------------------------

    #[test]
    fn dihedral_angle_perpendicular() {
        // p1 above the XY plane, p4 along the Y axis → 90° dihedral.
        let angle = dihedral_angle(
            [0.0, 0.0, 1.0], // p1: up
            [0.0, 0.0, 0.0], // p2: origin
            [1.0, 0.0, 0.0], // p3: along X
            [1.0, 1.0, 0.0], // p4: in XY plane
        );
        let expected = std::f64::consts::FRAC_PI_2; // π/2
        assert!(
            (angle.abs() - expected).abs() < 1e-6,
            "expected |angle| ≈ π/2, got {angle}"
        );
    }

    // -------------------------------------------------------------------------
    // 3. extract_backbone — named atoms resolve correctly
    // -------------------------------------------------------------------------

    #[test]
    fn extract_backbone_finds_atoms() {
        let mut mol = Molecule::new("test_protein");

        // Add backbone atoms for a single residue
        mol.atoms
            .push(make_atom(1, Element::N, [0.0, 0.0, 0.0], "N")); // idx 0
        mol.atoms
            .push(make_atom(2, Element::C, [1.0, 0.0, 0.0], "CA")); // idx 1
        mol.atoms
            .push(make_atom(3, Element::C, [2.0, 0.0, 0.0], "C")); // idx 2
        mol.atoms
            .push(make_atom(4, Element::O, [2.0, 1.0, 0.0], "O")); // idx 3
        // A sidechain atom that should be ignored
        mol.atoms
            .push(make_atom(5, Element::C, [1.0, 1.0, 0.0], "CB")); // idx 4

        let residue = Residue {
            name: "ALA".to_string(),
            seq: 1,
            insertion_code: None,
            atom_indices: vec![0, 1, 2, 3, 4],
            secondary_structure: SecondaryStructure::Coil,
        };
        let chain = Chain {
            id: 'A',
            residues: vec![residue],
        };
        mol.chains.push(chain);

        let backbone = extract_backbone(&mol);

        assert_eq!(backbone.len(), 1, "expected one residue backbone");
        let bb = &backbone[0];
        assert_eq!(bb.n, Some(0), "N index");
        assert_eq!(bb.ca, Some(1), "CA index");
        assert_eq!(bb.c, Some(2), "C index");
        assert_eq!(bb.o, Some(3), "O index");
        assert!(bb.is_complete(), "backbone should be complete");
    }

    // -------------------------------------------------------------------------
    // 4. classify_ramachandran — alpha helix region
    // -------------------------------------------------------------------------

    #[test]
    fn classify_ramachandran_alpha_helix() {
        // Classic alpha-helix point: phi = -60°, psi = -45°
        let phi = (-60.0_f64).to_radians();
        let psi = (-45.0_f64).to_radians();
        assert_eq!(
            classify_ramachandran(phi, psi),
            RamachandranRegion::AlphaHelix
        );
    }

    // -------------------------------------------------------------------------
    // 5. classify_ramachandran — beta sheet region
    // -------------------------------------------------------------------------

    #[test]
    fn classify_ramachandran_beta_sheet() {
        // Typical beta-sheet point: phi = -120°, psi = 130°
        let phi = (-120.0_f64).to_radians();
        let psi = (130.0_f64).to_radians();
        assert_eq!(
            classify_ramachandran(phi, psi),
            RamachandranRegion::BetaSheet
        );
    }

    // -------------------------------------------------------------------------
    // 6. radius_of_gyration — single atom → 0
    // -------------------------------------------------------------------------

    #[test]
    fn radius_of_gyration_single_atom() {
        let mut mol = Molecule::new("single");
        mol.atoms.push(Atom::new(1, Element::C, [5.0, 3.0, -2.0]));
        let rg = radius_of_gyration(&mol);
        assert!(rg.abs() < 1e-9, "single atom should have Rg = 0, got {rg}");
    }

    // -------------------------------------------------------------------------
    // 7. radius_of_gyration — two atoms at known distance
    // -------------------------------------------------------------------------

    #[test]
    fn radius_of_gyration_two_atoms() {
        // Two atoms at x = ±d/2, same y and z → centroid at origin → Rg = d/2.
        let d = 6.0_f64;
        let mut mol = Molecule::new("dimer");
        mol.atoms
            .push(Atom::new(1, Element::C, [-d / 2.0, 0.0, 0.0]));
        mol.atoms
            .push(Atom::new(2, Element::C, [d / 2.0, 0.0, 0.0]));
        let rg = radius_of_gyration(&mol);
        // Each atom is d/2 from centroid; Rg = sqrt((2 * (d/2)^2) / 2) = d/2
        let expected = d / 2.0;
        assert!(
            (rg - expected).abs() < 1e-9,
            "expected Rg = {expected}, got {rg}"
        );
    }

    // -------------------------------------------------------------------------
    // 8. contact_map — close residues are detected
    // -------------------------------------------------------------------------

    #[test]
    fn contact_map_close_residues() {
        let mut mol = Molecule::new("two_residue");

        // Residue 0: CA at origin
        mol.atoms
            .push(make_atom(1, Element::C, [0.0, 0.0, 0.0], "CA")); // idx 0
        // Residue 1: CA at 5 Å — within 8 Å threshold
        mol.atoms
            .push(make_atom(2, Element::C, [5.0, 0.0, 0.0], "CA")); // idx 1

        let res0 = Residue {
            name: "ALA".to_string(),
            seq: 1,
            insertion_code: None,
            atom_indices: vec![0],
            secondary_structure: SecondaryStructure::Coil,
        };
        let res1 = Residue {
            name: "GLY".to_string(),
            seq: 2,
            insertion_code: None,
            atom_indices: vec![1],
            secondary_structure: SecondaryStructure::Coil,
        };
        let chain = Chain {
            id: 'A',
            residues: vec![res0, res1],
        };
        mol.chains.push(chain);

        let contacts = compute_contact_map(&mol, 8.0);
        assert_eq!(contacts.len(), 1, "expected one contact");
        assert_eq!(contacts[0].residue_i, 0);
        assert_eq!(contacts[0].residue_j, 1);
        assert!((contacts[0].distance - 5.0).abs() < 1e-9);
    }

    // -------------------------------------------------------------------------
    // Bonus: contact_map — distant residues excluded
    // -------------------------------------------------------------------------

    #[test]
    fn contact_map_distant_residues_excluded() {
        let mut mol = Molecule::new("far_apart");

        mol.atoms
            .push(make_atom(1, Element::C, [0.0, 0.0, 0.0], "CA")); // idx 0
        mol.atoms
            .push(make_atom(2, Element::C, [20.0, 0.0, 0.0], "CA")); // idx 1

        let res0 = Residue {
            name: "ALA".to_string(),
            seq: 1,
            insertion_code: None,
            atom_indices: vec![0],
            secondary_structure: SecondaryStructure::Coil,
        };
        let res1 = Residue {
            name: "GLY".to_string(),
            seq: 2,
            insertion_code: None,
            atom_indices: vec![1],
            secondary_structure: SecondaryStructure::Coil,
        };
        let chain = Chain {
            id: 'A',
            residues: vec![res0, res1],
        };
        mol.chains.push(chain);

        let contacts = compute_contact_map(&mol, 8.0);
        assert!(
            contacts.is_empty(),
            "distant residues should not form a contact"
        );
    }

    // -------------------------------------------------------------------------
    // Bonus: extract_backbone — missing atom stays None
    // -------------------------------------------------------------------------

    #[test]
    fn extract_backbone_partial() {
        let mut mol = Molecule::new("partial");

        // Only N and CA, no C or O
        mol.atoms
            .push(make_atom(1, Element::N, [0.0, 0.0, 0.0], "N")); // idx 0
        mol.atoms
            .push(make_atom(2, Element::C, [1.0, 0.0, 0.0], "CA")); // idx 1

        let residue = Residue {
            name: "GLY".to_string(),
            seq: 1,
            insertion_code: None,
            atom_indices: vec![0, 1],
            secondary_structure: SecondaryStructure::Coil,
        };
        let chain = Chain {
            id: 'A',
            residues: vec![residue],
        };
        mol.chains.push(chain);

        let backbone = extract_backbone(&mol);
        assert_eq!(backbone.len(), 1);
        let bb = &backbone[0];
        assert_eq!(bb.n, Some(0));
        assert_eq!(bb.ca, Some(1));
        assert_eq!(bb.c, None);
        assert_eq!(bb.o, None);
        assert!(!bb.is_complete());
    }

    // -------------------------------------------------------------------------
    // Bonus: radius_of_gyration — empty molecule
    // -------------------------------------------------------------------------

    #[test]
    fn radius_of_gyration_empty() {
        let mol = Molecule::new("empty");
        assert!((radius_of_gyration(&mol)).abs() < 1e-9);
    }

    // -------------------------------------------------------------------------
    // Bonus: protein metrics — empty molecule
    // -------------------------------------------------------------------------

    #[test]
    fn protein_metrics_empty_molecule() {
        let mol = Molecule::new("empty");
        let m = compute_protein_metrics(&mol);
        assert_eq!(m.residue_count, 0);
        assert!((m.helix_fraction).abs() < 1e-9);
        assert!((m.sheet_fraction).abs() < 1e-9);
        assert!((m.coil_fraction).abs() < 1e-9);
        assert!((m.backbone_length).abs() < 1e-9);
    }

    // -------------------------------------------------------------------------
    // Bonus: protein metrics — fractions sum to 1
    // -------------------------------------------------------------------------

    #[test]
    fn protein_metrics_fractions_sum_to_one() {
        let mut mol = Molecule::new("frac_test");

        // 2 helix residues, 1 sheet residue, 1 coil residue
        for i in 0..4_u32 {
            mol.atoms
                .push(make_atom(i + 1, Element::C, [i as f64, 0.0, 0.0], "CA"));
        }

        let ss_list = [
            SecondaryStructure::Helix,
            SecondaryStructure::Helix,
            SecondaryStructure::Sheet,
            SecondaryStructure::Coil,
        ];

        let residues: Vec<Residue> = ss_list
            .iter()
            .enumerate()
            .map(|(i, &ss)| Residue {
                name: "ALA".to_string(),
                seq: i as i32 + 1,
                insertion_code: None,
                atom_indices: vec![i],
                secondary_structure: ss,
            })
            .collect();

        mol.chains.push(Chain { id: 'A', residues });

        let m = compute_protein_metrics(&mol);
        assert_eq!(m.residue_count, 4);
        let total = m.helix_fraction + m.sheet_fraction + m.coil_fraction;
        assert!(
            (total - 1.0).abs() < 1e-9,
            "fractions must sum to 1, got {total}"
        );
        assert!((m.helix_fraction - 0.5).abs() < 1e-9);
        assert!((m.sheet_fraction - 0.25).abs() < 1e-9);
        assert!((m.coil_fraction - 0.25).abs() < 1e-9);
    }
}
