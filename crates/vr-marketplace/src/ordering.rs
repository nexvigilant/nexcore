//! Order workflow — state machine for marketplace orders.
//!
//! Orders flow through a defined lifecycle from Draft to Completed (or Cancelled/Disputed).
//! Commission rates depend on order value and provider type, using a sliding scale
//! for CRO services.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use vr_core::{Money, OrderId, ProgramId, ProviderId, TenantId, VrError};

use crate::catalog::CatalogEntryId;
use crate::providers::ProviderType;

/// The lifecycle state of a marketplace order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order is being prepared, not yet submitted.
    Draft,
    /// Submitted to provider for review.
    Submitted,
    /// Provider accepted the order.
    Accepted,
    /// Work is underway.
    InProgress,
    /// Data/results have been delivered by the provider.
    DataDelivered,
    /// Quality control passed — results meet specifications.
    QcPassed,
    /// Quality control failed — results do not meet specifications.
    QcFailed,
    /// Order fully completed and closed.
    Completed,
    /// Order cancelled before completion.
    Cancelled,
    /// Under dispute resolution.
    Disputed,
}

/// A marketplace order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier.
    pub id: OrderId,
    /// Tenant that placed the order.
    pub tenant_id: TenantId,
    /// The catalog entry (service) being ordered.
    pub service_id: CatalogEntryId,
    /// The provider fulfilling the order.
    pub provider_id: ProviderId,
    /// Optional drug program this order is associated with.
    pub program_id: Option<ProgramId>,
    /// Current lifecycle status.
    pub status: OrderStatus,
    /// Detailed order specifications (quantities, conditions, etc.).
    pub order_details: serde_json::Value,
    /// Price quoted at order creation.
    pub quoted_price: Money,
    /// Actual price after completion (may differ from quote).
    pub actual_price: Option<Money>,
    /// Platform commission on this order.
    pub commission: Option<Money>,
    /// When the order was submitted.
    pub submitted_at: Option<DateTime>,
    /// When the order was completed.
    pub completed_at: Option<DateTime>,
}

impl Order {
    /// Create a new draft order.
    #[must_use]
    pub fn new_draft(
        tenant_id: TenantId,
        service_id: CatalogEntryId,
        provider_id: ProviderId,
        program_id: Option<ProgramId>,
        order_details: serde_json::Value,
        quoted_price: Money,
    ) -> Self {
        Self {
            id: OrderId::new(),
            tenant_id,
            service_id,
            provider_id,
            program_id,
            status: OrderStatus::Draft,
            order_details,
            quoted_price,
            actual_price: None,
            commission: None,
            submitted_at: None,
            completed_at: None,
        }
    }

    /// Whether the order is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, OrderStatus::Completed | OrderStatus::Cancelled)
    }
}

/// Validate that a state transition is allowed.
///
/// Valid transitions form a DAG:
/// ```text
/// Draft → Submitted → Accepted → InProgress → DataDelivered → QcPassed → Completed
///                                                             ↘ QcFailed → InProgress (rework)
/// Draft → Cancelled
/// Submitted → Cancelled
/// Accepted → Cancelled
/// InProgress → Cancelled
/// Any non-terminal → Disputed
/// Disputed → Cancelled
/// Disputed → InProgress (resolved, resume work)
/// ```
pub fn validate_order_transition(from: &OrderStatus, to: &OrderStatus) -> Result<(), VrError> {
    let valid = match (from, to) {
        // Happy path
        (OrderStatus::Draft, OrderStatus::Submitted) => true,
        (OrderStatus::Submitted, OrderStatus::Accepted) => true,
        (OrderStatus::Accepted, OrderStatus::InProgress) => true,
        (OrderStatus::InProgress, OrderStatus::DataDelivered) => true,
        (OrderStatus::DataDelivered, OrderStatus::QcPassed) => true,
        (OrderStatus::DataDelivered, OrderStatus::QcFailed) => true,
        (OrderStatus::QcPassed, OrderStatus::Completed) => true,

        // QC failure rework loop
        (OrderStatus::QcFailed, OrderStatus::InProgress) => true,

        // Cancellation from non-terminal states
        (OrderStatus::Draft, OrderStatus::Cancelled) => true,
        (OrderStatus::Submitted, OrderStatus::Cancelled) => true,
        (OrderStatus::Accepted, OrderStatus::Cancelled) => true,
        (OrderStatus::InProgress, OrderStatus::Cancelled) => true,

        // Dispute from any non-terminal state
        (s, OrderStatus::Disputed)
            if !matches!(s, OrderStatus::Completed | OrderStatus::Cancelled) =>
        {
            true
        }

        // Dispute resolution
        (OrderStatus::Disputed, OrderStatus::Cancelled) => true,
        (OrderStatus::Disputed, OrderStatus::InProgress) => true,

        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(VrError::InvalidInput {
            message: format!("invalid order transition from {from:?} to {to:?}"),
        })
    }
}

