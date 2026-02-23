//! Molecular interaction detection for protein-ligand and protein-protein visualization.
//!
//! Detects and classifies non-covalent interactions: hydrogen bonds, salt bridges,
//! hydrophobic contacts, pi-stacking, and cation-pi interactions. Each interaction
//! type has geometry-aware detection with configurable distance/angle cutoffs.
//!
//! Primitive formula: interaction = κ(donor, acceptor) × N(distance) × ∂(angle_cutoff)
//!   κ: Comparison (donor vs acceptor identity test)
//!   N: Quantity (distance measurement)
//!   ∂: Boundary (cutoff threshold gate)

use serde::{Deserialize, Serialize};

use crate::molecular::{Atom, BondOrder, Element, Molecule};

// ============================================================================
// Error type
// ============================================================================

/// Errors that can arise during interaction detection.
#[derive(Debug, Clone)]
pub enum InteractionError {
    /// Atom index referenced in a bond does not exist in the atom list.
    InvalidAtomIndex(usize),
    /// Ring detection failed because there are too few atoms.
    InsufficientAtoms,
}

impl std::fmt::Display for InteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAtomIndex(i) => write!(f, "atom index {i} is out of bounds"),
            Self::InsufficientAtoms => write!(f, "insufficient atoms to detect rings"),
        }
    }
}

impl std::error::Error for InteractionError {}

// ============================================================================
// InteractionType
// ============================================================================

/// Classification of a detected non-covalent interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionType {
    /// N-H···O or N-H···N hydrogen bond (2.5–3.5 Å, angle > 120°).
    HydrogenBond,
    /// Electrostatic attraction between oppositely charged residues (< 4.0 Å).
    SaltBridge,
    /// Hydrophobic atom contacts between non-polar carbons (< 4.5 Å).
    HydrophobicContact,
    /// Aromatic ring stacking: parallel (face-to-face) or T-shaped (edge-to-face).
    PiStacking,
    /// Interaction between a cation and an aromatic pi system (< 6.0 Å).
    CationPi,
    /// Halogen (Cl/Br/I) acting as electrophilic donor to an O/N/S acceptor.
    HalogenBond,
    /// Metal ion coordinated by N/O/S donors (< 2.8 Å).
    MetalCoordination,
}

// ============================================================================
// Interaction
// ============================================================================

/// A single detected non-covalent interaction between two atom indices.
///
/// `atom_i` and `atom_j` are 0-based indices into `Molecule.atoms`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// Interaction classification.
    pub interaction_type: InteractionType,
    /// First atom (0-based index into `Molecule.atoms`).
    pub atom_i: usize,
    /// Second atom (0-based index into `Molecule.atoms`).
    pub atom_j: usize,
    /// Inter-atomic distance in angstroms.
    pub distance: f64,
    /// Relevant geometry angle in degrees, if applicable (e.g. D-H···A for H-bonds).
    pub angle: Option<f64>,
    /// Estimated free energy contribution in kcal/mol (negative = favorable).
    pub energy_estimate: f64,
    /// Detection confidence in [0.0, 1.0] based on geometry quality.
    pub confidence: f64,
}

// ============================================================================
// InteractionCutoffs
// ============================================================================

/// Configurable distance and angle thresholds for interaction detection.
///
/// All distances are in angstroms; all angles are in degrees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCutoffs {
    /// Maximum donor–acceptor distance for hydrogen bonds (default 3.5 Å).
    pub hbond_distance: f64,
    /// Minimum D-H···A angle for hydrogen bonds (default 120°).
    pub hbond_angle_min: f64,
    /// Maximum centre–centre distance for salt bridges (default 4.0 Å).
    pub salt_bridge_distance: f64,
    /// Maximum atom–atom distance for hydrophobic contacts (default 4.5 Å).
    pub hydrophobic_distance: f64,
    /// Maximum ring-centre–ring-centre distance for pi stacking (default 5.5 Å).
    pub pi_stack_distance: f64,
    /// Maximum inter-plane angle (degrees) for parallel pi stacking (default 30°).
    pub pi_stack_angle_max: f64,
    /// Maximum cation–ring-centre distance for cation-pi (default 6.0 Å).
    pub cation_pi_distance: f64,
    /// Maximum halogen–acceptor distance for halogen bonds (default 3.5 Å).
    pub halogen_bond_distance: f64,
    /// Maximum metal–ligand distance for metal coordination (default 2.8 Å).
    pub metal_coord_distance: f64,
}

impl Default for InteractionCutoffs {
    fn default() -> Self {
        Self {
            hbond_distance: 3.5,
            hbond_angle_min: 120.0,
            salt_bridge_distance: 4.0,
            hydrophobic_distance: 4.5,
            pi_stack_distance: 5.5,
            pi_stack_angle_max: 30.0,
            cation_pi_distance: 6.0,
            halogen_bond_distance: 3.5,
            metal_coord_distance: 2.8,
        }
    }
}

// ============================================================================
// InteractionSummary
// ============================================================================

/// Aggregated results from running all interaction detectors on a molecule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSummary {
    /// All detected interactions.
    pub interactions: Vec<Interaction>,
    /// Number of detected hydrogen bonds.
    pub hbond_count: usize,
    /// Number of detected salt bridges.
    pub salt_bridge_count: usize,
    /// Number of detected hydrophobic contacts.
    pub hydrophobic_count: usize,
    /// Number of detected pi-stacking interactions.
    pub pi_stack_count: usize,
    /// Number of detected cation-pi interactions.
    pub cation_pi_count: usize,
    /// Number of detected halogen bonds.
    pub halogen_bond_count: usize,
    /// Number of detected metal coordination bonds.
    pub metal_coord_count: usize,
    /// Sum of all `energy_estimate` values (kcal/mol).
    pub total_energy: f64,
}

