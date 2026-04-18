#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

//! # NexVigilant Core — Lymphatic System
//!
//! Overflow drainage modeled after the biological lymphatic system.
//! Handles output styles, notification hooks, thymic selection, and passive overflow.
//!
//! ## Biological Mapping (per Biological Alignment v2.0 §12)
//!
//! | Biological Component | Claude Code Analog | Function |
//! |---------------------|--------------------|----------|
//! | Lymph drainage | Output styles | Handle overflow that doesn't fit normal output |
//! | Lymph nodes (~600) | Notification hooks | Distributed inspection points per domain |
//! | Thymus | Output style calibration | Learning "self" vs "non-self" |
//! | No pump (passive) | Piggybacks on activity | Styles reshape existing output, not generate new |
//! | Thymic selection | 95%+ rejection | Most candidate rules rejected during testing |
//!
//! ## Output Styles as Lymph Channels
//!
//! ```text
//! Default     = Normal venous return (standard output)
//! Explanatory = Lymph collecting educational context
//! Learning    = Lymph collecting TODO(human) markers
//! Custom      = Custom lymphatic channels (domain-specific)
//! ```
//!
//! ## Key Property: Passive Operation
//!
//! Just like biological lymph moves when muscles contract (primary activity),
//! output styles are passive — they reshape existing output without generating
//! new content. The `DrainageResult.passive` field is always `true`.

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
    reason = "Lymphatic output-channel model intentionally uses fixed-domain transport structs and simple bounded metrics"
)]

pub mod circulation_bridge;
pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// OutputStyle — Lymph channel selection
// ============================================================================

/// Which lymphatic channel to use for output presentation.
///
/// Maps biological lymph drainage channels to Claude Code output styles.
/// - Default: normal venous return (standard output)
/// - Explanatory: lymph collecting educational context
/// - Learning: lymph collecting TODO(human) markers
/// - Custom: domain-specific lymphatic channels
///
/// Tier: T2-P (Σ sum + μ mapping), dominant Σ
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputStyle {
    /// Standard output — normal venous return
    Default,
    /// Educational context — explanatory lymph collection
    Explanatory,
    /// TODO(human) markers — learning lymph collection
    Learning,
    /// Domain-specific — custom lymphatic channel
    Custom(String),
}

impl core::fmt::Display for OutputStyle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Explanatory => write!(f, "explanatory"),
            Self::Learning => write!(f, "learning"),
            Self::Custom(name) => write!(f, "custom({name})"),
        }
    }
}

// ============================================================================
// OutputStyleConfig — Thymic calibration of output style
// ============================================================================

/// Configuration for an output style channel.
///
/// The `keep_coding_instructions` field implements thymic selection:
/// - `true` = "self" (coding context preserved)
/// - `false` = "non-self" (coding context stripped, non-coding persona)
///
/// Verbosity and formality are continuous 0.0-1.0 scales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStyleConfig {
    /// Which output style channel to use
    pub style: OutputStyle,
    /// Thymic selection: is this "self" (coding context)?
    /// true = keep coding instructions, false = non-self persona
    pub keep_coding_instructions: bool,
    /// How verbose the output should be (0.0 = terse, 1.0 = maximal)
    pub verbosity: f64,
    /// How formal the output should be (0.0 = casual, 1.0 = academic)
    pub formality: f64,
}

impl OutputStyleConfig {
    /// Create a new output style config with default parameters.
    #[must_use]
    pub fn new(style: OutputStyle) -> Self {
        Self {
            style,
            keep_coding_instructions: true,
            verbosity: 0.5,
            formality: 0.5,
        }
    }

    /// Create a config with thymic selection applied.
    #[must_use]
    pub fn with_thymic_selection(mut self, keep_coding: bool) -> Self {
        self.keep_coding_instructions = keep_coding;
        self
    }

    /// Set verbosity (clamped to 0.0-1.0).
    #[must_use]
    pub fn with_verbosity(mut self, v: f64) -> Self {
        self.verbosity = v.clamp(0.0, 1.0);
        self
    }

