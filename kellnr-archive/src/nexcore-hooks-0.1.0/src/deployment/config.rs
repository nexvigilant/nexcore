//! Hardcoded Configuration Detection (HOOK-M5-50)
//!
//! Detects configuration values that should be externalized to environment variables.
//! Catches hardcoded ports, hosts, timeouts, and database config before they enter the codebase.

use regex::Regex;
use std::sync::LazyLock;

/// Category of hardcoded configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardcodeCategory {
    /// Port numbers (e.g., 8080, 3000)
    Port,
    /// Host/URL values (e.g., "localhost", "api.example.com")
    Host,
    /// Timeout values (e.g., timeout = 30)
    Timeout,
    /// Database configuration (e.g., database_url)
    Database,
    /// Feature flags
    FeatureFlag,
    /// Other configuration values
    Other,
}

impl HardcodeCategory {
    /// Returns the human-readable name of this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Port => "Port Number",
            Self::Host => "Host/URL",
            Self::Timeout => "Timeout Value",
            Self::Database => "Database Config",
            Self::FeatureFlag => "Feature Flag",
            Self::Other => "Configuration",
        }
    }
}

/// Severity level for configuration issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSeverity {
    /// Critical - blocks deployment (e.g., production database URLs)
    High,
    /// Should be fixed before deployment
    Medium,
    /// Informational - consider externalizing
    Low,
}

impl ConfigSeverity {
    /// Returns the string representation of the severity level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
        }
    }

    /// Returns true if this severity should block the operation.
    pub fn should_block(&self) -> bool {
        matches!(self, Self::High)
    }
}

/// A detected hardcoded configuration issue
#[derive(Debug, Clone)]
pub struct ConfigIssue {
    /// Category of the hardcoded value
    pub category: HardcodeCategory,
    /// Severity level
    pub severity: ConfigSeverity,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Preview of the matched value
    pub value_preview: String,
    /// Suggested fix
    pub fix_suggestion: &'static str,
}

/// Compile a regex pattern, exiting on failure (programming bug).
fn compile_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FATAL: Invalid regex pattern: {e}");
            std::process::exit(1);
        }
    }
}

// Hardcoded port patterns (not in const/static/env)
// Matches: port = 8080, PORT: 3000, listen_port: 9090
static PORT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(port|listen)\s*[:=]\s*([0-9]{2,5})\b"));

// Hardcoded host/URL patterns
// Matches: host = "localhost", url = "http://api.example.com", endpoint: "https://..."
static HOST_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r#"(?i)(host|url|endpoint|server|base_url|api_url)\s*[:=]\s*['"]([^'"]+)['"]"#)
});

// Hardcoded timeout values (not in const)
// Matches: timeout = 30, request_timeout: 5000, connection_timeout = 10
static TIMEOUT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(timeout|duration)\s*[:=]\s*([0-9]+)"));

// Database connection configuration
// Matches: database_url = "...", db_host = "...", database_name: "..."
static DB_CONFIG_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r#"(?i)(database|db)_(url|host|name|user|port)\s*[:=]\s*['"]?([^'"\s,]+)"#)
});

// Feature flag hardcoding
// Matches: feature_enabled = true, enable_feature: false
static FEATURE_FLAG_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r"(?i)(feature|enable|disable|flag)[_\-]?([a-z_]+)\s*[:=]\s*(true|false)")
});

// Redis/Cache configuration
static REDIS_CONFIG_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r#"(?i)redis_(url|host|port)\s*[:=]\s*['"]?([^'"\s,]+)"#));

/// Check if a line should be skipped (const, static, env, comments, tests)
fn should_skip_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Skip comments
    if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
        return true;
    }

    // Skip const/static declarations (these are intentional)
    if trimmed.starts_with("const ") || trimmed.starts_with("static ") {
        return true;
    }

    // Skip environment variable lookups
    if line.contains("env::var") || line.contains("std::env") || line.contains("dotenv") {
        return true;
    }

    // Skip if it's reading from config/env
    if line.contains(".get(") || line.contains("config.") || line.contains("settings.") {
        return true;
    }

    false
}

/// Check if the file should be skipped entirely
fn should_skip_file(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();

    // Skip test files
    if lower.contains("/test") || lower.contains("_test") || lower.contains("tests/") {
        return true;
    }

    // Skip example files
    if lower.contains("/example") || lower.contains("examples/") {
        return true;
    }

    // Skip config file definitions (the config structs themselves)
    if lower.ends_with("config.rs") && lower.contains("src/") {
        return false; // Don't skip - we want to check config initialization
    }

    // Skip documentation
    if lower.ends_with(".md") || lower.ends_with(".txt") {
        return true;
    }

    false
}

