//! Deployment & Operations Integrity Framework (Module 5)
//!
//! This module ensures code is deployment-ready, not just development-complete.

pub mod config;
pub mod health;
pub mod observability;
pub mod resilience;
pub mod secrets;

// Re-export secrets functionality
pub use secrets::{SecretCategory, SecretFinding, SecretSeverity, scan_for_secrets};
pub use secrets::{calculate_shannon_entropy, format_secret_findings, passes_luhn, redact_secret};

// Re-export config functionality
pub use config::{
    ConfigIssue, ConfigSeverity, HardcodeCategory, detect_hardcoded_config, format_config_issues,
};

// Re-export health functionality
pub use health::{
    HealthAnalysis, HealthIssue, ShutdownIssue, analyze_health_and_shutdown, format_health_analysis,
};

// Re-export resilience functionality
pub use resilience::{
    ResilienceIssue, ResilienceIssueKind, analyze_resilience_patterns, format_resilience_issues,
};

// Re-export observability functionality
pub use observability::{
    LoggingCoverage, ObservabilityFinding, ObservabilityIssue, analyze_observability,
    format_observability_findings,
};
