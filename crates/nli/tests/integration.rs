//! Integration tests for the NLI pipeline.
//!
//! Uses mockall to create mock implementations of all trait boundaries.

use mockall::mock;
use mockall::predicate::*;
use nli::{
    NliError,
    acoustic::AsrProcessor,
    config::NliConfig,
    generation::{HudRenderer, TtsEngine},
    pragmatic::{ConversationContext, SessionStore},
    semantic::{DomainVocabulary, EntityExtractor, IntentClassifier},
    types::{
        AudioFrame, ClassifiedIntent, IntentKind, PipelineInput, SessionId, Slot, SlotValue,
        TonePreference, Transcript, TurnRole, UserModel,
    },
};

// --- Mock definitions ---

mock! {
    pub Asr {}
    #[async_trait::async_trait]
    impl AsrProcessor for Asr {
        async fn transcribe(&self, frame: &AudioFrame) -> Result<Transcript, NliError>;
    }
}

mock! {
    pub Classifier {}
    #[async_trait::async_trait]
    impl IntentClassifier for Classifier {
        async fn classify(&self, transcript: &Transcript) -> Result<ClassifiedIntent, NliError>;
    }
}

mock! {
    pub Extractor {}
    #[async_trait::async_trait]
    impl EntityExtractor for Extractor {
        async fn extract(
            &self,
            transcript: &Transcript,
            vocab: &DomainVocabulary,
        ) -> Result<Vec<Slot>, NliError>;
    }
}

mock! {
    pub Store {}
    #[async_trait::async_trait]
    impl SessionStore for Store {
        async fn load(&self, session_id: &SessionId) -> Result<Option<ConversationContext>, NliError>;
        async fn save(&self, session_id: &SessionId, context: &ConversationContext) -> Result<(), NliError>;
    }
}

mock! {
    pub Tts {}
    #[async_trait::async_trait]
    impl TtsEngine for Tts {
        async fn synthesize(&self, text: &str) -> Result<Vec<u8>, NliError>;
    }
}

mock! {
    pub Hud {}
    #[async_trait::async_trait]
    impl HudRenderer for Hud {
        async fn render(&self, text: &str) -> Result<Vec<u8>, NliError>;
    }
}

// --- Helpers ---

fn default_user_model() -> UserModel {
    UserModel {
        tone_preference: TonePreference::Casual,
        modality_preference: nli::types::OutputModality::VoiceOnly,
        expertise_level: 0.5,
    }
}

fn text_input(session: &str, text: &str) -> PipelineInput {
    PipelineInput {
        session_id: SessionId::new(session),
        audio: None,
        text_override: Some(text.to_string()),
        user_model: default_user_model(),
        module_context: None,
    }
}

fn make_pipeline(
    classifier: MockClassifier,
    extractor: MockExtractor,
    store: MockStore,
) -> nli::NliPipeline {
    let mut asr = MockAsr::new();
    asr.expect_transcribe().returning(|_| {
        Ok(Transcript {
            text: "mock".to_string(),
            confidence: 0.9,
            speech_detected: true,
        })
    });

    nli::NliPipeline::from_config(
        NliConfig::default(),
        Box::new(asr),
        Box::new(classifier),
        Box::new(extractor),
        DomainVocabulary::new(),
        Box::new(store),
        None,
        None,
    )
}

// --- Tests ---

#[tokio::test]
async fn test_text_only_conversational_intent() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Conversational, 0.8)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "hello there"))
        .await
        .unwrap();

    assert_eq!(output.intent.kind, IntentKind::Conversational);
    assert!(!output.response_text.is_empty());
}

#[tokio::test]
async fn test_crisis_intent_produces_voice_only_modality() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Crisis, 0.95)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "emergency overdose"))
        .await
        .unwrap();

    assert_eq!(output.intent.kind, IntentKind::Crisis);
    assert_eq!(output.modality, nli::types::OutputModality::VoiceOnly);
}

#[tokio::test]
async fn test_signal_detection_intent_classified() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::SignalDetection, 0.85)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "run signal detection"))
        .await
        .unwrap();

    assert_eq!(output.intent.kind, IntentKind::SignalDetection);
}

#[tokio::test]
async fn test_slots_propagated_to_output() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::DrugSafetyQuery, 0.8)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| {
        Ok(vec![Slot {
            name: "drug_name".to_string(),
            value: SlotValue::Drug("Semaglutide".to_string()),
            confidence: 0.95,
        }])
    });

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "safety of semaglutide"))
        .await
        .unwrap();

    assert_eq!(output.intent.slots.len(), 1);
    assert_eq!(output.intent.slots[0].name, "drug_name");
}

#[tokio::test]
async fn test_session_context_loaded_and_saved() {
    let session_id = SessionId::new("session-abc");

    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Navigation, 0.7)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().times(1).returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let input = text_input("session-abc", "help me navigate");
    pipeline.process(input).await.unwrap();
    // save was called exactly once — verified by mockall times(1)
}

