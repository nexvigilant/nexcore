//! Publishing pipeline orchestrator.
//!
//! Seven-stage pipeline: INGEST → METADATA → STRUCTURE → STYLE → COVER → GENERATE → VALIDATE.
//! Takes a `.docx` file path and produces a validated `.epub` file.

use std::path::{Path, PathBuf};

use crate::chapter::{self, Chapter, HeadingLevel};
use crate::cover::CoverSpec;
use crate::docx::DocxDocument;
use crate::epub::{self, EpubManifest, EpubOptions};
use crate::error::{PublishingError, Result};
use crate::kindle::{self, KdpComplianceReport};
use crate::metadata::BookMetadata;
use crate::validate::{self, EpubValidation};

/// Publishing target format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishingTarget {
    /// Standard EPUB 3.0.
    Epub,
    /// Kindle-optimized EPUB (passes KDP compliance).
    Kindle,
    /// Both EPUB and Kindle-optimized.
    Both,
}

/// Configuration for the publishing pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Input DOCX file path.
    pub input: PathBuf,
    /// Output directory for generated files.
    pub output_dir: PathBuf,
    /// Output filename (without extension). Defaults to input filename.
    pub output_name: Option<String>,
    /// Publishing target.
    pub target: PublishingTarget,
    /// Optional cover image path.
    pub cover_path: Option<PathBuf>,
    /// Override metadata (merged with DOCX-extracted metadata).
    pub metadata_override: Option<BookMetadata>,
    /// Heading level to split chapters on.
    pub chapter_split_level: HeadingLevel,
    /// Custom CSS (overrides default).
    pub custom_css: Option<String>,
    /// Skip validation step.
    pub skip_validation: bool,
}

impl PipelineConfig {
    /// Create a minimal pipeline config.
    pub fn new(input: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            output_dir: output_dir.into(),
            output_name: None,
            target: PublishingTarget::Epub,
            cover_path: None,
            metadata_override: None,
            chapter_split_level: HeadingLevel::H1,
            custom_css: None,
            skip_validation: false,
        }
    }

    /// Set the publishing target.
    pub fn with_target(mut self, target: PublishingTarget) -> Self {
        self.target = target;
        self
    }

    /// Set the cover image path.
    pub fn with_cover(mut self, path: impl Into<PathBuf>) -> Self {
        self.cover_path = Some(path.into());
        self
    }

    /// Set metadata override.
    pub fn with_metadata(mut self, metadata: BookMetadata) -> Self {
        self.metadata_override = Some(metadata);
        self
    }

    /// Derive the output filename stem.
    fn output_stem(&self) -> String {
        if let Some(ref name) = self.output_name {
            return name.clone();
        }
        self.input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("book")
            .to_string()
    }
}

/// Result of a completed publishing pipeline run.
#[derive(Debug)]
pub struct PipelineResult {
    /// Stages that were executed.
    pub stages: Vec<StageResult>,
    /// Final EPUB file path (if generated).
    pub epub_path: Option<PathBuf>,
    /// EPUB manifest summary.
    pub manifest: Option<EpubManifest>,
    /// Validation result.
    pub validation: Option<EpubValidation>,
    /// KDP compliance report (if Kindle target).
    pub kdp_report: Option<KdpComplianceReport>,
    /// Total word count.
    pub word_count: usize,
    /// Chapter count.
    pub chapter_count: usize,
}

/// Result of a single pipeline stage.
#[derive(Debug)]
pub struct StageResult {
    pub stage: PipelineStage,
    pub success: bool,
    pub message: String,
}

/// Pipeline stages in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Ingest,
    Metadata,
    Structure,
    Style,
    Cover,
    Generate,
    Validate,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ingest => write!(f, "INGEST"),
            Self::Metadata => write!(f, "METADATA"),
            Self::Structure => write!(f, "STRUCTURE"),
            Self::Style => write!(f, "STYLE"),
            Self::Cover => write!(f, "COVER"),
            Self::Generate => write!(f, "GENERATE"),
            Self::Validate => write!(f, "VALIDATE"),
        }
    }
}

