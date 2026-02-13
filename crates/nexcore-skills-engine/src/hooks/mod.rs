//! # Hooks Module
//!
//! File organization, validation, and monitoring hooks for Claude Code.
//!
//! ## Features
//!
//! - **File Policy Engine** - Dynamic policy enforcement from YAML configuration
//! - **Blindspot Checker** - Post-write self-review reminders
//! - **Staleness Detection** - Identify stale and orphaned files
//! - **Directory Scanner** - Scan for violations and stale files
//! - **Organization Reports** - Directory health and structure analysis
//!
//! ## CLI Commands
//!
//! ```text
//! nexcore hooks validate <path>       Validate file placement against policies
//! nexcore hooks staleness <path>      Check if a file is stale
//! nexcore hooks categorize <path>     Get file category
//! nexcore hooks scan <dir>            Scan directory for violations
//! nexcore hooks policy                Show loaded policy configuration
//! nexcore hooks blindspot <path>      Generate blindspot check for file type
//! nexcore hooks schema-version        Output schema version for compatibility checks
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use nexcore_vigilance::skills::hooks::{PolicyFile, validate_file, scan_directory};
//! use std::path::Path;
//!
//! // Load policy
//! let policy = PolicyFile::load_or_default(None);
//!
//! // Validate a single file
//! let result = validate_file(Path::new("src/main.rs"), &policy);
//! if result.has_warnings() {
//!     println!("{:#?}", result.warnings);
//! }
//!
//! // Scan a directory
//! let scan = scan_directory(Path::new("."), 3, &policy);
//! println!("Found {} violations", scan.violations.len());
//! ```

/// Schema version for hooks module output format.
/// Increment this when making breaking changes to JSON output structure.
/// Bash scripts check this to ensure compatibility.
pub const SCHEMA_VERSION: u32 = 1;

pub mod blindspot;
pub mod policy;
pub mod scanner;
pub mod staleness;
pub mod validation;

// Re-export primary types
pub use blindspot::{BlindspotCheck, BlindspotType};
pub use policy::{
    ForbiddenZones, PlacementRule, PolicyError, PolicyFile, PolicySettings, ProjectStructure,
    StalenessConfig, StalenessRule, expand_path, is_in_path, matches_glob,
};
pub use scanner::{
    ScanOptions, ScanResult, ScanSummary, format_scan_result, scan_directory, scan_with_options,
};
pub use staleness::{
    StalenessResult, StalenessSummary, check_staleness, format_staleness_result, get_file_age_days,
};
pub use validation::{
    ValidationResult, ValidationWarning, categorize_file, format_validation_result, validate_file,
};

/// Convenience struct for hook validation (backward compatible)
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Hook name
    pub hook: String,
    /// Whether validation passed
    pub passed: bool,
    /// Issues found
    pub issues: Vec<String>,
}

/// Validate file placement according to policy (simple API)
pub fn validate_placement(path: &std::path::Path) -> HookResult {
    let policy = PolicyFile::load_or_default(None);
    let result = validate_file(path, &policy);

    HookResult {
        hook: "placement".to_string(),
        passed: result.warnings.is_empty(),
        issues: result.warnings.iter().map(|w| w.message.clone()).collect(),
    }
}

/// Check for stale files (simple API)
pub fn check_staleness_simple(path: &std::path::Path, max_age_days: u64) -> HookResult {
    let mut issues = Vec::new();

    if let Ok(metadata) = path.metadata() {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.elapsed() {
                let days = duration.as_secs() / 86400;
                if days > max_age_days {
                    issues.push(format!(
                        "File is {} days old (threshold: {})",
                        days, max_age_days
                    ));
                }
            }
        }
    }

    HookResult {
        hook: "staleness".to_string(),
        passed: issues.is_empty(),
        issues,
    }
}
