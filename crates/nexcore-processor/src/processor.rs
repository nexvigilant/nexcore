//! Core processor trait — μ (mapping).
//!
//! A processor is a bounded sequential mapping: `∂(σ(μ))`.
//! This module defines the irreducible kernel: the mapping itself.

use crate::ProcessorError;

/// The core processor primitive — a mapping from input to output.
///
/// Every processor is fundamentally a μ: `I → O` with fallibility.
/// Boundary (∂), sequence (σ), and state (ς) are compositional
/// enrichments layered on top.
pub trait Processor {
    /// Input type consumed by this processor.
    type Input;
    /// Output type produced by this processor.
    type Output;

    /// Execute the mapping: `μ(input) → Result<output>`.
    ///
    /// # Errors
    /// Returns `ProcessorError` when the mapping fails — boundary
    /// rejection, transformation error, or state corruption.
    fn process(&self, input: Self::Input) -> Result<Self::Output, ProcessorError>;

    /// Human-readable name for diagnostics and pipeline tracing.
    fn name(&self) -> &str;
}

/// Adapter: wrap a pure function as a `Processor`.
///
/// The simplest processor subtype — no state, no feedback,
/// just `μ: I → Result<O>`.
pub struct FnProcessor<I, O, F>
where
    F: Fn(I) -> Result<O, ProcessorError>,
{
    name: String,
    f: F,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O, F> FnProcessor<I, O, F>
where
    F: Fn(I) -> Result<O, ProcessorError>,
{
    /// Create a new function-based processor.
    pub fn new(name: impl Into<String>, f: F) -> Self {
        Self {
            name: name.into(),
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I, O, F> Processor for FnProcessor<I, O, F>
where
    F: Fn(I) -> Result<O, ProcessorError>,
{
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Result<Self::Output, ProcessorError> {
        (self.f)(input)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
