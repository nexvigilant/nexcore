//! # GroundsTo implementations for nexcore-skill-exec types
//!
//! Skill execution engine types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `ExecutionRequest` -- Mapping (mu) dominant as it maps params to execution.
//! - `ExecutionResult` -- State (varsigma) dominant as it captures execution outcome.
//! - `ExecutionStatus` -- Comparison (kappa) dominant as it classifies outcomes.
//! - `ExecutionMethod` -- Comparison (kappa) dominant as it discriminates exec types.
//! - `SkillInfo` -- State (varsigma) dominant as it captures skill metadata.
//! - `CompositeExecutor`, `ShellExecutor` -- Mapping (mu) dominant as executors.
//! - `ParameterValidator` -- Boundary (partial) dominant as validation boundary.
//! - `ExecutionError` -- Boundary (partial) dominant as error boundary.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::models::ExecutionMethod;
use crate::{
    CompositeExecutor, ExecutionError, ExecutionRequest, ExecutionResult, ExecutionStatus,
    ParameterValidator, ShellExecutor, SkillInfo,
};

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

/// ExecutionRequest: T2-C (mu + varsigma + lambda + N), dominant mu
///
/// Request to execute a skill with parameters, timeout, env, and working dir.
/// Mapping-dominant as it maps parameters to an execution invocation.
/// State is secondary (encapsulated request state).
/// Location is tertiary (working_dir). Quantity is quaternary (timeout duration).
impl GroundsTo for ExecutionRequest {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- params -> execution
            LexPrimitiva::State,    // varsigma -- request state
            LexPrimitiva::Location, // lambda -- working_dir
            LexPrimitiva::Quantity, // N -- timeout duration
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// ExecutionResult: T3 (varsigma + mu + N + lambda + kappa + exists), dominant varsigma
///
/// Complete execution result with status, output, artifacts, timing, and captured IO.
/// State-dominant as it captures the full outcome state of an execution.
impl GroundsTo for ExecutionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- execution outcome state
            LexPrimitiva::Mapping,    // mu -- execution -> result
            LexPrimitiva::Quantity,   // N -- duration_ms, exit_code
            LexPrimitiva::Location,   // lambda -- artifact paths
            LexPrimitiva::Comparison, // kappa -- status classification
            LexPrimitiva::Existence,  // exists -- optional error, exit_code
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// ExecutionStatus: T1-Universal (kappa), dominant kappa
///
/// Completed, Failed, Timeout, Cancelled. Pure comparison discriminant.
impl GroundsTo for ExecutionStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- status classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// ExecutionMethod: T2-P (kappa + lambda), dominant kappa
///
/// Shell, Binary, or Library execution method. Comparison-dominant as it
/// discriminates between execution types. Location is secondary (path to executable).
impl GroundsTo for ExecutionMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- method type discrimination
            LexPrimitiva::Location,   // lambda -- executable path
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// SkillInfo: T2-C (varsigma + lambda + sigma + exists), dominant varsigma
///
/// Skill metadata for execution: name, path, methods, schemas.
/// State-dominant as it captures the discoverable skill state.
impl GroundsTo for SkillInfo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- skill metadata state
            LexPrimitiva::Location,  // lambda -- path
            LexPrimitiva::Sequence,  // sigma -- ordered execution_methods
            LexPrimitiva::Existence, // exists -- optional schemas
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Executor types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// CompositeExecutor: T2-P (mu + kappa + lambda), dominant mu
///
/// Delegates to appropriate sub-executors (binary > shell).
/// Mapping-dominant as it maps skill name -> execution.
impl GroundsTo for CompositeExecutor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- name -> execution
            LexPrimitiva::Comparison, // kappa -- executor selection
            LexPrimitiva::Location,   // lambda -- skills_dir
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// ShellExecutor: T2-P (mu + sigma + lambda), dominant mu
///
/// Executes skills via shell scripts or binaries.
/// Mapping-dominant as it maps skill + request -> result.
impl GroundsTo for ShellExecutor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- skill + request -> result
            LexPrimitiva::Sequence, // sigma -- stdin/stdout pipeline
            LexPrimitiva::Location, // lambda -- script path discovery
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Validator and error types
// ---------------------------------------------------------------------------

/// ParameterValidator: T2-P (partial + kappa + exists), dominant partial
///
/// JSON Schema validation for skill parameters.
/// Boundary-dominant as it enforces parameter validity boundaries.
impl GroundsTo for ParameterValidator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- validation boundary
            LexPrimitiva::Comparison, // kappa -- schema type checking
            LexPrimitiva::Existence,  // exists -- optional schema
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ExecutionError: T2-C (partial + kappa + Sigma + lambda), dominant partial
///
/// Error boundary for skill execution with path and process-related variants.
impl GroundsTo for ExecutionError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- variant discrimination
            LexPrimitiva::Sum,        // Sigma -- aggregated error sources
            LexPrimitiva::Location,   // lambda -- path-related errors
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn execution_request_is_t2c() {
        assert_eq!(ExecutionRequest::tier(), Tier::T2Composite);
    }

    #[test]
    fn execution_result_is_t3() {
        assert_eq!(ExecutionResult::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn execution_status_is_t1() {
        assert_eq!(ExecutionStatus::tier(), Tier::T1Universal);
    }

    #[test]
    fn execution_method_is_t2p() {
        assert_eq!(ExecutionMethod::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_info_is_t2c() {
        assert_eq!(SkillInfo::tier(), Tier::T2Composite);
    }

    #[test]
    fn composite_executor_is_t2p() {
        assert_eq!(CompositeExecutor::tier(), Tier::T2Primitive);
    }

    #[test]
    fn shell_executor_is_t2p() {
        assert_eq!(ShellExecutor::tier(), Tier::T2Primitive);
    }

    #[test]
    fn parameter_validator_is_t2p() {
        assert_eq!(ParameterValidator::tier(), Tier::T2Primitive);
    }

    #[test]
    fn execution_error_is_t2c() {
        assert_eq!(ExecutionError::tier(), Tier::T2Composite);
    }

    #[test]
    fn execution_request_dominant_is_mapping() {
        let comp = ExecutionRequest::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn execution_result_dominant_is_state() {
        let comp = ExecutionResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn parameter_validator_dominant_is_boundary() {
        let comp = ParameterValidator::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }
}
