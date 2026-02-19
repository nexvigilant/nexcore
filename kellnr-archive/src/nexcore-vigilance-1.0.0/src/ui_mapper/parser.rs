//! CLI specification parser

use super::{error::Result, types::*};
use std::fs;
use std::path::Path;

/// Parser for CLI specifications from various sources
pub struct CliParser;

impl CliParser {
    /// Parse CLI spec from YAML file
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<CliCommand> {
        let content = fs::read_to_string(path)?;
        Self::from_yaml_str(&content)
    }

    /// Parse CLI spec from YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<CliCommand> {
        serde_yaml::from_str(yaml).map_err(|e| e.into())
    }

    /// Parse CLI spec from JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<CliCommand> {
        let content = fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Parse CLI spec from JSON string
    pub fn from_json_str(json: &str) -> Result<CliCommand> {
        serde_json::from_str(json).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
name: test
description: Test command
subcommands: []
args:
  - name: target
    arg_type: String
    required: true
    help: Target argument
    possible_values: []
flags: []
output_type:
  type_name: Result
  fields: []
  format: Json
examples: []
"#;

        let cmd = CliParser::from_yaml_str(yaml).unwrap();
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.args.len(), 1);
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{
  "name": "test",
  "description": "Test command",
  "subcommands": [],
  "args": [
    {
      "name": "target",
      "arg_type": "String",
      "required": true,
      "help": "Target argument",
      "possible_values": []
    }
  ],
  "flags": [],
  "output_type": {
    "type_name": "Result",
    "fields": [],
    "format": "Json"
  },
  "examples": []
}"#;

        let cmd = CliParser::from_json_str(json).unwrap();
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.args.len(), 1);
    }
}
