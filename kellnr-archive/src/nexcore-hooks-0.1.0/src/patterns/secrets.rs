//! Secret detection patterns.
//!
//! Centralized patterns for detecting credentials, API keys, and secrets.
//! Used by `pretool_secret_scanner` and any other security-related hooks.

use once_cell::sync::Lazy;
use regex::Regex;

use super::safe_regex;

/// Keywords that suggest potential secrets (case-insensitive matching)
pub const SECRET_KEYWORDS: &[&str] = &[
    "api_key",
    "apikey",
    "api-key",
    "secret",
    "password",
    "passwd",
    "token",
    "private_key",
    "privatekey",
    "secret_key",
    "secretkey",
    "access_key",
    "accesskey",
    "auth_token",
    "bearer",
    "credential",
];

/// Prefixes that indicate cloud provider credentials
pub const CLOUD_PREFIXES: &[&str] = &[
    "aws_",     // AWS credentials
    "AKIA",     // AWS access key IDs
    "ASIA",     // AWS temporary credentials
    "ghp_",     // GitHub personal access token
    "gho_",     // GitHub OAuth token
    "ghu_",     // GitHub user-to-server token
    "ghs_",     // GitHub server-to-server token
    "ghr_",     // GitHub refresh token
    "sk-",      // OpenAI API keys
    "pk_live_", // Stripe publishable key (live)
    "sk_live_", // Stripe secret key (live)
    "rk_live_", // Stripe restricted key (live)
    "xoxb-",    // Slack bot token
    "xoxp-",    // Slack user token
    "xoxa-",    // Slack app token
    "xoxr-",    // Slack refresh token
    "AIza",     // Google API key
    "ya29.",    // Google OAuth token
    "GOCSPX-",  // Google OAuth client secret
];

/// AWS Access Key ID pattern (AKIA followed by 16 alphanumeric chars)
pub static AWS_KEY: Lazy<Regex> = Lazy::new(|| safe_regex(r"(?:AKIA|ASIA)[A-Z0-9]{16}"));

/// GitHub Personal Access Token pattern (ghp_ followed by 36 chars)
pub static GITHUB_PAT: Lazy<Regex> = Lazy::new(|| safe_regex(r"gh[pousr]_[A-Za-z0-9]{36,}"));

/// OpenAI API Key pattern (sk- followed by 48+ chars)
pub static OPENAI_KEY: Lazy<Regex> = Lazy::new(|| safe_regex(r"sk-[A-Za-z0-9]{48,}"));

/// Stripe API Key pattern
pub static STRIPE_KEY: Lazy<Regex> =
    Lazy::new(|| safe_regex(r"(?:sk|pk|rk)_(?:live|test)_[A-Za-z0-9]{24,}"));

/// Slack Token pattern
pub static SLACK_TOKEN: Lazy<Regex> = Lazy::new(|| safe_regex(r"xox[bpasru]-[A-Za-z0-9\-]{10,}"));

/// Google API Key pattern
pub static GOOGLE_KEY: Lazy<Regex> = Lazy::new(|| safe_regex(r"AIza[A-Za-z0-9\-_]{35}"));

