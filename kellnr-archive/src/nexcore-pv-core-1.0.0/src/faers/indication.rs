//! FAERS Indication (INDI) file parsing.

use super::parser::{get_field, parse_int_field};
use serde::{Deserialize, Serialize};

/// Indication record from FAERS INDI file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSIndication {
    /// Primary ID
    pub primaryid: u64,
    /// Case ID
    pub caseid: u64,
    /// Drug sequence number this indication belongs to
    pub indi_drug_seq: u32,
    /// Indication preferred term (MedDRA PT)
    pub indi_pt: String,
}

impl FAERSIndication {
    /// Parse an indication record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            indi_drug_seq: parse_int_field(get_field(fields, 2)).unwrap_or(1) as u32,
            indi_pt: get_field(fields, 3).to_string(),
        }
    }
}

/// Parse an INDI line into a FAERSIndication struct.
#[must_use]
pub fn parse_indication_line(fields: &[&str]) -> FAERSIndication {
    FAERSIndication::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_indication() {
        let line = "12345$67890$1$Rheumatoid arthritis";
        let fields = split_faers_line(line);
        let indication = FAERSIndication::parse(&fields);

        assert_eq!(indication.primaryid, 12345);
        assert_eq!(indication.caseid, 67890);
        assert_eq!(indication.indi_drug_seq, 1);
        assert_eq!(indication.indi_pt, "Rheumatoid arthritis");
    }

    #[test]
    fn test_parse_indication_multiple_drugs() {
        let line1 = "12345$67890$1$Pain";
        let line2 = "12345$67890$2$Inflammation";

        let fields1 = split_faers_line(line1);
        let fields2 = split_faers_line(line2);

        let ind1 = FAERSIndication::parse(&fields1);
        let ind2 = FAERSIndication::parse(&fields2);

        assert_eq!(ind1.indi_drug_seq, 1);
        assert_eq!(ind2.indi_drug_seq, 2);
        assert_eq!(ind1.indi_pt, "Pain");
        assert_eq!(ind2.indi_pt, "Inflammation");
    }
}
