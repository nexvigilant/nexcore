// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Structured OS journal — typed event recording with severity-controlled retention.
//!
//! Inspired by Apple Unified Logging (severity→storage policy), ETW (multi-dimensional
//! keywords), and journald (structured key-value entries). Designed for synchronous
//! nexcore-os — no async runtime required.
//!
//! ## Primitive Grounding
//!
//! - σ Sequence: Monotonic entry ordering (sequence numbers)
//! - μ Mapping: Subsystem + Category → event classification
//! - N Quantity: Entry counts, ring buffer depth, keyword bitmask
//! - ∂ Boundary: Severity controls retention boundary
//! - ν Frequency: Tick-aligned entry timestamps
//! - π Persistence: Entries survive within retention window

use std::collections::VecDeque;
use std::fmt;

// ── Severity (7 levels, syslog-aligned) ─────────────────────────────

/// Journal entry severity level.
///
/// Severity controls storage policy (higher severity = longer retention).
///
/// Tier: T2-P (∂ Boundary — severity as retention boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Severity {
    /// Detailed diagnostic trace (disabled by default, zero-cost when off).
    Debug = 0,
    /// General informational messages.
    Info = 1,
    /// Normal but significant conditions.
    Notice = 2,
    /// Warning conditions — may indicate problems.
    Warning = 3,
    /// Error conditions — operation failed.
    Error = 4,
    /// Critical conditions — subsystem unable to function.
    Critical = 5,
    /// System unusable — immediate attention required.
    Fatal = 6,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Notice => write!(f, "NOTICE"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRIT"),
            Self::Fatal => write!(f, "FATAL"),
        }
    }
}

// ── Subsystem ───────────────────────────────────────────────────────

/// OS subsystem identifier.
///
/// Tier: T2-P (λ Location — identifies event origin)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subsystem {
    /// OS kernel (boot, lifecycle, tick loop).
    Kernel,
    /// Boot sequence.
    Boot,
    /// Security monitor (PAMP/DAMP threat detection).
    Security,
    /// Service manager (lifecycle transitions).
    Service,
    /// Trust engine (Bayesian trust scoring).
    Trust,
    /// Network manager.
    Network,
    /// Audio manager.
    Audio,
    /// IPC event bus.
    Ipc,
    /// Vault (encrypted secrets).
    Vault,
    /// User authentication.
    User,
    /// STOS state machine runtime.
    Stos,
}

impl fmt::Display for Subsystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel => write!(f, "kernel"),
            Self::Boot => write!(f, "boot"),
            Self::Security => write!(f, "security"),
            Self::Service => write!(f, "service"),
            Self::Trust => write!(f, "trust"),
            Self::Network => write!(f, "network"),
            Self::Audio => write!(f, "audio"),
            Self::Ipc => write!(f, "ipc"),
            Self::Vault => write!(f, "vault"),
            Self::User => write!(f, "user"),
            Self::Stos => write!(f, "stos"),
        }
    }
}

// ── Keywords (bitmask for multi-dimensional tagging) ────────────────

/// Event keyword bitmask — a single event can belong to multiple categories.
///
/// Inspired by ETW keyword bitmask. Use `|` to combine.
///
/// Tier: T2-C (Σ Sum + μ Mapping — multi-tag classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keywords(pub u64);

impl Keywords {
    pub const NONE: Self = Self(0);
    pub const LIFECYCLE: Self = Self(1 << 0);
    pub const PERFORMANCE: Self = Self(1 << 1);
    pub const SECURITY: Self = Self(1 << 2);
    pub const NETWORK: Self = Self(1 << 3);
    pub const STORAGE: Self = Self(1 << 4);
    pub const TRUST: Self = Self(1 << 5);
    pub const STATE_CHANGE: Self = Self(1 << 6);
    pub const BOOT: Self = Self(1 << 7);
    pub const ERROR: Self = Self(1 << 8);
    pub const DIAGNOSTIC: Self = Self(1 << 9);
    pub const AUDIO: Self = Self(1 << 10);
    pub const USER: Self = Self(1 << 11);
    pub const IPC: Self = Self(1 << 12);
    pub const RESOURCE: Self = Self(1 << 13);
    pub const CONFIG: Self = Self(1 << 14);

