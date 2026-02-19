//! # PVDSL Runtime Values
//!
//! Runtime value types for the PVDSL virtual machine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A runtime value in PVDSL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RuntimeValue {
    /// String value
    String(String),
    /// Numeric value (f64)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// List of values
    List(Vec<RuntimeValue>),
    /// Dictionary/map of values
    Dict(HashMap<String, RuntimeValue>),
    /// Null/none value
    Null,
}

impl RuntimeValue {
    /// Check if the value is truthy
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Null => false,
            RuntimeValue::Number(n) => *n != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::List(l) => !l.is_empty(),
            RuntimeValue::Dict(d) => !d.is_empty(),
        }
    }

    /// Get as number if possible
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            RuntimeValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string if possible
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            RuntimeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as boolean if possible
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RuntimeValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::String(s) => write!(f, "{s}"),
            RuntimeValue::Number(n) => write!(f, "{n}"),
            RuntimeValue::Boolean(b) => write!(f, "{b}"),
            RuntimeValue::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            RuntimeValue::Dict(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            RuntimeValue::Null => write!(f, "null"),
        }
    }
}

impl From<f64> for RuntimeValue {
    fn from(n: f64) -> Self {
        RuntimeValue::Number(n)
    }
}

impl From<i64> for RuntimeValue {
    fn from(n: i64) -> Self {
        RuntimeValue::Number(n as f64)
    }
}

impl From<bool> for RuntimeValue {
    fn from(b: bool) -> Self {
        RuntimeValue::Boolean(b)
    }
}

impl From<String> for RuntimeValue {
    fn from(s: String) -> Self {
        RuntimeValue::String(s)
    }
}

impl From<&str> for RuntimeValue {
    fn from(s: &str) -> Self {
        RuntimeValue::String(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truthiness() {
        assert!(RuntimeValue::Boolean(true).is_truthy());
        assert!(!RuntimeValue::Boolean(false).is_truthy());
        assert!(!RuntimeValue::Null.is_truthy());
        assert!(RuntimeValue::Number(1.0).is_truthy());
        assert!(!RuntimeValue::Number(0.0).is_truthy());
        assert!(RuntimeValue::String("hello".to_string()).is_truthy());
        assert!(!RuntimeValue::String(String::new()).is_truthy());
    }
}