#[tokio::test]
async fn test_existing_context_used() {
    let sid = SessionId::new("existing-session");
    let mut existing_ctx = ConversationContext::new(sid.clone(), 20);
    existing_ctx
        .remembered_entities
        .insert("drug_name".to_string(), "Metformin".to_string());

    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::DrugSafetyQuery, 0.8)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store
        .expect_load()
        .returning(move |_| Ok(Some(existing_ctx.clone())));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("existing-session", "what about it"))
        .await
        .unwrap();

    // Coreference "it" should have been resolved to "Metformin"
    assert!(
        output.resolved_coreferences.contains_key("it"),
        "expected coreference resolution for 'it'"
    );
}

#[tokio::test]
async fn test_no_audio_no_text_returns_error() {
    let mut clf = MockClassifier::new();
    let mut ext = MockExtractor::new();
    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));

    let mut pipeline = make_pipeline(clf, ext, store);
    let input = PipelineInput {
        session_id: SessionId::new("s1"),
        audio: None,
        text_override: None,
        user_model: default_user_model(),
        module_context: None,
    };

    let result = pipeline.process(input).await;
    assert!(result.is_err());
    matches!(result.unwrap_err(), NliError::AsrFailure(_));
}

#[tokio::test]
async fn test_module_context_applied() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::SignalDetection, 0.9)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let mut input = text_input("s1", "run signal detection");
    input.module_context = Some("signal-detection".to_string());

    let output = pipeline.process(input).await.unwrap();
    assert_eq!(output.intent.kind, IntentKind::SignalDetection);
}

#[tokio::test]
async fn test_regulatory_tone_expands_contractions() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Conversational, 0.7)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    // Override pipeline with regulatory tone user model.
    let mut asr = MockAsr::new();
    asr.expect_transcribe().returning(|_| {
        Ok(Transcript {
            text: "mock".to_string(),
            confidence: 0.9,
            speech_detected: true,
        })
    });

    let mut pipeline = nli::NliPipeline::from_config(
        NliConfig::default(),
        Box::new(asr),
        Box::new(clf),
        Box::new(ext),
        DomainVocabulary::new(),
        Box::new(store),
        None,
        None,
    );

    let input = PipelineInput {
        session_id: SessionId::new("s1"),
        audio: None,
        text_override: Some("I understand".to_string()),
        user_model: UserModel {
            tone_preference: TonePreference::Regulatory,
            modality_preference: nli::types::OutputModality::VoiceOnly,
            expertise_level: 0.8,
        },
        module_context: None,
    };

    let output = pipeline.process(input).await.unwrap();
    // Response should not contain contractions (regulatory tone).
    assert!(
        !output.response_text.contains("don't"),
        "got: {}",
        output.response_text
    );
}

#[tokio::test]
async fn test_voice_summary_present_for_voice_modality() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Navigation, 0.7)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline.process(text_input("s1", "help")).await.unwrap();

    // VoiceOnly modality should produce a voice summary.
    assert!(output.voice_summary.is_some());
}

#[tokio::test]
async fn test_proactive_surface_false_first_turn() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::DrugSafetyQuery, 0.8)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "tell me about semaglutide safety"))
        .await
        .unwrap();

    // First turn → T=0 → proactive surface = false
    assert!(!output.proactively_surfaced);
}

#[tokio::test]
async fn test_intent_unknown_for_unclear_input() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Unknown, 0.3)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline.process(text_input("s1", "xyzzy")).await.unwrap();

    assert_eq!(output.intent.kind, IntentKind::Unknown);
    assert!(!output.response_text.is_empty());
}

#[tokio::test]
async fn test_drug_safety_query_response_not_empty() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::DrugSafetyQuery, 0.85)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "adverse events for warfarin"))
        .await
        .unwrap();

    assert_eq!(output.intent.kind, IntentKind::DrugSafetyQuery);
    assert!(!output.response_text.is_empty());
}

#[tokio::test]
async fn test_causality_assessment_intent() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::CausalityAssessment, 0.85)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "naranjo assessment for this reaction"))
        .await
        .unwrap();

    assert_eq!(output.intent.kind, IntentKind::CausalityAssessment);
}

#[tokio::test]
async fn test_report_submission_intent() {
    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::ReportSubmission, 0.80)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store.expect_save().returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    let output = pipeline
        .process(text_input("s1", "submit icsr report"))
        .await
        .unwrap();

    assert_eq!(output.intent.kind, IntentKind::ReportSubmission);
}

#[tokio::test]
async fn test_session_id_preserved_in_context() {
    let sid = "unique-session-42";

    let mut clf = MockClassifier::new();
    clf.expect_classify()
        .returning(|_| Ok(ClassifiedIntent::new(IntentKind::Conversational, 0.6)));

    let mut ext = MockExtractor::new();
    ext.expect_extract().returning(|_, _| Ok(vec![]));

    let mut store = MockStore::new();
    store.expect_load().returning(|_| Ok(None));
    store
        .expect_save()
        .withf(move |id, _ctx| id.0 == sid)
        .times(1)
        .returning(|_, _| Ok(()));

    let mut pipeline = make_pipeline(clf, ext, store);
    pipeline.process(text_input(sid, "hello")).await.unwrap();
}
