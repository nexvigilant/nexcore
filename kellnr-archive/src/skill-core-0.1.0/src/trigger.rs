//! Trigger patterns for skill activation

use serde::{Deserialize, Serialize};

/// Trigger pattern that activates a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    /// Slash command (e.g., "/extract")
    Command(String),
    /// Regex pattern match on user input
    Pattern(String),
    /// Keyword presence
    Keyword(String),
    /// Always active (background skill)
    Always,
}

impl Trigger {
    /// Create a command trigger
    pub fn command(cmd: &str) -> Self {
        Self::Command(cmd.to_string())
    }

    /// Create a pattern trigger
    pub fn pattern(pat: &str) -> Self {
        Self::Pattern(pat.to_string())
    }

    /// Create a keyword trigger
    pub fn keyword(kw: &str) -> Self {
        Self::Keyword(kw.to_string())
    }

    /// Check if trigger matches input
    pub fn matches(&self, input: &str) -> bool {
        match self {
            Self::Command(cmd) => input.starts_with(cmd),
            Self::Pattern(pat) => match_pattern(pat, input),
            Self::Keyword(kw) => input.to_lowercase().contains(&kw.to_lowercase()),
            Self::Always => true,
        }
    }
}

fn match_pattern(pat: &str, input: &str) -> bool {
    regex::Regex::new(pat)
        .map(|r| r.is_match(input))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_trigger() {
        let cmd = Trigger::command("/test");
        assert!(cmd.matches("/test foo bar"));
        assert!(!cmd.matches("test foo"));
    }

    #[test]
    fn test_keyword_trigger() {
        let kw = Trigger::keyword("extract");
        assert!(kw.matches("please extract primitives"));
        assert!(!kw.matches("please retract"));
    }
}
