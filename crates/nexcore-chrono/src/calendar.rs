//! Civil calendar arithmetic — Hinnant algorithm for days ↔ (year, month, day).
//!
//! Public domain algorithm by Howard Hinnant. Pure arithmetic, no branches in hot path.
//! Handles negative days (dates before 1970) and the full Gregorian calendar.

/// Returns `true` if `year` is a leap year in the Gregorian calendar.
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

/// Returns the number of days in the given month (1-12) for the given year.
///
/// Returns 0 for invalid months.
#[must_use]
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Convert a civil date (year, month, day) to days since Unix epoch (1970-01-01 = day 0).
///
/// Based on Howard Hinnant's `days_from_civil` algorithm.
/// Assumes proleptic Gregorian calendar. No validation — caller must ensure valid date.
#[must_use]
#[allow(
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "Hinnant algorithm: all arithmetic provably bounded within Gregorian calendar range"
)]
pub const fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    // Shift year so March is month 0 (simplifies leap day handling)
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let m = if month <= 2 {
        month as i64 + 9
    } else {
        month as i64 - 3
    };
    let d = day as i64;

    // Era (400-year cycle)
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64; // year of era [0, 399]
    let doy = (153 * m + 2) / 5 + d - 1; // day of year [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy as u64; // day of era [0, 146096]

    (era * 146097 + doe as i64 - 719468) as i32
}

/// Convert days since Unix epoch to a civil date (year, month, day).
///
/// Based on Howard Hinnant's `civil_from_days` algorithm.
/// Returns `(year, month, day)` where month is 1-12 and day is 1-31.
#[must_use]
#[allow(
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "Hinnant algorithm: all arithmetic provably bounded within Gregorian calendar range"
)]
pub const fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let z = days as i64 + 719468;

    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month starting from March=0 [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    (y as i32, m as u32, d as u32)
}

/// Day of week from days since epoch. 0 = Monday through 6 = Sunday.
///
/// 1970-01-01 was a Thursday (day 3 in Monday-based indexing).
#[must_use]
#[allow(
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "weekday arithmetic: result bounded to [0, 6]"
)]
pub const fn weekday_from_days(days: i32) -> u32 {
    // Thursday = 3 in Monday=0 system
    let d = ((days as i64) + 3).rem_euclid(7);
    d as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Leap year tests ---

    #[test]
    fn test_leap_year_regular() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2000)); // 400-year rule
        assert!(is_leap_year(1600));
    }

    #[test]
    fn test_not_leap_year_century() {
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(1800));
    }

    #[test]
    fn test_not_leap_year_regular() {
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2025));
    }

    // --- Days in month tests ---

    #[test]
    fn test_days_in_month_regular() {
        assert_eq!(days_in_month(2025, 1), 31);
        assert_eq!(days_in_month(2025, 4), 30);
        assert_eq!(days_in_month(2025, 2), 28);
    }

    #[test]
    fn test_days_in_month_leap_feb() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2100, 2), 28);
    }

    #[test]
    fn test_days_in_month_invalid() {
        assert_eq!(days_in_month(2025, 0), 0);
        assert_eq!(days_in_month(2025, 13), 0);
    }

    // --- Hinnant reference validation ---

    #[test]
    fn test_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn test_negative_day() {
        assert_eq!(days_from_civil(1969, 12, 31), -1);
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn test_y2k() {
        // 2000-01-01 = 946684800 seconds / 86400 = 10957 days
        assert_eq!(days_from_civil(2000, 1, 1), 10957);
        assert_eq!(civil_from_days(10957), (2000, 1, 1));
    }

    #[test]
    fn test_leap_day_2000() {
        let d = days_from_civil(2000, 2, 29);
        assert_eq!(civil_from_days(d), (2000, 2, 29));
        // Day after leap day
        assert_eq!(civil_from_days(d + 1), (2000, 3, 1));
    }

    #[test]
    fn test_no_leap_day_2100() {
        // 2100-02-28 exists, 2100-03-01 is the next day
        let d = days_from_civil(2100, 2, 28);
        assert_eq!(civil_from_days(d + 1), (2100, 3, 1));
    }

    #[test]
    fn test_leap_day_2024() {
        let d = days_from_civil(2024, 2, 29);
        assert_eq!(civil_from_days(d), (2024, 2, 29));
        // 2024-02-29 = 1709164800 / 86400 = 19782 days
        assert_eq!(d, 19782);
    }

    #[test]
    fn test_today_2026() {
        let d = days_from_civil(2026, 2, 25);
        assert_eq!(civil_from_days(d), (2026, 2, 25));
    }

    #[test]
    fn test_deep_negative() {
        // 1582-10-15 — Gregorian adoption
        let d = days_from_civil(1582, 10, 15);
        assert_eq!(civil_from_days(d), (1582, 10, 15));
        assert!(d < 0);
    }

    #[test]
    fn test_round_trip_range() {
        // Test a wide range of dates round-trip correctly
        for days in [-100000, -50000, -1000, -1, 0, 1, 1000, 10000, 20000, 50000] {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(
                days_from_civil(y, m, d),
                days,
                "round-trip failed for day {days} => ({y}, {m}, {d})"
            );
        }
    }

    #[test]
    fn test_month_boundaries() {
        // Jan 31 → Feb 1
        let jan31 = days_from_civil(2025, 1, 31);
        assert_eq!(civil_from_days(jan31 + 1), (2025, 2, 1));

        // Feb 28 (non-leap) → Mar 1
        let feb28 = days_from_civil(2025, 2, 28);
        assert_eq!(civil_from_days(feb28 + 1), (2025, 3, 1));

        // Dec 31 → Jan 1 next year
        let dec31 = days_from_civil(2025, 12, 31);
        assert_eq!(civil_from_days(dec31 + 1), (2026, 1, 1));
    }

    // --- Weekday tests ---

    #[test]
    fn test_weekday_epoch() {
        // 1970-01-01 was a Thursday = 3 (Monday=0)
        assert_eq!(weekday_from_days(0), 3);
    }

    #[test]
    fn test_weekday_known() {
        // 2026-02-25 is a Wednesday = 2 (Monday=0)
        let d = days_from_civil(2026, 2, 25);
        assert_eq!(weekday_from_days(d), 2);
    }

    #[test]
    fn test_weekday_sequential() {
        // 7 consecutive days should produce 0-6 in some order
        let day0 = weekday_from_days(0); // Thursday = 3
        for i in 0..7 {
            let wd = weekday_from_days(i);
            assert_eq!(wd, ((day0 as i32 + i) % 7) as u32);
        }
    }

    #[test]
    fn test_weekday_negative() {
        // 1969-12-31 was a Wednesday = 2
        assert_eq!(weekday_from_days(-1), 2);
    }
}
