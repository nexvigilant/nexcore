//! Learned corrections from user feedback
//!
//! Corrections record mistakes and their fixes, with application tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A learned correction from user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    /// What was wrong (the mistake)
    #[serde(alias = "original")]
    pub mistake: String,

    /// What should have been done (the correction)
    #[serde(alias = "corrected")]
    pub correction: String,

    /// Context when this occurred
    #[serde(alias = "reason")]
    pub context: Option<String>,

    /// When this correction was learned
    #[serde(default = "Utc::now")]
    pub learned_at: DateTime<Utc>,

    /// Number of times this correction was applied
    #[serde(default)]
    pub application_count: u32,
}

impl Correction {
    /// Create a new correction
    #[must_use]
    pub fn new(mistake: impl Into<String>, correction: impl Into<String>) -> Self {
        Self {
            mistake: mistake.into(),
            correction: correction.into(),
            context: None,
            learned_at: Utc::now(),
            application_count: 0,
        }
    }

    /// Record that this correction was applied
    pub fn mark_applied(&mut self) {
        self.application_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction() {
        let mut correction = Correction::new("used unwrap", "use expect with message");
        assert_eq!(correction.application_count, 0);

        correction.mark_applied();
        correction.mark_applied();
        assert_eq!(correction.application_count, 2);
    }

    #[test]
    fn test_correction_with_context() {
        let mut correction = Correction::new("unwrap()", "expect() with message");
        correction.context = Some("In error handling code".into());

        assert_eq!(correction.context, Some("In error handling code".into()));
    }

    #[test]
    fn test_correction_unicode() {
        let correction = Correction::new("使用了 unwrap", "应该使用 expect 并添加错误消息");

        assert_eq!(correction.mistake, "使用了 unwrap");
        assert_eq!(correction.correction, "应该使用 expect 并添加错误消息");
    }
}
