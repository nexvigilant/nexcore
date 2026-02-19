//! Resilience Pattern Checker (HOOK-M5-51)
//!
//! Verifies external calls have proper timeout and fault-tolerance handling.
//! Detects HTTP clients, database connections, and gRPC calls without timeout configuration.

use regex::Regex;
use std::sync::LazyLock;

/// Kind of external call detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallKind {
    /// HTTP client (reqwest, hyper, ureq)
    Http,
    /// Database connection (sqlx, diesel, tokio-postgres)
    Database,
    /// gRPC call (tonic)
    Grpc,
    /// Redis/cache operations
    Cache,
    /// Generic external service call
    External,
}

impl CallKind {
    /// Returns the human-readable name of this call kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "HTTP Client",
            Self::Database => "Database",
            Self::Grpc => "gRPC",
            Self::Cache => "Cache/Redis",
            Self::External => "External Call",
        }
    }
}

/// Resilience issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResilienceSeverity {
    /// High - should block (e.g., database without timeout)
    High,
    /// Medium - should warn
    Medium,
    /// Low - informational
    Low,
}

impl ResilienceSeverity {
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

/// A detected external call
#[derive(Debug, Clone)]
pub struct ExternalCall {
    /// Kind of external call
    pub kind: CallKind,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Preview of the matched code
    pub code_preview: String,
    /// Whether timeout is configured
    pub has_timeout: bool,
    /// Whether fault tolerance is present
    pub has_fault_tolerance: bool,
}

/// A resilience issue
#[derive(Debug, Clone)]
pub struct ResilienceIssue {
    /// Kind of issue
    pub kind: ResilienceIssueKind,
    /// Severity level
    pub severity: ResilienceSeverity,
    /// Line number (1-indexed)
    pub line: usize,
    /// Related call kind
    pub call_kind: CallKind,
    /// Code preview
    pub code_preview: String,
    /// Suggested fix
    pub fix_suggestion: &'static str,
}

/// Kind of resilience issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResilienceIssueKind {
    /// External call without timeout
    MissingTimeout,
    /// External call without fault tolerance
    MissingFaultTolerance,
    /// No circuit breaker for repeated failures
    NoCircuitBreaker,
    /// Connection pool without size limit
    UnboundedPool,
}

impl ResilienceIssueKind {
    /// Returns the human-readable name of this issue kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingTimeout => "Missing Timeout",
            Self::MissingFaultTolerance => "Missing Fault Tolerance",
            Self::NoCircuitBreaker => "No Circuit Breaker",
            Self::UnboundedPool => "Unbounded Pool",
        }
    }
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

// HTTP client patterns (reqwest, hyper, ureq)
static HTTP_CLIENT: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r"(reqwest|hyper|ureq|Client)::.*?(get|post|put|delete|patch|request|send)")
});

// Timeout configuration patterns
static TIMEOUT_CONFIG: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(
        r"(?i)(\.timeout\(|with_timeout\(|timeout_config|timeout:|connect_timeout|read_timeout|write_timeout)",
    )
});

// Database pool patterns (sqlx, diesel, tokio-postgres)
static DB_POOL: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(
        r"(Pool|PgPool|MySqlPool|SqlitePool|PgPoolOptions|ConnectionPool)::?(new|connect|create|builder)",
    )
});

// Database pool timeout
static POOL_TIMEOUT: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(
        r"(?i)(acquire_timeout|connection_timeout|idle_timeout|max_lifetime|pool_timeout)",
    )
});

// gRPC patterns (tonic)
static GRPC_CALL: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(tonic|GrpcClient|Channel)::.*?(connect|call|send)"));

// Fault tolerance patterns (backoff, resilience)
static FAULT_TOLERANCE: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(backoff|exponential|with_backoff|max_attempts|attempt)"));

// Circuit breaker patterns
static CIRCUIT_BREAKER: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(circuit_breaker|CircuitBreaker|failsafe|breaker)"));

// Redis/cache patterns
static REDIS_CLIENT: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(redis|Redis|RedisClient|Cache)::.*?(get|set|connect|cmd)"));

/// Check if a line should be skipped
fn should_skip_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Skip comments
    if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
        return true;
    }

    // Skip use statements
    if trimmed.starts_with("use ") {
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

    // Only check Rust files
    !lower.ends_with(".rs")
}

