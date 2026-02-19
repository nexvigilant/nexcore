//! UI mapping generator from CLI commands

use super::{error::Result, types::*};

/// Main mapper that generates UI mappings from CLI commands
pub struct CliUiMapper {
    route_prefix: String,
    auto_navigation: bool,
}

impl CliUiMapper {
    /// Create a new mapper with default settings
    pub fn new() -> Self {
        Self {
            route_prefix: String::new(),
            auto_navigation: true,
        }
    }

    /// Set the route prefix for generated UI routes
    pub fn with_route_prefix(mut self, prefix: String) -> Self {
        self.route_prefix = prefix;
        self
    }

    /// Enable or disable automatic navigation generation
    pub fn with_auto_navigation(mut self, enabled: bool) -> Self {
        self.auto_navigation = enabled;
        self
    }

    /// Generate UI mappings for a CLI command and all its subcommands
    pub fn generate_mappings(&self, cli: &CliCommand) -> Result<Vec<UiMapping>> {
        let mut mappings = Vec::new();

        // Generate mapping for the command itself
        if !cli.subcommands.is_empty() {
            // If it has subcommands, create a dashboard/list page
            let mapping = self.generate_dashboard_mapping(cli, &[])?;
            mappings.push(mapping);
        } else {
            // Leaf command - create a form page
            let mapping = self.generate_form_mapping(cli, &[])?;
            mappings.push(mapping);
        }

        // Generate mappings for all subcommands
        for subcmd in &cli.subcommands {
            let parent_path = vec![cli.name.clone()];
            let sub_mappings = self.generate_subcommand_mappings(subcmd, &parent_path)?;
            mappings.extend(sub_mappings);
        }

        // Check for duplicate routes
        self.check_duplicate_routes(&mappings)?;

        Ok(mappings)
    }

    fn check_duplicate_routes(&self, mappings: &[UiMapping]) -> Result<()> {
        use std::collections::HashSet;
        let mut seen = HashSet::new();

        for mapping in mappings {
            if !seen.insert(&mapping.route) {
                return Err(super::error::MapperError::ValidationError {
                    message: format!("Duplicate route detected: {}", mapping.route),
                });
            }
        }

        Ok(())
    }

    fn generate_subcommand_mappings(
        &self,
        cmd: &CliCommand,
        parent_path: &[String],
    ) -> Result<Vec<UiMapping>> {
        let mut mappings = Vec::new();
        let current_path: Vec<String> = parent_path
            .iter()
            .chain(std::iter::once(&cmd.name))
            .cloned()
            .collect();

        if !cmd.subcommands.is_empty() {
            // Has subcommands - create dashboard
            let mapping = self.generate_dashboard_mapping(cmd, parent_path)?;
            mappings.push(mapping);

            // Recursively generate for subcommands
            for subcmd in &cmd.subcommands {
                let sub_mappings = self.generate_subcommand_mappings(subcmd, &current_path)?;
                mappings.extend(sub_mappings);
            }
        } else {
            // Leaf command - create form
            let mapping = self.generate_form_mapping(cmd, parent_path)?;
            mappings.push(mapping);
        }

        Ok(mappings)
    }

    fn generate_form_mapping(&self, cmd: &CliCommand, parent_path: &[String]) -> Result<UiMapping> {
        let route = self.build_route(parent_path, &cmd.name);
        let cli_command_path = if parent_path.is_empty() {
            vec![cmd.name.clone()]
        } else {
            parent_path
                .iter()
                .chain(std::iter::once(&cmd.name))
                .cloned()
                .collect()
        };

        // Generate UI components from args and flags
        let mut components = Vec::new();

        // Add components for args
        for arg in &cmd.args {
            components.push(self.arg_to_component(arg));
        }

        // Add components for flags
        for flag in &cmd.flags {
            components.push(self.flag_to_component(flag));
        }

        // Generate output display if there are output fields
        let output_display = if !cmd.output_type.fields.is_empty() {
            Some(self.generate_output_display(&cmd.output_type))
        } else {
            None
        };

        Ok(UiMapping {
            route,
            page_type: PageType::Form,
            title: self.generate_title(&cmd.name, &cmd.description),
            components,
            navigation: self.generate_navigation(parent_path, &cmd.name),
            output_display,
            cli_command_path,
        })
    }

