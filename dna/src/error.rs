//! DNA error types with ∂ Boundary markers.
//!
//! Every error variant represents a boundary violation in the DNA computation domain.
//! Zero external dependencies — manual Display + Error impls.

use core::fmt;

/// Tier: T2-P (∂ Boundary)
///
/// All errors represent boundary violations — invalid inputs, exceeded limits,
/// or structural impossibilities in DNA computation.
#[derive(Debug)]
pub enum DnaError {
    /// Invalid nucleotide character. ∂[nucleotide]
    InvalidBase(char),

    /// Strand length not divisible by 3 for codon extraction. ∂[codon]
    IncompleteCodon(usize),

    /// Attempted to transcribe an already-RNA strand. ∂[transcribe]
    AlreadyRna,

    /// No AUG start codon found during translation. ∂[translate]
    NoStartCodon,

    /// VM stack exceeded capacity. ∂[vm]
    StackOverflow(usize, usize),

    /// VM stack underflow — not enough operands. ∂[vm]
    StackUnderflow(usize),

    /// Division by zero in VM arithmetic. ∂[vm]
    DivisionByZero(usize),

    /// VM execution limit exceeded. ∂[vm]
    ExecutionLimit(u64),

    /// No AUG entry point found in VM program. ∂[vm]
    NoEntryPoint,

    /// Invalid memory or jump address. ∂[vm]
    InvalidAddress(usize),

    /// Strand length mismatch for comparison operations. ∂[ops]
    LengthMismatch(usize, usize),

    /// Index out of bounds for mutation. ∂[ops]
    IndexOutOfBounds(usize, usize),

    /// Unknown mnemonic in assembly source. ∂[asm]
    UnknownMnemonic(String),

    /// Undefined label reference. ∂[asm]
    UndefinedLabel(String),

    /// Duplicate label definition. ∂[asm]
    DuplicateLabel(String),

    /// Syntax error in assembly source. ∂[asm]
    SyntaxError(usize, String),

    /// Invalid literal value. ∂[asm]
    InvalidLiteral(String),

    /// I/O error (for CLI file operations). ∂[io]
    IoError(String),

    /// Gene not found in genome. ∂[gene]
    GeneNotFound(String),

    /// Parity check failed at the given 16-codon block index. ∂[vm]
    ParityError(usize),

    /// Type mismatch in data operation. ∂[data]
    TypeMismatch { expected: String, found: String },

    /// Schema violation (wrong fields in frame row). ∂[data]
    SchemaViolation(String),

    /// Field not found in record. ∂[data]
    FieldNotFound(String),

    /// Invalid TLV structure during decode. ∂[data]
    InvalidTlv(String),

    /// Cannot ligate collections of different types. ∂[data]
    CollectionTypeMismatch,
}

impl fmt::Display for DnaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBase(ch) => {
                write!(
                    f,
                    "invalid nucleotide base: '{ch}' (expected A, T, G, C, or U)"
                )
            }
            Self::IncompleteCodon(len) => {
                write!(
                    f,
                    "incomplete codon: strand has {len} bases (not divisible by 3)"
                )
            }
            Self::AlreadyRna => write!(f, "strand is already RNA, cannot transcribe"),
            Self::NoStartCodon => write!(f, "no start codon (AUG) found in strand"),
            Self::StackOverflow(pc, depth) => {
                write!(
                    f,
                    "stack overflow at pc={pc}: depth {depth} exceeds capacity"
                )
            }
            Self::StackUnderflow(pc) => {
                write!(f, "stack underflow at pc={pc}: insufficient operands")
            }
            Self::DivisionByZero(pc) => write!(f, "division by zero at pc={pc}"),
            Self::ExecutionLimit(cycles) => write!(f, "execution limit reached: {cycles} cycles"),
            Self::NoEntryPoint => {
                write!(f, "no entry point (AUG codon) found in program")
            }
            Self::InvalidAddress(addr) => {
                write!(f, "invalid address {addr} at current pc")
            }
            Self::LengthMismatch(a, b) => {
                write!(f, "strand length mismatch: {a} vs {b}")
            }
            Self::IndexOutOfBounds(pos, len) => {
                write!(
                    f,
                    "mutation index {pos} out of bounds for strand of length {len}"
                )
            }
            Self::UnknownMnemonic(m) => {
                write!(f, "unknown mnemonic: '{m}'")
            }
            Self::UndefinedLabel(l) => {
                write!(f, "undefined label: '@{l}'")
            }
            Self::DuplicateLabel(l) => {
                write!(f, "duplicate label: '{l}'")
            }
            Self::SyntaxError(line, msg) => {
                write!(f, "syntax error at line {line}: {msg}")
            }
            Self::InvalidLiteral(v) => {
                write!(f, "invalid literal value: '{v}'")
            }
            Self::IoError(msg) => {
                write!(f, "I/O error: {msg}")
            }
            Self::GeneNotFound(name) => {
                write!(f, "gene not found: '{name}'")
            }
            Self::ParityError(block) => {
                write!(f, "parity check failed at block {block}")
            }
            Self::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {expected}, found {found}")
            }
            Self::SchemaViolation(msg) => {
                write!(f, "schema violation: {msg}")
            }
            Self::FieldNotFound(name) => {
                write!(f, "field not found: '{name}'")
            }
            Self::InvalidTlv(msg) => {
                write!(f, "invalid TLV: {msg}")
            }
            Self::CollectionTypeMismatch => {
                write!(f, "cannot ligate collections of different types")
            }
        }
    }
}

impl std::error::Error for DnaError {}

/// Convenience Result alias for DNA operations.
pub type Result<T> = std::result::Result<T, DnaError>;
