//! # Voice Agent System (7.3)
//! Audio-to-Intent pipeline: ASR, LLM routing, and TTS synthesis.

pub mod audio;

use serde::{Deserialize, Serialize};

/// Audio codec frame definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub pcm_data: Vec<i16>,
    pub sample_rate: u32,
}

/// System intent inferred from ASR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceIntent {
    SystemDiagnostic,
    NavToWaypoint(String),
    PowerEmergencyOverride,
    None,
}

/// Interface for the voice processing pipeline.
pub trait VoiceAgent {
    /// Transcribes audio frames and returns inferred intent.
    fn process_audio(&mut self, frame: AudioFrame) -> Result<VoiceIntent, nexcore_error::NexError>;
    /// Generates voice response for the HUD/Bone-conduction panel.
    fn synthesize(&self, text: &str) -> Result<Vec<i16>, nexcore_error::NexError>;
}
