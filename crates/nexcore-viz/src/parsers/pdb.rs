//! PDB (Protein Data Bank) file format parser.
//!
//! Parses the PDB flat-file format (wwPDB format v3.30) and produces a
//! [`crate::molecular::Molecule`] with full chain/residue hierarchy,
//! secondary-structure annotations, and covalent bonds.
//!
//! # Record types handled
//!
//! | Record   | Columns parsed |
//! |----------|----------------|
//! | `ATOM`   | serial(7-11), name(13-16), resName(18-20), chainID(22), resSeq(23-26), x(31-38), y(39-46), z(47-54), occupancy(55-60), tempFactor(61-66), element(77-78) |
//! | `HETATM` | same as ATOM |
//! | `CONECT` | atom serial + up to 4 bonded serials |
//! | `HELIX`  | initChainID(20), initSeqNum(22-25), endChainID(32), endSeqNum(34-37), helixClass(39-40) |
//! | `SHEET`  | initChainID(22), initSeqNum(23-26), endChainID(33), endSeqNum(34-37) |
//! | `END` / `ENDMDL` | terminate parsing |
//!
//! # Bond inference
//!
//! 1. If the file contains `CONECT` records they are used as-is (Single order).
//! 2. If no `CONECT` records are present, distance-based detection is applied:
//!    a bond is added when
//!    `distance(a, b) < (covalent_radius(a) + covalent_radius(b)) × 1.2`.
//!
//! # Example
//!
//! ```text
//! // Pseudocode — see parse_pdb for the real function signature.
//! // let mol = parse_pdb(pdb_string)?;
//! // assert_eq!(mol.chains.len(), 1);
//! ```

use std::collections::HashMap;

use crate::molecular::{Atom, Bond, BondOrder, Chain, Element, Molecule, Residue, SecondaryStructure};

// ============================================================================
// Error type
// ============================================================================

/// Errors produced by the PDB parser.
///
/// Carries the 1-indexed line number and a human-readable description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-indexed line number where the error occurred.
    pub line: usize,
    /// Human-readable description.
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self { line, message: message.into() }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PDB parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// Secondary structure range key
// ============================================================================

/// Identifies a contiguous secondary-structure range on a single chain.
#[derive(Debug, Clone)]
struct SecStrRange {
    chain_id: char,
    init_seq: i32,
    end_seq: i32,
    kind: SecondaryStructure,
}

// ============================================================================
// Column extraction helpers
// ============================================================================

/// Extract a fixed-width column from a PDB line (1-indexed, inclusive).
///
/// Returns `""` if the line is shorter than the requested range.
fn col(line: &str, start: usize, end: usize) -> &str {
    // PDB columns are 1-indexed; convert to 0-indexed byte offsets.
    let s = start.saturating_sub(1);
    let e = end;
    let bytes = line.as_bytes();
    if s >= bytes.len() {
        return "";
    }
    let e = e.min(bytes.len());
    // PDB files are ASCII; byte-slice boundaries are always char boundaries.
    &line[s..e]
}

