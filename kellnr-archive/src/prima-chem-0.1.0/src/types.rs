// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Core molecular types.
//!
//! ## Primitive Grounding
//!
//! - Atom: N (atomic_number) + λ (position) + ς (charge/state)
//! - Bond: → (connects atoms) + N (order) + ∂ (type boundary)
//! - Molecule: Σ (sum of atoms) + μ (bond mapping) + σ (atom sequence)

use crate::element::Element;
use crate::error::{ChemError, ChemResult};
use crate::geometry::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Atom identifier (index in molecule).
pub type AtomId = usize;

/// Bond order (single, double, triple, aromatic).
///
/// ## Tier: T1 (N only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BondOrder {
    /// Single bond (order 1).
    Single = 1,
    /// Double bond (order 2).
    Double = 2,
    /// Triple bond (order 3).
    Triple = 3,
    /// Aromatic bond (order 1.5, represented as 4).
    Aromatic = 4,
}

impl BondOrder {
    /// Create from numeric order.
    pub fn from_order(n: u8) -> ChemResult<Self> {
        match n {
            1 => Ok(Self::Single),
            2 => Ok(Self::Double),
            3 => Ok(Self::Triple),
            4 => Ok(Self::Aromatic),
            _ => Err(ChemError::InvalidBondOrder(n)),
        }
    }

    /// Get numeric order (for electron counting).
    #[must_use]
    pub fn as_electrons(&self) -> u8 {
        match self {
            Self::Single => 2,
            Self::Double => 4,
            Self::Triple => 6,
            Self::Aromatic => 3, // Average
        }
    }

    /// Get contribution to valence.
    #[must_use]
    pub fn valence_contribution(&self) -> u8 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
            Self::Aromatic => 1, // Counts as 1 for valence
        }
    }
}

/// Bond type (stereochemistry).
///
/// ## Tier: T2-P (∂ + λ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BondType {
    /// No stereochemistry specified.
    #[default]
    None,
    /// Wedge bond (coming out of page).
    Wedge,
    /// Dash bond (going into page).
    Dash,
    /// Cis configuration.
    Cis,
    /// Trans configuration.
    Trans,
}

/// A chemical bond between two atoms.
///
/// ## Tier: T2-P (→ + N + ∂)
///
/// Grounding:
/// - → (Causality): Connects atom1 to atom2
/// - N (Numeric): Bond order
/// - ∂ (Boundary): Stereochemistry type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    /// First atom ID.
    pub atom1: AtomId,
    /// Second atom ID.
    pub atom2: AtomId,
    /// Bond order.
    pub order: BondOrder,
    /// Bond type (stereochemistry).
    pub bond_type: BondType,
}

impl Bond {
    /// Create a new single bond.
    #[must_use]
    pub fn single(atom1: AtomId, atom2: AtomId) -> Self {
        Self {
            atom1,
            atom2,
            order: BondOrder::Single,
            bond_type: BondType::None,
        }
    }

    /// Create a new double bond.
    #[must_use]
    pub fn double(atom1: AtomId, atom2: AtomId) -> Self {
        Self {
            atom1,
            atom2,
            order: BondOrder::Double,
            bond_type: BondType::None,
        }
    }

    /// Create a new triple bond.
    #[must_use]
    pub fn triple(atom1: AtomId, atom2: AtomId) -> Self {
        Self {
            atom1,
            atom2,
            order: BondOrder::Triple,
            bond_type: BondType::None,
        }
    }

    /// Create an aromatic bond.
    #[must_use]
    pub fn aromatic(atom1: AtomId, atom2: AtomId) -> Self {
        Self {
            atom1,
            atom2,
            order: BondOrder::Aromatic,
            bond_type: BondType::None,
        }
    }

    /// Check if this bond involves an atom.
    #[must_use]
    pub fn involves(&self, atom: AtomId) -> bool {
        self.atom1 == atom || self.atom2 == atom
    }

    /// Get the other atom in the bond.
    #[must_use]
    pub fn other(&self, atom: AtomId) -> Option<AtomId> {
        if self.atom1 == atom {
            Some(self.atom2)
        } else if self.atom2 == atom {
            Some(self.atom1)
        } else {
            None
        }
    }
}

/// An atom in a molecule.
///
/// ## Tier: T2-P (N + λ + ς)
///
/// Grounding:
/// - N (Numeric): Atomic number (element identity)
/// - λ (Location): 3D position
/// - ς (State): Formal charge, isotope
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atom {
    /// Atomic number (N primitive - defines element).
    pub atomic_number: u8,
    /// Formal charge.
    pub charge: i8,
    /// Mass number (for isotopes, 0 = natural abundance).
    pub mass_number: u16,
    /// 3D position (λ primitive).
    pub position: Vec3,
    /// Number of implicit hydrogens.
    pub implicit_h: u8,
    /// Atom is aromatic.
    pub aromatic: bool,
}

