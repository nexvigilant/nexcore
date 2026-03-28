//! # T1 Primitive Glossary — Foundation Types
//!
//! This module encodes four of the fifteen Lex Primitiva T1 primitives as
//! Rust types with compiler-enforced contracts.
//!
//! | Type | T1 Symbol | Semantics |
//! |------|-----------|-----------|
//! | [`State<T>`] | ς | Timestamped snapshot — observed state at a moment |
//! | [`Boundary<T>`] | ∂ | Inside/outside classifier — structurally valid range |
//! | [`Void`] | ∅ | Zero-sized marker — meaningful absence is the answer |
//! | [`Existence<T>`] | ∃ | Option newtype — naming makes the conservitor law explicit |
//!
//! These are the substrate on which every higher NexVigilant type builds.
//! Misuse is a compile error, not a runtime panic.

use std::ops::Sub;

use serde::{Deserialize, Serialize};

// ─── BoundaryError ───────────────────────────────────────────────────────────

/// Error returned when a [`Boundary`] cannot be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryError {
    /// `lower > upper` — the bounds are inverted.
    InvertedBounds,
}

impl std::fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundaryError::InvertedBounds => {
                f.write_str("boundary lower bound exceeds upper bound (inverted bounds)")
            }
        }
    }
}

impl std::error::Error for BoundaryError {}

// ─── State ───────────────────────────────────────────────────────────────────

/// A timestamped snapshot of a value `T`.
///
/// `State<T>` represents ς — observed state at a point in time.
/// It is deliberately `Clone`-but-not-`Copy`: copying state silently would
/// lose the semantics of "this was observed at *this* moment".
///
/// `captured_at` is stored as Unix epoch milliseconds.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::foundation::State;
///
/// let s = State::new(42u32);
/// assert_eq!(*s.value(), 42u32);
/// // age_millis returns millis elapsed since capture (always >= 0)
/// let _ = s.age_millis();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State<T: Clone> {
    value: T,
    /// Unix epoch milliseconds at which the state was captured.
    captured_at: u64,
}

impl<T: Clone> State<T> {
    /// Capture `value` at the current wall-clock time (epoch millis).
    pub fn new(value: T) -> Self {
        Self {
            value,
            captured_at: epoch_millis_now(),
        }
    }

    /// Reference to the inner value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Epoch milliseconds at which this snapshot was taken.
    #[inline]
    pub fn captured_at(&self) -> u64 {
        self.captured_at
    }

    /// Milliseconds elapsed since this snapshot was captured.
    ///
    /// Returns 0 if the system clock moved backwards.
    pub fn age_millis(&self) -> u64 {
        epoch_millis_now().saturating_sub(self.captured_at)
    }
}

/// Return the current time as Unix epoch milliseconds.
///
/// Uses `std::time::SystemTime` — no external dependencies.
fn epoch_millis_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ─── Boundary ────────────────────────────────────────────────────────────────

/// An inclusive [`lower`, `upper`] range that classifies values as inside or outside.
///
/// `Boundary<T>` represents ∂ — the dividing surface between two regions.
/// Construction fails (returns `Err(BoundaryError::InvertedBounds)`) when
/// `lower > upper`, so every live instance is structurally sound.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::foundation::{Boundary, BoundaryError};
///
/// let b = Boundary::new(0.0_f64, 1.0_f64).unwrap();
/// assert!(b.contains(&0.5));
/// assert!(!b.contains(&1.5));
///
/// assert_eq!(Boundary::new(1.0_f64, 0.0_f64), Err(BoundaryError::InvertedBounds));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary<T: PartialOrd> {
    lower: T,
    upper: T,
}

impl<T: PartialOrd> Boundary<T> {
    /// Construct an inclusive [`lower`, `upper`] boundary.
    ///
    /// Returns `Err(BoundaryError::InvertedBounds)` if `lower > upper`.
    pub fn new(lower: T, upper: T) -> Result<Self, BoundaryError> {
        if lower > upper {
            return Err(BoundaryError::InvertedBounds);
        }
        Ok(Self { lower, upper })
    }

