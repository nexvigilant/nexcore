//! Performance pattern detection for Rust code.

use regex::Regex;

/// An allocation site detected in the code
#[derive(Debug, Clone)]
pub struct AllocationSite {
    /// Line number (1-indexed)
    pub line: usize,
    /// The allocation pattern matched
    pub pattern: &'static str,
    /// The actual code snippet
    pub code: String,
    /// Advisory note
    pub note: &'static str,
    /// Whether inside a loop
    pub in_loop: bool,
    /// Whether in a hot path
    pub in_hot_path: bool,
}

/// A clone site detected in the code
#[derive(Debug, Clone)]
pub struct CloneSite {
    /// Line number
    pub line: usize,
    /// Code containing the clone
    pub code: String,
    /// Classification
    pub classification: CloneClassification,
    /// Suggested fix
    pub suggestion: &'static str,
}

/// Classification of clone necessity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneClassification {
    /// Unnecessary clone
    Unnecessary,
    /// Clone inside loop
    InLoop,
    /// Borrow checker workaround
    BorrowWorkaround,
    /// Large type clone
    LargeType,
    /// Necessary clone
    Necessary,
}

/// Memory growth pattern detected
#[derive(Debug, Clone)]
pub struct MemoryGrowthSite {
    /// Line number
    pub line: usize,
    /// Pattern matched
    pub pattern: &'static str,
    /// The code
    pub code: String,
    /// Is static
    pub is_static: bool,
    /// Issue description
    pub issue: &'static str,
    /// Severity
    pub severity: &'static str,
}

/// Async anti-pattern detected
#[derive(Debug, Clone)]
pub struct AsyncIssue {
    /// Line number
    pub line: usize,
    /// Pattern
    pub pattern: String,
    /// Code
    pub code: String,
    /// Issue
    pub issue: &'static str,
    /// Severity
    pub severity: &'static str,
    /// Fix
    pub fix: &'static str,
}

/// Complexity annotation
#[derive(Debug, Clone)]
pub struct ComplexityAnnotation {
    /// Function name
    pub function: String,
    /// Line number
    pub line: usize,
    /// Documented complexity
    pub documented: Option<String>,
    /// Inferred complexity
    pub inferred: String,
    /// Confidence
    pub confidence: &'static str,
    /// Matches documented
    pub matches: bool,
}

const ALLOC_PATTERNS: &[(&str, &str, &str)] = &[
    (
        "Vec::new()",
        r"Vec::new\s*\(\s*\)",
        "Consider Vec::with_capacity()",
    ),
    ("String::from()", r"String::from\s*\(", "Heap allocation"),
    (
        ".to_string()",
        r"\.to_string\s*\(\s*\)",
        "Creates new String",
    ),
    ("format!()", r"format!\s*\(", "Allocates String"),
    (".collect()", r"\.collect\s*\(\s*\)", "Allocates collection"),
];

const HOT_MARKERS: &[&str] = &["#[inline]", "// HOT PATH", "// PERF:"];

fn compile_regex(pattern: &str) -> Option<Regex> {
    Regex::new(pattern).ok()
}

fn find_loop_lines(content: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut depth: usize = 0;
    let mut in_loop = false;
    let mut loop_depth: usize = 0;
    let re = compile_regex(r"^\s*(for|while|loop)\s");
    for (i, l) in content.lines().enumerate() {
        if let Some(ref r) = re {
            if r.is_match(l) {
                in_loop = true;
                loop_depth = depth;
            }
        }
        for c in l.chars() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if in_loop && depth <= loop_depth {
                        in_loop = false;
                    }
                }
                _ => {}
            }
        }
        if in_loop {
            lines.push(i);
        }
    }
    lines
}

fn find_hot_lines(content: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut in_hot = false;
    let mut depth: usize = 0;
    let mut hot_depth: usize = 0;
    for (i, l) in content.lines().enumerate() {
        for m in HOT_MARKERS {
            if l.contains(m) {
                in_hot = true;
                hot_depth = depth;
                break;
            }
        }
        for c in l.chars() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if in_hot && depth <= hot_depth {
                        in_hot = false;
                    }
                }
                _ => {}
            }
        }
        if in_hot {
            lines.push(i);
        }
    }
    lines
}

