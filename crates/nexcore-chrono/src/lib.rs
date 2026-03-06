//! Sovereign DateTime engine — UTC timestamps, calendar dates, durations with zero external dependencies.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(any(test, clippy)),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::allow_attributes_without_reason
    )
)]

pub mod calendar;
pub mod components;
pub mod date;
pub mod datetime;
pub mod duration;
pub mod error;
pub mod format;
pub mod naive_datetime;
pub mod parse;

#[cfg(feature = "serde")]
pub mod serde_impl;

// Re-exports for clean consumer imports.
pub use calendar::{
    civil_from_days, days_from_civil, days_in_month, is_leap_year, weekday_from_days,
};
pub use components::{DateComponents, DateTimeComponents, DayOfWeek};
pub use date::Date;
pub use datetime::DateTime;
pub use duration::Duration;
pub use error::ChronoError;
pub use format::format_date_components;
pub use naive_datetime::NaiveDateTime;
pub use parse::{parse_iso8601_date, parse_naive_with_format, parse_rfc3339};
