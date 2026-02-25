//! # NexVigilant Core — Digestive System
//!
//! Sequential data processing pipeline modeled after the biological digestive tract.
//! Transforms raw, unstructured input into categorized, usable components.
//!
//! ## Pipeline
//!
//! ```text
//! Mouth(chew/taste) → Stomach(ingest/digest) → SmallIntestine(absorb) → Liver(store/metabolize)
//! ```
//!
//! ## Organ Mapping (Apps Script → Rust)
//!
//! | JS Organ | Rust Type | Function |
//! |----------|-----------|----------|
//! | `DIGESTIVE.mouth` | `Mouth` | Parse/tokenize raw input, assess quality |
//! | `DIGESTIVE.stomach` | `Stomach` | Decompose into structured nutrients (keys, numbers, values, metadata) |
//! | `DIGESTIVE.smallIntestine` | `SmallIntestine` | Triage: immediate use, storage, or waste |
//! | `DIGESTIVE.liver` | `Liver` | Deep transformation, energy extraction, detoxification |
//!
//! ## Claude Code Skill Pipeline Mapping (Biological Alignment v2.0 Section 7)
//!
//! The [`claude_code`] module maps the digestive system to Claude Code's skill execution
//! pipeline. Each biological organ corresponds to a stage in skill processing:
//!
//! | Organ | Skill Stage | Key Type |
//! |-------|------------|----------|
//! | Mouth | Trigger detection | [`claude_code::SkillTrigger`] |
//! | Esophagus | File loading | [`claude_code::SkillLoad`] |
//! | Stomach | Frontmatter parsing | [`claude_code::SkillFrontmatter`] |
//! | Small Intestine | Execution (90% value) | [`claude_code::SkillExecution`] |
//! | Sphincters | Gate control | [`claude_code::Sphincter`] |
//! | Microbiome | `!command` substitutions | [`claude_code::Microbiome`] |
//!
//! ## Tier Classification
//!
//! - `Quality`: T2-P (Sigma + kappa) — quality classification
//! - `DataKind`: T2-P (Sigma + mu) — data type classification
//! - `Fragment`: T2-P (sigma + partial) — ordered data pieces
//! - `Taste`: T2-P (kappa + mu + N) — quality assessment
//! - `Nutrients`: T2-C (mu + times + sigma + N) — decomposed components
//! - `Absorbed`: T2-C (partial + arrow + sigma + varsigma) — triage result
//! - `Metabolized`: T2-P (arrow + N + pi) — transformation result
//! - `DigestiveError`: T2-P (partial + Sigma) — pipeline errors

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::self_only_used_in_recursion,
    clippy::too_long_first_doc_paragraph,
    reason = "Digestive-system ontology uses explicit closed schemas and recursive extraction logic for pedagogical clarity"
)]

pub mod claude_code;
pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// Error Type
// ============================================================================

/// Errors during digestive processing.
/// Tier: T2-P (Boundary + Sum), dominant Boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DigestiveError {
    /// Input was empty — nothing to digest
    EmptyInput,
    /// Input data was malformed or unparseable
    Malformed(String),
    /// Processing capacity exceeded
    CapacityExceeded { limit: usize, actual: usize },
}

impl core::fmt::Display for DigestiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "empty input — nothing to digest"),
            Self::Malformed(msg) => write!(f, "malformed input: {msg}"),
            Self::CapacityExceeded { limit, actual } => {
                write!(f, "capacity exceeded: {actual} > {limit}")
            }
        }
    }
}

impl std::error::Error for DigestiveError {}

// ============================================================================
// Quality — Mouth assessment result
// ============================================================================

/// Data quality classification from Mouth::taste.
/// Maps JS: `assessQuality()` → "poor" | "empty" | "rich" | "normal"
/// Tier: T2-P (Sum + Comparison), dominant Sum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quality {
    /// No usable data
    Poor,
    /// Container exists but has no content
    Empty,
    /// Large, information-dense input
    Rich,
    /// Standard, processable input
    Normal,
}

