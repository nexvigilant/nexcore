//! strftime-subset formatting.
//!
//! Supports only the 6 specifiers confirmed used in the codebase audit:
//! `%Y`, `%m`, `%d`, `%H`, `%M`, `%S`, plus convenience aliases `%F`, `%T`, `%%`.

use crate::error::ChronoError;
use core::fmt::Write;

/// Format date/time components using a strftime-compatible format string.
///
/// Supported specifiers:
/// - `%Y` — 4-digit year (2026)
/// - `%m` — 2-digit month (01-12)
/// - `%d` — 2-digit day (01-31)
/// - `%H` — 2-digit hour (00-23)
/// - `%M` — 2-digit minute (00-59)
/// - `%S` — 2-digit second (00-59)
/// - `%F` — shorthand for `%Y-%m-%d`
/// - `%T` — shorthand for `%H:%M:%S`
/// - `%%` — literal percent sign
#[allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::too_many_arguments,
    reason = "format parser: index bounded by loop, args mirror strftime component set"
)]
pub fn format_date_components(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    _microsecond: u32,
    fmt: &str,
) -> Result<String, ChronoError> {
    let mut result = String::with_capacity(fmt.len().saturating_add(8));
    let bytes = fmt.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            i += 1;
            if i >= bytes.len() {
                // Trailing percent — just append it
                result.push('%');
                break;
            }
            match bytes[i] {
                b'Y' => write_specifier(&mut result, year, 4)?,
                b'm' => write_specifier_u32(&mut result, month)?,
                b'd' => write_specifier_u32(&mut result, day)?,
                b'H' => write_specifier_u32(&mut result, hour)?,
                b'M' => write_specifier_u32(&mut result, minute)?,
                b'S' => write_specifier_u32(&mut result, second)?,
                b'F' => {
                    // %F = %Y-%m-%d
                    write_specifier(&mut result, year, 4)?;
                    result.push('-');
                    write_specifier_u32(&mut result, month)?;
                    result.push('-');
                    write_specifier_u32(&mut result, day)?;
                }
                b'T' => {
                    // %T = %H:%M:%S
                    write_specifier_u32(&mut result, hour)?;
                    result.push(':');
                    write_specifier_u32(&mut result, minute)?;
                    result.push(':');
                    write_specifier_u32(&mut result, second)?;
                }
                b'%' => {
                    result.push('%');
                }
                other => {
                    return Err(ChronoError::InvalidFormat {
                        specifier: char::from(other),
                    });
                }
            }
        } else {
            result.push(char::from(bytes[i]));
        }
        i += 1;
    }

    Ok(result)
}

/// Write a 4-digit signed integer (year) to the buffer.
fn write_specifier(buf: &mut String, value: i32, width: usize) -> Result<(), ChronoError> {
    write!(buf, "{value:0>width$}").map_err(|_| ChronoError::Overflow)
}

/// Write a 2-digit unsigned integer to the buffer.
fn write_specifier_u32(buf: &mut String, value: u32) -> Result<(), ChronoError> {
    write!(buf, "{value:02}").map_err(|_| ChronoError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(pattern: &str) -> Result<String, ChronoError> {
        format_date_components(2026, 2, 25, 14, 30, 45, 0, pattern)
    }

    #[test]
    fn test_year() {
        assert_eq!(fmt("%Y").expect("ok"), "2026");
    }

    #[test]
    fn test_month() {
        assert_eq!(fmt("%m").expect("ok"), "02");
    }

    #[test]
    fn test_day() {
        assert_eq!(fmt("%d").expect("ok"), "25");
    }

    #[test]
    fn test_hour() {
        assert_eq!(fmt("%H").expect("ok"), "14");
    }

    #[test]
    fn test_minute() {
        assert_eq!(fmt("%M").expect("ok"), "30");
    }

    #[test]
    fn test_second() {
        assert_eq!(fmt("%S").expect("ok"), "45");
    }

    #[test]
    fn test_date_shorthand() {
        assert_eq!(fmt("%F").expect("ok"), "2026-02-25");
    }

    #[test]
    fn test_time_shorthand() {
        assert_eq!(fmt("%T").expect("ok"), "14:30:45");
    }

    #[test]
    fn test_percent_literal() {
        assert_eq!(fmt("%%").expect("ok"), "%");
    }

    // Actual format strings from the codebase

    #[test]
    fn test_codebase_compact_timestamp() {
        assert_eq!(fmt("%Y%m%d-%H%M%S").expect("ok"), "20260225-143045");
    }

    #[test]
    fn test_codebase_underscore_timestamp() {
        assert_eq!(fmt("%Y%m%d_%H%M%S").expect("ok"), "20260225_143045");
    }

    #[test]
    fn test_codebase_no_separator() {
        assert_eq!(fmt("%Y%m%d%H%M%S").expect("ok"), "20260225143045");
    }

    #[test]
    fn test_codebase_display_full() {
        assert_eq!(fmt("%Y-%m-%d %H:%M:%S").expect("ok"), "2026-02-25 14:30:45");
    }

    #[test]
    fn test_codebase_display_no_seconds() {
        assert_eq!(fmt("%Y-%m-%d %H:%M").expect("ok"), "2026-02-25 14:30");
    }

    #[test]
    fn test_codebase_date_only() {
        assert_eq!(fmt("%Y-%m-%d").expect("ok"), "2026-02-25");
    }

    #[test]
    fn test_codebase_iso_with_z() {
        assert_eq!(
            fmt("%Y-%m-%dT%H:%M:%SZ").expect("ok"),
            "2026-02-25T14:30:45Z"
        );
    }

    #[test]
    fn test_codebase_compact_date() {
        assert_eq!(fmt("%Y%m%d").expect("ok"), "20260225");
    }

    #[test]
    fn test_codebase_year_only() {
        assert_eq!(fmt("%Y").expect("ok"), "2026");
    }

    #[test]
    fn test_invalid_specifier() {
        assert!(fmt("%z").is_err());
    }

    #[test]
    fn test_plain_text() {
        assert_eq!(
            format_date_components(2026, 2, 25, 0, 0, 0, 0, "hello").expect("ok"),
            "hello"
        );
    }

    #[test]
    fn test_zero_padded_month() {
        assert_eq!(
            format_date_components(2026, 1, 5, 0, 0, 0, 0, "%Y-%m-%d").expect("ok"),
            "2026-01-05"
        );
    }
}