    /// Set formality (clamped to 0.0-1.0).
    #[must_use]
    pub fn with_formality(mut self, f: f64) -> Self {
        self.formality = f.clamp(0.0, 1.0);
        self
    }
}

// ============================================================================
// OverflowItem — Individual item in the lymphatic overflow
// ============================================================================

/// A single item that has overflowed from the primary output channel.
///
/// Like interstitial fluid collected by lymphatic capillaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverflowItem {
    /// The overflowed content
    pub content: String,
    /// Where this overflow originated
    pub source: String,
    /// Priority (0 = lowest, 255 = highest)
    pub priority: u8,
    /// Whether this item has been drained (processed)
    pub drained: bool,
}

impl OverflowItem {
    /// Create a new overflow item.
    #[must_use]
    pub fn new(content: impl Into<String>, source: impl Into<String>, priority: u8) -> Self {
        Self {
            content: content.into(),
            source: source.into(),
            priority,
            drained: false,
        }
    }
}

// ============================================================================
// DrainageResult — Result of lymph drainage operation
// ============================================================================

/// Result of draining overflow content through a lymphatic channel.
///
/// The `passive` field is always `true` — lymphatic drainage piggybacks
/// on primary activity, never generates new content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainageResult {
    /// The formatted output after drainage
    pub formatted_output: String,
    /// Number of overflow items successfully drained
    pub items_drained: usize,
    /// Tokens saved by reshaping content
    pub tokens_saved: usize,
    /// Always true — lymphatic system is passive (no pump)
    pub passive: bool,
}

// ============================================================================
// LymphDrainage — The drainage system
// ============================================================================

/// Lymphatic drainage system that reshapes overflow content to fit output constraints.
///
/// Like the lymphatic system collecting interstitial fluid, this collects
/// overflow items and drains them through the configured output style.
///
/// Tier: T2-C (σ sequence + ∂ boundary + μ mapping), dominant σ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LymphDrainage {
    /// Output style configuration (the channel shape)
    pub style_config: OutputStyleConfig,
    /// Items awaiting drainage
    pub overflow_items: Vec<OverflowItem>,
}

impl LymphDrainage {
    /// Create a new drainage system with the given style config.
    #[must_use]
    pub fn new(style_config: OutputStyleConfig) -> Self {
        Self {
            style_config,
            overflow_items: Vec::new(),
        }
    }

    /// Add an overflow item to the drainage queue.
    pub fn add_item(&mut self, item: OverflowItem) {
        self.overflow_items.push(item);
    }

    /// Drain content: reshape it to fit within max_tokens.
    ///
    /// This is the core lymphatic function: take overflow content and
    /// reshape it according to the output style, respecting capacity limits.
    /// Operation is always passive (piggybacks on primary activity).
    pub fn drain(&mut self, content: &str, max_tokens: usize) -> DrainageResult {
        let original_len = content.len();

        // Apply style-based reshaping
        let formatted = match &self.style_config.style {
            OutputStyle::Default => {
                // Normal venous return — truncate if needed
                if content.len() > max_tokens {
                    let truncated: String = content.chars().take(max_tokens).collect();
                    truncated
                } else {
                    content.to_string()
                }
            }
            OutputStyle::Explanatory => {
                // Collect educational context — prefix with context marker
                let prefix = "[explanatory] ";
                let available = max_tokens.saturating_sub(prefix.len());
                let body: String = content.chars().take(available).collect();
                format!("{prefix}{body}")
            }
            OutputStyle::Learning => {
                // Collect TODO(human) markers — prefix with learning marker
                let prefix = "[TODO(human)] ";
                let available = max_tokens.saturating_sub(prefix.len());
                let body: String = content.chars().take(available).collect();
                format!("{prefix}{body}")
            }
            OutputStyle::Custom(channel) => {
                // Custom channel — prefix with channel name
                let prefix = format!("[{channel}] ");
                let available = max_tokens.saturating_sub(prefix.len());
                let body: String = content.chars().take(available).collect();
                format!("{prefix}{body}")
            }
        };

        let tokens_saved = original_len.saturating_sub(formatted.len());

        // Mark overflow items as drained
        let mut items_drained = 0;
        for item in &mut self.overflow_items {
            if !item.drained {
                item.drained = true;
                items_drained += 1;
            }
        }

        DrainageResult {
            formatted_output: formatted,
            items_drained,
            tokens_saved,
            passive: true, // Always passive — no pump
        }
    }
}

