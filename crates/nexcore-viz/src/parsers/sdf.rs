//! MDL SDF / MOL V2000 parser.
//!
//! Parses Structure Data Files (SDF) and standalone MOL files conforming to the
//! V2000 connection-table format defined in the Elsevier MDL CTfile Formats
//! specification (2011 edition).
//!
//! # Format overview
//!
//! A MOL block is structured as:
//!
//! ```text
//! Line 1  — molecule name
//! Line 2  — program / timestamp metadata
//! Line 3  — comment
//! Line 4  — counts line: aaabbblllfffcccsssxxxrrrpppiiimmmvvvvvv
//!             aaa = number of atoms  (chars 1-3)
//!             bbb = number of bonds  (chars 4-6)
//!             ...                    (fields 7-11 are legacy / unused here)
//!             vvvvvv = "V2000"       (chars 35-39)
//! Atom block (one line per atom):
//!   xxxxx.xxxxyyyyy.yyyyzzzzz.zzzz aaaddcccssshhhbbbvvvHHHrrriiimmmnnneee
//!   x/y/z = coordinates (10.4 fixed-point fields)
//!   aaa   = element symbol (chars 32-34, right-padded with spaces)
//! Bond block (one line per bond):
//!   111222tttsssxxxrrrccc
//!   111 = first atom number  (1-indexed, chars 1-3)
//!   222 = second atom number (1-indexed, chars 4-6)
//!   ttt = bond type          (chars 7-9)
//! Properties block:
//!   M  END — mandatory terminator
//! ```
//!
//! In an SDF the MOL block is followed by optional SD tag/value pairs and a
//! `$$$$` record separator.
//!
//! # Examples
//!
//! Parse a single MOL block:
//!
//! ```rust
//! use nexcore_viz::parsers::sdf::parse_mol;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mol_text = "\
//! Water
//!      RDKit          3D
//!
//!   3  2  0  0  0  0  0  0  0  0999 V2000
//!     0.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
//!     0.7570    0.5860    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
//!    -0.7570    0.5860    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
//!   1  2  1  0
//!   1  3  1  0
//! M  END
//! ";
//!     let mol = parse_mol(mol_text)?;
//!     assert_eq!(mol.name, "Water");
//!     assert_eq!(mol.atoms.len(), 3);
//!     assert_eq!(mol.bonds.len(), 2);
//!     Ok(())
//! }
//! ```
//!
//! Parse a multi-record SDF:
//!
//! ```rust
//! use nexcore_viz::parsers::sdf::parse_sdf;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let sdf = "\
//! Methane
//!
//!
//!   1  0  0  0  0  0  0  0  0  0999 V2000
//!     0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
//! M  END
//! $$$$
//! ";
//!     let molecules = parse_sdf(sdf)?;
//!     assert_eq!(molecules.len(), 1);
//!     assert_eq!(
//!         molecules[0].atoms[0].element,
//!         nexcore_viz::molecular::Element::C
//!     );
//!     Ok(())
//! }
//! ```

use crate::molecular::{Atom, Bond, BondOrder, Element, Molecule};

// ============================================================================
// Error type
// ============================================================================

/// Errors produced by the SDF / MOL V2000 parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input ended before the required header lines were found.
    ///
    /// Contains the number of lines actually present and the number expected.
    UnexpectedEof {
        /// Lines seen before EOF.
        found: usize,
        /// Lines the parser needed to continue.
        needed: usize,
    },

    /// The counts line (line 4) could not be decoded.
    ///
    /// Contains the offending line content and a description of what failed.
    MalformedCountsLine {
        /// Raw text of the offending line.
        line: String,
        /// Human-readable description of the parsing failure.
        reason: &'static str,
    },

    /// An atom line in the atom block could not be decoded.
    MalformedAtomLine {
        /// 1-based line number within the input text.
        line_number: usize,
        /// Raw text of the offending line.
        line: String,
    },

    /// A bond line in the bond block could not be decoded.
    MalformedBondLine {
        /// 1-based line number within the input text.
        line_number: usize,
        /// Raw text of the offending line.
        line: String,
    },

    /// A bond references an atom index that is out of range.
    BondAtomOutOfRange {
        /// 1-based atom index from the file.
        atom_index: usize,
        /// Number of atoms parsed.
        atom_count: usize,
    },

    /// `M  END` terminator was not found before the end of input.
    MissingMEnd,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof { found, needed } => write!(
                f,
                "unexpected EOF: found {found} lines, needed at least {needed}"
            ),
            Self::MalformedCountsLine { line, reason } => {
                write!(f, "malformed counts line ({reason}): {line:?}")
            }
            Self::MalformedAtomLine { line_number, line } => {
                write!(f, "malformed atom line {line_number}: {line:?}")
            }
            Self::MalformedBondLine { line_number, line } => {
                write!(f, "malformed bond line {line_number}: {line:?}")
            }
            Self::BondAtomOutOfRange {
                atom_index,
                atom_count,
            } => write!(
                f,
                "bond references atom {atom_index} but only {atom_count} atoms were parsed"
            ),
            Self::MissingMEnd => write!(f, "M  END terminator not found before end of input"),
        }
    }
}

