//! Parameter validation using JSON Schema.

use crate::{ExecutionError, Result};
use jsonschema::Validator;
use serde_json::Value;

/// Validates parameters against JSON Schema.
pub struct ParameterValidator {
    /// Compiled schema validator.
    validator: Option<Validator>,
}

impl ParameterValidator {
    /// Create a validator from a JSON Schema.
    ///
    /// If schema is None or null, all parameters are accepted.
    pub fn new(schema: Option<&Value>) -> Result<Self> {
        let validator = match schema {
            Some(s) if !s.is_null() => {
                let v = Validator::new(s).map_err(|e| ExecutionError::ValidationFailed {
                    message: format!("Invalid schema: {e}"),
                })?;
                Some(v)
            }
            _ => None,
        };

        Ok(Self { validator })
    }

    /// Validate parameters against the schema.
    ///
    /// Returns Ok(()) if valid, or an error with validation messages.
    pub fn validate(&self, params: &Value) -> Result<()> {
        let Some(validator) = &self.validator else {
            // No schema = accept anything
            return Ok(());
        };

        // Use iter_errors to collect all validation errors
        let errors: Vec<String> = validator
            .iter_errors(params)
            .map(|e| format!("{}: {}", e.instance_path, e))
            .collect();

        if !errors.is_empty() {
            return Err(ExecutionError::ValidationFailed {
                message: errors.join("; "),
            });
        }

        Ok(())
    }

    /// Check if validator has a schema.
    #[must_use]
    pub fn has_schema(&self) -> bool {
        self.validator.is_some()
    }
}

impl Default for ParameterValidator {
    fn default() -> Self {
        Self { validator: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_schema_accepts_all() {
        let validator = ParameterValidator::new(None).unwrap();
        assert!(validator.validate(&json!({"any": "value"})).is_ok());
        assert!(!validator.has_schema());
    }

    #[test]
    fn test_null_schema_accepts_all() {
        let validator = ParameterValidator::new(Some(&Value::Null)).unwrap();
        assert!(validator.validate(&json!({"any": "value"})).is_ok());
    }

    #[test]
    fn test_schema_validation() {
        let schema = json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string" },
                "phase": { "type": "integer", "minimum": 0, "maximum": 4 }
            }
        });

        let validator = ParameterValidator::new(Some(&schema)).unwrap();
        assert!(validator.has_schema());

        // Valid params
        assert!(validator.validate(&json!({"path": "/some/path"})).is_ok());
        assert!(
            validator
                .validate(&json!({"path": "/x", "phase": 2}))
                .is_ok()
        );

        // Missing required
        assert!(validator.validate(&json!({})).is_err());

        // Wrong type
        assert!(validator.validate(&json!({"path": 123})).is_err());

        // Out of range
        assert!(
            validator
                .validate(&json!({"path": "/x", "phase": 10}))
                .is_err()
        );
    }
}
