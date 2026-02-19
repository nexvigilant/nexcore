//! Usage metering — tracks and aggregates platform resource consumption.
//!
//! Every billable action on the platform emits a [`MeterEvent`]. Events are
//! aggregated into [`UsageAggregation`] records for billing period totals.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use vr_core::{TenantId, UserId};

/// A single metered usage event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterEvent {
    /// Unique event identifier.
    pub event_id: Uuid,
    /// Tenant that generated this event.
    pub tenant_id: TenantId,
    /// User who triggered the event.
    pub user_id: UserId,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// What type of usage this represents.
    pub meter_type: MeterType,
    /// Numeric quantity (e.g., 1.0 for a single API call, bytes for storage).
    pub quantity: f64,
    /// Arbitrary key-value metadata for auditing.
    pub metadata: HashMap<String, String>,
}

/// Types of metered usage on the platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MeterType {
    /// Compound scoring operation.
    CompoundScored,
    /// ML model prediction.
    MlPrediction {
        /// Which model was used.
        model_id: String,
    },
    /// Virtual screening run.
    VirtualScreen {
        /// Number of compounds screened in this run.
        compounds_screened: u64,
    },
    /// Data ingested into the platform.
    DataIngested {
        /// Bytes ingested.
        bytes: u64,
    },
    /// Storage currently used.
    StorageUsed {
        /// Bytes stored.
        bytes: u64,
    },
    /// API call.
    ApiCall {
        /// Which endpoint was called.
        endpoint: String,
    },
    /// CRO order facilitated through the marketplace.
    CroOrderFacilitated {
        /// Total order value in cents.
        order_value_cents: u64,
    },
    /// Marketplace AI model used.
    MarketplaceModelUsed {
        /// Which marketplace model.
        model_id: String,
    },
    /// Expert engagement session.
    ExpertEngagement {
        /// Which expert.
        expert_id: String,
        /// Duration in hours.
        hours: f64,
    },
}

/// Aggregated usage for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageAggregation {
    /// Tenant these totals belong to.
    pub tenant_id: TenantId,
    /// Start of the billing period (inclusive).
    pub period_start: DateTime<Utc>,
    /// End of the billing period (exclusive).
    pub period_end: DateTime<Utc>,
    /// Total compounds scored.
    pub compounds_scored: u64,
    /// Total ML predictions run.
    pub ml_predictions: u64,
    /// Total virtual screening runs.
    pub virtual_screens: u64,
    /// Current storage usage in bytes (max observed during period).
    pub storage_bytes: u64,
    /// Total API calls.
    pub api_calls: u64,
    /// Total CRO order value facilitated, in cents.
    pub cro_order_total_cents: u64,
    /// Total marketplace model uses.
    pub marketplace_model_uses: u64,
}

