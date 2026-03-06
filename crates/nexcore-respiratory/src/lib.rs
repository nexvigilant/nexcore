//! # NexVigilant Core — Respiratory System
//!
//! I/O management: intake from external sources, filter, extract useful data, exhale waste.
//!
//! ## Pipeline
//!
//! ```text
//! Intake(inhale/filter) → Exchange(extract/exhale) → Rhythm(regulate rate)
//! ```
//!
//! ## Biological Alignment (v2.0 §10)
//!
//! The [`claude_code`] module maps respiratory biology to Claude Code's context window
//! management. Context is the exchange surface; tools inhale data in, AUTOCOMPACT
//! passively exhales stale context, and reasoning is the gas exchange itself.
//!
//! | Biological Concept | Claude Code Mechanism | Module |
//! |-------------------|----------------------|--------|
//! | Lungs | Context window | [`claude_code::VitalCapacity`] |
//! | Inhalation (pull) | Tool invocations (Read, Grep, Glob, MCP) | [`claude_code::Inhalation`] |
//! | Exhalation (passive) | AUTOCOMPACT | [`claude_code::Exhalation`] |
//! | Gas exchange | Reasoning (input → output tokens) | [`claude_code::GasExchange`] |
//! | Dead space | System prompt + CLAUDE.md overhead | [`claude_code::DeadSpace`] |
//! | Context fork | Subagent isolation (separate lungs) | [`claude_code::ContextFork`] |
//! | Tidal volume | Working context per compaction cycle | [`claude_code::TidalVolume`] |
//! | Respiratory health | Session diagnostic | [`claude_code::RespiratoryHealth`] |
//!
//! ## Organ Mapping (Apps Script → Rust)
//!
//! | JS Organ | Rust Type | Function |
//! |----------|-----------|----------|
//! | `RESPIRATORY.intake` | `Intake` | Inhale from sources, filter unwanted |
//! | `RESPIRATORY.exchange` | `Exchange` | Extract useful, exhale waste |
//! | `RESPIRATORY.rhythm` | `Rhythm` | Regulate breathing rate based on demand |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "Respiratory model is intentionally explicit and uses bounded operational metrics"
)]

pub mod claude_code;
pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// Error Type
// ============================================================================

/// Errors during respiratory operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RespiratoryError {
    /// No sources available
    NoSources,
    /// Intake failed for a source
    IntakeFailed(String),
}

impl core::fmt::Display for RespiratoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSources => write!(f, "no sources available for intake"),
            Self::IntakeFailed(src) => write!(f, "intake failed: {src}"),
        }
    }
}

impl std::error::Error for RespiratoryError {}

// ============================================================================
// Input Sources
// ============================================================================

/// Classification of input sources.
/// Maps JS: `intake.emailIntake/sheetIntake/propertyIntake`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    /// External API/webhook input
    Api,
    /// File-based input
    File,
    /// Configuration/property input
    Config,
    /// Event/signal input
    Event,
}

// ============================================================================
// Inhaled — Filtered intake
// ============================================================================

/// Result of inhaling from sources with filtering applied.
/// Maps JS: `intake.inhale()` → collects from all sources, applies filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inhaled {
    /// Items by source
    pub items: Vec<InhaledItem>,
    /// Number of items filtered out
    pub filtered_count: usize,
}

/// A single inhaled item from a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InhaledItem {
    /// Where this came from
    pub source: InputSource,
    /// The raw content
    pub content: String,
    /// Priority (lower = more urgent)
    pub priority: u8,
}

// ============================================================================
// Extracted / Exhaled — Exchange results
// ============================================================================

/// Useful data extracted during gas exchange.
/// Maps JS: `exchange.extract()` → { useful: true, data: {...} }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extracted {
    /// The useful content
    pub content: String,
    /// Source classification
    pub source: InputSource,
    /// Priority from original item
    pub priority: u8,
}

/// Waste data expelled during exhalation.
/// Maps JS: `exchange.exhale(waste)`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Exhaled {
    /// Waste items removed
    pub waste: Vec<String>,
}

/// Combined result of a gas exchange cycle.
/// Maps JS: `exchange.process()` → { extracted, waste }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeResult {
    /// Useful data extracted
    pub extracted: Vec<Extracted>,
    /// Waste expelled
    pub exhaled: Exhaled,
}

// ============================================================================
// Breathing Rate
// ============================================================================

/// Current breathing rate, adjusted by demand.
/// Maps JS: `rhythm.rate` + `rhythm.regulate()`
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BreathingRate {
    /// Current rate (breaths per cycle)
    pub rate: u32,
    /// Minimum rate
    pub min_rate: u32,
    /// Maximum rate
    pub max_rate: u32,
    /// Normal resting rate
    pub resting_rate: u32,
}

impl Default for BreathingRate {
    fn default() -> Self {
        Self {
            rate: 12,
            min_rate: 8,
            max_rate: 20,
            resting_rate: 12,
        }
    }
}

impl BreathingRate {
    /// Regulate rate based on demand (0-100).
    /// Maps JS: `rhythm.regulate()` → adjust based on calculateDemand()
    pub fn regulate(&mut self, demand: u32) {
        if demand > 80 {
            self.rate = self.max_rate;
        } else if demand < 20 {
            self.rate = self.min_rate;
        } else {
            self.rate = self.resting_rate;
        }
    }

    /// Whether currently hyperventilating (at max rate).
    pub fn is_hyperventilating(&self) -> bool {
        self.rate >= self.max_rate
    }
}

// ============================================================================
// Intake — Input collection and filtering
// ============================================================================

