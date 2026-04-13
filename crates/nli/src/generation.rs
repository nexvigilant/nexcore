//! Layer 4: Response generation, TTS, HUD rendering, modality routing, tone calibration.

use crate::{
    config::GenerationConfig,
    error::NliError,
    types::{ContentCategory, IntentKind, OutputModality, TonePreference, UserModel},
};

/// TTS engine trait — converts text to audio bytes.
#[async_trait::async_trait]
pub trait TtsEngine: Send + Sync {
    /// Synthesize speech from text. Returns raw audio bytes.
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, NliError>;
}

/// HUD renderer trait — renders text to a display surface.
#[async_trait::async_trait]
pub trait HudRenderer: Send + Sync {
    /// Render text to the HUD. Returns rendered output (bytes or markup).
    async fn render(&self, text: &str) -> Result<Vec<u8>, NliError>;
}

/// Routes content to the appropriate output modality.
pub struct ModalityRouter {
    config: GenerationConfig,
}

impl ModalityRouter {
    /// Create a new modality router.
    pub fn new(config: GenerationConfig) -> Self {
        Self { config }
    }

    /// Select output modality using 5 priority rules.
    ///
    /// Priority order (highest first):
    /// 1. Crisis content → VoiceOnly
    /// 2. Contains table markers (| or repeated \t) → Both
    /// 3. Content length > threshold → Both
    /// 4. DataHeavy category → HudOnly
    /// 5. Default → VoiceOnly
    pub fn select(
        &self,
        content: &str,
        category: &ContentCategory,
        intent_kind: &IntentKind,
        _user_model: &UserModel,
    ) -> OutputModality {
        // Rule 1: Crisis → voice only (fastest path to user).
        if matches!(intent_kind, IntentKind::Crisis) || matches!(category, ContentCategory::Crisis)
        {
            return OutputModality::VoiceOnly;
        }

        // Rule 2: Table markers present → both.
        let has_table = content.contains('|') || content.contains('\t');
        if has_table {
            return OutputModality::Both;
        }

        // Rule 3: Long content → both.
        if content.len() > self.config.long_content_threshold {
            return OutputModality::Both;
        }

        // Rule 4: Data heavy → HUD only.
        if matches!(category, ContentCategory::DataHeavy) {
            return OutputModality::HudOnly;
        }

        // Rule 5: Default.
        OutputModality::VoiceOnly
    }
}

/// Calibrates response tone to match user preference and content context.
pub struct ToneCalibrator {
    config: GenerationConfig,
}

impl ToneCalibrator {
    /// Create a new tone calibrator.
    pub fn new(config: GenerationConfig) -> Self {
        Self { config }
    }

    /// Calibrate text to the appropriate tone.
    ///
    /// - Regulatory: expand contractions (don't → do not, etc.)
    /// - Crisis: truncate sentences to max words each
    /// - Casual: pass through (respect user preference as-is)
    pub fn calibrate(&self, text: &str, tone: &TonePreference) -> String {
        match tone {
            TonePreference::Regulatory => self.expand_contractions(text),
            TonePreference::Crisis => self.truncate_sentences(text),
            TonePreference::Casual => text.to_string(),
        }
    }

    fn expand_contractions(&self, text: &str) -> String {
        let contractions: &[(&str, &str)] = &[
            ("don't", "do not"),
            ("can't", "cannot"),
            ("won't", "will not"),
            ("shouldn't", "should not"),
            ("couldn't", "could not"),
            ("wouldn't", "would not"),
            ("isn't", "is not"),
            ("aren't", "are not"),
            ("wasn't", "was not"),
            ("weren't", "were not"),
            ("hasn't", "has not"),
            ("haven't", "have not"),
            ("hadn't", "had not"),
            ("didn't", "did not"),
            ("doesn't", "does not"),
            ("it's", "it is"),
            ("that's", "that is"),
            ("there's", "there is"),
            ("they're", "they are"),
            ("we're", "we are"),
            ("you're", "you are"),
            ("I'm", "I am"),
            ("I've", "I have"),
            ("I'll", "I will"),
            ("I'd", "I would"),
        ];

        let mut result = text.to_string();
        for (contraction, expansion) in contractions {
            // Case-insensitive replacement.
            let lower = result.to_lowercase();
            if lower.contains(&contraction.to_lowercase()) {
                result = result.replace(contraction, expansion);
                // Handle capitalized first letter.
                let cap = capitalize_first(contraction);
                let cap_exp = capitalize_first(expansion);
                result = result.replace(&cap, &cap_exp);
            }
        }
        result
    }

