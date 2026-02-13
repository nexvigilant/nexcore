//! Memory Growth Detector (Hook 43)
//!
//! Detects unbounded memory growth patterns that lead to OOM under load.
//!
//! # Purpose
//!
//! Prevents production OOM crashes by detecting code patterns that accumulate
//! data without bounds. Critical for long-running services and the homeostasis
//! control loop (Guardian) where memory pressure degrades system performance.
//!
//! # Hook Event
//!
//! - **Event**: `PreToolUse:Write` (runs before file writes)
//! - **Matcher**: Rust files only (`.rs`), excludes test files
//! - **Tier**: `review` (quality enforcement)
//!
//! # Patterns Detected
//!
//! | Pattern | Severity | Issue |
//! |---------|----------|-------|
//! | Unbounded static collections | Critical | No size limit |
//! | `lazy_static! { HashMap }` | Critical | No eviction policy |
//! | `Arc<Mutex<Vec>>` shared | High | Unbounded shared collection |
//! | `.push()` in loop without `.truncate()` | High | Growing without limit |
//!
//! # Exit Codes
//!
//! - `0`: No memory growth issues detected
//! - `1`: High-severity warning (suggest adding `// BOUNDED:` comment)
//! - `2`: Critical issue blocks execution (must fix or use LruCache)
//!
//! # Safe Patterns
//!
//! Use bounded caches (moka, lru) or add `// BOUNDED: reason` comments
//! to document intentional growth limits.
//!
//! # Examples
//!
//! ```rust,ignore
//! // BAD: Unbounded static collection (will trigger CRITICAL)
//! lazy_static! {
//!     static ref CACHE: Mutex<HashMap<String, Data>> = Mutex::new(HashMap::new());
//! }
//!
//! // GOOD: Bounded cache with eviction
//! use moka::sync::Cache;
//! static CACHE: Lazy<Cache<String, Data>> = Lazy::new(|| {
//!     Cache::builder().max_capacity(10_000).build()
//! });
//!
//! // GOOD: Documented intentional limit
//! static ref ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
//! // BOUNDED: max 100 items, enforced by add_item() check
//! ```
//!
//! # Integration
//!
//! This hook integrates with the Guardian homeostasis loop by preventing
//! memory-related harm types (Type F: Saturation) at the code level before
//! they can manifest at runtime.

use nexcore_hooks::parser::performance::detect_memory_growth;
use nexcore_hooks::{
    exit_block, exit_success_auto, exit_warn, is_rust_file, is_test_file, read_input,
};

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

    if !is_rust_file(file_path) || is_test_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let sites = detect_memory_growth(content);
    if sites.is_empty() {
        exit_success_auto();
    }

    let critical: Vec<_> = sites.iter().filter(|s| s.severity == "critical").collect();
    let high: Vec<_> = sites.iter().filter(|s| s.severity == "high").collect();

    if !critical.is_empty() {
        let mut msg = format!("MEMORY GROWTH - {} critical issue(s)\n\n", critical.len());
        for s in &critical {
            msg.push_str(&format!(
                "Line {}: {}\n  Issue: {}\n\n",
                s.line, s.code, s.issue
            ));
        }
        msg.push_str("Static collections without size limits cause OOM.\n");
        msg.push_str("Use LruCache, moka, or implement eviction policy.");
        exit_block(&msg);
    }

    if !high.is_empty() {
        let mut msg = format!("MEMORY GROWTH - {} high-severity issue(s)\n\n", high.len());
        for s in &high {
            msg.push_str(&format!(
                "Line {}: {}\n  Issue: {}\n\n",
                s.line, s.code, s.issue
            ));
        }
        msg.push_str("Add // BOUNDED: comment if growth is intentionally limited.");
        exit_warn(&msg);
    }

    exit_success_auto();
}
