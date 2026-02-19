//! FAERS Reaction (REAC) file parsing.

use super::parser::{get_field, parse_int_field, parse_str_field};
use serde::{Deserialize, Serialize};

/// Reaction record from FAERS REAC file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSReaction {
    /// Primary ID
    pub primaryid: u64,
    /// Case ID
    pub caseid: u64,
    /// Preferred term (MedDRA PT)
    pub pt: String,
    /// Drug reaction action taken
    pub drug_rec_act: Option<String>,
}

impl FAERSReaction {
    /// Parse a reaction record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
            pt: get_field(fields, 2).to_string(),
            drug_rec_act: parse_str_field(get_field(fields, 3)),
        }
    }
}

/// Parse a REAC line into a FAERSReaction struct.
#[must_use]
pub fn parse_reaction_line(fields: &[&str]) -> FAERSReaction {
    FAERSReaction::parse(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faers::parser::split_faers_line;

    #[test]
    fn test_parse_reaction() {
        let line = "12345$67890$Headache$Drug withdrawn";
        let fields = split_faers_line(line);
        let reaction = FAERSReaction::parse(&fields);

        assert_eq!(reaction.primaryid, 12345);
        assert_eq!(reaction.caseid, 67890);
        assert_eq!(reaction.pt, "Headache");
        assert_eq!(reaction.drug_rec_act, Some("Drug withdrawn".to_string()));
    }

    #[test]
    fn test_parse_reaction_empty_action() {
        let line = "12345$67890$Nausea$";
        let fields = split_faers_line(line);
        let reaction = FAERSReaction::parse(&fields);

        assert_eq!(reaction.pt, "Nausea");
        assert_eq!(reaction.drug_rec_act, None);
    }
}
