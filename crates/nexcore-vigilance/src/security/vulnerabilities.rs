//! Code vulnerability analysis.
//!
//! Detects common security vulnerabilities:
//! - SQL Injection
//! - XSS (Cross-Site Scripting)
//! - Command Injection
//! - Path Traversal
//! - Insecure Deserialization
//! - Weak Cryptography

use crate::security::config::SeverityLevel;
use crate::security::scanner::{SecurityIssue, SecurityScanner};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// A vulnerability pattern definition.
#[derive(Debug, Clone)]
pub struct VulnerabilityPattern {
    pub name: &'static str,
    pub pattern: &'static str,
    pub severity: SeverityLevel,
    pub cwe_id: &'static str,
    pub description: &'static str,
    pub remediation: &'static str,
    pub confidence: &'static str,
    /// File extensions this pattern applies to (None = all)
    pub languages: Option<&'static [&'static str]>,
}

/// SQL Injection patterns.
pub const SQL_INJECTION_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "SQL Injection - String Concatenation",
        pattern: r"(execute|query|cursor|exec)\s*\([^)]*[\+%].*SELECT|INSERT|UPDATE|DELETE|DROP",
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-89",
        description: "SQL query constructed using string concatenation with user input",
        remediation: "Use parameterized queries or prepared statements",
        confidence: "high",
        languages: Some(&["py", "php", "java", "cs"]),
    },
    VulnerabilityPattern {
        name: "SQL Injection - Format String",
        pattern: r"\.format\([^)]*\).*(?:SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)",
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-89",
        description: "SQL query using string formatting which may be vulnerable",
        remediation: "Use ORM or parameterized queries with placeholders",
        confidence: "high",
        languages: Some(&["py"]),
    },
    VulnerabilityPattern {
        name: "SQL Injection - f-string",
        pattern: r#"f["'].*\{.*\}.*(?:SELECT|INSERT|UPDATE|DELETE|DROP|CREATE)"#,
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-89",
        description: "SQL query using f-string interpolation",
        remediation: "Never use f-strings for SQL. Use parameterized queries.",
        confidence: "high",
        languages: Some(&["py"]),
    },
];

/// XSS patterns.
pub const XSS_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "XSS - innerHTML Assignment",
        pattern: r#"\.innerHTML\s*=\s*(?!["'<])"#,
        severity: SeverityLevel::High,
        cwe_id: "CWE-79",
        description: "Direct assignment to innerHTML with potentially unsafe content",
        remediation: "Use textContent or DOMPurify to sanitize HTML",
        confidence: "medium",
        languages: Some(&["js", "ts", "jsx", "tsx"]),
    },
    VulnerabilityPattern {
        name: "XSS - eval() Usage",
        pattern: r"\beval\s*\(",
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-95",
        description: "Use of eval() can execute arbitrary code",
        remediation: "Avoid eval(). Use JSON.parse() for JSON or safer alternatives",
        confidence: "high",
        languages: Some(&["js", "ts", "py"]),
    },
    VulnerabilityPattern {
        name: "XSS - dangerouslySetInnerHTML",
        pattern: r"dangerouslySetInnerHTML\s*=\s*\{",
        severity: SeverityLevel::High,
        cwe_id: "CWE-79",
        description: "React dangerouslySetInnerHTML usage",
        remediation: "Sanitize HTML using DOMPurify or avoid dangerouslySetInnerHTML",
        confidence: "medium",
        languages: Some(&["jsx", "tsx"]),
    },
];

/// Command Injection patterns.
pub const COMMAND_INJECTION_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "Command Injection - os.system",
        pattern: r"os\.system\s*\([^)]*[\+%\{]",
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-78",
        description: "Command execution using os.system with user input",
        remediation: "Use subprocess with shell=False and argument list",
        confidence: "high",
        languages: Some(&["py"]),
    },
    VulnerabilityPattern {
        name: "Command Injection - subprocess shell=True",
        pattern: r"subprocess\.(call|run|Popen|check_output)\s*\([^)]*shell\s*=\s*True",
        severity: SeverityLevel::High,
        cwe_id: "CWE-78",
        description: "Subprocess with shell=True can lead to command injection",
        remediation: "Use shell=False and pass command as list of arguments",
        confidence: "high",
        languages: Some(&["py"]),
    },
    VulnerabilityPattern {
        name: "Command Injection - exec()",
        pattern: r"\bexec\s*\(",
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-95",
        description: "Use of exec() can execute arbitrary code",
        remediation: "Avoid exec(). Redesign to use safer alternatives",
        confidence: "high",
        languages: Some(&["py", "php"]),
    },
];

/// Path Traversal patterns.
pub const PATH_TRAVERSAL_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "Path Traversal - Direct File Access",
        pattern: r"open\s*\([^)]*[\+%]\s*[^,)]*\s*[,)]",
        severity: SeverityLevel::High,
        cwe_id: "CWE-22",
        description: "File path constructed from user input",
        remediation: "Validate and sanitize file paths. Use os.path.basename()",
        confidence: "medium",
        languages: Some(&["py"]),
    },
    VulnerabilityPattern {
        name: "Path Traversal - Node.js File Read",
        pattern: r"(readFile|readFileSync|createReadStream)\s*\([^)]*\+",
        severity: SeverityLevel::High,
        cwe_id: "CWE-22",
        description: "Node.js file read with concatenated path",
        remediation: "Validate paths. Use path.join() and path.normalize()",
        confidence: "medium",
        languages: Some(&["js", "ts"]),
    },
];