// ============================================================================
// InspectionResult — Lymph node inspection outcome
// ============================================================================

/// Result of a lymph node inspecting content passing through.
///
/// Like a biological lymph node detecting pathogens:
/// - Clear: content is safe
/// - Suspicious: content has anomalies worth noting
/// - Threat: content is dangerous, trigger immune response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionResult {
    /// Content passed inspection — no issues found
    Clear,
    /// Content has anomalies — worth noting but not blocking
    Suspicious(String),
    /// Content is dangerous — trigger immune response
    Threat(String),
}

// ============================================================================
// LymphNode — Distributed inspection point
// ============================================================================

/// A lymph node: a domain-specific inspection point for content.
///
/// Like biological lymph nodes (~600 in the human body), each inspects
/// its own domain and can trigger local immune responses.
///
/// Tier: T2-P (∂ boundary + κ comparison), dominant ∂
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LymphNode {
    /// Domain this node inspects (e.g., "security", "compliance", "format")
    pub domain: String,
    /// Pattern to match against content for this domain
    pub hook_matcher: String,
    /// Total number of inspections performed
    pub inspection_count: u64,
    /// Number of threats detected
    pub threats_detected: u64,
}

impl LymphNode {
    /// Create a new lymph node for the given domain.
    #[must_use]
    pub fn new(domain: impl Into<String>, hook_matcher: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            hook_matcher: hook_matcher.into(),
            inspection_count: 0,
            threats_detected: 0,
        }
    }

    /// Inspect content passing through this lymph node.
    ///
    /// Checks if the content matches the hook_matcher pattern.
    /// Increments counters and returns the inspection result.
    pub fn inspect(&mut self, content: &str) -> InspectionResult {
        self.inspection_count += 1;

        if content.contains(&self.hook_matcher) {
            self.threats_detected += 1;
            InspectionResult::Threat(format!(
                "domain '{}' detected pattern '{}' in content",
                self.domain, self.hook_matcher
            ))
        } else if content.len() > 10000 {
            // Suspiciously large content
            InspectionResult::Suspicious(format!(
                "domain '{}': content size {} exceeds 10000 threshold",
                self.domain,
                content.len()
            ))
        } else {
            InspectionResult::Clear
        }
    }

    /// Detection rate: threats / inspections (0.0 if no inspections).
    #[must_use]
    pub fn detection_rate(&self) -> f64 {
        if self.inspection_count == 0 {
            return 0.0;
        }
        (self.threats_detected as f64) / (self.inspection_count as f64)
    }
}

// ============================================================================
// ThymicVerdict — Outcome of thymic selection
// ============================================================================

/// Verdict from thymic selection: is this "self" or "non-self"?
///
/// Like positive/negative selection in the biological thymus:
/// - Self_: recognized as belonging to the system (coding context)
/// - NonSelf: foreign, should be rejected (non-coding persona)
/// - Uncertain: cannot determine — defaults to conservative handling
///
/// Tier: T2-P (Σ sum + κ comparison), dominant Σ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThymicVerdict {
    /// Recognized as "self" — belongs to the system
    Self_,
    /// Foreign — does not belong, should be rejected
    NonSelf,
    /// Cannot determine — conservative default
    Uncertain,
}

