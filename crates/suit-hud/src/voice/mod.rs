//! Voice Agent — 7.3
//!
//! On-device wake word, ASR (Whisper), LLM routing (local → cloud),
//! TTS (Piper), and MCP tool bridge for suit telemetry/nav/comms.
//!
//! Bridges to `nexcore-audio` for VAD, AEC, NoiseGate, and STT.

use serde::{Deserialize, Serialize};

/// 7.3.1 Wake-word detector state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WakeWordState {
    /// Listening for wake word.
    Listening,
    /// Wake word detected — ASR pipeline active.
    Activated,
    /// Cooldown after command processed.
    Cooldown,
    /// Disabled (manual input only).
    Disabled,
}

/// 7.3.2 ASR backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsrBackend {
    /// On-device Whisper (nexcore-audio STT).
    WhisperLocal,
    /// Cloud fallback (Claude API or external STT).
    CloudFallback,
    /// Hybrid — local first, cloud if confidence < threshold.
    Hybrid,
}

/// 7.3.3 LLM routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmRoute {
    /// Small on-device model (fast, limited).
    Local {
        /// Model name.
        model: String,
        /// Max tokens.
        max_tokens: u32,
    },
    /// Claude API (full capability).
    Cloud {
        /// Model ID (e.g., "claude-sonnet-4-6").
        model: String,
        /// Whether to use tool_use.
        tools_enabled: bool,
    },
    /// Cascade: try local first, escalate to cloud if needed.
    Cascade {
        /// Local model.
        local_model: String,
        /// Cloud model.
        cloud_model: String,
        /// Confidence threshold to escalate.
        escalation_threshold: f32,
    },
}

impl Default for LlmRoute {
    fn default() -> Self {
        Self::Cascade {
            local_model: "llama-3.2-3b".to_string(),
            cloud_model: "claude-sonnet-4-6".to_string(),
            escalation_threshold: 0.7,
        }
    }
}

/// 7.3.4 TTS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    /// Voice model path.
    pub model: String,
    /// Speech rate (1.0 = normal).
    pub speed: f32,
    /// Output route.
    pub output: AudioOutput,
}

/// Audio output routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioOutput {
    /// 7.1.5 Bone conduction transducer.
    BoneConduction,
    /// In-helmet speaker.
    Speaker,
    /// External speaker (suit PA).
    External,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            model: "en_GB-alan-medium".to_string(),
            speed: 0.8,
            output: AudioOutput::BoneConduction,
        }
    }
}

/// 7.3.5 MCP tool bridge — suit telemetry, nav, comms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpBridge {
    /// Connected MCP servers.
    pub servers: Vec<String>,
    /// Available tool count.
    pub tool_count: u32,
    /// Whether the bridge is active.
    pub active: bool,
    /// Last command routed.
    pub last_command: Option<String>,
}

impl Default for McpBridge {
    fn default() -> Self {
        Self {
            servers: vec!["nexcore".to_string(), "nexvigilant-station".to_string()],
            tool_count: 0,
            active: false,
            last_command: None,
        }
    }
}

/// A parsed voice command from ASR output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    /// Raw transcribed text.
    pub raw_text: String,
    /// Parsed intent (e.g., "navigate", "status", "threat_scan").
    pub intent: String,
    /// Extracted parameters.
    pub params: std::collections::HashMap<String, String>,
    /// ASR confidence (0.0..1.0).
    pub confidence: f32,
    /// Whether this requires cloud LLM to resolve.
    pub needs_llm: bool,
}

/// Complete voice agent state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Wake word detector state.
    pub wake_word: WakeWordState,
    /// ASR backend.
    pub asr: AsrBackend,
    /// LLM routing config.
    pub llm_route: LlmRoute,
    /// TTS config.
    pub tts: TtsConfig,
    /// MCP bridge state.
    pub mcp_bridge: McpBridge,
    /// Whether the agent is currently processing a command.
    pub processing: bool,
    /// Last processed command (if any).
    pub last_command: Option<VoiceCommand>,
    /// Commands processed this session.
    pub commands_processed: u64,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            wake_word: WakeWordState::Listening,
            asr: AsrBackend::Hybrid,
            llm_route: LlmRoute::default(),
            tts: TtsConfig::default(),
            mcp_bridge: McpBridge::default(),
            processing: false,
            last_command: None,
            commands_processed: 0,
        }
    }
}

impl AgentState {
    /// Process an audio frame through the voice pipeline.
    ///
    /// Chains: nexcore-audio VAD → wake word check → ASR (if speech detected).
    /// Returns the pipeline result for this frame.
    pub fn process_audio_frame(
        &mut self,
        frame: &[f32],
        vad: &mut nexcore_audio::vad::VoiceDetector,
    ) -> AudioPipelineResult {
        let vad_result = vad.process(frame);

        match self.wake_word {
            WakeWordState::Listening => {
                // In listening mode — only care about speech detection for wake word
                if vad_result.is_speech {
                    AudioPipelineResult::SpeechDetected {
                        energy: vad_result.energy,
                        state: vad_result.state,
                    }
                } else {
                    AudioPipelineResult::Silence
                }
            }
            WakeWordState::Activated => {
                // Wake word was detected — accumulate speech for ASR
                if vad_result.is_speech {
                    AudioPipelineResult::Recording {
                        energy: vad_result.energy,
                    }
                } else {
                    // Speech ended — ready for transcription
                    AudioPipelineResult::ReadyForAsr
                }
            }
            WakeWordState::Cooldown => AudioPipelineResult::Silence,
            WakeWordState::Disabled => AudioPipelineResult::Silence,
        }
    }