impl core::error::Error for ParseError {}

// ============================================================================
// Public API
// ============================================================================

/// Parse a single MOL V2000 block from `input`.
///
/// The input may optionally contain trailing `$$$$` and SD tag lines; those are
/// silently ignored.  Only the first MOL block is parsed.
///
/// # Errors
///
/// Returns [`ParseError`] if the input violates V2000 format constraints.
///
/// # Examples
///
/// ```rust
/// use nexcore_viz::parsers::sdf::parse_mol;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let src = "\
/// Ethane
///
///
///   2  1  0  0  0  0  0  0  0  0999 V2000
///     0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
///     1.5400    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
///   1  2  1  0
/// M  END
/// ";
///     let mol = parse_mol(src)?;
///     assert_eq!(mol.atoms.len(), 2);
///     assert_eq!(mol.bonds.len(), 1);
///     Ok(())
/// }
/// ```
pub fn parse_mol(input: &str) -> Result<Molecule, ParseError> {
    let lines: Vec<&str> = input.lines().collect();
    parse_mol_block(&lines, 0).map(|(mol, _)| mol)
}

/// Parse all MOL records from an SDF string.
///
/// Records are separated by `$$$$` lines.  The function returns all
/// successfully parsed records, stopping and returning an error on the first
/// record that fails to parse.
///
/// # Errors
///
/// Returns the first [`ParseError`] encountered while parsing any record.
///
/// # Examples
///
/// ```rust
/// use nexcore_viz::parsers::sdf::parse_sdf;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let sdf = "\
/// Methane
///
///
///   1  0  0  0  0  0  0  0  0  0999 V2000
///     0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
/// M  END
/// $$$$
/// ";
///     let molecules = parse_sdf(sdf)?;
///     assert_eq!(molecules.len(), 1);
///     Ok(())
/// }
/// ```
pub fn parse_sdf(input: &str) -> Result<Vec<Molecule>, ParseError> {
    let lines: Vec<&str> = input.lines().collect();
    let mut molecules = Vec::new();
    let mut cursor = 0_usize;

    while cursor < lines.len() {
        // Skip blank lines between records (tolerates extra blank lines after $$$$)
        if lines[cursor].trim().is_empty() {
            cursor += 1;
            continue;
        }

        // A `$$$$` line between records — skip it
        if lines[cursor].trim() == "$$$$" {
            cursor += 1;
            continue;
        }

        let (mol, next_cursor) = parse_mol_block(&lines, cursor)?;
        molecules.push(mol);
        cursor = next_cursor;
    }

    Ok(molecules)
}

// ============================================================================
// Core parsing logic
// ============================================================================

