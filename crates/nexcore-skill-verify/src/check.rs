//! Check trait and types.

use crate::{CheckResult, VerifyContext};

pub trait Check: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, ctx: &mut VerifyContext) -> CheckResult;
}

pub type BoxedCheck = Box<dyn Check>;

pub struct FnCheck<F>
where
    F: Fn(&mut VerifyContext) -> CheckResult + Send + Sync,
{
    name: String,
    description: String,
    check_fn: F,
}

impl<F> FnCheck<F>
where
    F: Fn(&mut VerifyContext) -> CheckResult + Send + Sync,
{
    pub fn new(name: impl Into<String>, description: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            check_fn,
        }
    }
}

impl<F> Check for FnCheck<F>
where
    F: Fn(&mut VerifyContext) -> CheckResult + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn run(&self, ctx: &mut VerifyContext) -> CheckResult {
        (self.check_fn)(ctx)
    }
}