impl core::fmt::Display for ThymicVerdict {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Self_ => write!(f, "self"),
            Self::NonSelf => write!(f, "non-self"),
            Self::Uncertain => write!(f, "uncertain"),
        }
    }
}

// ============================================================================
// ThymicSelection — Self vs non-self classification
// ============================================================================

/// Thymic selection engine: classifies candidates as "self" or "non-self".
///
/// In biology, the thymus educates T-cells to distinguish self from non-self,
/// rejecting 95%+ of candidates. Here, it classifies output style rules:
/// - "self" patterns = coding instructions, technical context
/// - "non-self" patterns = non-coding personas, off-topic content
///
/// Tier: T2-C (κ comparison + ∂ boundary + ∃ existence), dominant κ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThymicSelection {
    /// Patterns recognized as "self" (coding context)
    pub self_patterns: Vec<String>,
    /// Patterns recognized as "non-self" (foreign)
    pub non_self_patterns: Vec<String>,
    /// Rejection rate during training (target: >0.95)
    pub rejection_rate: f64,
}

impl ThymicSelection {
    /// Create a new thymic selection engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            self_patterns: Vec::new(),
            non_self_patterns: Vec::new(),
            rejection_rate: 0.0,
        }
    }

    /// Add a "self" pattern (recognized as belonging).
    pub fn add_self_pattern(&mut self, pattern: impl Into<String>) {
        self.self_patterns.push(pattern.into());
    }

    /// Add a "non-self" pattern (recognized as foreign).
    pub fn add_non_self_pattern(&mut self, pattern: impl Into<String>) {
        self.non_self_patterns.push(pattern.into());
    }

    /// Classify a candidate as self, non-self, or uncertain.
    ///
    /// Checks against known patterns. If a candidate matches a self pattern,
    /// it is classified as Self_. If it matches a non-self pattern, NonSelf.
    /// If neither or both, Uncertain.
    #[must_use]
    pub fn classify(&self, candidate: &str) -> ThymicVerdict {
        let matches_self = self
            .self_patterns
            .iter()
            .any(|p| candidate.contains(p.as_str()));
        let matches_non_self = self
            .non_self_patterns
            .iter()
            .any(|p| candidate.contains(p.as_str()));

        match (matches_self, matches_non_self) {
            (true, false) => ThymicVerdict::Self_,
            (false, true) => ThymicVerdict::NonSelf,
            _ => ThymicVerdict::Uncertain, // Both or neither
        }
    }

    /// Selection pressure: the rejection rate (target: >95%).
    ///
    /// In biological thymic selection, 95-98% of developing thymocytes
    /// fail positive or negative selection and undergo apoptosis.
    #[must_use]
    pub fn selection_pressure(&self) -> f64 {
        self.rejection_rate
    }

    /// Update the rejection rate based on classification results.
    ///
    /// `total_candidates` is the total tested, `rejected` is how many
    /// were classified as NonSelf or Uncertain.
    pub fn update_rejection_rate(&mut self, total_candidates: usize, rejected: usize) {
        if total_candidates == 0 {
            self.rejection_rate = 0.0;
        } else {
            self.rejection_rate = (rejected as f64) / (total_candidates as f64);
        }
    }
}