/// Parse one MOL block starting at `start` within `lines`.
///
/// Returns the parsed [`Molecule`] and the index of the first line *after* the
/// record (i.e. after `$$$$` if present, or after `M  END` if no separator).
fn parse_mol_block(lines: &[&str], start: usize) -> Result<(Molecule, usize), ParseError> {
    // ---- Header block (lines 1-3) ------------------------------------------
    // Lines are 0-indexed in the slice; "line N" in spec means lines[start + N - 1].
    if lines.len() < start + 4 {
        return Err(ParseError::UnexpectedEof {
            found: lines.len().saturating_sub(start),
            needed: 4,
        });
    }

    let mol_name = lines[start].trim().to_string();
    // lines[start + 1] — program/timestamp: ignored
    // lines[start + 2] — comment: ignored

    // ---- Counts line (line 4) -----------------------------------------------
    let counts_line = lines[start + 3];
    let (num_atoms, num_bonds) = parse_counts_line(counts_line)?;

    // ---- Atom block ---------------------------------------------------------
    let atom_block_start = start + 4;
    let atom_block_end = atom_block_start + num_atoms;

    if lines.len() < atom_block_end {
        return Err(ParseError::UnexpectedEof {
            found: lines.len().saturating_sub(atom_block_start),
            needed: num_atoms,
        });
    }

    let mut atoms = Vec::with_capacity(num_atoms);
    for i in 0..num_atoms {
        let line_idx = atom_block_start + i;
        let line = lines[line_idx];
        let atom = parse_atom_line(line, i + 1, line_idx + 1)?;
        atoms.push(atom);
    }

    // ---- Bond block ---------------------------------------------------------
    let bond_block_start = atom_block_end;
    let bond_block_end = bond_block_start + num_bonds;

    if lines.len() < bond_block_end {
        return Err(ParseError::UnexpectedEof {
            found: lines.len().saturating_sub(bond_block_start),
            needed: num_bonds,
        });
    }

    let mut bonds = Vec::with_capacity(num_bonds);
    for i in 0..num_bonds {
        let line_idx = bond_block_start + i;
        let line = lines[line_idx];
        let bond = parse_bond_line(line, line_idx + 1, atoms.len())?;
        bonds.push(bond);
    }

    // ---- Properties block — scan for M  END --------------------------------
    let mut cursor = bond_block_end;
    let mut found_m_end = false;

    while cursor < lines.len() {
        let trimmed = lines[cursor].trim();

        if trimmed == "M  END" {
            found_m_end = true;
            cursor += 1;
            break;
        }

        // Any `$$$$` without a preceding `M  END` is format-violating but some
        // generators omit the terminator.  Accept it gracefully.
        if trimmed == "$$$$" {
            found_m_end = true; // treat as implicit end
            cursor += 1;
            break;
        }

        cursor += 1;
    }

    if !found_m_end {
        return Err(ParseError::MissingMEnd);
    }

    // ---- Skip trailing SD tag/value pairs until $$$$ -----------------------
    while cursor < lines.len() {
        if lines[cursor].trim() == "$$$$" {
            cursor += 1;
            break;
        }
        cursor += 1;
    }

    // ---- Assemble Molecule --------------------------------------------------
    let mut mol = Molecule::new(&mol_name);
    mol.atoms = atoms;
    mol.bonds = bonds;
    mol.source_format = Some("SDF/V2000".to_string());

    Ok((mol, cursor))
}

// ============================================================================
// Field parsers
// ============================================================================

/// Parse the counts line and return `(num_atoms, num_bonds)`.
///
/// V2000 layout (1-based, inclusive):
/// - chars 1-3  : number of atoms
/// - chars 4-6  : number of bonds
/// - chars 35-39: "V2000"
///
/// Fields are right-justified and space-padded, so we trim before parsing.
fn parse_counts_line(line: &str) -> Result<(usize, usize), ParseError> {
    // Minimum meaningful length: 6 chars (atom + bond counts)
    if line.len() < 6 {
        return Err(ParseError::MalformedCountsLine {
            line: line.to_string(),
            reason: "line too short to contain atom and bond counts",
        });
    }

    let atom_field = &line[0..3];
    let bond_field = &line[3..6];

    let num_atoms =
        atom_field
            .trim()
            .parse::<usize>()
            .map_err(|_| ParseError::MalformedCountsLine {
                line: line.to_string(),
                reason: "atom count field is not a valid integer",
            })?;

    let num_bonds =
        bond_field
            .trim()
            .parse::<usize>()
            .map_err(|_| ParseError::MalformedCountsLine {
                line: line.to_string(),
                reason: "bond count field is not a valid integer",
            })?;

    Ok((num_atoms, num_bonds))
}

