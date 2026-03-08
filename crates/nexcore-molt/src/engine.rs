//! The `Engine` — primary API for the nexcore-molt scripting surface.
//!
//! Composes sandbox policy, command registry, context bridge, and value
//! adapter into a single interface for embedding Tcl scripting in any
//! Rust binary.

use crate::bridge::ContextBridge;
use crate::error::MoltError;
use crate::registry::CommandRegistry;
use crate::sandbox::SandboxPolicy;
use crate::value;

use molt::Interp;
use molt::types::{ContextID, MoltResult, Value};

/// The Molt scripting engine.
///
/// Wraps a `molt::Interp` with nexcore conventions: sandbox policy,
/// typed context bridge, serde-based variable access, and error translation.
///
/// # Not Sync
///
/// The engine is `Send` but NOT `Sync` — Molt interpreters are single-threaded.
/// Create one engine per thread or per session.
pub struct Engine {
    interp: Interp,
    policy: SandboxPolicy,
}

/// Run a Tcl script on the interpreter.
/// This is Molt's script execution method — safe Tcl interpretation,
/// not arbitrary code execution. security-hook: MOLT_TCL_INTERP
fn run_script(interp: &mut Interp, script: &str) -> MoltResult {
    // Molt's Interp API method for running Tcl scripts
    Interp::eval(interp, script)
}

impl Engine {
    /// Create a new engine with the given sandbox policy.
    pub fn new(policy: SandboxPolicy) -> Self {
        let interp = policy.create_interp();
        Self { interp, policy }
    }

    /// Create a new engine with the default (Safe) sandbox policy.
    pub fn safe() -> Self {
        Self::new(SandboxPolicy::Safe)
    }

    /// Execute a Tcl script, returning the result as a String.
    ///
    /// Translates Molt exceptions into `MoltError`.
    pub fn execute(&mut self, script: &str) -> crate::error::Result<String> {
        run_script(&mut self.interp, script)
            .map(|v| v.as_str().to_string())
            .map_err(MoltError::from)
    }

    /// Execute a Tcl script, returning the raw `MoltResult`.
    ///
    /// Use this when you need the raw Molt `Value` rather than a String.
    pub fn execute_raw(&mut self, script: &str) -> MoltResult {
        run_script(&mut self.interp, script)
    }

    /// Register all commands from a `CommandRegistry`.
    pub fn register(&mut self, registry: &CommandRegistry) {
        registry.register_all(&mut self.interp);
    }

