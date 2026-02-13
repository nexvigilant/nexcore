//! FAERS Deleted records (DELETE) file parsing.

use super::parser::{get_field, parse_int_field};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Deleted record from FAERS DELETE file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FAERSDelete {
    /// Primary ID of the record to be deleted
    pub primaryid: u64,
    /// Case ID of the record to be deleted
    pub caseid: u64,
}

impl FAERSDelete {
    /// Parse a deleted record from FAERS fields.
    #[must_use]
    pub fn parse(fields: &[&str]) -> Self {
        Self {
            primaryid: parse_int_field(get_field(fields, 0)).unwrap_or(0),
            caseid: parse_int_field(get_field(fields, 1)).unwrap_or(0),
        }
    }
}

/// Load a set of deleted primary IDs from a FAERS DELETE file.
pub fn load_deleted_ids<P: AsRef<Path>>(path: P) -> HashSet<u64> {
    let mut deleted_ids = HashSet::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            return deleted_ids;
        }
    };
    let reader = BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue;
        } // Skip header
        if let Ok(l) = line {
            let fields: Vec<&str> = l.split('$').collect();
            if fields.len() >= 1 {
                if let Ok(id) = fields[0].parse::<u64>() {
                    deleted_ids.insert(id);
                }
            }
        }
    }
    deleted_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_delete() {
        let fields = ["12345", "67890"];
        let del = FAERSDelete::parse(&fields);
        assert_eq!(del.primaryid, 12345);
        assert_eq!(del.caseid, 67890);
    }
}
