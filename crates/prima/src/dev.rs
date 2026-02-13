// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Dev Mode — File Watcher for Prima Development
//!
//! Zero-dependency file watching using `std::fs::metadata().modified()` polling.
//!
//! ## Usage
//!
//! ```text
//! prima dev examples/hello.true          # Watch and re-run on change
//! prima dev examples/hello.true --ast    # Also show AST
//! prima dev examples/hello.true --check  # Also check .not.true sibling
//! prima develop examples/hello.true      # Alias for dev
//! ```
//!
//! ## Tier: T2-C (σ + ν + ς)

use crate::error::{PrimaError, PrimaResult};
use crate::nottrue;
use crate::value::Value;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Configuration for dev watch mode.
#[derive(Debug, Clone)]
pub struct DevConfig {
    /// Path to the `.true` file to watch.
    pub file: PathBuf,
    /// Show AST after parsing.
    pub show_ast: bool,
    /// Show tokens after lexing.
    pub show_tokens: bool,
    /// Show grounding trace.
    pub show_trace: bool,
    /// Run `.not.true` sibling check.
    pub run_check: bool,
    /// Poll interval for file changes.
    pub poll_interval: Duration,
}

impl DevConfig {
    /// Create a new DevConfig with defaults.
    #[must_use]
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            show_ast: false,
            show_tokens: false,
            show_trace: false,
            run_check: false,
            poll_interval: Duration::from_millis(300),
        }
    }
}

/// Result of a single dev compilation cycle.
#[derive(Debug, Clone)]
pub struct DevResult {
    /// Program output (Display of the final Value).
    pub output: String,
    /// Number of tokens lexed.
    pub tokens: usize,
    /// Number of statements parsed.
    pub statements: usize,
    /// Time taken for compilation + execution.
    pub elapsed: Duration,
    /// Tier classification code.
    pub tier: String,
    /// Primitive composition string.
    pub composition: String,
}

/// Run a single compilation cycle (no watch loop).
///
/// Returns `DevResult` on success, or formats the error for display.
pub fn run_once(config: &DevConfig) -> Result<DevResult, String> {
    let source = std::fs::read_to_string(&config.file).map_err(|e| format!("∂[io]: {}", e))?;

    let start = Instant::now();

    // Tokenize
    let tokens = crate::tokenize(&source).map_err(|e| format!("∂[lex]: {}", e))?;
    let token_count = tokens.len();

    // Parse
    let program = crate::parser::Parser::new(tokens)
        .parse()
        .map_err(|e| format!("∂[parse]: {}", e))?;
    let stmt_count = program.statements.len();

    // Compile + run
    let module = crate::codegen::Compiler::new()
        .compile(&program)
        .map_err(|e| format!("∂[compile]: {}", e))?;
    let result: Value = crate::vm::VM::new()
        .run(&module)
        .map_err(|e| format!("∂[runtime]: {}", e))?;

    let elapsed = start.elapsed();

    Ok(DevResult {
        output: format!("{}", result),
        tokens: token_count,
        statements: stmt_count,
        elapsed,
        tier: result.tier().code().to_string(),
        composition: format!("{}", result.composition),
    })
}

/// Format the display header.
fn format_header(config: &DevConfig, result: &Result<DevResult, String>) -> String {
    let file_display = config.file.display();
    let mut out = String::new();

    out.push_str("\x1b[2J\x1b[1;1H"); // clear screen + home
    out.push_str("═══ Prima Dev ═══════════════════════════════════\n");

    match result {
        Ok(r) => {
            out.push_str(&format!(
                " File: {}\n Time: {:.1}ms | Tokens: {} | Stmts: {}\n",
                file_display,
                r.elapsed.as_secs_f64() * 1000.0,
                r.tokens,
                r.statements,
            ));
            out.push_str("═════════════════════════════════════════════════\n\n");
            out.push_str(&format!("{} : {} ({})\n", r.output, r.tier, r.composition));
        }
        Err(e) => {
            out.push_str(&format!(" File: {}\n", file_display));
            out.push_str("═════════════════════════════════════════════════\n\n");
            out.push_str(e);
            out.push('\n');
        }
    }

    out
}

