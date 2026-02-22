//! Molecular data types for 3D visualization.
//!
//! Foundation module defining atoms, bonds, molecules, and element properties
//! (VdW radii, CPK colors, covalent radii) used by all molecular rendering.
//!
//! Primitive formula: molecule = σ(atoms) × μ(bonds) + N(properties)
//!   σ: Sequence (ordered atom list)
//!   μ: Mapping (bond connectivity)
//!   N: Quantity (physical properties per element)

use serde::{Deserialize, Serialize};

// ============================================================================
// Element — periodic table subset for drug-relevant atoms
// ============================================================================

/// Chemical element identifiers covering atoms found in drug molecules,
/// biologics, and common solvents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    H,
    He,
    C,
    N,
    O,
    F,
    Na,
    Mg,
    P,
    S,
    Cl,
    K,
    Ca,
    Fe,
    Zn,
    Br,
    I,
    Se,
    /// Unknown or unsupported element
    Other,
}

impl Element {
    /// Parse element from 1-2 character symbol string.
    #[must_use]
    pub fn from_symbol(s: &str) -> Self {
        match s.trim() {
            "H" => Self::H,
            "He" => Self::He,
            "C" => Self::C,
            "N" => Self::N,
            "O" => Self::O,
            "F" => Self::F,
            "Na" => Self::Na,
            "Mg" => Self::Mg,
            "P" => Self::P,
            "S" => Self::S,
            "Cl" => Self::Cl,
            "K" => Self::K,
            "Ca" => Self::Ca,
            "Fe" => Self::Fe,
            "Zn" => Self::Zn,
            "Br" => Self::Br,
            "I" => Self::I,
            "Se" => Self::Se,
            _ => Self::Other,
        }
    }

    /// Chemical symbol string.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            Self::H => "H",
            Self::He => "He",
            Self::C => "C",
            Self::N => "N",
            Self::O => "O",
            Self::F => "F",
            Self::Na => "Na",
            Self::Mg => "Mg",
            Self::P => "P",
            Self::S => "S",
            Self::Cl => "Cl",
            Self::K => "K",
            Self::Ca => "Ca",
            Self::Fe => "Fe",
            Self::Zn => "Zn",
            Self::Br => "Br",
            Self::I => "I",
            Self::Se => "Se",
            Self::Other => "X",
        }
    }

    /// Van der Waals radius in angstroms.
    /// Source: Bondi (1964) + Rowland & Taylor (1996).
    #[must_use]
    pub fn vdw_radius(self) -> f64 {
        match self {
            Self::H => 1.20,
            Self::He => 1.40,
            Self::C => 1.70,
            Self::N => 1.55,
            Self::O => 1.52,
            Self::F => 1.47,
            Self::Na => 2.27,
            Self::Mg => 1.73,
            Self::P => 1.80,
            Self::S => 1.80,
            Self::Cl => 1.75,
            Self::K => 2.75,
            Self::Ca => 2.31,
            Self::Fe => 2.04,
            Self::Zn => 2.10,
            Self::Br => 1.85,
            Self::I => 1.98,
            Self::Se => 1.90,
            Self::Other => 1.70,
        }
    }

    /// Covalent radius in angstroms (single bond).
    /// Source: Cordero et al. (2008).
    #[must_use]
    pub fn covalent_radius(self) -> f64 {
        match self {
            Self::H => 0.31,
            Self::He => 0.28,
            Self::C => 0.76,
            Self::N => 0.71,
            Self::O => 0.66,
            Self::F => 0.57,
            Self::Na => 1.66,
            Self::Mg => 1.41,
            Self::P => 1.07,
            Self::S => 1.05,
            Self::Cl => 1.02,
            Self::K => 2.03,
            Self::Ca => 1.76,
            Self::Fe => 1.32,
            Self::Zn => 1.22,
            Self::Br => 1.20,
            Self::I => 1.39,
            Self::Se => 1.20,
            Self::Other => 1.50,
        }
    }

    /// CPK color as hex string (Corey-Pauling-Koltun convention).
    #[must_use]
    pub fn cpk_color(self) -> &'static str {
        match self {
            Self::H => "#FFFFFF",
            Self::He => "#D9FFFF",
            Self::C => "#909090",
            Self::N => "#3050F8",
            Self::O => "#FF0D0D",
            Self::F => "#90E050",
            Self::Na => "#AB5CF2",
            Self::Mg => "#8AFF00",
            Self::P => "#FF8000",
            Self::S => "#FFFF30",
            Self::Cl => "#1FF01F",
            Self::K => "#8F40D4",
            Self::Ca => "#3DFF00",
            Self::Fe => "#E06633",
            Self::Zn => "#7D80B0",
            Self::Br => "#A62929",
            Self::I => "#940094",
            Self::Se => "#FFA100",
            Self::Other => "#FF1493",
        }
    }

    /// Atomic mass in daltons (g/mol).
    #[must_use]
    pub fn atomic_mass(self) -> f64 {
        match self {
            Self::H => 1.008,
            Self::He => 4.003,
            Self::C => 12.011,
            Self::N => 14.007,
            Self::O => 15.999,
            Self::F => 18.998,
            Self::Na => 22.990,
            Self::Mg => 24.305,
            Self::P => 30.974,
            Self::S => 32.065,
            Self::Cl => 35.453,
            Self::K => 39.098,
            Self::Ca => 40.078,
            Self::Fe => 55.845,
            Self::Zn => 65.380,
            Self::Br => 79.904,
            Self::I => 126.904,
            Self::Se => 78.971,
            Self::Other => 0.0,
        }
    }
}

