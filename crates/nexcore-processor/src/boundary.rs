//! Boundary — ∂ (acceptance criteria).
//!
//! A boundary defines what a processor accepts and rejects.
//! Placed at entry (pre-condition) or exit (post-condition),
//! it is the ∂ in `∂(σ(μ))`.

use crate::ProcessorError;

/// A validation gate that accepts or rejects values.
///
/// Boundaries are composable: `AND`, `OR`, `NOT` produce
/// compound boundaries from simple predicates.
pub trait Boundary<T> {
    /// Test whether the value passes the boundary.
    ///
    /// # Errors
    /// Returns `ProcessorError::BoundaryRejection` with reason on failure.
    fn check(&self, value: &T) -> Result<(), ProcessorError>;

    /// Human-readable description of what this boundary enforces.
    fn description(&self) -> &str;
}

/// A boundary built from a predicate function.
pub struct PredicateBoundary<T, F>
where
    F: Fn(&T) -> bool,
{
    description: String,
    predicate: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> PredicateBoundary<T, F>
where
    F: Fn(&T) -> bool,
{
    /// Create a boundary from a predicate and description.
    pub fn new(description: impl Into<String>, predicate: F) -> Self {
        Self {
            description: description.into(),
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Boundary<T> for PredicateBoundary<T, F>
where
    F: Fn(&T) -> bool,
{
    fn check(&self, value: &T) -> Result<(), ProcessorError> {
        if (self.predicate)(value) {
            Ok(())
        } else {
            Err(ProcessorError::BoundaryRejection {
                boundary: self.description.clone(),
                reason: "predicate returned false".into(),
            })
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Logical AND of two boundaries — both must pass.
pub struct AndBoundary<T, A, B>
where
    A: Boundary<T>,
    B: Boundary<T>,
{
    left: A,
    right: B,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, A, B> AndBoundary<T, A, B>
where
    A: Boundary<T>,
    B: Boundary<T>,
{
    /// Combine two boundaries with AND logic.
    pub fn new(left: A, right: B) -> Self {
        Self {
            left,
            right,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, A, B> Boundary<T> for AndBoundary<T, A, B>
where
    A: Boundary<T>,
    B: Boundary<T>,
{
    fn check(&self, value: &T) -> Result<(), ProcessorError> {
        self.left.check(value)?;
        self.right.check(value)?;
        Ok(())
    }

    fn description(&self) -> &str {
        // Returns left description — callers can inspect both via the struct
        self.left.description()
    }
}

/// Logical OR of two boundaries — either must pass.
pub struct OrBoundary<T, A, B>
where
    A: Boundary<T>,
    B: Boundary<T>,
{
    left: A,
    right: B,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, A, B> OrBoundary<T, A, B>
where
    A: Boundary<T>,
    B: Boundary<T>,
{
    /// Combine two boundaries with OR logic.
    pub fn new(left: A, right: B) -> Self {
        Self {
            left,
            right,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, A, B> Boundary<T> for OrBoundary<T, A, B>
where
    A: Boundary<T>,
    B: Boundary<T>,
{
    fn check(&self, value: &T) -> Result<(), ProcessorError> {
        match self.left.check(value) {
            Ok(()) => Ok(()),
            Err(_) => self.right.check(value),
        }
    }

    fn description(&self) -> &str {
        self.left.description()
    }
}