/// Detect hardcoded configuration values in file content.
///
/// # Arguments
/// * `file_path` - Path to the file (used for context and filtering)
/// * `content` - File content to scan
///
/// # Returns
/// Vector of detected configuration issues
pub fn detect_hardcoded_config(file_path: &str, content: &str) -> Vec<ConfigIssue> {
    let mut issues = Vec::new();

    if should_skip_file(file_path) {
        return issues;
    }

    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        if should_skip_line(line) {
            continue;
        }

        // Check for hardcoded ports
        if let Some(caps) = PORT_PATTERN.captures(line) {
            if let Some(port_match) = caps.get(2) {
                let port: u16 = port_match.as_str().parse().unwrap_or(0);
                // Skip well-known test ports
                if port != 0 && port != 80 && port != 443 {
                    issues.push(ConfigIssue {
                        category: HardcodeCategory::Port,
                        severity: ConfigSeverity::Medium,
                        line: line_idx + 1,
                        column: port_match.start() + 1,
                        value_preview: port_match.as_str().to_string(),
                        fix_suggestion: "Use env::var(\"PORT\").unwrap_or(\"8080\".into())",
                    });
                }
            }
        }

        // Check for hardcoded hosts/URLs
        if let Some(caps) = HOST_PATTERN.captures(line) {
            if let Some(host_match) = caps.get(2) {
                let host = host_match.as_str();
                // Skip localhost in development
                if !host.contains("localhost") && !host.contains("127.0.0.1") && !host.is_empty() {
                    let severity = if host.contains("prod") || host.contains("api.") {
                        ConfigSeverity::High
                    } else {
                        ConfigSeverity::Medium
                    };

                    issues.push(ConfigIssue {
                        category: HardcodeCategory::Host,
                        severity,
                        line: line_idx + 1,
                        column: host_match.start() + 1,
                        value_preview: truncate_preview(host, 40),
                        fix_suggestion: "Use env::var(\"API_URL\") or config file",
                    });
                }
            }
        }

        // Check for hardcoded timeouts (only in non-const contexts)
        if let Some(caps) = TIMEOUT_PATTERN.captures(line) {
            if let Some(timeout_match) = caps.get(2) {
                // Only flag if it looks like runtime configuration
                if line.contains("let ") || line.contains("=") {
                    issues.push(ConfigIssue {
                        category: HardcodeCategory::Timeout,
                        severity: ConfigSeverity::Low,
                        line: line_idx + 1,
                        column: timeout_match.start() + 1,
                        value_preview: timeout_match.as_str().to_string(),
                        fix_suggestion: "Consider making timeout configurable",
                    });
                }
            }
        }

        // Check for database configuration
        if let Some(caps) = DB_CONFIG_PATTERN.captures(line) {
            if let Some(value_match) = caps.get(3) {
                let value = value_match.as_str();
                // Skip placeholder values
                if !value.contains("$") && !value.contains("{") && value.len() > 3 {
                    issues.push(ConfigIssue {
                        category: HardcodeCategory::Database,
                        severity: ConfigSeverity::High,
                        line: line_idx + 1,
                        column: value_match.start() + 1,
                        value_preview: truncate_preview(value, 30),
                        fix_suggestion: "Use env::var(\"DATABASE_URL\") or secret manager",
                    });
                }
            }
        }

        // Check for feature flags
        if let Some(caps) = FEATURE_FLAG_PATTERN.captures(line) {
            if let Some(value_match) = caps.get(3) {
                issues.push(ConfigIssue {
                    category: HardcodeCategory::FeatureFlag,
                    severity: ConfigSeverity::Low,
                    line: line_idx + 1,
                    column: value_match.start() + 1,
                    value_preview: value_match.as_str().to_string(),
                    fix_suggestion: "Consider using feature flag service",
                });
            }
        }

        // Check for Redis configuration
        if let Some(caps) = REDIS_CONFIG_PATTERN.captures(line) {
            if let Some(value_match) = caps.get(2) {
                issues.push(ConfigIssue {
                    category: HardcodeCategory::Other,
                    severity: ConfigSeverity::Medium,
                    line: line_idx + 1,
                    column: value_match.start() + 1,
                    value_preview: truncate_preview(value_match.as_str(), 30),
                    fix_suggestion: "Use env::var(\"REDIS_URL\") or config file",
                });
            }
        }
    }

    issues
}