// ============================================================================
// Atom
// ============================================================================

/// A single atom in 3D space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    /// Atom serial number (1-indexed from file)
    pub id: u32,
    /// Chemical element
    pub element: Element,
    /// 3D position in angstroms [x, y, z]
    pub position: [f64; 3],
    /// Formal charge
    pub charge: i8,
    /// Atom name (e.g. "CA", "N", "O" in PDB)
    pub name: String,
    /// Residue name for protein atoms (e.g. "ALA", "GLY")
    pub residue_name: Option<String>,
    /// Residue sequence number
    pub residue_seq: Option<i32>,
    /// Chain identifier (single char, e.g. 'A', 'B')
    pub chain_id: Option<char>,
    /// Temperature factor (B-factor) from crystallography
    pub b_factor: Option<f64>,
}

impl Atom {
    /// Create a simple atom with position and element.
    #[must_use]
    pub fn new(id: u32, element: Element, position: [f64; 3]) -> Self {
        Self {
            id,
            element,
            position,
            charge: 0,
            name: element.symbol().to_string(),
            residue_name: None,
            residue_seq: None,
            chain_id: None,
            b_factor: None,
        }
    }

    /// Distance to another atom in angstroms.
    #[must_use]
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.position[0] - other.position[0];
        let dy = self.position[1] - other.position[1];
        let dz = self.position[2] - other.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

// ============================================================================
// Bond
// ============================================================================

/// Bond order (single, double, triple, aromatic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BondOrder {
    Single,
    Double,
    Triple,
    Aromatic,
}

impl BondOrder {
    /// Parse from integer (SDF convention: 1=single, 2=double, 3=triple, 4=aromatic).
    #[must_use]
    pub fn from_sdf(n: u8) -> Self {
        match n {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Triple,
            4 => Self::Aromatic,
            _ => Self::Single,
        }
    }

    /// Numeric order for radius/rendering calculations.
    #[must_use]
    pub fn order(self) -> f64 {
        match self {
            Self::Single => 1.0,
            Self::Double => 2.0,
            Self::Triple => 3.0,
            Self::Aromatic => 1.5,
        }
    }

    /// Cylinder radius multiplier for bond rendering.
    #[must_use]
    pub fn render_radius(self) -> f64 {
        match self {
            Self::Single => 0.1,
            Self::Double => 0.07,
            Self::Triple => 0.05,
            Self::Aromatic => 0.08,
        }
    }
}

/// A chemical bond between two atoms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    /// Index of first atom (0-indexed into Molecule.atoms)
    pub atom1: usize,
    /// Index of second atom (0-indexed into Molecule.atoms)
    pub atom2: usize,
    /// Bond order
    pub order: BondOrder,
}

// ============================================================================
// Secondary Structure
// ============================================================================

/// Protein secondary structure assignment for a residue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecondaryStructure {
    /// Alpha helix
    Helix,
    /// Beta sheet/strand
    Sheet,
    /// Loop/coil (no regular structure)
    Coil,
    /// 3-10 helix
    Helix310,
    /// Pi helix
    HelixPi,
    /// Beta turn
    Turn,
}

/// A residue in a protein chain with secondary structure annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residue {
    /// Residue name (3-letter code, e.g. "ALA")
    pub name: String,
    /// Sequence number
    pub seq: i32,
    /// Insertion code (rare, usually ' ')
    pub insertion_code: Option<char>,
    /// Indices into Molecule.atoms for atoms belonging to this residue
    pub atom_indices: Vec<usize>,
    /// Secondary structure assignment
    pub secondary_structure: SecondaryStructure,
}

/// A protein chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    /// Chain identifier (e.g. 'A')
    pub id: char,
    /// Residues in sequence order
    pub residues: Vec<Residue>,
}

// ============================================================================
// Molecule
// ============================================================================

