//! Secret & Credential Scanner (HOOK-M5-49)
//!
//! Prevents secrets, credentials, and sensitive data from entering the codebase.
//! Blocks writes containing API keys, passwords, tokens, or PII patterns.

use regex::Regex;
use std::sync::LazyLock;

/// Severity of a secret finding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl SecretSeverity {
    /// Returns the string representation of the severity level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
        }
    }

    /// Returns true if this severity level should block the operation.
    pub fn should_block(&self) -> bool {
        matches!(self, Self::Critical | Self::High)
    }
}

/// Category of secret pattern
#[derive(Debug, Clone, Copy)]
pub enum SecretCategory {
    ApiKey,
    Password,
    Token,
    PrivateKey,
    Pii,
    HighEntropy,
}

impl SecretCategory {
    /// Returns the human-readable name of this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "API Key",
            Self::Password => "Password",
            Self::Token => "Token",
            Self::PrivateKey => "Private Key",
            Self::Pii => "PII",
            Self::HighEntropy => "High Entropy String",
        }
    }
}

/// A detected secret in the code
#[derive(Debug, Clone)]
pub struct SecretFinding {
    /// Name of the pattern that matched
    pub pattern_name: &'static str,
    /// Category of the secret
    pub category: SecretCategory,
    /// Severity level
    pub severity: SecretSeverity,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Redacted preview of the matched value
    pub value_preview: String,
    /// Context around the finding
    pub context: String,
    /// Suggested fix
    pub fix_suggestion: &'static str,
}

/// Compile a regex pattern, exiting on failure (programming bug).
fn compile_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FATAL: Invalid regex pattern: {}", e);
            std::process::exit(1);
        }
    }
}

// Quote character class: matches ' or "
// Use non-raw string with proper escapes
const QUOTE_CLASS: &str = r#"['""]"#;

// Build patterns at runtime to avoid self-detection
static AWS_ACCESS_KEY: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"AKIA[0-9A-Z]{16}"));

static AWS_SECRET_KEY: LazyLock<Regex> = LazyLock::new(|| {
    // AWS secret with quotes around it
    compile_regex(&format!(
        r"(?i)aws(.{{0,20}})?{q}[0-9a-zA-Z/+]{{40}}{q}",
        q = QUOTE_CLASS
    ))
});

static GITHUB_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(&format!("gh[pousr]_[A-Za-z0-9]{{36,}}")));

static GITHUB_PAT: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(&format!("ghp_[A-Za-z0-9]{{36}}")));

static GOOGLE_API_KEY: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"AIza[0-9A-Za-z\-_]{35}"));

static SLACK_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(&format!(
        "xox[baprs]-[0-9]{{10,13}}-[0-9]{{10,13}}[a-zA-Z0-9-]*"
    ))
});

static STRIPE_KEY: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(&format!("sk_live_[0-9a-zA-Z]{{24,}}")));

static GENERIC_API_KEY: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(&format!(
        r"(?i)(api[_\-]?key|apikey){q}?\s*[:=]\s*{q}[a-zA-Z0-9]{{20,}}{q}",
        q = QUOTE_CLASS
    ))
});

static PASSWORD: LazyLock<Regex> = LazyLock::new(|| {
    // Match password assignments with quoted values
    compile_regex(r#"(?i)(password|passwd|pwd)['"]?\s*[:=]\s*['"].{8,}['"]"#)
});

static DB_URL: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)(mongodb|postgres|mysql|redis)://[^:]+:[^@]+@"));

static JWT: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+"));

static BEARER: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"(?i)bearer\s+[a-zA-Z0-9\-_.]+"));

static RSA_KEY: LazyLock<Regex> =
    LazyLock::new(|| compile_regex("-----BEGIN RSA PRIVATE KEY-----"));
static SSH_KEY: LazyLock<Regex> =
    LazyLock::new(|| compile_regex("-----BEGIN OPENSSH PRIVATE KEY-----"));
static PGP_KEY: LazyLock<Regex> =
    LazyLock::new(|| compile_regex("-----BEGIN PGP PRIVATE KEY BLOCK-----"));
static SSN: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"\b\d{3}-\d{2}-\d{4}\b"));

static CREDIT_CARD: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})\b")
});

static HIGH_ENTROPY: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(&format!(r"{q}[a-zA-Z0-9+/=]{{32,}}{q}", q = QUOTE_CLASS)));

/// Calculate Shannon entropy of a string.
///
/// # Arguments
/// * `s` - The string to analyze
///
/// # Returns
/// Entropy in bits per character (0.0 to ~8.0 for ASCII)
pub fn calculate_shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut freq = [0u32; 256];
    for b in s.bytes() {
        freq[b as usize] += 1;
    }
    let len = s.len() as f64;
    freq.iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// Validate a number using the Luhn algorithm.