fn is_justified(lines: &[&str], idx: usize, markers: &[&str]) -> bool {
    let start = idx.saturating_sub(3);
    for i in start..=idx {
        if let Some(l) = lines.get(i) {
            for m in markers {
                if l.contains(m) {
                    return true;
                }
            }
        }
    }
    false
}

/// Detect allocations in code.
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Allocation sites found
pub fn detect_allocations(content: &str) -> Vec<AllocationSite> {
    let mut sites = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let loops = find_loop_lines(content);
    let hot = find_hot_lines(content);
    for (name, pat, note) in ALLOC_PATTERNS {
        if let Some(re) = compile_regex(pat) {
            for (i, l) in lines.iter().enumerate() {
                if re.is_match(l) && !is_justified(&lines, i, &["// ALLOC:", "// PERF:"]) {
                    sites.push(AllocationSite {
                        line: i + 1,
                        pattern: name,
                        code: l.trim().to_string(),
                        note,
                        in_loop: loops.contains(&i),
                        in_hot_path: hot.contains(&i),
                    });
                }
            }
        }
    }
    sites
}

/// Detect clones in code.
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Clone sites found
pub fn detect_clones(content: &str) -> Vec<CloneSite> {
    let mut sites = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let loops = find_loop_lines(content);
    let pat = format!(".cl{}()", "one");
    for (i, l) in lines.iter().enumerate() {
        if l.contains(&pat) && !is_justified(&lines, i, &["// CLONE:"]) {
            let class = if loops.contains(&i) {
                CloneClassification::InLoop
            } else if l.contains("self.") {
                CloneClassification::BorrowWorkaround
            } else {
                CloneClassification::Necessary
            };
            sites.push(CloneSite {
                line: i + 1,
                code: l.trim().to_string(),
                classification: class,
                suggestion: match class {
                    CloneClassification::InLoop => "Hoist outside loop",
                    _ => "Consider alternatives",
                },
            });
        }
    }
    sites
}

/// Detect memory growth.
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Growth sites found
pub fn detect_memory_growth(content: &str) -> Vec<MemoryGrowthSite> {
    let mut sites = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let loops = find_loop_lines(content);
    let push = format!(".pu{}(", "sh");
    for (i, l) in lines.iter().enumerate() {
        if loops.contains(&i) && l.contains(&push) && !is_justified(&lines, i, &["// BOUNDED:"]) {
            sites.push(MemoryGrowthSite {
                line: i + 1,
                pattern: "unbounded push",
                code: l.trim().to_string(),
                is_static: false,
                issue: "Collection grows in loop without bound",
                severity: "high",
            });
        }
    }
    sites
}

/// Detect async issues.
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Async issues found
pub fn detect_async_issues(content: &str) -> Vec<AsyncIssue> {
    let mut issues = Vec::new();
    let re = compile_regex(r"async\s+fn\s+\w+");
    let mut in_async = false;
    let mut depth: usize = 0;
    let mut async_depth: usize = 0;
    for (i, l) in content.lines().enumerate() {
        if let Some(ref r) = re {
            if r.is_match(l) {
                in_async = true;
                async_depth = depth;
            }
        }
        for c in l.chars() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if in_async && depth <= async_depth {
                        in_async = false;
                    }
                }
                _ => {}
            }
        }
        if in_async && l.contains("std::thread::sleep") {
            issues.push(AsyncIssue {
                line: i + 1,
                pattern: "blocking sleep".to_string(),
                code: l.trim().to_string(),
                issue: "Blocking call in async",
                severity: "critical",
                fix: "Use tokio::time::sleep",
            });
        }
    }
    issues
}

/// Analyze complexity.
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Complexity annotations
pub fn analyze_complexity(content: &str) -> Vec<ComplexityAnnotation> {
    let mut anns = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let fn_re = match compile_regex(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)") {
        Some(r) => r,
        None => return anns,
    };
    for (i, l) in lines.iter().enumerate() {
        if let Some(caps) = fn_re.captures(l) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str();
                if n == "main" || n == "new" {
                    continue;
                }
                let mut loops = 0;
                let mut d: usize = 0;
                for line in lines.iter().take(lines.len().min(i + 50)).skip(i) {
                    if line.contains("for ") || line.contains("while ") {
                        loops += 1;
                    }
                    for c in line.chars() {
                        match c {
                            '{' => d += 1,
                            '}' => {
                                d = d.saturating_sub(1);
                                if d == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    if d == 0 {
                        break;
                    }
                }
                if loops >= 2 {
                    anns.push(ComplexityAnnotation {
                        function: n.to_string(),
                        line: i + 1,
                        documented: None,
                        inferred: "O(n²)".to_string(),
                        confidence: "high",
                        matches: false,
                    });
                }
            }
        }
    }
    anns
}