/// A complete molecular structure with atoms, bonds, and optional protein hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Molecule {
    /// Human-readable name
    pub name: String,
    /// All atoms
    pub atoms: Vec<Atom>,
    /// All bonds
    pub bonds: Vec<Bond>,
    /// Protein chains (empty for small molecules)
    pub chains: Vec<Chain>,
    /// Source format identifier
    pub source_format: Option<String>,
}

impl Molecule {
    /// Create an empty molecule with a name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            atoms: Vec::new(),
            bonds: Vec::new(),
            chains: Vec::new(),
            source_format: None,
        }
    }

    /// Total molecular weight in daltons.
    #[must_use]
    pub fn molecular_weight(&self) -> f64 {
        self.atoms.iter().map(|a| a.element.atomic_mass()).sum()
    }

    /// Molecular formula as string (e.g. "C9H8O4").
    #[must_use]
    pub fn formula(&self) -> String {
        use std::collections::BTreeMap;
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for atom in &self.atoms {
            *counts.entry(atom.element.symbol()).or_insert(0) += 1;
        }

        // Hill system: C first, H second, then alphabetical
        let mut result = String::new();
        if let Some(&c) = counts.get("C") {
            result.push('C');
            if c > 1 {
                result.push_str(&c.to_string());
            }
            counts.remove("C");
            if let Some(&h) = counts.get("H") {
                result.push('H');
                if h > 1 {
                    result.push_str(&h.to_string());
                }
                counts.remove("H");
            }
        }
        for (symbol, count) in &counts {
            result.push_str(symbol);
            if *count > 1 {
                result.push_str(&count.to_string());
            }
        }
        result
    }

    /// Bounding box as ([min_x, min_y, min_z], [max_x, max_y, max_z]).
    #[must_use]
    pub fn bounding_box(&self) -> ([f64; 3], [f64; 3]) {
        if self.atoms.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }
        let mut min = [f64::MAX; 3];
        let mut max = [f64::MIN; 3];
        for atom in &self.atoms {
            for i in 0..3 {
                if atom.position[i] < min[i] {
                    min[i] = atom.position[i];
                }
                if atom.position[i] > max[i] {
                    max[i] = atom.position[i];
                }
            }
        }
        (min, max)
    }

    /// Center of geometry.
    #[must_use]
    pub fn center_of_geometry(&self) -> [f64; 3] {
        if self.atoms.is_empty() {
            return [0.0; 3];
        }
        let n = self.atoms.len() as f64;
        let mut center = [0.0; 3];
        for atom in &self.atoms {
            for (c, &p) in center.iter_mut().zip(&atom.position) {
                *c += p;
            }
        }
        for c in &mut center {
            *c /= n;
        }
        center
    }

    /// Center the molecule at the origin.
    pub fn center_at_origin(&mut self) {
        let center = self.center_of_geometry();
        for atom in &mut self.atoms {
            for (p, &c) in atom.position.iter_mut().zip(&center) {
                *p -= c;
            }
        }
    }

    /// Get atoms bonded to a given atom index.
    #[must_use]
    pub fn bonded_to(&self, atom_idx: usize) -> Vec<usize> {
        let mut neighbors = Vec::new();
        for bond in &self.bonds {
            if bond.atom1 == atom_idx {
                neighbors.push(bond.atom2);
            } else if bond.atom2 == atom_idx {
                neighbors.push(bond.atom1);
            }
        }
        neighbors
    }

    /// Count atoms by element.
    #[must_use]
    pub fn element_counts(&self) -> std::collections::HashMap<Element, usize> {
        let mut counts = std::collections::HashMap::new();
        for atom in &self.atoms {
            *counts.entry(atom.element).or_insert(0) += 1;
        }
        counts
    }

    /// Unique chain IDs present.
    #[must_use]
    pub fn chain_ids(&self) -> Vec<char> {
        self.chains.iter().map(|c| c.id).collect()
    }

    /// Is this a protein (has chain/residue hierarchy)?
    #[must_use]
    pub fn is_protein(&self) -> bool {
        !self.chains.is_empty()
    }

    /// Serialize to JSON for MCP tool output / frontend consumption.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize to pretty JSON.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// ============================================================================
// Rendering helpers
// ============================================================================

/// Ball-and-stick rendering parameters for a single atom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRenderData {
    /// Position [x, y, z] in angstroms
    pub position: [f64; 3],
    /// Sphere radius (scaled VdW or fixed)
    pub radius: f64,
    /// CPK color hex
    pub color: String,
    /// Element symbol
    pub label: String,
    /// Atom serial ID
    pub id: u32,
}

/// Bond rendering parameters for a cylinder between two atoms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondRenderData {
    /// Start position [x, y, z]
    pub start: [f64; 3],
    /// End position [x, y, z]
    pub end: [f64; 3],
    /// Cylinder radius
    pub radius: f64,
    /// Bond order (for multi-cylinder rendering)
    pub order: f64,
    /// Start atom color
    pub color_start: String,
    /// End atom color
    pub color_end: String,
}

