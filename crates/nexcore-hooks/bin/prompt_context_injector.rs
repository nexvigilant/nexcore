//! Rust Context Injector
//!
//! Event: UserPromptSubmit
//! Injects Rust-specific context into prompts.

use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let prompt = match input.get_prompt() {
        Some(p) => p.to_lowercase(),
        None => exit_skip_prompt(),
    };

    // Check if prompt is about Rust
    let rust_keywords = [
        "rust", "cargo", "crate", "struct", "impl", "trait", "async", "tokio",
    ];
    let is_rust_related = rust_keywords.iter().any(|k| prompt.contains(k));

    if !is_rust_related {
        exit_skip_prompt();
    }

    // Inject Rust context
    let context = r#"
Rust Development Context:
- Use idiomatic Rust patterns (Result, Option, iterators)
- Prefer thiserror for error types, anyhow for applications
- Use clippy with -D warnings
- Follow ownership/borrowing best practices
"#;

    exit_with_context(context);
}
