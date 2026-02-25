//! DateTime type — UTC timestamp with microsecond precision.

use crate::calendar::{civil_from_days, days_from_civil, days_in_month};
use crate::components::DateTimeComponents;
use crate::date::Date;
use crate::duration::Duration;
use crate::error::ChronoError;
use crate::format::format_date_components;
use core::fmt;
use core::ops::{Add, Sub};
use core::str::FromStr;

/// Microseconds per second.
const MICROS_PER_SECOND: i64 = 1_000_000;
/// Microseconds per millisecond.
const MICROS_PER_MILLI: i64 = 1_000;
/// Microseconds per day.
const MICROS_PER_DAY: i64 = 86_400_000_000;
/// Seconds per day.
const SECS_PER_DAY: i64 = 86_400;

/// UTC timestamp with microsecond precision.
///
/// Internal representation: microseconds since 1970-01-01T00:00:00Z.
/// Replaces `chrono::DateTime<Utc>` across 78 crates (1,362 type annotations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    micros: i64,
}

impl DateTime {
    /// Unix epoch: 1970-01-01T00:00:00Z.
    pub const EPOCH: Self = Self { micros: 0 };

    /// Current UTC time. Infallible — matches chrono's `Utc::now()` behavior.
    ///
    /// If the system clock is before 1970 (should not happen), returns epoch.
    #[must_use]
    #[allow(
        clippy::as_conversions,
        reason = "SystemTime seconds/micros always fit in i64 for realistic timestamps"
    )]
    pub fn now() -> Self {
        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = dur.as_secs() as i64;
        let sub_micros = i64::from(dur.subsec_micros());
        Self {
            micros: secs
                .saturating_mul(MICROS_PER_SECOND)
                .saturating_add(sub_micros),
        }
    }

    /// Create from Unix timestamp (seconds since epoch).
    #[must_use]
    pub const fn from_timestamp(secs: i64) -> Self {
        Self {
            micros: secs.saturating_mul(MICROS_PER_SECOND),
        }
    }

    /// Create from milliseconds since epoch.
    #[must_use]
    pub const fn from_timestamp_millis(ms: i64) -> Self {
        Self {
            micros: ms.saturating_mul(MICROS_PER_MILLI),
        }
    }

    /// Create from microseconds since epoch.
    #[must_use]
    pub const fn from_timestamp_micros(us: i64) -> Self {
        Self { micros: us }
    }

    /// Create from date and time components.
    #[allow(
        clippy::too_many_arguments,
        reason = "datetime constructor mirrors chrono API: year/month/day/hour/minute/second"
    )]
    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, ChronoError> {
        Self::from_components(DateTimeComponents {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond: 0,
        })
    }

    /// Create from full components struct.
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "component values validated above: hour<=23, minute<=59, second<=59, microsecond<=999999"
    )]
    pub fn from_components(c: DateTimeComponents) -> Result<Self, ChronoError> {
        // Validate date
        if !(1..=12).contains(&c.month) || c.day < 1 || c.day > days_in_month(c.year, c.month) {
            return Err(ChronoError::InvalidDate {
                year: c.year,
                month: c.month,
                day: c.day,
            });
        }
        // Validate time
        if c.hour > 23 || c.minute > 59 || c.second > 59 || c.microsecond > 999_999 {
            return Err(ChronoError::InvalidTime {
                hour: c.hour,
                minute: c.minute,
                second: c.second,
            });
        }

        let days = i64::from(days_from_civil(c.year, c.month, c.day));
        let time_micros = i64::from(c.hour) * 3_600_000_000
            + i64::from(c.minute) * 60_000_000
            + i64::from(c.second) * MICROS_PER_SECOND
            + i64::from(c.microsecond);

        Ok(Self {
            micros: days
                .saturating_mul(MICROS_PER_DAY)
                .saturating_add(time_micros),
        })
    }

    /// Decompose into days-since-epoch and time-of-day microseconds.
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "div_euclid/rem_euclid guarantee: days fits i32 for realistic dates, time_micros in [0, MICROS_PER_DAY)"
    )]
    fn decompose(&self) -> (i32, i64) {
        let mut days = self.micros.div_euclid(MICROS_PER_DAY);
        let mut time_micros = self.micros.rem_euclid(MICROS_PER_DAY);

        // Ensure time_micros is non-negative
        if time_micros < 0 {
            days -= 1;
            time_micros += MICROS_PER_DAY;
        }

        (days as i32, time_micros)
    }

    /// Unix timestamp in seconds.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.micros / MICROS_PER_SECOND
    }

    /// Unix timestamp in milliseconds.
    #[must_use]
    pub const fn timestamp_millis(&self) -> i64 {
        self.micros / MICROS_PER_MILLI
    }

    /// Unix timestamp in microseconds.
    #[must_use]
    pub const fn timestamp_micros(&self) -> i64 {
        self.micros
    }

    /// Year component.
    #[must_use]
    pub fn year(&self) -> i32 {
        let (days, _) = self.decompose();
        civil_from_days(days).0
    }

    /// Month component (1-12).
    #[must_use]
    pub fn month(&self) -> u32 {
        let (days, _) = self.decompose();
        civil_from_days(days).1
    }

    /// Day component (1-31).
    #[must_use]
    pub fn day(&self) -> u32 {
        let (days, _) = self.decompose();
        civil_from_days(days).2
    }

    /// Hour component (0-23).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 23]"
    )]
    pub fn hour(&self) -> u32 {
        let (_, time) = self.decompose();
        (time / 3_600_000_000) as u32
    }

    /// Minute component (0-59).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 59]"
    )]
    pub fn minute(&self) -> u32 {
        let (_, time) = self.decompose();
        ((time % 3_600_000_000) / 60_000_000) as u32
    }

    /// Second component (0-59).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 59]"
    )]
    pub fn second(&self) -> u32 {
        let (_, time) = self.decompose();
        ((time % 60_000_000) / MICROS_PER_SECOND) as u32
    }

    /// Microsecond component (0-999999).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "time_micros from decompose is in [0, MICROS_PER_DAY), result bounded to [0, 999999]"
    )]
    pub fn microsecond(&self) -> u32 {
        let (_, time) = self.decompose();
        (time % MICROS_PER_SECOND) as u32
    }

    /// Extract the date portion.
    #[must_use]
    pub fn date(&self) -> Date {
        let (days, _) = self.decompose();
        Date::from_days_since_epoch(days)
    }

    /// Extract all components at once.
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

    /// Format as RFC 3339 string: "YYYY-MM-DDTHH:MM:SSZ".
    ///
    /// Includes microsecond fractional seconds only if non-zero.
    #[must_use]
    pub fn to_rfc3339(&self) -> String {
        let c = self.components();
        if c.microsecond > 0 {
            alloc::format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
                c.year,
                c.month,
                c.day,
                c.hour,
                c.minute,
                c.second,
                c.microsecond,
            )
        } else {
            alloc::format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                c.year,
                c.month,
                c.day,
                c.hour,
                c.minute,
                c.second,
            )
        }
    }

    /// Format using a strftime-compatible format string.
    ///
    /// Supported specifiers: `%Y`, `%m`, `%d`, `%H`, `%M`, `%S`, `%F`, `%T`, `%%`.
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

    /// Parse an RFC 3339 datetime string.
    ///
    /// Convenience static method delegating to [`crate::parse::parse_rfc3339`].
    pub fn parse_rfc3339(s: &str) -> Result<Self, ChronoError> {
        crate::parse::parse_rfc3339(s)
    }

    /// Alias for [`Self::parse_rfc3339`] — chrono API compatibility.
    pub fn parse_from_rfc3339(s: &str) -> Result<Self, ChronoError> {
        Self::parse_rfc3339(s)
    }

    /// Compute signed duration between two datetimes.
    ///
    /// Equivalent to `self - other`. Provided for chrono API compatibility.
    #[must_use]
    pub fn signed_duration_since(self, other: Self) -> Duration {
        Duration::microseconds(self.micros.saturating_sub(other.micros))
    }

    /// Current local time (system UTC offset applied).
    ///
    /// Reads the `TZ` environment variable or falls back to UTC.
    /// For the 7 call sites that use `Local::now()` for file naming.
    #[must_use]
    pub fn now_local() -> Self {
        let utc = Self::now();
        let offset_secs = local_utc_offset_seconds();
        Self {
            micros: utc
                .micros
                .saturating_add(i64::from(offset_secs).saturating_mul(MICROS_PER_SECOND)),
        }
    }
}

