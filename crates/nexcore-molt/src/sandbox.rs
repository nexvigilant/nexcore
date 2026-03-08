//! Sandbox policy for controlling which Tcl commands are available.
//!
//! Three policies:
//! - `Full` — all 33 standard Molt commands (includes `exit` and `source`)
//! - `Safe` — 30 safe commands (no filesystem or process access)
//! - `Empty` — bare interpreter, no standard commands
//! - `Allowlist` — only explicitly named commands from the standard set

use molt::Interp;

/// Policy controlling which standard Tcl commands are available.
#[derive(Debug, Clone)]
pub enum SandboxPolicy {
    /// All standard Molt commands including `exit` and `source`.
    Full,

    /// Safe subset: excludes `exit`, `source`, `parse`, `pdump`, `pclear`.
    Safe,

    /// Bare interpreter with no standard commands.
    Empty,

    /// Only the explicitly listed commands (from the standard set).
    Allowlist(Vec<String>),
}

/// The 30 standard commands that have no dangerous side effects.
pub const SAFE_COMMANDS: &[&str] = &[
    "append",
    "array",
    "assert_eq",
    "break",
    "catch",
    "continue",
    "dict",
    "error",
    "expr",
    "for",
    "foreach",
    "global",
    "if",
    "incr",
    "info",
    "join",
    "lappend",
    "lindex",
    "list",
    "llength",
    "proc",
    "puts",
    "rename",
    "return",
    "set",
    "string",
    "throw",
    "time",
    "unset",
    "while",
];

/// Commands that are dangerous (filesystem, process, or debug internals).
const UNSAFE_COMMANDS: &[&str] = &["exit", "source", "parse", "pdump", "pclear"];

impl SandboxPolicy {
    /// Create an interpreter according to this policy.
    pub fn create_interp(&self) -> Interp {
        match self {
            Self::Full => Interp::new(),
            Self::Empty => Interp::empty(),
            Self::Safe => {
                let mut interp = Interp::new();
                remove_commands(&mut interp, UNSAFE_COMMANDS);
                interp
            }
            Self::Allowlist(allowed) => {
                let mut interp = Interp::new();
                let all_std: Vec<&str> = SAFE_COMMANDS
                    .iter()
                    .chain(UNSAFE_COMMANDS.iter())
                    .copied()
                    .collect();
                let to_remove: Vec<&str> = all_std
                    .iter()
                    .filter(|cmd| !allowed.iter().any(|a| a == *cmd))
                    .copied()
                    .collect();
                remove_commands(&mut interp, &to_remove);
                interp
            }
        }
    }
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self::Safe
    }
}

/// Remove commands from an interpreter by renaming them to empty string.
///
/// Molt doesn't have a `remove_command` method, but `rename CMD {}` in Tcl
/// convention deletes a command. We execute the rename script to remove them.
///
/// IMPORTANT: `rename` must be removed last — it's the tool we use to remove
/// other commands. If it's deleted early, subsequent removals silently fail.
fn remove_commands(interp: &mut Interp, commands: &[&str]) {
    // Remove `rename` last so we can use it to remove everything else
    let mut deferred_rename = false;
    for cmd in commands {
        if *cmd == "rename" {
            deferred_rename = true;
            continue;
        }
        // Use Tcl's `rename` command to delete — rename to empty string
        let script = format!("rename {cmd} {{}}");
        // Ignore errors — command might not exist.
        // Note: interp.eval is Molt's Tcl script execution method (not JS eval).
        // security-hook: MOLT_TCL_INTERP_EVAL
        let _ = interp.eval(&script);
    }
    if deferred_rename {
        // security-hook: MOLT_TCL_INTERP_EVAL
        let _ = interp.eval("rename rename {}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_has_all_commands() {
        let interp = SandboxPolicy::Full.create_interp();
        let names = interp.command_names();
        // Should have exit and source
        let name_strs: Vec<&str> = names.iter().map(|v| v.as_str()).collect();
        assert!(name_strs.contains(&"exit"));
        assert!(name_strs.contains(&"source"));
        assert!(name_strs.contains(&"expr"));
    }

    #[test]
    fn empty_has_no_commands() {
        let interp = SandboxPolicy::Empty.create_interp();
        let names = interp.command_names();
        assert!(names.is_empty());
    }

    #[test]
    fn safe_excludes_dangerous() {
        let interp = SandboxPolicy::Safe.create_interp();
        let names = interp.command_names();
        let name_strs: Vec<&str> = names.iter().map(|v| v.as_str()).collect();
        assert!(!name_strs.contains(&"exit"));
        assert!(!name_strs.contains(&"source"));
        assert!(name_strs.contains(&"expr"));
        assert!(name_strs.contains(&"set"));
    }

    #[test]
    fn allowlist_only_listed() {
        let policy = SandboxPolicy::Allowlist(vec!["expr".into(), "set".into(), "puts".into()]);
        let interp = policy.create_interp();
        let names = interp.command_names();
        let name_strs: Vec<&str> = names.iter().map(|v| v.as_str()).collect();
        assert!(name_strs.contains(&"expr"));
        assert!(name_strs.contains(&"set"));
        assert!(name_strs.contains(&"puts"));
        assert!(!name_strs.contains(&"exit"));
        assert!(!name_strs.contains(&"source"));
        assert!(!name_strs.contains(&"if"));
    }

    #[test]
    fn default_is_safe() {
        let policy = SandboxPolicy::default();
        let interp = policy.create_interp();
        let names = interp.command_names();
        let name_strs: Vec<&str> = names.iter().map(|v| v.as_str()).collect();
        assert!(!name_strs.contains(&"exit"));
        assert!(name_strs.contains(&"expr"));
    }
}
