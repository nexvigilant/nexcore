//! FAERS Therapy (THER) file parsing.

use super::parser::{get_field, parse_faers_date, parse_int_field, parse_str_field};
use serde::{Deserialize, Serialize};

/// Therapy record from FAERS THER file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSTherapy {
    /// Primary ID
    pub primaryid: u64,
    /// Case ID
    pub caseid: u64,
    /// Drug sequence number this therapy belongs to
    pub dsg_drug_seq: u32,
    /// Start date (YYYYMMDD)
    pub start_dt: Option<String>,
    /// End date (YYYYMMDD)
    pub end_dt: Option<String>,
    /// Duration value
    pub dur: Option<String>,
    /// Duration code (YR, MON, WK, DY, HR)
    pub dur_cod: Option<String>,
}

impl FAERSTherapy {
    /// Parse a therapy record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            dsg_drug_seq: parse_int_field(get_field(fields, 2)).unwrap_or(1) as u32,
            start_dt: parse_faers_date(get_field(fields, 3)),
            end_dt: parse_faers_date(get_field(fields, 4)),
            dur: parse_str_field(get_field(fields, 5)),
            dur_cod: parse_str_field(get_field(fields, 6)),
        }
    }
}

/// Parse a THER line into a FAERSTherapy struct.
#[must_use]
pub fn parse_therapy_line(fields: &[&str]) -> FAERSTherapy {
    FAERSTherapy::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_therapy() {
        let line = "12345$67890$1$20230101$20230115$14$DY";
        let fields = split_faers_line(line);
        let therapy = FAERSTherapy::parse(&fields);

        assert_eq!(therapy.primaryid, 12345);
        assert_eq!(therapy.caseid, 67890);
        assert_eq!(therapy.dsg_drug_seq, 1);
        assert_eq!(therapy.start_dt, Some("20230101".to_string()));
        assert_eq!(therapy.end_dt, Some("20230115".to_string()));
        assert_eq!(therapy.dur, Some("14".to_string()));
        assert_eq!(therapy.dur_cod, Some("DY".to_string()));
    }

    #[test]
    fn test_parse_therapy_partial() {
        let line = "12345$67890$1$20230101$$$";
        let fields = split_faers_line(line);
        let therapy = FAERSTherapy::parse(&fields);

        assert_eq!(therapy.start_dt, Some("20230101".to_string()));
        assert_eq!(therapy.end_dt, None);
        assert_eq!(therapy.dur, None);
        assert_eq!(therapy.dur_cod, None);
    }
}
