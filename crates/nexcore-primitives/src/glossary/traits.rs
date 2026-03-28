//! # Glossary Enforcement Traits
//!
//! Traits that wire the primitive types together and enforce the Lex Primitiva
//! contracts at compile time. Implementing any of these traits declares that
//! a type participates in the corresponding primitive protocol.
//!
//! The traits use **associated types** rather than generics on the primary
//! dimension so that each implementing type has exactly one canonical
//! measurement type, one snapshot type, etc. Generic parameters are reserved
//! for secondary dimensions that naturally vary per call site.

use super::comparison::Quantity;
use super::foundation::{Boundary, State};

// ─── PersistError ─────────────────────────────────────────────────────────────

/// Errors that may occur when persisting a value.
///
/// `PersistError` is the error type for the [`Persistent`] trait's
/// `persist` method. It covers the two failure modes that arise when
/// writing to external storage: I/O failure and serialisation failure.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::traits::PersistError;
///
/// let e = PersistError::IoError("disk full".to_string());
/// assert!(matches!(e, PersistError::IoError(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistError {
    /// A low-level I/O error, e.g. disk full or permission denied.
    IoError(String),
    /// The value could not be serialised to the target format.
    SerializationError(String),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
            Self::SerializationError(msg) => write!(f, "serialization error: {msg}"),
        }
    }
}

impl std::error::Error for PersistError {}

// ─── HasBoundary ──────────────────────────────────────────────────────────────

/// A type that carries an explicit ∂ (Boundary) on its primary measure.
///
/// Implementing `HasBoundary` declares that the type has a well-defined
/// valid range. The associated type `Measure` is the dimension being bounded;
/// it must implement `PartialOrd` so the boundary can classify values.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::foundation::Boundary;
/// use nexcore_primitives::glossary::traits::HasBoundary;
///
/// struct NormalRange {
///     boundary: Boundary<f64>,
/// }
///
/// impl HasBoundary for NormalRange {
///     type Measure = f64;
///
///     fn boundary(&self) -> &Boundary<f64> {
///         &self.boundary
///     }
/// }
///
/// let r = NormalRange {
///     boundary: Boundary::new(80.0_f64, 120.0_f64).unwrap(),
/// };
/// assert!(r.boundary().contains(&100.0));
/// ```
pub trait HasBoundary {
    /// The dimension being bounded. Must be `PartialOrd` so that
    /// [`Boundary`] can classify values against the [min, max] range.
    type Measure: PartialOrd;

    /// Return a reference to the type's boundary.
    fn boundary(&self) -> &Boundary<Self::Measure>;
}

// ─── HasState ─────────────────────────────────────────────────────────────────

/// A type that exposes its current ς (State) as a timestamped snapshot.
///
/// Implementing `HasState` declares that the type's current observable
/// condition can be captured at a point in time. The associated type
/// `Snapshot` must be `Clone` so callers can hold copies without borrowing
/// the original.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::foundation::State;
/// use nexcore_primitives::glossary::traits::HasState;
///
/// struct Sensor {
///     reading: f64,
/// }
///
/// impl HasState for Sensor {
///     type Snapshot = f64;
///
///     fn state(&self) -> State<f64> {
///         State::new(self.reading)
///     }
/// }
///
/// let sensor = Sensor { reading: 98.6 };
/// assert_eq!(*sensor.state().value(), 98.6);
/// ```
pub trait HasState {
    /// The observable snapshot type. Must be `Clone` so callers can
    /// retain copies without keeping a borrow on the original.
    type Snapshot: Clone;

    /// Capture the current state as a timestamped snapshot.
    fn state(&self) -> State<Self::Snapshot>;
}

// ─── Measurable ───────────────────────────────────────────────────────────────

/// A type that yields a [`Quantity`] (N) representation of its primary value.
///
/// `Measurable` is the single-method protocol for types that have a
/// meaningful numeric magnitude with an associated unit. The returned
/// `Quantity` carries both the `f64` value and its unit string, preventing
/// silent unit mixing at call sites.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::comparison::Quantity;
/// use nexcore_primitives::glossary::traits::Measurable;
///
/// struct PatientWeight {
///     kg: f64,
/// }
///
/// impl Measurable for PatientWeight {
///     fn measure(&self) -> Quantity {
///         Quantity::new(self.kg, "kg")
///     }
/// }
///
/// let w = PatientWeight { kg: 72.5 };
/// assert_eq!(w.measure().value, 72.5);
/// assert_eq!(w.measure().unit, "kg");
/// ```
pub trait Measurable {
    /// Return the primary quantity for this value.
    fn measure(&self) -> Quantity;
}

// ─── Persistent ───────────────────────────────────────────────────────────────

/// A type that supports explicit persistence (π) with dirty tracking.
///
/// `Persistent` declares that a type can be written to external storage
/// and tracks whether its in-memory state has diverged from what was last
/// persisted. Callers check [`is_dirty`](Persistent::is_dirty) before
/// deciding whether a write is needed.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::traits::{Persistent, PersistError};
///
/// struct InMemoryStore {
///     data: String,
///     dirty: bool,
/// }
///
/// impl Persistent for InMemoryStore {
///     fn persist(&mut self) -> Result<(), PersistError> {
///         // In a real implementation, write `self.data` to disk.
///         self.dirty = false;
///         Ok(())
///     }
///
///     fn is_dirty(&self) -> bool {
///         self.dirty
///     }
/// }
///
/// let mut store = InMemoryStore { data: "hello".into(), dirty: true };
/// assert!(store.is_dirty());
/// store.persist().unwrap();
/// assert!(!store.is_dirty());
/// ```
pub trait Persistent {
    /// Flush any unsaved changes to external storage.
    ///
    /// On success, implementations SHOULD clear the dirty flag. On failure
    /// the dirty flag MUST remain set so the next call will retry.
    fn persist(&mut self) -> Result<(), PersistError>;

