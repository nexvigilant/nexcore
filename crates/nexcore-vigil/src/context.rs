use crate::memory::MemoryLayer;
use crate::models::Event;
use crate::projects::ProjectRegistry;
use nexcore_chrono::DateTime;
use nexcore_error::NexError;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tiktoken_rs::cl100k_base;
use tokio::sync::RwLock;

/// Convert fmt::Error to NexError.
fn fmt_err(e: std::fmt::Error) -> NexError {
    NexError::new(e.to_string())
}

/// v2.0 Context Assembler: Token-aware context compression and caching.
pub struct ContextAssembler {
    ksb_root: PathBuf,
    memory: Arc<MemoryLayer>,
    registry: Arc<ProjectRegistry>,
    conversation_buffer: RwLock<Vec<(String, String)>>,
    cached_instructions: RwLock<Option<String>>,
    max_tokens: usize,
}

impl ContextAssembler {
    pub fn new(
        ksb_root: PathBuf,
        memory: Arc<MemoryLayer>,
        registry: Arc<ProjectRegistry>,
    ) -> Self {
        Self {
            ksb_root,
            memory,
            registry,
            conversation_buffer: RwLock::new(Vec::new()),
            cached_instructions: RwLock::new(None),
            max_tokens: 8192, // Titan default
        }
    }

    pub async fn build_context(&self, event: &Event) -> nexcore_error::Result<String> {
        let mut context = String::with_capacity(8192);
        let bpe = cl100k_base().map_err(|e| nexcore_error::NexError::msg(e.to_string()))?;

        // 1. Core Identity
        if let Some(i) = self.get_instructions().await {
            writeln!(context, "# Core Instructions\n\n{}\n", i).map_err(fmt_err)?;
        }

        // 2. Project Status
        writeln!(
            context,
            "# Current Projects\n\n{}\n",
            self.registry.get_briefing()
        )
        .map_err(fmt_err)?;

        // 3. Relevant Knowledge (Top-K)
        let query = self.event_to_query(event);
        let docs = self.memory.search(&query, 5).await?;
        if !docs.is_empty() {
            writeln!(context, "# Relevant Knowledge\n\n{:?}\n", docs).map_err(fmt_err)?;
        }

        // 4. Conversation History (Token-aware pruning)
        {
            let buffer = self.conversation_buffer.read().await;
            if !buffer.is_empty() {
                writeln!(context, "# Recent Conversation\n").map_err(fmt_err)?;
                let mut history_str = String::new();
                for (role, content) in buffer.iter().rev() {
                    let next_line = format!("**{}**: {}\n", role, content);
                    if bpe
                        .encode_with_special_tokens(&(context.clone() + &history_str + &next_line))
                        .len()
                        > self.max_tokens
                    {
                        break;
                    }
                    history_str.insert_str(0, &next_line);
                }
                context.push_str(&history_str);
            }
        }

        // 5. System State & Current Event
        writeln!(context, "\n# Current State").map_err(fmt_err)?;
        writeln!(context, "- Time: {}", DateTime::now().to_rfc3339()).map_err(fmt_err)?;
        writeln!(context, "- Source: {}", event.source).map_err(fmt_err)?;
        writeln!(context, "- Type: {}\n", event.event_type).map_err(fmt_err)?;

        writeln!(context, "## Current Event Payload\n{}\n", event.payload).map_err(fmt_err)?;

        Ok(context)
    }

    async fn get_instructions(&self) -> Option<String> {
        {
            let cache = self.cached_instructions.read().await;
            if cache.is_some() {
                return cache.clone();
            }
        }
        let path = self.ksb_root.join("CLAUDE.md");
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            let mut cache = self.cached_instructions.write().await;
            *cache = Some(content.clone());
            return Some(content);
        }
        None
    }

    fn event_to_query(&self, event: &Event) -> String {
        event
            .payload
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or(&event.event_type)
            .to_string()
    }

    pub async fn add_to_buffer(&self, role: String, content: String) {
        let mut buffer = self.conversation_buffer.write().await;
        buffer.push((role, content));
        if buffer.len() > 50 {
            buffer.remove(0);
        }
    }
}
