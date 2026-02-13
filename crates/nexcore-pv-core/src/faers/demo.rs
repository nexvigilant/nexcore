//! FAERS Demographics (DEMO) file parsing.

use super::parser::{
    get_field, parse_faers_date, parse_float_field, parse_int_field, parse_str_field,
};
use serde::{Deserialize, Serialize};

/// Demographics record from FAERS DEMO file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSDemo {
    /// Primary ID (unique per report version)
    pub primaryid: u64,
    /// Case ID (unique per case)
    pub caseid: u64,
    /// Case version number
    pub caseversion: u32,
    /// Initial/Followup code
    pub i_f_code: String,
    /// Event date (YYYYMMDD)
    pub event_dt: Option<String>,
    /// Manufacturer receive date
    pub mfr_dt: Option<String>,
    /// Initial FDA receive date
    pub init_fda_dt: Option<String>,
    /// FDA receive date
    pub fda_dt: Option<String>,
    /// Report code (EXP=Expedited, PER=Periodic, DIR=Direct)
    pub rept_cod: Option<String>,
    /// Authorization number
    pub auth_num: Option<String>,
    /// Manufacturer control number
    pub mfr_num: Option<String>,
    /// Manufacturer/sender
    pub mfr_sndr: Option<String>,
    /// Literature reference
    pub lit_ref: Option<String>,
    /// Age at event
    pub age: Option<f64>,
    /// Age code (YR, MON, WK, DY, HR, DEC)
    pub age_cod: Option<String>,
    /// Age group
    pub age_grp: Option<String>,
    /// Sex (M, F, UNK)
    pub sex: Option<String>,
    /// E-submission flag
    pub e_sub: Option<String>,
    /// Weight
    pub wt: Option<f64>,
    /// Weight code (KG, LBS)
    pub wt_cod: Option<String>,
    /// Report date
    pub rept_dt: Option<String>,
    /// To manufacturer date
    pub to_mfr: Option<String>,
    /// Occupation code
    pub occp_cod: Option<String>,
    /// Reporter country
    pub reporter_country: Option<String>,
    /// Occurrence country
    pub occr_country: Option<String>,
}

impl FAERSDemo {
    /// Parse a demographics record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            caseversion: parse_int_field(get_field(fields, 2)).unwrap_or(1) as u32,
            i_f_code: get_field(fields, 3).to_string(),
            event_dt: parse_faers_date(get_field(fields, 4)),
            mfr_dt: parse_faers_date(get_field(fields, 5)),
            init_fda_dt: parse_faers_date(get_field(fields, 6)),
            fda_dt: parse_faers_date(get_field(fields, 7)),
            rept_cod: parse_str_field(get_field(fields, 8)),
            auth_num: parse_str_field(get_field(fields, 9)),
            mfr_num: parse_str_field(get_field(fields, 10)),
            mfr_sndr: parse_str_field(get_field(fields, 11)),
            lit_ref: parse_str_field(get_field(fields, 12)),
            age: parse_float_field(get_field(fields, 13)),
            age_cod: parse_str_field(get_field(fields, 14)),
            age_grp: parse_str_field(get_field(fields, 15)),
            sex: parse_str_field(get_field(fields, 16)),
            e_sub: parse_str_field(get_field(fields, 17)),
            wt: parse_float_field(get_field(fields, 18)),
            wt_cod: parse_str_field(get_field(fields, 19)),
            rept_dt: parse_faers_date(get_field(fields, 20)),
            to_mfr: parse_faers_date(get_field(fields, 21)),
            occp_cod: parse_str_field(get_field(fields, 22)),
            reporter_country: parse_str_field(get_field(fields, 23)),
            occr_country: parse_str_field(get_field(fields, 24)),
        }
    }
}

/// Parse a DEMO line into a FAERSDemo struct.
#[must_use]
pub fn parse_demo_line(fields: &[&str]) -> FAERSDemo {
    FAERSDemo::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_demo() {
        let line = "12345$67890$1$I$20230115$20230120$20230101$20230125$EXP$$$PFIZER$$45$YR$Adult$F$Y$70$KG$$$$US$US$";
        let fields = split_faers_line(line);
        let demo = FAERSDemo::parse(&fields);

        assert_eq!(demo.primaryid, 12345);
        assert_eq!(demo.caseid, 67890);
        assert_eq!(demo.caseversion, 1);
        assert_eq!(demo.i_f_code, "I");
        assert_eq!(demo.event_dt, Some("20230115".to_string()));
        assert_eq!(demo.rept_cod, Some("EXP".to_string()));
        assert_eq!(demo.age, Some(45.0));
        assert_eq!(demo.age_cod, Some("YR".to_string()));
        assert_eq!(demo.sex, Some("F".to_string()));
        assert_eq!(demo.wt, Some(70.0));
        assert_eq!(demo.wt_cod, Some("KG".to_string()));
        assert_eq!(demo.e_sub, Some("Y".to_string()));
        assert_eq!(demo.reporter_country, Some("US".to_string()));
        assert_eq!(demo.mfr_sndr, Some("PFIZER".to_string()));
    }
}