extern crate alloc;

/// Get the local UTC offset in seconds by checking environment.
///
/// Returns 0 (UTC) if TZ cannot be determined. No unsafe, no libc FFI.
fn local_utc_offset_seconds() -> i32 {
    // Strategy: parse simple fixed-offset TZ values like "EST5EDT" or numeric offsets.
    // For file naming purposes (the only use case), UTC is an acceptable fallback.
    // Full IANA timezone resolution is explicitly out of scope.
    if let Ok(tz) = std::env::var("TZ") {
        parse_tz_offset(&tz)
    } else {
        0
    }
}

/// Parse a simple TZ offset string. Returns offset in seconds.
///
/// Handles formats like "UTC+5", "UTC-5", "EST+5" (POSIX: positive = west of Greenwich).
/// Returns 0 for unrecognized formats.
#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "TZ string parsing: index bounds checked by loop condition, byte→i32 bounded by ASCII digit range"
)]
fn parse_tz_offset(tz: &str) -> i32 {
    // POSIX TZ format: std offset [dst [offset] [,rule]]
    // The offset after std name is hours west of UTC (positive = west)
    // We need to find the numeric part
    let bytes = tz.as_bytes();
    let mut i = 0;

    // Skip alphabetic prefix (timezone name)
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }

    if i >= bytes.len() {
        return 0;
    }

    // Parse optional sign and hours
    let negative = bytes[i] == b'-';
    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }

    let mut hours: i32 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        hours = hours
            .saturating_mul(10)
            .saturating_add((bytes[i] - b'0') as i32);
        i += 1;
    }

    // POSIX convention: positive offset = west of GMT, so negate for UTC offset
    let offset_hours = if negative { hours } else { -hours };
    offset_hours.saturating_mul(3600)
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self {
            micros: self.micros.saturating_add(rhs.num_microseconds()),
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Self {
            micros: self.micros.saturating_sub(rhs.num_microseconds()),
        }
    }
}