/// Generic high-entropy string (potential secret value)
pub static HIGH_ENTROPY: Lazy<Regex> = Lazy::new(|| safe_regex(r#"["'][A-Za-z0-9+/=]{32,}["']"#));

/// Quick heuristic check for potential secrets (fast, low false-negative rate)
///
/// Returns true if content might contain secrets and warrants deeper analysis.
pub fn might_contain_secrets(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();

    // Check keywords
    for keyword in SECRET_KEYWORDS {
        if lower.contains(keyword) {
            return true;
        }
    }

    // Check cloud prefixes (case-sensitive for some)
    for prefix in CLOUD_PREFIXES {
        if content.contains(prefix) {
            return true;
        }
    }

    false
}

/// Deep analysis for actual secret patterns (slower, precise)
///
/// Returns a list of detected secrets with their type.
pub fn detect_secrets(content: &str) -> Vec<SecretMatch> {
    let mut matches = Vec::new();

    // Check each pattern
    for mat in AWS_KEY.find_iter(content) {
        matches.push(SecretMatch {
            secret_type: SecretType::AwsKey,
            value: mat.as_str().to_string(),
            start: mat.start(),
            end: mat.end(),
        });
    }

    for mat in GITHUB_PAT.find_iter(content) {
        matches.push(SecretMatch {
            secret_type: SecretType::GitHubToken,
            value: mat.as_str().to_string(),
            start: mat.start(),
            end: mat.end(),
        });
    }

    for mat in OPENAI_KEY.find_iter(content) {
        matches.push(SecretMatch {
            secret_type: SecretType::OpenAiKey,
            value: mat.as_str().to_string(),
            start: mat.start(),
            end: mat.end(),
        });
    }

    for mat in STRIPE_KEY.find_iter(content) {
        matches.push(SecretMatch {
            secret_type: SecretType::StripeKey,
            value: mat.as_str().to_string(),
            start: mat.start(),
            end: mat.end(),
        });
    }

    for mat in SLACK_TOKEN.find_iter(content) {
        matches.push(SecretMatch {
            secret_type: SecretType::SlackToken,
            value: mat.as_str().to_string(),
            start: mat.start(),
            end: mat.end(),
        });
    }

    for mat in GOOGLE_KEY.find_iter(content) {
        matches.push(SecretMatch {
            secret_type: SecretType::GoogleKey,
            value: mat.as_str().to_string(),
            start: mat.start(),
            end: mat.end(),
        });
    }

    // Check for generic key=value patterns with long values
    for line in content.lines() {
        let line_lower = line.to_ascii_lowercase();
        if SECRET_KEYWORDS.iter().any(|k| line_lower.contains(k)) {
            if let Some(eq_pos) = line.find('=') {
                let value_part = &line[eq_pos + 1..];
                let alphanum_count = value_part.chars().filter(|c| c.is_alphanumeric()).count();
                if alphanum_count > 20 {
                    matches.push(SecretMatch {
                        secret_type: SecretType::GenericSecret,
                        value: "[REDACTED]".to_string(),
                        start: 0,
                        end: line.len(),
                    });
                }
            }
        }
    }

    matches
}

/// Type of secret detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretType {
    AwsKey,
    GitHubToken,
    OpenAiKey,
    StripeKey,
    SlackToken,
    GoogleKey,
    GenericSecret,
}

impl SecretType {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::AwsKey => "AWS Access Key",
            Self::GitHubToken => "GitHub Token",
            Self::OpenAiKey => "OpenAI API Key",
            Self::StripeKey => "Stripe API Key",
            Self::SlackToken => "Slack Token",
            Self::GoogleKey => "Google API Key",
            Self::GenericSecret => "Generic Secret",
        }
    }

    /// Get remediation suggestion
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::AwsKey => "Use AWS_ACCESS_KEY_ID environment variable",
            Self::GitHubToken => "Use GITHUB_TOKEN environment variable",
            Self::OpenAiKey => "Use OPENAI_API_KEY environment variable",
            Self::StripeKey => "Use STRIPE_SECRET_KEY environment variable",
            Self::SlackToken => "Use SLACK_TOKEN environment variable",
            Self::GoogleKey => "Use GOOGLE_API_KEY environment variable",
            Self::GenericSecret => "Store in environment variable or secrets manager",
        }
    }
}

/// A detected secret match
#[derive(Debug, Clone)]
pub struct SecretMatch {
    pub secret_type: SecretType,
    pub value: String, // Redacted or partial
    pub start: usize,
    pub end: usize,
}

impl SecretMatch {
    /// Get a safe display string (redacted)
    pub fn redacted_value(&self) -> String {
        if self.value.len() <= 8 {
            "[REDACTED]".to_string()
        } else {
            format!(
                "{}...{}",
                &self.value[..4],
                &self.value[self.value.len() - 4..]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_might_contain_secrets() {
        assert!(might_contain_secrets("api_key=abc123"));
        assert!(might_contain_secrets("AKIA1234567890ABCDEF"));
        assert!(might_contain_secrets(
            "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        ));
        assert!(!might_contain_secrets("hello world"));
        assert!(!might_contain_secrets("fn main() {}"));
    }

    #[test]
    fn test_aws_key_detection() {
        let content = "aws_access_key_id=AKIAIOSFODNN7EXAMPLE";
        assert!(AWS_KEY.is_match(content));

        let matches = detect_secrets(content);
        assert!(matches.iter().any(|m| m.secret_type == SecretType::AwsKey));
    }

    #[test]
    fn test_github_pat_detection() {
        let content = "token: ghp_1234567890abcdefghijklmnopqrstuvwxyz";
        assert!(GITHUB_PAT.is_match(content));

        let matches = detect_secrets(content);
        assert!(
            matches
                .iter()
                .any(|m| m.secret_type == SecretType::GitHubToken)
        );
    }

    #[test]
    fn test_openai_key_detection() {
        let content = "OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        assert!(OPENAI_KEY.is_match(content));

        let matches = detect_secrets(content);
        assert!(
            matches
                .iter()
                .any(|m| m.secret_type == SecretType::OpenAiKey)
        );
    }

    #[test]
    fn test_no_false_positives() {
        // Should not match normal code
        let content = r#"
            fn main() {
                let x = "hello";
                println!("{}", x);
            }
        "#;
        assert!(!might_contain_secrets(content));
        assert!(detect_secrets(content).is_empty());
    }

    #[test]
    fn test_secret_type_names() {
        assert_eq!(SecretType::AwsKey.name(), "AWS Access Key");
        assert_eq!(SecretType::GitHubToken.name(), "GitHub Token");
    }
}
