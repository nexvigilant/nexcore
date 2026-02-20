// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! SMILES parser — converts token stream into prima_chem::Molecule.

use super::token::{BondToken, SmilesToken};
use crate::error::{MolcoreError, MolcoreResult};
use prima_chem::element::Element;
use prima_chem::types::BondOrder;
use prima_chem::{Atom, Bond, Molecule};
use std::collections::HashMap;

/// Parse a SMILES string into a Molecule.
///
/// This is the primary entry point for SMILES parsing.
pub fn parse(smiles: &str) -> MolcoreResult<Molecule> {
    let tokens = super::lexer::lex(smiles)?;
    parse_tokens(&tokens)
}

/// Parse a token stream into a Molecule.
pub fn parse_tokens(tokens: &[SmilesToken]) -> MolcoreResult<Molecule> {
    let mut mol = Molecule::new();
    let mut prev_atom: Option<usize> = None;
    let mut pending_bond: Option<BondOrder> = None;
    let mut branch_stack: Vec<(usize, Option<BondOrder>)> = Vec::new();
    let mut ring_map: HashMap<u8, (usize, Option<BondOrder>)> = HashMap::new();

    for token in tokens {
        match token {
            SmilesToken::OrganicAtom { symbol, aromatic } => {
                let elem = Element::from_symbol(symbol)
                    .ok_or_else(|| MolcoreError::UnknownElement(symbol.clone()))?;
                let mut atom = Atom::new(elem);
                if *aromatic {
                    atom.aromatic = true;
                }
                let idx = mol.add_atom(atom);

                if let Some(prev) = prev_atom {
                    let order = pending_bond.take().unwrap_or_else(|| {
                        if mol.atoms.get(prev).is_some_and(|a| a.aromatic) && *aromatic {
                            BondOrder::Aromatic
                        } else {
                            BondOrder::Single
                        }
                    });
                    let _ = mol.add_bond(Bond {
                        atom1: prev,
                        atom2: idx,
                        order,
                        bond_type: prima_chem::types::BondType::None,
                    });
                }
                prev_atom = Some(idx);
            }

            SmilesToken::BracketAtom {
                isotope,
                symbol,
                aromatic,
                hcount,
                charge,
                class: _,
            } => {
                let elem = Element::from_symbol(symbol)
                    .ok_or_else(|| MolcoreError::UnknownElement(symbol.clone()))?;
                let mut atom = Atom::new(elem);
                atom.aromatic = *aromatic;
                atom.charge = *charge;
                if let Some(iso) = isotope {
                    atom.mass_number = *iso;
                }
                if let Some(h) = hcount {
                    atom.implicit_h = *h;
                }
                let idx = mol.add_atom(atom);

                if let Some(prev) = prev_atom {
                    let order = pending_bond.take().unwrap_or(BondOrder::Single);
                    let _ = mol.add_bond(Bond {
                        atom1: prev,
                        atom2: idx,
                        order,
                        bond_type: prima_chem::types::BondType::None,
                    });
                }
                prev_atom = Some(idx);
            }

            SmilesToken::Bond(bond_token) => {
                pending_bond = Some(match bond_token {
                    BondToken::Single => BondOrder::Single,
                    BondToken::Double => BondOrder::Double,
                    BondToken::Triple => BondOrder::Triple,
                    BondToken::Aromatic => BondOrder::Aromatic,
                    BondToken::Up | BondToken::Down => BondOrder::Single,
                });
            }

            SmilesToken::RingClosure(digit) => {
                if let Some(prev) = prev_atom {
                    if let Some((ring_atom, ring_bond)) = ring_map.remove(digit) {
                        let order = pending_bond
                            .take()
                            .or(ring_bond)
                            .unwrap_or(BondOrder::Single);
                        let _ = mol.add_bond(Bond {
                            atom1: ring_atom,
                            atom2: prev,
                            order,
                            bond_type: prima_chem::types::BondType::None,
                        });
                    } else {
                        ring_map.insert(*digit, (prev, pending_bond.take()));
                    }
                }
            }

            SmilesToken::BranchOpen => {
                if let Some(prev) = prev_atom {
                    branch_stack.push((prev, pending_bond.take()));
                }
            }

            SmilesToken::BranchClose => {
                if let Some((atom, bond)) = branch_stack.pop() {
                    prev_atom = Some(atom);
                    pending_bond = bond;
                } else {
                    return Err(MolcoreError::UnmatchedParen);
                }
            }

            SmilesToken::Dot => {
                prev_atom = None;
                pending_bond = None;
            }

            SmilesToken::Wildcard => {
                let atom = Atom::carbon();
                let idx = mol.add_atom(atom);
                if let Some(prev) = prev_atom {
                    let order = pending_bond.take().unwrap_or(BondOrder::Single);
                    let _ = mol.add_bond(Bond {
                        atom1: prev,
                        atom2: idx,
                        order,
                        bond_type: prima_chem::types::BondType::None,
                    });
                }
                prev_atom = Some(idx);
            }
        }
    }

    // Check for unclosed rings
    if let Some((&digit, _)) = ring_map.iter().next() {
        return Err(MolcoreError::UnclosedRing(digit));
    }

    fill_implicit_hydrogens(&mut mol);

    Ok(mol)
}