impl Default for ThymicSelection {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// OverflowHandler — Capacity management with edema detection
// ============================================================================

/// Overflow handler: manages capacity and drains excess items.
///
/// Like the lymphatic system preventing tissue edema (swelling),
/// this tracks capacity and drains overflow when load exceeds 80%.
///
/// Tier: T2-C (∅ void + ∂ boundary + σ sequence), dominant ∅
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverflowHandler {
    /// Maximum capacity before overflow
    pub max_capacity: usize,
    /// Current load
    pub current_load: usize,
    /// Queue of overflow items waiting to be drained
    pub overflow_queue: Vec<OverflowItem>,
}

impl OverflowHandler {
    /// Create a new overflow handler with the given capacity.
    #[must_use]
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            current_load: 0,
            overflow_queue: Vec::new(),
        }
    }

    /// Accept an item if capacity is available.
    ///
    /// Returns `true` if the item was accepted (capacity available),
    /// `false` if it was queued as overflow.
    pub fn accept(&mut self, item: OverflowItem) -> bool {
        let item_size = item.content.len();
        if self.current_load + item_size <= self.max_capacity {
            self.current_load += item_size;
            true
        } else {
            // Queue as overflow
            self.overflow_queue.push(item);
            false
        }
    }

    /// Drain excess overflow items, returning them for processing.
    ///
    /// Removes all items from the overflow queue and returns them.
    /// Like lymphatic drainage clearing interstitial buildup.
    pub fn drain_excess(&mut self) -> Vec<OverflowItem> {
        let drained: Vec<OverflowItem> = self.overflow_queue.drain(..).collect();
        drained
    }

    /// Is the system edematous? (overflow > 80% capacity)
    ///
    /// Edema = tissue swelling from fluid buildup.
    /// When load exceeds 80% of capacity, the system is edematous
    /// and drainage should be prioritized.
    #[must_use]
    pub fn is_edematous(&self) -> bool {
        if self.max_capacity == 0 {
            return !self.overflow_queue.is_empty();
        }
        // Load > 80% of capacity OR overflow queue is non-empty
        let load_fraction = (self.current_load as f64) / (self.max_capacity as f64);
        load_fraction > 0.8 || !self.overflow_queue.is_empty()
    }

    /// Current utilization as a fraction (0.0-1.0+).
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.max_capacity == 0 {
            return if self.current_load == 0 { 0.0 } else { 1.0 };
        }
        (self.current_load as f64) / (self.max_capacity as f64)
    }
}

// ============================================================================
// LymphaticHealth — System health snapshot
// ============================================================================

/// Health snapshot of the lymphatic system.
///
/// Captures the observable state of all lymphatic components:
/// drainage status, node count, thymic rejection rate, edema status.
///
/// Tier: T2-C (ς state + κ comparison + ∂ boundary), dominant ς
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LymphaticHealth {
    /// Whether lymphatic drainage is actively processing
    pub drainage_active: bool,
    /// Number of lymph nodes in the system
    pub lymph_node_count: usize,
    /// Thymic rejection rate (target: >0.95)
    pub thymic_rejection_rate: f64,
    /// Whether the system is edematous (overloaded)
    pub is_edematous: bool,
    /// Whether the system operates passively (always true for lymphatic)
    pub passive_operation: bool,
}

impl LymphaticHealth {
    /// Create a health snapshot from system components.
    #[must_use]
    pub fn assess(
        drainage: &LymphDrainage,
        nodes: &[LymphNode],
        thymus: &ThymicSelection,
        handler: &OverflowHandler,
    ) -> Self {
        Self {
            drainage_active: !drainage.overflow_items.is_empty(),
            lymph_node_count: nodes.len(),
            thymic_rejection_rate: thymus.selection_pressure(),
            is_edematous: handler.is_edematous(),
            passive_operation: true, // Always passive — no pump
        }
    }

    /// Is the system healthy?
    ///
    /// Healthy = drainage active OR no overflow, nodes present,
    /// thymic rejection rate above 0.95, not edematous.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.lymph_node_count > 0
            && self.thymic_rejection_rate >= 0.95
            && !self.is_edematous
            && self.passive_operation
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── OutputStyle tests ───────────────────────────────────────────────

    #[test]
    fn test_output_style_display() {
        assert_eq!(OutputStyle::Default.to_string(), "default");
        assert_eq!(OutputStyle::Explanatory.to_string(), "explanatory");
        assert_eq!(OutputStyle::Learning.to_string(), "learning");
        assert_eq!(
            OutputStyle::Custom("pharma".to_string()).to_string(),
            "custom(pharma)"
        );
    }