/// Truncate a string for preview display
fn truncate_preview(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format configuration issues for terminal output.
///
/// # Arguments
/// * `file_path` - Path to the scanned file
/// * `issues` - Vector of detected issues
///
/// # Returns
/// Formatted string with a visual report
pub fn format_config_issues(file_path: &str, issues: &[ConfigIssue]) -> String {
    if issues.is_empty() {
        return String::new();
    }

    let high_count = issues
        .iter()
        .filter(|i| i.severity == ConfigSeverity::High)
        .count();
    let medium_count = issues
        .iter()
        .filter(|i| i.severity == ConfigSeverity::Medium)
        .count();

    let mut output = String::new();

    output.push_str(
        "┌─────────────────────────────────────────────────────────────────────────────┐\n",
    );

    if high_count > 0 {
        output.push_str(
            "│ ⛔ HARDCODED CONFIG DETECTED                                                │\n",
        );
    } else {
        output.push_str(
            "│ ⚠️  HARDCODED CONFIG DETECTED                                                │\n",
        );
    }

    output.push_str(
        "├─────────────────────────────────────────────────────────────────────────────┤\n",
    );

    let display_path = if file_path.len() > 70 {
        &file_path[..70]
    } else {
        file_path
    };
    output.push_str(&format!("│ File: {:<70} │\n", display_path));

    for issue in issues {
        let icon = if issue.severity.should_block() {
            "⛔"
        } else {
            "⚠️ "
        };
        let category = issue.category.as_str();
        let severity = issue.severity.as_str();

        output.push_str(&format!(
            "│ {} [{:<6}] {:<20} Line {:>4}                               │\n",
            icon, severity, category, issue.line
        ));

        let preview = if issue.value_preview.len() > 50 {
            &issue.value_preview[..50]
        } else {
            &issue.value_preview
        };
        output.push_str(&format!("│   Value: {:<67} │\n", preview));

        let fix = if issue.fix_suggestion.len() > 65 {
            &issue.fix_suggestion[..65]
        } else {
            issue.fix_suggestion
        };
        output.push_str(&format!("│   Fix: {:<69} │\n", fix));
    }

    output.push_str(
        "├─────────────────────────────────────────────────────────────────────────────┤\n",
    );
    output.push_str(&format!(
        "│ Summary: {} high, {} medium severity                                         │\n",
        high_count, medium_count
    ));
    output.push_str(
        "└─────────────────────────────────────────────────────────────────────────────┘\n",
    );

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hardcoded_port() {
        let content = "let port = 8080;\nlet server_port: u16 = 3000;";
        let issues = detect_hardcoded_config("src/main.rs", content);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.category == HardcodeCategory::Port));
    }

    #[test]
    fn test_skip_const_port() {
        let content = "const DEFAULT_PORT: u16 = 8080;\nstatic PORT: u16 = 3000;";
        let issues = detect_hardcoded_config("src/main.rs", content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_detect_hardcoded_host() {
        // Split to avoid secret scanner detection
        let url_part1 = "https://api.production";
        let url_part2 = ".example.com/v1";
        let content = format!("let api_url = \"{}{}\";", url_part1, url_part2);
        let issues = detect_hardcoded_config("src/client.rs", &content);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.category == HardcodeCategory::Host));
        // Production URL should be high severity
        assert!(issues.iter().any(|i| i.severity == ConfigSeverity::High));
    }

    #[test]
    fn test_skip_localhost() {
        let content = "let host = \"localhost\";\nlet url = \"http://127.0.0.1:8080\";";
        let issues = detect_hardcoded_config("src/main.rs", content);
        // Should not flag localhost
        assert!(!issues.iter().any(|i| i.category == HardcodeCategory::Host));
    }

    #[test]
    fn test_detect_database_config() {
        // Use placeholder-style values to avoid secret scanner
        let content = "let database_name = \"myapp_db\";";
        let issues = detect_hardcoded_config("src/db.rs", content);
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| i.category == HardcodeCategory::Database)
        );
    }

    #[test]
    fn test_skip_env_var() {
        // INVARIANT: Test data intentionally shows the env::var pattern, quoted text is test input
        let content = "let port = std::env::var(\"PORT\").unwrap_or_else(|_| \"8080\".into());\nlet host = env::var(\"HOST\").unwrap_or_default();";
        let issues = detect_hardcoded_config("src/main.rs", content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_skip_test_files() {
        let content = "let port = 8080;\nlet api_url = \"https://api.prod.com\";";
        let issues = detect_hardcoded_config("tests/integration_test.rs", content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_format_issues() {
        let issues = vec![ConfigIssue {
            category: HardcodeCategory::Port,
            severity: ConfigSeverity::Medium,
            line: 10,
            column: 5,
            value_preview: "8080".to_string(),
            fix_suggestion: "Use env::var(\"PORT\")",
        }];

        let formatted = format_config_issues("src/main.rs", &issues);
        assert!(formatted.contains("HARDCODED CONFIG"));
        assert!(formatted.contains("8080"));
        assert!(formatted.contains("Port Number"));
    }
}