// ============================================================================
// RingInfo
// ============================================================================

/// An aromatic ring detected in a molecule, with its geometric descriptor.
#[derive(Debug, Clone)]
pub struct RingInfo {
    /// 0-based atom indices forming the ring, in traversal order.
    pub atom_indices: Vec<usize>,
    /// Centroid of the ring atoms [x, y, z] in angstroms.
    pub center: [f64; 3],
    /// Unit normal vector of the ring plane [x, y, z].
    pub normal: [f64; 3],
}

// ============================================================================
// Private geometry helpers
// ============================================================================

/// Euclidean distance between two 3D points.
fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Angle at vertex `b` formed by points `a–b–c`, returned in degrees.
///
/// Returns 0.0 if either arm has zero length (degenerate).
fn angle_between(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> f64 {
    let ba = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];

    let dot = ba[0] * bc[0] + ba[1] * bc[1] + ba[2] * bc[2];
    let len_ba = (ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2]).sqrt();
    let len_bc = (bc[0] * bc[0] + bc[1] * bc[1] + bc[2] * bc[2]).sqrt();

    let denom = len_ba * len_bc;
    if denom < 1e-12 {
        return 0.0;
    }

    // Clamp to [-1, 1] to guard against floating-point overshoot.
    let cos_theta = (dot / denom).clamp(-1.0, 1.0);
    cos_theta.acos().to_degrees()
}

