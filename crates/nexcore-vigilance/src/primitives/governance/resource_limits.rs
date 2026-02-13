//! # Resource Limits (Bill of Rights)
//!
//! Implementation of Amendment VIII: Excessive compute latencies shall not
//! be required, nor excessive memory fines imposed, nor cruel and unusual
//! panics inflicted.

use super::Verdict;
use serde::{Deserialize, Serialize};

/// T3: ResourceGuard — Protects against excessive resource consumption.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceGuard {
    /// Component being monitored
    pub component: String,
    /// Current compute latency in milliseconds
    pub compute_latency_ms: u64,
    /// Maximum acceptable latency
    pub max_latency_ms: u64,
    /// Current memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Maximum acceptable memory
    pub max_memory_bytes: u64,
    /// Whether a panic occurred
    pub panic_occurred: bool,
    /// Whether the panic was "cruel and unusual" (unrecoverable)
    pub panic_cruel: bool,
}

impl ResourceGuard {
    /// Check if resource usage is constitutional.
    pub fn is_constitutional(&self) -> bool {
        !self.has_excessive_latency() && !self.has_excessive_memory() && !self.has_cruel_panic()
    }

    /// Check for excessive compute latency.
    pub fn has_excessive_latency(&self) -> bool {
        self.compute_latency_ms > self.max_latency_ms
    }

    /// Check for excessive memory usage.
    pub fn has_excessive_memory(&self) -> bool {
        self.memory_usage_bytes > self.max_memory_bytes
    }

    /// Check for cruel and unusual panics.
    pub fn has_cruel_panic(&self) -> bool {
        self.panic_occurred && self.panic_cruel
    }

    /// Identify all violations.
    pub fn violations(&self) -> Vec<ResourceViolation> {
        let mut violations = Vec::new();
        if self.has_excessive_latency() {
            violations.push(ResourceViolation::ExcessiveLatency {
                actual_ms: self.compute_latency_ms,
                limit_ms: self.max_latency_ms,
            });
        }
        if self.has_excessive_memory() {
            violations.push(ResourceViolation::ExcessiveMemory {
                actual_bytes: self.memory_usage_bytes,
                limit_bytes: self.max_memory_bytes,
            });
        }
        if self.has_cruel_panic() {
            violations.push(ResourceViolation::CruelPanic);
        }
        violations
    }

    /// Render a verdict.
    pub fn verdict(&self) -> Verdict {
        if self.is_constitutional() {
            Verdict::Permitted
        } else if self.has_cruel_panic() {
            Verdict::Rejected
        } else {
            Verdict::Flagged
        }
    }
}

/// T3: Specific resource violations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceViolation {
    /// Compute latency exceeded the acceptable limit
    ExcessiveLatency { actual_ms: u64, limit_ms: u64 },
    /// Memory usage exceeded the acceptable limit
    ExcessiveMemory { actual_bytes: u64, limit_bytes: u64 },
    /// A cruel and unusual panic was inflicted (unrecoverable crash)
    CruelPanic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_limits_constitutional() {
        let guard = ResourceGuard {
            component: "signal_detector".to_string(),
            compute_latency_ms: 100,
            max_latency_ms: 1000,
            memory_usage_bytes: 1_000_000,
            max_memory_bytes: 10_000_000,
            panic_occurred: false,
            panic_cruel: false,
        };
        assert!(guard.is_constitutional());
        assert!(guard.violations().is_empty());
        assert_eq!(guard.verdict(), Verdict::Permitted);
    }

    #[test]
    fn excessive_latency() {
        let guard = ResourceGuard {
            component: "slow_query".to_string(),
            compute_latency_ms: 5000,
            max_latency_ms: 1000,
            memory_usage_bytes: 100,
            max_memory_bytes: 10_000_000,
            panic_occurred: false,
            panic_cruel: false,
        };
        assert!(!guard.is_constitutional());
        assert!(guard.has_excessive_latency());
        assert_eq!(guard.violations().len(), 1);
        assert_eq!(guard.verdict(), Verdict::Flagged);
    }

    #[test]
    fn excessive_memory() {
        let guard = ResourceGuard {
            component: "memory_hog".to_string(),
            compute_latency_ms: 50,
            max_latency_ms: 1000,
            memory_usage_bytes: 100_000_000,
            max_memory_bytes: 10_000_000,
            panic_occurred: false,
            panic_cruel: false,
        };
        assert!(!guard.is_constitutional());
        assert!(guard.has_excessive_memory());
    }

    #[test]
    fn cruel_panic_rejected() {
        let guard = ResourceGuard {
            component: "crasher".to_string(),
            compute_latency_ms: 50,
            max_latency_ms: 1000,
            memory_usage_bytes: 100,
            max_memory_bytes: 10_000_000,
            panic_occurred: true,
            panic_cruel: true,
        };
        assert!(!guard.is_constitutional());
        assert_eq!(guard.verdict(), Verdict::Rejected);
    }

    #[test]
    fn recoverable_panic_not_cruel() {
        let guard = ResourceGuard {
            component: "caught_panic".to_string(),
            compute_latency_ms: 50,
            max_latency_ms: 1000,
            memory_usage_bytes: 100,
            max_memory_bytes: 10_000_000,
            panic_occurred: true,
            panic_cruel: false,
        };
        assert!(guard.is_constitutional());
    }

    #[test]
    fn multiple_violations() {
        let guard = ResourceGuard {
            component: "disaster".to_string(),
            compute_latency_ms: 99_999,
            max_latency_ms: 1000,
            memory_usage_bytes: 999_999_999,
            max_memory_bytes: 10_000_000,
            panic_occurred: true,
            panic_cruel: true,
        };
        assert_eq!(guard.violations().len(), 3);
        assert_eq!(guard.verdict(), Verdict::Rejected);
    }
}