impl Atom {
    /// Create a new atom with default properties.
    #[must_use]
    pub fn new(element: &'static Element) -> Self {
        Self {
            atomic_number: element.atomic_number,
            charge: 0,
            mass_number: 0,
            position: Vec3::ZERO,
            implicit_h: 0,
            aromatic: false,
        }
    }

    /// Create an atom from atomic number.
    #[must_use]
    pub fn from_atomic_number(n: u8) -> Self {
        Self {
            atomic_number: n,
            charge: 0,
            mass_number: 0,
            position: Vec3::ZERO,
            implicit_h: 0,
            aromatic: false,
        }
    }

    /// Get the element reference.
    #[must_use]
    pub fn element(&self) -> Option<&'static Element> {
        Element::from_atomic_number(self.atomic_number)
    }

    /// Create a carbon atom (atomic number 6).
    #[must_use]
    pub fn carbon() -> Self {
        Self::from_atomic_number(6)
    }

    /// Create a hydrogen atom (atomic number 1).
    #[must_use]
    pub fn hydrogen() -> Self {
        Self::from_atomic_number(1)
    }

    /// Create an oxygen atom (atomic number 8).
    #[must_use]
    pub fn oxygen() -> Self {
        Self::from_atomic_number(8)
    }

    /// Create a nitrogen atom (atomic number 7).
    #[must_use]
    pub fn nitrogen() -> Self {
        Self::from_atomic_number(7)
    }

    /// Set the charge.
    #[must_use]
    pub fn with_charge(mut self, charge: i8) -> Self {
        self.charge = charge;
        self
    }

    /// Set the position.
    #[must_use]
    pub fn with_position(mut self, pos: Vec3) -> Self {
        self.position = pos;
        self
    }

    /// Set aromatic flag.
    #[must_use]
    pub fn aromatic(mut self) -> Self {
        self.aromatic = true;
        self
    }
}

/// A molecule — a collection of atoms and bonds.
///
/// ## Tier: T2-C (Σ + μ + σ + λ)
///
/// Grounding:
/// - Σ (Sum): Collection of atoms
/// - μ (Mapping): Bond mapping (atom pairs → bond)
/// - σ (Sequence): Ordered atom list
/// - λ (Location): 3D geometry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Molecule {
    /// Atoms in the molecule (σ primitive).
    pub atoms: Vec<Atom>,
    /// Bonds between atoms (μ primitive).
    pub bonds: Vec<Bond>,
    /// Molecule name.
    pub name: Option<String>,
    /// Net charge.
    pub charge: i32,
    /// Spin multiplicity (2S+1).
    pub multiplicity: u8,
}

impl Molecule {
    /// Create an empty molecule.
    #[must_use]
    pub fn new() -> Self {
        Self {
            atoms: Vec::new(),
            bonds: Vec::new(),
            name: None,
            charge: 0,
            multiplicity: 1,
        }
    }

    /// Create a molecule builder.
    #[must_use]
    pub fn builder() -> MoleculeBuilder {
        MoleculeBuilder::new()
    }

    /// Add an atom, returning its ID.
    pub fn add_atom(&mut self, atom: Atom) -> AtomId {
        let id = self.atoms.len();
        self.atoms.push(atom);
        id
    }

    /// Add a bond between atoms.
    pub fn add_bond(&mut self, bond: Bond) -> ChemResult<()> {
        // Validate atom IDs
        if bond.atom1 >= self.atoms.len() {
            return Err(ChemError::AtomNotFound(bond.atom1));
        }
        if bond.atom2 >= self.atoms.len() {
            return Err(ChemError::AtomNotFound(bond.atom2));
        }
        self.bonds.push(bond);
        Ok(())
    }

    /// Get atom by ID.
    #[must_use]
    pub fn atom(&self, id: AtomId) -> Option<&Atom> {
        self.atoms.get(id)
    }

    /// Get mutable atom by ID.
    pub fn atom_mut(&mut self, id: AtomId) -> Option<&mut Atom> {
        self.atoms.get_mut(id)
    }

    /// Get bonds involving an atom.
    #[must_use]
    pub fn bonds_for_atom(&self, atom: AtomId) -> Vec<&Bond> {
        self.bonds.iter().filter(|b| b.involves(atom)).collect()
    }

    /// Get neighbors of an atom.
    #[must_use]
    pub fn neighbors(&self, atom: AtomId) -> Vec<AtomId> {
        self.bonds.iter().filter_map(|b| b.other(atom)).collect()
    }

    /// Count atoms of a specific element.
    #[must_use]
    pub fn count_element(&self, symbol: &str) -> usize {
        self.atoms
            .iter()
            .filter(|a| a.element().is_some_and(|e| e.symbol == symbol))
            .count()
    }