    /// Returns `true` when the in-memory state has changed since the last
    /// successful [`persist`](Persistent::persist) call.
    fn is_dirty(&self) -> bool;
}

// ─── Irreversible ─────────────────────────────────────────────────────────────

/// A type that represents an action that cannot be undone (∝).
///
/// Implementing `Irreversible` declares that an action has been committed
/// and its effects are permanent. The trait provides introspection into
/// whether the action is committed and why it was committed.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::traits::Irreversible;
///
/// struct DatabaseDrop {
///     committed: bool,
///     reason: String,
/// }
///
/// impl Irreversible for DatabaseDrop {
///     fn is_committed(&self) -> bool {
///         self.committed
///     }
///
///     fn commitment_reason(&self) -> &str {
///         &self.reason
///     }
/// }
///
/// let drop = DatabaseDrop {
///     committed: true,
///     reason: "schema migration v4".to_string(),
/// };
/// assert!(drop.is_committed());
/// assert_eq!(drop.commitment_reason(), "schema migration v4");
/// ```
pub trait Irreversible {
    /// Returns `true` when the action has been committed and cannot be undone.
    fn is_committed(&self) -> bool;

    /// A human-readable justification for why the action was committed.
    fn commitment_reason(&self) -> &str;
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::foundation::Boundary;
    use crate::glossary::foundation::State;

    // ── PersistError ─────────────────────────────────────────────────────────

    #[test]
    fn persist_error_io_displays_message() {
        let e = PersistError::IoError("disk full".to_string());
        assert!(e.to_string().contains("disk full"));
    }

    #[test]
    fn persist_error_serialization_displays_message() {
        let e = PersistError::SerializationError("invalid utf-8".to_string());
        assert!(e.to_string().contains("invalid utf-8"));
    }

    #[test]
    fn persist_error_variants_are_distinct() {
        let io = PersistError::IoError("x".into());
        let ser = PersistError::SerializationError("x".into());
        assert_ne!(io, ser);
    }

    // ── HasBoundary ──────────────────────────────────────────────────────────

    struct NormalRange {
        boundary: Boundary<f64>,
    }

    impl HasBoundary for NormalRange {
        type Measure = f64;
        fn boundary(&self) -> &Boundary<f64> {
            &self.boundary
        }
    }

    #[test]
    fn has_boundary_contains_midpoint() {
        let r = NormalRange {
            boundary: Boundary::new(80.0_f64, 120.0_f64).unwrap(),
        };
        assert!(r.boundary().contains(&100.0));
        assert!(!r.boundary().contains(&60.0));
    }

    // ── HasState ─────────────────────────────────────────────────────────────

    struct Sensor {
        reading: f64,
    }

    impl HasState for Sensor {
        type Snapshot = f64;
        fn state(&self) -> State<f64> {
            State::new(self.reading)
        }
    }

    #[test]
    fn has_state_captures_reading() {
        let sensor = Sensor { reading: 98.6 };
        let s = sensor.state();
        assert_eq!(*s.value(), 98.6);
        assert!(s.captured_at() > 0);
    }

    // ── Measurable ───────────────────────────────────────────────────────────

    struct PatientWeight {
        kg: f64,
    }

    impl Measurable for PatientWeight {
        fn measure(&self) -> Quantity {
            Quantity::new(self.kg, "kg")
        }
    }

    #[test]
    fn measurable_returns_quantity_with_unit() {
        let w = PatientWeight { kg: 72.5 };
        let q = w.measure();
        assert_eq!(q.value, 72.5);
        assert_eq!(q.unit, "kg");
    }

    // ── Persistent ───────────────────────────────────────────────────────────

    struct InMemoryStore {
        dirty: bool,
    }

    impl Persistent for InMemoryStore {
        fn persist(&mut self) -> Result<(), PersistError> {
            self.dirty = false;
            Ok(())
        }
        fn is_dirty(&self) -> bool {
            self.dirty
        }
    }

    #[test]
    fn persistent_persist_clears_dirty() {
        let mut store = InMemoryStore { dirty: true };
        assert!(store.is_dirty());
        store.persist().unwrap();
        assert!(!store.is_dirty());
    }

    #[test]
    fn persistent_failing_persist_preserves_dirty() {
        struct FailingStore;
        impl Persistent for FailingStore {
            fn persist(&mut self) -> Result<(), PersistError> {
                Err(PersistError::IoError("no disk".into()))
            }
            fn is_dirty(&self) -> bool {
                true
            }
        }
        let mut s = FailingStore;
        let result = s.persist();
        assert!(result.is_err());
        assert!(s.is_dirty());
    }

    // ── Irreversible ─────────────────────────────────────────────────────────

    struct SealedAudit {
        reason: String,
    }

    impl Irreversible for SealedAudit {
        fn is_committed(&self) -> bool {
            true
        }
        fn commitment_reason(&self) -> &str {
            &self.reason
        }
    }

    #[test]
    fn irreversible_committed_returns_true() {
        let audit = SealedAudit {
            reason: "end of quarter seal".to_string(),
        };
        assert!(audit.is_committed());
    }

    #[test]
    fn irreversible_reason_matches_construction() {
        let audit = SealedAudit {
            reason: "migration v12".to_string(),
        };
        assert_eq!(audit.commitment_reason(), "migration v12");
    }
}