/// Format optional AST display.
fn format_ast(config: &DevConfig) -> String {
    if !config.show_ast {
        return String::new();
    }
    let source = match std::fs::read_to_string(&config.file) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let program = match crate::parse(&source) {
        Ok(p) => p,
        Err(_) => return String::new(),
    };

    let mut out = String::from("\n─── AST ───\n");
    for (i, s) in program.statements.iter().enumerate() {
        out.push_str(&format!("  [{}] {:?}\n", i, s));
    }
    out
}

/// Format optional token display.
fn format_tokens(config: &DevConfig) -> String {
    if !config.show_tokens {
        return String::new();
    }
    let source = match std::fs::read_to_string(&config.file) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let tokens = match crate::tokenize(&source) {
        Ok(t) => t,
        Err(_) => return String::new(),
    };

    let mut out = String::from("\n─── Tokens ───\n");
    for t in &tokens {
        out.push_str(&format!("  {} ({})\n", t, t.dominant_primitive().symbol()));
    }
    out
}

/// Format optional trace display.
fn format_trace(result: &Result<DevResult, String>) -> String {
    match result {
        Ok(r) => {
            let mut out = String::from("\n─── Trace ───\n");
            out.push_str(&format!("  Tier: {}\n", r.tier));
            out.push_str(&format!("  Composition: {}\n", r.composition));
            out.push_str("  Grounds to: {0, 1} ∎\n");
            out
        }
        Err(_) => String::new(),
    }
}

/// Format optional .not.true check.
fn format_check(config: &DevConfig) -> String {
    if !config.run_check {
        return String::new();
    }

    let not_true_path = sibling_not_true(&config.file);
    let source = match std::fs::read_to_string(&not_true_path) {
        Ok(s) => s,
        Err(_) => {
            return format!(
                "\n─── Check ───\n  No sibling: {}\n",
                not_true_path.display()
            );
        }
    };

    let name = not_true_path.display().to_string();
    let report = nottrue::check(&source, &name);
    let mut out = String::from("\n─── Check (.not.true) ───\n");
    out.push_str(&format!("{}", report));
    if report.all_passed() {
        out.push_str("  All antipatterns hold.\n");
    }
    out
}

/// Derive the `.not.true` sibling path from a `.true` path.
///
/// `examples/hello.true` → `examples/hello.not.true`
fn sibling_not_true(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(
        "{}.{}",
        stem.to_string_lossy(),
        nottrue::FILE_EXTENSION_NOT
    ))
}