/// The intake organ: collects from external sources and filters.
/// Maps JS: `RESPIRATORY.intake`
pub struct Intake {
    /// Patterns to filter out (e.g., "noreply", spam sources)
    pub filters: Vec<String>,
}

impl Default for Intake {
    fn default() -> Self {
        Self {
            filters: vec!["noreply".to_string(), "spam".to_string()],
        }
    }
}

impl Intake {
    /// Inhale from a set of raw inputs, applying filters.
    /// Maps JS: `intake.inhale()` → collect + filter
    pub fn inhale(&self, raw_inputs: Vec<(InputSource, String)>) -> Inhaled {
        let mut inhaled = Inhaled::default();

        for (source, content) in raw_inputs {
            if self.should_filter(&content) {
                inhaled.filtered_count += 1;
                continue;
            }

            let priority = if content.to_lowercase().contains("urgent") {
                1
            } else {
                3
            };

            inhaled.items.push(InhaledItem {
                source,
                content,
                priority,
            });
        }

        inhaled
    }

    fn should_filter(&self, content: &str) -> bool {
        let lower = content.to_lowercase();
        self.filters.iter().any(|f| lower.contains(f))
    }
}

// ============================================================================
// Exchange — Data extraction and waste expulsion
// ============================================================================

/// The exchange organ: separates useful data from waste.
/// Maps JS: `RESPIRATORY.exchange`
pub struct Exchange;

impl Exchange {
    /// Process inhaled items: extract useful data, exhale waste.
    /// Maps JS: `exchange.process()`
    pub fn process(&self, inhaled: &Inhaled) -> ExchangeResult {
        let mut result = ExchangeResult::default();

        for item in &inhaled.items {
            if self.is_useful(&item.content) {
                result.extracted.push(Extracted {
                    content: item.content.clone(),
                    source: item.source,
                    priority: item.priority,
                });
            } else {
                result.exhaled.waste.push(item.content.clone());
            }
        }

        result
    }

    fn is_useful(&self, content: &str) -> bool {
        !content.trim().is_empty() && content.len() > 1
    }
}

// ============================================================================
// RespiratorySystem — Full orchestrator
// ============================================================================

/// The complete respiratory system.
/// Maps JS: `RESPIRATORY` + `breathe()` function
pub struct RespiratorySystem {
    pub intake: Intake,
    pub exchange: Exchange,
    pub rhythm: BreathingRate,
}

impl Default for RespiratorySystem {
    fn default() -> Self {
        Self {
            intake: Intake::default(),
            exchange: Exchange,
            rhythm: BreathingRate::default(),
        }
    }
}

impl RespiratorySystem {
    /// Run one breathing cycle: inhale → exchange → regulate.
    pub fn breathe(
        &mut self,
        inputs: Vec<(InputSource, String)>,
        demand: u32,
    ) -> Result<ExchangeResult, RespiratoryError> {
        if inputs.is_empty() {
            return Err(RespiratoryError::NoSources);
        }

        let inhaled = self.intake.inhale(inputs);
        let result = self.exchange.process(&inhaled);
        self.rhythm.regulate(demand);

        Ok(result)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intake_filters_spam() {
        let intake = Intake::default();
        let inputs = vec![
            (InputSource::Api, "valid data".to_string()),
            (InputSource::Api, "from noreply@test.com".to_string()),
            (InputSource::Api, "spam content here".to_string()),
        ];
        let inhaled = intake.inhale(inputs);
        assert_eq!(inhaled.items.len(), 1);
        assert_eq!(inhaled.filtered_count, 2);
    }

    #[test]
    fn intake_detects_urgent() {
        let intake = Intake::default();
        let inputs = vec![(InputSource::Event, "urgent: server down".to_string())];
        let inhaled = intake.inhale(inputs);
        assert_eq!(inhaled.items[0].priority, 1);
    }

    #[test]
    fn exchange_separates_useful_from_waste() {
        let exchange = Exchange;
        let inhaled = Inhaled {
            items: vec![
                InhaledItem {
                    source: InputSource::Api,
                    content: "good data".to_string(),
                    priority: 3,
                },
                InhaledItem {
                    source: InputSource::Api,
                    content: "x".to_string(), // too short
                    priority: 3,
                },
                InhaledItem {
                    source: InputSource::Api,
                    content: "   ".to_string(), // whitespace only
                    priority: 3,
                },
            ],
            filtered_count: 0,
        };
        let result = exchange.process(&inhaled);
        assert_eq!(result.extracted.len(), 1);
        assert_eq!(result.exhaled.waste.len(), 2);
    }

    #[test]
    fn breathing_rate_regulation() {
        let mut rate = BreathingRate::default();
        assert_eq!(rate.rate, 12);

        rate.regulate(90);
        assert_eq!(rate.rate, 20);
        assert!(rate.is_hyperventilating());

        rate.regulate(10);
        assert_eq!(rate.rate, 8);
        assert!(!rate.is_hyperventilating());

        rate.regulate(50);
        assert_eq!(rate.rate, 12);
    }

    #[test]
    fn full_breathing_cycle() {
        let mut system = RespiratorySystem::default();
        let inputs = vec![
            (InputSource::Api, "api response data".to_string()),
            (InputSource::Config, "config value".to_string()),
        ];
        let result = system.breathe(inputs, 50);
        assert!(result.is_ok());
        let exchange = result.ok().unwrap_or_else(|| ExchangeResult::default());
        assert_eq!(exchange.extracted.len(), 2);
    }

    #[test]
    fn full_breathing_rejects_no_sources() {
        let mut system = RespiratorySystem::default();
        let result = system.breathe(vec![], 50);
        assert!(result.is_err());
    }
}
