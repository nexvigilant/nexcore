//! NaiveDateTime — datetime without timezone assertion.
//!
//! For OMOP CDM DATETIME columns where timezone is source-system-defined.

use crate::calendar::{civil_from_days, days_from_civil, days_in_month};
use crate::components::DateTimeComponents;
use crate::datetime::DateTime;
use crate::error::ChronoError;
use crate::format::format_date_components;
use core::fmt;

/// Microseconds per second.
const MICROS_PER_SECOND: i64 = 1_000_000;
/// Microseconds per day.
const MICROS_PER_DAY: i64 = 86_400_000_000;

/// A datetime without timezone assertion.
///
/// Structurally identical to `DateTime` but semantically different — makes no
/// timezone claim. Used for OMOP CDM DATETIME columns (20 occurrences, 4 files).
///
/// Replaces `chrono::NaiveDateTime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NaiveDateTime {
    micros: i64,
}

impl NaiveDateTime {
    /// Create from date and time components.
    #[allow(
        clippy::too_many_arguments,
        clippy::arithmetic_side_effects,
        reason = "datetime constructor mirrors chrono API, component values validated below"
    )]
    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, ChronoError> {
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return Err(ChronoError::InvalidDate { year, month, day });
        }
        if hour > 23 || minute > 59 || second > 59 {
            return Err(ChronoError::InvalidTime {
                hour,
                minute,
                second,
            });
        }

        let days = i64::from(days_from_civil(year, month, day));
        let time_micros = i64::from(hour) * 3_600_000_000
            + i64::from(minute) * 60_000_000
            + i64::from(second) * MICROS_PER_SECOND;

        Ok(Self {
            micros: days
                .saturating_mul(MICROS_PER_DAY)
                .saturating_add(time_micros),
        })
    }

    /// Create from Unix timestamp (seconds). No timezone assertion.
    #[must_use]
    pub const fn from_timestamp(secs: i64) -> Self {
        Self {
            micros: secs.saturating_mul(MICROS_PER_SECOND),
        }
    }

    /// Decompose into days and time-of-day microseconds.
    #[allow(
        clippy::as_conversions,
        reason = "div_euclid result fits i32 for realistic dates"
    )]
    fn decompose(&self) -> (i32, i64) {
        let days = self.micros.div_euclid(MICROS_PER_DAY);
        let time_micros = self.micros.rem_euclid(MICROS_PER_DAY);
        (days as i32, time_micros)
    }

    /// Year component.
    #[must_use]
    pub fn year(&self) -> i32 {
        civil_from_days(self.decompose().0).0
    }

    /// Month component (1-12).
    #[must_use]
    pub fn month(&self) -> u32 {
        civil_from_days(self.decompose().0).1
    }

    /// Day component (1-31).
    #[must_use]
    pub fn day(&self) -> u32 {
        civil_from_days(self.decompose().0).2
    }

    /// Hour component (0-23).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 23]"
    )]
    pub fn hour(&self) -> u32 {
        (self.decompose().1 / 3_600_000_000) as u32
    }

    /// Minute component (0-59).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 59]"
    )]
    pub fn minute(&self) -> u32 {
        ((self.decompose().1 % 3_600_000_000) / 60_000_000) as u32
    }

    /// Second component (0-59).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 59]"
    )]
    pub fn second(&self) -> u32 {
        ((self.decompose().1 % 60_000_000) / MICROS_PER_SECOND) as u32
    }

    /// Interpret as UTC `DateTime` (explicit timezone upgrade).
    #[must_use]
    pub const fn to_datetime(&self) -> DateTime {
        DateTime::from_timestamp_micros(self.micros)
    }

    /// Extract all components.
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), all component values bounded"
    )]
    pub fn components(&self) -> DateTimeComponents {
        let (days, time) = self.decompose();
        let (year, month, day) = civil_from_days(days);
        DateTimeComponents {
            year,
            month,
            day,
            hour: (time / 3_600_000_000) as u32,
            minute: ((time % 3_600_000_000) / 60_000_000) as u32,
            second: ((time % 60_000_000) / MICROS_PER_SECOND) as u32,
            microsecond: (time % MICROS_PER_SECOND) as u32,
        }
    }

    /// Format using a strftime-compatible format string.
    pub fn format(&self, fmt: &str) -> Result<String, ChronoError> {
        let c = self.components();
        format_date_components(
            c.year,
            c.month,
            c.day,
            c.hour,
            c.minute,
            c.second,
            c.microsecond,
            fmt,
        )
    }

    /// Unix timestamp in seconds (no timezone assertion).
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.micros / MICROS_PER_SECOND
    }
}

impl fmt::Display for NaiveDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = self.components();
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            c.year, c.month, c.day, c.hour, c.minute, c.second,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ymd_hms() {
        let ndt = NaiveDateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        assert_eq!(ndt.year(), 2026);
        assert_eq!(ndt.month(), 2);
        assert_eq!(ndt.day(), 25);
        assert_eq!(ndt.hour(), 14);
        assert_eq!(ndt.minute(), 30);
        assert_eq!(ndt.second(), 0);
    }

    #[test]
    fn test_invalid_date() {
        assert!(NaiveDateTime::from_ymd_hms(2025, 2, 29, 0, 0, 0).is_err());
    }

    #[test]
    fn test_invalid_time() {
        assert!(NaiveDateTime::from_ymd_hms(2026, 1, 1, 24, 0, 0).is_err());
    }

    #[test]
    fn test_from_timestamp() {
        let ndt = NaiveDateTime::from_timestamp(946684800);
        assert_eq!(ndt.year(), 2000);
        assert_eq!(ndt.month(), 1);
        assert_eq!(ndt.day(), 1);
    }

    #[test]
    fn test_to_datetime() {
        let ndt = NaiveDateTime::from_ymd_hms(2026, 2, 25, 14, 0, 0).expect("valid");
        let dt = ndt.to_datetime();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.hour(), 14);
    }

    #[test]
    fn test_components() {
        let ndt = NaiveDateTime::from_ymd_hms(2026, 6, 15, 10, 30, 45).expect("valid");
        let c = ndt.components();
        assert_eq!(c.year, 2026);
        assert_eq!(c.month, 6);
        assert_eq!(c.day, 15);
        assert_eq!(c.hour, 10);
        assert_eq!(c.minute, 30);
        assert_eq!(c.second, 45);
    }

    #[test]
    fn test_display() {
        let ndt = NaiveDateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        assert_eq!(ndt.to_string(), "2026-02-25T14:30:00");
    }

    #[test]
    fn test_ordering() {
        let a = NaiveDateTime::from_ymd_hms(2026, 1, 1, 0, 0, 0).expect("valid");
        let b = NaiveDateTime::from_ymd_hms(2026, 1, 2, 0, 0, 0).expect("valid");
        assert!(a < b);
    }

    #[test]
    fn test_timestamp() {
        let ndt = NaiveDateTime::from_timestamp(1000);
        assert_eq!(ndt.timestamp(), 1000);
    }

    #[test]
    fn test_round_trip_components() {
        let ndt = NaiveDateTime::from_ymd_hms(2024, 2, 29, 23, 59, 59).expect("valid");
        let c = ndt.components();
        let ndt2 = NaiveDateTime::from_ymd_hms(c.year, c.month, c.day, c.hour, c.minute, c.second)
            .expect("valid");
        assert_eq!(ndt, ndt2);
    }
}
