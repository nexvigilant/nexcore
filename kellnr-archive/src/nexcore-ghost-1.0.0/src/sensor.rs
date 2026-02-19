//! # Ghost Sensor — PII Leak Detection
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | ∂ Boundary | PII leak patterns are boundary violations |
//! | σ Sequence | Signals accumulate in ordered sequence |
//!
//! ## Tier: T2-P (PiiLeakPattern), T3 (GhostSensor, GhostSignal)
//!
//! Returns `Vec<GhostSignal>` (crate-local), NOT `Signal<T>` (guardian-engine).
//! The adapter in guardian-engine wraps GhostSignal into its own type.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::mode::GhostMode;
use crate::scrubber::PiiFieldPattern;

/// Patterns that indicate a PII leak in output data.
///
/// ## Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PiiLeakPattern {
    /// Raw identifier found where pseudonym expected.
    RawIdentifier {
        /// Field name containing the raw identifier.
        field: String,
    },
    /// Email address pattern detected in output.
    EmailPattern {
        /// Field name containing the email pattern.
        field: String,
    },
    /// Fine-grained location data in output (lat/lon to >2 decimal places).
    FineGrainedLocation {
        /// Field name containing the location.
        field: String,
    },
    /// National identifier (SSN, passport) in output.
    NationalIdentifier {
        /// Field name containing the national ID.
        field: String,
    },
    /// Phone number pattern in output.
    PhonePattern {
        /// Field name containing the phone number.
        field: String,
    },
    /// Unredacted date of birth in output.
    DateOfBirth {
        /// Field name containing the DOB.
        field: String,
    },
}

impl PiiLeakPattern {
    /// Severity weight for triage ordering (higher = more severe).
    #[must_use]
    pub const fn severity_weight(&self) -> u32 {
        match self {
            Self::NationalIdentifier { .. } => 1000,
            Self::RawIdentifier { .. } => 800,
            Self::EmailPattern { .. } => 600,
            Self::PhonePattern { .. } => 500,
            Self::DateOfBirth { .. } => 400,
            Self::FineGrainedLocation { .. } => 300,
        }
    }
}

impl fmt::Display for PiiLeakPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawIdentifier { field } => write!(f, "PII_LEAK:raw_id({field})"),
            Self::EmailPattern { field } => write!(f, "PII_LEAK:email({field})"),
            Self::FineGrainedLocation { field } => write!(f, "PII_LEAK:geo({field})"),
            Self::NationalIdentifier { field } => write!(f, "PII_LEAK:natl_id({field})"),
            Self::PhonePattern { field } => write!(f, "PII_LEAK:phone({field})"),
            Self::DateOfBirth { field } => write!(f, "PII_LEAK:dob({field})"),
        }
    }
}

/// A signal emitted when PII is detected in output.
///
/// ## Tier: T3 (∂ + σ + ς)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostSignal {
    /// The leak pattern detected.
    pub pattern: PiiLeakPattern,
    /// The ghost mode that was active when detected.
    pub mode: GhostMode,
    /// ISO 8601 timestamp of detection.
    pub detected_at: String,
    /// Brief description of the detection context.
    pub context: String,
}

impl fmt::Display for GhostSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GHOST_SIGNAL[{}]: {} ({})",
            self.mode, self.pattern, self.context
        )
    }
}

/// DAMP signal emitter for PII leak patterns.
///
/// Scans field maps for PII that should have been scrubbed.
/// Adapter-ready: returns `Vec<GhostSignal>`, not guardian `Signal<T>`.
///
/// ## Tier: T3 (∂ + σ + ς + μ)
#[derive(Debug)]
pub struct GhostSensor {
    mode: GhostMode,
    signals: Vec<GhostSignal>,
}

impl GhostSensor {
    /// Create a new sensor for the given mode.
    #[must_use]
    pub fn new(mode: GhostMode) -> Self {
        Self {
            mode,
            signals: Vec::new(),
        }
    }

    /// Scan a field map for PII leaks.
    ///
    /// Returns the number of new signals detected.
    pub fn scan(&mut self, fields: &std::collections::HashMap<String, String>) -> usize {
        if !self.mode.is_active() {
            return 0;
        }

        let mut count = 0;
        for (name, value) in fields {
            if let Some(signal) = self.check_field(name, value) {
                // Dedup: skip if we already have a signal for same field + pattern
                let dominated = self
                    .signals
                    .iter()
                    .any(|s| format!("{}", s.pattern) == format!("{}", signal.pattern));
                if !dominated {
                    self.signals.push(signal);
                    count += 1;
                }
            }
        }
        count
    }

