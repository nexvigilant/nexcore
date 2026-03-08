//! Command registry for bulk registration of Tcl commands.
//!
//! Collects `CommandEntry` structs and registers them with an `Interp`.
//! Supports both plain commands and context commands.

use molt::Interp;
use molt::types::{CommandFunc, ContextID, Value};

/// A single command entry ready for registration.
pub struct CommandEntry {
    /// The Tcl command name.
    pub name: &'static str,

    /// The Rust function implementing the command.
    pub func: CommandFunc,

    /// Optional context ID for stateful commands.
    pub context_id: Option<ContextID>,

    /// Brief help text for the command.
    pub help: &'static str,

    /// Argument signature (e.g., "name ?value?").
    pub args: &'static str,
}

impl CommandEntry {
    /// Create a plain command entry (no context).
    pub fn new(name: &'static str, func: CommandFunc) -> Self {
        Self {
            name,
            func,
            context_id: None,
            help: "",
            args: "",
        }
    }

    /// Create a context command entry.
    pub fn with_context(name: &'static str, func: CommandFunc, ctx: ContextID) -> Self {
        Self {
            name,
            func,
            context_id: Some(ctx),
            help: "",
            args: "",
        }
    }

    /// Set the help text.
    pub fn set_help(&mut self, help: &'static str) -> &mut Self {
        self.help = help;
        self
    }

    /// Set the argument signature.
    pub fn set_args(&mut self, args: &'static str) -> &mut Self {
        self.args = args;
        self
    }
}

/// A collection of command entries for bulk registration.
#[derive(Default)]
pub struct CommandRegistry {
    entries: Vec<CommandEntry>,
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plain command.
    pub fn add(&mut self, name: &'static str, func: CommandFunc) -> &mut CommandEntry {
        self.entries.push(CommandEntry::new(name, func));
        let idx = self.entries.len() - 1;
        &mut self.entries[idx]
    }

    /// Add a context command.
    pub fn add_with_context(
        &mut self,
        name: &'static str,
        func: CommandFunc,
        ctx: ContextID,
    ) -> &mut CommandEntry {
        self.entries
            .push(CommandEntry::with_context(name, func, ctx));
        let idx = self.entries.len() - 1;
        &mut self.entries[idx]
    }

    /// Register all entries with the interpreter.
    pub fn register_all(&self, interp: &mut Interp) {
        for entry in &self.entries {
            if let Some(ctx) = entry.context_id {
                interp.add_context_command(entry.name, entry.func, ctx);
            } else {
                interp.add_command(entry.name, entry.func);
            }
        }
    }

    /// Return the names of all registered commands.
    pub fn names(&self) -> Vec<&'static str> {
        self.entries.iter().map(|e| e.name).collect()
    }

    /// Return an iterator over all entries.
    pub fn entries(&self) -> &[CommandEntry] {
        &self.entries
    }

    /// Return the number of registered commands.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use molt::molt_ok;
    use molt::types::MoltResult;

    fn cmd_hello(_interp: &mut Interp, _ctx: ContextID, _args: &[Value]) -> MoltResult {
        molt_ok!("hello")
    }

    fn cmd_world(_interp: &mut Interp, _ctx: ContextID, _args: &[Value]) -> MoltResult {
        molt_ok!("world")
    }

    #[test]
    fn add_and_list() {
        let mut reg = CommandRegistry::new();
        reg.add("hello", cmd_hello)
            .set_help("Say hello")
            .set_args("");
        reg.add("world", cmd_world);

        assert_eq!(reg.len(), 2);
        assert!(!reg.is_empty());
        assert_eq!(reg.names(), vec!["hello", "world"]);
    }

    #[test]
    fn register_with_interp() {
        let mut reg = CommandRegistry::new();
        reg.add("hello", cmd_hello);

        let mut interp = Interp::empty();
        reg.register_all(&mut interp);

        let names = interp.command_names();
        let name_strs: Vec<&str> = names.iter().map(|v| v.as_str()).collect();
        assert!(name_strs.contains(&"hello"));
    }

    #[test]
    fn empty_registry() {
        let reg = CommandRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }
}
