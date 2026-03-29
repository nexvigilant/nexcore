//! Publishing MCP tools — DOCX to EPUB pipeline and EPUB reader.

use crate::params::publishing::{PublishDocxParams, ReadEpubParams};
use nexcore_publishing::chapter::HeadingLevel;
use nexcore_publishing::pipeline::{PipelineConfig, PublishingTarget};
use nexcore_publishing::read::EpubReader;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Convert a DOCX manuscript to EPUB.
pub fn publish_docx_to_epub(params: PublishDocxParams) -> Result<CallToolResult, McpError> {
    let mut config = PipelineConfig::new(&params.input_path, &params.output_dir);

    // Set target
    if let Some(ref target) = params.target {
        config = config.with_target(match target.to_lowercase().as_str() {
            "kindle" => PublishingTarget::Kindle,
            "both" => PublishingTarget::Both,
            _ => PublishingTarget::Epub,
        });
    }

    // Set cover
    if let Some(ref cover) = params.cover_path {
        config = config.with_cover(cover);
    }

    // Set metadata override if any field provided
    if params.title.is_some() || params.author.is_some() || params.language.is_some() {
        let title = params.title.unwrap_or_default();
        let author = params.author.unwrap_or_else(|| "Unknown Author".into());
        let lang = params.language.unwrap_or_else(|| "en".into());
        let mut meta = nexcore_publishing::metadata::BookMetadata::new(title, author, lang);
        meta.description = params.description;
        config = config.with_metadata(meta);
    }

    config.chapter_split_level = HeadingLevel::H1;

    match nexcore_publishing::pipeline::run(&config) {
        Ok(result) => {
            let report = result.to_report();
            let summary = json!({
                "success": result.success(),
                "epub_path": result.epub_path.as_ref().map(|p| p.display().to_string()),
                "chapters": result.chapter_count,
                "words": result.word_count,
                "kdp_compliant": result.kdp_report.as_ref().map(|r| r.compliant),
                "stages": result.stages.iter().map(|s| json!({
                    "stage": s.stage.to_string(),
                    "success": s.success,
                    "message": s.message,
                })).collect::<Vec<_>>(),
            });

            if result.success() {
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&summary).unwrap_or(report),
                )]))
            } else {
                Ok(CallToolResult::error(vec![Content::text(
                    serde_json::to_string_pretty(&summary).unwrap_or(report),
                )]))
            }
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Publishing failed: {e}"
        ))])),
    }
}

/// Read and extract content from an existing EPUB file.
pub fn read_epub(params: ReadEpubParams) -> Result<CallToolResult, McpError> {
    let reader = match EpubReader::open(&params.path) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Cannot open EPUB: {e}"
            ))]));
        }
    };

    let format = params.format.as_deref().unwrap_or("text");

    // If specific chapter requested, return its content
    if let Some(idx) = params.chapter {
        return match reader.chapter(idx) {
            Some(ch) => {
                let content = if format == "html" { &ch.html } else { &ch.text };
                let result = json!({
                    "title": ch.title,
                    "chapter": idx,
                    "word_count": ch.word_count,
                    "content": content,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "Chapter {idx} not found (book has {} chapters)",
                reader.chapters.len()
            ))])),
        };
    }

    // Return metadata + TOC
    let toc: Vec<_> = reader
        .chapters
        .iter()
        .map(|ch| {
            json!({
                "order": ch.order,
                "title": ch.title,
                "word_count": ch.word_count,
            })
        })
        .collect();

    let result = json!({
        "title": reader.metadata.title,
        "author": reader.metadata.primary_author(),
        "language": reader.metadata.language,
        "chapters": reader.chapters.len(),
        "total_words": reader.total_words,
        "has_cover": reader.cover.is_some(),
        "table_of_contents": toc,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
