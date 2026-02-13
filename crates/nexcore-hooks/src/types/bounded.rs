//! Bounded numeric types.

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{HookError, HookResult};

/// Timeout in seconds, bounded 1-3600 (1 hour max).
///
/// # Invariant
/// `1 <= self.0 <= 3600`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TimeoutSeconds(u16);

impl TimeoutSeconds {
    /// Minimum timeout value.
    pub const MIN: u16 = 1;
    /// Maximum timeout value (1 hour).
    pub const MAX: u16 = 3600;
    /// Default timeout (60 seconds).
    pub const DEFAULT: TimeoutSeconds = TimeoutSeconds(60);

    /// Create new timeout with bounds validation.
    pub fn new(seconds: u16) -> HookResult<Self> {
        if !(Self::MIN..=Self::MAX).contains(&seconds) {
            Err(HookError::ValidationFailed(format!(
                "timeout {} outside range [{}, {}]",
                seconds,
                Self::MIN,
                Self::MAX
            )))
        } else {
            Ok(Self(seconds))
        }
    }

    /// Get inner value.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> u16 {
        self.0
    }

    /// Convert to milliseconds for API compatibility.
    #[inline]
    #[must_use]
    pub const fn as_millis(&self) -> u64 {
        self.0 as u64 * 1000
    }

    /// Convert to std Duration.
    #[inline]
    #[must_use]
    pub const fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.0 as u64)
    }
}

impl Default for TimeoutSeconds {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'de> Deserialize<'de> for TimeoutSeconds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = u16::deserialize(deserializer)?;
        TimeoutSeconds::new(v).map_err(serde::de::Error::custom)
    }
}

/// Exit code with semantic meaning.
///
/// # Claude Code Exit Code Semantics
/// - 0: Success (stdout shown in verbose mode)
/// - 2: Blocking error (stderr fed to Claude)
/// - Other: Non-blocking error (stderr shown in verbose mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ExitCode {
    /// Success - hook completed normally.
    Success = 0,
    /// Non-blocking error - logged but execution continues.
    NonBlockingError = 1,
    /// Blocking error - stops tool execution, feedback to Claude.
    BlockingError = 2,
}

impl ExitCode {
    /// Convert to process exit code.
    #[inline]
    #[must_use]
    pub const fn code(&self) -> i32 {
        *self as i32
    }

    /// Exit the process with this code.
    pub fn exit(&self) -> ! {
        std::process::exit(self.code())
    }

    /// Convert to std ExitCode for main return.
    #[inline]
    #[must_use]
    pub const fn as_std(&self) -> std::process::ExitCode {
        match self {
            Self::Success => std::process::ExitCode::SUCCESS,
            Self::NonBlockingError | Self::BlockingError => std::process::ExitCode::FAILURE,
        }
    }
}

impl From<ExitCode> for i32 {
    fn from(e: ExitCode) -> i32 {
        e.code()
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(e: ExitCode) -> std::process::ExitCode {
        e.as_std()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn timeout_bounds_min() {
        assert!(TimeoutSeconds::new(0).is_err());
        assert!(TimeoutSeconds::new(1).is_ok());
    }

    #[test]
    fn timeout_bounds_max() {
        assert!(TimeoutSeconds::new(3600).is_ok());
        assert!(TimeoutSeconds::new(3601).is_err());
    }

    #[test]
    fn timeout_millis() {
        let t = TimeoutSeconds::new(60).unwrap();
        assert_eq!(t.as_millis(), 60_000);
    }

    #[test]
    fn timeout_duration() {
        let t = TimeoutSeconds::new(30).unwrap();
        assert_eq!(t.as_duration(), std::time::Duration::from_secs(30));
    }

    #[test]
    fn timeout_default() {
        let t = TimeoutSeconds::default();
        assert_eq!(t.get(), 60);
    }

    #[test]
    fn exit_codes() {
        assert_eq!(ExitCode::Success.code(), 0);
        assert_eq!(ExitCode::NonBlockingError.code(), 1);
        assert_eq!(ExitCode::BlockingError.code(), 2);
    }

    #[test]
    fn exit_code_into_i32() {
        let code: i32 = ExitCode::BlockingError.into();
        assert_eq!(code, 2);
    }

    #[test]
    fn timeout_serde_roundtrip() {
        let t = TimeoutSeconds::new(120).unwrap();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "120");
        let parsed: TimeoutSeconds = serde_json::from_str(&json).unwrap();
        assert_eq!(t, parsed);
    }

    #[test]
    fn timeout_serde_rejects_invalid() {
        let json = "0";
        let result: Result<TimeoutSeconds, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