// ============================================================================
// DataKind — Type classification
// ============================================================================

/// Classification of input data type for processing dispatch.
/// Maps JS: `identifyType()` → "array" | "date" | "string" | "number" | "object"
/// Tier: T2-P (Sum + Mapping), dominant Sum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataKind {
    /// Ordered collection of values
    Array,
    /// Temporal value
    Timestamp,
    /// Text content
    Text,
    /// Numeric value
    Number,
    /// Structured key-value data
    Object,
    /// Unrecognized type
    Unknown(()),
}

// ============================================================================
// Fragment — Mouth output
// ============================================================================

/// An ordered piece of broken-down input data, produced by Mouth::chew.
/// Maps JS: `breakDown(item)` → split strings, extract object values
/// Tier: T2-P (Sequence + Boundary), dominant Sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    /// Position in the original input sequence
    pub index: usize,
    /// The raw string content of this fragment
    pub content: String,
    /// The detected data kind
    pub kind: DataKind,
}

// ============================================================================
// Taste — Mouth assessment
// ============================================================================

/// Quick quality assessment result from Mouth::taste.
/// Maps JS: `taste(data)` → { quality, type, size }
/// Tier: T2-P (Comparison + Mapping + Quantity), dominant Comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taste {
    /// Assessed quality level
    pub quality: Quality,
    /// Detected primary data kind
    pub kind: DataKind,
    /// Size in bytes of the input
    pub size: usize,
}

// ============================================================================
// Nutrients — Stomach output
// ============================================================================

/// Structured decomposition of digested input into categorized components.
/// Maps JS: `stomach.breakdown()` → { proteins, carbs, fats, vitamins }
///
/// | Biological | Computational | Extraction |
/// |-----------|---------------|------------|
/// | Proteins | Structure keys | Object keys, field names |
/// | Carbs | Quick values | Numbers, booleans (fast access) |
/// | Fats | Stored values | String values, nested content |
/// | Vitamins | Metadata | Type info, counts, timestamps |
///
/// Tier: T2-C (Mapping + Product + Sequence + Quantity), dominant Mapping
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nutrients {
    /// Structural keys extracted (object keys, field names)
    pub proteins: Vec<String>,
    /// Quick-access numeric values
    pub carbs: Vec<f64>,
    /// Stored string/content values
    pub fats: Vec<String>,
    /// Metadata about the digested input
    pub vitamins: Metadata,
}

/// Metadata extracted during digestion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    /// Original data kind
    pub kind: Option<DataKind>,
    /// Number of elements/fields in original
    pub element_count: usize,
    /// Timestamp of digestion
    pub digested_at: Option<String>,
}

// ============================================================================
// Absorbed — SmallIntestine output
// ============================================================================

/// Triage result from SmallIntestine::absorb.
/// Maps JS: `smallIntestine.absorb()` → { immediate, stored, waste }
/// Tier: T2-C (Boundary + Causality + Sequence + State), dominant Boundary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Absorbed {
    /// High-priority items routed for immediate use
    pub immediate: Vec<String>,
    /// Normal-priority items routed to storage
    pub stored: Vec<String>,
    /// Rejected items (null, empty, useless)
    pub waste: Vec<String>,
}

// ============================================================================
// Metabolized — Liver output
// ============================================================================

/// Result of liver metabolization: transformation with energy extraction.
/// Maps JS: `liver.metabolize()` → { original, processed, energy }
/// Tier: T2-P (Causality + Quantity + Persistence), dominant Causality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metabolized {
    /// The original input before processing
    pub original: String,
    /// The transformed output
    pub processed: String,
    /// Energy value extracted (information content proxy)
    pub energy: usize,
}

// ============================================================================
// Mouth — Initial parsing/tokenization
// ============================================================================