/// Fill implicit hydrogen counts for organic subset atoms.
///
/// In SMILES, organic subset atoms (B, C, N, O, P, S, F, Cl, Br, I)
/// have implicit hydrogens to satisfy their default valence.
fn fill_implicit_hydrogens(mol: &mut Molecule) {
    for idx in 0..mol.atoms.len() {
        let atom = &mol.atoms[idx];
        // Only fill for atoms that don't already have explicit hcount
        if atom.implicit_h > 0 {
            continue;
        }

        let elem = match atom.element() {
            Some(e) => e,
            None => continue,
        };

        if !elem.is_organic() {
            continue;
        }

        let valence = elem.default_valence();
        let bonds = mol.bonds_for_atom(idx);
        let bond_sum: u8 = bonds.iter().map(|b| b.order.valence_contribution()).sum();
        let charge_adj = mol.atoms[idx].charge.unsigned_abs();

        // For aromatic atoms (lowercase SMILES), the pi system effectively
        // consumes one additional valence unit. Without this correction
        // aromatic C would get implicit_h=2 (wrong, should be 1) and
        // aromatic N would get implicit_h=1 (wrong for pyridine, should be 0).
        // For aromatic atoms (lowercase SMILES), the pi system effectively
        // consumes one additional valence unit.  This applies whenever the
        // atom is aromatic and has at least one aromatic bond — even if
        // it also has non-aromatic substituent bonds (e.g. the ring C
        // bonded to -OH in phenol or the ester O in aspirin).
        let aromatic_bonus: u8 = if mol.atoms[idx].aromatic {
            let has_aromatic_bond = bonds.iter().any(|b| b.order == BondOrder::Aromatic);
            if has_aromatic_bond && matches!(mol.atoms[idx].atomic_number, 6 | 7) {
                1
            } else {
                0
            }
        } else {
            0
        };

        let total = bond_sum + charge_adj + aromatic_bonus;
        if total < valence {
            mol.atoms[idx].implicit_h = valence - total;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_methane() {
        let mol = parse("C").unwrap_or_default();
        assert_eq!(mol.atom_count(), 1);
        assert_eq!(mol.atoms[0].implicit_h, 4); // CH4
    }

    #[test]
    fn test_parse_ethane() {
        let mol = parse("CC").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bond_count(), 1);
        assert_eq!(mol.atoms[0].implicit_h, 3);
        assert_eq!(mol.atoms[1].implicit_h, 3);
    }

    #[test]
    fn test_parse_ethanol() {
        let mol = parse("CCO").unwrap_or_default();
        assert_eq!(mol.atom_count(), 3);
        assert_eq!(mol.bond_count(), 2);
    }

    #[test]
    fn test_parse_double_bond() {
        let mol = parse("C=O").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bonds[0].order, BondOrder::Double);
    }

    #[test]
    fn test_parse_benzene() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        assert_eq!(mol.atom_count(), 6);
        assert_eq!(mol.bond_count(), 6);
    }

    #[test]
    fn test_parse_branch() {
        let mol = parse("CC(O)C").unwrap_or_default();
        assert_eq!(mol.atom_count(), 4);
        assert_eq!(mol.bond_count(), 3);
        let center_bonds = mol.bonds_for_atom(1);
        assert_eq!(center_bonds.len(), 3);
    }

    #[test]
    fn test_parse_bracket_atom() {
        let mol = parse("[Fe]").unwrap_or_default();
        assert_eq!(mol.atom_count(), 1);
        assert_eq!(mol.atoms[0].atomic_number, 26);
    }

    #[test]
    fn test_parse_charged() {
        let mol = parse("[NH4+]").unwrap_or_default();
        assert_eq!(mol.atom_count(), 1);
        assert_eq!(mol.atoms[0].charge, 1);
        assert_eq!(mol.atoms[0].implicit_h, 4);
    }

    #[test]
    fn test_parse_nacl_disconnected() {
        let mol = parse("[Na+].[Cl-]").unwrap_or_default();
        assert_eq!(mol.atom_count(), 2);
        assert_eq!(mol.bond_count(), 0);
    }

    #[test]
    fn test_parse_aspirin() {
        let mol = parse("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default();
        assert!(mol.atom_count() > 10);
        assert!(mol.bond_count() > 10);
    }

    #[test]
    fn test_parse_caffeine() {
        let result = parse("Cn1cnc2c1c(=O)n(c(=O)n2C)C");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_molecular_weight_ethanol() {
        let mol = parse("CCO").unwrap_or_default();
        let mw = mol.molecular_weight();
        assert!((mw - 46.07).abs() < 0.1, "Expected ~46.07, got {mw}");
    }

    #[test]
    fn test_unclosed_ring_error() {
        let result = parse("C1CC");
        assert!(result.is_err());
    }

    #[test]
    fn test_unmatched_paren_error() {
        let result = parse("CC)");
        assert!(result.is_err());
    }
}
