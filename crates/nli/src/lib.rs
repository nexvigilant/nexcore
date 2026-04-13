//! Natural Language Interface — 4-layer pipeline for ASR → intent classification
//! → context management → response generation.
//!
//! # Pipeline (12 steps)
//!
//! 1. VAD: detect speech in audio frame
//! 2. ASR: transcribe audio to text
//! 3. Disfluency filter: remove filler words
//! 4. Coreference resolution: substitute pronouns with prior entities
//! 5. Intent classification: classify user intent kind
//! 6. Entity extraction: extract drug names, MedDRA terms, slots
//! 7. Session store load: retrieve conversation context
//! 8. Proactive surface check: S = U × R × T
//! 9. Module context apply: update active module key
//! 10. Tone calibration: adapt register to user preference
//! 11. Modality routing: select Voice/HUD/Both
//! 12. Session store save: persist updated context

pub mod acoustic;
pub mod config;
pub mod error;
pub mod generation;
pub mod pragmatic;
pub mod semantic;
pub mod types;

pub use error::NliError;

use crate::{
    acoustic::{AcousticLayer, AsrProcessor},
    config::NliConfig,
    generation::{GenerationRouter, HudRenderer, TtsEngine},
    pragmatic::{ConversationContext, SessionStore},
    semantic::{DomainVocabulary, EntityExtractor, IntentClassifier, SemanticEngine},
    types::{
        ClassifiedIntent, ContentCategory, IntentKind, PipelineInput, PipelineOutput, SessionId,
        Transcript, Turn, TurnRole,
    },
};
use std::collections::HashMap;

/// The top-level NLI pipeline.
pub struct NliPipeline {
    acoustic: AcousticLayer,
    semantic: SemanticEngine,
    session_store: Box<dyn SessionStore>,
    generation: GenerationRouter,
    config: NliConfig,
}

impl NliPipeline {
    /// Build a pipeline from configuration with all provided components.
    pub fn from_config(
        config: NliConfig,
        asr_processor: Box<dyn AsrProcessor>,
        classifier: Box<dyn IntentClassifier>,
        extractor: Box<dyn EntityExtractor>,
        vocabulary: DomainVocabulary,
        session_store: Box<dyn SessionStore>,
        tts: Option<Box<dyn TtsEngine>>,
        hud: Option<Box<dyn HudRenderer>>,
    ) -> Self {
        let acoustic = AcousticLayer::new(config.vad.clone(), config.asr.clone(), asr_processor);
        let semantic = SemanticEngine::new(classifier, extractor, vocabulary);
        let generation = GenerationRouter::new(config.generation.clone(), tts, hud);

        Self {
            acoustic,
            semantic,
            session_store,
            generation,
            config,
        }
    }

