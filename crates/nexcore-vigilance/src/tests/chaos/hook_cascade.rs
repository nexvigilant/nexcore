//! # Hook Cascade Failure Tests
//!
//! CTVP Phase 1: Test hook failure containment.
//!
//! ## Test Scenarios
//!
//! 1. Single hook failure - Verify other hooks still execute
//! 2. Cascading failures - Verify failure isolation
//! 3. Hook timeout - Verify timeout doesn't block pipeline
//! 4. Hook panic containment - Verify panic is caught and reported

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::{ChaosTestResult, FaultInjector};

/// Hook execution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookResult {
    /// Hook executed successfully
    Allow,
    /// Hook issued a warning
    Warn(String),
    /// Hook blocked the operation
    Block(String),
    /// Hook failed with error
    Error(String),
    /// Hook timed out
    Timeout(Duration),
}

impl HookResult {
    /// Check if the result allows continuation
    #[must_use]
    pub fn allows_continuation(&self) -> bool {
        matches!(self, HookResult::Allow | HookResult::Warn(_))
    }

    /// Check if the result is a failure
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            HookResult::Block(_) | HookResult::Error(_) | HookResult::Timeout(_)
        )
    }
}

/// Simulated hook for testing
pub struct SimulatedHook {
    /// Hook name
    pub name: String,
    /// Configured result
    result: HookResult,
    /// Execution count
    execution_count: Arc<AtomicU64>,
    /// Fault injector
    fault_injector: Option<FaultInjector>,
}

impl SimulatedHook {
    /// Create a new simulated hook
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            result: HookResult::Allow,
            execution_count: Arc::new(AtomicU64::new(0)),
            fault_injector: None,
        }
    }

    /// Set the hook result
    #[must_use]
    pub fn with_result(mut self, result: HookResult) -> Self {
        self.result = result;
        self
    }

    /// Attach a fault injector
    #[must_use]
    pub fn with_fault_injector(mut self, injector: FaultInjector) -> Self {
        self.fault_injector = Some(injector);
        self
    }

    /// Get execution count
    #[must_use]
    pub fn executions(&self) -> u64 {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// Execute the hook
    pub fn execute(&self) -> HookResult {
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        // Check for injected fault
        if self
            .fault_injector
            .as_ref()
            .map_or(false, FaultInjector::is_active)
        {
            return HookResult::Error(format!("Injected fault in hook '{}'", self.name));
        }

        self.result.clone()
    }
}

/// Hook pipeline that executes multiple hooks in sequence
pub struct HookPipeline {
    /// Hooks in the pipeline
    hooks: Vec<SimulatedHook>,
    /// Whether to stop on first failure
    stop_on_failure: bool,
    /// Whether to contain panics
    contain_panics: bool,
}

impl HookPipeline {
    /// Create a new pipeline
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            stop_on_failure: true,
            contain_panics: true,
        }
    }

    /// Add a hook to the pipeline
    pub fn add_hook(&mut self, hook: SimulatedHook) {
        self.hooks.push(hook);
    }

    /// Configure to continue on failure
    #[must_use]
    pub fn continue_on_failure(mut self) -> Self {
        self.stop_on_failure = false;
        self
    }

    /// Execute all hooks and collect results
    pub fn execute(&self) -> PipelineResult {
        let mut results = Vec::new();
        let mut stopped_early = false;

        for hook in &self.hooks {
            let result = if self.contain_panics {
                // Catch panics
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hook.execute())) {
                    Ok(r) => r,
                    Err(_) => HookResult::Error(format!("Hook '{}' panicked", hook.name)),
                }
            } else {
                hook.execute()
            };

            let is_failure = result.is_failure();
            results.push((hook.name.clone(), result));

            if is_failure && self.stop_on_failure {
                stopped_early = true;
                break;
            }
        }

        PipelineResult {
            results,
            stopped_early,
            total_hooks: self.hooks.len(),
        }
    }
}

impl Default for HookPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of pipeline execution
#[derive(Debug)]
pub struct PipelineResult {
    /// Individual hook results
    pub results: Vec<(String, HookResult)>,
    /// Whether execution stopped early
    pub stopped_early: bool,
    /// Total number of hooks in pipeline
    pub total_hooks: usize,
}