/// Rendering mode for molecular visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode {
    /// Spheres at VdW radius + cylinder bonds
    BallAndStick,
    /// Spheres at full VdW radius, no explicit bonds
    SpaceFilling,
    /// Lines only, element-colored
    Wireframe,
    /// Protein secondary structure (helices as ribbons, sheets as arrows)
    Ribbon,
    /// Molecular surface (SES or VdW)
    Surface,
}

/// Generate rendering data for a molecule in a given mode.
///
/// Returns atom spheres and bond cylinders ready for frontend consumption.
#[must_use]
pub fn render_molecule(mol: &Molecule, mode: RenderMode) -> (Vec<AtomRenderData>, Vec<BondRenderData>) {
    let scale = match mode {
        RenderMode::BallAndStick => 0.3,
        RenderMode::SpaceFilling => 1.0,
        RenderMode::Wireframe => 0.1,
        RenderMode::Ribbon => 0.2,
        RenderMode::Surface => 0.3,
    };

    let atoms: Vec<AtomRenderData> = mol
        .atoms
        .iter()
        .map(|a| AtomRenderData {
            position: a.position,
            radius: a.element.vdw_radius() * scale,
            color: a.element.cpk_color().to_string(),
            label: a.element.symbol().to_string(),
            id: a.id,
        })
        .collect();

    let bonds: Vec<BondRenderData> = if mode == RenderMode::SpaceFilling {
        Vec::new() // Space-filling mode has no explicit bonds
    } else {
        mol.bonds
            .iter()
            .filter_map(|b| {
                let a1 = mol.atoms.get(b.atom1)?;
                let a2 = mol.atoms.get(b.atom2)?;
                Some(BondRenderData {
                    start: a1.position,
                    end: a2.position,
                    radius: b.order.render_radius(),
                    order: b.order.order(),
                    color_start: a1.element.cpk_color().to_string(),
                    color_end: a2.element.cpk_color().to_string(),
                })
            })
            .collect()
    };

    (atoms, bonds)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> Molecule {
        let mut mol = Molecule::new("Water");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
        mol.atoms.push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
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

    #[test]
    fn formula_hill_system() {
        let mol = water();
        assert_eq!(mol.formula(), "H2O");
    }

    #[test]
    fn molecular_weight_water() {
        let mol = water();
        let mw = mol.molecular_weight();
        assert!((mw - 18.015).abs() < 0.01);
    }

    #[test]
    fn bounding_box() {
        let mol = water();
        let (min, max) = mol.bounding_box();
        assert!(min[0] < 0.0);
        assert!(max[0] > 0.0);
    }

    #[test]
    fn center_of_geometry() {
        let mol = water();
        let center = mol.center_of_geometry();
        assert!((center[1] - 0.391).abs() < 0.01);
    }

    #[test]
    fn bonded_to_oxygen() {
        let mol = water();
        let neighbors = mol.bonded_to(0);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn element_from_symbol() {
        assert_eq!(Element::from_symbol("C"), Element::C);
        assert_eq!(Element::from_symbol("Cl"), Element::Cl);
        assert_eq!(Element::from_symbol("Xx"), Element::Other);
    }

    #[test]
    fn cpk_colors_distinct() {
        assert_ne!(Element::C.cpk_color(), Element::N.cpk_color());
        assert_ne!(Element::O.cpk_color(), Element::S.cpk_color());
    }

    #[test]
    fn render_ball_and_stick() {
        let mol = water();
        let (atoms, bonds) = render_molecule(&mol, RenderMode::BallAndStick);
        assert_eq!(atoms.len(), 3);
        assert_eq!(bonds.len(), 2);
        assert!(atoms[0].radius < atoms[0].position[0].abs() + 1.0); // scaled down
    }

    #[test]
    fn render_space_filling_no_bonds() {
        let mol = water();
        let (atoms, bonds) = render_molecule(&mol, RenderMode::SpaceFilling);
        assert_eq!(atoms.len(), 3);
        assert!(bonds.is_empty());
        // Full VdW radius for oxygen
        assert!((atoms[0].radius - 1.52).abs() < 0.01);
    }

    #[test]
    fn serialize_molecule() {
        let mol = water();
        let json = mol.to_json();
        assert!(json.is_ok());
        let s = json.unwrap_or_default();
        assert!(s.contains("Water"));
        assert!(s.contains("\"O\""));
    }

    #[test]
    fn distance_calculation() {
        let a1 = Atom::new(1, Element::O, [0.0, 0.0, 0.0]);
        let a2 = Atom::new(2, Element::H, [1.0, 0.0, 0.0]);
        assert!((a1.distance_to(&a2) - 1.0).abs() < 1e-10);
    }
}