/// Get the last-modified time for a file, returning epoch on error.
fn get_mtime(path: &Path) -> SystemTime {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Run the watch loop. Blocks until SIGINT (Ctrl+C).
///
/// Polls `config.file` for mtime changes every `config.poll_interval`.
pub fn watch(config: &DevConfig) -> PrimaResult<()> {
    if !config.file.exists() {
        return Err(PrimaError::runtime(format!(
            "file not found: {}",
            config.file.display()
        )));
    }

    let mut last_mtime = SystemTime::UNIX_EPOCH;
    let mut last_run = Instant::now() - config.poll_interval; // allow immediate first run

    loop {
        let current_mtime = get_mtime(&config.file);

        if current_mtime != last_mtime && last_run.elapsed() >= config.poll_interval {
            last_mtime = current_mtime;
            last_run = Instant::now();

            let result = run_once(config);

            // Build full display
            let mut display = format_header(config, &result);

            if config.show_ast {
                display.push_str(&format_ast(config));
            }
            if config.show_tokens {
                display.push_str(&format_tokens(config));
            }
            if config.show_trace {
                display.push_str(&format_trace(&result));
            }
            if config.run_check {
                display.push_str(&format_check(config));
            }

            display.push_str("\n─── Watching... (Ctrl+C to exit) ───\n");

            print!("{}", display);
        }

        std::thread::sleep(config.poll_interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_dev_config_default() {
        let config = DevConfig::new(PathBuf::from("test.true"));
        assert!(!config.show_ast);
        assert!(!config.show_tokens);
        assert!(!config.show_trace);
        assert!(!config.run_check);
        assert_eq!(config.poll_interval, Duration::from_millis(300));
        assert_eq!(config.file, PathBuf::from("test.true"));
    }

    #[test]
    fn test_run_once_simple() {
        // Create a temp file with valid Prima source
        let dir = std::env::temp_dir().join("prima_dev_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test_run_once.true");
        {
            let mut f = std::fs::File::create(&file)
                .ok()
                .unwrap_or_else(|| panic!("failed to create temp file"));
            let _ = f.write_all(b"42");
        }

        let config = DevConfig::new(file.clone());
        let result = run_once(&config);
        assert!(result.is_ok(), "run_once failed: {:?}", result);

        let r = result.ok().unwrap_or_else(|| panic!("unreachable"));
        assert_eq!(r.output, "42");
        assert!(r.tokens > 0);
        assert!(r.statements > 0);
        assert_eq!(r.tier, "T1");

        // Cleanup
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_once_with_error() {
        let dir = std::env::temp_dir().join("prima_dev_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test_run_once_error.true");
        {
            let mut f = std::fs::File::create(&file)
                .ok()
                .unwrap_or_else(|| panic!("failed to create temp file"));
            // Invalid source — parse error
            let _ = f.write_all(b"let = = =");
        }

        let config = DevConfig::new(file.clone());
        let result = run_once(&config);
        // Should be Err with a readable message
        assert!(result.is_err());
        let err = result.err().unwrap_or_default();
        assert!(!err.is_empty());

        // Cleanup
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_once_with_ast() {
        let dir = std::env::temp_dir().join("prima_dev_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test_run_once_ast.true");
        {
            let mut f = std::fs::File::create(&file)
                .ok()
                .unwrap_or_else(|| panic!("failed to create temp file"));
            let _ = f.write_all(b"1 + 2");
        }

        let mut config = DevConfig::new(file.clone());
        config.show_ast = true;
        let result = run_once(&config);
        assert!(result.is_ok());

        // format_ast should produce output
        let ast_out = format_ast(&config);
        assert!(ast_out.contains("AST"), "AST section missing: {}", ast_out);
        assert!(ast_out.contains("[0]"), "No statement index: {}", ast_out);

        // Cleanup
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_once_with_tokens() {
        let dir = std::env::temp_dir().join("prima_dev_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test_run_once_tokens.true");
        {
            let mut f = std::fs::File::create(&file)
                .ok()
                .unwrap_or_else(|| panic!("failed to create temp file"));
            let _ = f.write_all(b"42");
        }

        let mut config = DevConfig::new(file.clone());
        config.show_tokens = true;

        let tokens_out = format_tokens(&config);
        assert!(tokens_out.contains("Tokens"), "Token section missing");

        // Cleanup
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_sibling_not_true() {
        let path = PathBuf::from("examples/hello.true");
        let sibling = sibling_not_true(&path);
        assert_eq!(sibling, PathBuf::from("examples/hello.not.true"));
    }

    #[test]
    fn test_sibling_not_true_nested() {
        let path = PathBuf::from("/home/user/code/main.true");
        let sibling = sibling_not_true(&path);
        assert_eq!(sibling, PathBuf::from("/home/user/code/main.not.true"));
    }

    #[test]
    fn test_format_header_success() {
        let config = DevConfig::new(PathBuf::from("test.true"));
        let result = Ok(DevResult {
            output: "42".to_string(),
            tokens: 1,
            statements: 1,
            elapsed: Duration::from_micros(500),
            tier: "T1".to_string(),
            composition: "N".to_string(),
        });
        let header = format_header(&config, &result);
        assert!(header.contains("Prima Dev"));
        assert!(header.contains("test.true"));
        assert!(header.contains("42 : T1 (N)"));
    }

    #[test]
    fn test_format_header_error() {
        let config = DevConfig::new(PathBuf::from("bad.true"));
        let result: Result<DevResult, String> = Err("∂[parse]: unexpected token".to_string());
        let header = format_header(&config, &result);
        assert!(header.contains("Prima Dev"));
        assert!(header.contains("∂[parse]"));
    }

    #[test]
    fn test_watch_missing_file() {
        let config = DevConfig::new(PathBuf::from("/nonexistent/path/never.true"));
        let result = watch(&config);
        assert!(result.is_err());
    }
}