    /// Returns `true` if `value` lies within [`lower`, `upper`] inclusive.
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        value >= &self.lower && value <= &self.upper
    }

    /// Returns `true` if `value` is within `epsilon` of either bound.
    ///
    /// Requires `T: Sub<Output = T> + Copy` so we can compute `|value - bound|`.
    pub fn is_at_edge(&self, value: &T, epsilon: &T) -> bool
    where
        T: Sub<Output = T> + Copy + PartialOrd,
    {
        // distance from lower bound
        let dist_lower = if *value >= self.lower {
            *value - self.lower
        } else {
            self.lower - *value
        };
        // distance from upper bound
        let dist_upper = if *value >= self.upper {
            *value - self.upper
        } else {
            self.upper - *value
        };
        &dist_lower <= epsilon || &dist_upper <= epsilon
    }

    /// Reference to the lower bound.
    #[inline]
    pub fn lower(&self) -> &T {
        &self.lower
    }

    /// Reference to the upper bound.
    #[inline]
    pub fn upper(&self) -> &T {
        &self.upper
    }
}

// ─── Void ────────────────────────────────────────────────────────────────────

/// Zero-sized marker for meaningful absence (∅).
///
/// `Void` is not "nothing happened" — it is "absence is the answer."
/// When a function returns `Void`, the absence itself IS the information.
/// Use it to make the distinction between "no data" and "data not yet
/// requested" explicit at the type level.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::foundation::Void;
///
/// let v = Void;
/// assert_eq!(std::mem::size_of_val(&v), 0);
/// assert_eq!(v, Void::default());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Void;

impl Default for Void {
    #[inline]
    fn default() -> Self {
        Void
    }
}

// ─── Existence ───────────────────────────────────────────────────────────────

/// An `Option<T>` newtype where `None` = void (∅) and `Some` = exists (∃).
///
/// `Existence<T>` gives the standard `Option` its Lex Primitiva semantics:
/// the naming makes the conservitor law explicit in calling code.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::foundation::Existence;
///
/// let p: Existence<u32> = Existence::present(42);
/// let a: Existence<u32> = Existence::absent();
///
/// assert!(p.is_present());
/// assert!(a.is_void());
/// assert_eq!(p.into_inner(), Some(42));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Existence<T>(Option<T>);

impl<T> Existence<T> {
    /// Construct an existing value (∃).
    #[inline]
    pub fn present(value: T) -> Self {
        Self(Some(value))
    }

    /// Construct a void (absent) existence (∅).
    #[inline]
    pub fn absent() -> Self {
        Self(None)
    }

    /// `true` when there is no value (∅).
    #[inline]
    pub fn is_void(&self) -> bool {
        self.0.is_none()
    }

    /// `true` when a value is present (∃).
    #[inline]
    pub fn is_present(&self) -> bool {
        self.0.is_some()
    }

