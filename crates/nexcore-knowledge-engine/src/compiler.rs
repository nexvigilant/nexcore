//! Knowledge compiler — full pipeline from raw sources to knowledge pack.
//!
//! Pipeline: Ingest → Extract → Compress → Graph → Pack

use nexcore_chrono::DateTime;
use nexcore_fs::dirs;

use crate::compression::StructuralCompressor;
use crate::concept_graph::{ConceptGraph, ConceptRelation};
use crate::error::{KnowledgeEngineError, Result};
use crate::extraction::ConceptExtractor;
use crate::ingest::{self, KnowledgeFragment, KnowledgeSource, RawKnowledge};
use crate::knowledge_pack::KnowledgePack;
use crate::scoring::{CompendiousScorer, ScoreResult};
use crate::store::KnowledgeStore;

/// Options for compilation.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub name: String,
    pub include_distillations: bool,
    pub include_artifacts: bool,
    pub include_implicit: bool,
    pub sources: Vec<RawKnowledge>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            include_distillations: true,
            include_artifacts: false,
            include_implicit: false,
            sources: Vec::new(),
        }
    }
}

/// Result of `compress_text` — before/after scores and the compressed output.
#[derive(Debug, Clone)]
pub struct CompressTextResult {
    pub original_score: ScoreResult,
    pub compressed_score: ScoreResult,
    pub compressed_text: String,
    pub compression_ratio: f64,
}

/// Knowledge compiler.
pub struct KnowledgeCompiler {
    compressor: StructuralCompressor,
    store: KnowledgeStore,
}

impl KnowledgeCompiler {
    pub fn new(store: KnowledgeStore) -> Self {
        Self {
            compressor: StructuralCompressor::new(),
            store,
        }
    }

    /// Compile knowledge from options into a pack.
    ///
    /// Sources that fail ingestion (e.g., compressed to empty) are counted and
    /// logged in the pack's stats rather than silently dropped.
    pub fn compile(&self, options: CompileOptions) -> Result<KnowledgePack> {
        let mut raw_sources = options.sources;

        // Load Brain distillations if requested
        if options.include_distillations {
            raw_sources.extend(self.load_distillations());
        }

        // Load Brain artifacts if requested
        if options.include_artifacts {
            raw_sources.extend(self.load_artifacts());
        }

        // Load implicit knowledge if requested
        if options.include_implicit {
            raw_sources.extend(self.load_implicit());
        }

        let total_sources = raw_sources.len();

        // Step 1: Ingest all sources into fragments
        let mut fragments: Vec<KnowledgeFragment> = Vec::new();
        let mut original_word_count = 0_usize;
        let mut failed_count = 0_usize;

        for raw in raw_sources {
            original_word_count += raw.text.split_whitespace().count();

            // Compress before ingesting
            let compressed = self.compressor.compress(&raw.text);
            let compressed_raw = RawKnowledge {
                text: compressed.compressed_text,
                source: raw.source,
                domain: raw.domain,
                timestamp: raw.timestamp,
            };

            match ingest::ingest(compressed_raw) {
                Ok(frag) => fragments.push(frag),
                Err(KnowledgeEngineError::EmptyInput) => {
                    // Source compressed to empty — count but don't fail the whole compile
                    failed_count += 1;
                }
                Err(e) => return Err(e),
            }
        }

        // Step 2: Build concept graph from fragments
        let mut graph = ConceptGraph::new();
        for frag in &fragments {
            for concept in &frag.concepts {
                graph.add_concept(&concept.term, concept.domain.clone());
            }

            // Connect co-occurring concepts within same fragment
            let terms: Vec<&str> = frag.concepts.iter().map(|c| c.term.as_str()).collect();
            for i in 0..terms.len() {
                for j in (i + 1)..terms.len() {
                    graph.add_edge(terms[i], terms[j], ConceptRelation::RelatedTo, 0.5);
                }
            }
        }

        // Step 3: Determine version
        let version = self
            .store
            .latest_version(&options.name)
            .map(|v| v + 1)
            .unwrap_or(1);

        // Step 4: Build pack
        let mut pack = KnowledgePack::new(options.name, version, fragments, graph);

        // Record source statistics
        pack.stats.sources_attempted = total_sources;
        pack.stats.sources_failed = failed_count;

        // Compute overall compression ratio
        let compressed_word_count = pack.stats.total_words;
        if original_word_count > 0 {
            pack.stats.compression_ratio =
                1.0 - (compressed_word_count as f64 / original_word_count as f64);
        }

        // Step 5: Persist
        self.store.save_pack(&pack)?;

        Ok(pack)
    }

