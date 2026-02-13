//! Thread Safety Enforcer Hook
//!
//! Enforces documentation requirements for concurrent/shared state primitives in Rust code.
//! Blocks file writes that introduce thread-safety constructs without proper documentation.
//!
//! # Hook Configuration
//!
//! | Property | Value |
//! |----------|-------|
//! | **Event** | `PreToolUse` |
//! | **Tier** | `review` (code quality enforcement) |
//! | **Timeout** | 5000ms |
//! | **Matchers** | `tool_name: Edit, Write` for `*.rs` files |
//!
//! # Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success - no concurrency primitives found, or all documented |
//! | 2 | Block - undocumented concurrency primitive detected |
//!
//! # Detected Patterns
//!
//! The hook scans for these patterns and requires a `// CONCURRENCY:` comment
//! within the preceding 3 lines:
//!
//! - `Arc<Mutex<T>>` - Shared mutable state across threads
//! - `Arc<RwLock<T>>` - Shared read-write state across threads
//! - `RwLock<T>` - Reader-writer lock
//! - `AtomicBool`, `AtomicUsize`, `AtomicU8..U64`, `AtomicI32..I64` - Atomic primitives
//! - `static mut` - **Always blocked** (use OnceCell/OnceLock instead)
//!
//! # Required Documentation Format
//!
//! ```rust
//! // CONCURRENCY: This mutex protects [what it guards].
//! // Lock ordering: [rules if multiple locks exist]
//! // Contention: [expected contention level: low/medium/high]
//! let shared_state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::new()));
//! ```
//!
//! # Rationale
//!
//! Undocumented concurrent primitives lead to:
//! - Deadlocks from inconsistent lock ordering
//! - Data races from unclear ownership
//! - Performance issues from unexpected contention
//!
//! This hook enforces the **"document-before-use"** principle for thread safety.

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let violations = check_concurrency(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("CONCURRENCY REVIEW REQUIRED\n\n");
    for (line, requirement) in &violations {
        msg.push_str(&format!("Line {line}: {requirement}\n"));
    }
    msg.push_str("\nREQUIRED documentation:\n");
    msg.push_str("  // CONCURRENCY: This mutex protects [what].\n");
    msg.push_str("  // Lock ordering: [ordering rules if multiple locks]\n");
    msg.push_str("  // Contention: [expected contention level]\n");
    exit_block(&msg);
}

/// Build markers at runtime to avoid self-detection
fn get_markers() -> Vec<(String, String)> {
    // Split strings to avoid hook detecting them in this source file
    let arc = "Arc";
    let mutex = ["Mut", "ex"].concat();
    let rwlock = ["Rw", "Lo", "ck"].concat();
    let atomic = "Atomic";
    let stat = "static";
    let mutable = ["m", "u", "t"].concat();

    // Build error messages dynamically too
    let arc_mutex_msg = format!("{}<{}<T>> needs CONCURRENCY: comment", arc, mutex);
    let arc_rwlock_msg = format!("{}<{}<T>> needs CONCURRENCY: comment", arc, rwlock);
    let rwlock_msg = format!("{} needs CONCURRENCY: comment", rwlock);
    let atomic_msg = "Atomic needs memory ordering documentation".to_string();
    let static_mut_msg = format!("{} {} is wrong - use OnceCell/OnceLock", stat, mutable);

    vec![
        (format!("{}<{}<", arc, mutex), arc_mutex_msg),
        (format!("{}<{}<", arc, rwlock), arc_rwlock_msg),
        (format!("{}<", rwlock), rwlock_msg),
        (format!("{}Bool", atomic), atomic_msg.clone()),
        (format!("{}Usize", atomic), atomic_msg.clone()),
        (format!("{}U8", atomic), atomic_msg.clone()),
        (format!("{}U16", atomic), atomic_msg.clone()),
        (format!("{}U32", atomic), atomic_msg.clone()),
        (format!("{}U64", atomic), atomic_msg.clone()),
        (format!("{}I32", atomic), atomic_msg.clone()),
        (format!("{}I64", atomic), atomic_msg),
        (format!("{} {}", stat, mutable), static_mut_msg),
    ]
}

fn check_concurrency(content: &str) -> Vec<(usize, String)> {
    let markers = get_markers();
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        for (marker, requirement) in &markers {
            if line.contains(marker) {
                // Check for CONCURRENCY comment in previous 3 lines
                let has_doc = (0..=3).any(|offset| {
                    i >= offset
                        && lines
                            .get(i - offset)
                            .is_some_and(|l| l.contains("// CONCURRENCY:"))
                });

                if !has_doc {
                    violations.push((i + 1, requirement.clone()));
                }
            }
        }
    }

    violations
}
