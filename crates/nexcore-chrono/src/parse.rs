//! Parsing — RFC 3339, ISO 8601, and strftime-subset format strings.

use crate::calendar::days_in_month;
use crate::date::Date;
use crate::datetime::DateTime;
use crate::error::ChronoError;
use crate::naive_datetime::NaiveDateTime;

/// Parse an RFC 3339 string into a `DateTime`.
///
/// Accepted formats:
/// - `YYYY-MM-DDTHH:MM:SSZ`
/// - `YYYY-MM-DDTHH:MM:SS.ffffffZ`
/// - `YYYY-MM-DDTHH:MM:SS+00:00`
/// - `YYYY-MM-DDTHH:MM:SS.ffffff+00:00`
#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "parser: all index accesses guarded by length checks, as-casts from i32 to u32 bounded by parsed digit values"
)]
pub fn parse_rfc3339(input: &str) -> Result<DateTime, ChronoError> {
    let bytes = input.as_bytes();

    // Minimum length: "YYYY-MM-DDTHH:MM:SSZ" = 20
    if bytes.len() < 20 {
        return Err(parse_err(input, "RFC 3339 (min 20 chars)"));
    }

    let year = parse_digits(bytes, 0, 4, input)?;
    expect_byte(bytes, 4, b'-', input)?;
    let month = parse_digits(bytes, 5, 2, input)? as u32;
    expect_byte(bytes, 7, b'-', input)?;
    let day = parse_digits(bytes, 8, 2, input)? as u32;

    // T or space separator
    if bytes[10] != b'T' && bytes[10] != b't' && bytes[10] != b' ' {
        return Err(parse_err(input, "RFC 3339 (expected T separator)"));
    }

    let hour = parse_digits(bytes, 11, 2, input)? as u32;
    expect_byte(bytes, 13, b':', input)?;
    let minute = parse_digits(bytes, 14, 2, input)? as u32;
    expect_byte(bytes, 16, b':', input)?;
    let second = parse_digits(bytes, 17, 2, input)? as u32;

    let mut microsecond: u32 = 0;
    let mut pos = 19;

    // Optional fractional seconds
    if pos < bytes.len() && bytes[pos] == b'.' {
        pos += 1;
        let frac_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        let frac_len = pos - frac_start;
        if frac_len == 0 {
            return Err(parse_err(input, "RFC 3339 (empty fractional seconds)"));
        }
        let frac_val = parse_digits(bytes, frac_start, frac_len, input)?;
        // Normalize to microseconds (6 digits)
        let mut v = frac_val as u32;
        if frac_len <= 6 {
            for _ in frac_len..6 {
                v = v.saturating_mul(10);
            }
        } else {
            for _ in 6..frac_len {
                v /= 10;
            }
        }
        microsecond = v;
    }

    // Timezone: Z, +HH:MM, or -HH:MM
    if pos >= bytes.len() {
        return Err(parse_err(input, "RFC 3339 (missing timezone)"));
    }

    let tz_offset_secs: i64 = match bytes[pos] {
        b'Z' | b'z' => 0,
        b'+' | b'-' => {
            let sign: i64 = if bytes[pos] == b'+' { 1 } else { -1 };
            pos += 1;
            if pos + 5 > bytes.len() {
                return Err(parse_err(input, "RFC 3339 (incomplete timezone offset)"));
            }
            let tz_hour = parse_digits(bytes, pos, 2, input)? as i64;
            // Colon is optional in some variants
            let tz_min_start = if pos + 2 < bytes.len() && bytes[pos + 2] == b':' {
                pos + 3
            } else {
                pos + 2
            };
            let tz_min = parse_digits(bytes, tz_min_start, 2, input)? as i64;
            sign * (tz_hour * 3600 + tz_min * 60)
        }
        _ => return Err(parse_err(input, "RFC 3339 (expected Z or +/-offset)")),
    };

    // Validate
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

    let dt = DateTime::from_components(crate::components::DateTimeComponents {
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
    })?;

    // Apply timezone offset (subtract to get UTC)
    Ok(dt - crate::duration::Duration::seconds(tz_offset_secs))
}

/// Parse an ISO 8601 date string ("YYYY-MM-DD") into a `Date`.
#[allow(
    clippy::as_conversions,
    reason = "parsed 2-digit values guaranteed to fit u32"
)]
pub fn parse_iso8601_date(input: &str) -> Result<Date, ChronoError> {
    let bytes = input.as_bytes();
    if bytes.len() < 10 {
        return Err(parse_err(input, "ISO 8601 date (YYYY-MM-DD)"));
    }

    let year = parse_digits(bytes, 0, 4, input)?;
    expect_byte(bytes, 4, b'-', input)?;
    let month = parse_digits(bytes, 5, 2, input)? as u32;
    expect_byte(bytes, 7, b'-', input)?;
    let day = parse_digits(bytes, 8, 2, input)? as u32;

    Date::from_ymd(year, month, day)
}