/// Parse a column as a trimmed string, returning `None` if blank.
fn col_str(line: &str, start: usize, end: usize) -> Option<String> {
    let s = col(line, start, end).trim();
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Parse a column as `i32`, returning `None` on blank or parse failure.
fn col_i32(line: &str, start: usize, end: usize) -> Option<i32> {
    col(line, start, end).trim().parse().ok()
}

/// Parse a column as `u32`, returning `None` on blank or parse failure.
fn col_u32(line: &str, start: usize, end: usize) -> Option<u32> {
    col(line, start, end).trim().parse().ok()
}

/// Parse a column as `f64`, returning `None` on blank or parse failure.
fn col_f64(line: &str, start: usize, end: usize) -> Option<f64> {
    col(line, start, end).trim().parse().ok()
}

/// Extract a single ASCII character from a 1-indexed column position.
fn col_char(line: &str, pos: usize) -> Option<char> {
    col(line, pos, pos).trim().chars().next()
}

// ============================================================================
// ATOM / HETATM record
// ============================================================================

/// Data extracted from one ATOM or HETATM record.
#[derive(Debug)]
struct AtomRecord {
    serial: u32,
    name: String,
    res_name: String,
    chain_id: char,
    res_seq: i32,
    x: f64,
    y: f64,
    z: f64,
    b_factor: Option<f64>,
    element: Element,
}

/// Parse one ATOM or HETATM line.
///
/// Returns `Err` if any mandatory field is missing or malformed.
fn parse_atom_line(line: &str, lineno: usize) -> Result<AtomRecord, ParseError> {
    let serial = col_u32(line, 7, 11)
        .ok_or_else(|| ParseError::new(lineno, "missing atom serial"))?;
    let name = col_str(line, 13, 16)
        .ok_or_else(|| ParseError::new(lineno, "missing atom name"))?;
    let res_name = col_str(line, 18, 20)
        .ok_or_else(|| ParseError::new(lineno, "missing residue name"))?;
    let chain_id = col_char(line, 22)
        .ok_or_else(|| ParseError::new(lineno, "missing chain ID"))?;
    let res_seq = col_i32(line, 23, 26)
        .ok_or_else(|| ParseError::new(lineno, "missing residue seq"))?;
    let x = col_f64(line, 31, 38)
        .ok_or_else(|| ParseError::new(lineno, "missing x coordinate"))?;
    let y = col_f64(line, 39, 46)
        .ok_or_else(|| ParseError::new(lineno, "missing y coordinate"))?;
    let z = col_f64(line, 47, 54)
        .ok_or_else(|| ParseError::new(lineno, "missing z coordinate"))?;
    let b_factor = col_f64(line, 61, 66);

    // Element column (77-78) is preferred; fall back to first alphabetic chars of name.
    let element = if let Some(sym) = col_str(line, 77, 78) {
        Element::from_symbol(&sym)
    } else {
        let sym: String = name.chars().filter(|c| c.is_alphabetic()).collect();
        Element::from_symbol(&sym)
    };

    Ok(AtomRecord { serial, name, res_name, chain_id, res_seq, x, y, z, b_factor, element })
}

// ============================================================================
// CONECT record
// ============================================================================

/// Returns `(source_serial, bonded_serials)` from a CONECT line.
///
/// CONECT format: atom(7-11), bond1(12-16), bond2(17-21), bond3(22-26), bond4(27-31).
fn parse_conect_line(line: &str) -> Option<(u32, Vec<u32>)> {
    let src = col_u32(line, 7, 11)?;
    let mut bonded = Vec::with_capacity(4);
    for (start, end) in [(12, 16), (17, 21), (22, 26), (27, 31)] {
        if let Some(s) = col_u32(line, start, end) {
            bonded.push(s);
        }
    }
    if bonded.is_empty() { None } else { Some((src, bonded)) }
}

// ============================================================================
// HELIX record
// ============================================================================

/// Parse one HELIX line.
///
/// PDB HELIX columns (v3.30):
/// - `initChainID`: col 20
/// - `initSeqNum`:  col 22-25
/// - `endChainID`:  col 32
/// - `endSeqNum`:   col 34-37
/// - `helixClass`:  col 39-40  (1=right-alpha, 3=pi, 5=3-10)
fn parse_helix_line(line: &str) -> Option<SecStrRange> {
    let init_chain = col_char(line, 20)?;
    let init_seq = col_i32(line, 22, 25)?;
    let end_seq = col_i32(line, 34, 37)?;

    let helix_class: i32 = col_i32(line, 39, 40).unwrap_or(1);
    let kind = match helix_class {
        5 => SecondaryStructure::Helix310,
        3 => SecondaryStructure::HelixPi,
        _ => SecondaryStructure::Helix,
    };

    Some(SecStrRange { chain_id: init_chain, init_seq, end_seq, kind })
}

// ============================================================================
// SHEET record
// ============================================================================

/// Parse one SHEET line.
///
/// PDB SHEET columns (v3.30):
/// - `initChainID`: col 22
/// - `initSeqNum`:  col 23-26
/// - `endChainID`:  col 33
/// - `endSeqNum`:   col 34-37
fn parse_sheet_line(line: &str) -> Option<SecStrRange> {
    let init_chain = col_char(line, 22)?;
    let init_seq = col_i32(line, 23, 26)?;
    let end_seq = col_i32(line, 34, 37)?;

    Some(SecStrRange {
        chain_id: init_chain,
        init_seq,
        end_seq,
        kind: SecondaryStructure::Sheet,
    })
}

// ============================================================================
// Distance-based bond detection
// ============================================================================

/// Add bonds between all atom pairs that fall within the covalent-radius cutoff.
///
/// The cutoff is `(r1 + r2) × 1.2` following the Hooft et al. criterion.
/// H-H pairs are skipped (not biochemically relevant at this scale).
///
/// This is O(n²) — acceptable for typical PDB structures (< 100 000 atoms),
/// but callers should prefer CONECT records when available.
fn infer_bonds_by_distance(atoms: &[Atom]) -> Vec<Bond> {
    let mut bonds = Vec::new();
    let n = atoms.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let a1 = &atoms[i];
            let a2 = &atoms[j];
            if a1.element == Element::H && a2.element == Element::H {
                continue;
            }
            let cutoff =
                (a1.element.covalent_radius() + a2.element.covalent_radius()) * 1.2;
            if a1.distance_to(a2) < cutoff {
                bonds.push(Bond { atom1: i, atom2: j, order: BondOrder::Single });
            }
        }
    }
    bonds
}

