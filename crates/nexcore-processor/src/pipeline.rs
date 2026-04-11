//! Pipeline — σ (sequential composition).
//!
//! A pipeline chains processors in order, threading the output
//! of each into the input of the next. Optional boundaries
//! gate entry and exit. This is the σ in `∂(σ(μ))`.

use crate::ProcessorError;
use crate::boundary::Boundary;
use crate::processor::Processor;

/// A two-stage pipeline that threads output of `A` into input of `B`.
///
/// `Pipeline<A, B>` itself implements `Processor`, so pipelines
/// compose recursively: `Pipeline<Pipeline<A, B>, C>` chains three.
pub struct Pipeline<A, B>
where
    A: Processor,
    B: Processor<Input = A::Output>,
{
    first: A,
    second: B,
}

impl<A, B> Pipeline<A, B>
where
    A: Processor,
    B: Processor<Input = A::Output>,
{
    /// Chain two processors into a pipeline.
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }

    /// Extend this pipeline with a third stage.
    pub fn then<C>(self, next: C) -> Pipeline<Self, C>
    where
        C: Processor<Input = B::Output>,
    {
        Pipeline {
            first: self,
            second: next,
        }
    }
}

impl<A, B> Processor for Pipeline<A, B>
where
    A: Processor,
    B: Processor<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn process(&self, input: Self::Input) -> Result<Self::Output, ProcessorError> {
        let intermediate =
            self.first
                .process(input)
                .map_err(|e| ProcessorError::PipelineError {
                    stage: 0,
                    source: Box::new(e),
                })?;
        self.second
            .process(intermediate)
            .map_err(|e| ProcessorError::PipelineError {
                stage: 1,
                source: Box::new(e),
            })
    }

    fn name(&self) -> &str {
        "pipeline"
    }
}

/// A processor wrapped with entry and/or exit boundaries.
///
/// This is the full `∂(μ)` — boundary-guarded mapping.
pub struct Bounded<P, InB, OutB>
where
    P: Processor,
    InB: Boundary<P::Input>,
    OutB: Boundary<P::Output>,
{
    processor: P,
    entry: InB,
    exit: OutB,
}

impl<P, InB, OutB> Bounded<P, InB, OutB>
where
    P: Processor,
    InB: Boundary<P::Input>,
    OutB: Boundary<P::Output>,
{
    /// Wrap a processor with entry and exit boundaries.
    pub fn new(processor: P, entry: InB, exit: OutB) -> Self {
        Self {
            processor,
            entry,
            exit,
        }
    }
}

impl<P, InB, OutB> Processor for Bounded<P, InB, OutB>
where
    P: Processor,
    InB: Boundary<P::Input>,
    OutB: Boundary<P::Output>,
{
    type Input = P::Input;
    type Output = P::Output;

    fn process(&self, input: Self::Input) -> Result<Self::Output, ProcessorError> {
        // ∂ entry gate
        self.entry.check(&input)?;
        // μ mapping
        let output = self.processor.process(input)?;
        // ∂ exit gate
        self.exit.check(&output)?;
        Ok(output)
    }

    fn name(&self) -> &str {
        self.processor.name()
    }
}

/// A boundary that accepts everything — used when only one
/// side (entry or exit) needs guarding.
pub struct OpenBoundary;

impl<T> Boundary<T> for OpenBoundary {
    fn check(&self, _value: &T) -> Result<(), ProcessorError> {
        Ok(())
    }

    fn description(&self) -> &str {
        "open (accepts all)"
    }
}