/// Find external calls in the content
pub fn find_external_calls(content: &str) -> Vec<ExternalCall> {
    let mut calls = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Track context - look at surrounding lines for timeout config
    for (line_idx, line) in lines.iter().enumerate() {
        if should_skip_line(line) {
            continue;
        }

        // Check for HTTP calls
        if let Some(m) = HTTP_CLIENT.find(line) {
            let context = get_context(&lines, line_idx, 5);
            calls.push(ExternalCall {
                kind: CallKind::Http,
                line: line_idx + 1,
                column: m.start() + 1,
                code_preview: truncate_preview(line.trim(), 60),
                has_timeout: TIMEOUT_CONFIG.is_match(&context),
                has_fault_tolerance: FAULT_TOLERANCE.is_match(&context),
            });
        }

        // Check for database pool creation
        if let Some(m) = DB_POOL.find(line) {
            let context = get_context(&lines, line_idx, 10);
            calls.push(ExternalCall {
                kind: CallKind::Database,
                line: line_idx + 1,
                column: m.start() + 1,
                code_preview: truncate_preview(line.trim(), 60),
                has_timeout: POOL_TIMEOUT.is_match(&context),
                has_fault_tolerance: FAULT_TOLERANCE.is_match(&context),
            });
        }

        // Check for gRPC calls
        if let Some(m) = GRPC_CALL.find(line) {
            let context = get_context(&lines, line_idx, 5);
            calls.push(ExternalCall {
                kind: CallKind::Grpc,
                line: line_idx + 1,
                column: m.start() + 1,
                code_preview: truncate_preview(line.trim(), 60),
                has_timeout: TIMEOUT_CONFIG.is_match(&context),
                has_fault_tolerance: FAULT_TOLERANCE.is_match(&context),
            });
        }

        // Check for Redis/cache calls
        if let Some(m) = REDIS_CLIENT.find(line) {
            let context = get_context(&lines, line_idx, 5);
            calls.push(ExternalCall {
                kind: CallKind::Cache,
                line: line_idx + 1,
                column: m.start() + 1,
                code_preview: truncate_preview(line.trim(), 60),
                has_timeout: TIMEOUT_CONFIG.is_match(&context),
                has_fault_tolerance: FAULT_TOLERANCE.is_match(&context),
            });
        }
    }

    calls
}

/// Get context around a line
fn get_context(lines: &[&str], line_idx: usize, radius: usize) -> String {
    let start = line_idx.saturating_sub(radius);
    let end = (line_idx + radius + 1).min(lines.len());
    lines[start..end].join("\n")
}

