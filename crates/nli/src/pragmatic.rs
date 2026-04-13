//! Layer 3: Conversation context management and session store.

use crate::{
    config::PragmaticConfig,
    error::NliError,
    types::{ClassifiedIntent, IntentKind, SessionId, Turn, TurnRole, UserModel},
};
use std::collections::HashMap;

/// Session store trait — read/write conversation state.
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    /// Load a conversation context for a session. Returns `None` if not found.
    async fn load(&self, session_id: &SessionId) -> Result<Option<ConversationContext>, NliError>;

    /// Persist a conversation context.
    async fn save(
        &self,
        session_id: &SessionId,
        context: &ConversationContext,
    ) -> Result<(), NliError>;
}

/// In-memory conversation context for a single session.
#[derive(Debug, Clone)]
pub struct ConversationContext {
    /// Session identifier.
    pub session_id: SessionId,
    /// Ordered history of turns (user + assistant).
    pub turn_history: Vec<Turn>,
    /// Maximum turns to retain.
    pub max_turns: usize,
    /// Active module context key (e.g. "signal-detection").
    pub active_module: Option<String>,
    /// Named entities remembered across turns (entity_type → value).
    pub remembered_entities: HashMap<String, String>,
}

impl ConversationContext {
    /// Create a new empty context for a session.
    pub fn new(session_id: SessionId, max_turns: usize) -> Self {
        Self {
            session_id,
            turn_history: Vec::new(),
            max_turns,
            active_module: None,
            remembered_entities: HashMap::new(),
        }
    }

    /// Append a turn to the history, trimming to `max_turns`.
    pub fn push_turn(&mut self, turn: Turn) {
        self.turn_history.push(turn);
        if self.turn_history.len() > self.max_turns {
            let excess = self.turn_history.len() - self.max_turns;
            self.turn_history.drain(..excess);
        }
    }

    /// Resolve coreferences in `text` by substituting pronouns with the
    /// most recently mentioned entity of the matching type.
    ///
    /// Scans for pronouns "it", "this", "that", "the drug", "the event"
    /// and replaces them with the last drug or MedDRA term from turn history.
    pub fn resolve_coreferences(&self, text: &str) -> (String, HashMap<String, String>) {
        let mut resolved = text.to_string();
        let mut applied: HashMap<String, String> = HashMap::new();

        let last_drug = self.last_entity("drug_name");
        let last_event = self.last_entity("event_term");

        // Simple token-level replacement for common referring expressions.
        let replacements: &[(&str, Option<&str>)] = &[
            ("the drug", last_drug.as_deref()),
            ("the event", last_event.as_deref()),
            ("the reaction", last_event.as_deref()),
            ("it", last_drug.as_deref().or(last_event.as_deref())),
            ("this", last_drug.as_deref().or(last_event.as_deref())),
            ("that", last_drug.as_deref().or(last_event.as_deref())),
        ];

        for (pronoun, replacement_opt) in replacements {
            if let Some(replacement) = replacement_opt {
                // Case-insensitive whole-token replacement.
                let lower = resolved.to_lowercase();
                if lower.contains(pronoun) {
                    resolved = replace_ignore_case(&resolved, pronoun, replacement);
                    applied.insert(pronoun.to_string(), replacement.to_string());
                }
            }
        }

        (resolved, applied)
    }

    /// Find the most recently mentioned entity of the given type from turn history.
    fn last_entity(&self, entity_type: &str) -> Option<String> {
        for turn in self.turn_history.iter().rev() {
            if turn.role != TurnRole::User {
                continue;
            }
            if let Some(intent) = &turn.intent {
                for slot in &intent.slots {
                    if slot.name == entity_type {
                        let val = match &slot.value {
                            crate::types::SlotValue::Text(s) => s.clone(),
                            crate::types::SlotValue::Drug(s) => s.clone(),
                            crate::types::SlotValue::MedDra(s) => s.clone(),
                            crate::types::SlotValue::Number(n) => n.to_string(),
                            crate::types::SlotValue::Bool(b) => b.to_string(),
                        };
                        return Some(val);
                    }
                }
            }
        }
        // Also check remembered_entities as a fallback.
        self.remembered_entities.get(entity_type).cloned()
    }

