//! Learned user preferences
//!
//! Preferences are key-value pairs with confidence levels that
//! increase through reinforcement and decrease through weakening.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A learned user preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    /// Preference key (e.g., "`code_style`", "`commit_format`")
    pub key: String,

    /// Preference value
    pub value: serde_json::Value,

    /// Optional description of what this preference means
    pub description: Option<String>,

    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,

    /// When this preference was last updated
    pub updated_at: DateTime<Utc>,

    /// Number of times this preference was reinforced
    pub reinforcement_count: u32,
}

impl Preference {
    /// Create a new preference
    #[must_use]
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            value,
            description: None,
            confidence: 0.5, // Start neutral
            updated_at: Utc::now(),
            reinforcement_count: 1,
        }
    }

    /// Reinforce this preference (increase confidence)
    pub fn reinforce(&mut self) {
        self.reinforcement_count += 1;
        // Asymptotically approach 1.0
        self.confidence = 1.0 - (1.0 / (f64::from(self.reinforcement_count) + 1.0));
        self.updated_at = Utc::now();
    }

    /// Weaken this preference (decrease confidence)
    pub fn weaken(&mut self) {
        if self.reinforcement_count > 0 {
            self.reinforcement_count = self.reinforcement_count.saturating_sub(1);
        }
        self.confidence = if self.reinforcement_count == 0 {
            0.0
        } else {
            1.0 - (1.0 / (f64::from(self.reinforcement_count) + 1.0))
        };
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_reinforcement() {
        let mut pref = Preference::new("test", serde_json::json!("value"));
        assert_eq!(pref.confidence, 0.5);
        assert_eq!(pref.reinforcement_count, 1);

        pref.reinforce();
        assert!(pref.confidence > 0.5);
        assert_eq!(pref.reinforcement_count, 2);

        // Reinforce many times
        for _ in 0..10 {
            pref.reinforce();
        }
        assert!(pref.confidence > 0.9);
        assert!(pref.confidence < 1.0); // Never quite reaches 1.0
    }

    #[test]
    fn test_preference_weakening() {
        let mut pref = Preference::new("test", serde_json::json!("value"));
        pref.reinforce();
        pref.reinforce();
        assert_eq!(pref.reinforcement_count, 3);

        pref.weaken();
        assert_eq!(pref.reinforcement_count, 2);

        pref.weaken();
        pref.weaken();
        pref.weaken(); // Can't go below 0
        assert_eq!(pref.reinforcement_count, 0);
        assert_eq!(pref.confidence, 0.0);
    }

    #[test]
    fn test_preference_unicode_key_and_value() {
        let pref = Preference::new("编程风格", serde_json::json!("函数式 🦀"));
        assert_eq!(pref.key, "编程风格");
        assert_eq!(pref.value, serde_json::json!("函数式 🦀"));
    }

    #[test]
    fn test_preference_complex_json_value() {
        let complex_value = serde_json::json!({
            "style": "functional",
            "prefer_async": true,
            "max_line_length": 100,
            "languages": ["rust", "typescript"]
        });

        let pref = Preference::new("code_style", complex_value.clone());
        assert_eq!(pref.value, complex_value);
    }

    #[test]
    fn test_preference_extreme_reinforcement() {
        let mut pref = Preference::new("test", serde_json::json!(true));

        // Reinforce 10,000 times
        for _ in 0..10_000 {
            pref.reinforce();
        }

        // Confidence should be very close to 1.0 but never reach it
        assert!(pref.confidence > 0.9999);
        assert!(pref.confidence < 1.0);
        assert_eq!(pref.reinforcement_count, 10_001);
    }

    #[test]
    fn test_preference_alternating_reinforce_weaken() {
        let mut pref = Preference::new("test", serde_json::json!("value"));

        // Alternate reinforcement and weakening
        for _ in 0..100 {
            pref.reinforce();
            pref.weaken();
        }

        // Should end up at initial count since we alternate
        assert_eq!(pref.reinforcement_count, 1);
        assert_eq!(pref.confidence, 0.5);
    }

    #[test]
    fn test_preference_description() {
        let mut pref = Preference::new("indent_style", serde_json::json!("tabs"));
        pref.description = Some("User prefers tabs for indentation".into());

        assert_eq!(
            pref.description,
            Some("User prefers tabs for indentation".into())
        );
    }

    #[test]
    fn test_preference_weaken_from_zero() {
        let mut pref = Preference::new("test", serde_json::json!("value"));

        // Weaken until count is 0
        pref.weaken();

        // Weaken again from 0 - should not go negative
        pref.weaken();
        pref.weaken();
        pref.weaken();

        assert_eq!(pref.reinforcement_count, 0);
        assert_eq!(pref.confidence, 0.0);
    }

    #[test]
    fn test_preference_null_json_value() {
        let pref = Preference::new("optional_setting", serde_json::Value::Null);
        assert!(pref.value.is_null());
    }

    #[test]
    fn test_preference_array_json_value() {
        let pref = Preference::new(
            "favorite_crates",
            serde_json::json!(["serde", "tokio", "axum", "thiserror"]),
        );

        assert!(pref.value.is_array());
        assert_eq!(pref.value.as_array().unwrap().len(), 4);
    }
}