impl Sub for DateTime {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        Duration::microseconds(self.micros.saturating_sub(rhs.micros))
    }
}

impl Default for DateTime {
    /// Returns the Unix epoch (1970-01-01T00:00:00Z).
    fn default() -> Self {
        Self { micros: 0 }
    }
}

impl From<std::time::SystemTime> for DateTime {
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
        reason = "SystemTime epoch duration: secs fits i64 for realistic dates, subsec_micros bounded [0, 999_999]"
    )]
    fn from(st: std::time::SystemTime) -> Self {
        match st.duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => {
                let secs = d.as_secs() as i64;
                let micros = d.subsec_micros() as i64;
                Self::from_timestamp_micros(
                    secs.saturating_mul(MICROS_PER_SECOND)
                        .saturating_add(micros),
                )
            }
            Err(e) => {
                let d = e.duration();
                let secs = d.as_secs() as i64;
                let micros = d.subsec_micros() as i64;
                Self::from_timestamp_micros(
                    secs.saturating_mul(MICROS_PER_SECOND)
                        .saturating_add(micros)
                        .saturating_neg(),
                )
            }
        }
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_rfc3339())
    }
}

impl FromStr for DateTime {
    type Err = ChronoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_rfc3339(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch() {
        let dt = DateTime::EPOCH;
        assert_eq!(dt.timestamp(), 0);
        assert_eq!(dt.year(), 1970);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_from_timestamp_y2k() {
        let dt = DateTime::from_timestamp(946684800);
        assert_eq!(dt.year(), 2000);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
    }

    #[test]
    fn test_from_timestamp_millis() {
        let dt = DateTime::from_timestamp_millis(946684800_000);
        assert_eq!(dt.timestamp(), 946684800);
    }

    #[test]
    fn test_from_timestamp_negative() {
        let dt = DateTime::from_timestamp(-86400);
        assert_eq!(dt.year(), 1969);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 31);
    }

    #[test]
    fn test_from_ymd_hms() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 25);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_from_ymd_hms_invalid_date() {
        assert!(DateTime::from_ymd_hms(2025, 2, 29, 0, 0, 0).is_err());
    }