/// Calculate the platform commission for an order.
///
/// CRO sliding scale (based on order value in cents):
/// - Under $10,000 (1_000_000 cents): 8% (800 bps)
/// - $10,000 to $49,999.99: 6% (600 bps)
/// - $50,000 to $99,999.99: 5% (500 bps)
/// - $100,000 and above: 3% (300 bps)
///
/// Non-CRO providers: flat 10% (1000 bps).
#[must_use]
pub fn calculate_order_commission(order_value: Money, provider_type: &ProviderType) -> Money {
    let bps = match provider_type {
        ProviderType::Cro => {
            let cents = order_value.cents();
            if cents < 1_000_000 {
                // Under $10,000
                800
            } else if cents < 5_000_000 {
                // $10,000 – $49,999.99
                600
            } else if cents < 10_000_000 {
                // $50,000 – $99,999.99
                500
            } else {
                // $100,000+
                300
            }
        }
        ProviderType::ModelCreator | ProviderType::Expert | ProviderType::DataProvider => 1000,
    };
    order_value.percent_bps(bps)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // --- Transition tests ---

    #[test]
    fn happy_path_transitions() {
        let transitions = [
            (OrderStatus::Draft, OrderStatus::Submitted),
            (OrderStatus::Submitted, OrderStatus::Accepted),
            (OrderStatus::Accepted, OrderStatus::InProgress),
            (OrderStatus::InProgress, OrderStatus::DataDelivered),
            (OrderStatus::DataDelivered, OrderStatus::QcPassed),
            (OrderStatus::QcPassed, OrderStatus::Completed),
        ];
        for (from, to) in &transitions {
            assert!(
                validate_order_transition(from, to).is_ok(),
                "expected {from:?} → {to:?} to be valid"
            );
        }
    }

    #[test]
    fn qc_failure_rework_loop() {
        assert!(
            validate_order_transition(&OrderStatus::DataDelivered, &OrderStatus::QcFailed).is_ok()
        );
        assert!(
            validate_order_transition(&OrderStatus::QcFailed, &OrderStatus::InProgress).is_ok()
        );
    }

    #[test]
    fn cancellation_from_non_terminal_states() {
        let cancellable = [
            OrderStatus::Draft,
            OrderStatus::Submitted,
            OrderStatus::Accepted,
            OrderStatus::InProgress,
        ];
        for from in &cancellable {
            assert!(
                validate_order_transition(from, &OrderStatus::Cancelled).is_ok(),
                "expected {from:?} → Cancelled to be valid"
            );
        }
    }

    #[test]
    fn dispute_from_active_states() {
        let disputable = [
            OrderStatus::Draft,
            OrderStatus::Submitted,
            OrderStatus::Accepted,
            OrderStatus::InProgress,
            OrderStatus::DataDelivered,
            OrderStatus::QcPassed,
            OrderStatus::QcFailed,
        ];
        for from in &disputable {
            assert!(
                validate_order_transition(from, &OrderStatus::Disputed).is_ok(),
                "expected {from:?} → Disputed to be valid"
            );
        }
    }

    #[test]
    fn cannot_dispute_from_terminal() {
        assert!(
            validate_order_transition(&OrderStatus::Completed, &OrderStatus::Disputed).is_err()
        );
        assert!(
            validate_order_transition(&OrderStatus::Cancelled, &OrderStatus::Disputed).is_err()
        );
    }

    #[test]
    fn dispute_resolution_paths() {
        assert!(validate_order_transition(&OrderStatus::Disputed, &OrderStatus::Cancelled).is_ok());
        assert!(
            validate_order_transition(&OrderStatus::Disputed, &OrderStatus::InProgress).is_ok()
        );
    }

    #[test]
    fn invalid_backward_transitions() {
        assert!(validate_order_transition(&OrderStatus::Accepted, &OrderStatus::Draft).is_err());
        assert!(
            validate_order_transition(&OrderStatus::Completed, &OrderStatus::InProgress).is_err()
        );
        assert!(validate_order_transition(&OrderStatus::Cancelled, &OrderStatus::Draft).is_err());
    }

    // --- Commission tests ---

    #[test]
    fn cro_commission_under_10k() {
        let value = Money::usd(500_000); // $5,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 8% of $5,000 = $400 = 40,000 cents
        assert_eq!(commission.cents(), 40_000);
    }

    #[test]
    fn cro_commission_10k_to_50k() {
        let value = Money::usd(2_500_000); // $25,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 6% of $25,000 = $1,500 = 150,000 cents
        assert_eq!(commission.cents(), 150_000);
    }

    #[test]
    fn cro_commission_50k_to_100k() {
        let value = Money::usd(7_500_000); // $75,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 5% of $75,000 = $3,750 = 375,000 cents
        assert_eq!(commission.cents(), 375_000);
    }

    #[test]
    fn cro_commission_over_100k() {
        let value = Money::usd(20_000_000); // $200,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 3% of $200,000 = $6,000 = 600,000 cents
        assert_eq!(commission.cents(), 600_000);
    }

    #[test]
    fn non_cro_commission_flat_10_percent() {
        let value = Money::usd(500_000); // $5,000
        for provider_type in &[
            ProviderType::ModelCreator,
            ProviderType::Expert,
            ProviderType::DataProvider,
        ] {
            let commission = calculate_order_commission(value, provider_type);
            // 10% of $5,000 = $500 = 50,000 cents
            assert_eq!(
                commission.cents(),
                50_000,
                "expected 10% for {provider_type:?}"
            );
        }
    }

    #[test]
    fn cro_commission_at_boundary_10k() {
        // Exactly $10,000 → 6% bracket
        let value = Money::usd(1_000_000); // $10,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 6% of $10,000 = $600 = 60,000 cents
        assert_eq!(commission.cents(), 60_000);
    }

    #[test]
    fn cro_commission_at_boundary_50k() {
        // Exactly $50,000 → 5% bracket
        let value = Money::usd(5_000_000); // $50,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 5% of $50,000 = $2,500 = 250_000 cents
        assert_eq!(commission.cents(), 250_000);
    }

    #[test]
    fn cro_commission_at_boundary_100k() {
        // Exactly $100,000 → 3% bracket
        let value = Money::usd(10_000_000); // $100,000
        let commission = calculate_order_commission(value, &ProviderType::Cro);
        // 3% of $100,000 = $3,000 = 300,000 cents
        assert_eq!(commission.cents(), 300_000);
    }

    #[test]
    fn order_new_draft() {
        let order = Order::new_draft(
            TenantId::new(),
            CatalogEntryId::new(),
            ProviderId::new(),
            None,
            serde_json::json!({"compounds": 50}),
            Money::usd(250_000),
        );
        assert_eq!(order.status, OrderStatus::Draft);
        assert!(order.actual_price.is_none());
        assert!(order.commission.is_none());
        assert!(order.submitted_at.is_none());
        assert!(!order.is_terminal());
    }

    #[test]
    fn terminal_states() {
        let mut order = Order::new_draft(
            TenantId::new(),
            CatalogEntryId::new(),
            ProviderId::new(),
            None,
            serde_json::json!({}),
            Money::usd(100),
        );
        order.status = OrderStatus::Completed;
        assert!(order.is_terminal());

        order.status = OrderStatus::Cancelled;
        assert!(order.is_terminal());

        order.status = OrderStatus::InProgress;
        assert!(!order.is_terminal());
    }
}
