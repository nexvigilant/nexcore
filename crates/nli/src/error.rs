//! Error types for the Natural Language Interface pipeline.

use nexcore_error_derive::Error;

/// Errors produced by the NLI pipeline.
#[derive(Debug, Error)]
pub enum NliError {
    /// ASR transcription failed.
    #[error("ASR transcription failed: {0}")]
    AsrFailure(String),

    /// Intent classification returned no result.
    #[error("intent classification failed: {0}")]
    ClassificationFailure(String),

    /// Entity extraction error.
    #[error("entity extraction failed: {0}")]
    EntityExtractionFailure(String),

    /// Session store read/write failure.
    #[error("session store error: {0}")]
    SessionStoreError(String),

    /// Context management error.
    #[error("context error: {0}")]
    ContextError(String),

    /// Response generation failure.
    #[error("generation failed: {0}")]
    GenerationFailure(String),

    /// Domain vocabulary load failure.
    #[error("vocabulary load failed: {0}")]
    VocabularyLoadFailure(String),

    /// TTS engine error.
    #[error("TTS engine error: {0}")]
    TtsError(String),

    /// HUD render error.
    #[error("HUD render error: {0}")]
    HudError(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// I/O error forwarded from std.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