    /// Consume and return the inner `Option<T>`.
    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> Default for Existence<T> {
    /// The default existence is void (∅).
    #[inline]
    fn default() -> Self {
        Self::absent()
    }
}

impl<T> From<Option<T>> for Existence<T> {
    #[inline]
    fn from(opt: Option<T>) -> Self {
        Self(opt)
    }
}

impl<T> From<Existence<T>> for Option<T> {
    #[inline]
    fn from(e: Existence<T>) -> Self {
        e.0
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // --- State ---

    #[test]
    fn state_new_captures_value() {
        let s = State::new(99u32);
        assert_eq!(*s.value(), 99u32);
    }

    #[test]
    fn state_captured_at_is_recent_epoch_millis() {
        let before = epoch_millis_now();
        let s = State::new("signal");
        let after = epoch_millis_now();
        // captured_at must lie in [before, after]
        assert!(s.captured_at() >= before);
        assert!(s.captured_at() <= after);
    }

    #[test]
    fn state_age_millis_non_negative() {
        let s = State::new(42u64);
        // age should be >= 0 (saturating_sub guarantees this)
        let _ = s.age_millis(); // just must not panic
    }

    #[test]
    fn state_clone_is_independent() {
        let s1 = State::new(vec![1u8, 2, 3]);
        let s2 = s1.clone();
        assert_eq!(s1.value(), s2.value());
        assert_eq!(s1.captured_at(), s2.captured_at());
    }

    // --- BoundaryError ---

    #[test]
    fn boundary_error_display() {
        let e = BoundaryError::InvertedBounds;
        let msg = format!("{e}");
        assert!(msg.contains("inverted"));
    }

    // --- Boundary ---

    #[test]
    fn boundary_valid_range_contains() {
        let b = Boundary::new(0u32, 10u32).unwrap();
        assert!(b.contains(&0));
        assert!(b.contains(&5));
        assert!(b.contains(&10));
        assert!(!b.contains(&11));
    }

    #[test]
    fn boundary_inverted_range_returns_err() {
        let result = Boundary::new(10u32, 0u32);
        assert_eq!(result, Err(BoundaryError::InvertedBounds));
    }

    #[test]
    fn boundary_equal_bounds_is_valid() {
        let b = Boundary::new(5u32, 5u32).unwrap();
        assert!(b.contains(&5));
        assert!(!b.contains(&4));
    }

    #[test]
    fn boundary_is_at_edge_lower() {
        let b = Boundary::new(0.0f64, 1.0f64).unwrap();
        // 0.05 is within 0.1 of the lower bound (0.0)
        assert!(b.is_at_edge(&0.05f64, &0.1f64));
    }

    #[test]
    fn boundary_is_at_edge_upper() {
        let b = Boundary::new(0.0f64, 1.0f64).unwrap();
        // 0.95 is within 0.1 of the upper bound (1.0)
        assert!(b.is_at_edge(&0.95f64, &0.1f64));
    }

    #[test]
    fn boundary_not_at_edge_interior() {
        let b = Boundary::new(0.0f64, 10.0f64).unwrap();
        // 5.0 is far from both bounds with epsilon 0.1
        assert!(!b.is_at_edge(&5.0f64, &0.1f64));
    }

    // --- Void ---

    #[test]
    fn void_is_zero_sized() {
        assert_eq!(std::mem::size_of::<Void>(), 0);
    }

    #[test]
    fn void_default_equals_void() {
        assert_eq!(Void::default(), Void);
    }

    #[test]
    fn void_copy_semantics() {
        let v1 = Void;
        let v2 = v1; // Copy
        assert_eq!(v1, v2);
    }

    #[test]
    fn void_serde_roundtrip() {
        let v = Void;
        let json = serde_json::to_string(&v).unwrap();
        let restored: Void = serde_json::from_str(&json).unwrap();
        assert_eq!(v, restored);
    }

    // --- Existence ---

    #[test]
    fn existence_present_is_present() {
        let e: Existence<u32> = Existence::present(42);
        assert!(e.is_present());
        assert!(!e.is_void());
    }

    #[test]
    fn existence_absent_is_void() {
        let e: Existence<u32> = Existence::absent();
        assert!(e.is_void());
        assert!(!e.is_present());
    }

    #[test]
    fn existence_into_inner_present() {
        let e = Existence::present(7u32);
        assert_eq!(e.into_inner(), Some(7u32));
    }

    #[test]
    fn existence_into_inner_absent() {
        let e: Existence<u32> = Existence::absent();
        assert_eq!(e.into_inner(), None);
    }

    #[test]
    fn existence_default_is_void() {
        let e: Existence<String> = Existence::default();
        assert!(e.is_void());
    }

    #[test]
    fn existence_roundtrip_from_option() {
        let opt: Option<u32> = Some(99);
        let e: Existence<u32> = opt.into();
        assert!(e.is_present());
        let back: Option<u32> = e.into();
        assert_eq!(back, Some(99));
    }

    #[test]
    fn existence_serde_roundtrip() {
        let e = Existence::present(42u32);
        let json = serde_json::to_string(&e).unwrap();
        let restored: Existence<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.into_inner(), Some(42u32));
    }
}
