//! Serde implementations for nexcore-chrono types.
//!
//! - `DateTime` serializes as RFC 3339 string
//! - `NaiveDateTime` serializes as ISO 8601 without Z suffix
//! - `Date` serializes as ISO 8601 date string
//! - `Duration` serializes as microseconds (i64)

#[cfg(feature = "serde")]
mod impl_serde {
    use crate::date::Date;
    use crate::datetime::DateTime;
    use crate::duration::Duration;
    use crate::naive_datetime::NaiveDateTime;
    use crate::parse::{parse_iso8601_date, parse_rfc3339};
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    // --- DateTime: RFC 3339 ---

    impl Serialize for DateTime {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_rfc3339())
        }
    }

    impl<'de> Deserialize<'de> for DateTime {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct DateTimeVisitor;

            impl Visitor<'_> for DateTimeVisitor {
                type Value = DateTime;

                fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    formatter.write_str("an RFC 3339 datetime string")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<DateTime, E> {
                    parse_rfc3339(value).map_err(de::Error::custom)
                }
            }

            deserializer.deserialize_str(DateTimeVisitor)
        }
    }

    // --- NaiveDateTime: ISO 8601 without Z ---

    impl Serialize for NaiveDateTime {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de> Deserialize<'de> for NaiveDateTime {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct NaiveDateTimeVisitor;

            impl Visitor<'_> for NaiveDateTimeVisitor {
                type Value = NaiveDateTime;

                fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    formatter.write_str("an ISO 8601 datetime string (no timezone)")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<NaiveDateTime, E> {
                    crate::parse::parse_naive_with_format(value, "%Y-%m-%dT%H:%M:%S")
                        .map_err(de::Error::custom)
                }
            }

            deserializer.deserialize_str(NaiveDateTimeVisitor)
        }
    }

    // --- Date: ISO 8601 date ---

    impl Serialize for Date {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_iso8601())
        }
    }

    impl<'de> Deserialize<'de> for Date {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct DateVisitor;

            impl Visitor<'_> for DateVisitor {
                type Value = Date;

                fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    formatter.write_str("an ISO 8601 date string (YYYY-MM-DD)")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<Date, E> {
                    parse_iso8601_date(value).map_err(de::Error::custom)
                }
            }

            deserializer.deserialize_str(DateVisitor)
        }
    }

    // --- Duration: microseconds as i64 ---

    impl Serialize for Duration {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_i64(self.num_microseconds())
        }
    }

    impl<'de> Deserialize<'de> for Duration {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct DurationVisitor;

            impl Visitor<'_> for DurationVisitor {
                type Value = Duration;

                fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    formatter.write_str("a duration in microseconds (i64)")
                }

                fn visit_i64<E: de::Error>(self, value: i64) -> Result<Duration, E> {
                    Ok(Duration::microseconds(value))
                }

                fn visit_u64<E: de::Error>(self, value: u64) -> Result<Duration, E> {
                    let micros = i64::try_from(value).map_err(de::Error::custom)?;
                    Ok(Duration::microseconds(micros))
                }
            }

            deserializer.deserialize_i64(DurationVisitor)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_datetime_serde_round_trip() {
            let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
            let json = serde_json::to_string(&dt).expect("serialize");
            let dt2: DateTime = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(dt, dt2);
        }

        #[test]
        fn test_datetime_serde_format() {
            let dt = DateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
            let json = serde_json::to_string(&dt).expect("serialize");
            assert_eq!(json, "\"2026-02-25T14:30:00Z\"");
        }

        #[test]
        fn test_naive_datetime_serde_round_trip() {
            let ndt = NaiveDateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
            let json = serde_json::to_string(&ndt).expect("serialize");
            let ndt2: NaiveDateTime = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(ndt, ndt2);
        }

        #[test]
        fn test_naive_datetime_serde_format() {
            let ndt = NaiveDateTime::from_ymd_hms(2026, 2, 25, 14, 30, 0).expect("valid");
            let json = serde_json::to_string(&ndt).expect("serialize");
            assert_eq!(json, "\"2026-02-25T14:30:00\"");
        }

        #[test]
        fn test_date_serde_round_trip() {
            let d = Date::from_ymd(2026, 2, 25).expect("valid");
            let json = serde_json::to_string(&d).expect("serialize");
            let d2: Date = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(d, d2);
        }

        #[test]
        fn test_date_serde_format() {
            let d = Date::from_ymd(2026, 2, 25).expect("valid");
            let json = serde_json::to_string(&d).expect("serialize");
            assert_eq!(json, "\"2026-02-25\"");
        }

        #[test]
        fn test_duration_serde_round_trip() {
            let dur = Duration::seconds(3600);
            let json = serde_json::to_string(&dur).expect("serialize");
            let dur2: Duration = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(dur, dur2);
        }

        #[test]
        fn test_duration_serde_format() {
            let dur = Duration::seconds(1);
            let json = serde_json::to_string(&dur).expect("serialize");
            assert_eq!(json, "1000000"); // 1 second = 1_000_000 microseconds
        }

        #[test]
        fn test_duration_negative_serde() {
            let dur = Duration::seconds(-60);
            let json = serde_json::to_string(&dur).expect("serialize");
            let dur2: Duration = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(dur, dur2);
        }
    }
}
