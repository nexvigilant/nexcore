//! # GroundsTo Trait
//!
//! Compile-time verification that types ground to T1 primitives.
//!
//! ## Usage
//!
//! ```ignore
//! impl GroundsTo for MyType {
//!     fn primitive_composition() -> PrimitiveComposition {
//!         PrimitiveComposition::new(vec![
//!             LexPrimitiva::Comparison,
//!             LexPrimitiva::Boundary,
//!         ])
//!         .with_dominant(LexPrimitiva::Comparison, 0.95)
//!     }
//! }
//! ```

use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::state_mode::StateMode;
use crate::tier::Tier;

/// Trait for types that ground to T1 primitives.
///
/// Implementing this trait declares how a type composes from the 16 Lex Primitiva.
pub trait GroundsTo {
    /// Returns the primitive composition that grounds this type.
    fn primitive_composition() -> PrimitiveComposition;

    /// Returns the dominant primitive (convenience method).
    fn dominant_primitive() -> Option<LexPrimitiva> {
        Self::primitive_composition().dominant
    }

    /// Returns true if this type is purely one primitive.
    fn is_pure_primitive() -> bool {
        Self::primitive_composition().is_pure()
    }

    /// Returns the tier classification.
    fn tier() -> Tier {
        Tier::classify(&Self::primitive_composition())
    }

    /// Returns the disambiguated State (ς) mode, if applicable.
    ///
    /// Default returns `None`. Override for types that involve state
    /// to specify whether the state is mutable, modal, or accumulated.
    fn state_mode() -> Option<StateMode> {
        None
    }
}

// ============================================================================
// Standard Library Type Groundings
// ============================================================================

impl GroundsTo for bool {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

impl GroundsTo for u8 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u16 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u32 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u64 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u128 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for usize {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i8 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i16 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i32 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i64 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i128 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for isize {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for f32 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for f64 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for () {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void]).with_dominant(LexPrimitiva::Void, 1.0)
    }
}

impl GroundsTo for char {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for String {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Sequence, 0.9)
            .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for str {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sequence, 1.0)
    }
}

impl<T> GroundsTo for Option<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Void, 0.95)
    }
}

impl<T, E> GroundsTo for Result<T, E> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.9)
    }
}

impl<T> GroundsTo for Vec<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<T> GroundsTo for Box<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Location, 0.9)
    }
}

impl<T> GroundsTo for std::rc::Rc<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Location, 0.8)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<T> GroundsTo for std::sync::Arc<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Location, 0.8)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<T> GroundsTo for std::cell::Cell<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

impl<T> GroundsTo for std::cell::RefCell<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::State, 0.9)
            .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<T> GroundsTo for std::sync::Mutex<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::State, 0.85)
            .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<K, V> GroundsTo for std::collections::HashMap<K, V> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.9)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<T> GroundsTo for std::collections::HashSet<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl<T> GroundsTo for std::collections::VecDeque<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Sequence, 0.9)
            .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for std::path::PathBuf {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Location, 0.95)
    }
}

impl GroundsTo for std::path::Path {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Location, 1.0)
    }
}

impl GroundsTo for std::time::Duration {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Quantity, 0.9)
    }
}

impl GroundsTo for std::time::Instant {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sequence, 0.9)
    }
}

// Tuple types: canonical product types in Rust

impl<A, B> GroundsTo for (A, B) {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Product, 1.0)
    }
}

impl<A, B, C> GroundsTo for (A, B, C) {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Product, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Product, 0.9)
    }
}