/// The mouth: first organ of the digestive tract.
/// Breaks raw input into fragments and assesses quality.
/// Maps JS: `DIGESTIVE.mouth`
pub struct Mouth {
    /// Maximum fragment size before further breaking
    pub max_fragment_size: usize,
}

impl Default for Mouth {
    fn default() -> Self {
        Self {
            max_fragment_size: 1024,
        }
    }
}

impl Mouth {
    /// Break input into ordered fragments.
    /// Maps JS: `mouth.chew(data)` → breakDown each item
    pub fn chew(&self, input: &str) -> Vec<Fragment> {
        if input.is_empty() {
            return Vec::new();
        }

        // Try JSON parse first
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(input) {
            return self.chew_json(&value);
        }

        // Fall back to whitespace tokenization
        input
            .split_whitespace()
            .enumerate()
            .map(|(i, token)| Fragment {
                index: i,
                content: token.to_string(),
                kind: classify_token(token),
            })
            .collect()
    }

    /// Assess quality of input without full processing.
    /// Maps JS: `mouth.taste(data)` → { quality, type, size }
    pub fn taste(&self, input: &str) -> Taste {
        let size = input.len();
        let quality = if input.is_empty() {
            Quality::Empty
        } else if size > 1000 {
            Quality::Rich
        } else if input.trim().is_empty() {
            Quality::Poor
        } else {
            Quality::Normal
        };

        let kind = if input.starts_with('{') || input.starts_with('[') {
            if input.starts_with('[') {
                DataKind::Array
            } else {
                DataKind::Object
            }
        } else if input.parse::<f64>().is_ok() {
            DataKind::Number
        } else {
            DataKind::Text
        };

        Taste {
            quality,
            kind,
            size,
        }
    }

    fn chew_json(&self, value: &serde_json::Value) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        self.extract_fragments(value, &mut fragments, 0);
        fragments
    }

    fn extract_fragments(&self, value: &serde_json::Value, out: &mut Vec<Fragment>, depth: usize) {
        if depth > 10 {
            return; // Prevent stack overflow on deeply nested data
        }

        match value {
            serde_json::Value::Null => {
                out.push(Fragment {
                    index: out.len(),
                    content: String::new(),
                    kind: DataKind::Unknown(()),
                });
            }
            serde_json::Value::Bool(b) => {
                out.push(Fragment {
                    index: out.len(),
                    content: b.to_string(),
                    kind: DataKind::Number,
                });
            }
            serde_json::Value::Number(n) => {
                out.push(Fragment {
                    index: out.len(),
                    content: n.to_string(),
                    kind: DataKind::Number,
                });
            }
            serde_json::Value::String(s) => {
                out.push(Fragment {
                    index: out.len(),
                    content: s.clone(),
                    kind: DataKind::Text,
                });
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.extract_fragments(item, out, depth + 1);
                }
            }
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    out.push(Fragment {
                        index: out.len(),
                        content: key.clone(),
                        kind: DataKind::Text,
                    });
                    self.extract_fragments(val, out, depth + 1);
                }
            }
        }
    }
}

// ============================================================================
// Stomach — Structured decomposition
// ============================================================================

/// The stomach: decomposes fragments into structured nutrients.
/// Maps JS: `DIGESTIVE.stomach`
pub struct Stomach {
    queue: Vec<Fragment>,
    /// Maximum batch size for digestion
    pub batch_size: usize,
}

impl Default for Stomach {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            batch_size: 10,
        }
    }
}

impl Stomach {
    /// Ingest a fragment into the stomach queue.
    /// Maps JS: `stomach.swallow(food)`
    pub fn ingest(&mut self, fragment: Fragment) {
        self.queue.push(fragment);
    }

    /// Ingest multiple fragments at once.
    pub fn ingest_batch(&mut self, fragments: Vec<Fragment>) {
        self.queue.extend(fragments);
    }