/// Dot product of two 3D vectors.
#[inline]
fn dot3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product of two 3D vectors.
#[inline]
fn cross3(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Normalize a 3D vector; returns zero vector if near-degenerate.
#[inline]
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

// ============================================================================
// Private atom classification helpers
// ============================================================================

/// True if the atom is a hydrogen-bond donor (N or O with potential H neighbours).
///
/// In practice we use the element alone because explicit H atoms are often
/// absent in PDB/SDF structures. An N or O is considered a donor.
fn is_hbond_donor(atom: &Atom) -> bool {
    matches!(atom.element, Element::N | Element::O)
}

/// True if the atom can act as a hydrogen-bond acceptor.
///
/// N, O, and S all have lone pairs.
fn is_hbond_acceptor(atom: &Atom) -> bool {
    matches!(atom.element, Element::N | Element::O | Element::S)
}

/// True if the atom carries a formal positive charge, or is a named residue atom
/// known to be permanently cationic (Lys NZ, Arg NH*/NE, His ND1/NE2 as proxy).
fn is_charged_positive(atom: &Atom) -> bool {
    if atom.charge > 0 {
        return true;
    }
    // Residue-name + atom-name heuristics for positively charged side chains.
    let res = atom.residue_name.as_deref().unwrap_or("");
    let name = atom.name.trim();
    match res {
        "LYS" => name == "NZ",
        "ARG" => matches!(name, "NH1" | "NH2" | "NE"),
        "HIS" => matches!(name, "ND1" | "NE2"),
        _ => false,
    }
}

/// True if the atom carries a formal negative charge, or is a named residue atom
/// known to be permanently anionic (Asp OD*, Glu OE*).
fn is_charged_negative(atom: &Atom) -> bool {
    if atom.charge < 0 {
        return true;
    }
    let res = atom.residue_name.as_deref().unwrap_or("");
    let name = atom.name.trim();
    match res {
        "ASP" => matches!(name, "OD1" | "OD2"),
        "GLU" => matches!(name, "OE1" | "OE2"),
        _ => false,
    }
}

/// True if the atom is hydrophobic: a carbon in a hydrophobic residue, or any
/// carbon that has no adjacent heteroatoms (approximated by element alone here).
fn is_hydrophobic(atom: &Atom) -> bool {
    if atom.element != Element::C {
        return false;
    }
    // If residue information is available, use residue identity.
    if let Some(res) = atom.residue_name.as_deref() {
        return matches!(
            res,
            "ALA" | "VAL" | "LEU" | "ILE" | "PHE" | "TRP" | "MET" | "PRO"
        );
    }
    // No residue info (small molecule context): treat all carbons as hydrophobic.
    true
}

/// True if the element is a halogen capable of halogen bonding.
fn is_halogen(element: Element) -> bool {
    matches!(element, Element::F | Element::Cl | Element::Br | Element::I)
}

/// True if the element is a biologically relevant metal ion.
fn is_metal(element: Element) -> bool {
    matches!(
        element,
        Element::Fe | Element::Zn | Element::Ca | Element::Mg | Element::Na | Element::K
    )
}

// ============================================================================
// Ring detection
// ============================================================================

/// Centroid of the atoms at the given indices.
fn ring_center(atoms: &[Atom], indices: &[usize]) -> [f64; 3] {
    let n = indices.len();
    if n == 0 {
        return [0.0; 3];
    }
    let mut center = [0.0_f64; 3];
    let mut count = 0usize;
    for &idx in indices {
        if let Some(atom) = atoms.get(idx) {
            for (c, &p) in center.iter_mut().zip(atom.position.iter()) {
                *c += p;
            }
            count += 1;
        }
    }
    if count == 0 {
        return [0.0; 3];
    }
    let n_f = count as f64;
    [center[0] / n_f, center[1] / n_f, center[2] / n_f]
}

/// Normal vector to the ring plane using the cross product of two edge vectors.
fn ring_normal(atoms: &[Atom], indices: &[usize]) -> [f64; 3] {
    if indices.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let get_pos = |idx: usize| -> Option<[f64; 3]> { atoms.get(idx).map(|a| a.position) };

    let p0 = match get_pos(indices[0]) {
        Some(p) => p,
        None => return [0.0, 0.0, 1.0],
    };
    let p1 = match get_pos(indices[1]) {
        Some(p) => p,
        None => return [0.0, 0.0, 1.0],
    };
    let p2 = match get_pos(indices[2]) {
        Some(p) => p,
        None => return [0.0, 0.0, 1.0],
    };

    let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    normalize(cross3(&v1, &v2))
}

/// Detect aromatic rings (5- and 6-membered) containing at least one aromatic bond.
///
/// Uses a simple DFS ring-closure search limited to rings of size 5 or 6.
pub fn detect_rings(mol: &Molecule) -> Vec<RingInfo> {
    // Build adjacency list restricted to aromatic bonds.
    let n = mol.atoms.len();
    let mut aromatic_adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    // Also build full adjacency to allow ring search.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    let mut has_aromatic_bond: Vec<bool> = vec![false; n];

    for bond in &mol.bonds {
        let a1 = bond.atom1;
        let a2 = bond.atom2;
        if a1 >= n || a2 >= n {
            continue;
        }
        adj[a1].push(a2);
        adj[a2].push(a1);
        if bond.order == BondOrder::Aromatic {
            aromatic_adj[a1].push(a2);
            aromatic_adj[a2].push(a1);
            has_aromatic_bond[a1] = true;
            has_aromatic_bond[a2] = true;
        }
    }

    // DFS to find all simple cycles of length 5–6 among atoms that have at
    // least one aromatic bond.
    let mut rings: Vec<RingInfo> = Vec::new();
    // Track which canonical ring atom-sets we have already emitted.
    let mut seen: std::collections::HashSet<Vec<usize>> = std::collections::HashSet::new();

    for (start, has_aromatic) in has_aromatic_bond.iter().enumerate() {
        if !has_aromatic {
            continue;
        }
        // DFS: path = current path, depth limit = 6.
        let mut stack: Vec<(usize, Vec<usize>)> = vec![(start, vec![start])];
        while let Some((node, path)) = stack.pop() {
            for &next in &adj[node] {
                if path.len() >= 3 && next == start {
                    // Closed a ring.
                    let ring_len = path.len();
                    if ring_len == 5 || ring_len == 6 {
                        // Require at least one aromatic bond in the ring.
                        let has_ar = path.windows(2).any(|w| {
                            aromatic_adj
                                .get(w[0])
                                .map(|adj_list| adj_list.contains(&w[1]))
                                .unwrap_or(false)
                        }) || path
                            .last()
                            .and_then(|&last| aromatic_adj.get(last))
                            .map(|adj_list| adj_list.contains(&start))
                            .unwrap_or(false);

                        if has_ar {
                            let mut key = path.clone();
                            key.sort_unstable();
                            if seen.insert(key) {
                                let center = ring_center(&mol.atoms, &path);
                                let normal = ring_normal(&mol.atoms, &path);
                                rings.push(RingInfo {
                                    atom_indices: path.clone(),
                                    center,
                                    normal,
                                });
                            }
                        }
                    }
                    // Do not extend further from a closed ring.
                    continue;
                }
                // Only extend if next is not already in the path and ring is still short enough.
                if path.len() < 6 && !path.contains(&next) {
                    let mut new_path = path.clone();
                    new_path.push(next);
                    stack.push((next, new_path));
                }
            }
        }
    }

    rings
}

// ============================================================================
// Public detection functions
// ============================================================================

/// Detect hydrogen bonds in a molecule.
///
/// Finds donor-acceptor atom pairs within `cutoffs.hbond_distance` where the
/// donor and acceptor are different atoms and both are N, O, or S. When a
/// bridging hydrogen atom is present, the D-H···A angle must exceed
/// `cutoffs.hbond_angle_min`. Energy estimated as:
/// `E ≈ -2.0 × (1.0 − (d − 2.8) / (cutoff − 2.8))`.
///
/// # Example
///
/// ```
/// use nexcore_viz::interaction::{detect_hydrogen_bonds, InteractionCutoffs};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("test");
/// mol.atoms.push(Atom::new(1, Element::N, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::O, [3.0, 0.0, 0.0]));
/// let cutoffs = InteractionCutoffs::default();
/// let hbonds = detect_hydrogen_bonds(&mol, &cutoffs);
/// assert!(!hbonds.is_empty());
/// ```
#[must_use]
pub fn detect_hydrogen_bonds(mol: &Molecule, cutoffs: &InteractionCutoffs) -> Vec<Interaction> {
    let mut result = Vec::new();
    let atoms = &mol.atoms;
    let n = atoms.len();

    // Build hydrogen-neighbour map so we can check D-H···A angles.
    // Maps atom index → list of H atom indices bonded to it.
    let mut h_neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for bond in &mol.bonds {
        let a1 = bond.atom1;
        let a2 = bond.atom2;
        if a1 >= n || a2 >= n {
            continue;
        }
        if atoms.get(a2).map(|a| a.element) == Some(Element::H) {
            h_neighbors[a1].push(a2);
        }
        if atoms.get(a1).map(|a| a.element) == Some(Element::H) {
            h_neighbors[a2].push(a1);
        }
    }

    for (i, h_neigh) in h_neighbors.iter().enumerate() {
        let donor = match atoms.get(i) {
            Some(a) => a,
            None => continue,
        };
        if !is_hbond_donor(donor) {
            continue;
        }
        for j in 0..n {
            if i == j {
                continue;
            }
            let acceptor = match atoms.get(j) {
                Some(a) => a,
                None => continue,
            };
            if !is_hbond_acceptor(acceptor) {
                continue;
            }
            // Donor and acceptor must not be the same element pair (self-type OK
            // but must be spatially distinct).
            let d = distance(&donor.position, &acceptor.position);
            if d > cutoffs.hbond_distance || d < 1.5 {
                continue;
            }

            // Try to measure D-H···A angle using bridging hydrogens.
            let h_atoms = h_neigh;
            let (angle_deg, angle_ok) = if h_atoms.is_empty() {
                // No explicit H: assume geometry is acceptable.
                (None, true)
            } else {
                let mut best_angle = 0.0_f64;
                for &h_idx in h_atoms {
                    if let Some(h_atom) = atoms.get(h_idx) {
                        let ang =
                            angle_between(&donor.position, &h_atom.position, &acceptor.position);
                        if ang > best_angle {
                            best_angle = ang;
                        }
                    }
                }
                (Some(best_angle), best_angle >= cutoffs.hbond_angle_min)
            };

            if !angle_ok {
                continue;
            }

            // Energy estimate: linear interpolation from 0 at cutoff to -2 at 2.8 Å.
            let d_range = cutoffs.hbond_distance - 2.8;
            let energy = if d_range > 1e-9 {
                -2.0 * (1.0 - (d - 2.8) / d_range).clamp(0.0, 1.0)
            } else {
                -2.0
            };

            // Confidence: 1.0 at 2.8 Å, 0.0 at cutoff.
            let confidence = if d_range > 1e-9 {
                (1.0 - (d - 2.8) / d_range).clamp(0.0, 1.0)
            } else {
                1.0
            };

            result.push(Interaction {
                interaction_type: InteractionType::HydrogenBond,
                atom_i: i,
                atom_j: j,
                distance: d,
                angle: angle_deg,
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Detect salt bridges between oppositely charged groups.
///
/// Finds positively- and negatively-charged atom pairs within
/// `cutoffs.salt_bridge_distance`. Energy: `E ≈ -5.0 × (1.0 − d / cutoff)`.
///
/// # Example
///
/// ```
/// use nexcore_viz::interaction::{detect_salt_bridges, InteractionCutoffs};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("test");
/// let mut lys_n = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
/// lys_n.charge = 1;
/// let mut asp_o = Atom::new(2, Element::O, [3.5, 0.0, 0.0]);
/// asp_o.charge = -1;
/// mol.atoms.push(lys_n);
/// mol.atoms.push(asp_o);
/// let cutoffs = InteractionCutoffs::default();
/// let bridges = detect_salt_bridges(&mol, &cutoffs);
/// assert!(!bridges.is_empty());
/// ```
#[must_use]
pub fn detect_salt_bridges(mol: &Molecule, cutoffs: &InteractionCutoffs) -> Vec<Interaction> {
    let mut result = Vec::new();
    let atoms = &mol.atoms;
    let n = atoms.len();

    for i in 0..n {
        let pos_atom = match atoms.get(i) {
            Some(a) => a,
            None => continue,
        };
        if !is_charged_positive(pos_atom) {
            continue;
        }
        for j in 0..n {
            if i == j {
                continue;
            }
            let neg_atom = match atoms.get(j) {
                Some(a) => a,
                None => continue,
            };
            if !is_charged_negative(neg_atom) {
                continue;
            }
            let d = distance(&pos_atom.position, &neg_atom.position);
            if d > cutoffs.salt_bridge_distance || d < 1.0 {
                continue;
            }

            let energy = -5.0 * (1.0 - d / cutoffs.salt_bridge_distance).clamp(0.0, 1.0);
            let confidence = (1.0 - d / cutoffs.salt_bridge_distance).clamp(0.0, 1.0);

            result.push(Interaction {
                interaction_type: InteractionType::SaltBridge,
                atom_i: i,
                atom_j: j,
                distance: d,
                angle: None,
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Detect hydrophobic contacts between non-polar carbon atoms.
///
/// Pairs are accepted only when the residue sequence numbers differ by more
/// than 3 (to exclude sequential backbone contacts). Energy:
/// `E ≈ -0.5 × (1.0 − d / cutoff)`.
///
/// # Example
///
/// ```
/// use nexcore_viz::interaction::{detect_hydrophobic_contacts, InteractionCutoffs};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("test");
/// let mut c1 = Atom::new(1, Element::C, [0.0, 0.0, 0.0]);
/// c1.residue_seq = Some(1);
/// let mut c2 = Atom::new(2, Element::C, [4.0, 0.0, 0.0]);
/// c2.residue_seq = Some(10);
/// mol.atoms.push(c1);
/// mol.atoms.push(c2);
/// let cutoffs = InteractionCutoffs::default();
/// let contacts = detect_hydrophobic_contacts(&mol, &cutoffs);
/// assert!(!contacts.is_empty());
/// ```
#[must_use]
pub fn detect_hydrophobic_contacts(
    mol: &Molecule,
    cutoffs: &InteractionCutoffs,
) -> Vec<Interaction> {
    let mut result = Vec::new();
    let atoms = &mol.atoms;
    let n = atoms.len();

    for i in 0..n {
        let ai = match atoms.get(i) {
            Some(a) => a,
            None => continue,
        };
        if !is_hydrophobic(ai) {
            continue;
        }
        for j in (i + 1)..n {
            let aj = match atoms.get(j) {
                Some(a) => a,
                None => continue,
            };
            if !is_hydrophobic(aj) {
                continue;
            }

            // Exclude sequentially adjacent residues (|seq_i - seq_j| <= 3).
            let seq_gap = match (ai.residue_seq, aj.residue_seq) {
                (Some(si), Some(sj)) => (si - sj).unsigned_abs() as usize,
                _ => usize::MAX, // no residue info → assume non-sequential
            };
            if seq_gap <= 3 {
                continue;
            }

            let d = distance(&ai.position, &aj.position);
            if d > cutoffs.hydrophobic_distance || d < 1.5 {
                continue;
            }

            let energy = -0.5 * (1.0 - d / cutoffs.hydrophobic_distance).clamp(0.0, 1.0);
            let confidence = (1.0 - d / cutoffs.hydrophobic_distance).clamp(0.0, 1.0);

            result.push(Interaction {
                interaction_type: InteractionType::HydrophobicContact,
                atom_i: i,
                atom_j: j,
                distance: d,
                angle: None,
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Detect aromatic pi-stacking interactions between ring pairs.
///
/// Two rings interact when their centres are within `cutoffs.pi_stack_distance`
/// and the inter-plane angle is either < `pi_stack_angle_max` (parallel) or
/// > `90° − pi_stack_angle_max` (T-shaped, edge-to-face). Energies are
/// > -2.5 kcal/mol (parallel) and -1.5 kcal/mol (T-shaped).
#[must_use]
pub fn detect_pi_stacking(mol: &Molecule, cutoffs: &InteractionCutoffs) -> Vec<Interaction> {
    let mut result = Vec::new();
    let rings = detect_rings(mol);

    for (i, ring_a) in rings.iter().enumerate() {
        for ring_b in rings.iter().skip(i + 1) {
            let d = distance(&ring_a.center, &ring_b.center);
            if d > cutoffs.pi_stack_distance || d < 1.0 {
                continue;
            }

            // Use abs(cos θ) to get angle between 0° and 90°.
            let cos_angle = dot3(&ring_a.normal, &ring_b.normal).abs().clamp(0.0, 1.0);
            let angle_deg = cos_angle.acos().to_degrees();

            let is_parallel = angle_deg < cutoffs.pi_stack_angle_max;
            let is_t_shaped = angle_deg > (90.0 - cutoffs.pi_stack_angle_max);

            if !is_parallel && !is_t_shaped {
                continue;
            }

            // Representative atom indices: first atom from each ring.
            let atom_i = ring_a.atom_indices.first().copied().unwrap_or(0);
            let atom_j = ring_b.atom_indices.first().copied().unwrap_or(0);

            let energy = if is_parallel { -2.5 } else { -1.5 };
            let confidence = (1.0 - d / cutoffs.pi_stack_distance).clamp(0.0, 1.0);

            result.push(Interaction {
                interaction_type: InteractionType::PiStacking,
                atom_i,
                atom_j,
                distance: d,
                angle: Some(angle_deg),
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Detect cation-pi interactions between metal/charged cations and aromatic rings.
///
/// Measures the distance from the cation to the ring centroid.
/// Energy: `E ≈ -3.0 × (1.0 − d / cutoff)`.
#[must_use]
pub fn detect_cation_pi(mol: &Molecule, cutoffs: &InteractionCutoffs) -> Vec<Interaction> {
    let mut result = Vec::new();
    let atoms = &mol.atoms;
    let n = atoms.len();
    let rings = detect_rings(mol);

    for i in 0..n {
        let cation = match atoms.get(i) {
            Some(a) => a,
            None => continue,
        };
        if !is_charged_positive(cation) && !is_metal(cation.element) {
            continue;
        }

        for ring in &rings {
            let d = distance(&cation.position, &ring.center);
            if d > cutoffs.cation_pi_distance || d < 1.0 {
                continue;
            }

            let ring_rep = ring.atom_indices.first().copied().unwrap_or(0);
            let energy = -3.0 * (1.0 - d / cutoffs.cation_pi_distance).clamp(0.0, 1.0);
            let confidence = (1.0 - d / cutoffs.cation_pi_distance).clamp(0.0, 1.0);

            result.push(Interaction {
                interaction_type: InteractionType::CationPi,
                atom_i: i,
                atom_j: ring_rep,
                distance: d,
                angle: None,
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Detect halogen bonds (X···A) where X ∈ {F, Cl, Br, I} and A ∈ {N, O, S}.
///
/// Energy: `E ≈ -1.5 × (1.0 − d / cutoff)`.
///
/// # Example
///
/// ```
/// use nexcore_viz::interaction::{detect_halogen_bonds, InteractionCutoffs};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("test");
/// mol.atoms.push(Atom::new(1, Element::Cl, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::O,  [3.0, 0.0, 0.0]));
/// let cutoffs = InteractionCutoffs::default();
/// let halbonds = detect_halogen_bonds(&mol, &cutoffs);
/// assert!(!halbonds.is_empty());
/// ```
#[must_use]
pub fn detect_halogen_bonds(mol: &Molecule, cutoffs: &InteractionCutoffs) -> Vec<Interaction> {
    let mut result = Vec::new();
    let atoms = &mol.atoms;
    let n = atoms.len();

    for i in 0..n {
        let hal = match atoms.get(i) {
            Some(a) => a,
            None => continue,
        };
        if !is_halogen(hal.element) {
            continue;
        }
        for j in 0..n {
            if i == j {
                continue;
            }
            let acc = match atoms.get(j) {
                Some(a) => a,
                None => continue,
            };
            if !is_hbond_acceptor(acc) {
                continue;
            }
            let d = distance(&hal.position, &acc.position);
            if d > cutoffs.halogen_bond_distance || d < 1.0 {
                continue;
            }

            let energy = -1.5 * (1.0 - d / cutoffs.halogen_bond_distance).clamp(0.0, 1.0);
            let confidence = (1.0 - d / cutoffs.halogen_bond_distance).clamp(0.0, 1.0);

            result.push(Interaction {
                interaction_type: InteractionType::HalogenBond,
                atom_i: i,
                atom_j: j,
                distance: d,
                angle: None,
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Detect metal coordination bonds between metal ions and donor atoms (N/O/S).
///
/// Energy: `E ≈ -10.0 × (1.0 − d / cutoff)`.
///
/// # Example
///
/// ```
/// use nexcore_viz::interaction::{detect_metal_coordination, InteractionCutoffs};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("test");
/// mol.atoms.push(Atom::new(1, Element::Zn, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::N,  [2.5, 0.0, 0.0]));
/// let cutoffs = InteractionCutoffs::default();
/// let coords = detect_metal_coordination(&mol, &cutoffs);
/// assert!(!coords.is_empty());
/// ```
#[must_use]
pub fn detect_metal_coordination(mol: &Molecule, cutoffs: &InteractionCutoffs) -> Vec<Interaction> {
    let mut result = Vec::new();
    let atoms = &mol.atoms;
    let n = atoms.len();

    for i in 0..n {
        let metal = match atoms.get(i) {
            Some(a) => a,
            None => continue,
        };
        if !is_metal(metal.element) {
            continue;
        }
        for j in 0..n {
            if i == j {
                continue;
            }
            let ligand = match atoms.get(j) {
                Some(a) => a,
                None => continue,
            };
            if !is_hbond_acceptor(ligand) {
                continue;
            }
            let d = distance(&metal.position, &ligand.position);
            if d > cutoffs.metal_coord_distance || d < 0.5 {
                continue;
            }

            let energy = -10.0 * (1.0 - d / cutoffs.metal_coord_distance).clamp(0.0, 1.0);
            let confidence = (1.0 - d / cutoffs.metal_coord_distance).clamp(0.0, 1.0);

            result.push(Interaction {
                interaction_type: InteractionType::MetalCoordination,
                atom_i: i,
                atom_j: j,
                distance: d,
                angle: None,
                energy_estimate: energy,
                confidence,
            });
        }
    }

    result
}

/// Run all detection functions and aggregate into an `InteractionSummary`.
///
/// # Example
///
/// ```
/// use nexcore_viz::interaction::{detect_all_interactions, InteractionCutoffs};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("test");
/// mol.atoms.push(Atom::new(1, Element::N, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::O, [3.0, 0.0, 0.0]));
/// let summary = detect_all_interactions(&mol, &InteractionCutoffs::default());
/// assert!(summary.total_energy < 0.0);
/// ```
#[must_use]
pub fn detect_all_interactions(mol: &Molecule, cutoffs: &InteractionCutoffs) -> InteractionSummary {
    let hbonds = detect_hydrogen_bonds(mol, cutoffs);
    let salt = detect_salt_bridges(mol, cutoffs);
    let hydrophobic = detect_hydrophobic_contacts(mol, cutoffs);
    let pi = detect_pi_stacking(mol, cutoffs);
    let cation_pi = detect_cation_pi(mol, cutoffs);
    let halogen = detect_halogen_bonds(mol, cutoffs);
    let metal = detect_metal_coordination(mol, cutoffs);

    let hbond_count = hbonds.len();
    let salt_bridge_count = salt.len();
    let hydrophobic_count = hydrophobic.len();
    let pi_stack_count = pi.len();
    let cation_pi_count = cation_pi.len();
    let halogen_bond_count = halogen.len();
    let metal_coord_count = metal.len();

    let mut interactions = Vec::new();
    interactions.extend(hbonds);
    interactions.extend(salt);
    interactions.extend(hydrophobic);
    interactions.extend(pi);
    interactions.extend(cation_pi);
    interactions.extend(halogen);
    interactions.extend(metal);

    let total_energy = interactions.iter().map(|ix| ix.energy_estimate).sum();

    InteractionSummary {
        interactions,
        hbond_count,
        salt_bridge_count,
        hydrophobic_count,
        pi_stack_count,
        cation_pi_count,
        halogen_bond_count,
        metal_coord_count,
        total_energy,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    /// Convenience: build a bare molecule with no bonds or chains.
    fn bare_mol(atoms: Vec<Atom>) -> Molecule {
        Molecule {
            name: String::new(),
            atoms,
            bonds: Vec::new(),
            chains: Vec::new(),
            source_format: None,
        }
    }

    // -----------------------------------------------------------------------
    // 1. distance — known points
    // -----------------------------------------------------------------------
    #[test]
    fn distance_known_points() {
        let a = [0.0_f64, 0.0, 0.0];
        let b = [3.0, 4.0, 0.0];
        let d = distance(&a, &b);
        assert!((d - 5.0).abs() < 1e-10, "expected 5.0, got {d}");
    }

    // -----------------------------------------------------------------------
    // 2. angle_between — right angle
    // -----------------------------------------------------------------------
    #[test]
    fn angle_right_angle() {
        // a=(1,0,0), vertex b=(0,0,0), c=(0,1,0) → 90°
        let a = [1.0_f64, 0.0, 0.0];
        let b = [0.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let ang = angle_between(&a, &b, &c);
        assert!((ang - 90.0).abs() < 1e-8, "expected 90°, got {ang}");
    }

    // -----------------------------------------------------------------------
    // 3. hbond_detected — N and O at 3.0 Å
    // -----------------------------------------------------------------------
    #[test]
    fn hbond_detected() {
        let n_atom = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
        let o_atom = Atom::new(2, Element::O, [3.0, 0.0, 0.0]);
        let mol = bare_mol(vec![n_atom, o_atom]);
        let cutoffs = InteractionCutoffs::default();
        let hbonds = detect_hydrogen_bonds(&mol, &cutoffs);
        // Expect at least one H-bond (both directions N→O and O→N may be found).
        assert!(!hbonds.is_empty(), "expected at least one H-bond");
        assert!(
            hbonds
                .iter()
                .any(|ix| ix.interaction_type == InteractionType::HydrogenBond),
            "wrong interaction type"
        );
    }

    // -----------------------------------------------------------------------
    // 4. hbond_too_far — N and O at 5.0 Å → no detection
    // -----------------------------------------------------------------------
    #[test]
    fn hbond_too_far() {
        let n_atom = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
        let o_atom = Atom::new(2, Element::O, [5.0, 0.0, 0.0]);
        let mol = bare_mol(vec![n_atom, o_atom]);
        let cutoffs = InteractionCutoffs::default();
        let hbonds = detect_hydrogen_bonds(&mol, &cutoffs);
        assert!(hbonds.is_empty(), "expected no H-bonds at 5 Å");
    }

    // -----------------------------------------------------------------------
    // 5. salt_bridge_detected — formal charges at 3.5 Å
    // -----------------------------------------------------------------------
    #[test]
    fn salt_bridge_detected() {
        let mut pos = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
        pos.charge = 1; // Lys NZ proxy
        let mut neg = Atom::new(2, Element::O, [3.5, 0.0, 0.0]);
        neg.charge = -1; // Asp OD1 proxy
        let mol = bare_mol(vec![pos, neg]);
        let cutoffs = InteractionCutoffs::default();
        let bridges = detect_salt_bridges(&mol, &cutoffs);
        assert!(!bridges.is_empty(), "expected a salt bridge");
    }

    // -----------------------------------------------------------------------
    // 6. hydrophobic_contact_detected — two carbons at 4.0 Å, seq gap > 3
    // -----------------------------------------------------------------------
    #[test]
    fn hydrophobic_contact_detected() {
        let mut c1 = Atom::new(1, Element::C, [0.0, 0.0, 0.0]);
        c1.residue_seq = Some(1);
        let mut c2 = Atom::new(2, Element::C, [4.0, 0.0, 0.0]);
        c2.residue_seq = Some(10);
        let mol = bare_mol(vec![c1, c2]);
        let cutoffs = InteractionCutoffs::default();
        let contacts = detect_hydrophobic_contacts(&mol, &cutoffs);
        assert!(!contacts.is_empty(), "expected a hydrophobic contact");
    }

    // -----------------------------------------------------------------------
    // 7. hydrophobic_sequential_excluded — adjacent residues (seq gap <= 3)
    // -----------------------------------------------------------------------
    #[test]
    fn hydrophobic_sequential_excluded() {
        let mut c1 = Atom::new(1, Element::C, [0.0, 0.0, 0.0]);
        c1.residue_seq = Some(5);
        let mut c2 = Atom::new(2, Element::C, [4.0, 0.0, 0.0]);
        c2.residue_seq = Some(7); // gap = 2, should be excluded
        let mol = bare_mol(vec![c1, c2]);
        let cutoffs = InteractionCutoffs::default();
        let contacts = detect_hydrophobic_contacts(&mol, &cutoffs);
        assert!(
            contacts.is_empty(),
            "sequential residue contacts should be excluded"
        );
    }

    // -----------------------------------------------------------------------
    // 8. halogen_bond_detected — Cl and O at 3.0 Å
    // -----------------------------------------------------------------------
    #[test]
    fn halogen_bond_detected() {
        let cl = Atom::new(1, Element::Cl, [0.0, 0.0, 0.0]);
        let o = Atom::new(2, Element::O, [3.0, 0.0, 0.0]);
        let mol = bare_mol(vec![cl, o]);
        let cutoffs = InteractionCutoffs::default();
        let halbonds = detect_halogen_bonds(&mol, &cutoffs);
        assert!(!halbonds.is_empty(), "expected a halogen bond");
        assert_eq!(halbonds[0].interaction_type, InteractionType::HalogenBond);
    }

    // -----------------------------------------------------------------------
    // 9. metal_coordination_detected — Zn and N at 2.5 Å
    // -----------------------------------------------------------------------
    #[test]
    fn metal_coordination_detected() {
        let zn = Atom::new(1, Element::Zn, [0.0, 0.0, 0.0]);
        let n = Atom::new(2, Element::N, [2.5, 0.0, 0.0]);
        let mol = bare_mol(vec![zn, n]);
        let cutoffs = InteractionCutoffs::default();
        let coords = detect_metal_coordination(&mol, &cutoffs);
        assert!(!coords.is_empty(), "expected a metal coordination bond");
        assert_eq!(
            coords[0].interaction_type,
            InteractionType::MetalCoordination
        );
        assert!((coords[0].distance - 2.5).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // 10. summary_counts_correct — detect_all_interactions counts match
    // -----------------------------------------------------------------------
    #[test]
    fn summary_counts_correct() {
        // Molecule with: 1 H-bond pair, 1 salt-bridge pair, 1 metal coord pair.
        let n_atom = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
        let o_atom = Atom::new(2, Element::O, [3.0, 0.0, 0.0]);

        let mut pos = Atom::new(3, Element::N, [10.0, 0.0, 0.0]);
        pos.charge = 1;
        let mut neg = Atom::new(4, Element::O, [13.0, 0.0, 0.0]);
        neg.charge = -1;

        let zn = Atom::new(5, Element::Zn, [20.0, 0.0, 0.0]);
        let n2 = Atom::new(6, Element::N, [22.5, 0.0, 0.0]);

        let mol = bare_mol(vec![n_atom, o_atom, pos, neg, zn, n2]);
        let cutoffs = InteractionCutoffs::default();
        let summary = detect_all_interactions(&mol, &cutoffs);

        // H-bonds: N(0)↔O(1) and O(1)↔N(0), so >= 1 pair detected bidirectionally.
        assert!(summary.hbond_count >= 1, "hbond_count should be >= 1");
        // Salt bridge: pos(2)↔neg(3) at 3.0 Å.
        assert!(
            summary.salt_bridge_count >= 1,
            "salt_bridge_count should be >= 1"
        );
        // Metal coord: Zn(4)↔N(5) at 2.5 Å.
        assert!(
            summary.metal_coord_count >= 1,
            "metal_coord_count should be >= 1"
        );
        // Total energy must be negative (all interactions are favorable).
        assert!(
            summary.total_energy < 0.0,
            "total_energy should be negative"
        );
        // Count consistency.
        assert_eq!(
            summary.hbond_count
                + summary.salt_bridge_count
                + summary.hydrophobic_count
                + summary.pi_stack_count
                + summary.cation_pi_count
                + summary.halogen_bond_count
                + summary.metal_coord_count,
            summary.interactions.len(),
            "per-type counts must sum to total interaction count"
        );
    }

    // -----------------------------------------------------------------------
    // 11. default_cutoffs — sanity check default values
    // -----------------------------------------------------------------------
    #[test]
    fn default_cutoffs_values() {
        let c = InteractionCutoffs::default();
        assert!((c.hbond_distance - 3.5).abs() < 1e-10);
        assert!((c.hbond_angle_min - 120.0).abs() < 1e-10);
        assert!((c.salt_bridge_distance - 4.0).abs() < 1e-10);
        assert!((c.metal_coord_distance - 2.8).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // 12. halogen_bond_too_far — Cl and O at 4.0 Å → not detected
    // -----------------------------------------------------------------------
    #[test]
    fn halogen_bond_too_far() {
        let cl = Atom::new(1, Element::Cl, [0.0, 0.0, 0.0]);
        let o = Atom::new(2, Element::O, [4.0, 0.0, 0.0]);
        let mol = bare_mol(vec![cl, o]);
        let cutoffs = InteractionCutoffs::default();
        let halbonds = detect_halogen_bonds(&mol, &cutoffs);
        assert!(
            halbonds.is_empty(),
            "halogen bond at 4 Å should not be detected (cutoff 3.5)"
        );
    }

    // -----------------------------------------------------------------------
    // 13. metal_too_far — Zn and N at 3.5 Å → not detected
    // -----------------------------------------------------------------------
    #[test]
    fn metal_too_far() {
        let zn = Atom::new(1, Element::Zn, [0.0, 0.0, 0.0]);
        let n = Atom::new(2, Element::N, [3.5, 0.0, 0.0]);
        let mol = bare_mol(vec![zn, n]);
        let cutoffs = InteractionCutoffs::default();
        let coords = detect_metal_coordination(&mol, &cutoffs);
        assert!(coords.is_empty(), "Zn-N at 3.5 Å exceeds cutoff 2.8 Å");
    }

    // -----------------------------------------------------------------------
    // 14. energy_is_negative — all favorable interactions have negative energy
    // -----------------------------------------------------------------------
    #[test]
    fn energy_is_negative() {
        let n_atom = Atom::new(1, Element::N, [0.0, 0.0, 0.0]);
        let o_atom = Atom::new(2, Element::O, [3.0, 0.0, 0.0]);
        let mol = bare_mol(vec![n_atom, o_atom]);
        let cutoffs = InteractionCutoffs::default();
        let hbonds = detect_hydrogen_bonds(&mol, &cutoffs);
        for ix in &hbonds {
            assert!(
                ix.energy_estimate <= 0.0,
                "H-bond energy should be <= 0, got {}",
                ix.energy_estimate
            );
        }
    }

    // -----------------------------------------------------------------------
    // 15. pi_stacking_aromatic_ring — benzene-like 6-membered ring
    // -----------------------------------------------------------------------
    #[test]
    fn pi_stacking_two_rings() {
        // Two planar hexagons offset by 4.0 Å along Z — should form parallel pi stack.
        // Ring 1: z=0 plane
        let r = 1.4_f64; // C-C bond length (aromatic)
        let mut atoms: Vec<Atom> = (0..6)
            .map(|k| {
                let angle = std::f64::consts::FRAC_PI_3 * k as f64;
                Atom::new(
                    (k + 1) as u32,
                    Element::C,
                    [r * angle.cos(), r * angle.sin(), 0.0],
                )
            })
            .collect();
        // Ring 2: z=4.0 plane
        let ring2: Vec<Atom> = (0..6)
            .map(|k| {
                let angle = std::f64::consts::FRAC_PI_3 * k as f64;
                Atom::new(
                    (k + 7) as u32,
                    Element::C,
                    [r * angle.cos(), r * angle.sin(), 4.0],
                )
            })
            .collect();
        atoms.extend(ring2);

        // Aromatic bonds for ring 1 (0-1, 1-2, 2-3, 3-4, 4-5, 5-0)
        let mut bonds: Vec<Bond> = (0..6)
            .map(|k| Bond {
                atom1: k,
                atom2: (k + 1) % 6,
                order: BondOrder::Aromatic,
            })
            .collect();
        // Aromatic bonds for ring 2 (6-7, 7-8, 8-9, 9-10, 10-11, 11-6)
        let ring2_bonds: Vec<Bond> = (0..6)
            .map(|k| Bond {
                atom1: 6 + k,
                atom2: 6 + (k + 1) % 6,
                order: BondOrder::Aromatic,
            })
            .collect();
        bonds.extend(ring2_bonds);

        let mol = Molecule {
            name: String::new(),
            atoms,
            bonds,
            chains: Vec::new(),
            source_format: None,
        };

        let rings = detect_rings(&mol);
        // Expect two rings to be found.
        assert_eq!(
            rings.len(),
            2,
            "should detect exactly two aromatic rings, found {}",
            rings.len()
        );

        let cutoffs = InteractionCutoffs::default();
        let pi_interactions = detect_pi_stacking(&mol, &cutoffs);
        assert!(
            !pi_interactions.is_empty(),
            "expected pi-stacking between two parallel rings"
        );
    }
}
