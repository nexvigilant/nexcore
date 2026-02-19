//! # Security Scanning
//!
//! Consolidated from `nexcore-security` crate.
//!
//! Security scanning library for detecting secrets, vulnerabilities, and dependency issues.
//!
//! ## Features
//!
//! - **Secrets Detection**: 30+ patterns for API keys, tokens, credentials
//! - **Vulnerability Analysis**: SQL injection, XSS, command injection, path traversal
//! - **Dependency Checking**: Parse and check package files for known vulnerabilities
//!
//! ## Conservation Law Reference
//!
//! Security scanning respects:
//! - **CL-6 (Entity Integrity)**: Security issues maintain complete audit trails
//! - **CL-10 (Causality)**: Temporal ordering of issue detection preserved
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_vigilance::security::{SecurityScanner, ScanConfig};
//! use std::path::Path;
//!
//! let config = ScanConfig::default();
//! let scanner = SecurityScanner::new(Path::new("."), config);
//! let result = scanner.scan();
//!
//! println!("Found {} issues", result.issues.len());
//! println!("Security score: {}/100", result.calculate_score());
//! ```

pub mod config;
pub mod dependencies;
pub mod scanner;
pub mod secrets;
pub mod vulnerabilities;

pub use config::{ScanConfig, SeverityLevel};
pub use dependencies::DependencyChecker;
pub use scanner::{ScanResult, SecurityIssue, SecurityScanner};
pub use secrets::SecretsDetector;
pub use vulnerabilities::{CodeAnalyzer, VulnerabilityPattern};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(SeverityLevel::Critical > SeverityLevel::High);
        assert!(SeverityLevel::High > SeverityLevel::Medium);
        assert!(SeverityLevel::Medium > SeverityLevel::Low);
        assert!(SeverityLevel::Low > SeverityLevel::Info);
    }

    #[test]
    fn test_default_config() {
        let config = ScanConfig::default();
        assert!(config.detect_secrets);
        assert!(config.analyze_code);
        assert!(config.check_dependencies);
        assert!(config.entropy_check_enabled);
    }

    #[test]
    fn test_security_issue_creation() {
        let issue = SecurityIssue {
            severity: SeverityLevel::High,
            category: "secrets".to_string(),
            title: "Test Issue".to_string(),
            description: "Test description".to_string(),
            file_path: "test.py".into(),
            line_number: Some(10),
            code_snippet: Some("secret = 'abc'".to_string()),
            remediation: Some("Remove the secret".to_string()),
            cwe_id: Some("CWE-798".to_string()),
            confidence: "high".to_string(),
            tov_signal: None,
            metadata: Default::default(),
        };

        assert_eq!(issue.severity, SeverityLevel::High);
        assert_eq!(issue.category, "secrets");
    }
}
