//! Check result types.

use std::time::Duration;

/// Outcome of a single verification check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    Passed {
        message: String,
    },
    Failed {
        message: String,
        suggestion: Option<String>,
    },
    Skipped {
        reason: String,
    },
}

impl CheckResult {
    pub fn passed(message: impl Into<String>) -> Self {
        Self::Passed {
            message: message.into(),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed {
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn failed_with_suggestion(
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::Failed {
            message: message.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    pub fn skipped(reason: impl Into<String>) -> Self {
        Self::Skipped {
            reason: reason.into(),
        }
    }

    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed { .. })
    }
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
    pub fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped { .. })
    }
}

/// Named result from running a check
#[derive(Debug, Clone)]
pub struct CheckOutcome {
    pub name: String,
    pub result: CheckResult,
    pub duration: Duration,
}

impl CheckOutcome {
    pub fn new(name: impl Into<String>, result: CheckResult, duration: Duration) -> Self {
        Self {
            name: name.into(),
            result,
            duration,
        }
    }
}