    #[test]
    fn test_output_style_serde_roundtrip() {
        let styles = vec![
            OutputStyle::Default,
            OutputStyle::Explanatory,
            OutputStyle::Learning,
            OutputStyle::Custom("test-channel".to_string()),
        ];
        for style in &styles {
            let json = serde_json::to_string(style).unwrap_or_default();
            let back: OutputStyle = serde_json::from_str(&json).unwrap_or(OutputStyle::Default);
            assert_eq!(&back, style);
        }
    }

    // ── OutputStyleConfig tests ─────────────────────────────────────────

    #[test]
    fn test_output_style_config_defaults() {
        let config = OutputStyleConfig::new(OutputStyle::Default);
        assert!(config.keep_coding_instructions);
        assert!((config.verbosity - 0.5).abs() < f64::EPSILON);
        assert!((config.formality - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_output_style_config_clamping() {
        let config = OutputStyleConfig::new(OutputStyle::Default)
            .with_verbosity(5.0)
            .with_formality(-1.0);
        assert!((config.verbosity - 1.0).abs() < f64::EPSILON);
        assert!((config.formality - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_thymic_selection_on_config() {
        let config = OutputStyleConfig::new(OutputStyle::Explanatory).with_thymic_selection(false);
        assert!(!config.keep_coding_instructions);
    }

    // ── LymphDrainage tests ─────────────────────────────────────────────

    #[test]
    fn test_drainage_default_style_within_limit() {
        let config = OutputStyleConfig::new(OutputStyle::Default);
        let mut drainage = LymphDrainage::new(config);
        let result = drainage.drain("hello world", 100);
        assert_eq!(result.formatted_output, "hello world");
        assert_eq!(result.tokens_saved, 0);
        assert!(result.passive);
    }

    #[test]
    fn test_drainage_default_style_truncation() {
        let config = OutputStyleConfig::new(OutputStyle::Default);
        let mut drainage = LymphDrainage::new(config);
        let result = drainage.drain("hello world", 5);
        assert_eq!(result.formatted_output, "hello");
        assert!(result.tokens_saved > 0);
        assert!(result.passive);
    }

    #[test]
    fn test_drainage_explanatory_style() {
        let config = OutputStyleConfig::new(OutputStyle::Explanatory);
        let mut drainage = LymphDrainage::new(config);
        let result = drainage.drain("some content", 100);
        assert!(result.formatted_output.starts_with("[explanatory] "));
        assert!(result.passive);
    }

    #[test]
    fn test_drainage_learning_style() {
        let config = OutputStyleConfig::new(OutputStyle::Learning);
        let mut drainage = LymphDrainage::new(config);
        let result = drainage.drain("review this", 100);
        assert!(result.formatted_output.starts_with("[TODO(human)] "));
        assert!(result.passive);
    }

    #[test]
    fn test_drainage_custom_style() {
        let config = OutputStyleConfig::new(OutputStyle::Custom("pharma".to_string()));
        let mut drainage = LymphDrainage::new(config);
        let result = drainage.drain("drug interaction", 100);
        assert!(result.formatted_output.starts_with("[pharma] "));
        assert!(result.passive);
    }

    #[test]
    fn test_drainage_marks_items_drained() {
        let config = OutputStyleConfig::new(OutputStyle::Default);
        let mut drainage = LymphDrainage::new(config);
        drainage.add_item(OverflowItem::new("item1", "source1", 5));
        drainage.add_item(OverflowItem::new("item2", "source2", 3));

        let result = drainage.drain("content", 100);
        assert_eq!(result.items_drained, 2);

        // Items should be marked as drained
        for item in &drainage.overflow_items {
            assert!(item.drained);
        }
    }

    // ── LymphNode tests ─────────────────────────────────────────────────

    #[test]
    fn test_lymph_node_clear() {
        let mut node = LymphNode::new("security", "SECRET_KEY");
        let result = node.inspect("normal content here");
        assert_eq!(result, InspectionResult::Clear);
        assert_eq!(node.inspection_count, 1);
        assert_eq!(node.threats_detected, 0);
    }

    #[test]
    fn test_lymph_node_threat_detection() {
        let mut node = LymphNode::new("security", "SECRET_KEY");
        let result = node.inspect("my SECRET_KEY is abc123");
        assert!(matches!(result, InspectionResult::Threat(_)));
        assert_eq!(node.inspection_count, 1);
        assert_eq!(node.threats_detected, 1);
    }

    #[test]
    fn test_lymph_node_suspicious_large_content() {
        let mut node = LymphNode::new("size", "not-matching");
        let large = "x".repeat(10001);
        let result = node.inspect(&large);
        assert!(matches!(result, InspectionResult::Suspicious(_)));
    }

    #[test]
    fn test_lymph_node_detection_rate() {
        let mut node = LymphNode::new("test", "bad");
        node.inspect("bad data");
        node.inspect("good data");
        node.inspect("more bad data");
        node.inspect("fine");
        assert!((node.detection_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lymph_node_empty_detection_rate() {
        let node = LymphNode::new("test", "pattern");
        assert!((node.detection_rate() - 0.0).abs() < f64::EPSILON);
    }

    // ── ThymicSelection tests ───────────────────────────────────────────

    #[test]
    fn test_thymic_classify_self() {
        let mut thymus = ThymicSelection::new();
        thymus.add_self_pattern("cargo test");
        thymus.add_self_pattern("fn main");
        thymus.add_non_self_pattern("write me a poem");

        assert_eq!(thymus.classify("run cargo test"), ThymicVerdict::Self_);
    }

    #[test]
    fn test_thymic_classify_non_self() {
        let mut thymus = ThymicSelection::new();
        thymus.add_self_pattern("cargo test");
        thymus.add_non_self_pattern("write me a poem");

        assert_eq!(
            thymus.classify("please write me a poem"),
            ThymicVerdict::NonSelf
        );
    }

    #[test]
    fn test_thymic_classify_uncertain() {
        let mut thymus = ThymicSelection::new();
        thymus.add_self_pattern("cargo test");
        thymus.add_non_self_pattern("write me a poem");

        // Neither matches
        assert_eq!(
            thymus.classify("random unrelated text"),
            ThymicVerdict::Uncertain
        );
    }

    #[test]
    fn test_thymic_classify_ambiguous() {
        let mut thymus = ThymicSelection::new();
        thymus.add_self_pattern("code");
        thymus.add_non_self_pattern("code review");

        // Both match -> Uncertain
        assert_eq!(
            thymus.classify("do a code review"),
            ThymicVerdict::Uncertain
        );
    }

    #[test]
    fn test_thymic_selection_pressure() {
        let mut thymus = ThymicSelection::new();
        thymus.update_rejection_rate(100, 96);
        assert!((thymus.selection_pressure() - 0.96).abs() < f64::EPSILON);
        assert!(thymus.selection_pressure() > 0.95); // Meets biological target
    }

    #[test]
    fn test_thymic_selection_pressure_zero() {
        let mut thymus = ThymicSelection::new();
        thymus.update_rejection_rate(0, 0);
        assert!((thymus.selection_pressure() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_thymic_verdict_display() {
        assert_eq!(ThymicVerdict::Self_.to_string(), "self");
        assert_eq!(ThymicVerdict::NonSelf.to_string(), "non-self");
        assert_eq!(ThymicVerdict::Uncertain.to_string(), "uncertain");
    }

    // ── OverflowHandler tests ───────────────────────────────────────────

    #[test]
    fn test_overflow_accept_within_capacity() {
        let mut handler = OverflowHandler::new(100);
        let item = OverflowItem::new("small", "test", 5);
        assert!(handler.accept(item));
        assert_eq!(handler.current_load, 5); // "small" = 5 bytes
    }

    #[test]
    fn test_overflow_reject_exceeds_capacity() {
        let mut handler = OverflowHandler::new(3);
        let item = OverflowItem::new("large content", "test", 5);
        assert!(!handler.accept(item));
        assert_eq!(handler.overflow_queue.len(), 1);
    }

    #[test]
    fn test_overflow_drain_excess() {
        let mut handler = OverflowHandler::new(3);
        handler.accept(OverflowItem::new("too big", "src1", 1));
        handler.accept(OverflowItem::new("also big", "src2", 2));

        let drained = handler.drain_excess();
        // Both items were too large, should be in overflow
        assert!(!drained.is_empty());
        assert!(handler.overflow_queue.is_empty());
    }

    #[test]
    fn test_overflow_edema_by_load() {
        let mut handler = OverflowHandler::new(100);
        handler.current_load = 81; // > 80%
        assert!(handler.is_edematous());
    }

    #[test]
    fn test_overflow_edema_by_queue() {
        let mut handler = OverflowHandler::new(100);
        handler.overflow_queue.push(OverflowItem::new("x", "y", 1));
        assert!(handler.is_edematous());
    }

    #[test]
    fn test_overflow_not_edematous() {
        let handler = OverflowHandler::new(100);
        assert!(!handler.is_edematous());
    }

    #[test]
    fn test_overflow_zero_capacity() {
        let mut handler = OverflowHandler::new(0);
        assert!(!handler.is_edematous()); // No overflow items
        handler.overflow_queue.push(OverflowItem::new("x", "y", 1));
        assert!(handler.is_edematous()); // Has overflow items
    }

    #[test]
    fn test_overflow_utilization() {
        let mut handler = OverflowHandler::new(100);
        handler.current_load = 50;
        assert!((handler.utilization() - 0.5).abs() < f64::EPSILON);
    }

    // ── LymphaticHealth tests ───────────────────────────────────────────

    #[test]
    fn test_health_assessment() {
        let config = OutputStyleConfig::new(OutputStyle::Default);
        let mut drainage = LymphDrainage::new(config);
        drainage.add_item(OverflowItem::new("pending", "src", 1));

        let nodes = vec![
            LymphNode::new("security", "SECRET"),
            LymphNode::new("format", "INVALID"),
        ];

        let mut thymus = ThymicSelection::new();
        thymus.update_rejection_rate(100, 96);

        let handler = OverflowHandler::new(1000);

        let health = LymphaticHealth::assess(&drainage, &nodes, &thymus, &handler);
        assert!(health.drainage_active);
        assert_eq!(health.lymph_node_count, 2);
        assert!((health.thymic_rejection_rate - 0.96).abs() < f64::EPSILON);
        assert!(!health.is_edematous);
        assert!(health.passive_operation);
    }

    #[test]
    fn test_health_is_healthy() {
        let health = LymphaticHealth {
            drainage_active: true,
            lymph_node_count: 5,
            thymic_rejection_rate: 0.96,
            is_edematous: false,
            passive_operation: true,
        };
        assert!(health.is_healthy());
    }

    #[test]
    fn test_health_unhealthy_no_nodes() {
        let health = LymphaticHealth {
            drainage_active: true,
            lymph_node_count: 0,
            thymic_rejection_rate: 0.96,
            is_edematous: false,
            passive_operation: true,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_health_unhealthy_low_rejection() {
        let health = LymphaticHealth {
            drainage_active: true,
            lymph_node_count: 5,
            thymic_rejection_rate: 0.5, // Below 0.95 target
            is_edematous: false,
            passive_operation: true,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_health_unhealthy_edematous() {
        let health = LymphaticHealth {
            drainage_active: true,
            lymph_node_count: 5,
            thymic_rejection_rate: 0.96,
            is_edematous: true,
            passive_operation: true,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_drainage_always_passive() {
        let config = OutputStyleConfig::new(OutputStyle::Default);
        let mut drainage = LymphDrainage::new(config);
        let result = drainage.drain("content", 100);
        assert!(result.passive, "Lymphatic drainage must always be passive");
    }
}
