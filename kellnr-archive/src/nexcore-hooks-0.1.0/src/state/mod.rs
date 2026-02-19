//! Session state management for cognitive integrity hooks.

pub mod problems;
mod session;

pub use problems::{
    DetectedProblem, Problem, ProblemCategory, ProblemRegistry, ProblemStatus, Severity,
};
pub use session::{Assumption, SessionState, now, state_file, verified_dir};
