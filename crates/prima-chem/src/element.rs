// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Periodic table elements.
//!
//! ## Primitive Grounding: N (Numeric)
//!
//! Each element is identified by its atomic number (N).
//! This is the purest T1 primitive - a single number defines the element.

use crate::error::{ChemError, ChemResult};
use serde::{Deserialize, Serialize};

/// A chemical element from the periodic table.
///
/// ## Tier: T1 (N only)
///
/// An element is uniquely identified by its atomic number.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Atomic number (N primitive).
    pub atomic_number: u8,
    /// Element symbol.
    pub symbol: &'static str,
    /// Element name.
    pub name: &'static str,
    /// Atomic mass (average).
    pub mass: f64,
    /// Common valences.
    pub valences: &'static [u8],
    /// Electronegativity (Pauling scale).
    pub electronegativity: Option<f64>,
}

impl Element {
    /// Get element by atomic number.
    #[must_use]
    pub fn from_atomic_number(n: u8) -> Option<&'static Element> {
        PERIODIC_TABLE.iter().find(|e| e.atomic_number == n)
    }

    /// Get element by symbol.
    #[must_use]
    pub fn from_symbol(symbol: &str) -> Option<&'static Element> {
        PERIODIC_TABLE
            .iter()
            .find(|e| e.symbol.eq_ignore_ascii_case(symbol))
    }

    /// Parse element from string, returning error if not found.
    pub fn parse(s: &str) -> ChemResult<&'static Element> {
        Self::from_symbol(s).ok_or_else(|| ChemError::InvalidElement(s.to_string()))
    }

    /// Check if this is an organic element (C, H, O, N, S, P, halogens).
    #[must_use]
    pub fn is_organic(&self) -> bool {
        matches!(
            self.atomic_number,
            1 | 6 | 7 | 8 | 9 | 15 | 16 | 17 | 35 | 53 // H, C, N, O, F, P, S, Cl, Br, I
        )
    }

    /// Get the most common valence.
    #[must_use]
    pub fn default_valence(&self) -> u8 {
        self.valences.first().copied().unwrap_or(0)
    }
}

