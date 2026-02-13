//! Configuration for security scanning.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Severity level for security issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
    /// Get the numeric weight for scoring.
    #[must_use]
    pub const fn weight(self) -> u32 {
        match self {
            Self::Critical => 20,
            Self::High => 10,
            Self::Medium => 5,
            Self::Low => 2,
            Self::Info => 0,
        }
    }
}

impl std::fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Configuration for security scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    // Feature flags
    /// Enable secret pattern detection
    pub detect_secrets: bool,
    /// Enable code vulnerability analysis
    pub analyze_code: bool,
    /// Enable dependency vulnerability checking
    pub check_dependencies: bool,
    /// Enable high-entropy string detection
    pub entropy_check_enabled: bool,
    /// Enable secret pattern matching
    pub secret_patterns_enabled: bool,

    // Vulnerability checks
    /// Check for SQL injection patterns
    pub check_sql_injection: bool,
    /// Check for XSS patterns
    pub check_xss: bool,
    /// Check for command injection patterns
    pub check_command_injection: bool,
    /// Check for path traversal patterns
    pub check_path_traversal: bool,
    /// Check for outdated dependencies
    pub check_outdated_dependencies: bool,

    // Thresholds
    /// Entropy threshold for high-entropy detection (default: 4.5)
    pub high_entropy_threshold: f64,
    /// Maximum file size to scan in MB
    pub max_file_size_mb: f64,
    /// Minimum security score to pass (0-100)
    pub min_security_score: u32,

    // Failure conditions
    /// Fail scan if critical issues found
    pub fail_on_critical: bool,
    /// Fail scan if high severity issues found
    pub fail_on_high: bool,

    // Filters
    /// File extensions to scan (empty = all)
    pub scan_extensions: HashSet<String>,
    /// Patterns to exclude from scanning
    pub exclude_patterns: HashSet<String>,
    /// Paths to whitelist (never flag)
    pub whitelisted_paths: HashSet<String>,
    /// Secrets to whitelist (known safe values)
    pub whitelisted_secrets: HashSet<String>,
    /// Custom secret patterns to add
    pub custom_secret_patterns: HashMap<String, String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        let scan_extensions: HashSet<String> = [
            ".py", ".js", ".ts", ".jsx", ".tsx", ".java", ".go", ".rs", ".rb", ".php", ".cs",
            ".cpp", ".c", ".h", ".hpp", ".sh", ".bash", ".zsh", ".yaml", ".yml", ".json", ".toml",
            ".xml", ".env", ".config", ".ini", ".sql",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let exclude_patterns: HashSet<String> = [
            "node_modules/*",
            ".git/*",
            "target/*",
            "dist/*",
            "build/*",
            "*.min.js",
            "*.min.css",
            "vendor/*",
            "__pycache__/*",
            ".venv/*",
            "venv/*",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Self {
            detect_secrets: true,
            analyze_code: true,
            check_dependencies: true,
            entropy_check_enabled: true,
            secret_patterns_enabled: true,
            check_sql_injection: true,
            check_xss: true,
            check_command_injection: true,
            check_path_traversal: true,
            check_outdated_dependencies: false,
            high_entropy_threshold: 4.5,
            max_file_size_mb: 10.0,
            min_security_score: 70,
            fail_on_critical: true,
            fail_on_high: false,
            scan_extensions,
            exclude_patterns,
            whitelisted_paths: HashSet::new(),
            whitelisted_secrets: HashSet::new(),
            custom_secret_patterns: HashMap::new(),
        }
    }
}

impl ScanConfig {
    /// Create a minimal config for quick scans.
    #[must_use]
    pub fn quick() -> Self {
        Self {
            entropy_check_enabled: false,
            check_outdated_dependencies: false,
            ..Default::default()
        }
    }

    /// Create a strict config for CI/CD pipelines.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            fail_on_critical: true,
            fail_on_high: true,
            min_security_score: 80,
            check_outdated_dependencies: true,
            ..Default::default()
        }
    }

    /// Add a custom secret pattern.
    pub fn add_secret_pattern(&mut self, name: &str, pattern: &str) {
        self.custom_secret_patterns
            .insert(name.to_string(), pattern.to_string());
    }

    /// Add an exclusion pattern.
    pub fn exclude(&mut self, pattern: &str) {
        self.exclude_patterns.insert(pattern.to_string());
    }

    /// Whitelist a path.
    pub fn whitelist_path(&mut self, path: &str) {
        self.whitelisted_paths.insert(path.to_string());
    }
}
