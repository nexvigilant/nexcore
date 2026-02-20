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
    ///
    /// Uses direct indexing since the table is contiguous 1-118.
    ///
    /// # Examples
    ///
    /// ```
    /// use prima_chem::element::Element;
    /// let hydrogen = Element::from_atomic_number(1).unwrap();
    /// assert_eq!(hydrogen.symbol, "H");
    /// ```
    #[must_use]
    pub fn from_atomic_number(n: u8) -> Option<&'static Element> {
        if n == 0 || n > 118 {
            return None;
        }
        PERIODIC_TABLE.get((n as usize) - 1)
    }

    /// Get element by symbol (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use prima_chem::element::Element;
    /// let gold = Element::from_symbol("Au").unwrap();
    /// assert_eq!(gold.atomic_number, 79);
    /// ```
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
/// Contains the first 118 elements with their properties,
/// sorted by atomic number 1-118 for O(1) indexed lookup.
pub static PERIODIC_TABLE: &[Element] = &[
    // -------------------------------------------------------------------------
    // Period 1
    // -------------------------------------------------------------------------
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
    // -------------------------------------------------------------------------
    // Period 2
    // -------------------------------------------------------------------------
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
    // -------------------------------------------------------------------------
    // Period 3
    // -------------------------------------------------------------------------
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
    // -------------------------------------------------------------------------
    // Period 4
    // -------------------------------------------------------------------------
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
        atomic_number: 21,
        symbol: "Sc",
        name: "Scandium",
        mass: 44.956,
        valences: &[3],
        electronegativity: Some(1.36),
    },
    Element {
        atomic_number: 22,
        symbol: "Ti",
        name: "Titanium",
        mass: 47.867,
        valences: &[4, 3, 2],
        electronegativity: Some(1.54),
    },
    Element {
        atomic_number: 23,
        symbol: "V",
        name: "Vanadium",
        mass: 50.942,
        valences: &[5, 4, 3, 2],
        electronegativity: Some(1.63),
    },
    Element {
        atomic_number: 24,
        symbol: "Cr",
        name: "Chromium",
        mass: 51.996,
        valences: &[3, 6, 2],
        electronegativity: Some(1.66),
    },
    Element {
        atomic_number: 25,
        symbol: "Mn",
        name: "Manganese",
        mass: 54.938,
        valences: &[2, 4, 7],
        electronegativity: Some(1.55),
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
        atomic_number: 27,
        symbol: "Co",
        name: "Cobalt",
        mass: 58.933,
        valences: &[2, 3],
        electronegativity: Some(1.88),
    },
    Element {
        atomic_number: 28,
        symbol: "Ni",
        name: "Nickel",
        mass: 58.693,
        valences: &[2, 3],
        electronegativity: Some(1.91),
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
        atomic_number: 31,
        symbol: "Ga",
        name: "Gallium",
        mass: 69.723,
        valences: &[3],
        electronegativity: Some(1.81),
    },
    Element {
        atomic_number: 32,
        symbol: "Ge",
        name: "Germanium",
        mass: 72.630,
        valences: &[4, 2],
        electronegativity: Some(2.01),
    },
    Element {
        atomic_number: 33,
        symbol: "As",
        name: "Arsenic",
        mass: 74.922,
        valences: &[3, 5],
        electronegativity: Some(2.18),
    },
    Element {
        atomic_number: 34,
        symbol: "Se",
        name: "Selenium",
        mass: 78.971,
        valences: &[2, 4, 6],
        electronegativity: Some(2.55),
    },
    Element {
        atomic_number: 35,
        symbol: "Br",
        name: "Bromine",
        mass: 79.904,
        valences: &[1],
        electronegativity: Some(2.96),
    },
    Element {
        atomic_number: 36,
        symbol: "Kr",
        name: "Krypton",
        mass: 83.798,
        valences: &[0],
        electronegativity: None,
    },
    // -------------------------------------------------------------------------
    // Period 5
    // -------------------------------------------------------------------------
    Element {
        atomic_number: 37,
        symbol: "Rb",
        name: "Rubidium",
        mass: 85.468,
        valences: &[1],
        electronegativity: Some(0.82),
    },
    Element {
        atomic_number: 38,
        symbol: "Sr",
        name: "Strontium",
        mass: 87.62,
        valences: &[2],
        electronegativity: Some(0.95),
    },
    Element {
        atomic_number: 39,
        symbol: "Y",
        name: "Yttrium",
        mass: 88.906,
        valences: &[3],
        electronegativity: Some(1.22),
    },
    Element {
        atomic_number: 40,
        symbol: "Zr",
        name: "Zirconium",
        mass: 91.224,
        valences: &[4],
        electronegativity: Some(1.33),
    },
    Element {
        atomic_number: 41,
        symbol: "Nb",
        name: "Niobium",
        mass: 92.906,
        valences: &[5, 3],
        electronegativity: Some(1.60),
    },
    Element {
        atomic_number: 42,
        symbol: "Mo",
        name: "Molybdenum",
        mass: 95.95,
        valences: &[6, 4, 2],
        electronegativity: Some(2.16),
    },
    Element {
        atomic_number: 43,
        symbol: "Tc",
        name: "Technetium",
        mass: 97.0,
        valences: &[7, 4],
        electronegativity: Some(1.90),
    },
    Element {
        atomic_number: 44,
        symbol: "Ru",
        name: "Ruthenium",
        mass: 101.07,
        valences: &[3, 4, 8],
        electronegativity: Some(2.20),
    },
    Element {
        atomic_number: 45,
        symbol: "Rh",
        name: "Rhodium",
        mass: 102.906,
        valences: &[3],
        electronegativity: Some(2.28),
    },
    Element {
        atomic_number: 46,
        symbol: "Pd",
        name: "Palladium",
        mass: 106.42,
        valences: &[2, 4],
        electronegativity: Some(2.20),
    },
    Element {
        atomic_number: 47,
        symbol: "Ag",
        name: "Silver",
        mass: 107.868,
        valences: &[1],
        electronegativity: Some(1.93),
    },
    Element {
        atomic_number: 48,
        symbol: "Cd",
        name: "Cadmium",
        mass: 112.414,
        valences: &[2],
        electronegativity: Some(1.69),
    },
    Element {
        atomic_number: 49,
        symbol: "In",
        name: "Indium",
        mass: 114.818,
        valences: &[3],
        electronegativity: Some(1.78),
    },
    Element {
        atomic_number: 50,
        symbol: "Sn",
        name: "Tin",
        mass: 118.710,
        valences: &[4, 2],
        electronegativity: Some(1.96),
    },
    Element {
        atomic_number: 51,
        symbol: "Sb",
        name: "Antimony",
        mass: 121.760,
        valences: &[3, 5],
        electronegativity: Some(2.05),
    },
    Element {
        atomic_number: 52,
        symbol: "Te",
        name: "Tellurium",
        mass: 127.60,
        valences: &[2, 4, 6],
        electronegativity: Some(2.10),
    },
    Element {
        atomic_number: 53,
        symbol: "I",
        name: "Iodine",
        mass: 126.904,
        valences: &[1],
        electronegativity: Some(2.66),
    },
    Element {
        atomic_number: 54,
        symbol: "Xe",
        name: "Xenon",
        mass: 131.293,
        valences: &[0],
        electronegativity: None,
    },
    // -------------------------------------------------------------------------
    // Period 6
    // -------------------------------------------------------------------------
    Element {
        atomic_number: 55,
        symbol: "Cs",
        name: "Cesium",
        mass: 132.905,
        valences: &[1],
        electronegativity: Some(0.79),
    },
    Element {
        atomic_number: 56,
        symbol: "Ba",
        name: "Barium",
        mass: 137.327,
        valences: &[2],
        electronegativity: Some(0.89),
    },
    // Lanthanides 57-71
    Element {
        atomic_number: 57,
        symbol: "La",
        name: "Lanthanum",
        mass: 138.905,
        valences: &[3],
        electronegativity: Some(1.10),
    },
    Element {
        atomic_number: 58,
        symbol: "Ce",
        name: "Cerium",
        mass: 140.116,
        valences: &[3, 4],
        electronegativity: Some(1.12),
    },
    Element {
        atomic_number: 59,
        symbol: "Pr",
        name: "Praseodymium",
        mass: 140.908,
        valences: &[3],
        electronegativity: Some(1.13),
    },
    Element {
        atomic_number: 60,
        symbol: "Nd",
        name: "Neodymium",
        mass: 144.242,
        valences: &[3],
        electronegativity: Some(1.14),
    },
    Element {
        atomic_number: 61,
        symbol: "Pm",
        name: "Promethium",
        mass: 145.0,
        valences: &[3],
        electronegativity: Some(1.13),
    },
    Element {
        atomic_number: 62,
        symbol: "Sm",
        name: "Samarium",
        mass: 150.36,
        valences: &[3, 2],
        electronegativity: Some(1.17),
    },
    Element {
        atomic_number: 63,
        symbol: "Eu",
        name: "Europium",
        mass: 151.964,
        valences: &[3, 2],
        electronegativity: Some(1.20),
    },
    Element {
        atomic_number: 64,
        symbol: "Gd",
        name: "Gadolinium",
        mass: 157.25,
        valences: &[3],
        electronegativity: Some(1.20),
    },
    Element {
        atomic_number: 65,
        symbol: "Tb",
        name: "Terbium",
        mass: 158.925,
        valences: &[3],
        electronegativity: Some(1.10),
    },
    Element {
        atomic_number: 66,
        symbol: "Dy",
        name: "Dysprosium",
        mass: 162.500,
        valences: &[3],
        electronegativity: Some(1.22),
    },
    Element {
        atomic_number: 67,
        symbol: "Ho",
        name: "Holmium",
        mass: 164.930,
        valences: &[3],
        electronegativity: Some(1.23),
    },
    Element {
        atomic_number: 68,
        symbol: "Er",
        name: "Erbium",
        mass: 167.259,
        valences: &[3],
        electronegativity: Some(1.24),
    },
    Element {
        atomic_number: 69,
        symbol: "Tm",
        name: "Thulium",
        mass: 168.934,
        valences: &[3],
        electronegativity: Some(1.25),
    },
    Element {
        atomic_number: 70,
        symbol: "Yb",
        name: "Ytterbium",
        mass: 173.045,
        valences: &[3, 2],
        electronegativity: Some(1.10),
    },
    Element {
        atomic_number: 71,
        symbol: "Lu",
        name: "Lutetium",
        mass: 174.967,
        valences: &[3],
        electronegativity: Some(1.27),
    },
    // Post-lanthanide Period 6 transition metals
    Element {
        atomic_number: 72,
        symbol: "Hf",
        name: "Hafnium",
        mass: 178.49,
        valences: &[4],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 73,
        symbol: "Ta",
        name: "Tantalum",
        mass: 180.948,
        valences: &[5],
        electronegativity: Some(1.50),
    },
    Element {
        atomic_number: 74,
        symbol: "W",
        name: "Tungsten",
        mass: 183.84,
        valences: &[6, 4, 2],
        electronegativity: Some(2.36),
    },
    Element {
        atomic_number: 75,
        symbol: "Re",
        name: "Rhenium",
        mass: 186.207,
        valences: &[7, 4],
        electronegativity: Some(1.90),
    },
    Element {
        atomic_number: 76,
        symbol: "Os",
        name: "Osmium",
        mass: 190.23,
        valences: &[4, 8, 3],
        electronegativity: Some(2.20),
    },
    Element {
        atomic_number: 77,
        symbol: "Ir",
        name: "Iridium",
        mass: 192.217,
        valences: &[3, 4],
        electronegativity: Some(2.20),
    },
    Element {
        atomic_number: 78,
        symbol: "Pt",
        name: "Platinum",
        mass: 195.084,
        valences: &[2, 4],
        electronegativity: Some(2.28),
    },
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
    Element {
        atomic_number: 81,
        symbol: "Tl",
        name: "Thallium",
        mass: 204.38,
        valences: &[1, 3],
        electronegativity: Some(1.62),
    },
    Element {
        atomic_number: 82,
        symbol: "Pb",
        name: "Lead",
        mass: 207.2,
        valences: &[2, 4],
        electronegativity: Some(2.33),
    },
    Element {
        atomic_number: 83,
        symbol: "Bi",
        name: "Bismuth",
        mass: 208.980,
        valences: &[3, 5],
        electronegativity: Some(2.02),
    },
    Element {
        atomic_number: 84,
        symbol: "Po",
        name: "Polonium",
        mass: 209.0,
        valences: &[2, 4],
        electronegativity: Some(2.00),
    },
    Element {
        atomic_number: 85,
        symbol: "At",
        name: "Astatine",
        mass: 210.0,
        valences: &[1],
        electronegativity: Some(2.20),
    },
    Element {
        atomic_number: 86,
        symbol: "Rn",
        name: "Radon",
        mass: 222.0,
        valences: &[0],
        electronegativity: None,
    },
    // -------------------------------------------------------------------------
    // Period 7
    // -------------------------------------------------------------------------
    Element {
        atomic_number: 87,
        symbol: "Fr",
        name: "Francium",
        mass: 223.0,
        valences: &[1],
        electronegativity: Some(0.70),
    },
    Element {
        atomic_number: 88,
        symbol: "Ra",
        name: "Radium",
        mass: 226.0,
        valences: &[2],
        electronegativity: Some(0.90),
    },
    // Actinides 89-103
    Element {
        atomic_number: 89,
        symbol: "Ac",
        name: "Actinium",
        mass: 227.0,
        valences: &[3],
        electronegativity: Some(1.10),
    },
    Element {
        atomic_number: 90,
        symbol: "Th",
        name: "Thorium",
        mass: 232.038,
        valences: &[4],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 91,
        symbol: "Pa",
        name: "Protactinium",
        mass: 231.036,
        valences: &[5, 4],
        electronegativity: Some(1.50),
    },
    Element {
        atomic_number: 92,
        symbol: "U",
        name: "Uranium",
        mass: 238.029,
        valences: &[4, 6],
        electronegativity: Some(1.38),
    },
    Element {
        atomic_number: 93,
        symbol: "Np",
        name: "Neptunium",
        mass: 237.0,
        valences: &[5, 4],
        electronegativity: Some(1.36),
    },
    Element {
        atomic_number: 94,
        symbol: "Pu",
        name: "Plutonium",
        mass: 244.0,
        valences: &[4, 3, 5, 6],
        electronegativity: Some(1.28),
    },
    Element {
        atomic_number: 95,
        symbol: "Am",
        name: "Americium",
        mass: 243.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 96,
        symbol: "Cm",
        name: "Curium",
        mass: 247.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 97,
        symbol: "Bk",
        name: "Berkelium",
        mass: 247.0,
        valences: &[3, 4],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 98,
        symbol: "Cf",
        name: "Californium",
        mass: 251.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 99,
        symbol: "Es",
        name: "Einsteinium",
        mass: 252.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 100,
        symbol: "Fm",
        name: "Fermium",
        mass: 257.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 101,
        symbol: "Md",
        name: "Mendelevium",
        mass: 258.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 102,
        symbol: "No",
        name: "Nobelium",
        mass: 259.0,
        valences: &[2, 3],
        electronegativity: Some(1.30),
    },
    Element {
        atomic_number: 103,
        symbol: "Lr",
        name: "Lawrencium",
        mass: 266.0,
        valences: &[3],
        electronegativity: Some(1.30),
    },
    // Post-actinide Period 7 transition metals (superheavy)
    Element {
        atomic_number: 104,
        symbol: "Rf",
        name: "Rutherfordium",
        mass: 267.0,
        valences: &[4],
        electronegativity: None,
    },
    Element {
        atomic_number: 105,
        symbol: "Db",
        name: "Dubnium",
        mass: 268.0,
        valences: &[5],
        electronegativity: None,
    },
    Element {
        atomic_number: 106,
        symbol: "Sg",
        name: "Seaborgium",
        mass: 269.0,
        valences: &[6],
        electronegativity: None,
    },
    Element {
        atomic_number: 107,
        symbol: "Bh",
        name: "Bohrium",
        mass: 270.0,
        valences: &[7],
        electronegativity: None,
    },
    Element {
        atomic_number: 108,
        symbol: "Hs",
        name: "Hassium",
        mass: 277.0,
        valences: &[8],
        electronegativity: None,
    },
    Element {
        atomic_number: 109,
        symbol: "Mt",
        name: "Meitnerium",
        mass: 278.0,
        valences: &[3],
        electronegativity: None,
    },
    Element {
        atomic_number: 110,
        symbol: "Ds",
        name: "Darmstadtium",
        mass: 281.0,
        valences: &[2],
        electronegativity: None,
    },
    Element {
        atomic_number: 111,
        symbol: "Rg",
        name: "Roentgenium",
        mass: 282.0,
        valences: &[1, 3],
        electronegativity: None,
    },
    Element {
        atomic_number: 112,
        symbol: "Cn",
        name: "Copernicium",
        mass: 285.0,
        valences: &[2],
        electronegativity: None,
    },
    Element {
        atomic_number: 113,
        symbol: "Nh",
        name: "Nihonium",
        mass: 286.0,
        valences: &[1, 3],
        electronegativity: None,
    },
    Element {
        atomic_number: 114,
        symbol: "Fl",
        name: "Flerovium",
        mass: 289.0,
        valences: &[2, 4],
        electronegativity: None,
    },
    Element {
        atomic_number: 115,
        symbol: "Mc",
        name: "Moscovium",
        mass: 290.0,
        valences: &[1, 3],
        electronegativity: None,
    },
    Element {
        atomic_number: 116,
        symbol: "Lv",
        name: "Livermorium",
        mass: 293.0,
        valences: &[2, 4],
        electronegativity: None,
    },
    Element {
        atomic_number: 117,
        symbol: "Ts",
        name: "Tennessine",
        mass: 294.0,
        valences: &[1],
        electronegativity: None,
    },
    Element {
        atomic_number: 118,
        symbol: "Og",
        name: "Oganesson",
        mass: 294.0,
        valences: &[0],
        electronegativity: None,
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

    #[test]
    fn test_all_118_elements_present() {
        assert_eq!(PERIODIC_TABLE.len(), 118);
    }

    #[test]
    fn test_selenium() {
        let se = Element::from_symbol("Se");
        assert!(se.is_some());
        let selenium = se.unwrap_or(&PERIODIC_TABLE[0]);
        assert_eq!(selenium.atomic_number, 34);
        // mass should be approximately 78.971
        let mass_diff = (selenium.mass - 78.971_f64).abs();
        assert!(mass_diff < 0.01, "Selenium mass out of expected range: {}", selenium.mass);
    }

    #[test]
    fn test_platinum() {
        let pt = Element::from_symbol("Pt");
        assert!(pt.is_some());
        let platinum = pt.unwrap_or(&PERIODIC_TABLE[0]);
        assert_eq!(platinum.atomic_number, 78);
    }

    #[test]
    fn test_from_atomic_number_optimized() {
        for n in 1_u8..=118 {
            let elem = Element::from_atomic_number(n);
            assert!(
                elem.is_some(),
                "from_atomic_number({n}) returned None — element missing from PERIODIC_TABLE"
            );
            let e = elem.unwrap_or(&PERIODIC_TABLE[0]);
            assert_eq!(
                e.atomic_number, n,
                "Element at index {n} has wrong atomic_number: {}",
                e.atomic_number
            );
        }
    }

    #[test]
    fn test_from_atomic_number_out_of_range() {
        assert!(
            Element::from_atomic_number(0).is_none(),
            "from_atomic_number(0) should return None"
        );
        assert!(
            Element::from_atomic_number(119).is_none(),
            "from_atomic_number(119) should return None"
        );
    }
}