/// A string allocation issue
#[derive(Debug, Clone)]
pub struct StringIssue {
    /// Line number (1-indexed)
    pub line: usize,
    /// The pattern matched
    pub pattern: &'static str,
    /// The actual code snippet
    pub code: String,
    /// Advisory note
    pub note: &'static str,
    /// Severity: "critical" or "warning"
    pub severity: &'static str,
}

/// An iterator issue
#[derive(Debug, Clone)]
pub struct IteratorIssue {
    /// Line number
    pub line: usize,
    /// Pattern matched
    pub pattern: &'static str,
    /// Code snippet
    pub code: String,
    /// Issue description
    pub issue: &'static str,
    /// Severity
    pub severity: &'static str,
    /// Suggested fix
    pub fix: &'static str,
}

/// A lock contention issue
#[derive(Debug, Clone)]
pub struct LockIssue {
    /// Line number
    pub line: usize,
    /// Pattern matched
    pub pattern: &'static str,
    /// Code snippet
    pub code: String,
    /// Issue description
    pub issue: &'static str,
    /// Severity
    pub severity: &'static str,
    /// Suggested fix
    pub fix: &'static str,
}

/// Detect string allocation issues.
///
/// Patterns detected:
/// - String concatenation with `+` in loops
/// - `format!()` followed by `.push_str()` (should use `write!`)
/// - Repeated `.to_string()` on same value
/// - `&String` parameters that should be `&str`
///
/// # Arguments
/// * `content` - Source code
/// # Returns
/// String issues found
pub fn detect_string_issues(content: &str) -> Vec<StringIssue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let loops = find_loop_lines(content);

    // Pattern 1: String concatenation with + in loops
    let concat_re = compile_regex(r#"=\s*\w+\s*\+\s*[&"]"#);
    for (i, l) in lines.iter().enumerate() {
        if loops.contains(&i) {
            if let Some(ref re) = concat_re {
                if re.is_match(l) && !is_justified(&lines, i, &["// STRING:"]) {
                    issues.push(StringIssue {
                        line: i + 1,
                        pattern: "string + in loop",
                        code: l.trim().to_string(),
                        note: "Use String::push_str() or write!() macro",
                        severity: "critical",
                    });
                }
            }
        }
    }

    // Pattern 2: format!() followed by push_str() - should use write!()
    for (i, l) in lines.iter().enumerate() {
        if l.contains("format!")
            && l.contains(".push_str")
            && !is_justified(&lines, i, &["// STRING:"])
        {
            issues.push(StringIssue {
                line: i + 1,
                pattern: "format! + push_str",
                code: l.trim().to_string(),
                note: "Use write!(&mut s, ...) instead",
                severity: "warning",
            });
        }
    }

    // Pattern 3: Repeated .to_string() - look for same variable
    let to_string_re = compile_regex(r"(\w+)\.to_string\(\)");
    let mut seen_conversions: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for (i, l) in lines.iter().enumerate() {
        if let Some(ref re) = to_string_re {
            for caps in re.captures_iter(l) {
                if let Some(var) = caps.get(1) {
                    let var_name = var.as_str().to_string();
                    if let Some(&first_line) = seen_conversions.get(&var_name) {
                        if i > first_line + 1
                            && i < first_line + 20
                            && !is_justified(&lines, i, &["// STRING:"])
                        {
                            issues.push(StringIssue {
                                line: i + 1,
                                pattern: "repeated to_string",
                                code: l.trim().to_string(),
                                note: "Cache the String or use &str",
                                severity: "warning",
                            });
                        }
                    } else {
                        seen_conversions.insert(var_name, i);
                    }
                }
            }
        }
    }

    // Pattern 4: &String parameters that should be &str
    let fn_param_re = compile_regex(r"fn\s+\w+\s*\([^)]*&\s*String[^)]*\)");
    for (i, l) in lines.iter().enumerate() {
        if let Some(ref re) = fn_param_re {
            if re.is_match(l) && !is_justified(&lines, i, &["// STRING:", "// API:"]) {
                issues.push(StringIssue {
                    line: i + 1,
                    pattern: "&String parameter",
                    code: l.trim().to_string(),
                    note: "Use &str instead of &String for flexibility",
                    severity: "warning",
                });
            }
        }
    }

    issues
}

