//! Layer 1: Voice Activity Detection, ASR trait, and disfluency filtering.

use crate::{
    config::{AsrConfig, VadConfig},
    error::NliError,
    types::{AudioFrame, Transcript},
};

/// Voice Activity Detector using RMS energy with adaptive ambient tracking.
pub struct VoiceActivityDetector {
    config: VadConfig,
    /// Exponential moving average of ambient RMS (noise floor).
    ambient_rms: f64,
    /// EMA smoothing factor for ambient noise floor update.
    ema_alpha: f64,
}

impl VoiceActivityDetector {
    /// Create a new VAD with the given configuration.
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            ambient_rms: 0.01,
            ema_alpha: 0.05,
        }
    }

    /// Compute RMS energy of a sample window.
    fn rms(samples: &[i16]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = samples
            .iter()
            .map(|&s| {
                let sf = f64::from(s);
                sf * sf
            })
            .sum();
        (sum_sq / samples.len() as f64).sqrt()
    }

    /// Determine whether the audio frame contains speech.
    ///
    /// Uses a sliding window RMS compared against an adaptive ambient
    /// noise floor. Returns `true` when RMS > ambient × threshold_multiplier.
    /// Also updates the internal ambient noise floor via EMA.
    pub fn is_speech(&mut self, frame: &AudioFrame) -> bool {
        let window = self.config.window_size_samples.min(frame.samples.len());
        let slice = &frame.samples[..window];
        let current_rms = Self::rms(slice);

        // Update ambient floor when signal is below threshold (silence).
        let threshold = self.ambient_rms * self.config.speech_threshold_multiplier;
        if current_rms <= threshold {
            self.ambient_rms =
                self.ema_alpha * current_rms + (1.0 - self.ema_alpha) * self.ambient_rms;
        }

        current_rms > self.ambient_rms * self.config.speech_threshold_multiplier
    }
}

/// ASR processor trait — accepts an audio frame and produces a transcript.
#[async_trait::async_trait]
pub trait AsrProcessor: Send + Sync {
    /// Transcribe an audio frame into text.
    async fn transcribe(&self, frame: &AudioFrame) -> Result<Transcript, NliError>;
}

/// Filters disfluencies (filler words) from a transcript.
pub struct DisfluencyFilter {
    fillers: Vec<String>,
}

impl DisfluencyFilter {
    /// Create a filter with the default filler word list.
    pub fn new() -> Self {
        Self {
            fillers: default_fillers(),
        }
    }

    /// Create a filter with a custom filler word list.
    pub fn with_fillers(fillers: Vec<String>) -> Self {
        Self { fillers }
    }

    /// Remove filler tokens from the input text.
    ///
    /// Splits on whitespace, discards any token that exactly matches a
    /// filler word (case-insensitive), and rejoins with single spaces.
    pub fn filter(&self, text: &str) -> String {
        text.split_whitespace()
            .filter(|token| {
                let lower = token.to_lowercase();
                // Strip trailing punctuation for comparison.
                let stripped = lower.trim_end_matches(|c: char| !c.is_alphanumeric());
                !self.fillers.iter().any(|f| f == stripped)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for DisfluencyFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// ASR engine that combines VAD gating with an underlying processor.
pub struct AcousticLayer {
    vad: VoiceActivityDetector,
    processor: Box<dyn AsrProcessor>,
    filter: DisfluencyFilter,
    asr_config: AsrConfig,
}

impl AcousticLayer {
    /// Create a new acoustic layer.
    pub fn new(
        vad_config: VadConfig,
        asr_config: AsrConfig,
        processor: Box<dyn AsrProcessor>,
    ) -> Self {
        Self {
            vad: VoiceActivityDetector::new(vad_config),
            processor,
            filter: DisfluencyFilter::new(),
            asr_config,
        }
    }

    /// Process an audio frame: VAD gate → ASR → disfluency filter.
    ///
    /// Returns `None` when no speech is detected; `Some(Transcript)` otherwise.
    pub async fn process(&mut self, frame: &AudioFrame) -> Result<Option<Transcript>, NliError> {
        if !self.vad.is_speech(frame) {
            return Ok(None);
        }

        let mut transcript = self.processor.transcribe(frame).await?;

        if transcript.confidence < self.asr_config.min_confidence {
            return Ok(None);
        }

        transcript.text = self.filter.filter(&transcript.text);
        transcript.speech_detected = true;
        Ok(Some(transcript))
    }
}

/// Default disfluency filler words.
fn default_fillers() -> Vec<String> {
    vec![
        "um".to_string(),
        "uh".to_string(),
        "like".to_string(),
        "you know".to_string(),
        "so".to_string(),
        "well".to_string(),
        "actually".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn silent_frame() -> AudioFrame {
        AudioFrame::new(vec![10i16; 800], 16000)
    }

    fn loud_frame() -> AudioFrame {
        AudioFrame::new(vec![3000i16; 800], 16000)
    }

    #[test]
    fn vad_silent_frame_not_speech() {
        let mut vad = VoiceActivityDetector::new(VadConfig::default());
        // A frame of all-zero samples has RMS = 0.0, which is always below
        // the initial ambient floor (0.01) × threshold multiplier (1.5) = 0.015.
        let silent = AudioFrame::new(vec![0i16; 800], 16000);
        assert!(!vad.is_speech(&silent));
    }

    #[test]
    fn vad_loud_frame_is_speech() {
        let mut vad = VoiceActivityDetector::new(VadConfig::default());
        assert!(vad.is_speech(&loud_frame()));
    }

    #[test]
    fn disfluency_filter_removes_fillers() {
        let filter = DisfluencyFilter::new();
        let input = "um the patient uh experienced like nausea";
        let output = filter.filter(input);
        assert_eq!(output, "the patient experienced nausea");
    }

    #[test]
    fn disfluency_filter_preserves_non_fillers() {
        let filter = DisfluencyFilter::new();
        let input = "semaglutide caused nausea";
        assert_eq!(filter.filter(input), "semaglutide caused nausea");
    }

    #[test]
    fn rms_empty_returns_zero() {
        assert_eq!(VoiceActivityDetector::rms(&[]), 0.0);
    }
}
