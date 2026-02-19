//! Jeopardy answer — the "What is X?" decode result.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A Jeopardy-style answer decoded from a balanced equation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JeopardyAnswer {
    /// The Jeopardy question (e.g. "What is Pharmacovigilance?").
    pub question: String,
    /// The concept name.
    pub concept: String,
    /// Match confidence (1.0 = exact match).
    pub confidence: f64,
    /// The formatted balanced equation.
    pub equation_display: String,
}

impl fmt::Display for JeopardyAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (confidence: {:.0}%)\n  {}",
            self.question,
            self.confidence * 100.0,
            self.equation_display
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jeopardy_answer_display() {
        let answer = JeopardyAnswer {
            question: "What is Pharmacovigilance?".to_string(),
            concept: "Pharmacovigilance".to_string(),
            confidence: 1.0,
            equation_display: "\"drug\"[...] + \"safety\"[...] -> \"PV\"[...]".to_string(),
        };
        let display = format!("{answer}");
        assert!(display.contains("What is Pharmacovigilance?"));
        assert!(display.contains("100%"));
    }

    #[test]
    fn test_jeopardy_answer_partial_confidence() {
        let answer = JeopardyAnswer {
            question: "What is Signal?".to_string(),
            concept: "Signal".to_string(),
            confidence: 0.85,
            equation_display: "eq".to_string(),
        };
        let display = format!("{answer}");
        assert!(display.contains("85%"));
    }
}
