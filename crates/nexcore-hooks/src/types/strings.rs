//! Validated string types eliminating primitive obsession.
//!
//! # Design Principle
//!
//! Instead of validating strings at usage sites, we validate at construction,
//! making invalid states unrepresentable.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::error::{HookError, HookResult};

/// A string guaranteed to be non-empty.
///
/// # Invariant
/// `self.0.len() > 0` — enforced at construction, immutable thereafter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Create a new NonEmptyString, validating non-emptiness.
    pub fn new(s: impl Into<String>) -> HookResult<Self> {
        let s = s.into();
        if s.is_empty() {
            Err(HookError::ValidationFailed("string cannot be empty".into()))
        } else {
            Ok(Self(s))
        }
    }

    /// Create without validation. Caller guarantees non-emptiness.
    ///
    /// # Safety (logical, not memory)
    /// Only use when source is known non-empty (e.g., string literals).
    #[inline]
    #[must_use]
    pub const fn new_unchecked(s: String) -> Self {
        Self(s)
    }

    /// Access inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return inner String.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Length in bytes (guaranteed > 0).
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always returns false (string is guaranteed non-empty).
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NonEmptyString::new(s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for NonEmptyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// A string with maximum length constraint.
///
/// # Invariant
/// `self.0.len() <= N` — enforced at construction.
///
/// # Type Parameter
/// `N` - Maximum byte length (const generic).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoundedString<const N: usize>(String);

impl<const N: usize> BoundedString<N> {
    /// Create with length validation.
    pub fn new(s: impl Into<String>) -> HookResult<Self> {
        let s = s.into();
        if s.len() > N {
            Err(HookError::ValidationFailed(format!(
                "string length {} exceeds maximum {}",
                s.len(),
                N
            )))
        } else {
            Ok(Self(s))
        }
    }

    /// Maximum allowed length.
    #[inline]
    #[must_use]
    pub const fn max_len() -> usize {
        N
    }

    /// Access inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Length in bytes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<const N: usize> AsRef<str> for BoundedString<N> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> fmt::Display for BoundedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de, const N: usize> Deserialize<'de> for BoundedString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BoundedString::new(s).map_err(serde::de::Error::custom)
    }
}

impl<const N: usize> Serialize for BoundedString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Command description (max 500 chars).
pub type CommandDescription = BoundedString<500>;
/// Short identifier (max 64 chars).
pub type ShortId = BoundedString<64>;
/// Long text field (max 4096 chars).
pub type LongText = BoundedString<4096>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_rejects_empty() {
        assert!(NonEmptyString::new("").is_err());
    }

    #[test]
    fn non_empty_accepts_content() {
        let s = NonEmptyString::new("hello").unwrap();
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn non_empty_display() {
        let s = NonEmptyString::new("test").unwrap();
        assert_eq!(format!("{s}"), "test");
    }

    #[test]
    fn bounded_rejects_overflow() {
        assert!(BoundedString::<5>::new("123456").is_err());
    }

    #[test]
    fn bounded_accepts_within_limit() {
        assert!(BoundedString::<5>::new("12345").is_ok());
        assert!(BoundedString::<5>::new("").is_ok());
    }

    #[test]
    fn bounded_at_limit() {
        let s = BoundedString::<5>::new("12345").unwrap();
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[test]
    fn non_empty_serde_roundtrip() {
        let original = NonEmptyString::new("test").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: NonEmptyString = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn non_empty_serde_rejects_empty() {
        let json = r#""""#;
        let result: Result<NonEmptyString, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
