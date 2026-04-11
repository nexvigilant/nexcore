//! Batch processing — apply a processor to a collection of inputs.
//!
//! `BatchResult` collects successes and failures separately,
//! allowing partial success without losing error context.

use crate::ProcessorError;
use crate::processor::Processor;

/// Result of processing a batch of inputs.
///
/// Unlike `Result<Vec<O>, E>` which fails on the first error,
/// `BatchResult` processes ALL inputs and separates successes
/// from failures with their original indices.
#[derive(Debug)]
pub struct BatchResult<O> {
    /// Successfully processed outputs with their original indices.
    pub successes: Vec<(usize, O)>,
    /// Failed inputs with their original indices and errors.
    pub failures: Vec<(usize, ProcessorError)>,
    /// Total number of inputs processed.
    pub total: usize,
}

impl<O> BatchResult<O> {
    /// True if all inputs succeeded.
    pub fn all_ok(&self) -> bool {
        self.failures.is_empty()
    }

    /// True if all inputs failed.
    pub fn all_err(&self) -> bool {
        self.successes.is_empty()
    }

    /// Number of successful outputs.
    pub fn success_count(&self) -> usize {
        self.successes.len()
    }

    /// Number of failures.
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Success rate as a fraction (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.successes.len() as f64 / self.total as f64
    }
}

/// Process a batch of inputs through a processor, collecting all results.
///
/// Every input is processed regardless of failures — partial success
/// is a valid outcome. Indices in `BatchResult` correspond to
/// positions in the input vector.
pub fn process_batch<P>(processor: &P, inputs: Vec<P::Input>) -> BatchResult<P::Output>
where
    P: Processor,
{
    let total = inputs.len();
    let mut successes = Vec::with_capacity(total);
    let mut failures = Vec::new();

    for (i, input) in inputs.into_iter().enumerate() {
        match processor.process(input) {
            Ok(output) => successes.push((i, output)),
            Err(e) => failures.push((i, e)),
        }
    }

    BatchResult {
        successes,
        failures,
        total,
    }
}

/// Process a batch, stopping at the first failure.
///
/// Returns all outputs collected before the failure, plus the error.
/// Use when partial results are not useful and you want fail-fast behavior.
pub fn process_batch_strict<P>(
    processor: &P,
    inputs: Vec<P::Input>,
) -> Result<Vec<P::Output>, (usize, ProcessorError)>
where
    P: Processor,
{
    let mut outputs = Vec::with_capacity(inputs.len());

    for (i, input) in inputs.into_iter().enumerate() {
        match processor.process(input) {
            Ok(output) => outputs.push(output),
            Err(e) => return Err((i, e)),
        }
    }

    Ok(outputs)
}