/// Aggregate a slice of meter events into period totals.
///
/// Events are counted by type. Storage is tracked as the maximum observed
/// value during the period (high-water mark). CRO order values are summed.
///
/// # Arguments
/// * `events` - The meter events to aggregate. Must all belong to the same
///   tenant and billing period (caller is responsible for filtering).
///
/// # Returns
/// A [`UsageAggregation`] with totals. If `events` is empty, returns an
/// aggregation with all zeros and a default tenant ID.
#[must_use]
pub fn aggregate_events(events: &[MeterEvent]) -> UsageAggregation {
    let (tenant_id, period_start, period_end) = if let Some(first) = events.first() {
        let mut earliest = first.timestamp;
        let mut latest = first.timestamp;
        for event in events.iter().skip(1) {
            if event.timestamp < earliest {
                earliest = event.timestamp;
            }
            if event.timestamp > latest {
                latest = event.timestamp;
            }
        }
        (first.tenant_id, earliest, latest)
    } else {
        let now = Utc::now();
        (TenantId::new(), now, now)
    };

    let mut compounds_scored: u64 = 0;
    let mut ml_predictions: u64 = 0;
    let mut virtual_screens: u64 = 0;
    let mut storage_bytes: u64 = 0;
    let mut api_calls: u64 = 0;
    let mut cro_order_total_cents: u64 = 0;
    let mut marketplace_model_uses: u64 = 0;

    for event in events {
        match &event.meter_type {
            MeterType::CompoundScored => {
                compounds_scored += event.quantity as u64;
            }
            MeterType::MlPrediction { .. } => {
                ml_predictions += event.quantity as u64;
            }
            MeterType::VirtualScreen { .. } => {
                virtual_screens += event.quantity as u64;
            }
            MeterType::StorageUsed { bytes } => {
                // High-water mark for storage.
                if *bytes > storage_bytes {
                    storage_bytes = *bytes;
                }
            }
            MeterType::DataIngested { .. } => {
                // Data ingestion events contribute to storage but are
                // tracked separately via StorageUsed snapshots.
            }
            MeterType::ApiCall { .. } => {
                api_calls += event.quantity as u64;
            }
            MeterType::CroOrderFacilitated { order_value_cents } => {
                cro_order_total_cents += order_value_cents;
            }
            MeterType::MarketplaceModelUsed { .. } => {
                marketplace_model_uses += event.quantity as u64;
            }
            MeterType::ExpertEngagement { .. } => {
                // Expert engagements are billed separately through commissions,
                // not through the usage aggregation pipeline.
            }
        }
    }

    UsageAggregation {
        tenant_id,
        period_start,
        period_end,
        compounds_scored,
        ml_predictions,
        virtual_screens,
        storage_bytes,
        api_calls,
        cro_order_total_cents,
        marketplace_model_uses,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_event(tenant_id: TenantId, meter_type: MeterType, quantity: f64) -> MeterEvent {
        MeterEvent {
            event_id: Uuid::new_v4(),
            tenant_id,
            user_id: UserId::new(),
            timestamp: Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap(),
            meter_type,
            quantity,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn aggregate_empty_events() {
        let agg = aggregate_events(&[]);
        assert_eq!(agg.compounds_scored, 0);
        assert_eq!(agg.ml_predictions, 0);
        assert_eq!(agg.virtual_screens, 0);
        assert_eq!(agg.storage_bytes, 0);
        assert_eq!(agg.api_calls, 0);
        assert_eq!(agg.cro_order_total_cents, 0);
        assert_eq!(agg.marketplace_model_uses, 0);
    }

    #[test]
    fn aggregate_compound_scoring() {
        let tid = TenantId::new();
        let events = vec![
            make_event(tid, MeterType::CompoundScored, 100.0),
            make_event(tid, MeterType::CompoundScored, 250.0),
            make_event(tid, MeterType::CompoundScored, 50.0),
        ];
        let agg = aggregate_events(&events);
        assert_eq!(agg.compounds_scored, 400);
    }

    #[test]
    fn aggregate_ml_predictions() {
        let tid = TenantId::new();
        let events = vec![
            make_event(
                tid,
                MeterType::MlPrediction {
                    model_id: "admet-v2".into(),
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::MlPrediction {
                    model_id: "tox-v1".into(),
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::MlPrediction {
                    model_id: "admet-v2".into(),
                },
                1.0,
            ),
        ];
        let agg = aggregate_events(&events);
        assert_eq!(agg.ml_predictions, 3);
    }

    #[test]
    fn aggregate_storage_high_water_mark() {
        let tid = TenantId::new();
        let events = vec![
            make_event(tid, MeterType::StorageUsed { bytes: 1_000_000 }, 1.0),
            make_event(tid, MeterType::StorageUsed { bytes: 5_000_000 }, 1.0),
            make_event(tid, MeterType::StorageUsed { bytes: 3_000_000 }, 1.0),
        ];
        let agg = aggregate_events(&events);
        // Should be the maximum, not the sum.
        assert_eq!(agg.storage_bytes, 5_000_000);
    }

    #[test]
    fn aggregate_cro_orders_sum_values() {
        let tid = TenantId::new();
        let events = vec![
            make_event(
                tid,
                MeterType::CroOrderFacilitated {
                    order_value_cents: 500_000,
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::CroOrderFacilitated {
                    order_value_cents: 1_200_000,
                },
                1.0,
            ),
        ];
        let agg = aggregate_events(&events);
        assert_eq!(agg.cro_order_total_cents, 1_700_000); // $17,000
    }

    #[test]
    fn aggregate_mixed_event_types() {
        let tid = TenantId::new();
        let events = vec![
            make_event(tid, MeterType::CompoundScored, 1000.0),
            make_event(
                tid,
                MeterType::MlPrediction {
                    model_id: "m1".into(),
                },
                5.0,
            ),
            make_event(
                tid,
                MeterType::VirtualScreen {
                    compounds_screened: 10_000,
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::ApiCall {
                    endpoint: "/api/v1/compounds".into(),
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::ApiCall {
                    endpoint: "/api/v1/predict".into(),
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::StorageUsed {
                    bytes: 50_000_000_000,
                },
                1.0,
            ),
            make_event(
                tid,
                MeterType::MarketplaceModelUsed {
                    model_id: "vendor-x".into(),
                },
                3.0,
            ),
        ];
        let agg = aggregate_events(&events);
        assert_eq!(agg.compounds_scored, 1000);
        assert_eq!(agg.ml_predictions, 5);
        assert_eq!(agg.virtual_screens, 1);
        assert_eq!(agg.api_calls, 2);
        assert_eq!(agg.storage_bytes, 50_000_000_000);
        assert_eq!(agg.marketplace_model_uses, 3);
    }

    #[test]
    fn aggregate_period_bounds_from_timestamps() {
        let tid = TenantId::new();
        let mut e1 = make_event(tid, MeterType::CompoundScored, 1.0);
        e1.timestamp = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        let mut e2 = make_event(tid, MeterType::CompoundScored, 1.0);
        e2.timestamp = Utc.with_ymd_and_hms(2025, 6, 30, 23, 59, 59).unwrap();

        let agg = aggregate_events(&[e1, e2]);
        assert_eq!(
            agg.period_start,
            Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap()
        );
        assert_eq!(
            agg.period_end,
            Utc.with_ymd_and_hms(2025, 6, 30, 23, 59, 59).unwrap()
        );
    }
}