    /// Combine two keyword sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Check if this set contains all bits from `other`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if this set has any overlap with `other`.
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for Keywords {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

// ── Typed Fields (ETW-inspired structured data) ─────────────────────

/// A typed field attached to a journal entry.
///
/// Self-describing — each field carries its name and typed value.
/// Inspired by ETW TraceLogging (named, typed, no format strings).
///
/// Tier: T2-C (μ Mapping + N Quantity — named typed values)
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub value: FieldValue,
}

/// Typed field values.
#[derive(Debug, Clone)]
pub enum FieldValue {
    /// UTF-8 string.
    Str(String),
    /// Unsigned 64-bit integer.
    U64(u64),
    /// Signed 64-bit integer.
    I64(i64),
    /// 64-bit float.
    F64(f64),
    /// Boolean.
    Bool(bool),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{s}"),
            Self::U64(v) => write!(f, "{v}"),
            Self::I64(v) => write!(f, "{v}"),
            Self::F64(v) => write!(f, "{v:.3}"),
            Self::Bool(v) => write!(f, "{v}"),
        }
    }
}

// ── JournalEntry ────────────────────────────────────────────────────

/// A structured journal entry.
///
/// Every OS event is recorded as a JournalEntry with typed fields,
/// multi-dimensional keywords, and severity-controlled retention.
///
/// Tier: T3 (σ + μ + N + ∂ + ν + λ)
#[derive(Debug, Clone)]
pub struct JournalEntry {
    /// Monotonic sequence number (unique, never reused).
    pub seq: u64,
    /// Tick count when entry was recorded.
    pub tick: u64,
    /// Event source subsystem.
    pub subsystem: Subsystem,
    /// Category within subsystem (free-form string).
    pub category: String,
    /// Severity level (controls retention).
    pub severity: Severity,
    /// Keyword bitmask for multi-dimensional classification.
    pub keywords: Keywords,
    /// Human-readable message.
    pub message: String,
    /// Typed structured fields (ETW-style).
    pub fields: Vec<Field>,
}

impl JournalEntry {
    /// Create a new journal entry builder.
    pub fn new(
        subsystem: Subsystem,
        category: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            seq: 0, // set by OsJournal on append
            tick: 0, // set by OsJournal on append
            subsystem,
            category: category.into(),
            severity,
            keywords: Keywords::NONE,
            message: message.into(),
            fields: Vec::new(),
        }
    }

    /// Add keyword tags.
    #[must_use]
    pub fn with_keywords(mut self, keywords: Keywords) -> Self {
        self.keywords = self.keywords | keywords;
        self
    }

    /// Add a string field.
    #[must_use]
    pub fn with_str(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(Field {
            name: name.into(),
            value: FieldValue::Str(value.into()),
        });
        self
    }

    /// Add a u64 field.
    #[must_use]
    pub fn with_u64(mut self, name: impl Into<String>, value: u64) -> Self {
        self.fields.push(Field {
            name: name.into(),
            value: FieldValue::U64(value),
        });
        self
    }

    /// Add an i64 field.
    #[must_use]
    pub fn with_i64(mut self, name: impl Into<String>, value: i64) -> Self {
        self.fields.push(Field {
            name: name.into(),
            value: FieldValue::I64(value),
        });
        self
    }

    /// Add an f64 field.
    #[must_use]
    pub fn with_f64(mut self, name: impl Into<String>, value: f64) -> Self {
        self.fields.push(Field {
            name: name.into(),
            value: FieldValue::F64(value),
        });
        self
    }

    /// Add a bool field.
    #[must_use]
    pub fn with_bool(mut self, name: impl Into<String>, value: bool) -> Self {
        self.fields.push(Field {
            name: name.into(),
            value: FieldValue::Bool(value),
        });
        self
    }
}

impl fmt::Display for JournalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{seq:>6}|T{tick:>6}] {sev:<6} {sub}.{cat}: {msg}",
            seq = self.seq,
            tick = self.tick,
            sev = self.severity,
            sub = self.subsystem,
            cat = self.category,
            msg = self.message,
        )?;
        for field in &self.fields {
            write!(f, " {name}={val}", name = field.name, val = field.value)?;
        }
        Ok(())
    }
}

