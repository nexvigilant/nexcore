//! Core data types for the Natural Language Interface pipeline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Opaque session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    /// Create a new session ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Classification of user intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentKind {
    /// Query for drug safety or adverse event information.
    DrugSafetyQuery,
    /// Request for signal detection analysis.
    SignalDetection,
    /// Causality assessment request.
    CausalityAssessment,
    /// Report submission intent.
    ReportSubmission,
    /// Navigation or help request.
    Navigation,
    /// General conversational input.
    Conversational,
    /// Crisis or urgent safety concern.
    Crisis,
    /// Unknown / unclassified intent.
    Unknown,
}

/// A named slot extracted from an utterance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    /// Slot name (e.g. "drug_name", "event_term").
    pub name: String,
    /// Extracted value.
    pub value: SlotValue,
    /// Confidence score [0.0, 1.0].
    pub confidence: f64,
}

/// The typed value of a slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlotValue {
    /// Plain text string.
    Text(String),
    /// Normalized drug name (canonical form).
    Drug(String),
    /// Normalized MedDRA term.
    MedDra(String),
    /// Numeric value.
    Number(f64),
    /// Boolean.
    Bool(bool),
}

/// An intent with its confidence and extracted slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedIntent {
    /// The classified intent kind.
    pub kind: IntentKind,
    /// Confidence score [0.0, 1.0].
    pub confidence: f64,
    /// Extracted slots associated with this intent.
    pub slots: Vec<Slot>,
}

impl ClassifiedIntent {
    /// Create a classified intent with no slots.
    pub fn new(kind: IntentKind, confidence: f64) -> Self {
        Self {
            kind,
            confidence,
            slots: Vec::new(),
        }
    }
}

/// A single conversational turn (user or assistant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// Who produced this turn.
    pub role: TurnRole,
    /// Raw utterance or response text.
    pub text: String,
    /// Classified intent (present for user turns).
    pub intent: Option<ClassifiedIntent>,
    /// Timestamp in milliseconds since epoch.
    pub timestamp_ms: u64,
}

/// Role in a conversation turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnRole {
    /// The human user.
    User,
    /// The NLI assistant.
    Assistant,
}

/// Tone preference for a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TonePreference {
    /// Formal regulatory language.
    Regulatory,
    /// Casual conversational.
    Casual,
    /// Crisis / urgent mode.
    Crisis,
}

/// Model of a user's interaction preferences and profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    /// Preferred tone for responses.
    pub tone_preference: TonePreference,
    /// Preferred output modalities.
    pub modality_preference: OutputModality,
    /// Domain expertise level [0.0 = novice, 1.0 = expert].
    pub expertise_level: f64,
}

impl Default for UserModel {
    fn default() -> Self {
        Self {
            tone_preference: TonePreference::Casual,
            modality_preference: OutputModality::VoiceOnly,
            expertise_level: 0.5,
        }
    }
}

/// Output delivery modality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputModality {
    /// Voice / TTS only.
    VoiceOnly,
    /// HUD / visual display only.
    HudOnly,
    /// Both voice and HUD.
    Both,
}

/// Content category for routing decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentCategory {
    /// Data-heavy output (tables, statistics).
    DataHeavy,
    /// Narrative text.
    Narrative,
    /// Crisis / urgent content.
    Crisis,
    /// General purpose.
    General,
}

/// A raw audio frame for VAD/ASR processing.
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// PCM samples, mono, 16-bit signed.
    pub samples: Vec<i16>,
    /// Sample rate in Hz.
    pub sample_rate_hz: u32,
}

impl AudioFrame {
    /// Create a new audio frame.
    pub fn new(samples: Vec<i16>, sample_rate_hz: u32) -> Self {
        Self {
            samples,
            sample_rate_hz,
        }
    }
}

/// The output of ASR: a raw transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    /// Raw text from the ASR engine.
    pub text: String,
    /// Confidence score [0.0, 1.0].
    pub confidence: f64,
    /// Whether speech was detected in the frame.
    pub speech_detected: bool,
}

/// Pipeline input bundle.
#[derive(Debug, Clone)]
pub struct PipelineInput {
    /// Session identifier for context lookup.
    pub session_id: SessionId,
    /// Raw audio frame (optional — text-only mode if absent).
    pub audio: Option<AudioFrame>,
    /// Text input (used directly if audio is absent, or after ASR).
    pub text_override: Option<String>,
    /// User model driving tone and modality decisions.
    pub user_model: UserModel,
    /// Active module context key (e.g. "signal-detection", "causality").
    pub module_context: Option<String>,
}

/// Final pipeline output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    /// The classified intent that drove this response.
    pub intent: ClassifiedIntent,
    /// Text of the generated response.
    pub response_text: String,
    /// Voice-optimized summary (≤100 words).
    pub voice_summary: Option<String>,
    /// Selected output modality.
    pub modality: OutputModality,
    /// Whether proactive surfacing was triggered.
    pub proactively_surfaced: bool,
    /// Resolved coreferences applied during context management.
    pub resolved_coreferences: HashMap<String, String>,
}
