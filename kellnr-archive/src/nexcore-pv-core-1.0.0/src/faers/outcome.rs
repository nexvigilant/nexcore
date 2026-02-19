//! FAERS Outcome (OUTC) file parsing.

use super::parser::{get_field, parse_int_field};
use crate::faers::OutcomeCode;
use serde::{Deserialize, Serialize};

/// Outcome record from FAERS OUTC file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSOutcome {
    /// Primary ID
    pub primaryid: u64,
    /// Case ID
    pub caseid: u64,
    /// Outcome code string
    pub outc_cod: String,
}

impl FAERSOutcome {
    /// Parse an outcome record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            outc_cod: get_field(fields, 2).to_string(),
        }
    }

    /// Get the parsed outcome code enum.
    #[must_use]
    pub fn outcome_code(&self) -> Option<OutcomeCode> {
        OutcomeCode::from_code(&self.outc_cod)
    }

    /// Check if this outcome indicates death.
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.outc_cod.to_uppercase() == "DE"
    }
}

/// Parse an OUTC line into a FAERSOutcome struct.
#[must_use]
pub fn parse_outcome_line(fields: &[&str]) -> FAERSOutcome {
    FAERSOutcome::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_outcome() {
        let line = "12345$67890$DE";
        let fields = split_faers_line(line);
        let outcome = FAERSOutcome::parse(&fields);

        assert_eq!(outcome.primaryid, 12345);
        assert_eq!(outcome.caseid, 67890);
        assert_eq!(outcome.outc_cod, "DE");
        assert!(outcome.is_fatal());
        assert_eq!(outcome.outcome_code(), Some(OutcomeCode::Death));
    }

    #[test]
    fn test_parse_outcome_hospitalization() {
        let line = "12345$67890$HO";
        let fields = split_faers_line(line);
        let outcome = FAERSOutcome::parse(&fields);

        assert!(!outcome.is_fatal());
        assert_eq!(outcome.outcome_code(), Some(OutcomeCode::Hospitalization));
    }
}
