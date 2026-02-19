//! Verification context.

use crate::VerifyError;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct VerifyContext {
    skill_path: PathBuf,
    skill_md_path: PathBuf,
    frontmatter: Option<serde_yaml::Value>,
    verbose: bool,
}

impl VerifyContext {
    pub fn new(skill_path: impl Into<PathBuf>) -> Self {
        let skill_path = skill_path.into();
        let skill_md_path = skill_path.join("SKILL.md");
        Self {
            skill_path,
            skill_md_path,
            frontmatter: None,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn skill_path(&self) -> &PathBuf {
        &self.skill_path
    }
    pub fn skill_md_path(&self) -> &PathBuf {
        &self.skill_md_path
    }
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn frontmatter(&mut self) -> Result<&serde_yaml::Value, VerifyError> {
        if self.frontmatter.is_none() {
            self.frontmatter = Some(self.load_frontmatter()?);
        }
        self.frontmatter
            .as_ref()
            .ok_or_else(|| VerifyError::InvalidYaml {
                message: "Failed to load frontmatter".into(),
            })
    }

    fn load_frontmatter(&self) -> Result<serde_yaml::Value, VerifyError> {
        let content =
            std::fs::read_to_string(&self.skill_md_path).map_err(|e| VerifyError::ReadError {
                path: self.skill_md_path.display().to_string(),
                source: e,
            })?;

        // Extract YAML between --- markers
        let mut lines = content.lines();
        if lines.next() != Some("---") {
            return Err(VerifyError::InvalidYaml {
                message: "Missing opening ---".into(),
            });
        }

        let yaml_lines: Vec<&str> = lines.take_while(|l| *l != "---").collect();
        let yaml_str = yaml_lines.join("\n");

        serde_yaml::from_str(&yaml_str).map_err(|e| VerifyError::InvalidYaml {
            message: e.to_string(),
        })
    }
}