    /// Get molecular formula (Hill notation).
    #[must_use]
    pub fn molecular_formula(&self) -> String {
        let mut counts: HashMap<&str, usize> = HashMap::new();

        for atom in &self.atoms {
            if let Some(elem) = atom.element() {
                *counts.entry(elem.symbol).or_insert(0) += 1;
            }
            // Count implicit hydrogens
            *counts.entry("H").or_insert(0) += atom.implicit_h as usize;
        }

        let mut formula = String::new();

        // Hill notation: C first, then H, then alphabetical
        if let Some(&c) = counts.get("C") {
            formula.push('C');
            if c > 1 {
                formula.push_str(&c.to_string());
            }
            counts.remove("C");

            if let Some(&h) = counts.get("H") {
                formula.push('H');
                if h > 1 {
                    formula.push_str(&h.to_string());
                }
                counts.remove("H");
            }
        }

        // Rest alphabetically
        let mut keys: Vec<_> = counts.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(&count) = counts.get(key) {
                formula.push_str(key);
                if count > 1 {
                    formula.push_str(&count.to_string());
                }
            }
        }

        formula
    }

    /// Calculate molecular weight.
    #[must_use]
    pub fn molecular_weight(&self) -> f64 {
        let mut weight = 0.0;
        for atom in &self.atoms {
            if let Some(elem) = atom.element() {
                weight += elem.mass;
            }
            weight += atom.implicit_h as f64 * 1.008; // H mass
        }
        weight
    }

    /// Get number of atoms.
    #[must_use]
    pub fn atom_count(&self) -> usize {
        self.atoms.len()
    }

    /// Get number of bonds.
    #[must_use]
    pub fn bond_count(&self) -> usize {
        self.bonds.len()
    }
}

/// Builder for molecules.
///
/// ## Tier: T2-C (μ + ν)
#[derive(Debug, Default)]
pub struct MoleculeBuilder {
    molecule: Molecule,
}

impl MoleculeBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            molecule: Molecule::new(),
        }
    }

    /// Set the molecule name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.molecule.name = Some(name.to_string());
        self
    }

    /// Add an atom.
    pub fn atom(mut self, atom: Atom) -> (Self, AtomId) {
        let id = self.molecule.add_atom(atom);
        (self, id)
    }

    /// Add a carbon atom.
    pub fn carbon(self) -> (Self, AtomId) {
        self.atom(Atom::carbon())
    }

    /// Add a hydrogen atom.
    pub fn hydrogen(self) -> (Self, AtomId) {
        self.atom(Atom::hydrogen())
    }

    /// Add a single bond.
    pub fn single_bond(mut self, a1: AtomId, a2: AtomId) -> ChemResult<Self> {
        self.molecule.add_bond(Bond::single(a1, a2))?;
        Ok(self)
    }

    /// Add a double bond.
    pub fn double_bond(mut self, a1: AtomId, a2: AtomId) -> ChemResult<Self> {
        self.molecule.add_bond(Bond::double(a1, a2))?;
        Ok(self)
    }

    /// Build the molecule.
    #[must_use]
    pub fn build(self) -> Molecule {
        self.molecule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_order() {
        assert_eq!(BondOrder::Single.valence_contribution(), 1);
        assert_eq!(BondOrder::Double.valence_contribution(), 2);
        assert_eq!(BondOrder::Triple.valence_contribution(), 3);
    }

    #[test]
    fn test_atom_creation() {
        let c = Atom::carbon();
        assert_eq!(c.atomic_number, 6);
        assert!(c.element().is_some_and(|e| e.symbol == "C"));
        assert_eq!(c.charge, 0);
    }

    #[test]
    fn test_atom_with_charge() {
        let n = Atom::nitrogen().with_charge(1);
        assert_eq!(n.charge, 1);
    }

    #[test]
    fn test_molecule_builder() {
        let (builder, c1) = MoleculeBuilder::new().carbon();
        let (builder, c2) = builder.carbon();
        let mol = builder
            .single_bond(c1, c2)
            .ok()
            .unwrap_or_else(MoleculeBuilder::new)
            .build();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bond_count(), 1);
    }

    #[test]
    fn test_molecular_formula() {
        // Build water: H2O
        let mut mol = Molecule::new();
        let o = mol.add_atom(Atom::oxygen());
        let h1 = mol.add_atom(Atom::hydrogen());
        let h2 = mol.add_atom(Atom::hydrogen());
        let _ = mol.add_bond(Bond::single(o, h1));
        let _ = mol.add_bond(Bond::single(o, h2));

        let formula = mol.molecular_formula();
        assert!(formula.contains('H'));
        assert!(formula.contains('O'));
    }

    #[test]
    fn test_neighbors() {
        let mut mol = Molecule::new();
        let c = mol.add_atom(Atom::carbon());
        let h1 = mol.add_atom(Atom::hydrogen());
        let h2 = mol.add_atom(Atom::hydrogen());
        let _ = mol.add_bond(Bond::single(c, h1));
        let _ = mol.add_bond(Bond::single(c, h2));

        let neighbors = mol.neighbors(c);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_molecular_weight() {
        // Methane CH4 = 12.011 + 4*1.008 = 16.043
        let mut mol = Molecule::new();
        let mut c = Atom::carbon();
        c.implicit_h = 4; // 4 implicit hydrogens
        mol.add_atom(c);

        let mw = mol.molecular_weight();
        assert!((mw - 16.043).abs() < 0.01);
    }
}