    /// Digest the current batch, extracting nutrients.
    /// Maps JS: `stomach.digest()` → processes batch_size items
    pub fn digest(&mut self) -> Vec<Nutrients> {
        let batch_end = self.queue.len().min(self.batch_size);
        let batch: Vec<Fragment> = self.queue.drain(..batch_end).collect();

        batch.iter().map(|f| self.breakdown(f)).collect()
    }

    /// How many fragments are waiting for digestion.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    fn breakdown(&self, fragment: &Fragment) -> Nutrients {
        let mut nutrients = Nutrients::default();

        match fragment.kind {
            DataKind::Object | DataKind::Text => {
                // Extract structural keys (proteins) from text
                for word in fragment.content.split_whitespace() {
                    if word.contains(':') || word.contains('=') {
                        nutrients.proteins.push(word.to_string());
                    } else if let Ok(n) = word.parse::<f64>() {
                        nutrients.carbs.push(n);
                    } else {
                        nutrients.fats.push(word.to_string());
                    }
                }
            }
            DataKind::Number => {
                if let Ok(n) = fragment.content.parse::<f64>() {
                    nutrients.carbs.push(n);
                }
            }
            DataKind::Array => {
                nutrients.fats.push(fragment.content.clone());
            }
            DataKind::Timestamp => {
                nutrients.vitamins.digested_at = Some(fragment.content.clone());
            }
            DataKind::Unknown(()) => {
                // Nothing useful extractable
            }
        }

        nutrients.vitamins.kind = Some(fragment.kind);
        nutrients.vitamins.element_count =
            nutrients.proteins.len() + nutrients.carbs.len() + nutrients.fats.len();
        nutrients.vitamins.digested_at = Some(nexcore_chrono::DateTime::now().to_rfc3339());

        nutrients
    }
}

// ============================================================================
// SmallIntestine — Absorption and triage
// ============================================================================

/// The small intestine: absorbs nutrients into useful, stored, or waste streams.
/// Maps JS: `DIGESTIVE.smallIntestine`
pub struct SmallIntestine;

impl SmallIntestine {
    /// Absorb nutrients, triaging into immediate, stored, and waste.
    /// Maps JS: `smallIntestine.absorb(nutrients)` → { immediate, stored, waste }
    pub fn absorb(&self, nutrients: &Nutrients) -> Absorbed {
        let mut absorbed = Absorbed::default();

        // Proteins (structural keys) → stored for reference
        for protein in &nutrients.proteins {
            if protein.is_empty() {
                absorbed.waste.push(protein.clone());
            } else {
                absorbed.stored.push(protein.clone());
            }
        }

        // Carbs (numbers) → immediate use (fast access values)
        for carb in &nutrients.carbs {
            absorbed.immediate.push(carb.to_string());
        }

        // Fats (values) → triage based on content
        for fat in &nutrients.fats {
            if fat.is_empty() {
                absorbed.waste.push(fat.clone());
            } else if is_urgent(fat) {
                absorbed.immediate.push(fat.clone());
            } else {
                absorbed.stored.push(fat.clone());
            }
        }

        absorbed
    }
}

// ============================================================================
// Liver — Storage, transformation, and detoxification
// ============================================================================

/// The liver: deep processing, storage, transformation, and detox.
/// Maps JS: `DIGESTIVE.liver`
pub struct Liver {
    storage: Vec<String>,
    /// Maximum storage capacity
    pub capacity: usize,
}

impl Default for Liver {
    fn default() -> Self {
        Self {
            storage: Vec::new(),
            capacity: 1000,
        }
    }
}

impl Liver {
    /// Store nutrients for later processing.
    /// Maps JS: `liver.store(nutrients)`
    pub fn store(&mut self, items: &[String]) {
        self.storage.extend_from_slice(items);

        // Enforce capacity limit (keep most recent)
        if self.storage.len() > self.capacity {
            let overflow = self.storage.len() - self.capacity;
            self.storage.drain(..overflow);
        }
    }

