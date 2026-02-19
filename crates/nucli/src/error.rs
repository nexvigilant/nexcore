//! Error types for nucli.
//!
//! Every variant represents a boundary violation (∂).
//! Display tests cover ALL variants from Day 1 (ELL Principle L3).

use core::fmt;

/// All nucli errors.
#[derive(Debug, PartialEq, Eq)]
pub enum NucliError {
    /// Character is not A, T, G, or C.
    InvalidNucleotide(char),

    /// Strand length is not divisible by 4 (each byte = 4 nucleotides).
    IncompleteTetrad(usize),

    /// Input was empty.
    EmptyInput,
}

impl fmt::Display for NucliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNucleotide(ch) => {
                write!(f, "invalid nucleotide: '{ch}' (expected A, T, G, or C)")
            }
            Self::IncompleteTetrad(len) => {
                write!(
                    f,
                    "incomplete tetrad: strand has {len} nucleotides (not divisible by 4)"
                )
            }
            Self::EmptyInput => write!(f, "empty input"),
        }
    }
}

impl std::error::Error for NucliError {}

/// Convenience Result alias.
pub type Result<T> = std::result::Result<T, NucliError>;

// ---------------------------------------------------------------------------
// Tests — L3: Error Display coverage from Day 1
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_nucleotide() {
        let e = NucliError::InvalidNucleotide('X');
        let s = format!("{e}");
        assert!(s.contains("invalid nucleotide"));
        assert!(s.contains("'X'"));
    }

    #[test]
    fn display_incomplete_tetrad() {
        let e = NucliError::IncompleteTetrad(7);
        let s = format!("{e}");
        assert!(s.contains("incomplete tetrad"));
        assert!(s.contains("7"));
        assert!(s.contains("not divisible by 4"));
    }

    #[test]
    fn display_empty_input() {
        let e = NucliError::EmptyInput;
        let s = format!("{e}");
        assert!(s.contains("empty input"));
    }

    #[test]
    fn implements_error_trait() {
        let e = NucliError::EmptyInput;
        let _: &dyn std::error::Error = &e;
    }

    #[test]
    fn debug_format() {
        let e = NucliError::InvalidNucleotide('Z');
        let s = format!("{e:?}");
        assert!(s.contains("InvalidNucleotide"));
    }

    #[test]
    fn equality() {
        assert_eq!(NucliError::EmptyInput, NucliError::EmptyInput);
        assert_ne!(
            NucliError::InvalidNucleotide('A'),
            NucliError::InvalidNucleotide('B')
        );
    }
}
