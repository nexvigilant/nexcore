// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! SMILES lexer — converts SMILES string to token stream.

use crate::error::{MolcoreError, MolcoreResult};
use super::token::{BondToken, SmilesToken};

/// Lex a SMILES string into tokens.
///
/// ## Examples
///
/// - `"C"` → `[OrganicAtom { symbol: "C", aromatic: false }]`
/// - `"CC"` → `[OrganicAtom("C"), OrganicAtom("C")]`
/// - `"C=O"` → `[OrganicAtom("C"), Bond(Double), OrganicAtom("O")]`
/// - `"c1ccccc1"` → aromatic ring with ring closures
pub fn lex(smiles: &str) -> MolcoreResult<Vec<SmilesToken>> {
    let chars: Vec<char> = smiles.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        let ch = chars[pos];
        match ch {
            // Organic subset atoms (uppercase)
            'B' => {
                if pos + 1 < chars.len() && chars[pos + 1] == 'r' {
                    tokens.push(SmilesToken::OrganicAtom { symbol: "Br".into(), aromatic: false });
                    pos += 2;
                } else {
                    tokens.push(SmilesToken::OrganicAtom { symbol: "B".into(), aromatic: false });
                    pos += 1;
                }
            }
            'C' => {
                if pos + 1 < chars.len() && chars[pos + 1] == 'l' {
                    tokens.push(SmilesToken::OrganicAtom { symbol: "Cl".into(), aromatic: false });
                    pos += 2;
                } else {
                    tokens.push(SmilesToken::OrganicAtom { symbol: "C".into(), aromatic: false });
                    pos += 1;
                }
            }
            'N' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "N".into(), aromatic: false });
                pos += 1;
            }
            'O' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "O".into(), aromatic: false });
                pos += 1;
            }
            'P' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "P".into(), aromatic: false });
                pos += 1;
            }
            'S' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "S".into(), aromatic: false });
                pos += 1;
            }
            'F' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "F".into(), aromatic: false });
                pos += 1;
            }
            'I' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "I".into(), aromatic: false });
                pos += 1;
            }
            // Aromatic organic subset (lowercase)
            'b' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "B".into(), aromatic: true });
                pos += 1;
            }
            'c' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "C".into(), aromatic: true });
                pos += 1;
            }
            'n' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "N".into(), aromatic: true });
                pos += 1;
            }
            'o' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "O".into(), aromatic: true });
                pos += 1;
            }
            'p' => {
                tokens.push(SmilesToken::OrganicAtom { symbol: "P".into(), aromatic: true });
                pos += 1;
            }
            's' => {
                if pos + 1 < chars.len() && chars[pos + 1] == 'e' {
                    // aromatic Se
                    tokens.push(SmilesToken::OrganicAtom { symbol: "Se".into(), aromatic: true });
                    pos += 2;
                } else {
                    tokens.push(SmilesToken::OrganicAtom { symbol: "S".into(), aromatic: true });
                    pos += 1;
                }
            }
            // Bracket atom
            '[' => {
                let (token, new_pos) = lex_bracket_atom(&chars, pos)?;
                tokens.push(token);
                pos = new_pos;
            }
            // Bonds
            '-' => { tokens.push(SmilesToken::Bond(BondToken::Single)); pos += 1; }
            '=' => { tokens.push(SmilesToken::Bond(BondToken::Double)); pos += 1; }
            '#' => { tokens.push(SmilesToken::Bond(BondToken::Triple)); pos += 1; }
            ':' => { tokens.push(SmilesToken::Bond(BondToken::Aromatic)); pos += 1; }
            '/' => { tokens.push(SmilesToken::Bond(BondToken::Up)); pos += 1; }
            '\\' => { tokens.push(SmilesToken::Bond(BondToken::Down)); pos += 1; }
            // Ring closures
            '%' => {
                // Two-digit ring closure: %12
                if pos + 2 < chars.len() && chars[pos + 1].is_ascii_digit() && chars[pos + 2].is_ascii_digit() {
                    let tens = chars[pos + 1].to_digit(10).unwrap_or(0) as u8;
                    let ones = chars[pos + 2].to_digit(10).unwrap_or(0) as u8;
                    tokens.push(SmilesToken::RingClosure(tens * 10 + ones));
                    pos += 3;
                } else {
                    return Err(MolcoreError::InvalidSmiles {
                        position: pos,
                        message: "Expected two digits after %".into(),
                    });
                }
            }
            '0'..='9' => {
                let digit = ch.to_digit(10).unwrap_or(0) as u8;
                tokens.push(SmilesToken::RingClosure(digit));
                pos += 1;
            }
            // Branching
            '(' => { tokens.push(SmilesToken::BranchOpen); pos += 1; }
            ')' => { tokens.push(SmilesToken::BranchClose); pos += 1; }
            // Dot (disconnected)
            '.' => { tokens.push(SmilesToken::Dot); pos += 1; }
            // Wildcard
            '*' => { tokens.push(SmilesToken::Wildcard); pos += 1; }
            // Whitespace — skip
            ' ' | '\t' | '\n' | '\r' => { pos += 1; }
            _ => {
                return Err(MolcoreError::InvalidSmiles {
                    position: pos,
                    message: format!("Unexpected character: '{ch}'"),
                });
            }
        }
    }

    Ok(tokens)
}

