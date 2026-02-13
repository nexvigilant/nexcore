//! Built-in verification checks.

use crate::{Check, CheckResult, FnCheck};

pub fn file_exists_check() -> impl Check {
    FnCheck::new("file_exists", "SKILL.md file exists", |ctx| {
        if ctx.skill_md_path().exists() {
            CheckResult::passed("SKILL.md found")
        } else {
            CheckResult::failed_with_suggestion(
                format!("SKILL.md not found at {:?}", ctx.skill_md_path()),
                "Create SKILL.md with YAML frontmatter",
            )
        }
    })
}

pub fn yaml_valid_check() -> impl Check {
    FnCheck::new("yaml_valid", "YAML frontmatter is valid", |ctx| {
        match ctx.frontmatter() {
            Ok(_) => CheckResult::passed("YAML frontmatter is valid"),
            Err(e) => CheckResult::failed(format!("Invalid YAML: {e}")),
        }
    })
}