// ── OsJournal ───────────────────────────────────────────────────────

/// OS-level structured event journal.
///
/// Append-only ring buffer with severity-controlled retention.
/// Entries at Error+ severity are always retained (up to error capacity).
/// Debug/Info/Notice/Warning rotate through the main ring buffer.
///
/// Tier: T3 (σ + μ + N + ∂ + ν + π)
pub struct OsJournal {
    /// Ring buffer of recent entries (all severities).
    entries: VecDeque<JournalEntry>,
    /// Separate buffer for Error+ entries (longer retention).
    errors: VecDeque<JournalEntry>,
    /// Maximum ring buffer capacity.
    max_entries: usize,
    /// Maximum error buffer capacity.
    max_errors: usize,
    /// Next sequence number.
    next_seq: u64,
    /// Total entries recorded since boot (including rotated out).
    total_recorded: u64,
    /// Total entries dropped (severity below threshold).
    total_dropped: u64,
    /// Minimum severity to record (entries below this are dropped).
    min_severity: Severity,
}

impl OsJournal {
    /// Create a new journal with default capacity.
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(1024),
            errors: VecDeque::with_capacity(256),
            max_entries: 4096,
            max_errors: 1024,
            next_seq: 1,
            total_recorded: 0,
            total_dropped: 0,
            min_severity: Severity::Info,
        }
    }

    /// Create a journal with custom capacity and minimum severity.
    pub fn with_config(max_entries: usize, max_errors: usize, min_severity: Severity) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(1024)),
            errors: VecDeque::with_capacity(max_errors.min(256)),
            max_entries,
            max_errors,
            next_seq: 1,
            total_recorded: 0,
            total_dropped: 0,
            min_severity,
        }
    }

    /// Record an entry at the current tick.
    ///
    /// Returns the assigned sequence number, or None if dropped (below min_severity).
    pub fn record(&mut self, mut entry: JournalEntry, tick: u64) -> Option<u64> {
        // Severity gate: drop entries below threshold
        if entry.severity < self.min_severity {
            self.total_dropped += 1;
            return None;
        }

        // Assign sequence number and tick
        let seq = self.next_seq;
        entry.seq = seq;
        entry.tick = tick;
        self.next_seq += 1;
        self.total_recorded += 1;

        // Error+ entries get duplicated into the error buffer (longer retention)
        if entry.severity >= Severity::Error {
            if self.errors.len() >= self.max_errors {
                self.errors.pop_front();
            }
            self.errors.push_back(entry.clone());
        }

        // All entries go into the main ring buffer
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);

        Some(seq)
    }

    /// Query recent entries, optionally filtered.
    pub fn query(&self, filter: &JournalFilter) -> Vec<&JournalEntry> {
        self.entries
            .iter()
            .filter(|e| filter.matches(e))
            .collect()
    }

    /// Query the error buffer (Error+ entries with longer retention).
    pub fn errors(&self) -> &VecDeque<JournalEntry> {
        &self.errors
    }

    /// Get the N most recent entries.
    pub fn recent(&self, count: usize) -> Vec<&JournalEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Total entries in the main buffer.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total entries recorded since boot (including rotated out).
    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    /// Total entries dropped (below min_severity).
    pub fn total_dropped(&self) -> u64 {
        self.total_dropped
    }

    /// Error buffer length.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Next sequence number (peek).
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }
}

impl Default for OsJournal {
    fn default() -> Self {
        Self::new()
    }
}

// ── JournalFilter ───────────────────────────────────────────────────

/// Filter for querying journal entries.
///
/// All fields are optional — unset fields match everything.
///
/// Tier: T2-C (κ Comparison + ∂ Boundary — filter criteria)
#[derive(Debug, Clone, Default)]
pub struct JournalFilter {
    /// Minimum severity (inclusive).
    pub min_severity: Option<Severity>,
    /// Subsystem match.
    pub subsystem: Option<Subsystem>,
    /// Keyword match (entry must contain ALL specified keywords).
    pub keywords: Option<Keywords>,
    /// Minimum tick (inclusive).
    pub since_tick: Option<u64>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

impl JournalFilter {
    /// Create an empty filter (matches everything).
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by minimum severity.
    #[must_use]
    pub fn severity(mut self, min: Severity) -> Self {
        self.min_severity = Some(min);
        self
    }

