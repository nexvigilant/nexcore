//! # Chaos Engineering Test Infrastructure
//!
//! CTVP Phase 1: Fault injection testing for graceful degradation verification.
//!
//! ## Test Categories
//!
//! - **mcp_failure** - MCP server unreachability scenarios
//! - **brain_corruption** - Recovery from corrupt artifact files
//! - **timeout_handling** - Skill validation timeout scenarios
//! - **hook_cascade** - Hook failure containment
//!
//! ## Usage
//!
//! ```bash
//! cargo test --features chaos-tests -p nexcore-vigilance
//! ```
//!
//! ## Design Principles
//!
//! 1. **Simulate failure conditions** - Create realistic fault scenarios
//! 2. **Verify graceful degradation** - No panics, proper error propagation
//! 3. **Verify recovery** - System returns to normal when fault clears
//! 4. **Property-based testing** - Use proptest for edge case discovery

pub mod brain_corruption;
pub mod hook_cascade;
pub mod mcp_failure;
pub mod timeout_handling;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Fault injection controller for chaos tests
///
/// Thread-safe controller that can inject and clear faults during test execution.
#[derive(Debug, Clone)]
pub struct FaultInjector {
    /// Whether the fault is currently active
    fault_active: Arc<AtomicBool>,
    /// Description of the fault
    fault_description: Arc<String>,
}

impl FaultInjector {
    /// Create a new fault injector
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            fault_active: Arc::new(AtomicBool::new(false)),
            fault_description: Arc::new(description.into()),
        }
    }

    /// Inject the fault (make it active)
    pub fn inject(&self) {
        self.fault_active.store(true, Ordering::SeqCst);
    }

    /// Clear the fault (deactivate it)
    pub fn clear(&self) {
        self.fault_active.store(false, Ordering::SeqCst);
    }

    /// Check if the fault is currently active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.fault_active.load(Ordering::SeqCst)
    }

    /// Get the fault description
    #[must_use]
    pub fn description(&self) -> &str {
        &self.fault_description
    }
}

/// Result of a chaos test
#[derive(Debug, Clone)]
pub struct ChaosTestResult {
    /// Test name
    pub test_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Details about degradation behavior
    pub degradation_details: Vec<String>,
    /// Errors that were properly propagated
    pub propagated_errors: Vec<String>,
    /// Recovery details after fault cleared
    pub recovery_details: Vec<String>,
}

impl ChaosTestResult {
    /// Create a new chaos test result
    #[must_use]
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            passed: true,
            degradation_details: Vec::new(),
            propagated_errors: Vec::new(),
            recovery_details: Vec::new(),
        }
    }

    /// Mark the test as failed
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.passed = false;
        self.degradation_details
            .push(format!("FAIL: {}", reason.into()));
    }

    /// Add a degradation detail
    pub fn add_degradation(&mut self, detail: impl Into<String>) {
        self.degradation_details.push(detail.into());
    }

    /// Add a propagated error
    pub fn add_propagated_error(&mut self, error: impl Into<String>) {
        self.propagated_errors.push(error.into());
    }

    /// Add a recovery detail
    pub fn add_recovery(&mut self, detail: impl Into<String>) {
        self.recovery_details.push(detail.into());
    }
}

/// Trait for types that support fault injection
pub trait FaultInjectable {
    /// Type of error produced when fault is active
    type Error;

    /// Check if should fail due to injected fault
    fn should_fail(&self, injector: &FaultInjector) -> Option<Self::Error>;
}

/// Macro to create a chaos test that verifies no panics occur
#[macro_export]
macro_rules! chaos_no_panic {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let result = std::panic::catch_unwind(|| $body);
            assert!(
                result.is_ok(),
                "Chaos test {} panicked when it should have gracefully degraded",
                stringify!($name)
            );
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_injector_lifecycle() {
        let injector = FaultInjector::new("test fault");

        // Initially inactive
        assert!(!injector.is_active());
        assert_eq!(injector.description(), "test fault");

        // Inject fault
        injector.inject();
        assert!(injector.is_active());

        // Clear fault
        injector.clear();
        assert!(!injector.is_active());
    }

    #[test]
    fn test_fault_injector_clone_shares_state() {
        let injector = FaultInjector::new("shared fault");
        let clone = injector.clone();

        injector.inject();
        assert!(clone.is_active());

        clone.clear();
        assert!(!injector.is_active());
    }

    #[test]
    fn test_chaos_test_result() {
        let mut result = ChaosTestResult::new("test_example");
        assert!(result.passed);

        result.add_degradation("handled timeout gracefully");
        result.add_propagated_error("TimeoutError after 5s");
        result.add_recovery("system resumed after fault cleared");

        assert!(result.passed);
        assert_eq!(result.degradation_details.len(), 1);
        assert_eq!(result.propagated_errors.len(), 1);
        assert_eq!(result.recovery_details.len(), 1);

        result.fail("unexpected behavior");
        assert!(!result.passed);
    }
}