    /// Check a single field for PII leak.
    fn check_field(&self, name: &str, value: &str) -> Option<GhostSignal> {
        // Skip already-scrubbed values
        if value == "[REDACTED]" || value == "[GENERALIZED]" || value.starts_with("PSEUDO_") {
            return None;
        }

        // Check each PII pattern
        for pattern in &PiiFieldPattern::ALL {
            if pattern.matches(name) {
                let leak = match pattern {
                    PiiFieldPattern::Name => PiiLeakPattern::RawIdentifier {
                        field: name.to_string(),
                    },
                    PiiFieldPattern::Email => PiiLeakPattern::EmailPattern {
                        field: name.to_string(),
                    },
                    PiiFieldPattern::Phone => PiiLeakPattern::PhonePattern {
                        field: name.to_string(),
                    },
                    PiiFieldPattern::Address | PiiFieldPattern::GeoLocation => {
                        PiiLeakPattern::FineGrainedLocation {
                            field: name.to_string(),
                        }
                    }
                    PiiFieldPattern::DateOfBirth => PiiLeakPattern::DateOfBirth {
                        field: name.to_string(),
                    },
                    PiiFieldPattern::NationalId => PiiLeakPattern::NationalIdentifier {
                        field: name.to_string(),
                    },
                    PiiFieldPattern::IpAddress => PiiLeakPattern::RawIdentifier {
                        field: name.to_string(),
                    },
                };
                return Some(GhostSignal {
                    pattern: leak,
                    mode: self.mode,
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    context: format!("field '{name}' contains unscrubbed PII value"),
                });
            }
        }
        None
    }

    /// Drain all accumulated signals.
    pub fn drain(&mut self) -> Vec<GhostSignal> {
        std::mem::take(&mut self.signals)
    }

    /// Number of pending signals.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.signals.len()
    }

    /// Whether any signals are pending.
    #[must_use]
    pub fn has_signals(&self) -> bool {
        !self.signals.is_empty()
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn leaked_fields() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("patient_name".to_string(), "John Doe".to_string());
        m.insert("email".to_string(), "john@example.com".to_string());
        m.insert("drug_name".to_string(), "Aspirin".to_string());
        m
    }

    fn clean_fields() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("patient_name".to_string(), "[REDACTED]".to_string());
        m.insert("email".to_string(), "[REDACTED]".to_string());
        m.insert("drug_name".to_string(), "Aspirin".to_string());
        m
    }

    #[test]
    fn detects_pii_leak() {
        let mut sensor = GhostSensor::new(GhostMode::Standard);
        let count = sensor.scan(&leaked_fields());
        assert!(count > 0);
        assert!(sensor.has_signals());
    }

    #[test]
    fn no_leak_in_clean_fields() {
        let mut sensor = GhostSensor::new(GhostMode::Standard);
        let count = sensor.scan(&clean_fields());
        assert_eq!(count, 0);
    }

    #[test]
    fn off_mode_no_detection() {
        let mut sensor = GhostSensor::new(GhostMode::Off);
        let count = sensor.scan(&leaked_fields());
        assert_eq!(count, 0);
    }

    #[test]
    fn drain_clears_signals() {
        let mut sensor = GhostSensor::new(GhostMode::Standard);
        sensor.scan(&leaked_fields());
        let signals = sensor.drain();
        assert!(!signals.is_empty());
        assert_eq!(sensor.pending_count(), 0);
    }

    #[test]
    fn dedup_same_field() {
        let mut sensor = GhostSensor::new(GhostMode::Standard);
        sensor.scan(&leaked_fields());
        let first_count = sensor.pending_count();
        sensor.scan(&leaked_fields());
        // Should not add duplicates
        assert_eq!(sensor.pending_count(), first_count);
    }

    #[test]
    fn severity_ordering() {
        let natl = PiiLeakPattern::NationalIdentifier {
            field: "ssn".into(),
        };
        let email = PiiLeakPattern::EmailPattern {
            field: "email".into(),
        };
        assert!(natl.severity_weight() > email.severity_weight());
    }

    #[test]
    fn signal_display() {
        let signal = GhostSignal {
            pattern: PiiLeakPattern::RawIdentifier {
                field: "name".into(),
            },
            mode: GhostMode::Strict,
            detected_at: "2026-01-01T00:00:00Z".into(),
            context: "test".into(),
        };
        let s = format!("{signal}");
        assert!(s.contains("GHOST_SIGNAL"));
        assert!(s.contains("Strict"));
    }

    #[test]
    fn leak_pattern_display() {
        let p = PiiLeakPattern::EmailPattern {
            field: "reporter_email".into(),
        };
        assert_eq!(format!("{p}"), "PII_LEAK:email(reporter_email)");
    }

    #[test]
    fn new_sensor_no_signals() {
        let sensor = GhostSensor::new(GhostMode::Standard);
        assert!(!sensor.has_signals());
        assert_eq!(sensor.pending_count(), 0);
    }

    #[test]
    fn non_pii_fields_ignored() {
        let mut fields = HashMap::new();
        fields.insert("drug_name".to_string(), "Aspirin".to_string());
        fields.insert("event_term".to_string(), "Headache".to_string());
        let mut sensor = GhostSensor::new(GhostMode::Maximum);
        let count = sensor.scan(&fields);
        assert_eq!(count, 0);
    }
}