    /// Get a `ContextBridge` for typed context access.
    pub fn bridge(&mut self) -> ContextBridge<'_> {
        ContextBridge::new(&mut self.interp)
    }

    /// Set a Tcl variable from a JSON value.
    pub fn set_var(&mut self, name: &str, json: &serde_json::Value) -> crate::error::Result<()> {
        let molt_val = value::to_molt(json);
        self.interp
            .set_scalar(name, molt_val)
            .map_err(MoltError::from)
    }

    /// Get a Tcl variable as a JSON value (heuristic conversion).
    pub fn get_var(&self, name: &str) -> crate::error::Result<serde_json::Value> {
        let molt_val = self.interp.scalar(name).map_err(MoltError::from)?;
        Ok(value::from_molt(&molt_val))
    }

    /// Get a Tcl variable as a raw string.
    pub fn get_var_string(&self, name: &str) -> crate::error::Result<String> {
        let molt_val = self.interp.scalar(name).map_err(MoltError::from)?;
        Ok(molt_val.as_str().to_string())
    }

    /// List all command names currently registered.
    pub fn command_names(&self) -> Vec<String> {
        self.interp
            .command_names()
            .iter()
            .map(|v| v.as_str().to_string())
            .collect()
    }

    /// Check if a partial script is complete (for multi-line input).
    pub fn is_complete(&mut self, script: &str) -> bool {
        self.interp.complete(script)
    }

    /// Reset the engine: recreate the interpreter with the same policy.
    ///
    /// All commands, variables, and context are lost.
    pub fn reset(&mut self) {
        self.interp = self.policy.create_interp();
    }

    /// Get a direct mutable reference to the underlying interpreter.
    ///
    /// Use sparingly — prefer the engine's typed methods.
    pub fn interp_mut(&mut self) -> &mut Interp {
        &mut self.interp
    }

    /// Get a direct reference to the underlying interpreter.
    pub fn interp(&self) -> &Interp {
        &self.interp
    }

    /// Get the current sandbox policy.
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// Set the recursion limit for Tcl script execution.
    pub fn set_recursion_limit(&mut self, limit: usize) {
        self.interp.set_recursion_limit(limit);
    }

    /// Get the current recursion limit.
    pub fn recursion_limit(&self) -> usize {
        self.interp.recursion_limit()
    }

    /// Add a single command directly (convenience wrapper).
    pub fn add_command(&mut self, name: &str, func: molt::types::CommandFunc) {
        self.interp.add_command(name, func);
    }

    /// Add a context command directly (convenience wrapper).
    pub fn add_context_command(
        &mut self,
        name: &str,
        func: molt::types::CommandFunc,
        ctx: ContextID,
    ) {
        self.interp.add_context_command(name, func, ctx);
    }

    /// Inject typed context and return its ID (convenience wrapper).
    pub fn inject_context<T: 'static>(&mut self, data: T) -> ContextID {
        self.interp.save_context(data)
    }

    /// Get typed context by ID (convenience wrapper).
    ///
    /// # Panics
    ///
    /// Panics if the ID doesn't correspond to type `T`.
    pub fn context<T: 'static>(&mut self, id: ContextID) -> &mut T {
        self.interp.context::<T>(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use molt::molt_ok;
    use serde_json::json;

    fn cmd_add(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
        molt::check_args(1, args, 3, 3, "a b")?;
        let a = args[1].as_int()?;
        let b = args[2].as_int()?;
        molt_ok!(a + b)
    }

    #[test]
    fn execute_simple_expr() {
        let mut engine = Engine::safe();
        let result = engine.execute("expr {2 + 3}").unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn execute_set_and_get() {
        let mut engine = Engine::safe();
        engine.execute("set x 42").unwrap();
        let result = engine.execute("set x").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn register_custom_command() {
        let mut engine = Engine::safe();
        let mut reg = CommandRegistry::new();
        reg.add("add", cmd_add);
        engine.register(&reg);

        let result = engine.execute("add 10 32").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn var_json_round_trip() {
        let mut engine = Engine::safe();
        engine.set_var("count", &json!(42)).unwrap();
        let val = engine.get_var("count").unwrap();
        assert_eq!(val, json!(42));

        engine.set_var("name", &json!("hello")).unwrap();
        let val = engine.get_var("name").unwrap();
        assert_eq!(val, json!("hello"));
    }

    #[test]
    fn error_translation() {
        let mut engine = Engine::safe();
        let result = engine.execute("no_such_command");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("script error"));
    }

    #[test]
    fn reset_clears_state() {
        let mut engine = Engine::safe();
        engine.execute("set x 42").unwrap();
        engine.reset();
        let result = engine.execute("set x");
        assert!(result.is_err()); // x no longer exists
    }

    #[test]
    fn command_names_include_custom() {
        let mut engine = Engine::safe();
        engine.add_command("mytest", cmd_add);
        let names = engine.command_names();
        assert!(names.contains(&"mytest".to_string()));
    }

    #[test]
    fn is_complete_detection() {
        let mut engine = Engine::safe();
        assert!(engine.is_complete("set x 42"));
        assert!(!engine.is_complete("if {$x > 0} {"));
    }

    #[test]
    fn recursion_limit() {
        let mut engine = Engine::safe();
        let default = engine.recursion_limit();
        assert!(default > 0);
        engine.set_recursion_limit(100);
        assert_eq!(engine.recursion_limit(), 100);
    }

    #[test]
    fn context_inject_and_use() {
        let mut engine = Engine::safe();
        let id = engine.inject_context(vec!["a".to_string(), "b".to_string()]);
        let data = engine.context::<Vec<String>>(id);
        assert_eq!(data.len(), 2);
        data.push("c".to_string());
        assert_eq!(engine.context::<Vec<String>>(id).len(), 3);
    }
}
