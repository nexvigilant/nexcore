//! Validation rules for CLI-UI contract enforcement

use super::{error::Result, types::*};
use std::collections::HashSet;

/// Validation rules that enforce the CLI-UI contract
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRule {
    /// Every UI action MUST map to a CLI command
    NoOrphanUi,
    /// Every form field MUST map to a CLI arg/flag
    NoGhostFields,
    /// UI input types MUST match CLI arg types
    TypeParity,
    /// UI required fields = CLI required args
    RequiredMatch,
    /// UI displays only what CLI outputs
    OutputParity,
}

impl ValidationRule {
    /// Get all validation rules
    pub fn all() -> Vec<ValidationRule> {
        vec![
            ValidationRule::NoOrphanUi,
            ValidationRule::NoGhostFields,
            ValidationRule::TypeParity,
            ValidationRule::RequiredMatch,
            ValidationRule::OutputParity,
        ]
    }

    /// Get the description of this rule
    pub fn description(&self) -> &str {
        match self {
            ValidationRule::NoOrphanUi => "Every UI action MUST map to a CLI command",
            ValidationRule::NoGhostFields => "Every form field MUST map to a CLI arg/flag",
            ValidationRule::TypeParity => "UI input types MUST match CLI arg types",
            ValidationRule::RequiredMatch => "UI required fields = CLI required args",
            ValidationRule::OutputParity => "UI displays only what CLI outputs",
        }
    }
}

/// Validator for CLI-UI mappings
pub struct Validator {
    rules: Vec<ValidationRule>,
}

impl Validator {
    /// Create a new validator with all rules enabled
    pub fn new() -> Self {
        Self {
            rules: ValidationRule::all(),
        }
    }

    /// Create a validator with specific rules
    pub fn with_rules(rules: Vec<ValidationRule>) -> Self {
        Self { rules }
    }

    /// Validate a UI mapping against a CLI command
    pub fn validate(&self, cli: &CliCommand, ui: &UiMapping) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        for rule in &self.rules {
            match rule {
                ValidationRule::NoOrphanUi => {
                    self.validate_no_orphan_ui(cli, ui, &mut report)?;
                }
                ValidationRule::NoGhostFields => {
                    self.validate_no_ghost_fields(cli, ui, &mut report)?;
                }
                ValidationRule::TypeParity => {
                    self.validate_type_parity(cli, ui, &mut report)?;
                }
                ValidationRule::RequiredMatch => {
                    self.validate_required_match(cli, ui, &mut report)?;
                }
                ValidationRule::OutputParity => {
                    self.validate_output_parity(cli, ui, &mut report)?;
                }
            }
        }

