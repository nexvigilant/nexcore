//! # Grammar Controller (σ + ∂)
//!
//! Enforces the {word/primitive} pattern across all tool outputs.

use nexcore_vigilance::lex_primitiva::LexPrimitiva;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Lexicon mapping domain terms to T1 primitive symbols.
pub struct PrimitiveLexicon {
    lexicon: HashMap<String, String>,
}

static CONTROLLER: OnceLock<PrimitiveLexicon> = OnceLock::new();

impl PrimitiveLexicon {
    /// Get or initialize the global controller.
    pub fn get() -> &'static Self {
        CONTROLLER.get_or_init(Self::init)
    }

    fn init() -> Self {
        let mut lexicon = HashMap::new();

        // --- Core Primitives ---
        for p in LexPrimitiva::all() {
            lexicon.insert(p.name().to_lowercase(), p.symbol().to_string());
        }

        // --- Domain Terms (Mapping to Primitives) ---
        let mappings = vec![
            ("signal", "ν"),
            ("node", "∃"),
            ("network", "μ"),
            ("impact", "∝"),
            ("probability", "N"),
            ("isolation", "ς"),
            ("boundary", "∂"),
            ("detected", "κ"),
            ("scan", "σ"),
            ("sequence", "σ"),
            ("trust", "μ"),
            ("state", "ς"),
            ("success", "N"),
            ("failure", "∅"),
            ("error", "∂"),
            ("latency", "σ"),
            ("target", "λ"),
            ("found", "∃"),
            ("match", "κ"),
            ("mars", "ρ"),
            ("alignment", "σ"),
            ("opposition", "κ"),
            ("recursion", "ρ"),
            ("cycle", "σ"),
        ];

        for (word, symbol) in mappings {
            lexicon.insert(word.to_string(), symbol.to_string());
        }

        Self { lexicon }
    }

    /// Apply the grammar pattern to a string.
    pub fn apply(&self, input: &str) -> String {
        let mut output = input.to_string();

        // This is a naive implementation: in a production version,
        // we'd use a regex or a proper lexer to avoid partial matches.
        // For the prototype, we'll iterate through the lexicon.
        for (word, symbol) in &self.lexicon {
            let pattern = format!("{{{}/{}}}", word, symbol);
            // Replace standalone words (case insensitive-ish)
            let find_word = format!(" {} ", word);
            let replace_word = format!(" {} ", pattern);
            output = output.replace(&find_word, &replace_word);

            // Check start/end of string
            if output.to_lowercase().starts_with(word) {
                output = format!("{}{}", pattern, &output[word.len()..]);
            }
        }

        output
    }
}
