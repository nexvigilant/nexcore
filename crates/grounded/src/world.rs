//! World implementations — execute experiments in reality.
//!
//! Grounds to: →(Causality) — cause produces observable effect.

use std::process::Command;

use nexcore_chrono::DateTime;

use crate::GroundedError;
use crate::feedback::{Experiment, Outcome, World};

/// Executes experiments as shell commands.
///
/// The experiment's `description` field is interpreted as a shell command.
/// The outcome captures stdout, stderr, and exit code.
///
/// # Safety
/// This executes arbitrary commands. Use with sandboxing in production.
pub struct BashWorld {
    shell: String,
    /// Reserved for future timeout enforcement.
    #[allow(
        dead_code,
        reason = "timeout enforcement not yet implemented; field preserves the API for when it is"
    )]
    timeout_secs: u64,
}

impl BashWorld {
    /// Create a new BashWorld with the specified shell.
    pub fn new(shell: impl Into<String>, timeout_secs: u64) -> Self {
        Self {
            shell: shell.into(),
            timeout_secs,
        }
    }

    /// Create with default zsh shell and 30s timeout.
    pub fn default_zsh() -> Self {
        Self {
            shell: "zsh".into(),
            timeout_secs: 30,
        }
    }
}

impl World for BashWorld {
    fn execute(&self, experiment: &Experiment) -> Result<Outcome, GroundedError> {
        let output = Command::new(&self.shell)
            .args(["-c", &experiment.description])
            .output()
            .map_err(|e| GroundedError::ExperimentFailed(format!("exec: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        Ok(Outcome {
            observation: if success {
                stdout.trim().to_string()
            } else {
                format!("FAILED (exit {exit_code}): {}", stderr.trim())
            },
            conclusive: true,
            evidence: serde_json::json!({
                "stdout": stdout.trim(),
                "stderr": stderr.trim(),
                "exit_code": exit_code,
                "success": success,
                "command": experiment.description,
            }),
            observed_at: DateTime::now(),
        })
    }
}

/// A world that always returns a predetermined outcome.
/// Useful for testing and simulation.
pub struct MockWorld {
    outcomes: Vec<Outcome>,
    index: std::cell::Cell<usize>,
}

impl MockWorld {
    /// Create a mock world that cycles through the given outcomes.
    pub fn new(outcomes: Vec<Outcome>) -> Self {
        Self {
            outcomes,
            index: std::cell::Cell::new(0),
        }
    }

    /// Create a mock world that always succeeds.
    pub fn always_succeeds() -> Self {
        Self::new(vec![Outcome {
            observation: "success".into(),
            conclusive: true,
            evidence: serde_json::json!({"mock": true}),
            observed_at: DateTime::now(),
        }])
    }

    /// Create a mock world that always fails.
    pub fn always_fails() -> Self {
        Self::new(vec![Outcome {
            observation: "failure".into(),
            conclusive: true,
            evidence: serde_json::json!({"mock": true, "success": false}),
            observed_at: DateTime::now(),
        }])
    }
}

impl World for MockWorld {
    fn execute(&self, _experiment: &Experiment) -> Result<Outcome, GroundedError> {
        if self.outcomes.is_empty() {
            return Err(GroundedError::ExperimentFailed(
                "mock world has no outcomes".into(),
            ));
        }
        let idx = self.index.get() % self.outcomes.len();
        self.index.set(idx + 1);
        self.outcomes
            .get(idx)
            .cloned()
            .ok_or_else(|| GroundedError::ExperimentFailed("mock index out of range".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::Experiment;

    #[test]
    fn bash_world_echo() {
        let world = BashWorld::new("bash", 10);
        let exp = Experiment {
            description: "echo hello".into(),
            hypothesis_claim: "echo works".into(),
            expected_if_true: "hello".into(),
            expected_if_false: "error".into(),
        };
        let outcome = world.execute(&exp);
        assert!(outcome.is_ok());
        let outcome = outcome.unwrap_or_else(|_| Outcome {
            observation: String::new(),
            conclusive: false,
            evidence: serde_json::Value::Null,
            observed_at: DateTime::now(),
        });
        assert_eq!(outcome.observation, "hello");
        assert!(outcome.conclusive);
    }

    #[test]
    fn bash_world_failing_command() {
        let world = BashWorld::new("bash", 10);
        let exp = Experiment {
            description: "exit 1".into(),
            hypothesis_claim: "should fail".into(),
            expected_if_true: "n/a".into(),
            expected_if_false: "exit 1".into(),
        };
        let outcome = world.execute(&exp);
        assert!(outcome.is_ok());
        let outcome = outcome.unwrap_or_else(|_| Outcome {
            observation: String::new(),
            conclusive: false,
            evidence: serde_json::Value::Null,
            observed_at: DateTime::now(),
        });
        assert!(outcome.observation.contains("FAILED"));
    }

    #[test]
    fn mock_world_cycles() {
        let world = MockWorld::new(vec![
            Outcome {
                observation: "first".into(),
                conclusive: true,
                evidence: serde_json::json!({}),
                observed_at: DateTime::now(),
            },
            Outcome {
                observation: "second".into(),
                conclusive: false,
                evidence: serde_json::json!({}),
                observed_at: DateTime::now(),
            },
        ]);

        let exp = Experiment {
            description: "test".into(),
            hypothesis_claim: "test".into(),
            expected_if_true: "".into(),
            expected_if_false: "".into(),
        };

        let o1 = world.execute(&exp).unwrap_or_else(|_| Outcome {
            observation: String::new(),
            conclusive: false,
            evidence: serde_json::Value::Null,
            observed_at: DateTime::now(),
        });
        assert_eq!(o1.observation, "first");

        let o2 = world.execute(&exp).unwrap_or_else(|_| Outcome {
            observation: String::new(),
            conclusive: false,
            evidence: serde_json::Value::Null,
            observed_at: DateTime::now(),
        });
        assert_eq!(o2.observation, "second");
    }
}