/// Truncate a string for preview display
fn truncate_preview(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Analyze resilience patterns in file content.
///
/// # Arguments
/// * `file_path` - Path to the file (used for context and filtering)
/// * `content` - File content to scan
///
/// # Returns
/// Vector of detected resilience issues
pub fn analyze_resilience_patterns(file_path: &str, content: &str) -> Vec<ResilienceIssue> {
    let mut issues = Vec::new();

    if should_skip_file(file_path) {
        return issues;
    }

    let calls = find_external_calls(content);
    let has_circuit_breaker = CIRCUIT_BREAKER.is_match(content);

    for call in &calls {
        // Check for missing timeout
        if !call.has_timeout {
            let severity = match call.kind {
                CallKind::Database => ResilienceSeverity::High, // DB without timeout is critical
                CallKind::Http | CallKind::Grpc => ResilienceSeverity::Medium,
                _ => ResilienceSeverity::Low,
            };

            let fix = match call.kind {
                CallKind::Http => "Add .timeout(Duration::from_secs(30))",
                CallKind::Database => "Add .acquire_timeout(Duration::from_secs(5))",
                CallKind::Grpc => "Add .timeout(Duration::from_secs(10))",
                _ => "Configure appropriate timeout",
            };

            issues.push(ResilienceIssue {
                kind: ResilienceIssueKind::MissingTimeout,
                severity,
                line: call.line,
                call_kind: call.kind,
                code_preview: call.code_preview.clone(),
                fix_suggestion: fix,
            });
        }

        // Check for missing fault tolerance (advisory only for now)
        if !call.has_fault_tolerance && matches!(call.kind, CallKind::Http | CallKind::Grpc) {
            issues.push(ResilienceIssue {
                kind: ResilienceIssueKind::MissingFaultTolerance,
                severity: ResilienceSeverity::Low,
                line: call.line,
                call_kind: call.kind,
                code_preview: call.code_preview.clone(),
                fix_suggestion: "Consider adding backoff with exponential delay",
            });
        }
    }

    // Check for circuit breaker if there are multiple external calls
    if calls.len() >= 3 && !has_circuit_breaker {
        issues.push(ResilienceIssue {
            kind: ResilienceIssueKind::NoCircuitBreaker,
            severity: ResilienceSeverity::Low,
            line: 1, // File-level issue
            call_kind: CallKind::External,
            code_preview: format!("{} external calls detected", calls.len()),
            fix_suggestion: "Consider circuit breaker for fault tolerance",
        });
    }

    issues
}

/// Format resilience issues for terminal output.
///
/// # Arguments
/// * `file_path` - Path to the scanned file
/// * `issues` - Vector of detected issues
///
/// # Returns
/// Formatted string with a visual report
pub fn format_resilience_issues(file_path: &str, issues: &[ResilienceIssue]) -> String {
    if issues.is_empty() {
        return String::new();
    }

    let high_count = issues
        .iter()
        .filter(|i| i.severity == ResilienceSeverity::High)
        .count();
    let medium_count = issues
        .iter()
        .filter(|i| i.severity == ResilienceSeverity::Medium)
        .count();

    let mut output = String::new();

    output.push_str(
        "┌─────────────────────────────────────────────────────────────────────────────┐\n",
    );

    if high_count > 0 {
        output.push_str(
            "│ ⛔ RESILIENCE ISSUES DETECTED                                               │\n",
        );
    } else {
        output.push_str(
            "│ ⚠️  RESILIENCE ISSUES DETECTED                                               │\n",
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
        let issue_type = issue.kind.as_str();
        let call_type = issue.call_kind.as_str();

        output.push_str(&format!(
            "│ {} [{:<6}] {} ({}) Line {:>4}                        │\n",
            icon,
            issue.severity.as_str(),
            issue_type,
            call_type,
            issue.line
        ));

        let preview = if issue.code_preview.len() > 60 {
            &issue.code_preview[..60]
        } else {
            &issue.code_preview
        };
        output.push_str(&format!("│   Code: {:<68} │\n", preview));

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
    fn test_detect_http_without_timeout() {
        // Use pattern that matches HTTP_CLIENT regex: (reqwest|hyper|ureq|Client)::...
        let content = r#"
            let response = reqwest::get("https://example.com").await?;
        "#;

        let issues = analyze_resilience_patterns("src/client.rs", content);
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| i.kind == ResilienceIssueKind::MissingTimeout)
        );
    }

    #[test]
    fn test_http_with_timeout_ok() {
        let content = r#"
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?;
            let response = client.get("https://example.com").send().await?;
        "#;

        let issues = analyze_resilience_patterns("src/client.rs", content);
        // Should not have MissingTimeout issue for HTTP
        let timeout_issues: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.kind == ResilienceIssueKind::MissingTimeout && i.call_kind == CallKind::Http
            })
            .collect();
        assert!(timeout_issues.is_empty());
    }

    #[test]
    fn test_database_without_timeout_high_severity() {
        let content = r#"
            let pool = PgPool::connect("postgres://...").await?;
        "#;

        let issues = analyze_resilience_patterns("src/db.rs", content);
        assert!(!issues.is_empty());

        let db_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.call_kind == CallKind::Database)
            .collect();
        assert!(!db_issues.is_empty());
        // Database without timeout should be high severity
        assert!(
            db_issues
                .iter()
                .any(|i| i.severity == ResilienceSeverity::High)
        );
    }

    #[test]
    fn test_database_with_pool_timeout_ok() {
        let content = r#"
            let pool = PgPoolOptions::new()
                .acquire_timeout(Duration::from_secs(5))
                .max_connections(10)
                .connect("postgres://...").await?;
        "#;

        let issues = analyze_resilience_patterns("src/db.rs", content);
        // Should not have high severity timeout issue
        let high_timeout_issues: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.kind == ResilienceIssueKind::MissingTimeout
                    && i.severity == ResilienceSeverity::High
            })
            .collect();
        assert!(high_timeout_issues.is_empty());
    }

    #[test]
    fn test_skip_test_files() {
        let content = r#"
            let client = reqwest::Client::new();
            let response = client.get("https://example.com").send().await?;
        "#;

        let issues = analyze_resilience_patterns("tests/integration.rs", content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_format_issues() {
        let issues = vec![ResilienceIssue {
            kind: ResilienceIssueKind::MissingTimeout,
            severity: ResilienceSeverity::High,
            line: 10,
            call_kind: CallKind::Database,
            code_preview: "PgPool::connect(...)".to_string(),
            fix_suggestion: "Add .acquire_timeout(Duration::from_secs(5))",
        }];

        let formatted = format_resilience_issues("src/db.rs", &issues);
        assert!(formatted.contains("RESILIENCE ISSUES"));
        assert!(formatted.contains("Missing Timeout"));
        assert!(formatted.contains("Database"));
    }
}