///
/// # Arguments
/// * `number` - String containing digits to validate
///
/// # Returns
/// `true` if the number passes the Luhn checksum
pub fn passes_luhn(number: &str) -> bool {
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let x = d * 2;
                if x > 9 { x - 9 } else { x }
            } else {
                d
            }
        })
        .sum();
    sum % 10 == 0
}

/// Redact a secret value for safe display.
///
/// # Arguments
/// * `value` - The secret value to redact
///
/// # Returns
/// Redacted string with asterisks
pub fn redact_secret(value: &str) -> String {
    if value.len() <= 12 {
        "*".repeat(value.len())
    } else {
        format!(
            "{}{}{}",
            &value[..4],
            "*".repeat(value.len().saturating_sub(8)),
            &value[value.len().saturating_sub(4)..]
        )
    }
}

fn skip_file(p: &str) -> bool {
    [".png", ".jpg", ".gif", ".ico", ".woff"]
        .iter()
        .any(|e| p.ends_with(e))
}

fn is_fixture(p: &str) -> bool {
    let l = p.to_lowercase();
    (l.contains("/test") || l.contains("_test")) && l.contains("fixture")
}

fn context(c: &str, l: usize, n: usize) -> String {
    let v: Vec<_> = c.lines().collect();
    v[l.saturating_sub(n)..(l + n).min(v.len())].join("\n")
}

/// Scan file content for secrets and credentials.
///
/// # Arguments
/// * `file_path` - Path to the file (used for fixture detection)
/// * `content` - File content to scan
///
/// # Returns
/// Vector of detected secret findings
pub fn scan_for_secrets(file_path: &str, content: &str) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    if skip_file(file_path) {
        return findings;
    }
    let fixture = is_fixture(file_path);
    let lines: Vec<_> = content.lines().collect();

    type PatternDef = (
        &'static LazyLock<Regex>,
        &'static str,
        SecretCategory,
        SecretSeverity,
        bool,
        &'static [&'static str],
        &'static str,
    );

    let pats: &[PatternDef] = &[
        (
            &AWS_ACCESS_KEY,
            "AWS Access Key",
            SecretCategory::ApiKey,
            SecretSeverity::Critical,
            false,
            &[],
            "Use env::var(\"AWS_ACCESS_KEY_ID\")",
        ),
        (
            &AWS_SECRET_KEY,
            "AWS Secret Key",
            SecretCategory::ApiKey,
            SecretSeverity::Critical,
            true,
            &[],
            "Use env::var(\"AWS_SECRET_ACCESS_KEY\")",
        ),
        (
            &GITHUB_TOKEN,
            "GitHub Token",
            SecretCategory::Token,
            SecretSeverity::Critical,
            false,
            &[],
            "Use env::var(\"GITHUB_TOKEN\")",
        ),
        (
            &GITHUB_PAT,
            "GitHub PAT",
            SecretCategory::Token,
            SecretSeverity::Critical,
            false,
            &[],
            "Use env::var(\"GITHUB_TOKEN\")",
        ),
        (
            &GOOGLE_API_KEY,
            "Google API Key",
            SecretCategory::ApiKey,
            SecretSeverity::Critical,
            false,
            &[],
            "Use env::var(\"GOOGLE_API_KEY\")",
        ),
        (
            &SLACK_TOKEN,
            "Slack Token",
            SecretCategory::Token,
            SecretSeverity::Critical,
            false,
            &[],
            "Use env::var(\"SLACK_TOKEN\")",
        ),
        (
            &STRIPE_KEY,
            "Stripe Key",
            SecretCategory::ApiKey,
            SecretSeverity::Critical,
            false,
            &[],
            "Use env::var(\"STRIPE_SECRET_KEY\")",
        ),
        (
            &GENERIC_API_KEY,
            "API Key",
            SecretCategory::ApiKey,
            SecretSeverity::High,
            true,
            &[],
            "Use environment variables",
        ),
        (
            &PASSWORD,
            "Password",
            SecretCategory::Password,
            SecretSeverity::Critical,
            true,
            &["password123", "changeme", "placeholder"],
            "Use secret manager",
        ),
        (
            &DB_URL,
            "Database URL",
            SecretCategory::Password,
            SecretSeverity::Critical,
            false,
            &["localhost", "127.0.0.1"],
            "Use env::var(\"DATABASE_URL\")",
        ),
        (
            &JWT,
            "JWT Token",
            SecretCategory::Token,
            SecretSeverity::High,
            false,
            &[],
            "Load tokens dynamically",
        ),
        (
            &BEARER,
            "Bearer Token",
            SecretCategory::Token,
            SecretSeverity::High,
            true,
            &[],
            "Use secure storage",
        ),
        (
            &RSA_KEY,
            "RSA Private Key",
            SecretCategory::PrivateKey,
            SecretSeverity::Critical,
            false,
            &[],
            "Never commit private keys",
        ),
        (
            &SSH_KEY,
            "SSH Private Key",
            SecretCategory::PrivateKey,
            SecretSeverity::Critical,
            false,
            &[],
            "Never commit private keys",
        ),
        (
            &PGP_KEY,
            "PGP Private Key",
            SecretCategory::PrivateKey,
            SecretSeverity::Critical,
            false,
            &[],
            "Never commit private keys",
        ),
        (
            &SSN,
            "SSN",
            SecretCategory::Pii,
            SecretSeverity::Critical,
            false,
            &[],
            "Never store SSN",
        ),
        (
            &CREDIT_CARD,
            "Credit Card",
            SecretCategory::Pii,
            SecretSeverity::Critical,
            false,
            &[],
            "Never store credit cards",
        ),
    ];

    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("//") || line.trim().starts_with("#") {
            continue;
        }
        for (re, name, cat, sev, ent, exc, fix) in pats.iter() {
            if let Some(m) = re.find(line) {
                let v: &str = m.as_str();
                if *ent && calculate_shannon_entropy(v) < 3.5 {
                    continue;
                }
                if exc.iter().any(|e| v.to_lowercase().contains(e)) {
                    continue;
                }
                if matches!(cat, SecretCategory::Pii) && name.contains("Credit") {
                    let digits: String = v.chars().filter(|c: &char| c.is_ascii_digit()).collect();
                    if !passes_luhn(&digits) {
                        continue;
                    }
                }
                if fixture {
                    continue;
                }
                findings.push(SecretFinding {
                    pattern_name: name,
                    category: cat.clone(),
                    severity: sev.clone(),
                    line: i + 1,
                    column: m.start() + 1,
                    value_preview: redact_secret(v),
                    context: context(content, i, 2),
                    fix_suggestion: fix,
                });
            }
        }
    }

    // High-entropy string check
    for (i, line) in lines.iter().enumerate() {
        if findings.iter().any(|f| f.line == i + 1) {
            continue;
        }
        for m in HIGH_ENTROPY.find_iter(line) {
            let v = m.as_str();
            if v.len() < 34 {
                continue;
            }
            let inner = &v[1..v.len() - 1];
            if calculate_shannon_entropy(inner) > 4.5 {
                findings.push(SecretFinding {
                    pattern_name: "High-Entropy",
                    category: SecretCategory::HighEntropy,
                    severity: SecretSeverity::Medium,
                    line: i + 1,
                    column: m.start() + 1,
                    value_preview: redact_secret(inner),
                    context: context(content, i, 1),
                    fix_suggestion: "Review if secret",
                });
            }
        }
    }
    findings
}

