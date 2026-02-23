//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to this crate.
//!
//! ## Cytokine Primitive Grounding
//!
//! | Domain Concept       | T1 Primitive | Symbol | Justification |
//! |----------------------|-------------|--------|---------------|
//! | Signal emission      | Causality   | →      | Emitter causes receptor activation |
//! | Receptor binding     | Mapping     | μ      | Typed matching of signal to handler |
//! | Cascade amplification| Recursion   | ρ      | One signal triggers many downstream signals |
//! | Concentration/strength| Quantity   | N      | Signal severity and potency |
//! | Signal scope         | Location    | λ      | Paracrine (local) vs endocrine (systemic) |
//! | Signal duration      | Persistence | π      | Transient vs sustained signaling |
//! | Receptor state       | State       | ς      | Unbound → bound → internalized |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