/// Detect iterator anti-patterns.
///
/// Patterns detected:
/// - `.collect()` immediately followed by `.iter()` (redundant allocation)
/// - `.collect::<Vec<_>>()` when only `.first()` or `.next()` needed
/// - Multiple passes over collected data that could be single pass
///
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Iterator issues found
pub fn detect_iterator_issues(content: &str) -> Vec<IteratorIssue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Pattern 1: .collect().iter() - redundant allocation
    let collect_iter_re = compile_regex(r"\.collect\s*\(\s*\)\s*\.iter\s*\(");
    for (i, l) in lines.iter().enumerate() {
        if let Some(ref re) = collect_iter_re {
            if re.is_match(l) && !is_justified(&lines, i, &["// ITER:"]) {
                issues.push(IteratorIssue {
                    line: i + 1,
                    pattern: "collect().iter()",
                    code: l.trim().to_string(),
                    issue: "Collecting then iterating allocates unnecessarily",
                    severity: "critical",
                    fix: "Chain iterators directly without collect()",
                });
            }
        }
    }

    // Pattern 2: .collect::<Vec<_>>() followed by .first()/.last()/.get(0)
    for (i, l) in lines.iter().enumerate() {
        let is_collect_vec = l.contains(".collect") && l.contains("Vec");
        let is_single_access = l.contains(".first()")
            || l.contains(".last()")
            || l.contains(".get(0)")
            || l.contains(".get(1)")
            || l.contains("[0]")
            || l.contains("[1]");

        if is_collect_vec && is_single_access && !is_justified(&lines, i, &["// ITER:"]) {
            issues.push(IteratorIssue {
                line: i + 1,
                pattern: "collect for single element",
                code: l.trim().to_string(),
                issue: "Allocating Vec just to get one element",
                severity: "critical",
                fix: "Use .next() or .nth(n) on the iterator directly",
            });
        }
    }

    // Pattern 3: Check for multipass - .iter() appearing multiple times on same variable
    // Look for patterns like: let v: Vec<_> = ... .collect(); ... v.iter() ... v.iter()
    let binding_re = compile_regex(r"let\s+(\w+)\s*(?::\s*Vec)?.*\.collect");
    let mut collected_vars: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (i, l) in lines.iter().enumerate() {
        if let Some(ref re) = binding_re {
            if let Some(caps) = re.captures(l) {
                if let Some(var) = caps.get(1) {
                    collected_vars.insert(var.as_str().to_string(), i);
                }
            }
        }
    }

    // Now check for multiple .iter() on collected vars
    for var in collected_vars.keys() {
        let mut iter_count = 0;
        let mut iter_lines = Vec::new();
        let pattern = format!("{}.iter()", var);
        for (i, l) in lines.iter().enumerate() {
            if l.contains(&pattern) {
                iter_count += 1;
                iter_lines.push(i + 1);
            }
        }
        if iter_count >= 2 {
            // Multiple iterations over collected data
            if !is_justified(&lines, iter_lines[1] - 1, &["// ITER:", "// MULTIPASS:"]) {
                issues.push(IteratorIssue {
                    line: iter_lines[1],
                    pattern: "multiple iterations",
                    code: format!(
                        "{} iterated {} times (lines {:?})",
                        var, iter_count, iter_lines
                    ),
                    issue: "Multiple passes over data that could be single pass",
                    severity: "warning",
                    fix: "Combine operations in a single iterator chain or use fold()",
                });
            }
        }
    }

    issues
}

