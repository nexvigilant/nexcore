//! Date type — calendar date without time or timezone.

use crate::calendar::{civil_from_days, days_from_civil, days_in_month, weekday_from_days};
use crate::components::{DateComponents, DayOfWeek};
use crate::duration::Duration;
use crate::error::ChronoError;
use crate::format::format_date_components;
use core::fmt;
use core::ops::{Add, Sub};

/// Seconds per day.
const SECS_PER_DAY: u64 = 86_400;

/// A calendar date without time or timezone.
///
/// Internal representation: days since 1970-01-01 (Unix epoch).
/// Covers OMOP CDM DATE columns and SOP milestones.
/// Replaces `chrono::NaiveDate` across 11 files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    days: i32,
}

impl Date {
    /// Unix epoch date: 1970-01-01.
    pub const EPOCH: Self = Self { days: 0 };

    /// Create a date from year, month, day.
    ///
    /// Returns `Err(InvalidDate)` if the date is invalid.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self, ChronoError> {
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return Err(ChronoError::InvalidDate { year, month, day });
        }
        Ok(Self {
            days: days_from_civil(year, month, day),
        })
    }

    /// Create from days since Unix epoch.
    #[must_use]
    pub const fn from_days_since_epoch(days: i32) -> Self {
        Self { days }
    }

    /// Today's date (UTC).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        reason = "u64 days since epoch fits in i32 for any realistic date (until year ~5.8M)"
    )]
    pub fn today() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let days = (now.as_secs() / SECS_PER_DAY) as i32;
        Self { days }
    }

    /// Year component.
    #[must_use]
    pub fn year(&self) -> i32 {
        civil_from_days(self.days).0
    }

    /// Month component (1-12).
    #[must_use]
    pub fn month(&self) -> u32 {
        civil_from_days(self.days).1
    }

    /// Day component (1-31).
    #[must_use]
    pub fn day(&self) -> u32 {
        civil_from_days(self.days).2
    }

    /// Day of week.
    #[must_use]
    pub fn day_of_week(&self) -> DayOfWeek {
        // weekday_from_days returns 0-6, DayOfWeek::from_number always succeeds for 0-6
        DayOfWeek::from_number(weekday_from_days(self.days)).unwrap_or(DayOfWeek::Monday) // unreachable for valid weekday
    }

    /// Days since Unix epoch.
    #[must_use]
    pub const fn days_since_epoch(&self) -> i32 {
        self.days
    }

    /// Extract all date components at once.
    #[must_use]
    pub fn components(&self) -> DateComponents {
        let (year, month, day) = civil_from_days(self.days);
        DateComponents { year, month, day }
    }

    /// Format using a strftime-compatible format string.
    ///
    /// Supported: `%Y`, `%m`, `%d`, `%F` (`%Y-%m-%d`).
    pub fn format(&self, fmt: &str) -> Result<String, ChronoError> {
        let (year, month, day) = civil_from_days(self.days);
        format_date_components(year, month, day, 0, 0, 0, 0, fmt)
    }

    /// ISO 8601 date string: "YYYY-MM-DD".
    #[must_use]
    pub fn to_iso8601(&self) -> String {
        let (y, m, d) = civil_from_days(self.days);
        alloc::format!("{y:04}-{m:02}-{d:02}")
    }
}

extern crate alloc;

impl Add<Duration> for Date {
    type Output = Self;

    #[allow(
        clippy::as_conversions,
        reason = "Duration days (i64) truncated to i32: dates beyond i32 range are unreachable"
    )]
    fn add(self, rhs: Duration) -> Self {
        let delta_days = rhs.num_days() as i32;
        Self {
            days: self.days.saturating_add(delta_days),
        }
    }
}

impl Sub<Duration> for Date {
    type Output = Self;

    #[allow(
        clippy::as_conversions,
        reason = "Duration days (i64) truncated to i32: dates beyond i32 range are unreachable"
    )]
    fn sub(self, rhs: Duration) -> Self {
        let delta_days = rhs.num_days() as i32;
        Self {
            days: self.days.saturating_sub(delta_days),
        }
    }
}

impl Sub for Date {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        let delta_days = i64::from(self.days).saturating_sub(i64::from(rhs.days));
        Duration::days(delta_days)
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::EPOCH
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, m, d) = civil_from_days(self.days);
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch() {
        let d = Date::EPOCH;
        assert_eq!(d.year(), 1970);
        assert_eq!(d.month(), 1);
        assert_eq!(d.day(), 1);
        assert_eq!(d.days_since_epoch(), 0);
    }

    #[test]
    fn test_from_ymd_valid() {
        let d = Date::from_ymd(2026, 2, 25).expect("valid date");
        assert_eq!(d.year(), 2026);
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 25);
    }

    #[test]
    fn test_from_ymd_invalid_month() {
        assert!(Date::from_ymd(2026, 13, 1).is_err());
        assert!(Date::from_ymd(2026, 0, 1).is_err());
    }

    #[test]
    fn test_from_ymd_invalid_day() {
        assert!(Date::from_ymd(2025, 2, 29).is_err()); // not a leap year
        assert!(Date::from_ymd(2025, 4, 31).is_err()); // April has 30 days
    }

    #[test]
    fn test_from_ymd_leap_day() {
        let d = Date::from_ymd(2024, 2, 29).expect("valid leap day");
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 29);
    }

    #[test]
    fn test_components() {
        let d = Date::from_ymd(2026, 2, 25).expect("valid");
        let c = d.components();
        assert_eq!(c.year, 2026);
        assert_eq!(c.month, 2);
        assert_eq!(c.day, 25);
    }

    #[test]
    fn test_display() {
        let d = Date::from_ymd(2026, 2, 25).expect("valid");
        assert_eq!(d.to_string(), "2026-02-25");
    }

    #[test]
    fn test_iso8601() {
        let d = Date::from_ymd(2026, 2, 5).expect("valid");
        assert_eq!(d.to_iso8601(), "2026-02-05");
    }

    #[test]
    fn test_add_duration() {
        let d = Date::from_ymd(2026, 2, 25).expect("valid");
        let next = d + Duration::days(5);
        assert_eq!(next, Date::from_ymd(2026, 3, 2).expect("valid"));
    }

    #[test]
    fn test_sub_duration() {
        let d = Date::from_ymd(2026, 3, 2).expect("valid");
        let prev = d - Duration::days(5);
        assert_eq!(prev, Date::from_ymd(2026, 2, 25).expect("valid"));
    }

    #[test]
    fn test_sub_dates() {
        let a = Date::from_ymd(2026, 3, 1).expect("valid");
        let b = Date::from_ymd(2026, 2, 1).expect("valid");
        assert_eq!((a - b).num_days(), 28);
    }

    #[test]
    fn test_ordering() {
        let a = Date::from_ymd(2025, 1, 1).expect("valid");
        let b = Date::from_ymd(2026, 1, 1).expect("valid");
        assert!(a < b);
    }

    #[test]
    fn test_day_of_week_epoch() {
        // 1970-01-01 was a Thursday
        assert_eq!(Date::EPOCH.day_of_week(), DayOfWeek::Thursday);
    }

    #[test]
    fn test_today_is_valid() {
        let t = Date::today();
        assert!(t.year() >= 2026);
        assert!(t.month() >= 1 && t.month() <= 12);
    }
}
