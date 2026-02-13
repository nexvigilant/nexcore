use serde::{Deserialize, Serialize};
use crate::types::{SkillName, SemVer};

#[derive(Debug, Serialize, Deserialize, Clone, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct SkillMetadata {
    pub name: SkillName,
    pub description: String,
    pub version: SemVer,
    #[serde(rename = "compliance-level")]
    pub compliance_level: Option<String>,
    pub categories: Option<Vec<String>>,
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(rename = "input-schema")]
    pub input_schema: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    MissingField(String),
    InvalidFormat(String),
    EmptyValue(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(s) => write!(f, "Missing required field: {s}"),
            Self::InvalidFormat(s) => write!(f, "Invalid format: {s}"),
            Self::EmptyValue(s) => write!(f, "Field cannot be empty: {s}"),
        }
    }
}

pub fn validate_metadata(meta: &SkillMetadata) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Rule: Description must be substantial
    if meta.description.trim().len() < 10 {
        errors.push(ValidationError::InvalidFormat("description is too short (min 10 chars)".to_string()));
    }

    // Rule: Categories if present must not be empty
    if let Some(ref cats) = meta.categories {
        if cats.is_empty() {
            errors.push(ValidationError::EmptyValue("categories".to_string()));
        }
    }

    // Rule: Input Schema must be non-empty if present
    if let Some(ref schema) = meta.input_schema {
        if schema.trim().is_empty() {
            errors.push(ValidationError::EmptyValue("input-schema".to_string()));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use crate::types::SkillName;

    #[rstest]
    #[case("valid-skill", true)]
    #[case("simple", true)]
    #[case("Invalid Name", false)]
    #[case("UPPERCASE", false)]
    #[case("", false)]
    #[case("  ", false)]
    fn test_skill_name_parsing(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(SkillName::parse(input).is_ok(), expected);
    }

    #[test]
    fn test_validate_valid_metadata() {
        let meta = SkillMetadata {
            name: SkillName::parse("valid-skill").unwrap(),
            description: "A very useful test skill description.".to_string(),
            version: SemVer::parse("1.0.0").unwrap(),
            compliance_level: None,
            categories: None,
            allowed_tools: None,
            input_schema: None,
        };
        assert!(validate_metadata(&meta).is_empty());
    }

    #[test]
    fn test_validate_short_description() {
        let meta = SkillMetadata {
            name: SkillName::parse("valid-skill").unwrap(),
            description: "too short".to_string(),
            version: SemVer::parse("1.0.0").unwrap(),
            compliance_level: None,
            categories: None,
            allowed_tools: None,
            input_schema: None,
        };
        let errors = validate_metadata(&meta);
        assert!(errors.contains(&ValidationError::InvalidFormat("description is too short (min 10 chars)".to_string())));
    }
}