/// Parse a single atom line from the V2000 atom block.
///
/// V2000 atom line layout (1-based, inclusive):
/// - chars 1-10  : x coordinate (10.4 fixed-point)
/// - chars 11-20 : y coordinate (10.4 fixed-point)
/// - chars 21-30 : z coordinate (10.4 fixed-point)
/// - char  31    : space
/// - chars 32-34 : element symbol (left-justified, space-padded)
///
/// `atom_id` is 1-based (as the SDF spec states).
/// `line_number` is the 1-based index within the full input (for error reporting).
fn parse_atom_line(line: &str, atom_id: usize, line_number: usize) -> Result<Atom, ParseError> {
    // Minimum length to reach end of element field (char 34 = index 33)
    if line.len() < 34 {
        return Err(ParseError::MalformedAtomLine {
            line_number,
            line: line.to_string(),
        });
    }

    let make_err = || ParseError::MalformedAtomLine {
        line_number,
        line: line.to_string(),
    };

    let x = line[0..10].trim().parse::<f64>().map_err(|_| make_err())?;
    let y = line[10..20].trim().parse::<f64>().map_err(|_| make_err())?;
    let z = line[20..30].trim().parse::<f64>().map_err(|_| make_err())?;

    // Element symbol occupies chars 32-34 (0-indexed: 31..34).
    // Char 31 is a mandatory space separator — we skip it.
    let symbol = &line[31..34];
    let element = Element::from_symbol(symbol);

    Ok(Atom::new(atom_id as u32, element, [x, y, z]))
}

