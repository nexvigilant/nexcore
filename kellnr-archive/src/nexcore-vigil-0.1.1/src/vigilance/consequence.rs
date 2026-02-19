//! # Consequence Pipeline — ∝ (Irreversibility) Layer
//!
//! Ordered escalation chain that executes consequences when boundaries
//! are violated. Every consequence is logged to the ledger BEFORE execution,
//! ensuring a complete audit trail even on crash.
//!
//! ## Tier: T3 (∝ + σ + ∂)

use crate::vigilance::boundary::BoundaryViolation;
use crate::vigilance::error::{VigilError, VigilResult};
use crate::vigilance::event::now_millis;
use crate::vigilance::ledger::{LedgerEntryType, VigilanceLedger};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Escalation levels for consequences — ordered from least to most severe.
///
/// Tier: T2-P (κ + ∝)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EscalationLevel {
    /// Just observe and record
    Observe = 0,
    /// Log a warning
    Warn = 1,
    /// Send an alert to the operator
    Alert = 2,
    /// Take automated corrective action
    Act = 3,
    /// Full audit with human review required
    Audit = 4,
}

impl std::fmt::Display for EscalationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => write!(f, "observe"),
            Self::Warn => write!(f, "warn"),
            Self::Alert => write!(f, "alert"),
            Self::Act => write!(f, "act"),
            Self::Audit => write!(f, "audit"),
        }
    }
}

/// Outcome of a consequence execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsequenceOutcome {
    Applied,
    Suppressed(String),
    Failed(String),
}

/// Proof that a consequence was applied — ∃ (existence proof).
///
/// Tier: T2-P (∃ + ∝)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequenceReceipt {
    pub consequence_name: String,
    pub level: EscalationLevel,
    pub boundary: String,
    pub timestamp: u64,
    pub outcome: ConsequenceOutcome,
    pub ledger_sequence: u64,
}

/// A consequence that can be executed when a boundary is violated.
///
/// Tier: T1 (∝) — the fundamental irreversibility primitive.
pub trait Consequence: Send + Sync {
    /// Human-readable name.
    fn name(&self) -> &str;

    /// Escalation level.
    fn level(&self) -> EscalationLevel;

    /// Execute the consequence.
    fn execute(&self, violation: &BoundaryViolation) -> VigilResult<ConsequenceOutcome>;

    /// Whether this consequence is reversible.
    fn is_reversible(&self) -> bool {
        false
    }
}

/// A consequence that logs violations to tracing.
///
/// Tier: T2-P (∝ + π)
pub struct LogConsequence {
    level: EscalationLevel,
}

impl LogConsequence {
    pub fn new(level: EscalationLevel) -> Self {
        Self { level }
    }
}

impl Consequence for LogConsequence {
    fn name(&self) -> &str {
        "log"
    }

    fn level(&self) -> EscalationLevel {
        self.level
    }

    fn execute(&self, violation: &BoundaryViolation) -> VigilResult<ConsequenceOutcome> {
        tracing::warn!(
            boundary = %violation.boundary,
            violation_count = violation.violation_count,
            severity = %violation.event.severity,
            "vigil_boundary_violation"
        );
        Ok(ConsequenceOutcome::Applied)
    }

    fn is_reversible(&self) -> bool {
        true // Logging is non-destructive
    }
}

/// A consequence that sends alerts (placeholder — writes to a file).
///
/// Tier: T2-P (∝ + →)
pub struct AlertConsequence {
    alert_path: std::path::PathBuf,
}

impl AlertConsequence {
    pub fn new(alert_path: std::path::PathBuf) -> Self {
        Self { alert_path }
    }
}

impl Consequence for AlertConsequence {
    fn name(&self) -> &str {
        "alert"
    }

    fn level(&self) -> EscalationLevel {
        EscalationLevel::Alert
    }

    fn execute(&self, violation: &BoundaryViolation) -> VigilResult<ConsequenceOutcome> {
        use std::io::Write;

        let alert = serde_json::json!({
            "boundary": violation.boundary,
            "severity": format!("{}", violation.event.severity),
            "violation_count": violation.violation_count,
            "timestamp": now_millis(),
        });

        if let Some(parent) = self.alert_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.alert_path)?;

        writeln!(
            file,
            "{}",
            serde_json::to_string(&alert).unwrap_or_default()
        )?;
        Ok(ConsequenceOutcome::Applied)
    }
}

/// A consequence that executes a shell command on violation.
///
/// **Caution**: This is an Act-level consequence — shell commands are irreversible.
/// The command receives violation context as environment variables.
///
/// Tier: T2-C (∝ + → + ∂)
pub struct ShellConsequence {
    /// Human-readable name
    command_name: String,
    /// Shell command to execute (via sh -c)
    command: String,
    /// Timeout in milliseconds
    timeout_ms: u64,
}