    /// Current storage count.
    pub fn storage_len(&self) -> usize {
        self.storage.len()
    }

    /// Metabolize all stored items, transforming and extracting energy.
    /// Maps JS: `liver.process()` + `liver.metabolize()`
    pub fn metabolize(&mut self) -> Vec<Metabolized> {
        let items: Vec<String> = self.storage.drain(..).collect();

        items
            .into_iter()
            .map(|original| {
                let processed = transform(&original);
                let energy = original.len(); // Information content proxy
                Metabolized {
                    original,
                    processed,
                    energy,
                }
            })
            .collect()
    }

    /// Detoxify harmful data, returning cleaned version or None if irrecoverable.
    /// Maps JS: `liver.detoxify(toxin)`
    pub fn detoxify(&self, toxin: &str) -> Option<String> {
        if toxin.is_empty() {
            return None;
        }

        // Strip known harmful patterns
        let cleaned = toxin
            .replace("<script>", "")
            .replace("</script>", "")
            .replace("javascript:", "")
            .replace("eval(", "")
            .replace("exec(", "");

        if cleaned.trim().is_empty() {
            None // Entirely toxic, nothing recoverable
        } else {
            Some(cleaned)
        }
    }
}

// ============================================================================
// DigestiveTract — Full pipeline orchestrator
// ============================================================================

/// The complete digestive tract: orchestrates the full pipeline.
/// Maps JS: `DIGESTIVE.process.transform()`
pub struct DigestiveTract {
    pub mouth: Mouth,
    pub stomach: Stomach,
    pub intestine: SmallIntestine,
    pub liver: Liver,
}

impl Default for DigestiveTract {
    fn default() -> Self {
        Self {
            mouth: Mouth::default(),
            stomach: Stomach::default(),
            intestine: SmallIntestine,
            liver: Liver::default(),
        }
    }
}

impl DigestiveTract {
    /// Run the full digestive pipeline on raw input.
    ///
    /// Pipeline: chew → ingest → digest → absorb → store/metabolize
    pub fn process(&mut self, input: &str) -> Result<DigestResult, DigestiveError> {
        // Taste first (quality gate)
        let taste = self.mouth.taste(input);
        if taste.quality == Quality::Empty {
            return Err(DigestiveError::EmptyInput);
        }

        // Mouth: chew into fragments
        let fragments = self.mouth.chew(input);
        if fragments.is_empty() {
            return Err(DigestiveError::EmptyInput);
        }

        // Stomach: ingest and digest
        self.stomach.ingest_batch(fragments);
        let all_nutrients = self.stomach.digest();

        // SmallIntestine: absorb each nutrient batch
        let mut total_absorbed = Absorbed::default();
        for nutrients in &all_nutrients {
            let batch = self.intestine.absorb(nutrients);
            total_absorbed.immediate.extend(batch.immediate);
            total_absorbed.stored.extend(batch.stored);
            total_absorbed.waste.extend(batch.waste);
        }

        // Liver: store and metabolize
        self.liver.store(&total_absorbed.stored);
        let metabolized = self.liver.metabolize();

        Ok(DigestResult {
            taste,
            nutrients: all_nutrients,
            absorbed: total_absorbed,
            metabolized,
        })
    }
}

/// Complete result of a full digestive cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestResult {
    /// Initial quality assessment
    pub taste: Taste,
    /// All extracted nutrients
    pub nutrients: Vec<Nutrients>,
    /// Triage results (immediate/stored/waste)
    pub absorbed: Absorbed,
    /// Liver-processed outputs
    pub metabolized: Vec<Metabolized>,
}

// ============================================================================
// Helper functions
// ============================================================================

