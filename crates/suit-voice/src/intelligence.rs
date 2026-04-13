//! # Voice Agent Intelligence Layers
//!
//! Multi-layer conversational intelligence pipeline (Semantic to Pragmatic Generation).

use serde::{Deserialize, Serialize};

/// Semantic Layer: NER, intent extraction, slot filling.
pub mod semantic {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SemanticParse {
        pub intent: String,
        pub slots: Vec<(String, String)>,
        pub confidence: f32,
    }
}

/// Pragmatic Layer: Context tracking and dialogue management.
pub mod pragmatic {
    use crate::semantic::SemanticParse;

    pub struct ConversationContext {
        pub history: Vec<SemanticParse>,
        pub current_topic: String,
    }
}

/// Generation Layer: TTS, modality switching, tone control.
pub mod generation {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Modality {
        Voice,
        HudOnly,
        Haptic,
    }

    pub struct OutputPackage {
        pub text: String,
        pub modality: Modality,
        pub prosody: f32, // Tone calibration
    }
}