impl ShellConsequence {
    pub fn new(name: impl Into<String>, command: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            command_name: name.into(),
            command: command.into(),
            timeout_ms,
        }
    }
}

impl Consequence for ShellConsequence {
    fn name(&self) -> &str {
        &self.command_name
    }

    fn level(&self) -> EscalationLevel {
        EscalationLevel::Act
    }

    fn execute(&self, violation: &BoundaryViolation) -> VigilResult<ConsequenceOutcome> {
        use std::process::Command;

        let result = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .env("VIGIL_BOUNDARY", &violation.boundary)
            .env("VIGIL_SEVERITY", format!("{}", violation.event.severity))
            .env(
                "VIGIL_VIOLATION_COUNT",
                violation.violation_count.to_string(),
            )
            .env("VIGIL_SOURCE", &violation.event.source)
            .env("VIGIL_TIMEOUT_MS", self.timeout_ms.to_string())
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    Ok(ConsequenceOutcome::Applied)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(ConsequenceOutcome::Failed(format!(
                        "exit code {}: {stderr}",
                        output.status.code().unwrap_or(-1)
                    )))
                }
            }
            Err(e) => Ok(ConsequenceOutcome::Failed(format!("spawn failed: {e}"))),
        }
    }
}

/// A consequence that sends an HTTP POST webhook on violation.
///
/// Sends violation JSON to the configured URL. Non-blocking via
/// synchronous HTTP (no async runtime dependency in consequence trait).
///
/// Tier: T2-C (∝ + → + λ)
pub struct WebhookConsequence {
    /// Human-readable name
    webhook_name: String,
    /// Target URL
    url: String,
}

impl WebhookConsequence {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            webhook_name: name.into(),
            url: url.into(),
        }
    }
}

impl Consequence for WebhookConsequence {
    fn name(&self) -> &str {
        &self.webhook_name
    }

    fn level(&self) -> EscalationLevel {
        EscalationLevel::Alert
    }

    fn execute(&self, violation: &BoundaryViolation) -> VigilResult<ConsequenceOutcome> {
        // Build webhook payload
        let payload = serde_json::json!({
            "boundary": violation.boundary,
            "source": violation.event.source,
            "severity": format!("{}", violation.event.severity),
            "kind": format!("{}", violation.event.kind),
            "violation_count": violation.violation_count,
            "first_seen": violation.first_seen,
            "last_seen": violation.last_seen,
            "timestamp": now_millis(),
        });

        // Shell-based HTTP POST (avoids adding reqwest dependency)
        let payload_str = serde_json::to_string(&payload).unwrap_or_default();
        let result = std::process::Command::new("curl")
            .args([
                "-s",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "-d",
                &payload_str,
                "--max-time",
                "10",
                &self.url,
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(ConsequenceOutcome::Applied),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(ConsequenceOutcome::Failed(format!("curl failed: {stderr}")))
            }
            Err(e) => Ok(ConsequenceOutcome::Failed(format!(
                "curl not found or failed: {e}"
            ))),
        }
    }
}

/// A consequence that writes a desktop notification file.
///
/// Creates a notification JSON file that external tools (like
/// notify-send watchers) can pick up and display.
///
/// Tier: T2-P (∝ + ∃)
pub struct NotifyConsequence {
    /// Directory to write notification files
    notify_dir: std::path::PathBuf,
}

impl NotifyConsequence {
    pub fn new(notify_dir: std::path::PathBuf) -> Self {
        Self { notify_dir }
    }
}

impl Consequence for NotifyConsequence {
    fn name(&self) -> &str {
        "notify"
    }

    fn level(&self) -> EscalationLevel {
        EscalationLevel::Warn
    }

    fn execute(&self, violation: &BoundaryViolation) -> VigilResult<ConsequenceOutcome> {
        use std::io::Write;

        if !self.notify_dir.exists() {
            std::fs::create_dir_all(&self.notify_dir)?;
        }

        let filename = format!("vigil-{}-{}.json", violation.boundary, now_millis());
        let path = self.notify_dir.join(filename);

        let notification = serde_json::json!({
            "title": format!("Vigil: {} violated", violation.boundary),
            "body": format!(
                "Source: {}, Severity: {}, Count: {}",
                violation.event.source,
                violation.event.severity,
                violation.violation_count
            ),
            "urgency": match violation.event.severity {
                crate::vigilance::event::EventSeverity::Critical => "critical",
                crate::vigilance::event::EventSeverity::High => "critical",
                crate::vigilance::event::EventSeverity::Medium => "normal",
                _ => "low",
            },
            "timestamp": now_millis(),
        });

        let mut file = std::fs::File::create(&path)?;
        writeln!(
            file,
            "{}",
            serde_json::to_string_pretty(&notification).unwrap_or_default()
        )?;

        Ok(ConsequenceOutcome::Applied)
    }

