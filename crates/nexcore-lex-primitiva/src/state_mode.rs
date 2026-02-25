//! # StateMode — Disambiguated State (ς) Primitive
//!
//! Resolves the equivocation (F2 fallacy) where "state" conflated three
//! distinct algebraic objects:
//!
//! | Mode | Algebraic Object | Reversible | Rust Example |
//! |------|-----------------|------------|-------------|
//! | `Mutable` | Value changing freely in place | Yes | `Cell<T>`, `Mutex<T>` |
//! | `Modal` | Discrete FSM mode | Constrained | `enum Phase { A, B }` |
//! | `Accumulated` | Monotonic evidence growth | No | `Vec<Event>`, audit trails |
//!
//! ## Tier: T1-Universal (refines ς without adding new primitives)

use serde::{Deserialize, Serialize};

/// Disambiguated mode of the State (ς) primitive.
///
/// Three algebraically distinct objects were conflated under "state":
/// - **Mutable**: freely changing value in place (reversible)
/// - **Modal**: discrete FSM transitions (constrained reversibility)
/// - **Accumulated**: monotonic evidence growth (irreversible)
///
/// ## Tier: T1-Universal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StateMode {
    /// Value changing freely in place. Reversible.
    /// Rust: `Cell<T>`, `Mutex<T>`, `String`, `Vec<T>`, `HashMap<K,V>`
    Mutable,

    /// Discrete finite-state machine mode. Constrained transitions.
    /// Rust: `enum Phase { Init, Running, Done }`, `CircuitBreaker`
    Modal,

    /// Monotonic evidence accumulation. Irreversible (append-only).
    /// Rust: `Vec<Event>` (audit trail), `BTreeMap<Time, Evidence>`
    Accumulated,
}

impl StateMode {
    /// Human-readable label for this mode.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Mutable => "mutable",
            Self::Modal => "modal",
            Self::Accumulated => "accumulated",
        }
    }

    /// One-line description of the algebraic object.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Mutable => "Value changing freely in place (reversible)",
            Self::Modal => "Discrete FSM mode with constrained transitions",
            Self::Accumulated => "Monotonic evidence growth (irreversible, append-only)",
        }
    }

    /// Whether this mode supports reversal (going back to previous state).
    #[must_use]
    pub const fn is_reversible(&self) -> bool {
        match self {
            Self::Mutable => true,
            Self::Modal => true, // Constrained but possible
            Self::Accumulated => false,
        }
    }

    /// All three modes in canonical order.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Mutable, Self::Modal, Self::Accumulated]
    }

    /// The symbol suffix appended to ς when mode is present.
    /// e.g., `ς-mut`, `ς-mod`, `ς-acc`
    #[must_use]
    pub const fn symbol_suffix(&self) -> &'static str {
        match self {
            Self::Mutable => "ς-mut",
            Self::Modal => "ς-mod",
            Self::Accumulated => "ς-acc",
        }
    }
}

impl std::fmt::Display for StateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol_suffix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_modes_count() {
        assert_eq!(StateMode::all().len(), 3);
    }

    #[test]
    fn test_reversibility() {
        assert!(StateMode::Mutable.is_reversible());
        assert!(StateMode::Modal.is_reversible());
        assert!(!StateMode::Accumulated.is_reversible());
    }

    #[test]
    fn test_labels() {
        assert_eq!(StateMode::Mutable.label(), "mutable");
        assert_eq!(StateMode::Modal.label(), "modal");
        assert_eq!(StateMode::Accumulated.label(), "accumulated");
    }

    #[test]
    fn test_serde_round_trip() {
        for mode in StateMode::all() {
            let json = serde_json::to_string(&mode).expect("serialize");
            let back: StateMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, mode, "Serde round-trip failed for {:?}", mode);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", StateMode::Mutable), "ς-mut");
        assert_eq!(format!("{}", StateMode::Modal), "ς-mod");
        assert_eq!(format!("{}", StateMode::Accumulated), "ς-acc");
    }
}