    #[test]
    fn test_from_ymd_hms_invalid_time() {
        assert!(DateTime::from_ymd_hms(2026, 1, 1, 25, 0, 0).is_err());
    }

    #[test]
    fn test_from_components() {
        let c = DateTimeComponents {
            year: 2024,
            month: 2,
            day: 29,
            hour: 12,
            minute: 0,
            second: 0,
            microsecond: 500_000,
        };
        let dt = DateTime::from_components(c).expect("valid");
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 29);
        assert_eq!(dt.microsecond(), 500_000);
    }

    #[test]
    fn test_timestamp_millis() {
        let dt = DateTime::from_ymd_hms(2026, 1, 1, 0, 0, 0).expect("valid");
        let ts = dt.timestamp();
        assert_eq!(dt.timestamp_millis(), ts.saturating_mul(1000));
    }

    #[test]
    fn test_components_round_trip() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 45).expect("valid");
        let c = dt.components();
        let dt2 = DateTime::from_components(c).expect("valid");
        assert_eq!(dt, dt2);
    }

    #[test]
    fn test_date_extraction() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        let d = dt.date();
        assert_eq!(d.year(), 2026);
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 25);
    }

    #[test]
    fn test_to_rfc3339() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        assert_eq!(dt.to_rfc3339(), "2026-02-25T14:30:00Z");
    }

    #[test]
    fn test_to_rfc3339_with_micros() {
        let c = DateTimeComponents {
            year: 2026,
            month: 2,
            day: 25,
            hour: 14,
            minute: 30,
            second: 0,
            microsecond: 123_456,
        };
        let dt = DateTime::from_components(c).expect("valid");
        assert_eq!(dt.to_rfc3339(), "2026-02-25T14:30:00.123456Z");
    }

    #[test]
    fn test_add_duration() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 23, 0, 0).expect("valid");
        let next = dt + Duration::hours(2);
        assert_eq!(next.day(), 26);
        assert_eq!(next.hour(), 1);
    }

    #[test]
    fn test_sub_duration() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 1, 0, 0).expect("valid");
        let prev = dt - Duration::hours(2);
        assert_eq!(prev.day(), 24);
        assert_eq!(prev.hour(), 23);
    }

    #[test]
    fn test_sub_datetimes() {
        let a = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        let b = DateTime::from_ymd_hms(2026, 2, 25, 14, 0, 0).expect("valid");
        let d = a - b;
        assert_eq!(d.num_minutes(), 30);
    }

    #[test]
    fn test_now_returns_recent() {
        let now = DateTime::now();
        assert!(now.year() >= 2026);
    }

    #[test]
    fn test_display() {
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
        assert_eq!(dt.to_string(), "2026-02-25T14:30:00Z");
    }

    #[test]
    fn test_ordering() {
        let a = DateTime::from_timestamp(100);
        let b = DateTime::from_timestamp(200);
        assert!(a < b);
    }

    #[test]
    fn test_reference_timestamps() {
        // Directive reference validation points
        assert_eq!(
            DateTime::from_timestamp(0).to_rfc3339(),
            "1970-01-01T00:00:00Z"
        );
        assert_eq!(
            DateTime::from_timestamp(946684800).to_rfc3339(),
            "2000-01-01T00:00:00Z"
        );
        assert_eq!(
            DateTime::from_timestamp(-86400).to_rfc3339(),
            "1969-12-31T00:00:00Z"
        );
    }

    #[test]
    fn test_leap_day_timestamp() {
        // 2024-02-29T00:00:00Z
        let dt = DateTime::from_ymd_hms(2024, 2, 29, 0, 0, 0).expect("valid");
        let expected_ts: i64 = 19782_i64.saturating_mul(SECS_PER_DAY);
        assert_eq!(dt.timestamp(), expected_ts);
    }

    #[test]
    fn test_tz_offset_parse() {
        assert_eq!(parse_tz_offset("EST5"), -5 * 3600);
        assert_eq!(parse_tz_offset("UTC"), 0);
        assert_eq!(parse_tz_offset("UTC+0"), 0);
        assert_eq!(parse_tz_offset("CET-1"), 3600);
        assert_eq!(parse_tz_offset(""), 0);
    }
}