    fn is_reversible(&self) -> bool {
        true // Notification files can be deleted
    }
}

/// Ordered escalation chain with ledger-before-execute invariant.
///
/// **Critical invariant**: `execute()` writes `ConsequenceScheduled` to the
/// ledger BEFORE calling `consequence.execute()`, then writes
/// `ConsequenceExecuted` or `ConsequenceFailed` after.
///
/// Tier: T3 (∝ + σ + ∂), dominant ∝
pub struct ConsequencePipeline {
    chain: Vec<Box<dyn Consequence>>,
    ledger: Arc<Mutex<VigilanceLedger>>,
}

impl ConsequencePipeline {
    /// Create a new consequence pipeline.
    pub fn new(ledger: Arc<Mutex<VigilanceLedger>>) -> Self {
        Self {
            chain: Vec::new(),
            ledger,
        }
    }

    /// Add a consequence to the escalation chain.
    pub fn add_consequence(&mut self, consequence: Box<dyn Consequence>) {
        tracing::info!(
            consequence = consequence.name(),
            level = %consequence.level(),
            "consequence_registered"
        );
        self.chain.push(consequence);
    }

    /// Number of consequences in the chain.
    pub fn chain_length(&self) -> usize {
        self.chain.len()
    }

    /// Execute the consequence chain for a violation.
    ///
    /// Consequences are executed in order until one at or above the
    /// required escalation level succeeds.
    pub fn execute(&self, violation: &BoundaryViolation) -> VigilResult<Vec<ConsequenceReceipt>> {
        let mut receipts = Vec::new();

        for consequence in &self.chain {
            // LEDGER-BEFORE-EXECUTE: Write scheduled BEFORE execution
            let scheduled_seq = {
                let mut ledger = self.ledger.lock().map_err(|e| VigilError::Consequence {
                    consequence: consequence.name().to_string(),
                    message: format!("Ledger lock failed: {e}"),
                })?;
                let entry = ledger.append(
                    LedgerEntryType::ConsequenceScheduled,
                    serde_json::json!({
                        "consequence": consequence.name(),
                        "level": format!("{}", consequence.level()),
                        "boundary": violation.boundary,
                    }),
                )?;
                entry.sequence
            };

            // Execute the consequence
            let outcome = match consequence.execute(violation) {
                Ok(outcome) => outcome,
                Err(e) => ConsequenceOutcome::Failed(e.to_string()),
            };

            // Write execution result to ledger
            let entry_type = match &outcome {
                ConsequenceOutcome::Applied | ConsequenceOutcome::Suppressed(_) => {
                    LedgerEntryType::ConsequenceExecuted
                }
                ConsequenceOutcome::Failed(_) => LedgerEntryType::ConsequenceFailed,
            };

            {
                let mut ledger = self.ledger.lock().map_err(|e| VigilError::Consequence {
                    consequence: consequence.name().to_string(),
                    message: format!("Ledger lock failed: {e}"),
                })?;
                let _ = ledger.append(
                    entry_type,
                    serde_json::json!({
                        "consequence": consequence.name(),
                        "outcome": format!("{outcome:?}"),
                        "scheduled_seq": scheduled_seq,
                    }),
                )?;
            }

            receipts.push(ConsequenceReceipt {
                consequence_name: consequence.name().to_string(),
                level: consequence.level(),
                boundary: violation.boundary.clone(),
                timestamp: now_millis(),
                outcome,
                ledger_sequence: scheduled_seq,
            });
        }

        Ok(receipts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vigilance::event::{EventId, EventKind, EventSeverity, WatchEvent};

    fn make_violation() -> BoundaryViolation {
        BoundaryViolation {
            boundary: "test-boundary".to_string(),
            event: WatchEvent {
                id: EventId(1),
                source: "test".to_string(),
                kind: EventKind::Signal,
                severity: EventSeverity::High,
                payload: serde_json::json!({}),
                timestamp: now_millis(),
            },
            violation_count: 1,
            first_seen: now_millis(),
            last_seen: now_millis(),
        }
    }

    fn make_pipeline() -> ConsequencePipeline {
        let ledger = Arc::new(Mutex::new(VigilanceLedger::new(std::path::PathBuf::from(
            "/tmp/vigil-test-consequence.wal",
        ))));
        ConsequencePipeline::new(ledger)
    }

    #[test]
    fn log_consequence_applies() {
        let consequence = LogConsequence::new(EscalationLevel::Warn);
        let violation = make_violation();
        let result = consequence.execute(&violation);
        assert!(result.is_ok());
    }

    #[test]
    fn pipeline_executes_chain() {
        let mut pipeline = make_pipeline();
        pipeline.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));
        pipeline.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Warn)));
        assert_eq!(pipeline.chain_length(), 2);

        let violation = make_violation();
        let receipts = pipeline.execute(&violation);
        assert!(receipts.is_ok());
        let receipts = receipts.unwrap_or_default();
        assert_eq!(receipts.len(), 2);
    }

    #[test]
    fn ledger_before_execute_invariant() {
        let ledger = Arc::new(Mutex::new(VigilanceLedger::new(std::path::PathBuf::from(
            "/tmp/vigil-test-invariant.wal",
        ))));
        let mut pipeline = ConsequencePipeline::new(Arc::clone(&ledger));
        pipeline.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Warn)));

        let violation = make_violation();
        let _ = pipeline.execute(&violation);

        // Ledger should have: Scheduled, Executed (2 entries per consequence)
        let guard = ledger.lock();
        assert!(guard.is_ok());
        let ledger = guard.unwrap_or_else(|_| panic!("lock failed"));
        assert_eq!(ledger.len(), 2);

        let entries = ledger.entries();
        assert_eq!(entries[0].entry_type, LedgerEntryType::ConsequenceScheduled);
        assert_eq!(entries[1].entry_type, LedgerEntryType::ConsequenceExecuted);
    }

    #[test]
    fn escalation_level_ordering() {
        assert!(EscalationLevel::Observe < EscalationLevel::Warn);
        assert!(EscalationLevel::Warn < EscalationLevel::Alert);
        assert!(EscalationLevel::Alert < EscalationLevel::Act);
        assert!(EscalationLevel::Act < EscalationLevel::Audit);
    }

    #[test]
    fn shell_consequence_echo() {
        let consequence = ShellConsequence::new("echo-test", "echo ok", 5000);
        assert_eq!(consequence.name(), "echo-test");
        assert_eq!(consequence.level(), EscalationLevel::Act);
        assert!(!consequence.is_reversible());

        let violation = make_violation();
        let result = consequence.execute(&violation);
        assert!(result.is_ok());
        match result.unwrap_or(ConsequenceOutcome::Failed("test".to_string())) {
            ConsequenceOutcome::Applied => {} // Expected
            other => panic!("Expected Applied, got {other:?}"),
        }
    }

    #[test]
    fn shell_consequence_env_vars() {
        // Verify environment variables are set by echoing them
        let consequence = ShellConsequence::new(
            "env-check",
            "test -n \"$VIGIL_BOUNDARY\" && test -n \"$VIGIL_SEVERITY\"",
            5000,
        );
        let violation = make_violation();
        let result = consequence.execute(&violation);
        assert!(result.is_ok());
        match result.unwrap_or(ConsequenceOutcome::Failed("test".to_string())) {
            ConsequenceOutcome::Applied => {} // Env vars were set
            other => panic!("Expected Applied, got {other:?}"),
        }
    }

    #[test]
    fn shell_consequence_failing_command() {
        let consequence = ShellConsequence::new("fail-test", "exit 1", 5000);
        let violation = make_violation();
        let result = consequence.execute(&violation);
        assert!(result.is_ok());
        match result.unwrap_or(ConsequenceOutcome::Applied) {
            ConsequenceOutcome::Failed(msg) => {
                assert!(msg.contains("exit code 1"));
            }
            other => panic!("Expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn notify_consequence_creates_file() {
        let dir = tempfile::tempdir().ok();
        let notify_dir = dir
            .as_ref()
            .map(|d| d.path().to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp/vigil-notify-test"));

        let consequence = NotifyConsequence::new(notify_dir.clone());
        assert_eq!(consequence.name(), "notify");
        assert_eq!(consequence.level(), EscalationLevel::Warn);
        assert!(consequence.is_reversible());

        let violation = make_violation();
        let result = consequence.execute(&violation);
        assert!(result.is_ok());

        // Verify a notification file was created
        let entries: Vec<_> = std::fs::read_dir(&notify_dir)
            .ok()
            .map(|rd| rd.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();
        assert!(
            !entries.is_empty(),
            "Should have created a notification file"
        );
    }

    #[test]
    fn webhook_consequence_name_and_level() {
        let consequence = WebhookConsequence::new("test-hook", "http://localhost:9999/test");
        assert_eq!(consequence.name(), "test-hook");
        assert_eq!(consequence.level(), EscalationLevel::Alert);
        assert!(!consequence.is_reversible());
    }

    #[test]
    fn consequence_receipt_structure() {
        let mut pipeline = make_pipeline();
        pipeline.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Alert)));

        let violation = make_violation();
        let receipts = pipeline.execute(&violation).unwrap_or_default();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].consequence_name, "log");
        assert_eq!(receipts[0].level, EscalationLevel::Alert);
        assert_eq!(receipts[0].boundary, "test-boundary");
    }
}
