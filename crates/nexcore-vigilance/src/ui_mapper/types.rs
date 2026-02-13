//! Core types for CLI-UI mapping

use serde::{Deserialize, Serialize};
/// Represents a complete CLI command structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliCommand {
    /// Command name (e.g., "scan", "guardian")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Nested subcommands
    #[serde(default)]
    pub subcommands: Vec<CliCommand>,

    /// Positional or named arguments
    #[serde(default)]
    pub args: Vec<CliArg>,

    /// Boolean or value flags
    #[serde(default)]
    pub flags: Vec<CliFlag>,

    /// Expected output schema
    pub output_type: OutputSchema,

    /// Examples of usage
    #[serde(default)]
    pub examples: Vec<String>,
}

/// CLI argument definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliArg {
    /// Argument name
    pub name: String,

    /// Data type
    pub arg_type: ArgType,

    /// Whether this argument is required
    pub required: bool,

    /// Default value if not provided
    #[serde(default)]
    pub default: Option<String>,

    /// Validation pattern (regex or custom)
    #[serde(default)]
    pub validation: Option<String>,

    /// Help text
    pub help: String,

    /// Possible values (for enums)
    #[serde(default)]
    pub possible_values: Vec<String>,
}

/// CLI flag definition (boolean or value-taking)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliFlag {
    /// Short form (e.g., 'v' for -v)
    pub short: Option<char>,

    /// Long form (e.g., "verbose" for --verbose)
    pub long: String,

    /// Description
    pub description: String,

    /// Whether the flag takes a value
    pub takes_value: bool,

    /// Value type if takes_value is true
    #[serde(default)]
    pub value_type: Option<ArgType>,

    /// Default value if flag is provided without value
    #[serde(default)]
    pub default_value: Option<String>,
}

/// Argument data types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArgType {
    String,
    Int,
    Float,
    Bool,
    /// Enum with specific allowed values
    Enum(Vec<String>),
    /// File path
    File,
    /// Directory path
    Directory,
    /// URL
    Url,
    /// Email address
    Email,
    /// Date (ISO 8601)
    Date,
    /// DateTime (ISO 8601)
    DateTime,
    /// JSON object
    Json,
    /// Array of another type
    Array(Box<ArgType>),
}

/// Output schema definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputSchema {
    /// Output type name
    pub type_name: String,

    /// Fields in the output
    pub fields: Vec<OutputField>,

    /// Output format (JSON, Table, Text, etc.)
    pub format: OutputFormat,
}

/// Individual output field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputField {
    pub name: String,
    pub field_type: ArgType,
    pub description: String,
    #[serde(default)]
    pub optional: bool,
}

/// Output format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Text,
    Yaml,
    Custom(String),
}

/// UI mapping for a CLI command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiMapping {
    /// Route in the UI (e.g., "/guardian/scan")
    pub route: String,

    /// Page type
    pub page_type: PageType,

    /// Page title
    pub title: String,

    /// UI components that map to CLI args/flags
    pub components: Vec<UiComponent>,

    /// Navigation position
    pub navigation: NavPosition,

    /// Output display configuration
    pub output_display: Option<OutputDisplay>,

    /// Original CLI command path
    pub cli_command_path: Vec<String>,
}

/// Page types in the UI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PageType {
    /// Form for inputting command arguments
    Form,
    /// Dashboard showing status/overview
    Dashboard,
    /// List of items
    List,
    /// Detail view of a single item
    Detail,
    /// Settings/configuration page
    Settings,
}

/// UI component that maps to CLI arg/flag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiComponent {
    /// Component type
    pub component_type: ComponentType,

    /// Display label
    pub label: String,

    /// Maps to CLI arg or flag name
    pub maps_to: String,

    /// Whether the field is required
    pub required: bool,

    /// Help text
    #[serde(default)]
    pub help: Option<String>,

    /// Default value
    #[serde(default)]
    pub default: Option<String>,

    /// Conditional display rules
    #[serde(default)]
    pub show_when: Option<String>,
}

/// UI component types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentType {
    TextField,
    NumberInput { min: Option<f64>, max: Option<f64> },
    Select { options: Vec<String> },
    MultiSelect { options: Vec<String> },
    Toggle,
    Checkbox,
    FileUpload { accept: Option<String> },
    DirectoryPicker,
    EmailInput,
    UrlInput,
    DatePicker,
    DateTimePicker,
    JsonEditor,
    TextArea,
}