    /// Determine whether a proactive signal surface is warranted.
    ///
    /// Implements S = U × R × T (Campion Signal Theory):
    /// - U (urgency): intent is Crisis or SignalDetection
    /// - R (relevance): an active module context is set
    /// - T (timing): at least one prior user turn exists
    ///
    /// Returns true when U × R × T ≥ config threshold.
    pub fn should_proactively_surface(
        &self,
        intent: &ClassifiedIntent,
        config: &PragmaticConfig,
    ) -> bool {
        let u: f64 = match intent.kind {
            IntentKind::Crisis => 1.0,
            IntentKind::SignalDetection | IntentKind::DrugSafetyQuery => 0.8,
            IntentKind::CausalityAssessment => 0.7,
            _ => 0.2,
        };

        let r: f64 = if self.active_module.is_some() {
            1.0
        } else {
            0.3
        };

        let prior_turns = self
            .turn_history
            .iter()
            .filter(|t| t.role == TurnRole::User)
            .count();
        let t: f64 = if prior_turns > 0 { 1.0 } else { 0.0 };

        let signal = u * r * t;
        signal >= config.proactive_signal_threshold
    }

    /// Apply module context — updates the active module key.
    pub fn apply_module_context(&mut self, module_key: Option<String>) {
        self.active_module = module_key;
    }
}

/// In-memory session store (for testing and single-process deployments).
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: std::sync::Mutex<HashMap<String, ConversationContext>>,
}

impl InMemorySessionStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl SessionStore for InMemorySessionStore {
    async fn load(&self, session_id: &SessionId) -> Result<Option<ConversationContext>, NliError> {
        let guard = self
            .sessions
            .lock()
            .map_err(|e| NliError::SessionStoreError(e.to_string()))?;
        Ok(guard.get(&session_id.0).cloned())
    }

    async fn save(
        &self,
        session_id: &SessionId,
        context: &ConversationContext,
    ) -> Result<(), NliError> {
        let mut guard = self
            .sessions
            .lock()
            .map_err(|e| NliError::SessionStoreError(e.to_string()))?;
        guard.insert(session_id.0.clone(), context.clone());
        Ok(())
    }
}

/// Replace all occurrences of `needle` in `haystack` case-insensitively.
fn replace_ignore_case(haystack: &str, needle: &str, replacement: &str) -> String {
    let lower_haystack = haystack.to_lowercase();
    let lower_needle = needle.to_lowercase();

    let mut result = String::with_capacity(haystack.len());
    let mut last_end = 0usize;
    let mut search_start = 0usize;

    while let Some(pos) = lower_haystack[search_start..].find(&lower_needle) {
        let abs_pos = search_start + pos;
        result.push_str(&haystack[last_end..abs_pos]);
        result.push_str(replacement);
        last_end = abs_pos + needle.len();
        search_start = last_end;
    }
    result.push_str(&haystack[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClassifiedIntent, IntentKind, Slot, SlotValue, Turn, TurnRole};

    fn make_drug_turn(drug: &str) -> Turn {
        Turn {
            role: TurnRole::User,
            text: format!("tell me about {drug}"),
            intent: Some(ClassifiedIntent {
                kind: IntentKind::DrugSafetyQuery,
                confidence: 0.9,
                slots: vec![Slot {
                    name: "drug_name".to_string(),
                    value: SlotValue::Drug(drug.to_string()),
                    confidence: 0.9,
                }],
            }),
            timestamp_ms: 0,
        }
    }

    #[test]
    fn push_turn_trims_to_max() {
        let mut ctx = ConversationContext::new(SessionId::new("s1"), 2);
        for _ in 0..5 {
            ctx.push_turn(make_drug_turn("semaglutide"));
        }
        assert_eq!(ctx.turn_history.len(), 2);
    }

    #[test]
    fn coreference_resolves_drug() {
        let mut ctx = ConversationContext::new(SessionId::new("s1"), 10);
        ctx.push_turn(make_drug_turn("Semaglutide"));

        let (resolved, applied) = ctx.resolve_coreferences("What are the risks of it?");
        assert!(resolved.contains("Semaglutide"), "resolved: {resolved}");
        assert!(applied.contains_key("it"));
    }

    #[test]
    fn proactive_surface_crisis_with_module() {
        let mut ctx = ConversationContext::new(SessionId::new("s1"), 10);
        ctx.push_turn(make_drug_turn("drug"));
        ctx.active_module = Some("signal-detection".to_string());

        let intent = ClassifiedIntent::new(IntentKind::Crisis, 0.95);
        let config = PragmaticConfig::default();
        assert!(ctx.should_proactively_surface(&intent, &config));
    }

    #[test]
    fn proactive_surface_no_prior_turns_false() {
        let ctx = ConversationContext::new(SessionId::new("s1"), 10);
        let intent = ClassifiedIntent::new(IntentKind::Crisis, 0.95);
        let config = PragmaticConfig::default();
        // T=0 because no prior turns → signal = 0
        assert!(!ctx.should_proactively_surface(&intent, &config));
    }
}