/// Run the full publishing pipeline.
pub fn run(config: &PipelineConfig) -> Result<PipelineResult> {
    let mut stages = Vec::new();

    // ─── Stage 1: INGEST ───
    let docx = match DocxDocument::from_file(&config.input) {
        Ok(doc) => {
            stages.push(StageResult {
                stage: PipelineStage::Ingest,
                success: true,
                message: format!(
                    "{} paragraphs, {} words from {}",
                    doc.paragraph_count(),
                    doc.word_count(),
                    config.input.display()
                ),
            });
            doc
        }
        Err(e) => {
            stages.push(StageResult {
                stage: PipelineStage::Ingest,
                success: false,
                message: format!("{e}"),
            });
            return Err(e);
        }
    };

    // ─── Stage 2: METADATA ───
    let metadata = if let Some(ref override_meta) = config.metadata_override {
        stages.push(StageResult {
            stage: PipelineStage::Metadata,
            success: true,
            message: format!("Using provided metadata: \"{}\"", override_meta.title),
        });
        override_meta.clone()
    } else {
        let meta = docx.to_metadata();
        let issues = meta.validate();
        if issues.is_empty() {
            stages.push(StageResult {
                stage: PipelineStage::Metadata,
                success: true,
                message: format!("Extracted: \"{}\" by {}", meta.title, meta.primary_author()),
            });
        } else {
            stages.push(StageResult {
                stage: PipelineStage::Metadata,
                success: false,
                message: format!("Metadata issues: {}", issues.join("; ")),
            });
            return Err(PublishingError::Metadata(issues.join("; ")));
        }
        meta
    };

    // ─── Stage 3: STRUCTURE ───
    let chapters = chapter::split_into_chapters(&docx.paragraphs, config.chapter_split_level);
    let chapter_count = chapters.len();
    let word_count: usize = chapters.iter().map(|c| c.word_count()).sum();

    if chapters.is_empty() {
        stages.push(StageResult {
            stage: PipelineStage::Structure,
            success: false,
            message: "No chapters detected — document may be empty or lack headings".into(),
        });
        return Err(PublishingError::Chapter(
            "No chapters could be extracted from the document".into(),
        ));
    }

    stages.push(StageResult {
        stage: PipelineStage::Structure,
        success: true,
        message: format!("{chapter_count} chapter(s), {word_count} words"),
    });

    // ─── Stage 4: STYLE ───
    let epub_options = EpubOptions {
        css: config
            .custom_css
            .clone()
            .unwrap_or_else(|| EpubOptions::default().css),
        ..EpubOptions::default()
    };
    stages.push(StageResult {
        stage: PipelineStage::Style,
        success: true,
        message: format!(
            "CSS: {} bytes ({})",
            epub_options.css.len(),
            if config.custom_css.is_some() {
                "custom"
            } else {
                "default"
            }
        ),
    });

    // ─── Stage 5: COVER ───
    let cover = if let Some(ref cover_path) = config.cover_path {
        match CoverSpec::from_file(cover_path) {
            Ok(spec) => {
                // Validate based on target
                let validation = match config.target {
                    PublishingTarget::Kindle | PublishingTarget::Both => spec.validate_kindle(),
                    PublishingTarget::Epub => spec.validate_epub(),
                };
                let cover_ok = validation.valid;
                stages.push(StageResult {
                    stage: PipelineStage::Cover,
                    success: cover_ok,
                    message: format!(
                        "{}x{} px, {} ({})",
                        spec.width,
                        spec.height,
                        spec.format.mime_type(),
                        if cover_ok {
                            "valid"
                        } else {
                            "failed validation"
                        }
                    ),
                });
                if !cover_ok {
                    let failures: Vec<_> = validation
                        .checks
                        .iter()
                        .filter(|c| !c.passed)
                        .map(|c| c.message.clone())
                        .collect();
                    return Err(PublishingError::Cover(failures.join("; ")));
                }
                Some(spec)
            }
            Err(e) => {
                stages.push(StageResult {
                    stage: PipelineStage::Cover,
                    success: false,
                    message: format!("{e}"),
                });
                return Err(e);
            }
        }
    } else {
        stages.push(StageResult {
            stage: PipelineStage::Cover,
            success: true,
            message: "No cover image (optional)".into(),
        });
        None
    };

    // ─── Stage 6: GENERATE ───
    let output_stem = config.output_stem();
    let epub_filename = format!("{output_stem}.epub");
    let epub_path = config.output_dir.join(&epub_filename);

    // Ensure output directory exists
    if !config.output_dir.exists() {
        std::fs::create_dir_all(&config.output_dir)?;
    }

    let manifest = epub::write_epub(
        &epub_path,
        &metadata,
        &chapters,
        cover.as_ref(),
        &epub_options,
    )?;

    stages.push(StageResult {
        stage: PipelineStage::Generate,
        success: true,
        message: format!(
            "{} ({} files, {} bytes)",
            epub_path.display(),
            manifest.file_count,
            manifest.total_bytes
        ),
    });

    // ─── Stage 7: VALIDATE ───
    let validation = if config.skip_validation {
        stages.push(StageResult {
            stage: PipelineStage::Validate,
            success: true,
            message: "Skipped (skip_validation=true)".into(),
        });
        None
    } else {
        let result = validate::validate_epub_file(&epub_path)?;
        stages.push(StageResult {
            stage: PipelineStage::Validate,
            success: result.valid,
            message: format!(
                "{} ({}/{} checks)",
                if result.valid { "VALID" } else { "INVALID" },
                result.checks.iter().filter(|c| c.passed).count(),
                result.checks.len()
            ),
        });
        Some(result)
    };

    // ─── KDP Compliance (if Kindle target) ───
    let kdp_report = if matches!(
        config.target,
        PublishingTarget::Kindle | PublishingTarget::Both
    ) {
        Some(kindle::check_kdp_compliance(
            &metadata,
            &chapters,
            cover.as_ref(),
        ))
    } else {
        None
    };

    Ok(PipelineResult {
        stages,
        epub_path: Some(epub_path),
        manifest: Some(manifest),
        validation,
        kdp_report,
        word_count,
        chapter_count,
    })
}

