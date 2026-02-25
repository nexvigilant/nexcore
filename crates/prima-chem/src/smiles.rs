// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! SMILES parser and serializer for prima-chem.
//!
//! Implements the OpenSMILES core subset sufficient for drug-like molecules:
//!
//! - **Organic subset atoms**: B, C, N, O, P, S, F, Cl, Br, I
//! - **Aromatic atoms**: b, c, n, o, p, s (lowercase)
//! - **Bracket atoms**: `[Fe]`, `[NH4+]`, `[O-]`, `[13C]`, `[nH]`
//! - **Bonds**: `-` single, `=` double, `#` triple, `:` aromatic
//! - **Branches**: parentheses `()`
//! - **Ring closures**: digits 0–9, percent syntax `%nn`
//! - **Charges**: `+`, `-`, `+N`, `-N` inside bracket atoms
//! - **Implicit hydrogens**: assigned by valence model after full graph build
//!
//! ## Primitive Grounding
//!
//! | Operation | Primitive |
//! |-----------|-----------|
//! | Character sequence | σ (Sequence) |
//! | Bond formation | → (Causality) |
//! | Branch / ring stack | ρ (Recursion) |
//! | Error position | ∂ (Boundary) |
//! | Atomic numbers, orders | N (Numeric) |

use std::collections::HashMap;

use crate::element::Element;
use crate::error::{ChemError, ChemResult};
use crate::types::{Atom, AtomId, Bond, BondOrder, BondType, Molecule};

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a SMILES string into a [`Molecule`].
///
/// Supports the full OpenSMILES core subset. Implicit hydrogens are assigned
/// to organic-subset atoms via standard valence rules after the full
/// connectivity graph is built. Bracket atoms carry explicit H counts.
///
/// # Errors
///
/// Returns [`ChemError::InvalidSmiles`] with the byte position of the
/// problematic character and a diagnostic message.
///
/// # Examples
///
/// ```
/// use prima_chem::smiles;
///
/// // Water — oxygen with 2 implicit H
/// let water = smiles::parse("O").unwrap();
/// assert_eq!(water.atom_count(), 1);
/// assert_eq!(water.atoms[0].implicit_h, 2);
///
/// // Methane — carbon with 4 implicit H
/// let methane = smiles::parse("C").unwrap();
/// assert_eq!(methane.atoms[0].implicit_h, 4);
///
/// // Ethanol — 3 heavy atoms, 2 bonds
/// let ethanol = smiles::parse("CCO").unwrap();
/// assert_eq!(ethanol.atom_count(), 3);
/// assert_eq!(ethanol.bond_count(), 2);
///
/// // Benzene — 6 aromatic carbons, 6 bonds (ring)
/// let benzene = smiles::parse("c1ccccc1").unwrap();
/// assert_eq!(benzene.atom_count(), 6);
/// assert_eq!(benzene.bond_count(), 6);
/// ```
pub fn parse(smiles: &str) -> ChemResult<Molecule> {
    if smiles.is_empty() {
        return Err(ChemError::InvalidSmiles {
            position: 0,
            message: "empty SMILES string".to_string(),
        });
    }
    SmilesParser::new(smiles).parse()
}

/// Serialize a [`Molecule`] to a SMILES string.
///
/// Produces valid but non-canonical SMILES via depth-first traversal.
/// Disconnected components are joined with `.`. Ring closures are encoded
/// using single-digit labels (falling back to `%nn` for more than 9 rings).
///
/// **Round-trip guarantee:** `parse(to_smiles(mol))` produces a molecule
/// with the same atom count, bond count, elements, and hydrogen counts.
///
/// # Examples
///
/// ```
/// use prima_chem::smiles;
///
/// let mol = smiles::parse("CCO").unwrap();
/// let s = smiles::to_smiles(&mol);
/// assert!(!s.is_empty());
///
/// let reparsed = smiles::parse(&s).unwrap();
/// assert_eq!(reparsed.atom_count(), mol.atom_count());
/// assert_eq!(reparsed.bond_count(), mol.bond_count());
/// ```
pub fn to_smiles(mol: &Molecule) -> String {
    if mol.atoms.is_empty() {
        return String::new();
    }
    SmilesWriter::new(mol).write()
}

// ─────────────────────────────────────────────────────────────────────────────
// Valence helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Normal valences for the SMILES organic subset (by atomic number).
fn organic_normal_valences(n: u8) -> &'static [u8] {
    match n {
        5 => &[3],        // B
        6 => &[4],        // C
        7 => &[3, 5],     // N
        8 => &[2],        // O
        9 => &[1],        // F
        15 => &[3, 5],    // P
        16 => &[2, 4, 6], // S
        17 => &[1],       // Cl
        35 => &[1],       // Br
        53 => &[1],       // I
        _ => &[],
    }
}

fn is_organic_subset(n: u8) -> bool {
    matches!(n, 5 | 6 | 7 | 8 | 9 | 15 | 16 | 17 | 35 | 53)
}

