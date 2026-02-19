//! Tier enforcement — feature gating, usage limits, overage handling.
//!
//! Real calculations for tier limit checking, usage tracking,
//! and soft/hard limit enforcement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use vr_core::{SubscriptionTier, TenantId, VrError, VrResult};

/// Current usage snapshot for a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsage {
    pub tenant_id: TenantId,
    pub programs_count: u32,
    pub users_count: u32,
    pub storage_bytes: u64,
    pub virtual_screens_this_month: u32,
    pub compounds_scored_this_month: u64,
    pub ml_predictions_this_month: u64,
    pub api_calls_this_month: u64,
    pub snapshot_at: DateTime<Utc>,
}

/// Result of a limit check — either allowed, warning, or blocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitCheckResult {
    /// Within limits, proceed normally.
    Allowed,
    /// Approaching limit (>80% used). Warn the user.
    Warning {
        resource: String,
        used: u64,
        limit: u64,
        percent_used: u8,
    },
    /// At or over limit. Block the action.
    Blocked {
        resource: String,
        used: u64,
        limit: u64,
    },
}

impl LimitCheckResult {
    pub fn is_allowed(&self) -> bool {
        !matches!(self, Self::Blocked { .. })
    }
}

/// Check if a tenant can create another program.
pub fn check_program_limit(tier: &SubscriptionTier, current_count: u32) -> LimitCheckResult {
    match tier.max_programs() {
        None => LimitCheckResult::Allowed,
        Some(max) => {
            if current_count >= max {
                LimitCheckResult::Blocked {
                    resource: "programs".into(),
                    used: current_count as u64,
                    limit: max as u64,
                }
            } else if current_count as f64 / max as f64 >= 0.8 {
                LimitCheckResult::Warning {
                    resource: "programs".into(),
                    used: current_count as u64,
                    limit: max as u64,
                    percent_used: ((current_count as f64 / max as f64) * 100.0) as u8,
                }
            } else {
                LimitCheckResult::Allowed
            }
        }
    }
}

/// Check if a tenant can add another user.
pub fn check_user_limit(tier: &SubscriptionTier, current_count: u32) -> LimitCheckResult {
    match tier.max_users() {
        None => LimitCheckResult::Allowed,
        Some(max) => {
            if current_count >= max {
                LimitCheckResult::Blocked {
                    resource: "users".into(),
                    used: current_count as u64,
                    limit: max as u64,
                }
            } else if current_count as f64 / max as f64 >= 0.8 {
                LimitCheckResult::Warning {
                    resource: "users".into(),
                    used: current_count as u64,
                    limit: max as u64,
                    percent_used: ((current_count as f64 / max as f64) * 100.0) as u8,
                }
            } else {
                LimitCheckResult::Allowed
            }
        }
    }
}

/// Check storage limit. Returns overage in bytes if over.
pub fn check_storage_limit(tier: &SubscriptionTier, used_bytes: u64) -> LimitCheckResult {
    let limit = tier.storage_bytes();
    if used_bytes > limit {
        LimitCheckResult::Blocked {
            resource: "storage_bytes".into(),
            used: used_bytes,
            limit,
        }
    } else if used_bytes as f64 / limit as f64 >= 0.8 {
        LimitCheckResult::Warning {
            resource: "storage_bytes".into(),
            used: used_bytes,
            limit,
            percent_used: ((used_bytes as f64 / limit as f64) * 100.0) as u8,
        }
    } else {
        LimitCheckResult::Allowed
    }
}

/// Calculate storage overage in whole GB (rounded up).
pub fn storage_overage_gb(tier: &SubscriptionTier, used_bytes: u64) -> u64 {
    let limit = tier.storage_bytes();
    if used_bytes <= limit {
        return 0;
    }
    let overage_bytes = used_bytes - limit;
    // Round up to nearest GB
    (overage_bytes + 1_073_741_823) / 1_073_741_824
}

/// Feature gate check — does this tier include a specific feature?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    BasicCompoundScoring,
    FullCompoundScoring,
    CustomCompoundScoring,
    SarTools,
    VirtualScreening,
    PlatformMlModels,
    MarketplaceMlModels,
    CustomModelTraining,
    ActiveLearning,
    KnowledgeGraph,
    PatentMonitoring,
    IntelDigest,
    CroMarketplace,
    DealPipeline,
    AssetListing,
    DealRoom,
    ApiAccess,
    CliTool,
    Sso,
    DataResidencyOptions,
    DedicatedSupport,
}

impl Feature {
    /// Minimum tier required for this feature.
    pub fn min_tier(&self) -> SubscriptionTier {
        use Feature::*;
        match self {
            BasicCompoundScoring => SubscriptionTier::Explorer,
            FullCompoundScoring => SubscriptionTier::Accelerator,
            CustomCompoundScoring => SubscriptionTier::Enterprise,
            SarTools => SubscriptionTier::Accelerator,
            VirtualScreening => SubscriptionTier::Academic,
            PlatformMlModels => SubscriptionTier::Explorer,
            MarketplaceMlModels => SubscriptionTier::Accelerator,
            CustomModelTraining => SubscriptionTier::Enterprise,
            ActiveLearning => SubscriptionTier::Accelerator,
            KnowledgeGraph => SubscriptionTier::Explorer,
            PatentMonitoring => SubscriptionTier::Accelerator,
            IntelDigest => SubscriptionTier::Accelerator,
            CroMarketplace => SubscriptionTier::Accelerator,
            DealPipeline => SubscriptionTier::Accelerator,
            AssetListing => SubscriptionTier::Accelerator,
            DealRoom => SubscriptionTier::Accelerator,
            ApiAccess => SubscriptionTier::Accelerator,
            CliTool => SubscriptionTier::Accelerator,
            Sso => SubscriptionTier::Enterprise,
            DataResidencyOptions => SubscriptionTier::Enterprise,
            DedicatedSupport => SubscriptionTier::Enterprise,
        }
    }

