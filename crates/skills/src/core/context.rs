//! Skill execution context

use std::collections::HashMap;

/// Context provided to skill during execution
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// Raw user input
    pub input: String,
    /// Parsed arguments (if command-style)
    pub args: Vec<String>,
    /// Key-value parameters
    pub params: HashMap<String, String>,
    /// Current working directory
    pub cwd: String,
    /// Session ID (if available)
    pub session_id: Option<String>,
}

impl SkillContext {
    /// Create new context from user input
    pub fn new(input: impl Into<String>) -> Self {
        let input = input.into();
        let args = parse_args(&input);
        let cwd = get_cwd();

        Self {
            input,
            args,
            params: HashMap::new(),
            cwd,
            session_id: None,
        }
    }

    /// Add a parameter
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

fn parse_args(input: &str) -> Vec<String> {
    input.split_whitespace().skip(1).map(String::from).collect()
}

fn get_cwd() -> String {
    std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = SkillContext::new("/test arg1 arg2");
        assert_eq!(ctx.args, vec!["arg1", "arg2"]);
    }
}