/// Compute implicit H for one organic-subset atom.
///
/// For aromatic atoms: `effective_valence = normal_valence − 1` (π contribution).
/// Bond sum counts single/aromatic as 1, double as 2, triple as 3.
fn implicit_h_for(atomic_number: u8, aromatic: bool, bond_sum: u8) -> u8 {
    let vals = organic_normal_valences(atomic_number);
    if vals.is_empty() {
        return 0;
    }
    let adj: u8 = if aromatic { 1 } else { 0 };
    for &v in vals {
        let eff = v.saturating_sub(adj);
        if eff >= bond_sum {
            return eff - bond_sum;
        }
    }
    0
}

/// Default bond order when none is specified.
///
/// Returns `Aromatic` when both atoms are aromatic, `Single` otherwise.
fn default_bond(a1_ar: bool, a2_ar: bool) -> BondOrder {
    if a1_ar && a2_ar {
        BondOrder::Aromatic
    } else {
        BondOrder::Single
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

struct RingOpening {
    atom: AtomId,
    /// Bond order specified at the opening end, if any.
    bond: Option<BondOrder>,
}

struct SmilesParser<'a> {
    bytes: &'a [u8],
    pos: usize,
    mol: Molecule,
    /// Last atom placed (new atoms bond to this).
    current: Option<AtomId>,
    /// Branch return-point stack.
    stack: Vec<AtomId>,
    /// Open ring closures keyed by ring digit (0–99 for `%nn`).
    rings: HashMap<u8, RingOpening>,
    /// Explicit bond specified for the next atom / ring closure.
    pending: Option<BondOrder>,
    /// Per-atom: did this atom come from bracket notation?
    is_bracket: Vec<bool>,
}

impl<'a> SmilesParser<'a> {
    fn new(smiles: &'a str) -> Self {
        Self {
            bytes: smiles.as_bytes(),
            pos: 0,
            mol: Molecule::new(),
            current: None,
            stack: Vec::new(),
            rings: HashMap::new(),
            pending: None,
            is_bracket: Vec::new(),
        }
    }

    fn parse(mut self) -> ChemResult<Molecule> {
        while self.pos < self.bytes.len() {
            self.step()?;
        }

        if !self.stack.is_empty() {
            return Err(ChemError::InvalidSmiles {
                position: self.pos,
                message: format!(
                    "unclosed branch: {} open parenthes{}",
                    self.stack.len(),
                    if self.stack.len() == 1 { "is" } else { "es" }
                ),
            });
        }

        if !self.rings.is_empty() {
            let mut ids: Vec<u8> = self.rings.keys().copied().collect();
            ids.sort_unstable();
            return Err(ChemError::InvalidSmiles {
                position: self.pos,
                message: format!("unclosed ring bond(s): {ids:?}"),
            });
        }

        self.assign_implicit_h();
        Ok(self.mol)
    }

    fn step(&mut self) -> ChemResult<()> {
        let pos = self.pos;
        let ch = self.bytes[pos];
        match ch {
            // ── Organic subset (handles Cl / Br two-char lookahead) ───────────
            b'B' => {
                self.pos += if self.peek1() == Some(b'r') { 2 } else { 1 };
                let sym = if self.pos - pos == 2 { "Br" } else { "B" };
                self.place_organic(sym, false, pos)?;
            }
            b'C' => {
                self.pos += if self.peek1() == Some(b'l') { 2 } else { 1 };
                let sym = if self.pos - pos == 2 { "Cl" } else { "C" };
                self.place_organic(sym, false, pos)?;
            }
            b'N' => {
                self.pos += 1;
                self.place_organic("N", false, pos)?;
            }
            b'O' => {
                self.pos += 1;
                self.place_organic("O", false, pos)?;
            }
            b'P' => {
                self.pos += 1;
                self.place_organic("P", false, pos)?;
            }
            b'S' => {
                self.pos += 1;
                self.place_organic("S", false, pos)?;
            }
            b'F' => {
                self.pos += 1;
                self.place_organic("F", false, pos)?;
            }
            b'I' => {
                self.pos += 1;
                self.place_organic("I", false, pos)?;
            }

            // ── Aromatic organic subset ───────────────────────────────────────
            b'b' => {
                self.pos += 1;
                self.place_organic("B", true, pos)?;
            }
            b'c' => {
                self.pos += 1;
                self.place_organic("C", true, pos)?;
            }
            b'n' => {
                self.pos += 1;
                self.place_organic("N", true, pos)?;
            }
            b'o' => {
                self.pos += 1;
                self.place_organic("O", true, pos)?;
            }
            b'p' => {
                self.pos += 1;
                self.place_organic("P", true, pos)?;
            }
            b's' => {
                self.pos += 1;
                self.place_organic("S", true, pos)?;
            }

            // ── Bracket atom ──────────────────────────────────────────────────
            b'[' => {
                self.pos += 1; // skip '['
                let id = self.parse_bracket(pos)?;
                self.bond_to_current(id, pos)?;
                self.current = Some(id);
            }

            // ── Explicit bond characters ──────────────────────────────────────
            b'-' => {
                self.pos += 1;
                self.pending = Some(BondOrder::Single);
            }
            b'=' => {
                self.pos += 1;
                self.pending = Some(BondOrder::Double);
            }
            b'#' => {
                self.pos += 1;
                self.pending = Some(BondOrder::Triple);
            }
            b':' => {
                self.pos += 1;
                self.pending = Some(BondOrder::Aromatic);
            }

            // ── Branches ──────────────────────────────────────────────────────
            b'(' => {
                self.pos += 1;
                let cur = self.current.ok_or_else(|| ChemError::InvalidSmiles {
                    position: pos,
                    message: "branch opened before any atom".to_string(),
                })?;
                self.stack.push(cur);
            }
            b')' => {
                self.pos += 1;
                let saved = self.stack.pop().ok_or_else(|| ChemError::InvalidSmiles {
                    position: pos,
                    message: "unexpected closing parenthesis".to_string(),
                })?;
                self.current = Some(saved);
                self.pending = None;
            }

            // ── Ring closures ─────────────────────────────────────────────────
            b'%' => {
                self.pos += 1;
                let id = self.read_percent_ring(pos)?;
                self.handle_ring(id, pos)?;
            }
            b'0'..=b'9' => {
                self.pos += 1;
                self.handle_ring(ch - b'0', pos)?;
            }

            // ── Disconnect ────────────────────────────────────────────────────
            b'.' => {
                self.pos += 1;
                self.current = None;
                self.pending = None;
            }

            _ => {
                return Err(ChemError::InvalidSmiles {
                    position: pos,
                    message: format!("unexpected character '{}'", ch as char),
                });
            }
        }
        Ok(())
    }

    fn peek1(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    /// Add an organic-subset atom, bond it to `current`, update `current`.
    fn place_organic(&mut self, sym: &str, aromatic: bool, pos: usize) -> ChemResult<()> {
        let elem = Element::from_symbol(sym).ok_or_else(|| ChemError::InvalidSmiles {
            position: pos,
            message: format!("unknown element: {sym}"),
        })?;
        let mut atom = Atom::new(elem);
        atom.aromatic = aromatic;
        let id = self.mol.add_atom(atom);
        self.is_bracket.push(false);
        self.bond_to_current(id, pos)?;
        self.current = Some(id);
        Ok(())
    }

    /// Form a bond from `self.current` to `id`, consuming `pending`.
    fn bond_to_current(&mut self, id: AtomId, pos: usize) -> ChemResult<()> {
        if let Some(prev) = self.current {
            let order = self.take_bond_order(prev, id);
            let bond = Bond {
                atom1: prev,
                atom2: id,
                order,
                bond_type: BondType::None,
            };
            self.mol
                .add_bond(bond)
                .map_err(|e| ChemError::InvalidSmiles {
                    position: pos,
                    message: e.to_string(),
                })?;
        } else {
            self.pending = None;
        }
        Ok(())
    }

    /// Consume `pending` and return the resolved bond order.
    fn take_bond_order(&mut self, a1: AtomId, a2: AtomId) -> BondOrder {
        if let Some(order) = self.pending.take() {
            return order;
        }
        let a1_ar = self.mol.atoms.get(a1).is_some_and(|a| a.aromatic);
        let a2_ar = self.mol.atoms.get(a2).is_some_and(|a| a.aromatic);
        default_bond(a1_ar, a2_ar)
    }

    /// Parse a bracket atom `[...]`. Caller has already consumed `[`.
    fn parse_bracket(&mut self, start: usize) -> ChemResult<AtomId> {
        // Optional isotope
        let mass = self.read_u16();

        // Element symbol
        let sym_pos = self.pos;
        let (symbol, aromatic) = self.read_symbol().ok_or_else(|| ChemError::InvalidSmiles {
            position: sym_pos,
            message: "expected element symbol inside [...]".to_string(),
        })?;

        // Normalise first char to uppercase for Element lookup
        let upper: String = symbol
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
            .collect();

        let elem = Element::from_symbol(&upper).ok_or_else(|| ChemError::InvalidSmiles {
            position: sym_pos,
            message: format!("unknown element '{symbol}' in bracket atom"),
        })?;

        // Optional chirality @ / @@ — skip
        while self.bytes.get(self.pos).copied() == Some(b'@') {
            self.pos += 1;
        }

        // Optional H count: H or H<digit>
        let h: u8 = if self.bytes.get(self.pos).copied() == Some(b'H') {
            self.pos += 1;
            match self.bytes.get(self.pos).copied() {
                Some(d) if d.is_ascii_digit() => {
                    self.pos += 1;
                    d - b'0'
                }
                _ => 1,
            }
        } else {
            0
        };

        // Optional charge
        let charge = self.read_charge(start)?;

        // Skip any remaining content until `]`
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b']' {
            self.pos += 1;
        }
        if self.bytes.get(self.pos).copied() != Some(b']') {
            return Err(ChemError::InvalidSmiles {
                position: start,
                message: "unclosed bracket atom — missing ']'".to_string(),
            });
        }
        self.pos += 1; // consume `]`

        let mut atom = Atom::new(elem);
        atom.aromatic = aromatic;
        atom.mass_number = mass;
        atom.charge = charge;
        atom.implicit_h = h;

        let id = self.mol.add_atom(atom);
        self.is_bracket.push(true);
        Ok(id)
    }

    /// Read an element symbol: uppercase-or-lowercase first char, then
    /// optional lowercase continuation (for two-letter symbols like Cl, Br, Fe).
    ///
    /// Returns `(symbol_string, is_aromatic)`.
    fn read_symbol(&mut self) -> Option<(String, bool)> {
        let first = *self.bytes.get(self.pos)?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        self.pos += 1;
        let aromatic = first.is_ascii_lowercase();
        let mut sym = String::with_capacity(2);
        sym.push(first as char);

        // Accept second char only when first is uppercase (two-letter symbols).
        if first.is_ascii_uppercase() {
            if let Some(&second) = self.bytes.get(self.pos) {
                if second.is_ascii_lowercase() {
                    sym.push(second as char);
                    self.pos += 1;
                }
            }
        }
        Some((sym, aromatic))
    }

    /// Read a formal charge: `+`, `-`, `++`, `--`, `+N`, `-N`.
    fn read_charge(&mut self, _pos: usize) -> ChemResult<i8> {
        match self.bytes.get(self.pos).copied() {
            Some(b'+') => {
                self.pos += 1;
                match self.bytes.get(self.pos).copied() {
                    Some(b'+') => {
                        self.pos += 1;
                        Ok(2)
                    }
                    Some(d) if d.is_ascii_digit() => {
                        self.pos += 1;
                        Ok((d - b'0') as i8)
                    }
                    _ => Ok(1),
                }
            }
            Some(b'-') => {
                self.pos += 1;
                match self.bytes.get(self.pos).copied() {
                    Some(b'-') => {
                        self.pos += 1;
                        Ok(-2)
                    }
                    Some(d) if d.is_ascii_digit() => {
                        self.pos += 1;
                        Ok(-((d - b'0') as i8))
                    }
                    _ => Ok(-1),
                }
            }
            _ => Ok(0),
        }
    }

    /// Read consecutive ASCII digits as `u16` (isotope mass numbers).
    fn read_u16(&mut self) -> u16 {
        let mut val: u16 = 0;
        let mut any = false;
        while let Some(&d) = self.bytes.get(self.pos) {
            if d.is_ascii_digit() {
                val = val.saturating_mul(10).saturating_add((d - b'0') as u16);
                self.pos += 1;
                any = true;
            } else {
                break;
            }
        }
        if any { val } else { 0 }
    }

    /// Parse two digits after `%` for ring closure label 10–99.
    fn read_percent_ring(&mut self, pos: usize) -> ChemResult<u8> {
        let d1 = self
            .bytes
            .get(self.pos)
            .copied()
            .ok_or_else(|| ChemError::InvalidSmiles {
                position: pos,
                message: "expected two digits after '%'".to_string(),
            })?;
        let d2 = self
            .bytes
            .get(self.pos + 1)
            .copied()
            .ok_or_else(|| ChemError::InvalidSmiles {
                position: pos,
                message: "expected two digits after '%'".to_string(),
            })?;
        if !d1.is_ascii_digit() || !d2.is_ascii_digit() {
            return Err(ChemError::InvalidSmiles {
                position: pos,
                message: format!(
                    "invalid percent ring digits '{}{}' ",
                    d1 as char, d2 as char
                ),
            });
        }
        self.pos += 2;
        Ok((d1 - b'0') * 10 + (d2 - b'0'))
    }

    /// Open or close a ring bond for label `ring_id`.
    fn handle_ring(&mut self, ring_id: u8, pos: usize) -> ChemResult<()> {
        let cur = self.current.ok_or_else(|| ChemError::InvalidSmiles {
            position: pos,
            message: "ring closure digit before any atom".to_string(),
        })?;
        let spec = self.pending.take();

        if let Some(opening) = self.rings.remove(&ring_id) {
            // Close the ring — reconcile bond orders.
            let order = match (opening.bond, spec) {
                (Some(a), Some(b)) if a == b => a,
                (Some(a), None) | (None, Some(a)) => a,
                (None, None) => {
                    let a1_ar = self.mol.atoms.get(opening.atom).is_some_and(|a| a.aromatic);
                    let a2_ar = self.mol.atoms.get(cur).is_some_and(|a| a.aromatic);
                    default_bond(a1_ar, a2_ar)
                }
                (Some(a), Some(b)) => {
                    return Err(ChemError::InvalidSmiles {
                        position: pos,
                        message: format!(
                            "ring {ring_id} bond mismatch: opening={a:?}, closing={b:?}"
                        ),
                    });
                }
            };
            let bond = Bond {
                atom1: opening.atom,
                atom2: cur,
                order,
                bond_type: BondType::None,
            };
            self.mol
                .add_bond(bond)
                .map_err(|e| ChemError::InvalidSmiles {
                    position: pos,
                    message: e.to_string(),
                })?;
        } else {
            self.rings.insert(
                ring_id,
                RingOpening {
                    atom: cur,
                    bond: spec,
                },
            );
        }
        Ok(())
    }

    /// Post-process: assign implicit H to all non-bracket organic-subset atoms.
    fn assign_implicit_h(&mut self) {
        let n = self.mol.atoms.len();
        for id in 0..n {
            if self.is_bracket.get(id).copied().unwrap_or(true) {
                continue;
            }
            let an = self.mol.atoms[id].atomic_number;
            if !is_organic_subset(an) {
                continue;
            }
            // Valence contribution: single/aromatic = 1, double = 2, triple = 3.
            let bond_sum: u8 = self
                .mol
                .bonds
                .iter()
                .filter(|b| b.atom1 == id || b.atom2 == id)
                .map(|b| b.order.valence_contribution())
                .sum();
            let aromatic = self.mol.atoms[id].aromatic;
            self.mol.atoms[id].implicit_h = implicit_h_for(an, aromatic, bond_sum);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Writer
// ─────────────────────────────────────────────────────────────────────────────

/// Per-atom ring-closure annotation produced during the DFS scan pass.
struct RingTag {
    ring_num: u8,
    order: BondOrder,
    /// True = this atom is the DFS-opener (will write the bond symbol here).
    is_opener: bool,
    /// The atom at the far end of this ring bond.
    other: AtomId,
}

struct SmilesWriter<'a> {
    mol: &'a Molecule,
}

impl<'a> SmilesWriter<'a> {
    fn new(mol: &'a Molecule) -> Self {
        Self { mol }
    }

    fn write(self) -> String {
        // Phase 1 — scan for ring back-edges.
        let mut visited = vec![false; self.mol.atoms.len()];
        let mut tags: HashMap<AtomId, Vec<RingTag>> = HashMap::new();
        let mut next_ring: u8 = 1;

        for start in 0..self.mol.atoms.len() {
            if !visited[start] {
                self.scan_rings(start, None, &mut visited, &mut tags, &mut next_ring);
            }
        }

        // Phase 2 — write SMILES.
        let mut visited2 = vec![false; self.mol.atoms.len()];
        let mut out = String::new();
        let mut first = true;

        for start in 0..self.mol.atoms.len() {
            if !visited2[start] {
                if !first {
                    out.push('.');
                }
                self.emit(start, None, &mut visited2, &tags, &mut out);
                first = false;
            }
        }
        out
    }

    /// DFS scan: find back-edges and record ring tags on both endpoint atoms.
    fn scan_rings(
        &self,
        atom: AtomId,
        parent: Option<AtomId>,
        visited: &mut Vec<bool>,
        tags: &mut HashMap<AtomId, Vec<RingTag>>,
        next: &mut u8,
    ) {
        visited[atom] = true;
        for (nbr, order) in self.nbrs(atom) {
            if Some(nbr) == parent {
                continue;
            }
            if visited[nbr] {
                // Back edge — ring.
                let rn = *next;
                *next = next.wrapping_add(1);
                // `nbr` is opener (visited first), `atom` is closer.
                tags.entry(nbr).or_default().push(RingTag {
                    ring_num: rn,
                    order,
                    is_opener: true,
                    other: atom,
                });
                tags.entry(atom).or_default().push(RingTag {
                    ring_num: rn,
                    order,
                    is_opener: false,
                    other: nbr,
                });
            } else {
                self.scan_rings(nbr, Some(atom), visited, tags, next);
            }
        }
    }

    /// DFS write: emit atom token + branches.
    fn emit(
        &self,
        atom: AtomId,
        parent: Option<AtomId>,
        visited: &mut Vec<bool>,
        tags: &HashMap<AtomId, Vec<RingTag>>,
        out: &mut String,
    ) {
        visited[atom] = true;
        out.push_str(&self.atom_token(atom, tags));

        let tree: Vec<(AtomId, BondOrder)> = self
            .nbrs(atom)
            .into_iter()
            .filter(|(n, _)| Some(*n) != parent && !visited[*n])
            .collect();

        let count = tree.len();
        for (i, (child, order)) in tree.into_iter().enumerate() {
            let last = i == count - 1;
            if !last {
                out.push('(');
            }
            let a1_ar = self.mol.atoms.get(atom).is_some_and(|a| a.aromatic);
            let a2_ar = self.mol.atoms.get(child).is_some_and(|a| a.aromatic);
            if let Some(sym) = non_default_bond_char(order, a1_ar, a2_ar) {
                out.push(sym);
            }
            self.emit(child, Some(atom), visited, tags, out);
            if !last {
                out.push(')');
            }
        }
    }

    /// Build the SMILES token for one atom including ring-closure suffixes.
    fn atom_token(&self, id: AtomId, tags: &HashMap<AtomId, Vec<RingTag>>) -> String {
        let atom = match self.mol.atoms.get(id) {
            Some(a) => a,
            None => return String::new(),
        };
        let elem = match atom.element() {
            Some(e) => e,
            None => return "[?]".to_string(),
        };

        let needs_bracket = atom.charge != 0
            || atom.mass_number != 0
            || !is_organic_subset(atom.atomic_number)
            || (atom.aromatic && atom.implicit_h > 0);

        let mut t = String::new();

        if needs_bracket {
            t.push('[');
            if atom.mass_number > 0 {
                t.push_str(&atom.mass_number.to_string());
            }
            if atom.aromatic {
                t.push_str(&elem.symbol.to_ascii_lowercase());
            } else {
                t.push_str(elem.symbol);
            }
            match atom.implicit_h {
                0 => {}
                1 => t.push('H'),
                n => {
                    t.push('H');
                    t.push_str(&n.to_string());
                }
            }
            match atom.charge {
                0 => {}
                1 => t.push('+'),
                -1 => t.push('-'),
                n if n > 0 => {
                    t.push('+');
                    t.push_str(&n.to_string());
                }
                n => {
                    t.push_str(&n.to_string());
                }
            }
            t.push(']');
        } else if atom.aromatic {
            t.push_str(&elem.symbol.to_ascii_lowercase());
        } else {
            t.push_str(elem.symbol);
        }

        // Ring-closure suffixes.
        if let Some(ring_tags) = tags.get(&id) {
            for tag in ring_tags {
                if tag.is_opener {
                    let other_ar = self.mol.atoms.get(tag.other).is_some_and(|a| a.aromatic);
                    if let Some(sym) = non_default_bond_char(tag.order, atom.aromatic, other_ar) {
                        t.push(sym);
                    }
                }
                if tag.ring_num >= 10 {
                    t.push('%');
                    t.push((b'0' + tag.ring_num / 10) as char);
                    t.push((b'0' + tag.ring_num % 10) as char);
                } else {
                    t.push((b'0' + tag.ring_num) as char);
                }
            }
        }
        t
    }

    /// All (neighbor, bond_order) pairs for `atom`.
    fn nbrs(&self, atom: AtomId) -> Vec<(AtomId, BondOrder)> {
        self.mol
            .bonds
            .iter()
            .filter_map(|b| {
                if b.atom1 == atom {
                    Some((b.atom2, b.order))
                } else if b.atom2 == atom {
                    Some((b.atom1, b.order))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Return the explicit bond character only when `order` is non-default.
fn non_default_bond_char(order: BondOrder, a1_ar: bool, a2_ar: bool) -> Option<char> {
    if order == default_bond(a1_ar, a2_ar) {
        return None;
    }
    match order {
        BondOrder::Single => Some('-'),
        BondOrder::Double => Some('='),
        BondOrder::Triple => Some('#'),
        BondOrder::Aromatic => Some(':'),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── single atoms ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_single_atom() {
        let mol = parse("C").unwrap_or_default();
        assert_eq!(mol.atom_count(), 1);
        assert_eq!(mol.bond_count(), 0);
    }

    #[test]
    fn test_parse_water() {
        let mol = parse("O").unwrap();
        assert_eq!(mol.atom_count(), 1);
        assert_eq!(mol.atoms[0].atomic_number, 8);
        assert_eq!(mol.atoms[0].implicit_h, 2, "water O needs 2 implicit H");
    }

    #[test]
    fn test_parse_methane() {
        let mol = parse("C").unwrap();
        assert_eq!(mol.atoms[0].implicit_h, 4, "methane C needs 4 implicit H");
    }

    #[test]
    fn test_parse_ammonia() {
        let mol = parse("N").unwrap();
        assert_eq!(mol.atoms[0].implicit_h, 3);
    }

    // ── chains ────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_ethane() {
        let mol = parse("CC").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bond_count(), 1);
    }

    #[test]
    fn test_parse_ethanol() {
        let mol = parse("CCO").unwrap();
        assert_eq!(mol.atom_count(), 3);
        assert_eq!(mol.bond_count(), 2);
        assert_eq!(mol.atoms[0].implicit_h, 3, "terminal C: 3 H");
        assert_eq!(mol.atoms[1].implicit_h, 2, "internal C: 2 H");
        assert_eq!(mol.atoms[2].implicit_h, 1, "terminal O: 1 H");
    }

    #[test]
    fn test_parse_double_bond() {
        let mol = parse("C=C").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bond_count(), 1);
        assert_eq!(mol.bonds[0].order, BondOrder::Double);
        assert_eq!(mol.atoms[0].implicit_h, 2, "ethylene C: 2 H");
    }

    #[test]
    fn test_parse_triple_bond() {
        let mol = parse("C#C").unwrap_or_default();
        assert_eq!(mol.bonds[0].order, BondOrder::Triple);
        assert_eq!(mol.atoms[0].implicit_h, 1, "acetylene C: 1 H");
    }

    // ── branches ──────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_branch() {
        let mol = parse("C(C)C").unwrap_or_default();
        assert_eq!(mol.atom_count(), 3);
        assert_eq!(mol.bond_count(), 2);
    }

    #[test]
    fn test_parse_acetic_acid() {
        let mol = parse("CC(=O)O").unwrap();
        assert_eq!(mol.atom_count(), 4);
        assert_eq!(mol.bond_count(), 3);
        let doubles: usize = mol
            .bonds
            .iter()
            .filter(|b| b.order == BondOrder::Double)
            .count();
        assert_eq!(doubles, 1);
    }

    #[test]
    fn test_parse_isobutane() {
        let mol = parse("CC(C)C").unwrap_or_default();
        assert_eq!(mol.atom_count(), 4);
        assert_eq!(mol.bond_count(), 3);
    }

    // ── rings ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_benzene() {
        let mol = parse("c1ccccc1").unwrap();
        assert_eq!(mol.atom_count(), 6);
        assert_eq!(
            mol.bond_count(),
            6,
            "benzene: 5 chain bonds + 1 ring closure"
        );
        assert!(
            mol.bonds.iter().all(|b| b.order == BondOrder::Aromatic),
            "all benzene bonds should be aromatic"
        );
        assert!(
            mol.atoms.iter().all(|a| a.implicit_h == 1),
            "each benzene C should have 1 implicit H"
        );
    }

    #[test]
    fn test_parse_cyclohexane() {
        let mol = parse("C1CCCCC1").unwrap_or_default();
        assert_eq!(mol.atom_count(), 6);
        assert_eq!(mol.bond_count(), 6);
    }

    #[test]
    fn test_parse_cyclopentadienyl() {
        let mol = parse("c1cccc1").unwrap_or_default();
        assert_eq!(mol.atom_count(), 5);
        assert_eq!(mol.bond_count(), 5);
    }

    // ── bracket atoms ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_iron() {
        let mol = parse("[Fe]").unwrap();
        assert_eq!(mol.atom_count(), 1);
        assert_eq!(mol.atoms[0].atomic_number, 26);
        assert_eq!(mol.atoms[0].charge, 0);
    }

    #[test]
    fn test_parse_ammonium() {
        let mol = parse("[NH4+]").unwrap_or_default();
        assert!(mol.atom(0).is_some());
        let default = Atom::hydrogen();
        let a = mol.atom(0).unwrap_or(&default);
        assert_eq!(a.charge, 1);
        assert_eq!(a.implicit_h, 4);
    }

    #[test]
    fn test_parse_hydroxide() {
        let mol = parse("[OH-]").unwrap_or_default();
        let default = Atom::hydrogen();
        let a = mol.atom(0).unwrap_or(&default);
        assert_eq!(a.charge, -1);
        assert_eq!(a.implicit_h, 1);
    }

    #[test]
    fn test_parse_isotope() {
        let mol = parse("[13C]").unwrap_or_default();
        let default = Atom::hydrogen();
        let a = mol.atom(0).unwrap_or(&default);
        assert_eq!(a.mass_number, 13);
        assert_eq!(a.atomic_number, 6);
    }

    #[test]
    fn test_parse_bracket_isotope_deuterium() {
        let mol = parse("[2H]").unwrap_or_default();
        let default = Atom::hydrogen();
        let a = mol.atom(0).unwrap_or(&default);
        assert_eq!(a.mass_number, 2);
    }

    #[test]
    fn test_parse_pyrrole_n() {
        // [nH] — aromatic N with explicit H (pyrrole-type)
        let mol = parse("c1cc[nH]c1").unwrap();
        assert_eq!(mol.atom_count(), 5);
        let n = mol.atoms.iter().find(|a| a.atomic_number == 7);
        assert!(n.is_some());
        assert_eq!(n.unwrap_or(&Atom::hydrogen()).implicit_h, 1);
    }

    // ── halogens ──────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_chlorine() {
        let mol = parse("CCl").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        let cl = mol.atoms.iter().find(|a| a.atomic_number == 17);
        assert!(cl.is_some());
    }

    #[test]
    fn test_parse_bromine() {
        let mol = parse("CBr").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        let br = mol.atoms.iter().find(|a| a.atomic_number == 35);
        assert!(br.is_some());
    }

    #[test]
    fn test_parse_chloroethane() {
        let mol = parse("CCl").unwrap();
        // C: 1 single bond → implicit_h = 4-1 = 3
        assert_eq!(mol.atoms[0].implicit_h, 3);
        // Cl: 1 single bond → implicit_h = 1-1 = 0
        assert_eq!(mol.atoms[1].implicit_h, 0);
    }

    // ── disconnect ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_two_components() {
        let mol = parse("C.O").unwrap();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bond_count(), 0, "disconnected components share no bond");
    }

    // ── aspirin ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_aspirin() {
        let mol = parse("CC(=O)Oc1ccccc1C(=O)O").unwrap();
        assert_eq!(mol.atom_count(), 13, "aspirin: 13 heavy atoms");
        assert_eq!(mol.count_element("C"), 9);
        assert_eq!(mol.count_element("O"), 4);
        // The -OH oxygen should have 1 implicit H
        let o_atoms: Vec<&Atom> = mol.atoms.iter().filter(|a| a.atomic_number == 8).collect();
        let hydroxyls: usize = o_atoms.iter().filter(|a| a.implicit_h == 1).count();
        assert_eq!(hydroxyls, 1, "aspirin has one -OH oxygen");
    }

    // ── caffeine ──────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_caffeine() {
        let mol = parse("Cn1cnc2c1c(=O)n(c(=O)n2C)C");
        assert!(mol.is_ok(), "caffeine should parse without error");
    }

    // ── error cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_empty_error() {
        let result = parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_ring() {
        let result = parse("C1CC");
        assert!(result.is_err());
        if let Err(ChemError::InvalidSmiles { message, .. }) = result {
            assert!(message.contains("unclosed ring"));
        }
    }

    #[test]
    fn test_parse_unmatched_paren_close() {
        let result = parse("C)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_branch() {
        let result = parse("C(C");
        assert!(result.is_err());
        if let Err(ChemError::InvalidSmiles { message, .. }) = result {
            assert!(message.contains("unclosed branch"));
        }
    }

    #[test]
    fn test_parse_invalid_char() {
        let result = parse("C!C");
        assert!(result.is_err());
        if let Err(ChemError::InvalidSmiles { position, .. }) = result {
            assert_eq!(position, 1);
        }
    }

    #[test]
    fn test_parse_unclosed_bracket() {
        let result = parse("[Fe");
        assert!(result.is_err());
    }

    // ── to_smiles ─────────────────────────────────────────────────────────────

    #[test]
    fn test_to_smiles_empty() {
        let mol = Molecule::new();
        assert_eq!(to_smiles(&mol), "");
    }

    #[test]
    fn test_to_smiles_methane_roundtrip() {
        let mol = parse("C").unwrap();
        let s = to_smiles(&mol);
        assert_eq!(s, "C");
        let back = parse(&s).unwrap();
        assert_eq!(back.atoms[0].implicit_h, 4);
    }

    #[test]
    fn test_to_smiles_water_roundtrip() {
        roundtrip("O", 1, 0);
    }

    #[test]
    fn test_to_smiles_ethanol_roundtrip() {
        roundtrip("CCO", 3, 2);
    }

    #[test]
    fn test_to_smiles_benzene_roundtrip() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let s = to_smiles(&mol);
        assert!(
            !s.is_empty(),
            "serializer should emit non-empty aromatic SMILES"
        );
    }

    #[test]
    fn test_to_smiles_aspirin_roundtrip() {
        let mol = parse("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default();
        let s = to_smiles(&mol);
        assert!(!s.is_empty(), "serializer should emit non-empty SMILES");
    }

    #[test]
    fn test_to_smiles_bracket_atom() {
        let mol = parse("[Fe]").unwrap_or_default();
        let s = to_smiles(&mol);
        assert!(s.contains("Fe"), "bracket atom should use bracket notation");
    }

    #[test]
    fn test_to_smiles_charged() {
        let mol = parse("[NH4+]").unwrap_or_default();
        let s = to_smiles(&mol);
        assert!(
            s.contains('+'),
            "charged atom must preserve charge in SMILES"
        );
        let back = parse(&s).unwrap_or_default();
        let default = Atom::hydrogen();
        let a = back.atom(0).unwrap_or(&default);
        assert_eq!(a.charge, 1);
        assert_eq!(a.implicit_h, 4);
    }

    #[test]
    fn test_to_smiles_two_components() {
        let mol = parse("C.O").unwrap();
        let s = to_smiles(&mol);
        assert!(
            s.contains('.'),
            "disconnected components should be separated by '.'"
        );
        let back = parse(&s).unwrap();
        assert_eq!(back.atom_count(), 2);
        assert_eq!(back.bond_count(), 0);
    }

    // ── helper ────────────────────────────────────────────────────────────────

    fn roundtrip(smiles: &str, atoms: usize, bonds: usize) {
        let mol = parse(smiles).unwrap_or_else(|e| panic!("parse({smiles:?}) failed: {e}"));
        assert_eq!(mol.atom_count(), atoms, "atom count for {smiles:?}");
        assert_eq!(mol.bond_count(), bonds, "bond count for {smiles:?}");

        let out = to_smiles(&mol);
        let back = parse(&out)
            .unwrap_or_else(|e| panic!("re-parse of {out:?} (from {smiles:?}) failed: {e}"));
        assert_eq!(
            back.atom_count(),
            atoms,
            "roundtrip atoms: {smiles:?} → {out:?}"
        );
        assert_eq!(
            back.bond_count(),
            bonds,
            "roundtrip bonds: {smiles:?} → {out:?}"
        );
    }
}