    /// Check if a tier has access to this feature.
    pub fn is_available(&self, tier: &SubscriptionTier) -> bool {
        tier.includes(&self.min_tier())
    }
}

/// Enforce that a feature is available for a tenant's tier.
pub fn require_feature(tier: &SubscriptionTier, feature: Feature) -> VrResult<()> {
    if feature.is_available(tier) {
        Ok(())
    } else {
        Err(VrError::TierInsufficient {
            current: *tier,
            required: feature.min_tier(),
        })
    }
}

/// Full usage check — run all limit checks and return the most severe result.
pub fn check_all_limits(tier: &SubscriptionTier, usage: &TenantUsage) -> Vec<LimitCheckResult> {
    let mut results = Vec::new();

    let program_check = check_program_limit(tier, usage.programs_count);
    if !matches!(program_check, LimitCheckResult::Allowed) {
        results.push(program_check);
    }

    let user_check = check_user_limit(tier, usage.users_count);
    if !matches!(user_check, LimitCheckResult::Allowed) {
        results.push(user_check);
    }

    let storage_check = check_storage_limit(tier, usage.storage_bytes);
    if !matches!(storage_check, LimitCheckResult::Allowed) {
        results.push(storage_check);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_program_limit() {
        // Explorer: 1 program max
        let result = check_program_limit(&SubscriptionTier::Explorer, 0);
        assert_eq!(result, LimitCheckResult::Allowed);

        let result = check_program_limit(&SubscriptionTier::Explorer, 1);
        assert!(matches!(result, LimitCheckResult::Blocked { .. }));
    }

    #[test]
    fn accelerator_program_warning_at_80_percent() {
        // Accelerator: 5 programs max. Warning at 4 (80%).
        let result = check_program_limit(&SubscriptionTier::Accelerator, 4);
        assert!(matches!(
            result,
            LimitCheckResult::Warning {
                percent_used: 80,
                ..
            }
        ));

        let result = check_program_limit(&SubscriptionTier::Accelerator, 3);
        assert_eq!(result, LimitCheckResult::Allowed);
    }

    #[test]
    fn enterprise_unlimited_programs() {
        let result = check_program_limit(&SubscriptionTier::Enterprise, 999);
        assert_eq!(result, LimitCheckResult::Allowed);
    }

    #[test]
    fn storage_overage_calculation() {
        // Explorer: 5 GB limit
        assert_eq!(
            storage_overage_gb(&SubscriptionTier::Explorer, 4 * 1_073_741_824),
            0
        );
        assert_eq!(
            storage_overage_gb(&SubscriptionTier::Explorer, 5 * 1_073_741_824),
            0
        );
        // 1 byte over = 1 GB overage (rounded up)
        assert_eq!(
            storage_overage_gb(&SubscriptionTier::Explorer, 5 * 1_073_741_824 + 1),
            1
        );
        // 2.5 GB over = 3 GB overage (rounded up)
        let over_2_5_gb = 5 * 1_073_741_824 + 2_684_354_560;
        assert_eq!(
            storage_overage_gb(&SubscriptionTier::Explorer, over_2_5_gb),
            3
        );
    }

    #[test]
    fn feature_gating() {
        assert!(Feature::BasicCompoundScoring.is_available(&SubscriptionTier::Explorer));
        assert!(!Feature::SarTools.is_available(&SubscriptionTier::Explorer));
        assert!(Feature::SarTools.is_available(&SubscriptionTier::Accelerator));
        assert!(!Feature::Sso.is_available(&SubscriptionTier::Accelerator));
        assert!(Feature::Sso.is_available(&SubscriptionTier::Enterprise));
    }

    #[test]
    fn require_feature_returns_error() {
        let result = require_feature(&SubscriptionTier::Explorer, Feature::DealPipeline);
        assert!(result.is_err());
        let result = require_feature(&SubscriptionTier::Enterprise, Feature::DealPipeline);
        assert!(result.is_ok());
    }

    #[test]
    fn academic_tier_features() {
        // Academic gets virtual screening but not deal pipeline
        assert!(Feature::VirtualScreening.is_available(&SubscriptionTier::Academic));
        assert!(!Feature::DealPipeline.is_available(&SubscriptionTier::Academic));
        // Academic gets API access (rank 1, API needs Accelerator rank 3)
        // Wait — Academic rank is 1, Accelerator is 3. So Academic does NOT include Accelerator features.
        assert!(!Feature::ApiAccess.is_available(&SubscriptionTier::Academic));
    }
}
