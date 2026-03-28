//! Primitive Glossary — T1 and T2-P Lex Primitiva encoded as Rust types.
//!
//! The compiler IS the validator. If it type-checks, the architecture respects
//! primitive contracts. State, Boundary, Void, Existence, Comparison, Threshold,
//! Quantity, Effect, Sequence, Persistence, Irreversibility — all enforced at
//! compile time.

pub mod causal;
pub mod comparison;
pub mod foundation;
pub mod traits;

pub use causal::{Effect, Irreversibility, Persistence, Sequence};
pub use comparison::{Comparison, Ordering, Quantity, Threshold};
pub use foundation::{Boundary, BoundaryError, Existence, State, Void};
pub use traits::{HasBoundary, HasState, Irreversible, Measurable, PersistError, Persistent};
