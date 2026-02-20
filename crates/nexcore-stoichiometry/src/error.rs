//! Stoichiometry error types.

use thiserror::Error;

/// Errors from stoichiometric operations.
#[derive(Debug, Error)]
pub enum StoichiometryError {
    /// Equation does not balance — mass is not conserved.
    #[error(
        "imbalance: reactant mass {reactant_mass:.2} Da != product mass {product_mass:.2} Da (delta={delta:.2})"
    )]
    Imbalance {
        reactant_mass: f64,
        product_mass: f64,
        delta: f64,
    },

    /// A word in the definition could not be decomposed to primitives.
    #[error("unknown word: '{word}' has no primitive mapping")]
    UnknownWord { word: String },

    /// The concept name is empty.
    #[error("empty concept name")]
    EmptyConcept,

    /// The definition text is empty.
    #[error("empty definition")]
    EmptyDefinition,

    /// A term is already registered in the dictionary.
    #[error("duplicate term: '{name}' already registered")]
    DuplicateTerm { name: String },

    /// Term not found in dictionary.
    #[error("term not found: '{name}'")]
    TermNotFound { name: String },

    /// Balance proof verification failed.
    #[error("balance proof invalid: {reason}")]
    InvalidProof { reason: String },
}
