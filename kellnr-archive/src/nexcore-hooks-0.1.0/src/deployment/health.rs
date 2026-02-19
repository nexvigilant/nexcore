//! Health Check Analysis (HOOK-M5-52)
//!
//! Verifies health endpoints and graceful shutdown handling.
//! Checks for /health, /ready, /live routes and signal handlers.

use regex::Regex;
use std::sync::LazyLock;

/// Kind of health issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthIssueKind {
    /// No health endpoint found
    MissingHealthEndpoint,
    /// No readiness probe
    MissingReadinessProbe,
    /// No liveness probe
    MissingLivenessProbe,
}

impl HealthIssueKind {
    /// Returns the human-readable name of this issue kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingHealthEndpoint => "Missing Health Endpoint",
            Self::MissingReadinessProbe => "Missing Readiness Probe",
            Self::MissingLivenessProbe => "Missing Liveness Probe",
        }
    }

    /// Returns the suggested fix for this issue.
    pub fn fix_suggestion(&self) -> &'static str {
        match self {
            Self::MissingHealthEndpoint => "Add GET /health endpoint returning 200 OK",
            Self::MissingReadinessProbe => "Add GET /ready for k8s readiness probe",
            Self::MissingLivenessProbe => "Add GET /live for k8s liveness probe",
        }
    }
}

/// A detected health check issue
#[derive(Debug, Clone)]
pub struct HealthIssue {
    /// Kind of issue
    pub kind: HealthIssueKind,
    /// Suggested fix
    pub suggestion: &'static str,
}

/// Kind of shutdown issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownIssueKind {
    /// No signal handler found
    MissingSignalHandler,
    /// No graceful shutdown logic
    MissingGracefulShutdown,
    /// No cleanup on termination
    MissingCleanup,
}

impl ShutdownIssueKind {
    /// Returns the human-readable name of this issue kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingSignalHandler => "Missing Signal Handler",
            Self::MissingGracefulShutdown => "Missing Graceful Shutdown",
            Self::MissingCleanup => "Missing Cleanup Logic",
        }
    }

    /// Returns the suggested fix for this issue.
    pub fn fix_suggestion(&self) -> &'static str {
        match self {
            Self::MissingSignalHandler => "Add tokio::signal::ctrl_c() or signal handler",
            Self::MissingGracefulShutdown => "Use with_graceful_shutdown() on server",
            Self::MissingCleanup => "Add cleanup logic for connections/resources",
        }
    }
}

/// A detected shutdown issue
#[derive(Debug, Clone)]
pub struct ShutdownIssue {
    /// Kind of issue
    pub kind: ShutdownIssueKind,
    /// Suggested fix
    pub suggestion: &'static str,
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

// Axum/Actix/Rocket health routes
static HEALTH_ROUTE: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r#"[./"](?:health|healthz|healthcheck)["/\)]"#));

static READY_ROUTE: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r#"[./"](?:ready|readyz|readiness)["/\)]"#));

static LIVE_ROUTE: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r#"[./"](?:live|livez|liveness)["/\)]"#));

// Signal handlers
static SIGNAL_HANDLER: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(signal|sigterm|sigint|ctrl_c|shutdown_signal)"));

// Tokio graceful shutdown
static GRACEFUL_SHUTDOWN: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(graceful|with_graceful_shutdown|shutdown_signal)"));

// Router/server patterns (to detect if this is a web app)
static WEB_APP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r"(?i)(Router::new|App::new|HttpServer|axum::serve|rocket::build)")
});

/// Check if content represents a web application
fn is_web_app(content: &str) -> bool {
    WEB_APP_PATTERN.is_match(content)
}

/// Check if content is a main/entry point
fn is_entry_point(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    lower.ends_with("main.rs")
        || lower.ends_with("lib.rs")
        || lower.contains("/server")
        || lower.contains("/app")
}

/// Check if the file should be skipped entirely
fn should_skip_file(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();

    // Skip test files
    if lower.contains("/test") || lower.contains("_test") || lower.contains("tests/") {
        return true;
    }

    // Only check Rust files
    !lower.ends_with(".rs")
}

/// Analyze health check configuration in file content.
///
/// # Arguments
/// * `content` - File content to scan
///
/// # Returns
/// Vector of detected health issues
pub fn analyze_health_checks(content: &str) -> Vec<HealthIssue> {
    let mut issues = Vec::new();

    // Only check if this looks like a web application
    if !is_web_app(content) {
        return issues;
    }

    // Check for health endpoint
    if !HEALTH_ROUTE.is_match(content) {
        issues.push(HealthIssue {
            kind: HealthIssueKind::MissingHealthEndpoint,
            suggestion: HealthIssueKind::MissingHealthEndpoint.fix_suggestion(),
        });
    }

    // Check for readiness probe (k8s)
    if !READY_ROUTE.is_match(content) {
        issues.push(HealthIssue {
            kind: HealthIssueKind::MissingReadinessProbe,
            suggestion: HealthIssueKind::MissingReadinessProbe.fix_suggestion(),
        });
    }

    // Check for liveness probe (k8s)
    if !LIVE_ROUTE.is_match(content) {
        issues.push(HealthIssue {
            kind: HealthIssueKind::MissingLivenessProbe,
            suggestion: HealthIssueKind::MissingLivenessProbe.fix_suggestion(),
        });
    }

    issues
}

