// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! 3D geometry for molecular structures.
//!
//! ## Primitive Grounding: λ (Location)
//!
//! All positions and transformations are grounded in λ (Location).
//! The λ primitive represents spatial positioning in 3D space.

use crate::error::ChemResult;
use crate::types::{AtomId, Molecule};
use serde::{Deserialize, Serialize};

/// 3D vector for atomic positions.
///
/// ## Tier: T1 (λ + N)
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Zero vector.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Unit X vector.
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    /// Unit Y vector.
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    /// Unit Z vector.
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    /// Create a new vector.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Calculate magnitude (length).
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize to unit vector.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag < 1e-10 {
            Self::ZERO
        } else {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        }
    }

    /// Dot product.
    #[must_use]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.
    #[must_use]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Distance to another point.
    #[must_use]
    pub fn distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Add vectors.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Subtract vectors.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Scale vector.
    #[must_use]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    /// Angle between two vectors (radians).
    #[must_use]
    pub fn angle(&self, other: &Self) -> f64 {
        let dot = self.dot(other);
        let mag = self.magnitude() * other.magnitude();
        if mag < 1e-10 {
            0.0
        } else {
            (dot / mag).clamp(-1.0, 1.0).acos()
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::add(&self, &other)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::sub(&self, &other)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        self.scale(s)
    }
}

/// Molecular geometry with bond lengths and angles.
///
/// ## Tier: T2-C (λ + μ + κ)
#[derive(Debug, Clone, Default)]
pub struct Geometry {
    /// Bond lengths (Å).
    pub bond_lengths: Vec<(AtomId, AtomId, f64)>,
    /// Bond angles (radians).
    pub bond_angles: Vec<(AtomId, AtomId, AtomId, f64)>,
    /// Dihedral angles (radians).
    pub dihedrals: Vec<(AtomId, AtomId, AtomId, AtomId, f64)>,
}

impl Geometry {
    /// Create new geometry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate from molecule.
    pub fn from_molecule(mol: &Molecule) -> Self {
        let mut geom = Self::new();

        // Calculate bond lengths
        for bond in &mol.bonds {
            if let (Some(a1), Some(a2)) = (mol.atom(bond.atom1), mol.atom(bond.atom2)) {
                let dist = a1.position.distance(&a2.position);
                geom.bond_lengths.push((bond.atom1, bond.atom2, dist));
            }
        }

        // Calculate bond angles (for atoms with 2+ neighbors)
        for (i, _atom) in mol.atoms.iter().enumerate() {
            let neighbors = mol.neighbors(i);
            if neighbors.len() >= 2 {
                for j in 0..neighbors.len() {
                    for k in (j + 1)..neighbors.len() {
                        let n1 = neighbors[j];
                        let n2 = neighbors[k];
                        if let (Some(a1), Some(center), Some(a2)) =
                            (mol.atom(n1), mol.atom(i), mol.atom(n2))
                        {
                            let v1 = a1.position.sub(&center.position);
                            let v2 = a2.position.sub(&center.position);
                            let angle = v1.angle(&v2);
                            geom.bond_angles.push((n1, i, n2, angle));
                        }
                    }
                }
            }
        }

        geom
    }
}

/// Builder for generating 3D coordinates.
///
/// ## Tier: T2-C (λ + μ + → + ν)
pub struct GeometryBuilder {
    /// Standard bond lengths by element pair.
    bond_lengths: std::collections::HashMap<(String, String), f64>,
}

impl Default for GeometryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometryBuilder {
    /// Create a new geometry builder with default bond lengths.
    #[must_use]
    pub fn new() -> Self {
        let mut bl = std::collections::HashMap::new();

        // Common bond lengths in Ångströms
        bl.insert(("C".to_string(), "C".to_string()), 1.54);
        bl.insert(("C".to_string(), "H".to_string()), 1.09);
        bl.insert(("C".to_string(), "O".to_string()), 1.43);
        bl.insert(("C".to_string(), "N".to_string()), 1.47);
        bl.insert(("O".to_string(), "H".to_string()), 0.96);
        bl.insert(("N".to_string(), "H".to_string()), 1.01);

        // Double bonds
        bl.insert(("C=".to_string(), "C".to_string()), 1.34);
        bl.insert(("C=".to_string(), "O".to_string()), 1.23);
        bl.insert(("C=".to_string(), "N".to_string()), 1.29);

        // Triple bonds
        bl.insert(("C#".to_string(), "C".to_string()), 1.20);
        bl.insert(("C#".to_string(), "N".to_string()), 1.16);

        Self { bond_lengths: bl }
    }

    /// Get bond length for element pair.
    #[must_use]
    pub fn get_bond_length(&self, e1: &str, e2: &str) -> f64 {
        // Try both orderings
        self.bond_lengths
            .get(&(e1.to_string(), e2.to_string()))
            .or_else(|| self.bond_lengths.get(&(e2.to_string(), e1.to_string())))
            .copied()
            .unwrap_or(1.5) // Default
    }