    fn generate_dashboard_mapping(
        &self,
        cmd: &CliCommand,
        parent_path: &[String],
    ) -> Result<UiMapping> {
        let route = self.build_route(parent_path, &cmd.name);
        let cli_command_path = if parent_path.is_empty() {
            vec![cmd.name.clone()]
        } else {
            parent_path
                .iter()
                .chain(std::iter::once(&cmd.name))
                .cloned()
                .collect()
        };

        Ok(UiMapping {
            route,
            page_type: PageType::Dashboard,
            title: self.generate_title(&cmd.name, &cmd.description),
            components: vec![],
            navigation: self.generate_navigation(parent_path, &cmd.name),
            output_display: None,
            cli_command_path,
        })
    }

    fn arg_to_component(&self, arg: &CliArg) -> UiComponent {
        let component_type = arg
            .arg_type
            .to_component_type(if arg.possible_values.is_empty() {
                None
            } else {
                Some(arg.possible_values.clone())
            });

        UiComponent {
            component_type,
            label: self.humanize_name(&arg.name),
            maps_to: arg.name.clone(),
            required: arg.required,
            help: Some(arg.help.clone()),
            default: arg.default.clone(),
            show_when: None,
        }
    }

    fn flag_to_component(&self, flag: &CliFlag) -> UiComponent {
        let component_type = if flag.takes_value {
            if let Some(ref vt) = flag.value_type {
                vt.to_component_type(None)
            } else {
                ComponentType::TextField
            }
        } else {
            ComponentType::Toggle
        };

        UiComponent {
            component_type,
            label: self.humanize_name(&flag.long),
            maps_to: flag.long.clone(),
            required: false, // Flags are generally optional
            help: Some(flag.description.clone()),
            default: flag.default_value.clone(),
            show_when: None,
        }
    }

    fn generate_output_display(&self, output: &OutputSchema) -> OutputDisplay {
        let field_mappings = output
            .fields
            .iter()
            .map(|f| {
                let display_type = match f.field_type {
                    ArgType::String => DisplayType::CodeBlock,
                    ArgType::Json => DisplayType::RawJson,
                    _ => DisplayType::ResultCard,
                };

                FieldMapping {
                    field_name: f.name.clone(),
                    display_as: display_type,
                }
            })
            .collect();

        OutputDisplay {
            display_type: match output.format {
                OutputFormat::Json => DisplayType::RawJson,
                OutputFormat::Table => DisplayType::DataTable,
                _ => DisplayType::ResultCard,
            },
            field_mappings,
        }
    }

    fn build_route(&self, parent_path: &[String], name: &str) -> String {
        let path = if parent_path.is_empty() {
            format!("/{}", name)
        } else {
            format!("/{}/{}", parent_path.join("/"), name)
        };

        if self.route_prefix.is_empty() {
            path
        } else {
            format!("{}{}", self.route_prefix, path)
        }
    }

    fn generate_navigation(&self, parent_path: &[String], _name: &str) -> NavPosition {
        NavPosition {
            parent: parent_path.last().cloned(),
            order: 0,
            icon: None,
        }
    }

    fn generate_title(&self, name: &str, description: &str) -> String {
        if description.is_empty() {
            self.humanize_name(name)
        } else {
            description.to_string()
        }
    }

    fn humanize_name(&self, name: &str) -> String {
        // Convert snake_case or kebab-case to Title Case
        name.split(&['-', '_'][..])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for CliUiMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_name() {
        let mapper = CliUiMapper::new();
        assert_eq!(mapper.humanize_name("test_name"), "Test Name");
        assert_eq!(mapper.humanize_name("test-name"), "Test Name");
        assert_eq!(mapper.humanize_name("testname"), "Testname");
    }

    #[test]
    fn test_generate_form_mapping() {
        let mapper = CliUiMapper::new();
        let cli = CliCommand {
            name: "scan".to_string(),
            description: "Scan command".to_string(),
            subcommands: vec![],
            args: vec![CliArg {
                name: "target".to_string(),
                arg_type: ArgType::String,
                required: true,
                default: None,
                validation: None,
                help: "Target to scan".to_string(),
                possible_values: vec![],
            }],
            flags: vec![],
            output_type: OutputSchema {
                type_name: "ScanResult".to_string(),
                fields: vec![],
                format: OutputFormat::Json,
            },
            examples: vec![],
        };

        let mapping = mapper.generate_form_mapping(&cli, &[]).unwrap();
        assert_eq!(mapping.route, "/scan");
        assert_eq!(mapping.page_type, PageType::Form);
        assert_eq!(mapping.components.len(), 1);
        assert_eq!(mapping.components[0].maps_to, "target");
    }
}
