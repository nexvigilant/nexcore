//! Duration type — signed time span with microsecond precision.

use crate::error::ChronoError;
use core::fmt;
use core::ops::{Add, Neg, Sub};

/// Microseconds per second.
const MICROS_PER_SECOND: i64 = 1_000_000;
/// Microseconds per millisecond.
const MICROS_PER_MILLI: i64 = 1_000;
/// Seconds per minute.
const SECS_PER_MINUTE: i64 = 60;
/// Seconds per hour.
const SECS_PER_HOUR: i64 = 3_600;
/// Seconds per day.
const SECS_PER_DAY: i64 = 86_400;
/// Seconds per week.
const SECS_PER_WEEK: i64 = 604_800;

/// A time duration with microsecond precision. Can be negative.
///
/// Replaces `chrono::Duration` across 62 files (223 operations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    micros: i64,
}

impl Duration {
    /// Zero duration.
    #[must_use]
    pub const fn zero() -> Self {
        Self { micros: 0 }
    }

    /// Create from microseconds.
    #[must_use]
    pub const fn microseconds(us: i64) -> Self {
        Self { micros: us }
    }

    /// Create from milliseconds.
    #[must_use]
    pub const fn milliseconds(ms: i64) -> Self {
        Self {
            micros: ms.saturating_mul(MICROS_PER_MILLI),
        }
    }

    /// Try to create from milliseconds (chrono API compatibility).
    ///
    /// Always succeeds — returns `Some` wrapping [`Self::milliseconds`].
    #[must_use]
    pub const fn try_milliseconds(ms: i64) -> Option<Self> {
        Some(Self::milliseconds(ms))
    }

    /// Create from seconds.
    #[must_use]
    pub const fn seconds(secs: i64) -> Self {
        Self {
            micros: secs.saturating_mul(MICROS_PER_SECOND),
        }
    }

    /// Create from minutes.
    #[must_use]
    pub const fn minutes(mins: i64) -> Self {
        Self::seconds(mins.saturating_mul(SECS_PER_MINUTE))
    }

    /// Create from hours.
    #[must_use]
    pub const fn hours(h: i64) -> Self {
        Self::seconds(h.saturating_mul(SECS_PER_HOUR))
    }

    /// Create from days.
    #[must_use]
    pub const fn days(d: i64) -> Self {
        Self::seconds(d.saturating_mul(SECS_PER_DAY))
    }

    /// Create from weeks.
    #[must_use]
    pub const fn weeks(w: i64) -> Self {
        Self::seconds(w.saturating_mul(SECS_PER_WEEK))
    }

    /// Total microseconds.
    #[must_use]
    pub const fn num_microseconds(&self) -> i64 {
        self.micros
    }

    /// Total milliseconds (truncated).
    #[must_use]
    pub const fn num_milliseconds(&self) -> i64 {
        self.micros / MICROS_PER_MILLI
    }

    /// Total seconds (truncated).
    #[must_use]
    pub const fn num_seconds(&self) -> i64 {
        self.micros / MICROS_PER_SECOND
    }

    /// Total minutes (truncated).
    #[must_use]
    pub const fn num_minutes(&self) -> i64 {
        self.num_seconds() / SECS_PER_MINUTE
    }

    /// Total hours (truncated).
    #[must_use]
    pub const fn num_hours(&self) -> i64 {
        self.num_seconds() / SECS_PER_HOUR
    }

    /// Total days (truncated).
    #[must_use]
    pub const fn num_days(&self) -> i64 {
        self.num_seconds() / SECS_PER_DAY
    }

    /// Whether this duration is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.micros == 0
    }

    /// Checked addition. Returns `Err(Overflow)` on overflow.
    pub fn checked_add(self, rhs: Self) -> Result<Self, ChronoError> {
        self.micros
            .checked_add(rhs.micros)
            .map(|m| Self { micros: m })
            .ok_or(ChronoError::Overflow)
    }

    /// Checked subtraction. Returns `Err(Overflow)` on overflow.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ChronoError> {
        self.micros
            .checked_sub(rhs.micros)
            .map(|m| Self { micros: m })
            .ok_or(ChronoError::Overflow)
    }

    /// Absolute value of this duration.
    #[must_use]
    pub const fn abs(self) -> Self {
        Self {
            micros: self.micros.saturating_abs(),
        }
    }

    /// Convert from `std::time::Duration` (unsigned, always non-negative).
    #[must_use]
    pub fn from_std(d: std::time::Duration) -> Self {
        let micros = i64::try_from(d.as_micros()).unwrap_or(i64::MAX);
        Self { micros }
    }

    /// Convert to `std::time::Duration`. Returns `None` if negative.
    #[must_use]
    pub fn to_std(self) -> Option<std::time::Duration> {
        if self.micros < 0 {
            None
        } else {
            #[allow(clippy::as_conversions, reason = "micros >= 0, cast to u64 is safe")]
            Some(std::time::Duration::from_micros(self.micros as u64))
        }
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            micros: self.micros.saturating_add(rhs.micros),
        }
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            micros: self.micros.saturating_sub(rhs.micros),
        }
    }
}