    /// Activate the agent after wake word detection.
    pub fn activate(&mut self) {
        self.wake_word = WakeWordState::Activated;
        self.processing = true;
    }

    /// Record a completed command.
    pub fn record_command(&mut self, command: VoiceCommand) {
        self.last_command = Some(command);
        self.commands_processed = self.commands_processed.saturating_add(1);
        self.wake_word = WakeWordState::Cooldown;
        self.processing = false;
    }

    /// Return to listening state after cooldown.
    pub fn resume_listening(&mut self) {
        self.wake_word = WakeWordState::Listening;
    }
}

/// Result of processing one audio frame through the voice pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioPipelineResult {
    /// No speech detected.
    Silence,
    /// Speech detected while listening for wake word.
    SpeechDetected {
        /// Frame energy.
        energy: f32,
        /// VAD state.
        state: nexcore_audio::vad::VadState,
    },
    /// Recording speech after wake word activation.
    Recording {
        /// Frame energy.
        energy: f32,
    },
    /// Speech ended after activation — send accumulated audio to ASR.
    ReadyForAsr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_state() {
        let a = AgentState::default();
        assert_eq!(a.wake_word, WakeWordState::Listening);
        assert_eq!(a.asr, AsrBackend::Hybrid);
        assert!(!a.processing);
        assert_eq!(a.commands_processed, 0);
    }

    #[test]
    fn llm_route_cascade_default() {
        let r = LlmRoute::default();
        match r {
            LlmRoute::Cascade {
                escalation_threshold,
                ..
            } => {
                assert!((escalation_threshold - 0.7).abs() < 0.01);
            }
            _ => panic!("expected Cascade"),
        }
    }

    #[test]
    fn tts_default_bone_conduction() {
        let t = TtsConfig::default();
        assert_eq!(t.output, AudioOutput::BoneConduction);
        assert!(t.speed > 0.0);
    }

    #[test]
    fn voice_command_construction() {
        let cmd = VoiceCommand {
            raw_text: "navigate to alpha waypoint".into(),
            intent: "navigate".into(),
            params: [("target".into(), "alpha".into())].into(),
            confidence: 0.92,
            needs_llm: false,
        };
        assert_eq!(cmd.intent, "navigate");
        assert!(!cmd.needs_llm);
    }

    #[test]
    fn mcp_bridge_default() {
        let b = McpBridge::default();
        assert!(!b.active);
        assert!(b.servers.contains(&"nexcore".to_string()));
    }

    #[test]
    fn agent_state_serializes() {
        let a = AgentState::default();
        let json = serde_json::to_string(&a);
        assert!(json.is_ok());
    }

    #[test]
    fn pipeline_silence_when_listening() {
        let mut agent = AgentState::default();
        let mut vad =
            nexcore_audio::vad::VoiceDetector::new(nexcore_audio::vad::VadConfig::default());
        let silence = vec![0.0f32; 480];
        let result = agent.process_audio_frame(&silence, &mut vad);
        assert!(matches!(result, AudioPipelineResult::Silence));
    }

    #[test]
    fn pipeline_detects_speech() {
        let mut agent = AgentState::default();
        let mut vad = nexcore_audio::vad::VoiceDetector::new(nexcore_audio::vad::VadConfig {
            onset_frames: 1,
            ..nexcore_audio::vad::VadConfig::default()
        });
        let speech: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();
        // Need 2 frames for onset→speech
        agent.process_audio_frame(&speech, &mut vad);
        let result = agent.process_audio_frame(&speech, &mut vad);
        assert!(matches!(result, AudioPipelineResult::SpeechDetected { .. }));
    }

    #[test]
    fn pipeline_records_after_activation() {
        let mut agent = AgentState::default();
        agent.activate();
        assert_eq!(agent.wake_word, WakeWordState::Activated);

        let mut vad = nexcore_audio::vad::VoiceDetector::new(nexcore_audio::vad::VadConfig {
            onset_frames: 1,
            ..nexcore_audio::vad::VadConfig::default()
        });
        let speech: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();
        vad.process(&speech); // prime onset
        let result = agent.process_audio_frame(&speech, &mut vad);
        assert!(matches!(result, AudioPipelineResult::Recording { .. }));
    }

    #[test]
    fn pipeline_ready_for_asr_on_silence() {
        let mut agent = AgentState::default();
        agent.activate();

        let mut vad =
            nexcore_audio::vad::VoiceDetector::new(nexcore_audio::vad::VadConfig::default());
        let silence = vec![0.0f32; 480];
        let result = agent.process_audio_frame(&silence, &mut vad);
        assert!(matches!(result, AudioPipelineResult::ReadyForAsr));
    }

    #[test]
    fn record_command_transitions_to_cooldown() {
        let mut agent = AgentState::default();
        agent.activate();
        agent.record_command(VoiceCommand {
            raw_text: "status".into(),
            intent: "status".into(),
            params: Default::default(),
            confidence: 0.95,
            needs_llm: false,
        });
        assert_eq!(agent.wake_word, WakeWordState::Cooldown);
        assert_eq!(agent.commands_processed, 1);
        assert!(!agent.processing);

        agent.resume_listening();
        assert_eq!(agent.wake_word, WakeWordState::Listening);
    }
}
