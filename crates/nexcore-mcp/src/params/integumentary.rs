//! Params for integumentary system (boundary protection, permissions, scarring) tools.

use serde::Deserialize;

/// Evaluate permission cascade for an action.
#[derive(Debug, Deserialize)]
pub struct IntegumentaryPermissionParams {
    /// The action being evaluated (e.g., "Write", "Bash", "Edit")
    pub action: String,
    /// Target path or resource
    pub target: Option<String>,
}

/// Analyze settings precedence stack.
#[derive(Debug, Deserialize)]
pub struct IntegumentarySettingsParams {
    /// Setting key to trace through precedence layers
    pub setting_key: Option<String>,
}

/// Check sandbox isolation layers.
#[derive(Debug, Deserialize)]
pub struct IntegumentarySandboxParams {}

/// Check scarring mechanisms (learned restrictions from past incidents).
#[derive(Debug, Deserialize)]
pub struct IntegumentaryScarringParams {
    /// Optional incident type to check scars for
    pub incident_type: Option<String>,
}

/// Get integumentary system health overview.
#[derive(Debug, Deserialize)]
pub struct IntegumentaryHealthParams {}
