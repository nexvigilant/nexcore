//! # Reflex Layer — pre-LLM stimulus pattern matcher
//!
//! Implements the **Reflex Arc** invariant from anatomy-physiology-as-code:
//! common stimulus patterns are matched and responded to locally without
//! routing to the LLM (Vertex Gemini). Closes Gap #1 in the Vigil gap audit.
//!
//! Biology: Hot stimulus → sensory neuron → spinal interneuron → motor
//! response. Brain learns about it AFTER the reflex completes.
//!
//! Code: webhook event → ReflexLayer::match_stimulus → cytokine verdict
//! emitted directly. LLM invocation is bypassed for matched stimuli.
//! Unmatched stimuli fall through to the existing DecisionEngine path.

use crate::models::Event;
use serde::{Deserialize, Serialize};
use tracing::info;

/// A single bio-response routing rule. Stimulus keywords → cytokine verdict.
///
/// Mirrors `~/vigil/etc/system-prompt.txt`'s routing table so the reflex
/// layer responds with the same verdict format the LLM would have produced.
#[derive(Debug, Clone, Serialize)]
pub struct ReflexRule {
    /// Human-readable name for telemetry.
    pub name: &'static str,
    /// Lowercase substrings; ANY match in the payload text triggers the rule.
    pub keywords: &'static [&'static str],
    /// Cytokine family the rule emits (il1, il10, il6, tnf_alpha, il2).
    pub family: &'static str,
    /// Severity level for the emitted signal (low, medium, high, critical).
    pub severity: &'static str,
    /// VERDICT prefix (e.g. "DOWN", "RECOVERED", "PV_SIGNAL").
    pub verdict_type: &'static str,
}

/// The default ruleset, derived from the bio-response system prompt.
/// Matches the eval scenarios s01-s05 in `~/vigil/training/scenarios.jsonl`.
pub const DEFAULT_RULES: &[ReflexRule] = &[
    ReflexRule {
        name: "service_recovered",
        keywords: &["recovered", "restarted successfully", "back up", "200 ok", "healthy"],
        family: "il10",
        severity: "low",
        verdict_type: "RECOVERED",
    },
    ReflexRule {
        name: "service_down",
        keywords: &["service failed", "consecutive", "health checks", "unreachable", "crashed", "down"],
        family: "il1",
        severity: "critical",
        verdict_type: "DOWN",
    },
    ReflexRule {
        name: "pv_signal",
        keywords: &["prr=", "disproportionality", "faers", "boxed-warning", "ic025=", "ebgm="],
        family: "il6",
        severity: "high",
        verdict_type: "PV_SIGNAL",
    },
    ReflexRule {
        name: "code_antipattern",
        keywords: &["panicked at", "unwrap()", "build failed", "stderr", "expect("],
        family: "tnf_alpha",
        severity: "medium",
        verdict_type: "ANTIPATTERN",
    },
    ReflexRule {
        name: "homeostasis_check",
        keywords: &["how is the system", "homeostasis", "report posture", "system check"],
        family: "il2",
        severity: "low",
        verdict_type: "HOMEOSTASIS",
    },
];

/// Result of a reflex match — drives both telemetry and the verdict emitted
/// in place of the LLM invocation.
#[derive(Debug, Clone, Serialize)]
pub struct ReflexMatch {
    pub rule_name: &'static str,
    pub family: &'static str,
    pub severity: &'static str,
    pub verdict: String,
    pub matched_keyword: &'static str,
}

/// The reflex layer itself. Stateless; rule list is compile-time so matching
/// is O(rules × keywords × payload_len) — typically microseconds.
pub struct ReflexLayer {
    rules: &'static [ReflexRule],
}

impl ReflexLayer {
    /// Construct with the default ruleset.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: DEFAULT_RULES }
    }

    /// Construct with a custom ruleset (useful for tests or swap-in
    /// stricter/laxer rules per deployment).
    #[must_use]
    pub fn with_rules(rules: &'static [ReflexRule]) -> Self {
        Self { rules }
    }

    /// Try to match a stimulus event against the rules.
    ///
    /// Extracts the searchable text from the event payload (msg, text, body,
    /// or full payload as JSON). Lowercases for case-insensitive substring
    /// matching. Returns the FIRST matching rule (rules are ordered such
    /// that more specific rules come first).
    #[must_use]
    pub fn match_stimulus(&self, event: &Event) -> Option<ReflexMatch> {
        let text = extract_text(event);
        if text.is_empty() {
            return None;
        }
        let lowered = text.to_lowercase();

        for rule in self.rules {
            for keyword in rule.keywords {
                if lowered.contains(keyword) {
                    let verdict = format!(
                        "VERDICT: {} family={} severity={}",
                        rule.verdict_type, rule.family, rule.severity
                    );
                    info!(
                        rule = %rule.name,
                        keyword = %keyword,
                        family = %rule.family,
                        "reflex_match"
                    );
                    return Some(ReflexMatch {
                        rule_name: rule.name,
                        family: rule.family,
                        severity: rule.severity,
                        verdict,
                        matched_keyword: keyword,
                    });
                }
            }
        }
        None
    }
}

impl Default for ReflexLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Pull the searchable text out of an event payload. Looks at common keys
/// (`msg`, `text`, `body`, `message`) first; falls back to the full payload
/// serialized as JSON so even unstructured payloads contribute matchable
/// content.
fn extract_text(event: &Event) -> String {
    for key in ["msg", "text", "body", "message"] {
        if let Some(s) = event.payload.get(key).and_then(|v| v.as_str()) {
            return s.to_string();
        }
    }
    serde_json::to_string(&event.payload).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Urgency;
    use nexcore_chrono::DateTime;
    use nexcore_id::NexId;

    fn evt(payload: serde_json::Value) -> Event {
        Event {
            id: NexId::v4(),
            source: "test".into(),
            event_type: "observe".into(),
            payload,
            priority: Urgency::Normal,
            timestamp: DateTime::now(),
            correlation_id: None,
        }
    }

    #[test]
    fn matches_service_down() {
        let layer = ReflexLayer::new();
        let e = evt(serde_json::json!({
            "msg": "nvnc-station.service failed 3 consecutive /health checks"
        }));
        let m = layer.match_stimulus(&e).expect("should match");
        assert_eq!(m.rule_name, "service_down");
        assert_eq!(m.family, "il1");
        assert!(m.verdict.contains("DOWN"));
    }

    #[test]
    fn matches_recovered() {
        let layer = ReflexLayer::new();
        let e = evt(serde_json::json!({
            "msg": "service restarted successfully, /health returns 200"
        }));
        let m = layer.match_stimulus(&e).expect("should match");
        assert_eq!(m.rule_name, "service_recovered");
        assert_eq!(m.family, "il10");
    }

    #[test]
    fn matches_pv_signal() {
        let layer = ReflexLayer::new();
        let e = evt(serde_json::json!({
            "msg": "metformin + lactic acidosis: PRR=29.64 with 18408 cases"
        }));
        let m = layer.match_stimulus(&e).expect("should match");
        assert_eq!(m.rule_name, "pv_signal");
    }

    #[test]
    fn no_match_returns_none() {
        let layer = ReflexLayer::new();
        let e = evt(serde_json::json!({"msg": "completely unrelated content here"}));
        assert!(layer.match_stimulus(&e).is_none());
    }

    #[test]
    fn empty_payload_returns_none() {
        let layer = ReflexLayer::new();
        let e = evt(serde_json::json!({}));
        // Falls back to JSON serialization which is "{}", no keywords match.
        assert!(layer.match_stimulus(&e).is_none());
    }
}