/// Detect lock contention and deadlock risks.
///
/// Patterns detected:
/// - Lock held across `.await` (deadlock risk)
/// - Lock acquired inside loop (contention)
/// - Nested locks (deadlock risk)
/// - `Mutex<T>` in async without `tokio::sync::Mutex`
///
/// # Arguments
/// * `content` - Source code
/// # Returns
/// Lock issues found
pub fn detect_lock_issues(content: &str) -> Vec<LockIssue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let loops = find_loop_lines(content);

    // Track if we're in an async function
    let async_fn_re = compile_regex(r"async\s+fn\s+\w+");
    let mut in_async = false;
    let mut async_depth: usize = 0;
    let mut depth: usize = 0;

    // Track lock acquisitions
    let lock_re = compile_regex(r"\.(lock|read|write)\s*\(\s*\)");
    let await_re = compile_regex(r"\.await");

    // Track std::sync::Mutex usage in async context
    let std_mutex_re = compile_regex(r"std::sync::(Mutex|RwLock)");
    let use_std_sync =
        content.contains("use std::sync::Mutex") || content.contains("use std::sync::RwLock");

    for (i, l) in lines.iter().enumerate() {
        // Track async function scope
        if let Some(ref re) = async_fn_re {
            if re.is_match(l) {
                in_async = true;
                async_depth = depth;
            }
        }

        // Track brace depth
        for c in l.chars() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if in_async && depth <= async_depth {
                        in_async = false;
                    }
                }
                _ => {}
            }
        }

        // Pattern 1: Lock acquired inside loop (contention)
        if loops.contains(&i) {
            if let Some(ref re) = lock_re {
                if re.is_match(l) && !is_justified(&lines, i, &["// LOCK:"]) {
                    issues.push(LockIssue {
                        line: i + 1,
                        pattern: "lock in loop",
                        code: l.trim().to_string(),
                        issue: "Lock acquired inside loop causes contention",
                        severity: "critical",
                        fix: "Acquire lock before loop, or use lock-free data structure",
                    });
                }
            }
        }

        // Pattern 2: Lock held across await (check if lock and await on same line or nearby)
        if in_async {
            if let Some(ref lock_match) = lock_re {
                if let Some(ref await_match) = await_re {
                    if lock_match.is_match(l) {
                        // Check if await is on same line or within next few lines
                        let has_await_nearby = await_match.is_match(l)
                            || lines
                                .get(i + 1)
                                .is_some_and(|next| await_match.is_match(next))
                            || lines
                                .get(i + 2)
                                .is_some_and(|next| await_match.is_match(next));

                        if has_await_nearby && !is_justified(&lines, i, &["// LOCK:"]) {
                            issues.push(LockIssue {
                                line: i + 1,
                                pattern: "lock across await",
                                code: l.trim().to_string(),
                                issue: "Lock held across .await can cause deadlock",
                                severity: "critical",
                                fix: "Release lock before await, or use tokio::sync::Mutex",
                            });
                        }
                    }
                }
            }
        }

        // Pattern 3: std::sync::Mutex in async context
        if in_async && use_std_sync {
            if let Some(ref re) = std_mutex_re {
                if re.is_match(l) && !is_justified(&lines, i, &["// LOCK:"]) {
                    issues.push(LockIssue {
                        line: i + 1,
                        pattern: "std Mutex in async",
                        code: l.trim().to_string(),
                        issue: "std::sync::Mutex in async fn can block executor",
                        severity: "warning",
                        fix: "Use tokio::sync::Mutex for async-aware locking",
                    });
                }
            }
        }

        // Pattern 4: Nested locks (multiple .lock() calls on same line or nearby)
        if let Some(ref re) = lock_re {
            let lock_count = re.find_iter(l).count();
            if lock_count >= 2 && !is_justified(&lines, i, &["// LOCK:", "// NESTED:"]) {
                issues.push(LockIssue {
                    line: i + 1,
                    pattern: "nested locks",
                    code: l.trim().to_string(),
                    issue: "Multiple locks on same line risks deadlock",
                    severity: "critical",
                    fix: "Acquire locks in consistent order, or restructure",
                });
            }
        }
    }

    issues
}

/// A SIMD optimization opportunity
#[derive(Debug, Clone)]
pub struct SimdOpportunity {
    /// Line number (1-indexed)
    pub line: usize,
    /// The pattern matched
    pub pattern: &'static str,
    /// The actual code snippet
    pub code: String,
    /// Description of the opportunity
    pub opportunity: &'static str,
    /// Suggested optimization
    pub suggestion: &'static str,
    /// Estimated speedup potential
    pub speedup: &'static str,
}