    /// Generate 3D coordinates for a molecule.
    ///
    /// Uses a simple distance geometry approach:
    /// 1. Place first atom at origin
    /// 2. Place second atom along X axis
    /// 3. Place subsequent atoms using bond angles
    pub fn generate_coordinates(&self, mol: &mut Molecule) -> ChemResult<()> {
        if mol.atoms.is_empty() {
            return Ok(());
        }

        // Place first atom at origin
        if let Some(atom) = mol.atoms.first_mut() {
            atom.position = Vec3::ZERO;
        }

        if mol.atoms.len() == 1 {
            return Ok(());
        }

        // Place second atom along X axis
        if mol.atoms.len() >= 2 && !mol.bonds.is_empty() {
            let sym0 = mol.atoms[0].element().map(|e| e.symbol).unwrap_or("C");
            let sym1 = mol.atoms[1].element().map(|e| e.symbol).unwrap_or("C");
            let bond_len = self.get_bond_length(sym0, sym1);
            if let Some(atom) = mol.atoms.get_mut(1) {
                atom.position = Vec3::new(bond_len, 0.0, 0.0);
            }
        }

        // Place remaining atoms
        // For simplicity, place in a spiral pattern
        for i in 2..mol.atoms.len() {
            // Find a connected atom
            let mut connected_to: Option<AtomId> = None;
            for bond in &mol.bonds {
                if bond.atom2 == i && bond.atom1 < i {
                    connected_to = Some(bond.atom1);
                    break;
                }
                if bond.atom1 == i && bond.atom2 < i {
                    connected_to = Some(bond.atom2);
                    break;
                }
            }

            if let Some(parent) = connected_to {
                if let Some(parent_atom) = mol.atom(parent) {
                    let parent_pos = parent_atom.position;
                    let parent_sym = parent_atom.element().map(|e| e.symbol).unwrap_or("C");

                    // Get the current atom's symbol
                    let child_sym = mol
                        .atoms
                        .get(i)
                        .and_then(|a| a.element())
                        .map(|e| e.symbol)
                        .unwrap_or("C");
                    let bond_len = self.get_bond_length(parent_sym, child_sym);

                    // Calculate position using tetrahedral angle (109.5°)
                    let angle = std::f64::consts::PI * 109.5 / 180.0;
                    let dihedral = (i as f64) * std::f64::consts::PI * 2.0 / 3.0;

                    let x = bond_len * angle.sin() * dihedral.cos();
                    let y = bond_len * angle.sin() * dihedral.sin();
                    let z = bond_len * angle.cos();

                    if let Some(atom) = mol.atoms.get_mut(i) {
                        atom.position = parent_pos.add(&Vec3::new(x, y, z));
                    }
                }
            } else {
                // Not connected, place at a distance
                if let Some(atom) = mol.atoms.get_mut(i) {
                    atom.position = Vec3::new(i as f64 * 2.0, 0.0, 0.0);
                }
            }
        }

        Ok(())
    }

    /// Calculate RMSD between two conformations.
    #[must_use]
    pub fn rmsd(mol1: &Molecule, mol2: &Molecule) -> Option<f64> {
        if mol1.atoms.len() != mol2.atoms.len() {
            return None;
        }

        let n = mol1.atoms.len();
        if n == 0 {
            return Some(0.0);
        }

        let mut sum_sq = 0.0;
        for (a1, a2) in mol1.atoms.iter().zip(mol2.atoms.iter()) {
            let d = a1.position.distance(&a2.position);
            sum_sq += d * d;
        }

        Some((sum_sq / n as f64).sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Atom;

    #[test]
    fn test_vec3_basic() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = Vec3::new(0.0, 0.0, 5.0);
        let n = v.normalize();
        assert!((n.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::X;
        let b = Vec3::Y;
        assert!((a.dot(&b)).abs() < 1e-10); // Perpendicular
    }

    #[test]
    fn test_vec3_cross() {
        let a = Vec3::X;
        let b = Vec3::Y;
        let c = a.cross(&b);
        assert!((c.z - 1.0).abs() < 1e-10); // X × Y = Z
    }

    #[test]
    fn test_vec3_distance() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(3.0, 4.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_angle() {
        let a = Vec3::X;
        let b = Vec3::Y;
        let angle = a.angle(&b);
        let expected = std::f64::consts::PI / 2.0;
        assert!((angle - expected).abs() < 1e-10);
    }

    #[test]
    fn test_geometry_builder() {
        let builder = GeometryBuilder::new();
        let cc_len = builder.get_bond_length("C", "C");
        assert!((cc_len - 1.54).abs() < 0.01);
    }

    #[test]
    fn test_generate_coordinates() {
        use crate::types::Bond;

        let mut mol = Molecule::new();
        mol.add_atom(Atom::carbon());
        mol.add_atom(Atom::carbon());
        let _ = mol.add_bond(Bond::single(0, 1));

        let builder = GeometryBuilder::new();
        let result = builder.generate_coordinates(&mut mol);
        assert!(result.is_ok());

        // First atom at origin
        assert!((mol.atoms[0].position.x).abs() < 1e-10);

        // Second atom along X axis at C-C bond length
        assert!((mol.atoms[1].position.x - 1.54).abs() < 0.1);
    }

    #[test]
    fn test_rmsd() {
        let mut mol1 = Molecule::new();
        mol1.add_atom(Atom::carbon().with_position(Vec3::new(0.0, 0.0, 0.0)));
        mol1.add_atom(Atom::carbon().with_position(Vec3::new(1.0, 0.0, 0.0)));

        let mut mol2 = Molecule::new();
        mol2.add_atom(Atom::carbon().with_position(Vec3::new(0.0, 0.0, 0.0)));
        mol2.add_atom(Atom::carbon().with_position(Vec3::new(1.0, 0.0, 0.0)));

        let rmsd = GeometryBuilder::rmsd(&mol1, &mol2);
        assert!(rmsd.is_some());
        assert!((rmsd.unwrap_or(999.0)).abs() < 1e-10);
    }
}