/// Parse a single bond line from the V2000 bond block.
///
/// V2000 bond line layout (1-based, inclusive):
/// - chars 1-3  : first atom number  (1-indexed)
/// - chars 4-6  : second atom number (1-indexed)
/// - chars 7-9  : bond type (1=single, 2=double, 3=triple, 4=aromatic)
///
/// Atom indices are validated against `atom_count`.
/// Returned [`Bond`] uses 0-based indices.
fn parse_bond_line(line: &str, line_number: usize, atom_count: usize) -> Result<Bond, ParseError> {
    if line.len() < 9 {
        return Err(ParseError::MalformedBondLine {
            line_number,
            line: line.to_string(),
        });
    }

    let make_err = || ParseError::MalformedBondLine {
        line_number,
        line: line.to_string(),
    };

    let a1_raw = line[0..3].trim().parse::<usize>().map_err(|_| make_err())?;
    let a2_raw = line[3..6].trim().parse::<usize>().map_err(|_| make_err())?;
    let order_raw = line[6..9].trim().parse::<u8>().map_err(|_| make_err())?;

    // Validate — SDF uses 1-based indices
    if a1_raw == 0 || a1_raw > atom_count {
        return Err(ParseError::BondAtomOutOfRange {
            atom_index: a1_raw,
            atom_count,
        });
    }
    if a2_raw == 0 || a2_raw > atom_count {
        return Err(ParseError::BondAtomOutOfRange {
            atom_index: a2_raw,
            atom_count,
        });
    }

    Ok(Bond {
        atom1: a1_raw - 1, // convert to 0-based
        atom2: a2_raw - 1,
        order: BondOrder::from_sdf(order_raw),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{BondOrder, Element};

    // ---- Aspirin test fixture -----------------------------------------------

    const ASPIRIN_SDF: &str = "\
Aspirin
     RDKit          3D

 21 21  0  0  0  0  0  0  0  0999 V2000
    1.2333    0.5540    0.7792 O   0  0  0  0  0  0  0  0  0  0  0  0
   -0.6952   -2.7148   -0.7502 O   0  0  0  0  0  0  0  0  0  0  0  0
    0.7958   -2.1843    0.8685 O   0  0  0  0  0  0  0  0  0  0  0  0
    1.7813    0.8105   -1.4821 O   0  0  0  0  0  0  0  0  0  0  0  0
   -0.0857    0.6088    0.4403 C   0  0  0  0  0  0  0  0  0  0  0  0
   -0.7927   -0.5515    0.1244 C   0  0  0  0  0  0  0  0  0  0  0  0
   -0.7288    1.8464    0.4133 C   0  0  0  0  0  0  0  0  0  0  0  0
   -2.1426   -0.4741   -0.2184 C   0  0  0  0  0  0  0  0  0  0  0  0
   -2.0787    1.9238    0.0706 C   0  0  0  0  0  0  0  0  0  0  0  0
   -2.7855    0.7636   -0.2453 C   0  0  0  0  0  0  0  0  0  0  0  0
   -0.1409   -1.8536    0.1477 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.0103   -0.1635   -0.3930 C   0  0  0  0  0  0  0  0  0  0  0  0
    3.5163   -0.5426   -0.3412 C   0  0  0  0  0  0  0  0  0  0  0  0
   -0.2555    2.7769    0.6593 H   0  0  0  0  0  0  0  0  0  0  0  0
   -2.6176   -1.3902   -0.4580 H   0  0  0  0  0  0  0  0  0  0  0  0
   -2.5635    2.8872    0.0506 H   0  0  0  0  0  0  0  0  0  0  0  0
   -3.8348    0.8238   -0.5090 H   0  0  0  0  0  0  0  0  0  0  0  0
    3.8781   -0.7190    0.6719 H   0  0  0  0  0  0  0  0  0  0  0  0
    4.0363    0.2992   -0.7822 H   0  0  0  0  0  0  0  0  0  0  0  0
    3.6088   -1.4325   -0.9620 H   0  0  0  0  0  0  0  0  0  0  0  0
   -1.6160   -2.4260   -0.6920 H   0  0  0  0  0  0  0  0  0  0  0  0
  1  5  1  0
  1 12  1  0
  2 11  1  0
  2 21  1  0
  3 11  2  0
  4 12  2  0
  5  6  2  0
  5  7  1  0
  6  8  1  0
  6 11  1  0
  7  9  2  0
  7 14  1  0
  8 10  2  0
  8 15  1  0
  9 10  1  0
  9 16  1  0
 10 17  1  0
 12 13  1  0
 13 18  1  0
 13 19  1  0
 13 20  1  0
M  END
$$$$
";

    // Helper: parse aspirin or fail the test with the error message.
    fn aspirin() -> Molecule {
        match parse_mol(ASPIRIN_SDF) {
            Ok(mol) => mol,
            Err(e) => {
                // Returning a default Molecule and asserting on it would give
                // confusing failures; surfacing the error message is clearer.
                // We use a deliberate assertion failure here — this only runs
                // inside #[cfg(test)] and is the canonical way to abort a test
                // with a descriptive message without unwrap/expect.
                assert!(false, "ASPIRIN_SDF failed to parse: {e}");
                // Unreachable but needed for type inference:
                Molecule::new("")
            }
        }
    }

    // ---- parse_mol tests ---------------------------------------------------

    #[test]
    fn aspirin_mol_name() {
        assert_eq!(aspirin().name, "Aspirin");
    }

    #[test]
    fn aspirin_atom_count() {
        assert_eq!(aspirin().atoms.len(), 21);
    }

    #[test]
    fn aspirin_bond_count() {
        assert_eq!(aspirin().bonds.len(), 21);
    }

    #[test]
    fn aspirin_source_format() {
        assert_eq!(aspirin().source_format.as_deref(), Some("SDF/V2000"));
    }

    #[test]
    fn aspirin_atom_serial_ids_are_one_based() {
        let mol = aspirin();
        for (i, atom) in mol.atoms.iter().enumerate() {
            assert_eq!(atom.id as usize, i + 1);
        }
    }

    #[test]
    fn aspirin_first_atom_is_oxygen() {
        let mol = aspirin();
        assert_eq!(mol.atoms[0].element, Element::O);
    }

    #[test]
    fn aspirin_fifth_atom_is_carbon() {
        // Atom 5 (index 4) is the first ring carbon
        assert_eq!(aspirin().atoms[4].element, Element::C);
    }

    #[test]
    fn aspirin_first_atom_coordinates() {
        let pos = aspirin().atoms[0].position;
        assert!((pos[0] - 1.2333).abs() < 1e-4, "x = {}", pos[0]);
        assert!((pos[1] - 0.5540).abs() < 1e-4, "y = {}", pos[1]);
        assert!((pos[2] - 0.7792).abs() < 1e-4, "z = {}", pos[2]);
    }

    #[test]
    fn aspirin_element_distribution() {
        let mol = aspirin();
        let counts = mol.element_counts();
        // Aspirin: C9H8O4
        assert_eq!(counts.get(&Element::C).copied().unwrap_or(0), 9);
        assert_eq!(counts.get(&Element::H).copied().unwrap_or(0), 8);
        assert_eq!(counts.get(&Element::O).copied().unwrap_or(0), 4);
    }

    #[test]
    fn aspirin_molecular_formula() {
        assert_eq!(aspirin().formula(), "C9H8O4");
    }

    #[test]
    fn aspirin_bond_zero_indexed() {
        // Bond 1: atoms 1 and 5 in file → indices 0 and 4
        let mol = aspirin();
        let b = &mol.bonds[0];
        assert_eq!(b.atom1, 0);
        assert_eq!(b.atom2, 4);
    }

    #[test]
    fn aspirin_single_bond_order() {
        // Bond 1 (1-5) is single
        assert_eq!(aspirin().bonds[0].order, BondOrder::Single);
    }

    #[test]
    fn aspirin_double_bond_order() {
        // Bond 5 (3-11, order 2) is double (index 4)
        assert_eq!(aspirin().bonds[4].order, BondOrder::Double);
    }

    #[test]
    fn aspirin_molecular_weight_approx() {
        let mw = aspirin().molecular_weight();
        // Aspirin MW ~180.16 g/mol
        assert!((mw - 180.16).abs() < 0.5, "MW = {mw}");
    }

    #[test]
    fn aspirin_no_chains() {
        let mol = aspirin();
        assert!(mol.chains.is_empty());
        assert!(!mol.is_protein());
    }

    #[test]
    fn aspirin_bounding_box_sanity() {
        let (min, max) = aspirin().bounding_box();
        // x range spans ~8 Å for aspirin
        assert!(max[0] - min[0] > 4.0);
        assert!(max[1] - min[1] > 3.0);
    }

    // ---- parse_sdf tests ---------------------------------------------------

    #[test]
    fn sdf_single_record_produces_one_molecule() {
        match parse_sdf(ASPIRIN_SDF) {
            Ok(molecules) => assert_eq!(molecules.len(), 1),
            Err(e) => assert!(false, "parse_sdf failed: {e}"),
        }
    }

    #[test]
    fn sdf_two_records() {
        let two_record = format!("{ASPIRIN_SDF}{ASPIRIN_SDF}");
        match parse_sdf(&two_record) {
            Ok(molecules) => assert_eq!(molecules.len(), 2),
            Err(e) => assert!(false, "parse_sdf failed: {e}"),
        }
    }

    #[test]
    fn sdf_preserves_names_across_records() {
        let two_record = format!("{ASPIRIN_SDF}{ASPIRIN_SDF}");
        match parse_sdf(&two_record) {
            Ok(molecules) => {
                assert_eq!(molecules[0].name, "Aspirin");
                assert_eq!(molecules[1].name, "Aspirin");
            }
            Err(e) => assert!(false, "parse_sdf failed: {e}"),
        }
    }

    #[test]
    fn sdf_single_atom_no_bonds() {
        let src = "\
Methane


  1  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
M  END
$$$$
";
        match parse_sdf(src) {
            Ok(molecules) => {
                assert_eq!(molecules.len(), 1);
                assert_eq!(molecules[0].atoms.len(), 1);
                assert!(molecules[0].bonds.is_empty());
                assert_eq!(molecules[0].atoms[0].element, Element::C);
            }
            Err(e) => assert!(false, "parse_sdf failed: {e}"),
        }
    }

    #[test]
    fn parse_mol_without_dollar_separator() {
        // Some generators omit the $$$$ at end of file
        let src = "\
Water


  3  2  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
    0.7570    0.5860    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
   -0.7570    0.5860    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0
  1  3  1  0
M  END
";
        match parse_mol(src) {
            Ok(mol) => {
                assert_eq!(mol.name, "Water");
                assert_eq!(mol.atoms.len(), 3);
            }
            Err(e) => assert!(false, "parse_mol failed: {e}"),
        }
    }

    // ---- Error path tests --------------------------------------------------

    #[test]
    fn error_on_empty_input() {
        let result = parse_mol("");
        assert!(matches!(result, Err(ParseError::UnexpectedEof { .. })));
    }

    #[test]
    fn error_on_truncated_header() {
        let result = parse_mol("Just one line\n");
        assert!(matches!(result, Err(ParseError::UnexpectedEof { .. })));
    }

    #[test]
    fn error_on_bad_counts_line() {
        let src = "\
Bad


XXXX BOND
M  END
";
        let result = parse_mol(src);
        assert!(
            matches!(result, Err(ParseError::MalformedCountsLine { .. })),
            "expected MalformedCountsLine, got {result:?}"
        );
    }

    #[test]
    fn error_on_truncated_atom_block() {
        let src = "\
TruncAtom


  3  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
M  END
";
        // Claims 3 atoms but provides only 1
        let result = parse_mol(src);
        assert!(
            matches!(result, Err(ParseError::UnexpectedEof { .. })),
            "expected UnexpectedEof, got {result:?}"
        );
    }

    #[test]
    fn error_on_bad_atom_coordinates() {
        let src = "\
BadCoords


  1  0  0  0  0  0  0  0  0  0999 V2000
    X.XXXX    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
M  END
";
        let result = parse_mol(src);
        assert!(
            matches!(result, Err(ParseError::MalformedAtomLine { .. })),
            "expected MalformedAtomLine, got {result:?}"
        );
    }

    #[test]
    fn error_on_bond_out_of_range() {
        let src = "\
OutOfRange


  1  1  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
  1  9  1  0
M  END
";
        // Bond references atom 9 but only 1 atom exists
        let result = parse_mol(src);
        assert!(
            matches!(
                result,
                Err(ParseError::BondAtomOutOfRange {
                    atom_index: 9,
                    atom_count: 1
                })
            ),
            "expected BondAtomOutOfRange, got {result:?}"
        );
    }

    #[test]
    fn error_on_missing_m_end() {
        let src = "\
NoEnd


  1  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
";
        let result = parse_mol(src);
        assert!(
            matches!(result, Err(ParseError::MissingMEnd)),
            "expected MissingMEnd, got {result:?}"
        );
    }

    #[test]
    fn parse_error_display_unexpected_eof() {
        let e = ParseError::UnexpectedEof {
            found: 2,
            needed: 4,
        };
        let s = e.to_string();
        assert!(s.contains("2"));
        assert!(s.contains("4"));
    }

    #[test]
    fn parse_error_display_malformed_counts() {
        let e = ParseError::MalformedCountsLine {
            line: "bad".to_string(),
            reason: "test reason",
        };
        let s = e.to_string();
        assert!(s.contains("test reason"));
        assert!(s.contains("bad"));
    }

    #[test]
    fn parse_error_display_bond_out_of_range() {
        let e = ParseError::BondAtomOutOfRange {
            atom_index: 5,
            atom_count: 3,
        };
        let s = e.to_string();
        assert!(s.contains('5'));
        assert!(s.contains('3'));
    }

    // ---- Miscellaneous correctness tests ------------------------------------

    #[test]
    fn unknown_element_maps_to_other() {
        let src = "\
Unknown


  1  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 Xx  0  0  0  0  0  0  0  0  0  0  0  0
M  END
$$$$
";
        match parse_mol(src) {
            Ok(mol) => assert_eq!(mol.atoms[0].element, Element::Other),
            Err(e) => assert!(false, "parse_mol failed: {e}"),
        }
    }

    #[test]
    fn aromatic_bond_order_parsed() {
        let src = "\
Benzene


  6  6  0  0  0  0  0  0  0  0999 V2000
    0.0000    1.4000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.2124    0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.2124   -0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    0.0000   -1.4000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
   -1.2124   -0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
   -1.2124    0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  4  0
  2  3  4  0
  3  4  4  0
  4  5  4  0
  5  6  4  0
  6  1  4  0
M  END
$$$$
";
        match parse_mol(src) {
            Ok(mol) => {
                for bond in &mol.bonds {
                    assert_eq!(bond.order, BondOrder::Aromatic);
                }
            }
            Err(e) => assert!(false, "parse_mol failed: {e}"),
        }
    }

    #[test]
    fn triple_bond_order_parsed() {
        let src = "\
Acetylene


  2  1  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.2000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  3  0
M  END
$$$$
";
        match parse_mol(src) {
            Ok(mol) => assert_eq!(mol.bonds[0].order, BondOrder::Triple),
            Err(e) => assert!(false, "parse_mol failed: {e}"),
        }
    }

    #[test]
    fn aspirin_bonded_atoms_of_first_oxygen() {
        // Atom 1 (O, index 0) bonds to atoms 5 and 12 (indices 4 and 11)
        let mol = aspirin();
        let mut neighbors = mol.bonded_to(0);
        neighbors.sort_unstable();
        assert_eq!(neighbors, vec![4, 11]);
    }

    #[test]
    fn empty_name_allowed() {
        let src = "


  1  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
M  END
$$$$
";
        let mol = parse_mol(src);
        assert!(mol.is_ok(), "parse_mol failed: {:?}", mol.err());
        let m = mol.ok();
        assert_eq!(m.map(|v| v.name.clone()), Some(String::new()));
    }
}
