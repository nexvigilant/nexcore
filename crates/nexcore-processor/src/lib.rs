//! # nexcore-processor
//!
//! Generic processor framework derived from T1 primitive decomposition:
//!
//! ```text
//! processor = ∂(σ(μ)) + {ς, →, ∝}
//!
//! Kernel (irreducible):
//!   μ — Mapping (processor.rs)     : input → output transformation
//!   σ — Sequence (pipeline.rs)     : ordered composition of processors
//!   ∂ — Boundary (boundary.rs)     : acceptance/rejection criteria
//!
//! Enrichments (subtype-dependent):
//!   ς — State (state.rs)           : mutable accumulation across invocations
//!   → — Causality                  : implicit in Processor::process return
//!   ∝ — Irreversibility            : implicit in consuming input by value
//! ```
//!
//! ## Processor Subtypes
//!
//! | Subtype | Composition | Example |
//! |---------|-------------|---------|
//! | Pure function | μ+σ+∂ | `FnProcessor` |
//! | Stateful | μ+σ+∂+ς | `Accumulator`, `Counter` |
//! | Bounded | ∂(μ) | `Bounded` with entry/exit gates |
//! | Pipeline | σ(μ₁, μ₂, ...) | `Pipeline::new(a, b).then(c)` |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_processor::{FnProcessor, Processor, ProcessorError};
//!
//! let double = FnProcessor::new("double", |x: i64| Ok(x * 2));
//! assert_eq!(double.process(21).unwrap(), 42);
//! ```

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

pub mod batch;
pub mod boundary;
pub mod error;
pub mod pipeline;
pub mod processor;
pub mod state;