impl Neg for Duration {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            micros: self.micros.saturating_neg(),
        }
    }
}

impl fmt::Display for Duration {
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "display decomposition: divisors are non-zero constants, unsigned_abs guarantees non-negative"
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_secs = self.num_seconds().unsigned_abs();
        let sign = if self.micros < 0 { "-" } else { "" };
        let secs_per_day = SECS_PER_DAY.unsigned_abs();
        let secs_per_hour = SECS_PER_HOUR.unsigned_abs();
        let secs_per_minute = SECS_PER_MINUTE.unsigned_abs();
        let days = total_secs / secs_per_day;
        let hours = (total_secs % secs_per_day) / secs_per_hour;
        let mins = (total_secs % secs_per_hour) / secs_per_minute;
        let secs = total_secs % secs_per_minute;

        if days > 0 {
            write!(f, "{sign}{days}d {hours}h {mins}m {secs}s")
        } else if hours > 0 {
            write!(f, "{sign}{hours}h {mins}m {secs}s")
        } else if mins > 0 {
            write!(f, "{sign}{mins}m {secs}s")
        } else {
            write!(f, "{sign}{secs}s")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        let d = Duration::zero();
        assert_eq!(d.num_microseconds(), 0);
        assert!(d.is_zero());
    }

    #[test]
    fn test_seconds() {
        let d = Duration::seconds(42);
        assert_eq!(d.num_seconds(), 42);
        assert_eq!(d.num_microseconds(), 42_000_000);
    }

    #[test]
    fn test_minutes() {
        let d = Duration::minutes(5);
        assert_eq!(d.num_seconds(), 300);
        assert_eq!(d.num_minutes(), 5);
    }

    #[test]
    fn test_hours() {
        let d = Duration::hours(2);
        assert_eq!(d.num_seconds(), 7200);
        assert_eq!(d.num_hours(), 2);
    }

    #[test]
    fn test_days() {
        let d = Duration::days(3);
        assert_eq!(d.num_seconds(), 259_200);
        assert_eq!(d.num_days(), 3);
    }

    #[test]
    fn test_weeks() {
        let d = Duration::weeks(1);
        assert_eq!(d.num_days(), 7);
    }

    #[test]
    fn test_milliseconds() {
        let d = Duration::milliseconds(1500);
        assert_eq!(d.num_seconds(), 1);
        assert_eq!(d.num_milliseconds(), 1500);
    }

    #[test]
    fn test_microseconds() {
        let d = Duration::microseconds(123_456);
        assert_eq!(d.num_microseconds(), 123_456);
    }

    #[test]
    fn test_add() {
        let a = Duration::seconds(30);
        let b = Duration::seconds(15);
        assert_eq!((a + b).num_seconds(), 45);
    }

    #[test]
    fn test_sub() {
        let a = Duration::seconds(30);
        let b = Duration::seconds(15);
        assert_eq!((a - b).num_seconds(), 15);
    }

    #[test]
    fn test_neg() {
        let d = Duration::seconds(42);
        assert_eq!((-d).num_seconds(), -42);
    }

    #[test]
    fn test_negative_duration() {
        let d = Duration::seconds(-10);
        assert_eq!(d.num_seconds(), -10);
        assert!(!d.is_zero());
    }

    #[test]
    fn test_abs() {
        assert_eq!(Duration::seconds(-42).abs(), Duration::seconds(42));
        assert_eq!(Duration::seconds(42).abs(), Duration::seconds(42));
    }

    #[test]
    fn test_checked_add_overflow() {
        let a = Duration::microseconds(i64::MAX);
        let b = Duration::microseconds(1);
        assert!(a.checked_add(b).is_err());
    }

    #[test]
    fn test_checked_sub_overflow() {
        let a = Duration::microseconds(i64::MIN);
        let b = Duration::microseconds(1);
        assert!(a.checked_sub(b).is_err());
    }

    #[test]
    fn test_from_std() {
        let std_dur = std::time::Duration::from_secs(60);
        let d = Duration::from_std(std_dur);
        assert_eq!(d.num_seconds(), 60);
    }

    #[test]
    fn test_display_seconds() {
        assert_eq!(Duration::seconds(45).to_string(), "45s");
    }

    #[test]
    fn test_display_minutes() {
        assert_eq!(Duration::seconds(90).to_string(), "1m 30s");
    }

    #[test]
    fn test_display_hours() {
        assert_eq!(Duration::seconds(3661).to_string(), "1h 1m 1s");
    }

    #[test]
    fn test_display_days() {
        assert_eq!(Duration::days(2).to_string(), "2d 0h 0m 0s");
    }

    #[test]
    fn test_display_negative() {
        assert_eq!(Duration::seconds(-45).to_string(), "-45s");
    }

    #[test]
    fn test_ordering() {
        assert!(Duration::seconds(10) > Duration::seconds(5));
        assert!(Duration::seconds(-1) < Duration::seconds(0));
    }
}