/// Cryptography patterns.
pub const CRYPTO_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "Weak Cryptography - MD5",
        pattern: r"\b(md5|MD5)\s*\(",
        severity: SeverityLevel::Medium,
        cwe_id: "CWE-327",
        description: "MD5 is cryptographically broken",
        remediation: "Use SHA-256 or stronger hashing algorithms",
        confidence: "high",
        languages: None,
    },
    VulnerabilityPattern {
        name: "Weak Cryptography - SHA1",
        pattern: r"\b(sha1|SHA1|SHA-1)\s*\(",
        severity: SeverityLevel::Medium,
        cwe_id: "CWE-327",
        description: "SHA-1 is deprecated and vulnerable to collisions",
        remediation: "Use SHA-256, SHA-384, or SHA-512",
        confidence: "high",
        languages: None,
    },
];

/// Deserialization patterns.
pub const DESERIALIZATION_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "Insecure Deserialization - pickle",
        pattern: r"pickle\.loads?\s*\(",
        severity: SeverityLevel::Critical,
        cwe_id: "CWE-502",
        description: "Pickle deserialization can lead to arbitrary code execution",
        remediation: "Use JSON or safer formats. Never unpickle untrusted data",
        confidence: "high",
        languages: Some(&["py"]),
    },
    VulnerabilityPattern {
        name: "Insecure Deserialization - YAML",
        pattern: r"yaml\.load\s*\([^)]*\)",
        severity: SeverityLevel::High,
        cwe_id: "CWE-502",
        description: "Unsafe YAML loading can execute arbitrary code",
        remediation: "Use yaml.safe_load() instead of yaml.load()",
        confidence: "medium",
        languages: Some(&["py"]),
    },
];

/// Info disclosure patterns.
pub const INFO_DISCLOSURE_PATTERNS: &[VulnerabilityPattern] = &[
    VulnerabilityPattern {
        name: "Debug Mode Enabled",
        pattern: r"(DEBUG|debug)\s*=\s*True",
        severity: SeverityLevel::Medium,
        cwe_id: "CWE-215",
        description: "Debug mode may expose sensitive information",
        remediation: "Disable debug mode in production",
        confidence: "medium",
        languages: Some(&["py"]),
    },
    VulnerabilityPattern {
        name: "Sensitive Info in Logs",
        pattern: r"(log|logger|print|console\.log)\s*\([^)]*\b(password|token|secret|key|credential)\b",
        severity: SeverityLevel::Low,
        cwe_id: "CWE-532",
        description: "Potentially sensitive information in logs",
        remediation: "Sanitize log output. Never log passwords or secrets",
        confidence: "low",
        languages: None,
    },
];

/// Code vulnerability analyzer.
pub struct CodeAnalyzer<'a> {
    scanner: &'a SecurityScanner,
    compiled_patterns: Vec<(Regex, &'static VulnerabilityPattern)>,
}

impl<'a> CodeAnalyzer<'a> {
    /// Create a new code analyzer.
    pub fn new(scanner: &'a SecurityScanner) -> Self {
        let config = scanner.config();
        let mut patterns: Vec<&VulnerabilityPattern> = Vec::new();

        if config.check_sql_injection {
            patterns.extend(SQL_INJECTION_PATTERNS);
        }
        if config.check_xss {
            patterns.extend(XSS_PATTERNS);
        }
        if config.check_command_injection {
            patterns.extend(COMMAND_INJECTION_PATTERNS);
        }
        if config.check_path_traversal {
            patterns.extend(PATH_TRAVERSAL_PATTERNS);
        }

        // Always check these
        patterns.extend(CRYPTO_PATTERNS);
        patterns.extend(DESERIALIZATION_PATTERNS);
        patterns.extend(INFO_DISCLOSURE_PATTERNS);

        let compiled_patterns: Vec<_> = patterns
            .into_iter()
            .filter_map(|p| Regex::new(p.pattern).ok().map(|r| (r, p)))
            .collect();

        Self {
            scanner,
            compiled_patterns,
        }
    }

    /// Get file language from extension.
    fn get_language(path: &Path) -> Option<String> {
        path.extension().map(|e| e.to_string_lossy().to_lowercase())
    }

    /// Check if pattern applies to this file.
    fn should_check(pattern: &VulnerabilityPattern, lang: Option<&str>) -> bool {
        match pattern.languages {
            None => true,
            Some(langs) => lang.is_some_and(|l| langs.contains(&l)),
        }
    }

    /// Analyze a single file.
    pub fn analyze_file(&self, file_path: &Path) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let lang = Self::get_language(file_path);
        let lang_ref = lang.as_deref();

        let lines = match self.scanner.read_file_lines(file_path) {
            Some(l) => l,
            None => return issues,
        };

        let relative_path = file_path
            .strip_prefix(self.scanner.root_path())
            .unwrap_or(file_path);

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;

            for (regex, pattern) in &self.compiled_patterns {
                if !Self::should_check(pattern, lang_ref) {
                    continue;
                }

                if regex.is_match(line) {
                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "language".to_string(),
                        lang_ref.unwrap_or("unknown").to_string(),
                    );

                    issues.push(SecurityIssue {
                        severity: pattern.severity,
                        category: "code_security".to_string(),
                        title: pattern.name.to_string(),
                        description: pattern.description.to_string(),
                        file_path: relative_path.to_path_buf(),
                        line_number: Some(line_num),
                        code_snippet: Some(line.chars().take(200).collect()),
                        remediation: Some(pattern.remediation.to_string()),
                        cwe_id: Some(pattern.cwe_id.to_string()),
                        confidence: pattern.confidence.to_string(),
                        tov_signal: None,
                        metadata,
                    });
                }
            }
        }

        issues
    }

    /// Run code vulnerability analysis.
    pub fn detect(&self) -> Vec<SecurityIssue> {
        if !self.scanner.config().analyze_code {
            return Vec::new();
        }

        let files = self.scanner.iter_files();
        files.iter().flat_map(|f| self.analyze_file(f)).collect()
    }
}