    /// Execute the full 12-step NLI pipeline.
    pub async fn process(&mut self, input: PipelineInput) -> Result<PipelineOutput, NliError> {
        // Steps 1-3: Acoustic layer (VAD → ASR → disfluency filter).
        let transcript: Transcript = if let Some(audio) = &input.audio {
            match self.acoustic.process(audio).await? {
                Some(t) => t,
                None => {
                    // No speech detected or low confidence — fall back to text override.
                    match &input.text_override {
                        Some(text) => Transcript {
                            text: text.clone(),
                            confidence: 1.0,
                            speech_detected: false,
                        },
                        None => {
                            return Err(NliError::AsrFailure(
                                "no speech detected and no text override provided".to_string(),
                            ));
                        }
                    }
                }
            }
        } else {
            // Text-only mode: skip acoustic layer entirely.
            match &input.text_override {
                Some(text) => Transcript {
                    text: text.clone(),
                    confidence: 1.0,
                    speech_detected: false,
                },
                None => {
                    return Err(NliError::AsrFailure(
                        "neither audio nor text_override provided".to_string(),
                    ));
                }
            }
        };

        // Step 7 (early): Load session context before coreference resolution.
        let mut context = self
            .session_store
            .load(&input.session_id)
            .await?
            .unwrap_or_else(|| {
                ConversationContext::new(
                    input.session_id.clone(),
                    self.config.pragmatic.max_history_turns,
                )
            });

        // Step 9: Apply module context before classification.
        context.apply_module_context(input.module_context.clone());

        // Step 4: Coreference resolution using loaded context.
        let (resolved_text, resolved_coreferences) = context.resolve_coreferences(&transcript.text);

        let resolved_transcript = Transcript {
            text: resolved_text,
            ..transcript
        };

        // Steps 5-6: Semantic layer (intent classification + entity extraction).
        let mut intent: ClassifiedIntent = self.semantic.process(&resolved_transcript).await?;

        // Step 8: Proactive surface check — S = U × R × T.
        let proactively_surfaced =
            context.should_proactively_surface(&intent, &self.config.pragmatic);

        // Record the user turn in context.
        let user_turn = Turn {
            role: TurnRole::User,
            text: resolved_transcript.text.clone(),
            intent: Some(intent.clone()),
            timestamp_ms: current_timestamp_ms(),
        };
        context.push_turn(user_turn);

        // Step 10: Tone calibration.
        let raw_response = build_raw_response(&intent);
        let calibrated = self
            .generation
            .calibrate_tone(&raw_response, &input.user_model.tone_preference);

        // Step 11: Modality routing.
        let category = content_category(&intent.kind);
        let modality = self.generation.select_modality(
            &calibrated,
            &category,
            &intent.kind,
            &input.user_model,
        );

        // Voice summary.
        let voice_summary = match modality {
            types::OutputModality::VoiceOnly | types::OutputModality::Both => {
                Some(self.generation.summarize_for_voice(&calibrated))
            }
            types::OutputModality::HudOnly => None,
        };

        // Record assistant turn.
        let assistant_turn = Turn {
            role: TurnRole::Assistant,
            text: calibrated.clone(),
            intent: None,
            timestamp_ms: current_timestamp_ms(),
        };
        context.push_turn(assistant_turn);

        // Step 12: Persist updated session context.
        self.session_store.save(&input.session_id, &context).await?;

        Ok(PipelineOutput {
            intent,
            response_text: calibrated,
            voice_summary,
            modality,
            proactively_surfaced,
            resolved_coreferences,
        })
    }
}

/// Build a placeholder raw response based on classified intent.
/// In a real deployment this would delegate to an LLM or template engine.
fn build_raw_response(intent: &ClassifiedIntent) -> String {
    match intent.kind {
        IntentKind::Crisis => {
            "This is an urgent situation. Please contact emergency services immediately."
                .to_string()
        }
        IntentKind::SignalDetection => {
            "Running signal detection analysis. Please wait while I retrieve the data.".to_string()
        }
        IntentKind::DrugSafetyQuery => {
            "Retrieving drug safety information for your query.".to_string()
        }
        IntentKind::CausalityAssessment => "Starting causality assessment workflow.".to_string(),
        IntentKind::ReportSubmission => {
            "Preparing report submission. I will guide you through the required fields.".to_string()
        }
        IntentKind::Navigation => "How can I help you navigate the system?".to_string(),
        IntentKind::Conversational => "I understand. How can I assist you further?".to_string(),
        IntentKind::Unknown => "I did not understand that request. Could you rephrase?".to_string(),
    }
}

/// Map intent kind to content category for modality routing.
fn content_category(kind: &IntentKind) -> ContentCategory {
    match kind {
        IntentKind::Crisis => ContentCategory::Crisis,
        IntentKind::SignalDetection | IntentKind::CausalityAssessment => ContentCategory::DataHeavy,
        _ => ContentCategory::General,
    }
}

/// Current timestamp in milliseconds (monotonic fallback to 0 on error).
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