    /// Load Brain distillations from `~/.claude/brain/distillations/`.
    fn load_distillations(&self) -> Vec<RawKnowledge> {
        let brain_dir = dirs::home_dir()
            .map(|h| h.join(".claude").join("brain").join("distillations"))
            .unwrap_or_default();

        if !brain_dir.exists() {
            return Vec::new();
        }

        let mut sources = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&brain_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json" || e == "md") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if !content.trim().is_empty() {
                            sources.push(RawKnowledge {
                                text: content,
                                source: KnowledgeSource::BrainDistillation,
                                domain: None,
                                timestamp: DateTime::now(),
                            });
                        }
                    }
                }
            }
        }
        sources
    }

    /// Load Brain artifacts from `~/.claude/brain/sessions/`.
    fn load_artifacts(&self) -> Vec<RawKnowledge> {
        let sessions_dir = dirs::home_dir()
            .map(|h| h.join(".claude").join("brain").join("sessions"))
            .unwrap_or_default();

        if !sessions_dir.exists() {
            return Vec::new();
        }

        let mut sources = Vec::new();
        for entry in nexcore_fs::walk::WalkDir::new(&sessions_dir)
            .max_depth(3)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                // Skip resolved snapshots — only ingest mutable artifacts
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.contains(".resolved") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(path) {
                    if content.len() > 50 {
                        // Skip trivially small files
                        sources.push(RawKnowledge {
                            text: content,
                            source: KnowledgeSource::BrainArtifact,
                            domain: None,
                            timestamp: DateTime::now(),
                        });
                    }
                }
            }
        }
        sources
    }

    /// Load implicit knowledge from `~/.claude/brain/implicit/`.
    fn load_implicit(&self) -> Vec<RawKnowledge> {
        let implicit_dir = dirs::home_dir()
            .map(|h| h.join(".claude").join("brain").join("implicit"))
            .unwrap_or_default();

        if !implicit_dir.exists() {
            return Vec::new();
        }

        let mut sources = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&implicit_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if !content.trim().is_empty() {
                            sources.push(RawKnowledge {
                                text: content,
                                source: KnowledgeSource::ImplicitKnowledge,
                                domain: None,
                                timestamp: DateTime::now(),
                            });
                        }
                    }
                }
            }
        }
        sources
    }

    /// Ingest and score a single text (no compilation).
    pub fn ingest_single(
        text: &str,
        source_type: KnowledgeSource,
        domain: Option<String>,
    ) -> Result<KnowledgeFragment> {
        let raw = RawKnowledge {
            text: text.to_string(),
            source: source_type,
            domain,
            timestamp: DateTime::now(),
        };
        ingest::ingest(raw)
    }

    /// Compress text and return before/after scores.
    pub fn compress_text(text: &str) -> CompressTextResult {
        let original_score = CompendiousScorer::score(text, &[]);
        let compressor = StructuralCompressor::new();
        let result = compressor.compress(text);
        let compressed_score = CompendiousScorer::score(&result.compressed_text, &[]);
        CompressTextResult {
            original_score,
            compressed_score,
            compressed_text: result.compressed_text,
            compression_ratio: result.compression_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_from_sources() {
        let store = KnowledgeStore::temp().unwrap();
        let compiler = KnowledgeCompiler::new(store);

        let options = CompileOptions {
            name: "test-pack".to_string(),
            include_distillations: false,
            include_artifacts: false,
            include_implicit: false,
            sources: vec![
                RawKnowledge {
                    text: "Signal detection uses PRR for safety analysis.".to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("pv".to_string()),
                    timestamp: DateTime::now(),
                },
                RawKnowledge {
                    text: "Rust traits enable polymorphism through trait objects and generics."
                        .to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("rust".to_string()),
                    timestamp: DateTime::now(),
                },
            ],
        };

        let pack = compiler.compile(options).unwrap();
        assert_eq!(pack.name, "test-pack");
        assert_eq!(pack.version, 1);
        assert_eq!(pack.stats.fragment_count, 2);
        assert_eq!(pack.stats.sources_attempted, 2);
        assert_eq!(pack.stats.sources_failed, 0);
        // Two domain-rich fragments must produce at least 5 distinct concepts
        assert!(
            pack.stats.concept_count >= 5,
            "expected >= 5 concepts, got {}",
            pack.stats.concept_count
        );
        // Domains should include "pv" from the first source text
        assert!(
            pack.stats.domains.contains(&"pv".to_string()),
            "expected 'pv' domain, got: {:?}",
            pack.stats.domains
        );
    }

    #[test]
    fn compile_tracks_failed_sources() {
        let store = KnowledgeStore::temp().unwrap();
        let compiler = KnowledgeCompiler::new(store);

        let options = CompileOptions {
            name: "fail-test".to_string(),
            include_distillations: false,
            include_artifacts: false,
            include_implicit: false,
            sources: vec![
                RawKnowledge {
                    text: "Signal detection uses PRR for safety.".to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: None,
                    timestamp: DateTime::now(),
                },
                // This source is ONLY verbose throat-clearing — compresses to empty.
                // No trailing period: the patterns consume all content, leaving "".
                RawKnowledge {
                    text: "It is important to note that needless to say as a matter of fact"
                        .to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: None,
                    timestamp: DateTime::now(),
                },
            ],
        };

        let pack = compiler.compile(options).unwrap();
        assert_eq!(pack.stats.sources_attempted, 2);
        // The throat-clearing source should have been dropped
        assert!(
            pack.stats.sources_failed >= 1,
            "expected at least 1 failed source, got {}",
            pack.stats.sources_failed
        );
    }

    #[test]
    fn ingest_single_works() {
        let frag = KnowledgeCompiler::ingest_single(
            "Homeostasis control loop monitors system health.",
            KnowledgeSource::FreeText,
            None,
        )
        .unwrap();
        assert!(!frag.concepts.is_empty());
    }

    #[test]
    fn compress_text_returns_named_struct() {
        let result = KnowledgeCompiler::compress_text(
            "In order to make a decision, we need to perform an analysis.",
        );
        assert!(result.compression_ratio > 0.0);
        assert!(!result.compressed_text.is_empty());
        assert!(
            result.compressed_score.expression_cost <= result.original_score.expression_cost,
            "compressed should have fewer or equal words"
        );
    }
}
