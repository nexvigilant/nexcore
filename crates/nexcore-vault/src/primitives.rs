//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to this crate.
//!
//! ## Vault Primitive Grounding
//!
//! | Domain Concept      | T1 Primitive | Symbol | Justification |
//! |---------------------|-------------|--------|---------------|
//! | Secret mapping      | Mapping     | μ      | SecretName → EncryptedValue bijection |
//! | Vault lifecycle     | State       | ς      | create → open → operate → save |
//! | Secret existence    | Existence   | ∃      | Presence check before get/delete |
//! | KDF iteration       | Recursion   | ρ      | PBKDF2 repeated hashing |
//! | Persistence on disk | Persistence | π      | Encrypted file survives process restart |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
