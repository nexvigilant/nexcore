//! FAERS Drug (DRUG) file parsing.

use super::parser::{get_field, parse_faers_date, parse_int_field, parse_str_field};
use serde::{Deserialize, Serialize};

/// Drug record from FAERS DRUG file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSDrug {
    /// Primary ID
    pub primaryid: u64,
    /// Case ID
    pub caseid: u64,
    /// Drug sequence number within case
    pub drug_seq: u32,
    /// Role code (PS, SS, C, I)
    pub role_cod: String,
    /// Drug name as reported
    pub drugname: String,
    /// Product active ingredient
    pub prod_ai: Option<String>,
    /// Validated drug name (V/B/M)
    pub val_vbm: Option<String>,
    /// Route of administration
    pub route: Option<String>,
    /// Dose verbatim
    pub dose_vbm: Option<String>,
    /// Cumulative dose character
    pub cum_dose_chr: Option<String>,
    /// Cumulative dose unit
    pub cum_dose_unit: Option<String>,
    /// Dechallenge (Y, N, U, D)
    pub dechal: Option<String>,
    /// Rechallenge (Y, N, U, D)
    pub rechal: Option<String>,
    /// Lot number
    pub lot_num: Option<String>,
    /// Expiration date
    pub exp_dt: Option<String>,
    /// NDA number
    pub nda_num: Option<String>,
    /// Dose amount
    pub dose_amt: Option<String>,
    /// Dose unit
    pub dose_unit: Option<String>,
    /// Dose form
    pub dose_form: Option<String>,
    /// Dose frequency
    pub dose_freq: Option<String>,
}

impl FAERSDrug {
    /// Parse a drug record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            drug_seq: parse_int_field(get_field(fields, 2)).unwrap_or(1) as u32,
            role_cod: get_field(fields, 3).to_string(),
            drugname: get_field(fields, 4).to_string(),
            prod_ai: parse_str_field(get_field(fields, 5)),
            val_vbm: parse_str_field(get_field(fields, 6)),
            route: parse_str_field(get_field(fields, 7)),
            dose_vbm: parse_str_field(get_field(fields, 8)),
            cum_dose_chr: parse_str_field(get_field(fields, 9)),
            cum_dose_unit: parse_str_field(get_field(fields, 10)),
            dechal: parse_str_field(get_field(fields, 11)),
            rechal: parse_str_field(get_field(fields, 12)),
            lot_num: parse_str_field(get_field(fields, 13)),
            exp_dt: parse_faers_date(get_field(fields, 14)),
            nda_num: parse_str_field(get_field(fields, 15)),
            dose_amt: parse_str_field(get_field(fields, 16)),
            dose_unit: parse_str_field(get_field(fields, 17)),
            dose_form: parse_str_field(get_field(fields, 18)),
            dose_freq: parse_str_field(get_field(fields, 19)),
        }
    }

    /// Check if this drug is a suspect (primary or secondary).
    #[must_use]
    pub fn is_suspect(&self) -> bool {
        let upper = self.role_cod.to_uppercase();
        upper == "PS" || upper == "SS"
    }
}

/// Parse a DRUG line into a FAERSDrug struct.
#[must_use]
pub fn parse_drug_line(fields: &[&str]) -> FAERSDrug {
    FAERSDrug::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_drug() {
        let line = "12345$67890$1$PS$ASPIRIN$ACETYLSALICYLIC ACID$1$ORAL$100 MG$1000$MG$Y$N$LOT123$20251231$NDA123$100$MG$TABLET$QD$";
        let fields = split_faers_line(line);
        let drug = FAERSDrug::parse(&fields);

        assert_eq!(drug.primaryid, 12345);
        assert_eq!(drug.caseid, 67890);
        assert_eq!(drug.drug_seq, 1);
        assert_eq!(drug.role_cod, "PS");
        assert_eq!(drug.drugname, "ASPIRIN");
        assert_eq!(drug.prod_ai, Some("ACETYLSALICYLIC ACID".to_string()));
        assert_eq!(drug.val_vbm, Some("1".to_string()));
        assert_eq!(drug.route, Some("ORAL".to_string()));
        assert_eq!(drug.exp_dt, Some("20251231".to_string()));
        assert_eq!(drug.dose_form, Some("TABLET".to_string()));
        assert!(drug.is_suspect());
    }

    #[test]
    fn test_is_suspect() {
        let mut drug = FAERSDrug::parse(&["12345", "67890", "1", "PS", "DRUG"]);
        assert!(drug.is_suspect());

        drug.role_cod = "SS".to_string();
        assert!(drug.is_suspect());

        drug.role_cod = "C".to_string();
        assert!(!drug.is_suspect());

        drug.role_cod = "I".to_string();
        assert!(!drug.is_suspect());
    }
}