fn classify_token(token: &str) -> DataKind {
    if token.parse::<f64>().is_ok() {
        DataKind::Number
    } else if token.contains('-') && token.len() >= 10 {
        // Heuristic: ISO date-like strings
        DataKind::Timestamp
    } else {
        DataKind::Text
    }
}

fn is_urgent(value: &str) -> bool {
    let lower = value.to_lowercase();
    lower.contains("urgent") || lower.contains("priority") || lower.contains("critical")
}

fn transform(input: &str) -> String {
    // Simple transformation: normalize whitespace, trim, uppercase first char
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut chars = trimmed.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mouth tests ---

    #[test]
    fn mouth_chew_empty_returns_empty() {
        let mouth = Mouth::default();
        let fragments = mouth.chew("");
        assert!(fragments.is_empty());
    }

    #[test]
    fn mouth_chew_text_splits_whitespace() {
        let mouth = Mouth::default();
        let fragments = mouth.chew("hello world 42");
        assert_eq!(fragments.len(), 3);
        assert_eq!(fragments[0].content, "hello");
        assert_eq!(fragments[2].kind, DataKind::Number);
    }

    #[test]
    fn mouth_chew_json_extracts_fields() {
        let mouth = Mouth::default();
        let fragments = mouth.chew(r#"{"name":"test","value":42}"#);
        assert!(fragments.len() >= 3); // at least: "name", "test", "value", 42
    }

    #[test]
    fn mouth_taste_empty() {
        let mouth = Mouth::default();
        let taste = mouth.taste("");
        assert_eq!(taste.quality, Quality::Empty);
        assert_eq!(taste.size, 0);
    }

    #[test]
    fn mouth_taste_rich() {
        let mouth = Mouth::default();
        let large = "x".repeat(1500);
        let taste = mouth.taste(&large);
        assert_eq!(taste.quality, Quality::Rich);
    }

    #[test]
    fn mouth_taste_json_object() {
        let mouth = Mouth::default();
        let taste = mouth.taste(r#"{"key":"value"}"#);
        assert_eq!(taste.kind, DataKind::Object);
    }

    #[test]
    fn mouth_taste_number() {
        let mouth = Mouth::default();
        let taste = mouth.taste("42.5");
        assert_eq!(taste.kind, DataKind::Number);
    }

    // --- Stomach tests ---

    #[test]
    fn stomach_ingest_and_digest() {
        let mut stomach = Stomach::default();
        stomach.ingest(Fragment {
            index: 0,
            content: "hello world".to_string(),
            kind: DataKind::Text,
        });
        assert_eq!(stomach.queue_len(), 1);

        let nutrients = stomach.digest();
        assert_eq!(nutrients.len(), 1);
        assert_eq!(stomach.queue_len(), 0);
    }

    #[test]
    fn stomach_extracts_numbers_as_carbs() {
        let mut stomach = Stomach::default();
        stomach.ingest(Fragment {
            index: 0,
            content: "42".to_string(),
            kind: DataKind::Number,
        });
        let nutrients = stomach.digest();
        assert_eq!(nutrients[0].carbs.len(), 1);
        let diff = (nutrients[0].carbs[0] - 42.0).abs();
        assert!(diff < f64::EPSILON);
    }

    #[test]
    fn stomach_batch_size_limits_processing() {
        let mut stomach = Stomach {
            batch_size: 2,
            ..Stomach::default()
        };
        for i in 0..5 {
            stomach.ingest(Fragment {
                index: i,
                content: format!("item{i}"),
                kind: DataKind::Text,
            });
        }
        let batch1 = stomach.digest();
        assert_eq!(batch1.len(), 2);
        assert_eq!(stomach.queue_len(), 3);
    }

    // --- SmallIntestine tests ---

    #[test]
    fn intestine_routes_numbers_to_immediate() {
        let intestine = SmallIntestine;
        let nutrients = Nutrients {
            carbs: vec![1.0, 2.0, 3.0],
            ..Nutrients::default()
        };
        let absorbed = intestine.absorb(&nutrients);
        assert_eq!(absorbed.immediate.len(), 3);
    }

    #[test]
    fn intestine_routes_urgent_to_immediate() {
        let intestine = SmallIntestine;
        let nutrients = Nutrients {
            fats: vec!["urgent fix needed".to_string(), "normal data".to_string()],
            ..Nutrients::default()
        };
        let absorbed = intestine.absorb(&nutrients);
        assert!(
            absorbed
                .immediate
                .contains(&"urgent fix needed".to_string())
        );
        assert!(absorbed.stored.contains(&"normal data".to_string()));
    }

    #[test]
    fn intestine_routes_empty_to_waste() {
        let intestine = SmallIntestine;
        let nutrients = Nutrients {
            fats: vec![String::new()],
            proteins: vec![String::new()],
            ..Nutrients::default()
        };
        let absorbed = intestine.absorb(&nutrients);
        assert_eq!(absorbed.waste.len(), 2);
    }

    // --- Liver tests ---

    #[test]
    fn liver_store_respects_capacity() {
        let mut liver = Liver {
            capacity: 3,
            ..Liver::default()
        };
        liver.store(&[
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ]);
        assert_eq!(liver.storage_len(), 3);
    }

    #[test]
    fn liver_metabolize_transforms() {
        let mut liver = Liver::default();
        liver.store(&["hello world".to_string()]);
        let results = liver.metabolize();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].processed, "Hello world");
        assert_eq!(results[0].energy, 11); // "hello world".len()
    }

    #[test]
    fn liver_detoxify_strips_scripts() {
        let liver = Liver::default();
        let result = liver.detoxify("<script>alert('xss')</script>");
        assert!(result.is_some());
        let cleaned = result.as_deref().unwrap_or("");
        assert!(!cleaned.contains("<script>"));
    }

    #[test]
    fn liver_detoxify_returns_none_for_pure_toxin() {
        let liver = Liver::default();
        let result = liver.detoxify("<script></script>");
        assert!(result.is_none());
    }

    // --- Full pipeline tests ---

    #[test]
    fn full_pipeline_processes_text() {
        let mut tract = DigestiveTract::default();
        let result = tract.process("hello world 42 urgent");
        assert!(result.is_ok());

        let r = result.ok().unwrap_or_else(|| DigestResult {
            taste: Taste {
                quality: Quality::Poor,
                kind: DataKind::Text,
                size: 0,
            },
            nutrients: vec![],
            absorbed: Absorbed::default(),
            metabolized: vec![],
        });
        assert_eq!(r.taste.quality, Quality::Normal);
        assert!(!r.nutrients.is_empty());
    }

    #[test]
    fn full_pipeline_processes_json() {
        let mut tract = DigestiveTract::default();
        let result = tract.process(r#"{"drug":"aspirin","count":150,"urgent":true}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn full_pipeline_rejects_empty() {
        let mut tract = DigestiveTract::default();
        let result = tract.process("");
        assert!(result.is_err());
    }

    #[test]
    fn full_pipeline_rejects_whitespace_only() {
        let mut tract = DigestiveTract::default();
        let result = tract.process("   ");
        // Quality::Poor but not Empty, still has fragments... actually:
        // mouth.taste("   ") → Poor (trimmed is empty)
        // But mouth.chew("   ") → empty (split_whitespace on whitespace = [])
        assert!(result.is_err());
    }

    // --- Classify token ---

    #[test]
    fn classify_number_token() {
        assert_eq!(classify_token("42"), DataKind::Number);
        assert_eq!(classify_token("3.14"), DataKind::Number);
    }

    #[test]
    fn classify_text_token() {
        assert_eq!(classify_token("hello"), DataKind::Text);
    }

    #[test]
    fn classify_timestamp_token() {
        assert_eq!(classify_token("2026-02-10"), DataKind::Timestamp);
    }
}