/// Format secret findings for terminal output.
///
/// # Arguments
/// * `file_path` - Path to the scanned file
/// * `findings` - Vector of detected secrets
///
/// # Returns
/// Formatted string with a visual report
pub fn format_secret_findings(file_path: &str, findings: &[SecretFinding]) -> String {
    if findings.is_empty() {
        return String::new();
    }
    let c = findings
        .iter()
        .filter(|f| f.severity == SecretSeverity::Critical)
        .count();
    let h = findings
        .iter()
        .filter(|f| f.severity == SecretSeverity::High)
        .count();
    let mut o = String::new();
    o.push_str("┌─────────────────────────────────────────────────────────────────────────────┐\n");
    o.push_str("│ ⛔ SECRET DETECTED                                                          │\n");
    o.push_str("├─────────────────────────────────────────────────────────────────────────────┤\n");
    let display_path = if file_path.len() > 70 {
        &file_path[..70]
    } else {
        file_path
    };
    o.push_str(&format!("│ File: {:<70} │\n", display_path));
    for f in findings {
        let icon = if f.severity.should_block() {
            "⛔"
        } else {
            "⚠"
        };
        o.push_str(&format!("│ {} {:<72} │\n", icon, f.pattern_name));
        let preview = if f.value_preview.len() > 66 {
            &f.value_preview[..66]
        } else {
            &f.value_preview
        };
        o.push_str(&format!("│   Line {}: {:<66} │\n", f.line, preview));
    }
    o.push_str(&format!(
        "│ ⛔ BLOCKING: {} critical, {} high                                            │\n",
        c, h
    ));
    o.push_str("└─────────────────────────────────────────────────────────────────────────────┘\n");
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_entropy() {
        assert!(calculate_shannon_entropy("aaaa") < 1.0);
    }
    #[test]
    fn test_luhn() {
        assert!(passes_luhn("4111111111111111"));
    }
    #[test]
    fn test_redact() {
        assert_eq!(redact_secret("short"), "*****");
    }
}