/// Parse a datetime string using a strftime format into a `DateTime`.
///
/// Supports `%Y`, `%m`, `%d`, `%H`, `%M`, `%S`.
#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "parser: indices guarded by loop bounds and length checks, as-casts bounded by digit parsing"
)]
pub fn parse_with_format(input: &str, fmt: &str) -> Result<DateTime, ChronoError> {
    let ibytes = input.as_bytes();
    let fbytes = fmt.as_bytes();
    let mut ipos = 0;
    let mut fpos = 0;

    let mut year: i32 = 1970;
    let mut month: u32 = 1;
    let mut day: u32 = 1;
    let mut hour: u32 = 0;
    let mut minute: u32 = 0;
    let mut second: u32 = 0;

    while fpos < fbytes.len() {
        if fbytes[fpos] == b'%' {
            fpos += 1;
            if fpos >= fbytes.len() {
                break;
            }
            match fbytes[fpos] {
                b'Y' => {
                    year = parse_digits(ibytes, ipos, 4, input)?;
                    ipos += 4;
                }
                b'm' => {
                    month = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'd' => {
                    day = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'H' => {
                    hour = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'M' => {
                    minute = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'S' => {
                    second = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'%' => {
                    if ipos < ibytes.len() && ibytes[ipos] == b'%' {
                        ipos += 1;
                    }
                }
                other => {
                    return Err(ChronoError::InvalidFormat {
                        specifier: other as char,
                    });
                }
            }
        } else {
            // Literal character — must match
            if ipos < ibytes.len() && ibytes[ipos] == fbytes[fpos] {
                ipos += 1;
            } else {
                return Err(parse_err(input, fmt));
            }
        }
        fpos += 1;
    }

    DateTime::from_ymd_hms(year, month, day, hour, minute, second)
}

/// Parse a datetime string using a strftime format into a `NaiveDateTime`.
#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "parser: indices guarded by loop bounds and length checks, as-casts bounded by digit parsing"
)]
pub fn parse_naive_with_format(input: &str, fmt: &str) -> Result<NaiveDateTime, ChronoError> {
    let ibytes = input.as_bytes();
    let fbytes = fmt.as_bytes();
    let mut ipos = 0;
    let mut fpos = 0;

    let mut year: i32 = 1970;
    let mut month: u32 = 1;
    let mut day: u32 = 1;
    let mut hour: u32 = 0;
    let mut minute: u32 = 0;
    let mut second: u32 = 0;

    while fpos < fbytes.len() {
        if fbytes[fpos] == b'%' {
            fpos += 1;
            if fpos >= fbytes.len() {
                break;
            }
            match fbytes[fpos] {
                b'Y' => {
                    year = parse_digits(ibytes, ipos, 4, input)?;
                    ipos += 4;
                }
                b'm' => {
                    month = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'd' => {
                    day = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'H' => {
                    hour = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'M' => {
                    minute = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'S' => {
                    second = parse_digits(ibytes, ipos, 2, input)? as u32;
                    ipos += 2;
                }
                b'%' => {
                    if ipos < ibytes.len() && ibytes[ipos] == b'%' {
                        ipos += 1;
                    }
                }
                other => {
                    return Err(ChronoError::InvalidFormat {
                        specifier: other as char,
                    });
                }
            }
        } else if ipos < ibytes.len() && ibytes[ipos] == fbytes[fpos] {
            ipos += 1;
        } else {
            return Err(parse_err(input, fmt));
        }
        fpos += 1;
    }

    NaiveDateTime::from_ymd_hms(year, month, day, hour, minute, second)
}

// --- Helpers ---

/// Parse `count` ASCII digits starting at `start` into an i32.
#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "bounds checked at entry, digit byte minus b'0' bounded to [0,9], overflow handled by checked_mul/checked_add"
)]
fn parse_digits(bytes: &[u8], start: usize, count: usize, input: &str) -> Result<i32, ChronoError> {
    if start.checked_add(count).is_none_or(|end| end > bytes.len()) {
        return Err(parse_err(input, "digits"));
    }
    let mut value: i32 = 0;
    for b in &bytes[start..start + count] {
        if !b.is_ascii_digit() {
            return Err(parse_err(input, "digits"));
        }
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add((*b - b'0') as i32))
            .ok_or(ChronoError::Overflow)?;
    }
    Ok(value)
}

/// Expect a specific byte at position.
#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    reason = "bounds checked: pos < bytes.len() before access, byte→char safe for ASCII"
)]
fn expect_byte(bytes: &[u8], pos: usize, expected: u8, input: &str) -> Result<(), ChronoError> {
    if pos >= bytes.len() || bytes[pos] != expected {
        return Err(parse_err(
            input,
            &alloc::format!("'{}' at position {pos}", expected as char),
        ));
    }
    Ok(())
}

/// Create a parse error.
fn parse_err(input: &str, expected: &str) -> ChronoError {
    ChronoError::ParseError {
        input: input.to_string(),
        expected: expected.to_string(),
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    // --- RFC 3339 parsing ---

    #[test]
    fn test_parse_rfc3339_basic() {
        let dt = parse_rfc3339("2026-02-25T14:30:00Z").expect("valid");
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 25);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_parse_rfc3339_with_micros() {
        let dt = parse_rfc3339("2026-02-25T14:30:00.123456Z").expect("valid");
        assert_eq!(dt.microsecond(), 123_456);
    }

    #[test]
    fn test_parse_rfc3339_with_millis() {
        let dt = parse_rfc3339("2026-02-25T14:30:00.123Z").expect("valid");
        assert_eq!(dt.microsecond(), 123_000);
    }

    #[test]
    fn test_parse_rfc3339_positive_offset() {
        // +05:00 means 5 hours ahead of UTC, so UTC is 5 hours earlier
        let dt = parse_rfc3339("2026-02-25T19:30:00+05:00").expect("valid");
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_rfc3339_negative_offset() {
        let dt = parse_rfc3339("2026-02-25T09:30:00-05:00").expect("valid");
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_rfc3339_lowercase_z() {
        let dt = parse_rfc3339("2026-02-25T14:30:00z").expect("valid");
        assert_eq!(dt.hour(), 14);
    }

    #[test]
    fn test_parse_rfc3339_epoch() {
        let dt = parse_rfc3339("1970-01-01T00:00:00Z").expect("valid");
        assert_eq!(dt.timestamp(), 0);
    }

    #[test]
    fn test_parse_rfc3339_too_short() {
        assert!(parse_rfc3339("2026-02-25").is_err());
    }

    #[test]
    fn test_parse_rfc3339_invalid_date() {
        assert!(parse_rfc3339("2025-02-29T00:00:00Z").is_err());
    }

    // --- RFC 3339 round-trip ---

    #[test]
    fn test_rfc3339_round_trip() {
        let original = "2026-02-25T14:30:45Z";
        let dt = parse_rfc3339(original).expect("valid");
        assert_eq!(dt.to_rfc3339(), original);
    }

    #[test]
    fn test_rfc3339_round_trip_micros() {
        let original = "2026-02-25T14:30:45.123456Z";
        let dt = parse_rfc3339(original).expect("valid");
        assert_eq!(dt.to_rfc3339(), original);
    }

    // --- ISO 8601 date parsing ---

    #[test]
    fn test_parse_iso8601_date() {
        let d = parse_iso8601_date("2026-02-25").expect("valid");
        assert_eq!(d.year(), 2026);
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 25);
    }

    #[test]
    fn test_parse_iso8601_date_invalid() {
        assert!(parse_iso8601_date("2025-02-29").is_err());
    }

    #[test]
    fn test_parse_iso8601_date_too_short() {
        assert!(parse_iso8601_date("2026-02").is_err());
    }

    // --- Format string parsing ---

    #[test]
    fn test_parse_with_format_full() {
        let dt = parse_with_format("2026-02-25 14:30:45", "%Y-%m-%d %H:%M:%S").expect("valid");
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_parse_with_format_compact() {
        let dt = parse_with_format("20260225143045", "%Y%m%d%H%M%S").expect("valid");
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 25);
    }

    #[test]
    fn test_parse_format_round_trip() {
        let fmt_str = "%Y-%m-%d %H:%M:%S";
        let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 45).expect("valid");
        let formatted = dt.format(fmt_str).expect("ok");
        let parsed = parse_with_format(&formatted, fmt_str).expect("valid");
        assert_eq!(dt, parsed);
    }

    #[test]
    fn test_parse_with_format_mismatch() {
        assert!(parse_with_format("2026/02/25", "%Y-%m-%d").is_err());
    }

    // --- NaiveDateTime parsing ---

    #[test]
    fn test_parse_naive() {
        let ndt =
            parse_naive_with_format("2026-02-25 14:30:00", "%Y-%m-%d %H:%M:%S").expect("valid");
        assert_eq!(ndt.year(), 2026);
        assert_eq!(ndt.hour(), 14);
    }
}