// Re-exports for ergonomic use
pub use batch::{BatchResult, process_batch, process_batch_strict};
pub use boundary::{AndBoundary, Boundary, OrBoundary, PredicateBoundary};
pub use error::ProcessorError;
pub use pipeline::{Bounded, OpenBoundary, Pipeline};
pub use processor::{FnProcessor, Processor};
pub use state::{Accumulator, Counter};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_fn_processor() {
        let double = FnProcessor::new("double", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x * 2)
        });
        let result = double.process(21);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn pipeline_chains_two_processors() {
        let double = FnProcessor::new("double", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x * 2)
        });
        let add_one = FnProcessor::new("add_one", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x + 1)
        });

        let pipe = Pipeline::new(double, add_one);
        // σ(μ₁, μ₂): 21 → 42 → 43
        let result = pipe.process(21);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(43));
    }

    #[test]
    fn pipeline_three_stages() {
        let a = FnProcessor::new("a", |x: i64| -> Result<i64, ProcessorError> { Ok(x + 1) });
        let b = FnProcessor::new("b", |x: i64| -> Result<i64, ProcessorError> { Ok(x * 3) });
        let c = FnProcessor::new("c", |x: i64| -> Result<i64, ProcessorError> { Ok(x - 5) });

        let pipe = Pipeline::new(a, b).then(c);
        // 10 → 11 → 33 → 28
        let result = pipe.process(10);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(28));
    }

    #[test]
    fn boundary_accepts_valid() {
        let positive = PredicateBoundary::new("must be positive", |x: &i64| *x > 0);
        assert!(positive.check(&42).is_ok());
    }

    #[test]
    fn boundary_rejects_invalid() {
        let positive = PredicateBoundary::new("must be positive", |x: &i64| *x > 0);
        let result = positive.check(&-1);
        assert!(result.is_err());
    }

    #[test]
    fn bounded_processor_gates_entry() {
        let double = FnProcessor::new("double", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x * 2)
        });
        let positive_entry = PredicateBoundary::new("positive input", |x: &i64| *x > 0);

        let guarded = Bounded::new(double, positive_entry, OpenBoundary);

        // Valid input passes through
        let result = guarded.process(5);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(10));

        // Invalid input rejected at ∂
        assert!(guarded.process(-1).is_err());
    }

    #[test]
    fn bounded_processor_gates_exit() {
        let negate = FnProcessor::new("negate", |x: i64| -> Result<i64, ProcessorError> { Ok(-x) });
        let positive_exit = PredicateBoundary::new("positive output", |x: &i64| *x > 0);

        let guarded = Bounded::new(negate, OpenBoundary, positive_exit);

        // negate(5) = -5, fails exit boundary
        assert!(guarded.process(5).is_err());

        // negate(-3) = 3, passes exit boundary
        let result = guarded.process(-3);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(3));
    }

    #[test]
    fn and_boundary_requires_both() {
        let positive = PredicateBoundary::new("positive", |x: &i64| *x > 0);
        let small = PredicateBoundary::new("< 100", |x: &i64| *x < 100);
        let both = AndBoundary::new(positive, small);

        assert!(both.check(&50).is_ok());
        assert!(both.check(&-1).is_err());
        assert!(both.check(&200).is_err());
    }

    #[test]
    fn or_boundary_requires_either() {
        let zero = PredicateBoundary::new("is zero", |x: &i64| *x == 0);
        let big = PredicateBoundary::new("> 100", |x: &i64| *x > 100);
        let either = OrBoundary::new(zero, big);

        assert!(either.check(&0).is_ok());
        assert!(either.check(&200).is_ok());
        assert!(either.check(&50).is_err());
    }

    #[test]
    fn counter_tracks_invocations() {
        let double = FnProcessor::new("double", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x * 2)
        });
        let mut counted = Counter::new(double);

        assert_eq!(counted.count(), 0);
        let _ = counted.process(1);
        let _ = counted.process(2);
        let _ = counted.process(3);
        assert_eq!(counted.count(), 3);
    }

    #[test]
    fn accumulator_maintains_state() {
        let identity = FnProcessor::new("id", |x: i64| -> Result<i64, ProcessorError> { Ok(x) });

        // ς: running sum of all outputs
        let mut summer = Accumulator::new(identity, 0i64, |state, output| state + output);

        let _ = summer.process(10);
        let _ = summer.process(20);
        let _ = summer.process(30);

        assert_eq!(*summer.state(), 60);
    }

    #[test]
    fn error_propagates_through_pipeline_with_stage_info() {
        let fail = FnProcessor::new("fail", |_x: i64| -> Result<i64, ProcessorError> {
            Err(ProcessorError::TransformError {
                processor: "fail".into(),
                reason: "always fails".into(),
            })
        });
        let unreachable =
            FnProcessor::new("unreachable", |x: i64| -> Result<i64, ProcessorError> {
                Ok(x)
            });

        let pipe = Pipeline::new(fail, unreachable);
        let result = pipe.process(1);
        assert!(result.is_err());

        // Verify stage 0 is reported (first processor failed)
        assert!(matches!(
            result,
            Err(ProcessorError::PipelineError { stage: 0, .. })
        ));
    }

    #[test]
    fn pipeline_error_reports_second_stage() {
        let pass = FnProcessor::new("pass", |x: i64| -> Result<i64, ProcessorError> { Ok(x) });
        let fail = FnProcessor::new("fail", |_x: i64| -> Result<i64, ProcessorError> {
            Err(ProcessorError::TransformError {
                processor: "fail".into(),
                reason: "stage 2 dies".into(),
            })
        });

        let pipe = Pipeline::new(pass, fail);
        let result = pipe.process(1);
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ProcessorError::PipelineError { stage: 1, .. })
        ));
    }

    #[test]
    fn processor_error_display() {
        let err = ProcessorError::BoundaryRejection {
            boundary: "positive".into(),
            reason: "value was -1".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("positive"));
        assert!(msg.contains("-1"));
    }

    // --- Batch tests ---

    #[test]
    fn batch_all_succeed() {
        let double = FnProcessor::new("double", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x * 2)
        });
        let result = process_batch(&double, vec![1, 2, 3, 4, 5]);

        assert!(result.all_ok());
        assert_eq!(result.success_count(), 5);
        assert_eq!(result.failure_count(), 0);
        assert!((result.success_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn batch_partial_failure() {
        // Reject negative inputs via the mapping itself
        let positive_only =
            FnProcessor::new("positive_only", |x: i64| -> Result<i64, ProcessorError> {
                if x > 0 {
                    Ok(x)
                } else {
                    Err(ProcessorError::TransformError {
                        processor: "positive_only".into(),
                        reason: format!("{x} is not positive"),
                    })
                }
            });

        let result = process_batch(&positive_only, vec![1, -2, 3, -4, 5]);

        assert!(!result.all_ok());
        assert!(!result.all_err());
        assert_eq!(result.success_count(), 3);
        assert_eq!(result.failure_count(), 2);
        // Failures at indices 1 and 3
        assert_eq!(result.failures[0].0, 1);
        assert_eq!(result.failures[1].0, 3);
    }

    #[test]
    fn batch_strict_stops_at_first_failure() {
        let positive_only =
            FnProcessor::new("positive_only", |x: i64| -> Result<i64, ProcessorError> {
                if x > 0 {
                    Ok(x)
                } else {
                    Err(ProcessorError::TransformError {
                        processor: "positive_only".into(),
                        reason: format!("{x} is not positive"),
                    })
                }
            });

        let result = process_batch_strict(&positive_only, vec![1, 2, -3, 4, 5]);
        assert!(result.is_err());

        if let Err((idx, _)) = result {
            assert_eq!(idx, 2); // Failed at index 2
        }
    }

    #[test]
    fn batch_empty_input() {
        let double = FnProcessor::new("double", |x: i64| -> Result<i64, ProcessorError> {
            Ok(x * 2)
        });
        let result = process_batch(&double, vec![]);

        assert!(result.all_ok()); // vacuously true
        assert_eq!(result.total, 0);
        assert!((result.success_rate() - 0.0).abs() < f64::EPSILON);
    }
}