impl PipelineResult {
    /// Check if all hooks passed
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|(_, r)| r.allows_continuation())
    }

    /// Get number of executed hooks
    #[must_use]
    pub fn executed_count(&self) -> usize {
        self.results.len()
    }

    /// Get number of failed hooks
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|(_, r)| r.is_failure()).count()
    }

    /// Get names of failed hooks
    #[must_use]
    pub fn failed_hooks(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|(_, r)| r.is_failure())
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ========== Basic Hook Tests ==========

    #[test]
    fn test_hook_allow_result() {
        let hook = SimulatedHook::new("test_hook");
        let result = hook.execute();

        assert_eq!(result, HookResult::Allow);
        assert!(result.allows_continuation());
        assert!(!result.is_failure());
        assert_eq!(hook.executions(), 1);
    }

    #[test]
    fn test_hook_block_result() {
        let hook =
            SimulatedHook::new("blocker").with_result(HookResult::Block("policy violation".into()));
        let result = hook.execute();

        assert!(!result.allows_continuation());
        assert!(result.is_failure());
    }

    #[test]
    fn test_hook_warn_result() {
        let hook = SimulatedHook::new("warner").with_result(HookResult::Warn("be careful".into()));
        let result = hook.execute();

        assert!(result.allows_continuation());
        assert!(!result.is_failure());
    }

    // ========== Pipeline Tests ==========

    #[test]
    fn test_pipeline_all_pass() {
        let mut pipeline = HookPipeline::new();
        pipeline.add_hook(SimulatedHook::new("hook1"));
        pipeline.add_hook(SimulatedHook::new("hook2"));
        pipeline.add_hook(SimulatedHook::new("hook3"));

        let result = pipeline.execute();

        assert!(result.all_passed());
        assert_eq!(result.executed_count(), 3);
        assert_eq!(result.failed_count(), 0);
        assert!(!result.stopped_early);
    }

    #[test]
    fn test_pipeline_stops_on_failure() {
        let mut pipeline = HookPipeline::new();
        pipeline.add_hook(SimulatedHook::new("hook1"));
        pipeline
            .add_hook(SimulatedHook::new("hook2").with_result(HookResult::Block("blocked".into())));
        pipeline.add_hook(SimulatedHook::new("hook3")); // Should not execute

        let result = pipeline.execute();

        assert!(!result.all_passed());
        assert_eq!(result.executed_count(), 2);
        assert!(result.stopped_early);
    }

    #[test]
    fn test_pipeline_continues_on_failure() {
        let mut pipeline = HookPipeline::new().continue_on_failure();
        pipeline.add_hook(SimulatedHook::new("hook1"));
        pipeline
            .add_hook(SimulatedHook::new("hook2").with_result(HookResult::Block("blocked".into())));
        pipeline.add_hook(SimulatedHook::new("hook3"));

        let result = pipeline.execute();

        assert!(!result.all_passed());
        assert_eq!(result.executed_count(), 3); // All executed
        assert_eq!(result.failed_count(), 1);
        assert!(!result.stopped_early);
    }

    // ========== Fault Injector Integration ==========

    #[test]
    fn test_fault_injector_hook_scenario() {
        let injector = FaultInjector::new("hook failure");
        let mut result = ChaosTestResult::new("hook_cascade_containment");

        let mut pipeline = HookPipeline::new().continue_on_failure();
        pipeline.add_hook(SimulatedHook::new("hook1"));
        pipeline.add_hook(SimulatedHook::new("hook2").with_fault_injector(injector.clone()));
        pipeline.add_hook(SimulatedHook::new("hook3"));

        // Phase 1: Normal operation
        let exec_result = pipeline.execute();
        assert!(exec_result.all_passed());
        result.add_degradation("Normal pipeline execution succeeded");

        // Phase 2: Inject fault
        injector.inject();

        // Phase 3: Execute with fault
        let exec_result = pipeline.execute();
        assert!(!exec_result.all_passed());
        assert_eq!(exec_result.failed_count(), 1);
        assert_eq!(exec_result.failed_hooks(), vec!["hook2"]);
        result.add_propagated_error("hook2 failed due to injected fault");

        // Phase 4: Verify other hooks still executed
        assert_eq!(exec_result.executed_count(), 3);
        result.add_degradation("Other hooks executed despite failure");

        // Phase 5: Clear fault
        injector.clear();

        // Phase 6: Verify recovery
        let exec_result = pipeline.execute();
        assert!(exec_result.all_passed());
        result.add_recovery("Normal operation restored");

        assert!(result.passed);
    }

    // ========== Cascade Containment Tests ==========

    #[test]
    fn test_multiple_failures_contained() {
        let injector1 = FaultInjector::new("fault1");
        let injector2 = FaultInjector::new("fault2");
        injector1.inject();
        injector2.inject();

        let mut pipeline = HookPipeline::new().continue_on_failure();
        pipeline.add_hook(SimulatedHook::new("hook1").with_fault_injector(injector1));
        pipeline.add_hook(SimulatedHook::new("hook2").with_fault_injector(injector2));
        pipeline.add_hook(SimulatedHook::new("hook3"));

        let result = pipeline.execute();

        // Both failures should be contained
        assert_eq!(result.failed_count(), 2);
        assert_eq!(result.executed_count(), 3);
        assert_eq!(result.failed_hooks(), vec!["hook1", "hook2"]);
    }

    #[test]
    fn test_timeout_result_is_failure() {
        let hook = SimulatedHook::new("slow_hook")
            .with_result(HookResult::Timeout(Duration::from_secs(30)));
        let result = hook.execute();

        assert!(result.is_failure());
        assert!(!result.allows_continuation());
    }

    // ========== No Panic Tests ==========

    #[test]
    fn test_no_panic_on_empty_pipeline() {
        let result = std::panic::catch_unwind(|| {
            let pipeline = HookPipeline::new();
            pipeline.execute()
        });
        assert!(result.is_ok(), "Empty pipeline should not panic");
    }

    #[test]
    fn test_pipeline_result_methods_never_panic() {
        let result = std::panic::catch_unwind(|| {
            let pipeline_result = PipelineResult {
                results: Vec::new(),
                stopped_early: false,
                total_hooks: 0,
            };

            let _ = pipeline_result.all_passed();
            let _ = pipeline_result.executed_count();
            let _ = pipeline_result.failed_count();
            let _ = pipeline_result.failed_hooks();
        });
        assert!(result.is_ok());
    }

    // ========== Property-Based Tests ==========

    proptest! {
        #[test]
        fn prop_hook_result_consistency(
            _name in "[a-zA-Z_][a-zA-Z0-9_]*",
            msg in ".*",
        ) {
            // Allow result should always allow continuation
            let allow = HookResult::Allow;
            prop_assert!(allow.allows_continuation());
            prop_assert!(!allow.is_failure());

            // Warn should allow continuation but not be failure
            let warn = HookResult::Warn(msg.clone());
            prop_assert!(warn.allows_continuation());
            prop_assert!(!warn.is_failure());

            // Block should not allow continuation and be failure
            let block = HookResult::Block(msg.clone());
            prop_assert!(!block.allows_continuation());
            prop_assert!(block.is_failure());

            // Error should not allow continuation and be failure
            let error = HookResult::Error(msg);
            prop_assert!(!error.allows_continuation());
            prop_assert!(error.is_failure());
        }

        #[test]
        fn prop_pipeline_executed_count_bounded(hook_count in 0usize..20) {
            let mut pipeline = HookPipeline::new().continue_on_failure();

            for i in 0..hook_count {
                pipeline.add_hook(SimulatedHook::new(format!("hook_{}", i)));
            }

            let result = pipeline.execute();

            // Executed count should equal hook count when continue_on_failure
            prop_assert_eq!(result.executed_count(), hook_count);
            prop_assert_eq!(result.total_hooks, hook_count);
        }

        #[test]
        fn prop_failed_hooks_subset_of_executed(
            pass_count in 0usize..10,
            fail_count in 0usize..10,
        ) {
            let mut pipeline = HookPipeline::new().continue_on_failure();

            for i in 0..pass_count {
                pipeline.add_hook(SimulatedHook::new(format!("pass_{}", i)));
            }
            for i in 0..fail_count {
                pipeline.add_hook(
                    SimulatedHook::new(format!("fail_{}", i))
                        .with_result(HookResult::Block("test".into()))
                );
            }

            let result = pipeline.execute();

            prop_assert!(result.failed_count() <= result.executed_count());
            prop_assert_eq!(result.failed_count(), fail_count);
        }
    }
}
