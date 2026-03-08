//! Ensemble (subcommand namespace) builder for Molt.
//!
//! Tcl ensembles group related commands under a namespace prefix:
//! `myns subcmd arg1 arg2`. Molt's `call_subcommand` handles dispatch.
//! This module provides a builder for constructing the subcommand table.

use molt::types::{CommandFunc, Subcommand};

/// Builder for creating a subcommand table.
///
/// # Example (conceptual)
///
/// ```ignore
/// let subs = EnsembleBuilder::new()
///     .add("list", cmd_list)
///     .add("get", cmd_get)
///     .add("set", cmd_set)
///     .build();
/// // Use with interp.call_subcommand(ctx, argv, 1, &subs)
/// ```
pub struct EnsembleBuilder {
    subs: Vec<Subcommand>,
}

impl EnsembleBuilder {
    /// Create a new empty ensemble builder.
    pub fn new() -> Self {
        Self { subs: Vec::new() }
    }

    /// Add a subcommand with a static name and function.
    ///
    /// Note: `name` must be `&'static str` because `Subcommand` requires it.
    #[must_use]
    pub fn add(mut self, name: &'static str, func: CommandFunc) -> Self {
        self.subs.push(Subcommand(name, func));
        self
    }

    /// Build the subcommand table.
    pub fn build(self) -> Vec<Subcommand> {
        self.subs
    }

    /// Return the number of subcommands.
    pub fn len(&self) -> usize {
        self.subs.len()
    }

    /// Check if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.subs.is_empty()
    }
}

impl Default for EnsembleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use molt::Interp;
    use molt::molt_ok;
    use molt::types::{ContextID, MoltResult, Value};

    fn sub_list(_interp: &mut Interp, _ctx: ContextID, _args: &[Value]) -> MoltResult {
        molt_ok!("listed")
    }

    fn sub_get(_interp: &mut Interp, _ctx: ContextID, _args: &[Value]) -> MoltResult {
        molt_ok!("got")
    }

    #[test]
    fn build_ensemble() {
        let subs = EnsembleBuilder::new()
            .add("list", sub_list)
            .add("get", sub_get)
            .build();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].0, "list");
        assert_eq!(subs[1].0, "get");
    }

    #[test]
    fn empty_ensemble() {
        let builder = EnsembleBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }
}