/// Navigation position in the UI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NavPosition {
    /// Parent navigation item
    pub parent: Option<String>,

    /// Display order
    pub order: usize,

    /// Icon name (optional)
    pub icon: Option<String>,
}

/// Output display configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputDisplay {
    /// Display type
    pub display_type: DisplayType,

    /// Field mappings to display components
    pub field_mappings: Vec<FieldMapping>,
}

/// Display types for output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisplayType {
    ResultCard,
    DataTable,
    StatusBadge,
    Timeline,
    CodeBlock,
    RawJson,
    Custom(String),
}

/// Maps output field to display component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldMapping {
    pub field_name: String,
    pub display_as: DisplayType,
}

impl CliCommand {
    /// Get the full command path as a vector
    pub fn command_path(&self) -> Vec<String> {
        vec![self.name.clone()]
    }

    /// Get all subcommands recursively
    pub fn all_subcommands(&self) -> Vec<&CliCommand> {
        let mut result = vec![];
        for sub in &self.subcommands {
            result.push(sub);
            result.extend(sub.all_subcommands());
        }
        result
    }

    /// Find a subcommand by path
    pub fn find_subcommand(&self, path: &[String]) -> Option<&CliCommand> {
        if path.is_empty() {
            return Some(self);
        }

        for sub in &self.subcommands {
            if sub.name == path[0] {
                return sub.find_subcommand(&path[1..]);
            }
        }

        None
    }
}

impl ArgType {
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &ArgType) -> bool {
        match (self, other) {
            (ArgType::String, ArgType::String) => true,
            (ArgType::Int, ArgType::Int) => true,
            (ArgType::Float, ArgType::Float) => true,
            (ArgType::Bool, ArgType::Bool) => true,
            (ArgType::File, ArgType::File) => true,
            (ArgType::Directory, ArgType::Directory) => true,
            (ArgType::Url, ArgType::Url) => true,
            (ArgType::Email, ArgType::Email) => true,
            (ArgType::Date, ArgType::Date) => true,
            (ArgType::DateTime, ArgType::DateTime) => true,
            (ArgType::Json, ArgType::Json) => true,
            (ArgType::Enum(a), ArgType::Enum(b)) => a == b,
            (ArgType::Array(a), ArgType::Array(b)) => a.is_compatible_with(b),
            // Allow some implicit conversions
            (ArgType::Int, ArgType::Float) => true,
            (ArgType::Float, ArgType::Int) => false, // Loss of precision
            _ => false,
        }
    }

    /// Get the corresponding UI component type
    pub fn to_component_type(&self, options: Option<Vec<String>>) -> ComponentType {
        match self {
            ArgType::String => ComponentType::TextField,
            ArgType::Int => ComponentType::NumberInput {
                min: None,
                max: None,
            },
            ArgType::Float => ComponentType::NumberInput {
                min: None,
                max: None,
            },
            ArgType::Bool => ComponentType::Toggle,
            ArgType::Enum(values) => ComponentType::Select {
                options: options.unwrap_or_else(|| values.clone()),
            },
            ArgType::File => ComponentType::FileUpload { accept: None },
            ArgType::Directory => ComponentType::DirectoryPicker,
            ArgType::Url => ComponentType::UrlInput,
            ArgType::Email => ComponentType::EmailInput,
            ArgType::Date => ComponentType::DatePicker,
            ArgType::DateTime => ComponentType::DateTimePicker,
            ArgType::Json => ComponentType::JsonEditor,
            ArgType::Array(_) => ComponentType::MultiSelect {
                options: options.unwrap_or_default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_type_compatibility() {
        assert!(ArgType::String.is_compatible_with(&ArgType::String));
        assert!(ArgType::Int.is_compatible_with(&ArgType::Float));
        assert!(!ArgType::Float.is_compatible_with(&ArgType::Int));
    }

    #[test]
    fn test_command_path() {
        let cmd = CliCommand {
            name: "guardian".to_string(),
            description: "Guardian commands".to_string(),
            subcommands: vec![],
            args: vec![],
            flags: vec![],
            output_type: OutputSchema {
                type_name: "Result".to_string(),
                fields: vec![],
                format: OutputFormat::Json,
            },
            examples: vec![],
        };

        assert_eq!(cmd.command_path(), vec!["guardian"]);
    }
}
