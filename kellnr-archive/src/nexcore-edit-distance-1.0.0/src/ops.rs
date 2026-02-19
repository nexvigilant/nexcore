//! # Edit Operations (T2-P Layer)
//!
//! Defines the three classical edit operations as a trait hierarchy.
//! Each operation is generic over element type `E`, enabling cross-domain use
//! (characters, nucleotides, tokens, etc.).
//!
//! Operations are zero-sized types by default — no runtime dispatch for common cases.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A single edit operation applied to transform source into target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditOp<E: Clone + Eq> {
    /// Insert element at position in target
    Insert {
        /// Position in the target sequence
        pos: usize,
        /// Element inserted
        elem: E,
    },
    /// Delete element at position from source
    Delete {
        /// Position in the source sequence
        pos: usize,
        /// Element deleted
        elem: E,
    },
    /// Replace element at position
    Substitute {
        /// Position in both sequences
        pos: usize,
        /// Original element (source)
        from: E,
        /// Replacement element (target)
        to: E,
    },
    /// Swap two adjacent elements (Damerau extension)
    Transpose {
        /// Position of first element
        pos: usize,
        /// First element
        first: E,
        /// Second element
        second: E,
    },
}

impl<E: Clone + Eq + fmt::Display> fmt::Display for EditOp<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Insert { pos, elem } => write!(f, "ins({elem}@{pos})"),
            Self::Delete { pos, elem } => write!(f, "del({elem}@{pos})"),
            Self::Substitute { pos, from, to } => write!(f, "sub({from}->{to}@{pos})"),
            Self::Transpose { pos, first, second } => {
                write!(f, "trans({first}<->{second}@{pos})")
            }
        }
    }
}

/// Defines which edit operations are permitted for a given metric.
///
/// This is the "ops" parameter in `edit_distance(ops, costs, solver)`.
/// Implement this trait to create custom operation sets.
pub trait OperationSet: Clone + Send + Sync + fmt::Debug {
    /// Whether insertion is allowed
    fn allows_insert(&self) -> bool;

    /// Whether deletion is allowed
    fn allows_delete(&self) -> bool;

    /// Whether substitution is allowed
    fn allows_substitute(&self) -> bool;

    /// Whether transposition is allowed (Damerau extension)
    fn allows_transpose(&self) -> bool;

    /// Human-readable name of this operation set
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Built-in operation sets (zero-sized types for monomorphization)
// ---------------------------------------------------------------------------

/// Standard Levenshtein operations: insert, delete, substitute.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StdOps;

impl OperationSet for StdOps {
    fn allows_insert(&self) -> bool {
        true
    }
    fn allows_delete(&self) -> bool {
        true
    }
    fn allows_substitute(&self) -> bool {
        true
    }
    fn allows_transpose(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "standard (ins/del/sub)"
    }
}

/// Damerau-Levenshtein operations: insert, delete, substitute, transpose.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DamerauOps;

impl OperationSet for DamerauOps {
    fn allows_insert(&self) -> bool {
        true
    }
    fn allows_delete(&self) -> bool {
        true
    }
    fn allows_substitute(&self) -> bool {
        true
    }
    fn allows_transpose(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "damerau (ins/del/sub/trans)"
    }
}

/// LCS (Longest Common Subsequence) operations: insert, delete only.
/// No substitution — forces alignment through indels.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct IndelOps;

impl OperationSet for IndelOps {
    fn allows_insert(&self) -> bool {
        true
    }
    fn allows_delete(&self) -> bool {
        true
    }
    fn allows_substitute(&self) -> bool {
        false
    }
    fn allows_transpose(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "indel (ins/del only)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_ops_allows_ins_del_sub() {
        let ops = StdOps;
        assert!(ops.allows_insert());
        assert!(ops.allows_delete());
        assert!(ops.allows_substitute());
        assert!(!ops.allows_transpose());
    }

    #[test]
    fn damerau_ops_allows_transpose() {
        let ops = DamerauOps;
        assert!(ops.allows_insert());
        assert!(ops.allows_delete());
        assert!(ops.allows_substitute());
        assert!(ops.allows_transpose());
    }

    #[test]
    fn indel_ops_no_substitute() {
        let ops = IndelOps;
        assert!(ops.allows_insert());
        assert!(ops.allows_delete());
        assert!(!ops.allows_substitute());
        assert!(!ops.allows_transpose());
    }

    #[test]
    fn edit_op_display() {
        let op: EditOp<char> = EditOp::Insert { pos: 3, elem: 'x' };
        assert_eq!(format!("{op}"), "ins(x@3)");

        let op: EditOp<char> = EditOp::Substitute {
            pos: 0,
            from: 'a',
            to: 'b',
        };
        assert_eq!(format!("{op}"), "sub(a->b@0)");
    }
}