    /// Filter by subsystem.
    #[must_use]
    pub fn subsystem(mut self, sub: Subsystem) -> Self {
        self.subsystem = Some(sub);
        self
    }

    /// Filter by keywords (must contain all specified).
    #[must_use]
    pub fn keywords(mut self, kw: Keywords) -> Self {
        self.keywords = Some(kw);
        self
    }

    /// Filter by minimum tick.
    #[must_use]
    pub fn since(mut self, tick: u64) -> Self {
        self.since_tick = Some(tick);
        self
    }

    /// Limit result count.
    #[must_use]
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Check if an entry matches this filter.
    fn matches(&self, entry: &JournalEntry) -> bool {
        if let Some(min_sev) = self.min_severity {
            if entry.severity < min_sev {
                return false;
            }
        }
        if let Some(sub) = self.subsystem {
            if entry.subsystem != sub {
                return false;
            }
        }
        if let Some(kw) = self.keywords {
            if !entry.keywords.contains(kw) {
                return false;
            }
        }
        if let Some(tick) = self.since_tick {
            if entry.tick < tick {
                return false;
            }
        }
        true
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_records_entry() {
        let mut journal = OsJournal::new();
        let entry = JournalEntry::new(Subsystem::Kernel, "boot", Severity::Info, "System booting");
        let seq = journal.record(entry, 1);
        assert_eq!(seq, Some(1));
        assert_eq!(journal.len(), 1);
        assert_eq!(journal.total_recorded(), 1);
    }

    #[test]
    fn journal_assigns_monotonic_seq() {
        let mut journal = OsJournal::new();
        for i in 0..5 {
            let entry = JournalEntry::new(Subsystem::Kernel, "tick", Severity::Info, format!("tick {i}"));
            let seq = journal.record(entry, i);
            assert_eq!(seq, Some(i + 1));
        }
        assert_eq!(journal.next_seq(), 6);
    }

    #[test]
    fn journal_drops_below_min_severity() {
        let mut journal = OsJournal::with_config(100, 50, Severity::Warning);
        let debug = JournalEntry::new(Subsystem::Kernel, "trace", Severity::Debug, "debug msg");
        let info = JournalEntry::new(Subsystem::Kernel, "trace", Severity::Info, "info msg");
        let warn = JournalEntry::new(Subsystem::Kernel, "trace", Severity::Warning, "warn msg");

        assert_eq!(journal.record(debug, 1), None);
        assert_eq!(journal.record(info, 2), None);
        assert!(journal.record(warn, 3).is_some());

        assert_eq!(journal.len(), 1);
        assert_eq!(journal.total_dropped(), 2);
    }

    #[test]
    fn journal_error_buffer_separate_retention() {
        let mut journal = OsJournal::new();
        let info = JournalEntry::new(Subsystem::Service, "start", Severity::Info, "started");
        let error = JournalEntry::new(Subsystem::Service, "crash", Severity::Error, "crashed");
        let crit = JournalEntry::new(Subsystem::Security, "breach", Severity::Critical, "breach detected");

        journal.record(info, 1);
        journal.record(error, 2);
        journal.record(crit, 3);

        assert_eq!(journal.len(), 3);
        assert_eq!(journal.error_count(), 2); // Error + Critical in error buffer
    }

    #[test]
    fn journal_ring_buffer_rotates() {
        let mut journal = OsJournal::with_config(5, 5, Severity::Info);
        for i in 0..10 {
            let entry = JournalEntry::new(Subsystem::Kernel, "tick", Severity::Info, format!("entry {i}"));
            journal.record(entry, i);
        }
        assert_eq!(journal.len(), 5); // Only last 5 retained
        assert_eq!(journal.total_recorded(), 10); // All 10 counted
        // Oldest entry should be seq 6 (entries 0-4 rotated out)
        let recent = journal.recent(1);
        assert_eq!(recent[0].seq, 10);
    }

    #[test]
    fn journal_filter_by_severity() {
        let mut journal = OsJournal::new();
        journal.record(JournalEntry::new(Subsystem::Kernel, "a", Severity::Info, "info"), 1);
        journal.record(JournalEntry::new(Subsystem::Kernel, "a", Severity::Warning, "warn"), 2);
        journal.record(JournalEntry::new(Subsystem::Kernel, "a", Severity::Error, "error"), 3);

        let filter = JournalFilter::new().severity(Severity::Warning);
        let results = journal.query(&filter);
        assert_eq!(results.len(), 2); // Warning + Error
    }

    #[test]
    fn journal_filter_by_subsystem() {
        let mut journal = OsJournal::new();
        journal.record(JournalEntry::new(Subsystem::Kernel, "a", Severity::Info, "k"), 1);
        journal.record(JournalEntry::new(Subsystem::Security, "b", Severity::Info, "s"), 2);
        journal.record(JournalEntry::new(Subsystem::Kernel, "c", Severity::Info, "k2"), 3);

        let filter = JournalFilter::new().subsystem(Subsystem::Kernel);
        let results = journal.query(&filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn journal_filter_by_keywords() {
        let mut journal = OsJournal::new();
        journal.record(
            JournalEntry::new(Subsystem::Security, "threat", Severity::Warning, "threat")
                .with_keywords(Keywords::SECURITY | Keywords::NETWORK),
            1,
        );
        journal.record(
            JournalEntry::new(Subsystem::Network, "conn", Severity::Info, "connected")
                .with_keywords(Keywords::NETWORK),
            2,
        );

        // Filter for entries tagged with BOTH security AND network
        let filter = JournalFilter::new().keywords(Keywords::SECURITY | Keywords::NETWORK);
        let results = journal.query(&filter);
        assert_eq!(results.len(), 1); // Only the threat entry has both
    }

    #[test]
    fn journal_filter_by_tick() {
        let mut journal = OsJournal::new();
        journal.record(JournalEntry::new(Subsystem::Kernel, "a", Severity::Info, "old"), 5);
        journal.record(JournalEntry::new(Subsystem::Kernel, "a", Severity::Info, "new"), 15);

        let filter = JournalFilter::new().since(10);
        let results = journal.query(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "new");
    }

    #[test]
    fn journal_entry_display() {
        let entry = JournalEntry {
            seq: 42,
            tick: 100,
            subsystem: Subsystem::Service,
            category: "start".to_string(),
            severity: Severity::Info,
            keywords: Keywords::LIFECYCLE,
            message: "guardian started".to_string(),
            fields: vec![
                Field { name: "service".to_string(), value: FieldValue::Str("guardian".to_string()) },
                Field { name: "priority".to_string(), value: FieldValue::U64(1) },
            ],
        };
        let display = format!("{entry}");
        assert!(display.contains("INFO"));
        assert!(display.contains("service.start"));
        assert!(display.contains("guardian started"));
        assert!(display.contains("service=guardian"));
        assert!(display.contains("priority=1"));
    }

    #[test]
    fn journal_entry_builder_chain() {
        let entry = JournalEntry::new(Subsystem::Trust, "degrade", Severity::Warning, "trust degraded")
            .with_keywords(Keywords::TRUST | Keywords::SECURITY)
            .with_f64("score", 0.45)
            .with_str("reason", "threat detected")
            .with_bool("critical", false);

        assert_eq!(entry.subsystem, Subsystem::Trust);
        assert_eq!(entry.fields.len(), 3);
        assert!(entry.keywords.contains(Keywords::TRUST));
        assert!(entry.keywords.contains(Keywords::SECURITY));
    }

    #[test]
    fn keywords_bitmask_operations() {
        let k1 = Keywords::SECURITY | Keywords::NETWORK;
        assert!(k1.contains(Keywords::SECURITY));
        assert!(k1.contains(Keywords::NETWORK));
        assert!(!k1.contains(Keywords::BOOT));
        assert!(k1.intersects(Keywords::SECURITY));
        assert!(!k1.intersects(Keywords::BOOT));

        let k2 = k1.union(Keywords::BOOT);
        assert!(k2.contains(Keywords::BOOT));
    }
}