/// Analyze graceful shutdown handling in file content.
///
/// # Arguments
/// * `file_path` - Path to the file
/// * `content` - File content to scan
///
/// # Returns
/// Vector of detected shutdown issues
pub fn analyze_graceful_shutdown(file_path: &str, content: &str) -> Vec<ShutdownIssue> {
    let mut issues = Vec::new();

    // Only check entry points for shutdown handling
    if !is_entry_point(file_path) {
        return issues;
    }

    // Only check if this looks like a web application
    if !is_web_app(content) {
        return issues;
    }

    // Check for signal handler
    if !SIGNAL_HANDLER.is_match(content) {
        issues.push(ShutdownIssue {
            kind: ShutdownIssueKind::MissingSignalHandler,
            suggestion: ShutdownIssueKind::MissingSignalHandler.fix_suggestion(),
        });
    }

    // Check for graceful shutdown
    if !GRACEFUL_SHUTDOWN.is_match(content) {
        issues.push(ShutdownIssue {
            kind: ShutdownIssueKind::MissingGracefulShutdown,
            suggestion: ShutdownIssueKind::MissingGracefulShutdown.fix_suggestion(),
        });
    }

    issues
}

/// Combined analysis result
#[derive(Debug, Default)]
pub struct HealthAnalysis {
    /// Health endpoint issues
    pub health_issues: Vec<HealthIssue>,
    /// Shutdown issues
    pub shutdown_issues: Vec<ShutdownIssue>,
}

impl HealthAnalysis {
    /// Returns true if there are any issues
    pub fn has_issues(&self) -> bool {
        !self.health_issues.is_empty() || !self.shutdown_issues.is_empty()
    }

    /// Returns the total number of issues
    pub fn issue_count(&self) -> usize {
        self.health_issues.len() + self.shutdown_issues.len()
    }
}

/// Analyze all health and shutdown patterns.
///
/// # Arguments
/// * `file_path` - Path to the file
/// * `content` - File content to scan
///
/// # Returns
/// Combined analysis result
pub fn analyze_health_and_shutdown(file_path: &str, content: &str) -> HealthAnalysis {
    if should_skip_file(file_path) {
        return HealthAnalysis::default();
    }

    HealthAnalysis {
        health_issues: analyze_health_checks(content),
        shutdown_issues: analyze_graceful_shutdown(file_path, content),
    }
}

/// Format health analysis for terminal output.
///
/// # Arguments
/// * `file_path` - Path to the scanned file
/// * `analysis` - Combined analysis result
///
/// # Returns
/// Formatted string with a visual report
pub fn format_health_analysis(file_path: &str, analysis: &HealthAnalysis) -> String {
    if !analysis.has_issues() {
        return String::new();
    }

    let mut output = String::new();

    output.push_str(
        "┌─────────────────────────────────────────────────────────────────────────────┐\n",
    );
    output.push_str(
        "│ ⚠️  HEALTH CHECK ADVISORY                                                    │\n",
    );
    output.push_str(
        "├─────────────────────────────────────────────────────────────────────────────┤\n",
    );

    let display_path = if file_path.len() > 70 {
        &file_path[..70]
    } else {
        file_path
    };
    output.push_str(&format!("│ File: {:<70} │\n", display_path));

    if !analysis.health_issues.is_empty() {
        output.push_str(
            "│ Health Endpoints:                                                           │\n",
        );
        for issue in &analysis.health_issues {
            output.push_str(&format!("│   ⚠️  {:<70} │\n", issue.kind.as_str()));
            output.push_str(&format!("│      Fix: {:<66} │\n", issue.suggestion));
        }
    }

    if !analysis.shutdown_issues.is_empty() {
        output.push_str(
            "│ Shutdown Handling:                                                          │\n",
        );
        for issue in &analysis.shutdown_issues {
            output.push_str(&format!("│   ⚠️  {:<70} │\n", issue.kind.as_str()));
            output.push_str(&format!("│      Fix: {:<66} │\n", issue.suggestion));
        }
    }

    output.push_str(
        "├─────────────────────────────────────────────────────────────────────────────┤\n",
    );
    output.push_str(&format!(
        "│ Total: {} issue(s) - advisory only, not blocking                            │\n",
        analysis.issue_count()
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
    fn test_detect_missing_health_endpoint() {
        let content = r#"
            let app = Router::new()
                .route("/api/users", get(list_users));
        "#;

        let issues = analyze_health_checks(content);
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| i.kind == HealthIssueKind::MissingHealthEndpoint)
        );
    }

    #[test]
    fn test_health_endpoint_present() {
        let content = r#"
            let app = Router::new()
                .route("/health", get(health_check))
                .route("/ready", get(readiness))
                .route("/live", get(liveness));
        "#;

        let issues = analyze_health_checks(content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_detect_missing_graceful_shutdown() {
        let content = r#"
            let app = Router::new()
                .route("/health", get(health_check));
            axum::serve(listener, app).await?;
        "#;

        let issues = analyze_graceful_shutdown("src/main.rs", content);
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| i.kind == ShutdownIssueKind::MissingSignalHandler)
        );
    }

    #[test]
    fn test_graceful_shutdown_present() {
        let content = r#"
            let app = Router::new()
                .route("/health", get(health_check));
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        "#;

        let issues = analyze_graceful_shutdown("src/main.rs", content);
        // Should not have MissingGracefulShutdown
        assert!(
            !issues
                .iter()
                .any(|i| i.kind == ShutdownIssueKind::MissingGracefulShutdown)
        );
    }

    #[test]
    fn test_skip_non_web_app() {
        let content = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;

        let issues = analyze_health_checks(content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_format_analysis() {
        let analysis = HealthAnalysis {
            health_issues: vec![HealthIssue {
                kind: HealthIssueKind::MissingHealthEndpoint,
                suggestion: "Add /health endpoint",
            }],
            shutdown_issues: vec![],
        };

        let formatted = format_health_analysis("src/main.rs", &analysis);
        assert!(formatted.contains("HEALTH CHECK"));
        assert!(formatted.contains("Missing Health Endpoint"));
    }
}
