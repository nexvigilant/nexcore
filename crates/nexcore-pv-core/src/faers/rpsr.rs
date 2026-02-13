//! FAERS Report Source (RPSR) file parsing.

use super::parser::{get_field, parse_int_field};
use serde::{Deserialize, Serialize};

/// Report source record from FAERS RPSR file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSRpsr {
    /// Primary ID
    pub primaryid: u64,
    /// Case ID
    pub caseid: u64,
    /// Report source code (e.g., HP=Health Professional)
    pub rpsr_cod: String,
}

impl FAERSRpsr {
    /// Parse a report source record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            rpsr_cod: get_field(fields, 2).to_string(),
        }
    }
}

/// Parse an RPSR line into a FAERSRpsr struct.
#[must_use]
pub fn parse_rpsr_line(fields: &[&str]) -> FAERSRpsr {
    FAERSRpsr::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_rpsr() {
        let line = "12345$67890$HP";
        let fields = split_faers_line(line);
        let rpsr = FAERSRpsr::parse(&fields);

        assert_eq!(rpsr.primaryid, 12345);
        assert_eq!(rpsr.caseid, 67890);
        assert_eq!(rpsr.rpsr_cod, "HP");
    }
}