/// Parse a bracket atom: [Fe], [NH4+], [13C@@H], [2H]
fn lex_bracket_atom(chars: &[char], start: usize) -> MolcoreResult<(SmilesToken, usize)> {
    let mut pos = start + 1; // skip '['

    // Optional isotope (digits before element)
    let mut isotope: Option<u16> = None;
    let iso_start = pos;
    while pos < chars.len() && chars[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos > iso_start {
        let iso_str: String = chars[iso_start..pos].iter().collect();
        isotope = iso_str.parse::<u16>().ok();
    }

    if pos >= chars.len() {
        return Err(MolcoreError::UnexpectedEnd);
    }

    // Element symbol (1-2 chars, first uppercase or lowercase for aromatic)
    let aromatic = chars[pos].is_ascii_lowercase();
    let mut symbol = String::new();

    if chars[pos].is_ascii_alphabetic() {
        symbol.push(chars[pos].to_ascii_uppercase());
        pos += 1;
        // Second char lowercase = two-letter element
        if pos < chars.len() && chars[pos].is_ascii_lowercase() {
            symbol.push(chars[pos]);
            pos += 1;
        }
    } else if chars[pos] == '*' {
        symbol.push('*');
        pos += 1;
    } else {
        return Err(MolcoreError::InvalidSmiles {
            position: pos,
            message: format!("Expected element symbol, got '{}'", chars[pos]),
        });
    }

    // Skip chirality (@, @@)
    while pos < chars.len() && chars[pos] == '@' {
        pos += 1;
    }

    // Hydrogen count: H, H2, H3...
    let mut hcount: Option<u8> = None;
    if pos < chars.len() && (chars[pos] == 'H' || chars[pos] == 'h') {
        pos += 1;
        if pos < chars.len() && chars[pos].is_ascii_digit() {
            hcount = Some(chars[pos].to_digit(10).unwrap_or(1) as u8);
            pos += 1;
        } else {
            hcount = Some(1);
        }
    }

    // Charge: +, ++, +2, -, --, -3
    let mut charge: i8 = 0;
    if pos < chars.len() && chars[pos] == '+' {
        charge = 1;
        pos += 1;
        if pos < chars.len() && chars[pos].is_ascii_digit() {
            charge = chars[pos].to_digit(10).unwrap_or(1) as i8;
            pos += 1;
        } else {
            while pos < chars.len() && chars[pos] == '+' {
                charge += 1;
                pos += 1;
            }
        }
    } else if pos < chars.len() && chars[pos] == '-' {
        charge = -1;
        pos += 1;
        if pos < chars.len() && chars[pos].is_ascii_digit() {
            charge = -(chars[pos].to_digit(10).unwrap_or(1) as i8);
            pos += 1;
        } else {
            while pos < chars.len() && chars[pos] == '-' {
                charge -= 1;
                pos += 1;
            }
        }
    }

    // Atom class: :123
    let mut class: Option<u16> = None;
    if pos < chars.len() && chars[pos] == ':' {
        pos += 1;
        let cls_start = pos;
        while pos < chars.len() && chars[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos > cls_start {
            let cls_str: String = chars[cls_start..pos].iter().collect();
            class = cls_str.parse::<u16>().ok();
        }
    }

    // Closing bracket
    if pos >= chars.len() || chars[pos] != ']' {
        return Err(MolcoreError::InvalidSmiles {
            position: pos,
            message: "Expected closing ']'".into(),
        });
    }
    pos += 1;

    Ok((
        SmilesToken::BracketAtom {
            isotope,
            symbol,
            aromatic,
            hcount,
            charge,
            class,
        },
        pos,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_single_atom() {
        let tokens = lex("C").unwrap_or_default();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], SmilesToken::OrganicAtom { symbol: "C".into(), aromatic: false });
    }

    #[test]
    fn test_lex_ethane() {
        let tokens = lex("CC").unwrap_or_default();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_lex_double_bond() {
        let tokens = lex("C=O").unwrap_or_default();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1], SmilesToken::Bond(BondToken::Double));
    }

    #[test]
    fn test_lex_benzene_aromatic() {
        let tokens = lex("c1ccccc1").unwrap_or_default();
        assert_eq!(tokens.len(), 8); // 6 atoms + 2 ring closures
        assert_eq!(tokens[0], SmilesToken::OrganicAtom { symbol: "C".into(), aromatic: true });
    }

    #[test]
    fn test_lex_branch() {
        let tokens = lex("CC(O)C").unwrap_or_default();
        assert!(tokens.contains(&SmilesToken::BranchOpen));
        assert!(tokens.contains(&SmilesToken::BranchClose));
    }

    #[test]
    fn test_lex_bracket_atom_iron() {
        let tokens = lex("[Fe]").unwrap_or_default();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], SmilesToken::BracketAtom { symbol, .. } if symbol == "Fe"));
    }

    #[test]
    fn test_lex_bracket_atom_charged() {
        let tokens = lex("[NH4+]").unwrap_or_default();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            &tokens[0],
            SmilesToken::BracketAtom { symbol, hcount: Some(4), charge: 1, .. }
            if symbol == "N"
        ));
    }

    #[test]
    fn test_lex_isotope() {
        let tokens = lex("[13C]").unwrap_or_default();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            &tokens[0],
            SmilesToken::BracketAtom { isotope: Some(13), symbol, .. }
            if symbol == "C"
        ));
    }

    #[test]
    fn test_lex_chlorine_vs_carbon() {
        let tokens = lex("Cl").unwrap_or_default();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], SmilesToken::OrganicAtom { symbol: "Cl".into(), aromatic: false });
    }

    #[test]
    fn test_lex_bromine_vs_boron() {
        let tokens = lex("Br").unwrap_or_default();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], SmilesToken::OrganicAtom { symbol: "Br".into(), aromatic: false });
    }

    #[test]
    fn test_lex_aspirin() {
        let result = lex("CC(=O)Oc1ccccc1C(=O)O");
        assert!(result.is_ok());
        let tokens = result.unwrap_or_default();
        assert!(tokens.len() > 10);
    }

    #[test]
    fn test_lex_triple_bond() {
        let tokens = lex("C#N").unwrap_or_default();
        assert_eq!(tokens[1], SmilesToken::Bond(BondToken::Triple));
    }

    #[test]
    fn test_lex_dot_disconnected() {
        let tokens = lex("[Na+].[Cl-]").unwrap_or_default();
        assert!(tokens.contains(&SmilesToken::Dot));
    }

    #[test]
    fn test_lex_invalid() {
        let result = lex("X");
        assert!(result.is_err());
    }
}
