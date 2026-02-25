//! Error types for nexcore-chrono.

use core::fmt;

/// Errors that can occur during date/time operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChronoError {
    /// Month not in 1..=12 or day not valid for month/year.
    InvalidDate { year: i32, month: u32, day: u32 },
    /// Hour/minute/second/microsecond out of range.
    InvalidTime { hour: u32, minute: u32, second: u32 },
    /// Format string contains unsupported specifier.
    InvalidFormat { specifier: char },
    /// Input string doesn't match expected format.
    ParseError { input: String, expected: String },
    /// Arithmetic overflow.
    Overflow,
}

impl fmt::Display for ChronoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate { year, month, day } => {
                write!(f, "invalid date: {year}-{month:02}-{day:02}")
            }
            Self::InvalidTime {
                hour,
                minute,
                second,
            } => {
                write!(f, "invalid time: {hour:02}:{minute:02}:{second:02}")
            }
            Self::InvalidFormat { specifier } => {
                write!(f, "unsupported format specifier: %{specifier}")
            }
            Self::ParseError { input, expected } => {
                write!(f, "parse error: expected {expected}, got '{input}'")
            }
            Self::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl std::error::Error for ChronoError {}

// NexError conversion: handled by blanket `impl<E: Error + Send + Sync + 'static> From<E> for NexError`
// in nexcore-error. ChronoError is Send + Sync + 'static, so `?` works automatically.