/// Detect loops that could benefit from SIMD vectorization.
///
/// Patterns detected:
/// - Simple numeric loops (element-wise add, multiply, compare)
/// - Element-wise operations on arrays/slices
/// - Reduction operations (sum, min, max, fold)
/// - Loops with predictable iteration counts
///
/// # Arguments
/// * `content` - Source code
/// # Returns
/// SIMD opportunities found
pub fn detect_simd_opportunities(content: &str) -> Vec<SimdOpportunity> {
    let mut opportunities = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let loops = find_loop_lines(content);

    // Pattern 1: Simple element-wise operations in loops
    // Look for: arr[i] = arr[i] + value, arr[i] *= value, etc.
    let element_op_re = compile_regex(r"\[.+\]\s*[+\-*/]=");
    let indexed_assign_re = compile_regex(r"\[.+\]\s*=\s*\[.+\]\s*[+\-*/]");

    for (i, l) in lines.iter().enumerate() {
        if !loops.contains(&i) {
            continue;
        }

        // Check for element-wise compound assignment
        if let Some(ref re) = element_op_re {
            if re.is_match(l) && !is_justified(&lines, i, &["// SIMD:", "// VECTORIZED:"]) {
                opportunities.push(SimdOpportunity {
                    line: i + 1,
                    pattern: "element-wise compound op",
                    code: l.trim().to_string(),
                    opportunity: "Element-wise operation in loop is SIMD-friendly",
                    suggestion: "Consider using std::simd, packed_simd, or rayon for parallelization",
                    speedup: "2-8x with SIMD",
                });
            }
        }

        // Check for indexed assignment with arithmetic
        if let Some(ref re) = indexed_assign_re {
            if re.is_match(l) && !is_justified(&lines, i, &["// SIMD:", "// VECTORIZED:"]) {
                opportunities.push(SimdOpportunity {
                    line: i + 1,
                    pattern: "indexed arithmetic",
                    code: l.trim().to_string(),
                    opportunity: "Indexed arithmetic operation is vectorizable",
                    suggestion: "Consider iter().zip() with SIMD intrinsics or rayon::par_iter()",
                    speedup: "2-8x with SIMD",
                });
            }
        }
    }

    // Pattern 2: Reduction operations (.sum(), .fold(), .min(), .max())
    // These can benefit from parallel reduction
    let reduction_re = compile_regex(r"\.(sum|fold|min|max|product)\s*\(");
    for (i, l) in lines.iter().enumerate() {
        if let Some(ref re) = reduction_re {
            // Check if it's a large reduction (on a range or collected iterator)
            let is_large_collection = l.contains("..") || l.contains(".iter()");
            if re.is_match(l)
                && is_large_collection
                && !is_justified(&lines, i, &["// SIMD:", "// PARALLEL:"])
            {
                opportunities.push(SimdOpportunity {
                    line: i + 1,
                    pattern: "reduction operation",
                    code: l.trim().to_string(),
                    opportunity: "Reduction can use parallel SIMD accumulation",
                    suggestion: "Consider rayon::par_iter().sum() or manual SIMD reduction",
                    speedup: "4-16x with parallel reduction",
                });
            }
        }
    }

    // Pattern 3: Numeric for loops with array operations
    // Look for: for i in 0..n { ... arr[i] ... }
    let numeric_for_re = compile_regex(r"for\s+\w+\s+in\s+\d+\s*\.\.");
    for (i, l) in lines.iter().enumerate() {
        if let Some(ref re) = numeric_for_re {
            if re.is_match(l) {
                // Check next few lines for array indexing
                let has_array_op = lines
                    .iter()
                    .skip(i + 1)
                    .take(9)
                    .take_while(|line| !line.contains('}'))
                    .any(|line| line.contains('[') && line.contains(']'));

                if has_array_op && !is_justified(&lines, i, &["// SIMD:", "// VECTORIZED:"]) {
                    opportunities.push(SimdOpportunity {
                        line: i + 1,
                        pattern: "numeric loop with indexing",
                        code: l.trim().to_string(),
                        opportunity: "Numeric loop with array access is SIMD candidate",
                        suggestion: "Consider iterator methods or explicit SIMD with std::simd",
                        speedup: "2-8x with vectorization",
                    });
                }
            }
        }
    }

    opportunities
}