    fn truncate_sentences(&self, text: &str) -> String {
        let max_words = self.config.crisis_max_words_per_sentence;
        text.split(". ")
            .map(|sentence| {
                let words: Vec<&str> = sentence.split_whitespace().collect();
                if words.len() > max_words {
                    words[..max_words].join(" ")
                } else {
                    sentence.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(". ")
    }
}

/// Routes generation requests and summarizes for voice delivery.
pub struct GenerationRouter {
    tts: Option<Box<dyn TtsEngine>>,
    hud: Option<Box<dyn HudRenderer>>,
    router: ModalityRouter,
    calibrator: ToneCalibrator,
}

impl GenerationRouter {
    /// Create a new generation router.
    pub fn new(
        config: GenerationConfig,
        tts: Option<Box<dyn TtsEngine>>,
        hud: Option<Box<dyn HudRenderer>>,
    ) -> Self {
        let router = ModalityRouter::new(config.clone());
        let calibrator = ToneCalibrator::new(config);
        Self {
            tts,
            hud,
            router,
            calibrator,
        }
    }

    /// Select modality for the given content.
    pub fn select_modality(
        &self,
        content: &str,
        category: &ContentCategory,
        intent_kind: &IntentKind,
        user_model: &UserModel,
    ) -> OutputModality {
        self.router
            .select(content, category, intent_kind, user_model)
    }

    /// Calibrate tone for the response.
    pub fn calibrate_tone(&self, text: &str, tone: &TonePreference) -> String {
        self.calibrator.calibrate(text, tone)
    }

    /// Summarize text for voice delivery.
    ///
    /// Returns the first 2 sentences or first 100 words, whichever is shorter.
    pub fn summarize_for_voice(&self, text: &str) -> String {
        // Take first 2 sentences.
        let sentences: Vec<&str> = text.splitn(3, ". ").collect();
        let sentence_summary = if sentences.len() >= 2 {
            format!("{}. {}.", sentences[0], sentences[1])
        } else {
            text.to_string()
        };

        // Take first 100 words.
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_summary = if words.len() > 100 {
            words[..100].join(" ")
        } else {
            text.to_string()
        };

        // Return whichever is shorter.
        if sentence_summary.len() <= word_summary.len() {
            sentence_summary
        } else {
            word_summary
        }
    }

    /// Synthesize speech via the TTS engine (if configured).
    pub async fn synthesize(&self, text: &str) -> Result<Option<Vec<u8>>, NliError> {
        match &self.tts {
            Some(engine) => Ok(Some(engine.synthesize(text).await?)),
            None => Ok(None),
        }
    }

    /// Render to HUD (if configured).
    pub async fn render_hud(&self, text: &str) -> Result<Option<Vec<u8>>, NliError> {
        match &self.hud {
            Some(engine) => Ok(Some(engine.render(text).await?)),
            None => Ok(None),
        }
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UserModel;

    #[test]
    fn modality_router_crisis_voice_only() {
        let router = ModalityRouter::new(GenerationConfig::default());
        let result = router.select(
            "short text",
            &ContentCategory::Crisis,
            &IntentKind::Conversational,
            &UserModel::default(),
        );
        assert_eq!(result, OutputModality::VoiceOnly);
    }

    #[test]
    fn modality_router_table_both() {
        let router = ModalityRouter::new(GenerationConfig::default());
        let result = router.select(
            "col1 | col2 | col3",
            &ContentCategory::General,
            &IntentKind::Conversational,
            &UserModel::default(),
        );
        assert_eq!(result, OutputModality::Both);
    }

    #[test]
    fn modality_router_long_content_both() {
        let router = ModalityRouter::new(GenerationConfig::default());
        let long = "word ".repeat(100); // 500 chars > 400 threshold
        let result = router.select(
            &long,
            &ContentCategory::General,
            &IntentKind::Conversational,
            &UserModel::default(),
        );
        assert_eq!(result, OutputModality::Both);
    }

    #[test]
    fn modality_router_data_heavy_hud_only() {
        let router = ModalityRouter::new(GenerationConfig::default());
        let result = router.select(
            "short",
            &ContentCategory::DataHeavy,
            &IntentKind::Conversational,
            &UserModel::default(),
        );
        assert_eq!(result, OutputModality::HudOnly);
    }

    #[test]
    fn modality_router_default_voice_only() {
        let router = ModalityRouter::new(GenerationConfig::default());
        let result = router.select(
            "brief answer",
            &ContentCategory::General,
            &IntentKind::Conversational,
            &UserModel::default(),
        );
        assert_eq!(result, OutputModality::VoiceOnly);
    }

    #[test]
    fn tone_calibrator_regulatory_expands_contractions() {
        let cal = ToneCalibrator::new(GenerationConfig::default());
        let result = cal.calibrate("don't use this drug", &TonePreference::Regulatory);
        assert!(result.contains("do not"), "got: {result}");
    }

    #[test]
    fn tone_calibrator_crisis_truncates() {
        let cal = ToneCalibrator::new(GenerationConfig::default());
        // 20 words per sentence, threshold is 15.
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
        let result = cal.calibrate(long, &TonePreference::Crisis);
        let word_count = result.split_whitespace().count();
        assert!(word_count <= 15, "got {word_count} words");
    }

    #[test]
    fn summarize_for_voice_takes_first_two_sentences() {
        let router = GenerationRouter::new(GenerationConfig::default(), None, None);
        let text = "First sentence. Second sentence. Third sentence.";
        let summary = router.summarize_for_voice(text);
        assert!(summary.contains("First"), "got: {summary}");
        assert!(summary.contains("Second"), "got: {summary}");
        // Third should not appear (within 2-sentence limit).
        assert!(!summary.contains("Third"), "got: {summary}");
    }
}
