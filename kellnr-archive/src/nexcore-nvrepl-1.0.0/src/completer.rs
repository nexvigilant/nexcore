//! Tab completion for NVREPL commands
//!
//! Tier: T3 (NvCompleter grounds to σ Sequence + μ Mapping)

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

/// All known top-level command names for completion
const COMMANDS: &[&str] = &[
    // Guardian (existing)
    "risk",
    "tick",
    "status",
    "reset",
    "originator",
    // Signal detection
    "signal",
    "prr",
    "ror",
    "ic",
    "ebgm",
    // Monitoring
    "health",
    "alerts",
    "sensors",
    "montick",
    // Patient safety
    "triage",
    "priority",
    "escalation",
    // Energy
    "energy",
    "decide",
    // Meta
    "help",
    "exit",
    "quit",
];

/// Tab-completing helper for the NVREPL
pub struct NvCompleter {
    hinter: HistoryHinter,
}

impl NvCompleter {
    pub fn new() -> Self {
        Self {
            hinter: HistoryHinter::new(),
        }
    }
}

impl Completer for NvCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let prefix = &line[..pos];

        // Only complete first word (command name)
        if prefix.contains(' ') {
            return Ok((pos, Vec::new()));
        }

        let lower = prefix.to_lowercase();
        let matches: Vec<Pair> = COMMANDS
            .iter()
            .filter(|cmd| cmd.starts_with(&lower))
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            })
            .collect();

        Ok((0, matches))
    }
}

impl Hinter for NvCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for NvCompleter {}
impl Validator for NvCompleter {}
impl Helper for NvCompleter {}
