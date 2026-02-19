//! Verifier with check registry.

use crate::{BoxedCheck, Check, CheckOutcome, CheckResult, FnCheck, VerifyContext};
use std::time::Instant;

pub struct Verifier {
    checks: Vec<BoxedCheck>,
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifier {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn with_standard_checks() -> Self {
        let mut v = Self::new();
        v.register(crate::checks::file_exists_check());
        v.register(crate::checks::yaml_valid_check());
        v
    }

    pub fn register(&mut self, check: impl Check + 'static) {
        self.checks.push(Box::new(check));
    }

    pub fn register_fn<F>(&mut self, name: impl Into<String>, desc: impl Into<String>, f: F)
    where
        F: Fn(&mut VerifyContext) -> CheckResult + Send + Sync + 'static,
    {
        self.register(FnCheck::new(name, desc, f));
    }

    pub fn run(&self, ctx: &mut VerifyContext) -> Vec<CheckOutcome> {
        self.checks
            .iter()
            .map(|check| {
                let start = Instant::now();
                let result = check.run(ctx);
                CheckOutcome::new(check.name(), result, start.elapsed())
            })
            .collect()
    }

    pub fn run_and_exit_code(&self, ctx: &mut VerifyContext) -> (Vec<CheckOutcome>, i32) {
        let outcomes = self.run(ctx);
        let has_failure = outcomes.iter().any(|o| o.result.is_failed());
        (outcomes, if has_failure { 1 } else { 0 })
    }
}
