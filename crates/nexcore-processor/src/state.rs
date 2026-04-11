//! Stateful processor — ς enrichment.
//!
//! Adds mutable state that persists across invocations.
//! Transforms a pure processor into a stateful one by
//! pairing it with an accumulator function.

use crate::ProcessorError;
use crate::processor::Processor;

/// Wraps a `Processor` with accumulating state.
///
/// On each `process` call:
/// 1. The inner processor maps `I → O`
/// 2. The accumulator updates state from `(&S, &O) → S`
/// 3. Output is returned
///
/// State is readable via `state()` at any time.
pub struct Accumulator<P, S, A>
where
    P: Processor,
    A: Fn(&S, &P::Output) -> S,
{
    inner: P,
    state: S,
    accumulate: A,
}

impl<P, S, A> Accumulator<P, S, A>
where
    P: Processor,
    A: Fn(&S, &P::Output) -> S,
{
    /// Create a stateful processor with initial state and accumulator.
    pub fn new(inner: P, initial_state: S, accumulate: A) -> Self {
        Self {
            inner,
            state: initial_state,
            accumulate,
        }
    }

    /// Read-only access to current state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Process input, updating internal state.
    ///
    /// # Errors
    /// Returns `ProcessorError` if the inner processor fails.
    pub fn process(&mut self, input: P::Input) -> Result<P::Output, ProcessorError> {
        let output = self.inner.process(input)?;
        self.state = (self.accumulate)(&self.state, &output);
        Ok(output)
    }

    /// Reset state to a new value.
    pub fn reset(&mut self, state: S) {
        self.state = state;
    }

    /// Human-readable name (delegates to inner).
    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

/// A counting processor — tracks how many items have been processed.
///
/// The simplest stateful enrichment: `ς = N` (count).
pub struct Counter<P>
where
    P: Processor,
{
    inner: P,
    count: usize,
}

impl<P> Counter<P>
where
    P: Processor,
{
    /// Wrap a processor with a counter.
    pub fn new(inner: P) -> Self {
        Self { inner, count: 0 }
    }

    /// How many items have been processed.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Process input, incrementing count on success.
    ///
    /// # Errors
    /// Returns `ProcessorError` if the inner processor fails.
    /// Count is NOT incremented on failure.
    pub fn process(&mut self, input: P::Input) -> Result<P::Output, ProcessorError> {
        let output = self.inner.process(input)?;
        self.count += 1;
        Ok(output)
    }

    /// Reset counter to zero.
    pub fn reset(&mut self) {
        self.count = 0;
    }
}