// ============================================================================
// Main parser
// ============================================================================

/// Parse a PDB-format string into a [`Molecule`].
///
/// # Record handling
///
/// - `ATOM` and `HETATM` records populate `molecule.atoms` and are grouped
///   into the chain/residue hierarchy in `molecule.chains`.
/// - `CONECT` records add explicit bonds.  If none are present, distance-based
///   bond detection is used as a fallback.
/// - `HELIX` and `SHEET` records set [`SecondaryStructure`] on each matching
///   residue.
/// - `END` and `ENDMDL` terminate parsing; subsequent lines are ignored
///   (first-model-only semantics for NMR ensembles).
///
/// # Errors
///
/// Returns [`ParseError`] if a mandatory field cannot be parsed on an
/// `ATOM`/`HETATM` line.  Missing or malformed `CONECT`/`HELIX`/`SHEET`
/// fields are silently skipped (consistent with real-world PDB quirks).
pub fn parse_pdb(input: &str) -> Result<Molecule, ParseError> {
    let mut atom_records: Vec<AtomRecord> = Vec::new();
    // Maps source_serial → list of bonded serials (from CONECT lines).
    let mut conect: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut sec_str_ranges: Vec<SecStrRange> = Vec::new();

    for (lineno_0, raw_line) in input.lines().enumerate() {
        let lineno = lineno_0 + 1;
        // PDB record type is the first 6 characters (left-justified, space-padded).
        let record = raw_line.get(..6).unwrap_or(raw_line).trim_end();

        match record {
            "ATOM" | "HETATM" => {
                let rec = parse_atom_line(raw_line, lineno)?;
                atom_records.push(rec);
            }
            "CONECT" => {
                if let Some((src, bonded)) = parse_conect_line(raw_line) {
                    conect.entry(src).or_default().extend(bonded);
                }
            }
            "HELIX" => {
                if let Some(range) = parse_helix_line(raw_line) {
                    sec_str_ranges.push(range);
                }
            }
            "SHEET" => {
                if let Some(range) = parse_sheet_line(raw_line) {
                    sec_str_ranges.push(range);
                }
            }
            "END" | "ENDMDL" => break,
            _ => {} // Ignore all other record types
        }
    }

    // -------------------------------------------------------------------------
    // Build Molecule.atoms and serial→index map
    // -------------------------------------------------------------------------
    let mut mol = Molecule::new("PDB structure");
    mol.source_format = Some("PDB".to_string());

    // serial → 0-indexed position in mol.atoms
    let mut serial_to_idx: HashMap<u32, usize> = HashMap::with_capacity(atom_records.len());

    for rec in &atom_records {
        let idx = mol.atoms.len();
        serial_to_idx.insert(rec.serial, idx);

        mol.atoms.push(Atom {
            id: rec.serial,
            element: rec.element,
            position: [rec.x, rec.y, rec.z],
            charge: 0,
            name: rec.name.clone(),
            residue_name: Some(rec.res_name.clone()),
            residue_seq: Some(rec.res_seq),
            chain_id: Some(rec.chain_id),
            b_factor: rec.b_factor,
        });
    }

    // -------------------------------------------------------------------------
    // Build Chain / Residue hierarchy
    // -------------------------------------------------------------------------
    // We track insertion order explicitly so chains and residues appear in the
    // same sequence as the file.
    let mut chain_order: Vec<char> = Vec::new();
    let mut chain_res_order: HashMap<char, Vec<i32>> = HashMap::new();
    // (chain_id, res_seq) → atom indices
    let mut residue_atoms: HashMap<(char, i32), Vec<usize>> = HashMap::new();
    // (chain_id, res_seq) → residue name
    let mut residue_names: HashMap<(char, i32), String> = HashMap::new();

    for (idx, rec) in atom_records.iter().enumerate() {
        let key = (rec.chain_id, rec.res_seq);
        if !residue_atoms.contains_key(&key) {
            if !chain_order.contains(&rec.chain_id) {
                chain_order.push(rec.chain_id);
                chain_res_order.insert(rec.chain_id, Vec::new());
            }
            chain_res_order
                .entry(rec.chain_id)
                .or_default()
                .push(rec.res_seq);
            residue_names.insert(key, rec.res_name.clone());
        }
        residue_atoms.entry(key).or_default().push(idx);
    }

    for chain_id in &chain_order {
        let res_seqs = chain_res_order.get(chain_id).cloned().unwrap_or_default();
        let mut residues: Vec<Residue> = Vec::with_capacity(res_seqs.len());

        for res_seq in &res_seqs {
            let key = (*chain_id, *res_seq);
            let atom_indices = residue_atoms.get(&key).cloned().unwrap_or_default();
            let name = residue_names.get(&key).cloned().unwrap_or_default();

            residues.push(Residue {
                name,
                seq: *res_seq,
                insertion_code: None,
                atom_indices,
                secondary_structure: SecondaryStructure::Coil,
            });
        }

        mol.chains.push(Chain { id: *chain_id, residues });
    }

    // -------------------------------------------------------------------------
    // Apply secondary structure annotations
    // -------------------------------------------------------------------------
    for range in &sec_str_ranges {
        if let Some(chain) = mol.chains.iter_mut().find(|c| c.id == range.chain_id) {
            for residue in &mut chain.residues {
                if residue.seq >= range.init_seq && residue.seq <= range.end_seq {
                    residue.secondary_structure = range.kind;
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Bonds: CONECT records take priority; fall back to distance detection
    // -------------------------------------------------------------------------
    if conect.is_empty() {
        mol.bonds = infer_bonds_by_distance(&mol.atoms);
    } else {
        let mut seen: HashMap<(usize, usize), bool> = HashMap::new();
        for (&src_serial, bonded_serials) in &conect {
            let Some(&src_idx) = serial_to_idx.get(&src_serial) else {
                continue;
            };
            for &bonded_serial in bonded_serials {
                let Some(&dst_idx) = serial_to_idx.get(&bonded_serial) else {
                    continue;
                };
                // Deduplicate: CONECT records appear for both directions.
                let key = (src_idx.min(dst_idx), src_idx.max(dst_idx));
                if seen.insert(key, true).is_none() {
                    mol.bonds.push(Bond {
                        atom1: key.0,
                        atom2: key.1,
                        order: BondOrder::Single,
                    });
                }
            }
        }
    }

    Ok(mol)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse or return an empty placeholder molecule, printing the error.
    fn parse_or_empty(src: &str) -> Molecule {
        parse_pdb(src).unwrap_or_else(|e| {
            eprintln!("parse_pdb error in test: {e}");
            Molecule::new("parse-error")
        })
    }

    /// Crambin (1CRN) excerpt — five atoms from the first THR residue, a HELIX
    /// record spanning residues 1-4, and an END terminator.
    const CRAMBIN_EXCERPT: &str = "\
ATOM      1  N   THR A   1      17.047  14.099   3.625  1.00 13.79           N  \n\
ATOM      2  CA  THR A   1      16.967  12.784   4.338  1.00 10.80           C  \n\
ATOM      3  C   THR A   1      15.685  12.755   5.133  1.00  9.19           C  \n\
ATOM      4  O   THR A   1      15.268  13.825   5.594  1.00  9.85           O  \n\
ATOM      5  CB  THR A   1      18.170  12.703   5.337  1.00 13.02           C  \n\
HELIX    1   1 THR A    1  ASN A    4  1                                   4\n\
END\n";

    #[test]
    fn atom_count() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        assert_eq!(mol.atoms.len(), 5, "expected 5 atoms");
    }

    #[test]
    fn chain_hierarchy() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        assert_eq!(mol.chains.len(), 1);
        assert_eq!(mol.chains[0].id, 'A');
        assert_eq!(mol.chains[0].residues.len(), 1);
        assert_eq!(mol.chains[0].residues[0].name, "THR");
        assert_eq!(mol.chains[0].residues[0].seq, 1);
    }

    #[test]
    fn helix_secondary_structure_applied() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        let ss = mol.chains[0].residues[0].secondary_structure;
        assert_eq!(ss, SecondaryStructure::Helix);
    }

    #[test]
    fn atom_positions_parsed() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        let n_atom = &mol.atoms[0];
        assert!((n_atom.position[0] - 17.047).abs() < 1e-3);
        assert!((n_atom.position[1] - 14.099).abs() < 1e-3);
        assert!((n_atom.position[2] - 3.625).abs() < 1e-3);
    }

    #[test]
    fn atom_elements_inferred() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        assert_eq!(mol.atoms[0].element, Element::N); // N atom
        assert_eq!(mol.atoms[1].element, Element::C); // CA atom
        assert_eq!(mol.atoms[3].element, Element::O); // O atom
    }

    #[test]
    fn atom_names_preserved() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        assert_eq!(mol.atoms[0].name, "N");
        assert_eq!(mol.atoms[1].name, "CA");
        assert_eq!(mol.atoms[4].name, "CB");
    }

    #[test]
    fn b_factors_parsed() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        let bf = mol.atoms[0].b_factor;
        assert!(bf.is_some());
        assert!((bf.unwrap_or(0.0) - 13.79).abs() < 1e-2);
    }

    #[test]
    fn atom_serials_match() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        for (i, atom) in mol.atoms.iter().enumerate() {
            assert_eq!(atom.id, (i + 1) as u32);
        }
    }

    #[test]
    fn distance_bonds_inferred_when_no_conect() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        // No CONECT records → distance inference.
        // The five atoms of THR form at least the N-CA and CA-C backbone bonds.
        assert!(!mol.bonds.is_empty(), "expected at least one inferred bond");
    }

    #[test]
    fn conect_bonds_override_distance() {
        let pdb = "\
ATOM      1  N   ALA A   1       0.000   0.000   0.000  1.00  0.00           N  \n\
ATOM      2  CA  ALA A   1       1.460   0.000   0.000  1.00  0.00           C  \n\
CONECT    1    2\n\
CONECT    2    1\n\
END\n";
        let mol = parse_or_empty(pdb);
        // Both directions deduplicate to one bond.
        assert_eq!(mol.bonds.len(), 1);
        assert_eq!(mol.bonds[0].order, BondOrder::Single);
    }

    #[test]
    fn end_terminates_parsing() {
        let pdb = "\
ATOM      1  N   ALA A   1       0.000   0.000   0.000  1.00  0.00           N  \n\
END\n\
ATOM      2  CA  ALA A   1       1.460   0.000   0.000  1.00  0.00           C  \n";
        let mol = parse_or_empty(pdb);
        assert_eq!(mol.atoms.len(), 1, "END should stop parsing");
    }

    #[test]
    fn endmdl_terminates_parsing() {
        let pdb = "\
ATOM      1  N   ALA A   1       0.000   0.000   0.000  1.00  0.00           N  \n\
ENDMDL\n\
ATOM      2  CA  ALA A   1       1.460   0.000   0.000  1.00  0.00           C  \n";
        let mol = parse_or_empty(pdb);
        assert_eq!(mol.atoms.len(), 1, "ENDMDL should stop parsing");
    }

    #[test]
    fn multiple_chains_parsed() {
        let pdb = "\
ATOM      1  N   ALA A   1       0.000   0.000   0.000  1.00  0.00           N  \n\
ATOM      2  N   GLY B   1       5.000   0.000   0.000  1.00  0.00           N  \n\
END\n";
        let mol = parse_or_empty(pdb);
        assert_eq!(mol.chains.len(), 2);
        assert_eq!(mol.chains[0].id, 'A');
        assert_eq!(mol.chains[1].id, 'B');
    }

    #[test]
    fn multiple_residues_in_chain() {
        let pdb = "\
ATOM      1  N   ALA A   1       0.000   0.000   0.000  1.00  0.00           N  \n\
ATOM      2  N   GLY A   2       5.000   0.000   0.000  1.00  0.00           N  \n\
ATOM      3  N   VAL A   3      10.000   0.000   0.000  1.00  0.00           N  \n\
END\n";
        let mol = parse_or_empty(pdb);
        assert_eq!(mol.chains.len(), 1);
        assert_eq!(mol.chains[0].residues.len(), 3);
    }

    #[test]
    fn sheet_secondary_structure() {
        let pdb = "\
ATOM      1  N   ALA A   2       0.000   0.000   0.000  1.00  0.00           N  \n\
ATOM      2  N   GLY A   3       4.000   0.000   0.000  1.00  0.00           N  \n\
SHEET    1   A 2 ALA A   2  GLY A   3  0\n\
END\n";
        let mol = parse_or_empty(pdb);
        for residue in &mol.chains[0].residues {
            assert_eq!(residue.secondary_structure, SecondaryStructure::Sheet);
        }
    }

    #[test]
    fn hetatm_parsed_identically_to_atom() {
        let pdb = "\
HETATM    1  C1  LIG A 100       1.000   2.000   3.000  1.00  5.00           C  \n\
END\n";
        let mol = parse_or_empty(pdb);
        assert_eq!(mol.atoms.len(), 1);
        assert_eq!(mol.atoms[0].element, Element::C);
        assert_eq!(mol.atoms[0].name, "C1");
    }

    #[test]
    fn source_format_is_pdb() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        assert_eq!(mol.source_format.as_deref(), Some("PDB"));
    }

    #[test]
    fn chain_ids_on_atoms() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        for atom in &mol.atoms {
            assert_eq!(atom.chain_id, Some('A'));
        }
    }

    #[test]
    fn residue_indices_reference_correct_atoms() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        let residue = &mol.chains[0].residues[0];
        assert_eq!(residue.atom_indices.len(), 5);
        for &idx in &residue.atom_indices {
            assert_eq!(mol.atoms[idx].residue_seq, Some(1));
        }
    }

    #[test]
    fn parse_error_on_missing_serial() {
        // Deliberately malformed: serial field (cols 7-11) is blank.
        let pdb = "ATOM         N   THR A   1      17.047  14.099   3.625  1.00 13.79           N  \n";
        let result = parse_pdb(pdb);
        assert!(result.is_err(), "expected ParseError for blank serial");
        if let Err(e) = result {
            assert_eq!(e.line, 1);
            assert!(e.message.contains("serial"), "error message should mention serial");
        }
    }

    #[test]
    fn parse_error_display() {
        let e = ParseError::new(42, "bad field");
        let s = format!("{e}");
        assert!(s.contains("42"));
        assert!(s.contains("bad field"));
    }

    #[test]
    fn empty_input_produces_empty_molecule() {
        let mol = parse_or_empty("END\n");
        assert!(mol.atoms.is_empty());
        assert!(mol.chains.is_empty());
        assert!(mol.bonds.is_empty());
    }

    #[test]
    fn residue_names_three_letter_code() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        // resName cols 18-20 gives "THR"
        assert_eq!(mol.chains[0].residues[0].name, "THR");
    }

    #[test]
    fn is_protein_true_for_pdb() {
        let mol = parse_or_empty(CRAMBIN_EXCERPT);
        assert!(mol.is_protein());
    }
}