        Ok(report)
    }

    fn validate_no_orphan_ui(
        &self,
        cli: &CliCommand,
        ui: &UiMapping,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Check if the CLI command path exists
        if ui.cli_command_path.is_empty() {
            report.add_error(
                ValidationRule::NoOrphanUi,
                format!("UI mapping '{}' has no CLI command path", ui.route),
            );
            return Ok(());
        }

        // Try to find the command
        let cmd = cli.find_subcommand(&ui.cli_command_path[1..]);
        if cmd.is_none() {
            report.add_error(
                ValidationRule::NoOrphanUi,
                format!(
                    "UI mapping '{}' references non-existent CLI command '{}'",
                    ui.route,
                    ui.cli_command_path.join(" ")
                ),
            );
        }

        Ok(())
    }

    fn validate_no_ghost_fields(
        &self,
        cli: &CliCommand,
        ui: &UiMapping,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Early return if path is empty (handled by NoOrphanUi)
        if ui.cli_command_path.is_empty() {
            return Ok(());
        }

        // Get the actual CLI command
        let cmd = match cli.find_subcommand(&ui.cli_command_path[1..]) {
            Some(c) => c,
            None => return Ok(()), // Already reported in NoOrphanUi
        };

        // Build a set of valid CLI arg/flag names
        let mut valid_names = HashSet::new();
        for arg in &cmd.args {
            valid_names.insert(arg.name.clone());
        }
        for flag in &cmd.flags {
            valid_names.insert(flag.long.clone());
            if let Some(short) = flag.short {
                valid_names.insert(short.to_string());
            }
        }

        // Check each UI component
        for component in &ui.components {
            if !valid_names.contains(&component.maps_to) {
                report.add_error(
                    ValidationRule::NoGhostFields,
                    format!(
                        "UI component '{}' maps to non-existent CLI arg/flag '{}'",
                        component.label, component.maps_to
                    ),
                );
            }
        }

        Ok(())
    }

    fn validate_type_parity(
        &self,
        cli: &CliCommand,
        ui: &UiMapping,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Early return if path is empty (handled by NoOrphanUi)
        if ui.cli_command_path.is_empty() {
            return Ok(());
        }

        let cmd = match cli.find_subcommand(&ui.cli_command_path[1..]) {
            Some(c) => c,
            None => return Ok(()),
        };

        for component in &ui.components {
            // Find corresponding CLI arg
            let cli_arg = cmd.args.iter().find(|a| a.name == component.maps_to);
            let cli_flag = cmd.flags.iter().find(|f| f.long == component.maps_to);

            let expected_type = if let Some(arg) = cli_arg {
                &arg.arg_type
            } else if let Some(flag) = cli_flag {
                if let Some(ref vt) = flag.value_type {
                    vt
                } else {
                    &ArgType::Bool
                }
            } else {
                continue; // Already reported in NoGhostFields
            };

            // Check if component type matches
            if !self.component_matches_type(&component.component_type, expected_type) {
                report.add_warning(
                    ValidationRule::TypeParity,
                    format!(
                        "UI component '{}' type {:?} may not match CLI type {:?}",
                        component.label, component.component_type, expected_type
                    ),
                );
            }
        }

        Ok(())
    }

    fn validate_required_match(
        &self,
        cli: &CliCommand,
        ui: &UiMapping,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Early return if path is empty (handled by NoOrphanUi)
        if ui.cli_command_path.is_empty() {
            return Ok(());
        }

        let cmd = match cli.find_subcommand(&ui.cli_command_path[1..]) {
            Some(c) => c,
            None => return Ok(()),
        };

        // Check that all required CLI args have corresponding required UI components
        for arg in &cmd.args {
            if arg.required {
                let ui_component = ui.components.iter().find(|c| c.maps_to == arg.name);
                match ui_component {
                    Some(comp) if !comp.required => {
                        report.add_error(
                            ValidationRule::RequiredMatch,
                            format!(
                                "CLI arg '{}' is required but UI component '{}' is not",
                                arg.name, comp.label
                            ),
                        );
                    }
                    None => {
                        report.add_error(
                            ValidationRule::RequiredMatch,
                            format!("Required CLI arg '{}' has no UI component", arg.name),
                        );
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn validate_output_parity(
        &self,
        cli: &CliCommand,
        ui: &UiMapping,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Early return if path is empty (handled by NoOrphanUi)
        if ui.cli_command_path.is_empty() {
            return Ok(());
        }

        let cmd = match cli.find_subcommand(&ui.cli_command_path[1..]) {
            Some(c) => c,
            None => return Ok(()),
        };

        if let Some(ref output_display) = ui.output_display {
            // Build set of valid output fields
            let valid_fields: HashSet<String> = cmd
                .output_type
                .fields
                .iter()
                .map(|f| f.name.clone())
                .collect();

            // Check each field mapping
            for mapping in &output_display.field_mappings {
                if !valid_fields.contains(&mapping.field_name) {
                    report.add_error(
                        ValidationRule::OutputParity,
                        format!(
                            "UI displays field '{}' which is not in CLI output schema",
                            mapping.field_name
                        ),
                    );
                }
            }
        }

        Ok(())
    }

    fn component_matches_type(&self, component_type: &ComponentType, arg_type: &ArgType) -> bool {
        matches!(
            (component_type, arg_type),
            (ComponentType::TextField, ArgType::String)
                | (ComponentType::NumberInput { .. }, ArgType::Int)
                | (ComponentType::NumberInput { .. }, ArgType::Float)
                | (ComponentType::Toggle, ArgType::Bool)
                | (ComponentType::Checkbox, ArgType::Bool)
                | (ComponentType::Select { .. }, ArgType::Enum(_))
                | (ComponentType::Select { .. }, ArgType::String)
                | (ComponentType::MultiSelect { .. }, ArgType::Array(_))
                | (ComponentType::FileUpload { .. }, ArgType::File)
                | (ComponentType::DirectoryPicker, ArgType::Directory)
                | (ComponentType::EmailInput, ArgType::Email)
                | (ComponentType::UrlInput, ArgType::Url)
                | (ComponentType::DatePicker, ArgType::Date)
                | (ComponentType::DateTimePicker, ArgType::DateTime)
                | (ComponentType::JsonEditor, ArgType::Json)
                | (ComponentType::TextArea, ArgType::String)
        )
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// Report of validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub rule: ValidationRule,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub rule: ValidationRule,
    pub message: String,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, rule: ValidationRule, message: String) {
        self.errors.push(ValidationError { rule, message });
    }

    pub fn add_warning(&mut self, rule: ValidationRule, message: String) {
        self.warnings.push(ValidationWarning { rule, message });
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cli() -> CliCommand {
        CliCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
            subcommands: vec![],
            args: vec![CliArg {
                name: "target".to_string(),
                arg_type: ArgType::String,
                required: true,
                default: None,
                validation: None,
                help: "Target".to_string(),
                possible_values: vec![],
            }],
            flags: vec![],
            output_type: OutputSchema {
                type_name: "Result".to_string(),
                fields: vec![],
                format: OutputFormat::Json,
            },
            examples: vec![],
        }
    }

    #[test]
    fn test_validator_required_match() {
        let cli = create_test_cli();
        let ui = UiMapping {
            route: "/test".to_string(),
            page_type: PageType::Form,
            title: "Test".to_string(),
            components: vec![UiComponent {
                component_type: ComponentType::TextField,
                label: "Target".to_string(),
                maps_to: "target".to_string(),
                required: false, // Should be true!
                help: None,
                default: None,
                show_when: None,
            }],
            navigation: NavPosition {
                parent: None,
                order: 0,
                icon: None,
            },
            output_display: None,
            cli_command_path: vec!["test".to_string()],
        };

        let validator = Validator::new();
        let report = validator.validate(&cli, &ui).unwrap();

        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1);
    }
}