impl PipelineResult {
    /// Format as a human-readable summary.
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Publishing Pipeline Report\n");
        report.push_str(&"=".repeat(40));
        report.push('\n');

        for stage in &self.stages {
            let icon = if stage.success { "+" } else { "-" };
            report.push_str(&format!("  [{icon}] {}: {}\n", stage.stage, stage.message));
        }

        report.push_str(&format!(
            "\nSummary: {} chapters, {} words\n",
            self.chapter_count, self.word_count
        ));

        if let Some(ref path) = self.epub_path {
            report.push_str(&format!("Output: {}\n", path.display()));
        }

        if let Some(ref kdp) = self.kdp_report {
            report.push_str(&format!(
                "KDP: {} ({}/{})\n",
                if kdp.compliant {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                },
                kdp.passed_count(),
                kdp.checks.len()
            ));
        }

        report
    }

    /// Whether the entire pipeline succeeded.
    pub fn success(&self) -> bool {
        self.stages.iter().all(|s| s.success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Build a test DOCX in a temp file and run the pipeline.
    fn run_test_pipeline(paragraphs: &[(&str, &str)]) -> Result<PipelineResult> {
        let docx_bytes = build_test_docx(paragraphs);
        let dir = tempfile::tempdir().map_err(|e| PublishingError::Io(e))?;
        let input_path = dir.path().join("test.docx");
        let output_dir = dir.path().join("output");

        std::fs::write(&input_path, &docx_bytes)?;

        let config = PipelineConfig::new(&input_path, &output_dir);
        let result = run(&config)?;

        // Keep tempdir alive until we're done
        let _keep = dir;
        Ok(result)
    }

    #[test]
    fn test_full_pipeline() {
        let result = run_test_pipeline(&[
            ("Heading1", "Chapter One"),
            (
                "Normal",
                "This is the first chapter with enough content to pass word count checks for testing purposes. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
            ),
            ("Heading1", "Chapter Two"),
            (
                "Normal",
                "This is the second chapter with additional content. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit.",
            ),
        ]);

        match result {
            Ok(r) => {
                assert!(r.success(), "Pipeline report:\n{}", r.to_report());
                assert_eq!(r.chapter_count, 2);
                assert!(r.epub_path.is_some());
            }
            Err(e) => panic!("Pipeline failed: {e}"),
        }
    }

    #[test]
    fn test_pipeline_with_metadata_override() {
        let docx_bytes = build_test_docx(&[
            ("Heading1", "Ch1"),
            (
                "Normal",
                "Content here with enough words to pass minimum content requirements for the test to succeed properly.",
            ),
        ]);
        let dir = tempfile::tempdir().expect("tempdir");
        let input_path = dir.path().join("test.docx");
        let output_dir = dir.path().join("output");
        std::fs::write(&input_path, &docx_bytes).expect("write");

        let meta = BookMetadata::new("Custom Title", "Custom Author", "en-US");
        let config = PipelineConfig::new(&input_path, &output_dir).with_metadata(meta);
        let result = run(&config).expect("pipeline");

        assert!(result.success());
        // Check the stage message mentions custom metadata
        assert!(
            result
                .stages
                .iter()
                .any(|s| s.message.contains("Custom Title"))
        );
    }

    #[test]
    fn test_pipeline_kindle_target() {
        let docx_bytes = build_test_docx(&[
            ("Heading1", "Chapter"),
            (
                "Normal",
                "Enough content here to pass the minimum word count for KDP compliance checking in the publishing pipeline test.",
            ),
        ]);
        let dir = tempfile::tempdir().expect("tempdir");
        let input_path = dir.path().join("test.docx");
        let output_dir = dir.path().join("output");
        std::fs::write(&input_path, &docx_bytes).expect("write");

        let config =
            PipelineConfig::new(&input_path, &output_dir).with_target(PublishingTarget::Kindle);
        let result = run(&config).expect("pipeline");

        assert!(result.kdp_report.is_some());
    }

    /// Build minimal DOCX for testing (mirrors docx.rs test helper).
    fn build_test_docx(paragraphs: &[(&str, &str)]) -> Vec<u8> {
        use crate::chapter::xml_escape;

        let mut buf = Vec::new();
        let cursor = Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("[Content_Types].xml", options).ok();
        std::io::Write::write_all(
            &mut zip,
            br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/></Types>"#,
        ).ok();

        zip.start_file("_rels/.rels", options).ok();
        std::io::Write::write_all(
            &mut zip,
            br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>"#,
        ).ok();

        zip.start_file("word/document.xml", options).ok();
        let mut doc = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
        );
        for (style, text) in paragraphs {
            doc.push_str(&format!(
                r#"<w:p><w:pPr><w:pStyle w:val="{s}"/></w:pPr><w:r><w:t>{t}</w:t></w:r></w:p>"#,
                s = xml_escape(style),
                t = xml_escape(text),
            ));
        }
        doc.push_str("</w:body></w:document>");
        std::io::Write::write_all(&mut zip, doc.as_bytes()).ok();

        zip.start_file("docProps/core.xml", options).ok();
        std::io::Write::write_all(
            &mut zip,
            br#"<?xml version="1.0" encoding="UTF-8"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test Book</dc:title><dc:creator>Test Author</dc:creator></cp:coreProperties>"#,
        ).ok();

        zip.finish().ok();
        buf
    }
}
