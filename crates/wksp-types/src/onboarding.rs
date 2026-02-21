//! Onboarding types — new user welcome flow state

use serde::{Deserialize, Serialize};

/// Onboarding step identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    Welcome,
    ProfileSetup,
    Interests,
    Complete,
}

/// Persisted onboarding state for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingState {
    pub user_id: String,
    pub current_step: OnboardingStep,
    pub is_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experience_years: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_goal: Option<String>,
    #[serde(default)]
    pub selected_domains: Vec<String>,
}

/// Extended user profile fields collected during onboarding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileExtended {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experience_years: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_goal: Option<String>,
    #[serde(default)]
    pub interest_domains: Vec<String>,
    #[serde(default)]
    pub interest_topics: Vec<String>,
}
