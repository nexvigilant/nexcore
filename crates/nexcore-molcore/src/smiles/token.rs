// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! SMILES token types.
//!
//! ## Tier: T2-P (σ + ∃)
//! Tokens form a sequence (σ) where each asserts existence (∃) of a molecular feature.

use serde::{Deserialize, Serialize};

/// A token from SMILES lexing.
///
/// ## Tier: T2-P (σ + ∃)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SmilesToken {
    /// Organic subset atom: B, C, N, O, P, S, F, Cl, Br, I
    /// Lowercase = aromatic: b, c, n, o, p, s
    OrganicAtom {
        symbol: String,
        aromatic: bool,
    },
    /// Bracket atom: [Fe], [NH4+], [13C], [2H]
    BracketAtom {
        isotope: Option<u16>,
        symbol: String,
        aromatic: bool,
        hcount: Option<u8>,
        charge: i8,
        class: Option<u16>,
    },
    /// Bond: -, =, #, :, / , backslash
    Bond(BondToken),
    /// Ring closure digit: 1-99 (% prefix for 2-digit)
    RingClosure(u8),
    /// Branch open: (
    BranchOpen,
    /// Branch close: )
    BranchClose,
    /// Dot (disconnected fragments): .
    Dot,
    /// Wildcard: *
    Wildcard,
}

/// Bond tokens from SMILES.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondToken {
    /// Single bond: -
    Single,
    /// Double bond: =
    Double,
    /// Triple bond: #
    Triple,
    /// Aromatic bond: :
    Aromatic,
    /// Up stereo: /
    Up,
    /// Down stereo: backslash
    Down,
}