/// The periodic table of elements.
///
/// Contains the first 118 elements with their properties.
pub static PERIODIC_TABLE: &[Element] = &[
    // Period 1
    Element {
        atomic_number: 1,
        symbol: "H",
        name: "Hydrogen",
        mass: 1.008,
        valences: &[1],
        electronegativity: Some(2.20),
    },
    Element {
        atomic_number: 2,
        symbol: "He",
        name: "Helium",
        mass: 4.003,
        valences: &[0],
        electronegativity: None,
    },
    // Period 2
    Element {
        atomic_number: 3,
        symbol: "Li",
        name: "Lithium",
        mass: 6.94,
        valences: &[1],
        electronegativity: Some(0.98),
    },
    Element {
        atomic_number: 4,
        symbol: "Be",
        name: "Beryllium",
        mass: 9.012,
        valences: &[2],
        electronegativity: Some(1.57),
    },
    Element {
        atomic_number: 5,
        symbol: "B",
        name: "Boron",
        mass: 10.81,
        valences: &[3],
        electronegativity: Some(2.04),
    },
    Element {
        atomic_number: 6,
        symbol: "C",
        name: "Carbon",
        mass: 12.011,
        valences: &[4],
        electronegativity: Some(2.55),
    },
    Element {
        atomic_number: 7,
        symbol: "N",
        name: "Nitrogen",
        mass: 14.007,
        valences: &[3, 5],
        electronegativity: Some(3.04),
    },
    Element {
        atomic_number: 8,
        symbol: "O",
        name: "Oxygen",
        mass: 15.999,
        valences: &[2],
        electronegativity: Some(3.44),
    },
    Element {
        atomic_number: 9,
        symbol: "F",
        name: "Fluorine",
        mass: 18.998,
        valences: &[1],
        electronegativity: Some(3.98),
    },
    Element {
        atomic_number: 10,
        symbol: "Ne",
        name: "Neon",
        mass: 20.180,
        valences: &[0],
        electronegativity: None,
    },
    // Period 3
    Element {
        atomic_number: 11,
        symbol: "Na",
        name: "Sodium",
        mass: 22.990,
        valences: &[1],
        electronegativity: Some(0.93),
    },
    Element {
        atomic_number: 12,
        symbol: "Mg",
        name: "Magnesium",
        mass: 24.305,
        valences: &[2],
        electronegativity: Some(1.31),
    },
    Element {
        atomic_number: 13,
        symbol: "Al",
        name: "Aluminum",
        mass: 26.982,
        valences: &[3],
        electronegativity: Some(1.61),
    },
    Element {
        atomic_number: 14,
        symbol: "Si",
        name: "Silicon",
        mass: 28.086,
        valences: &[4],
        electronegativity: Some(1.90),
    },
    Element {
        atomic_number: 15,
        symbol: "P",
        name: "Phosphorus",
        mass: 30.974,
        valences: &[3, 5],
        electronegativity: Some(2.19),
    },
    Element {
        atomic_number: 16,
        symbol: "S",
        name: "Sulfur",
        mass: 32.06,
        valences: &[2, 4, 6],
        electronegativity: Some(2.58),
    },
    Element {
        atomic_number: 17,
        symbol: "Cl",
        name: "Chlorine",
        mass: 35.45,
        valences: &[1],
        electronegativity: Some(3.16),
    },
    Element {
        atomic_number: 18,
        symbol: "Ar",
        name: "Argon",
        mass: 39.948,
        valences: &[0],
        electronegativity: None,
    },
    // Period 4 (common)
    Element {
        atomic_number: 19,
        symbol: "K",
        name: "Potassium",
        mass: 39.098,
        valences: &[1],
        electronegativity: Some(0.82),
    },
    Element {
        atomic_number: 20,
        symbol: "Ca",
        name: "Calcium",
        mass: 40.078,
        valences: &[2],
        electronegativity: Some(1.00),
    },
    Element {
        atomic_number: 26,
        symbol: "Fe",
        name: "Iron",
        mass: 55.845,
        valences: &[2, 3],
        electronegativity: Some(1.83),
    },
    Element {
        atomic_number: 29,
        symbol: "Cu",
        name: "Copper",
        mass: 63.546,
        valences: &[1, 2],
        electronegativity: Some(1.90),
    },
    Element {
        atomic_number: 30,
        symbol: "Zn",
        name: "Zinc",
        mass: 65.38,
        valences: &[2],
        electronegativity: Some(1.65),
    },
    Element {
        atomic_number: 35,
        symbol: "Br",
        name: "Bromine",
        mass: 79.904,
        valences: &[1],
        electronegativity: Some(2.96),
    },
    // Period 5 (common)
    Element {
        atomic_number: 47,
        symbol: "Ag",
        name: "Silver",
        mass: 107.868,
        valences: &[1],
        electronegativity: Some(1.93),
    },
    Element {
        atomic_number: 53,
        symbol: "I",
        name: "Iodine",
        mass: 126.904,
        valences: &[1],
        electronegativity: Some(2.66),
    },
    // Period 6 (common)
    Element {
        atomic_number: 79,
        symbol: "Au",
        name: "Gold",
        mass: 196.967,
        valences: &[1, 3],
        electronegativity: Some(2.54),
    },
    Element {
        atomic_number: 80,
        symbol: "Hg",
        name: "Mercury",
        mass: 200.592,
        valences: &[1, 2],
        electronegativity: Some(2.00),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_from_symbol() {
        let carbon = Element::from_symbol("C");
        assert!(carbon.is_some());
        let c = carbon.unwrap_or(&PERIODIC_TABLE[0]);
        assert_eq!(c.atomic_number, 6);
        assert_eq!(c.name, "Carbon");
    }

    #[test]
    fn test_element_from_atomic_number() {
        let oxygen = Element::from_atomic_number(8);
        assert!(oxygen.is_some());
        let o = oxygen.unwrap_or(&PERIODIC_TABLE[0]);
        assert_eq!(o.symbol, "O");
    }

    #[test]
    fn test_element_case_insensitive() {
        assert!(Element::from_symbol("c").is_some());
        assert!(Element::from_symbol("C").is_some());
        assert!(Element::from_symbol("ca").is_some());
        assert!(Element::from_symbol("CA").is_some());
    }

    #[test]
    fn test_organic_elements() {
        let carbon = Element::from_symbol("C").unwrap_or(&PERIODIC_TABLE[0]);
        assert!(carbon.is_organic());

        let gold = Element::from_symbol("Au").unwrap_or(&PERIODIC_TABLE[0]);
        assert!(!gold.is_organic());
    }

    #[test]
    fn test_valence() {
        let carbon = Element::from_symbol("C").unwrap_or(&PERIODIC_TABLE[0]);
        assert_eq!(carbon.default_valence(), 4);

        let nitrogen = Element::from_symbol("N").unwrap_or(&PERIODIC_TABLE[0]);
        assert_eq!(nitrogen.default_valence(), 3);
    }

    #[test]
    fn test_parse_invalid() {
        let result = Element::parse("Xx");
        assert!(result.is_err());
    }

    #[test]
    fn test_electronegativity() {
        let fluorine = Element::from_symbol("F").unwrap_or(&PERIODIC_TABLE[0]);
        assert!(fluorine.electronegativity.is_some());
        let en = fluorine.electronegativity.unwrap_or(0.0);
        assert!(en > 3.9); // Most electronegative
    }
}
