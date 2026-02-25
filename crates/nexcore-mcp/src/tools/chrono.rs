//! Chrono MCP Tools — sovereign datetime engine
//!
//! 4 tools exposing `nexcore_chrono` to Claude Code:
//!
//! | Tool | Purpose |
//! |------|---------|
//! | `chrono_now` | Current UTC timestamp in all formats |
//! | `chrono_parse` | Parse a datetime string (RFC 3339, ISO 8601, or with format) |
//! | `chrono_format` | Format a Unix timestamp using a strftime pattern |
//! | `chrono_duration` | Create and inspect a duration |
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Current time | State | ς |
//! | Parse → components | Mapping | μ |
//! | Format output | Sequence | σ |
//! | Duration arithmetic | Quantity | N |

use crate::params::chrono::{ChronoDurationParams, ChronoFormatParams, ChronoParseParams};
use nexcore_chrono::{
    ChronoError, DateTime, Duration, parse_iso8601_date, parse_naive_with_format, parse_rfc3339,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// chrono_now
// ============================================================================

/// Return the current UTC timestamp in multiple formats.
///
/// No parameters required. Returns RFC 3339, Unix seconds, Unix milliseconds,
/// and broken-down year/month/day/hour/minute/second components.
pub fn chrono_now() -> Result<CallToolResult, McpError> {
    let now = DateTime::now();
    let c = now.components();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "rfc3339": now.to_rfc3339(),
            "unix_seconds": now.timestamp(),
            "unix_millis": now.timestamp_millis(),
            "unix_micros": now.timestamp_micros(),
            "components": {
                "year": c.year,
                "month": c.month,
                "day": c.day,
                "hour": c.hour,
                "minute": c.minute,
                "second": c.second,
                "microsecond": c.microsecond,
            },
        })
        .to_string(),
    )]))
}

// ============================================================================
// chrono_parse
// ============================================================================

/// Parse a datetime string and return its components.
///
/// Auto-detection order (when no `format` is provided):
/// 1. RFC 3339 / ISO 8601 datetime (e.g. "2026-02-25T14:30:00Z")
/// 2. ISO 8601 date only (e.g. "2026-02-25") — time defaults to midnight UTC
///
/// When `format` is provided, the input is parsed as a naive datetime using the
/// supplied strftime pattern (e.g. "%Y-%m-%d %H:%M:%S").
pub fn chrono_parse(params: ChronoParseParams) -> Result<CallToolResult, McpError> {
    let input = &params.input;

    // Attempt to resolve the input to a UTC DateTime.
    let parse_outcome = try_parse(input, params.format.as_deref());

    match parse_outcome {
        Ok(dt) => {
            let c = dt.components();
            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "input": input,
                    "rfc3339": dt.to_rfc3339(),
                    "unix_seconds": dt.timestamp(),
                    "unix_millis": dt.timestamp_millis(),
                    "components": {
                        "year": c.year,
                        "month": c.month,
                        "day": c.day,
                        "hour": c.hour,
                        "minute": c.minute,
                        "second": c.second,
                        "microsecond": c.microsecond,
                    },
                })
                .to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(
            json!({
                "error": "parse_failed",
                "input": input,
                "detail": e.to_string(),
                "hint": "Accepted formats: RFC 3339 (2026-02-25T14:30:00Z), ISO 8601 date (2026-02-25), or provide 'format' with a strftime pattern",
            })
            .to_string(),
        )])),
    }
}

/// Try to parse `input` as a UTC `DateTime`.
///
/// When `fmt` is `Some`, uses explicit strftime pattern via `parse_naive_with_format`
/// (result treated as UTC). When `fmt` is `None`, tries RFC 3339 then ISO 8601 date.
fn try_parse(input: &str, fmt: Option<&str>) -> Result<DateTime, ChronoError> {
    if let Some(format_str) = fmt {
        // Explicit format: parse as naive then promote to UTC.
        let naive = parse_naive_with_format(input, format_str)?;
        Ok(naive.to_datetime())
    } else {
        // Auto-detect RFC 3339 first.
        if let Ok(dt) = parse_rfc3339(input) {
            return Ok(dt);
        }
        // Fallback: ISO 8601 date only — midnight UTC.
        let date = parse_iso8601_date(input)?;
        DateTime::from_ymd_hms(date.year(), date.month(), date.day(), 0, 0, 0)
    }
}

// ============================================================================
// chrono_format
// ============================================================================

/// Format a Unix timestamp using a strftime-compatible pattern.
///
/// When `timestamp` is omitted, the current UTC time is used.
/// Supported specifiers: `%Y`, `%m`, `%d`, `%H`, `%M`, `%S`, `%F`, `%T`, `%%`.
pub fn chrono_format(params: ChronoFormatParams) -> Result<CallToolResult, McpError> {
    let dt = params
        .timestamp
        .map(DateTime::from_timestamp)
        .unwrap_or_else(DateTime::now);

    match dt.format(&params.format) {
        Ok(formatted) => Ok(CallToolResult::success(vec![Content::text(
            json!({
                "formatted": formatted,
                "format": params.format,
                "unix_seconds": dt.timestamp(),
                "rfc3339": dt.to_rfc3339(),
            })
            .to_string(),
        )])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(
            json!({
                "error": "format_failed",
                "format": params.format,
                "detail": e.to_string(),
                "hint": "Supported specifiers: %Y %m %d %H %M %S %F %T %%",
            })
            .to_string(),
        )])),
    }
}

// ============================================================================
// chrono_duration
// ============================================================================

/// Create a duration from a value and unit, returning it expressed in all units.
///
/// Accepted units: "seconds", "minutes", "hours", "days", "weeks".
/// Negative values produce negative durations.
pub fn chrono_duration(params: ChronoDurationParams) -> Result<CallToolResult, McpError> {
    let duration = match params.unit.as_str() {
        "seconds" => Duration::seconds(params.value),
        "minutes" => Duration::minutes(params.value),
        "hours" => Duration::hours(params.value),
        "days" => Duration::days(params.value),
        "weeks" => Duration::weeks(params.value),
        other => {
            return Ok(CallToolResult::error(vec![Content::text(
                json!({
                    "error": "unknown_unit",
                    "unit": other,
                    "accepted": ["seconds", "minutes", "hours", "days", "weeks"],
                })
                .to_string(),
            )]));
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "input": {
                "value": params.value,
                "unit": params.unit,
            },
            "total": {
                "microseconds": duration.num_microseconds(),
                "milliseconds": duration.num_milliseconds(),
                "seconds": duration.num_seconds(),
                "minutes": duration.num_minutes(),
                "hours": duration.num_hours(),
                "days": duration.num_days(),
            },
            "display": duration.to_string(),
            "is_zero": duration.is_zero(),
        })
        .to_string(),
    )]))
}