impl<A, B, C, D> GroundsTo for (A, B, C, D) {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Product, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_groundings() {
        assert_eq!(u32::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert!(u32::is_pure_primitive());
        assert_eq!(u32::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_option_grounding() {
        assert_eq!(
            <Option<i32>>::dominant_primitive(),
            Some(LexPrimitiva::Void)
        );
        assert_eq!(<Option<i32>>::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_result_grounding() {
        assert_eq!(
            <Result<(), ()>>::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_unit_grounding() {
        assert_eq!(<()>::dominant_primitive(), Some(LexPrimitiva::Void));
        assert!(<()>::is_pure_primitive());
    }

    #[test]
    fn test_vec_grounding() {
        assert_eq!(
            <Vec<i32>>::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(<Vec<i32>>::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_hashmap_grounding() {
        use std::collections::HashMap;
        assert_eq!(
            <HashMap<String, i32>>::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_path_grounding() {
        assert_eq!(
            <std::path::Path>::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
        assert!(<std::path::Path>::is_pure_primitive());
    }

    #[test]
    fn test_tuple_pair_grounding() {
        assert_eq!(
            <(i32, i32)>::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
        assert!(<(i32, i32)>::is_pure_primitive());
        assert_eq!(<(i32, i32)>::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_tuple_triple_grounding() {
        assert_eq!(
            <(i32, i32, i32)>::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
        assert!(!<(i32, i32, i32)>::is_pure_primitive());
        assert_eq!(<(i32, i32, i32)>::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_tuple_quad_grounding() {
        assert_eq!(
            <(i32, i32, i32, i32)>::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
        assert_eq!(<(i32, i32, i32, i32)>::tier(), Tier::T2Primitive);
    }

    // ── StateMode integration tests ─────────────────────────────────────

    #[test]
    fn test_stdlib_state_modes() {
        use crate::state_mode::StateMode;
        use std::cell::RefCell;
        use std::collections::{HashMap, HashSet, VecDeque};
        use std::rc::Rc;
        use std::sync::{Arc, Mutex};

        // All 9 stdlib types annotated in Step 4 must return Mutable
        assert_eq!(<String>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(<Vec<i32>>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(<Rc<i32>>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(<Arc<i32>>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(<RefCell<i32>>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(<Mutex<i32>>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(
            <HashMap<String, i32>>::state_mode(),
            Some(StateMode::Mutable)
        );
        assert_eq!(<HashSet<i32>>::state_mode(), Some(StateMode::Mutable));
        assert_eq!(<VecDeque<i32>>::state_mode(), Some(StateMode::Mutable));

        // Verify composition also carries the mode
        let comp = <String>::primitive_composition();
        assert_eq!(comp.state_mode, Some(StateMode::Mutable));
    }

    #[test]
    fn test_non_state_types_have_no_mode() {
        use crate::state_mode::StateMode;

        // Pure value types have no state mode
        assert_eq!(<u32>::state_mode(), None);
        assert_eq!(<bool>::state_mode(), None);
        assert_eq!(<()>::state_mode(), None);
        assert_eq!(<f64>::state_mode(), None);
        assert_eq!(<(i32, i32)>::state_mode(), None);
        assert_eq!(<Option<i32>>::state_mode(), None);
        assert_eq!(<Result<(), ()>>::state_mode(), None);

        // Composition also has None
        let comp = <u32>::primitive_composition();
        assert_eq!(comp.state_mode, None);
    }

    #[test]
    fn test_state_mode_backward_compat() {
        // Old JSON without state_mode deserializes to None
        let json = r#"{
            "primitives": ["state"],
            "dominant": "state",
            "confidence": 0.85
        }"#;
        let comp: PrimitiveComposition =
            serde_json::from_str(json).expect("backward compat deserialization");
        assert_eq!(comp.state_mode, None);

        // New JSON with state_mode round-trips correctly
        let json_with_mode = r#"{
            "primitives": ["state"],
            "dominant": "state",
            "confidence": 0.85,
            "state_mode": "mutable"
        }"#;
        let comp_with: PrimitiveComposition =
            serde_json::from_str(json_with_mode).expect("new format deserialization");
        assert_eq!(
            comp_with.state_mode,
            Some(crate::state_mode::StateMode::Mutable)
        );

        // Serializing without mode omits the field
        let serialized = serde_json::to_string(&comp).expect("serialization");
        assert!(!serialized.contains("state_mode"));

        // Serializing with mode includes the field
        let serialized_with = serde_json::to_string(&comp_with).expect("serialization");
        assert!(serialized_with.contains("state_mode"));
    }
}
