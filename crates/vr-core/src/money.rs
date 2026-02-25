//! Money types — all financial amounts in cents to avoid floating point.
//!
//! All prices, commissions, invoices, and billing use integer cents (u64).
//! This prevents floating-point rounding errors in financial calculations.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Money amount in the smallest currency unit (cents for USD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Money {
    /// Amount in cents (1/100th of currency unit).
    cents: u64,
    /// ISO 4217 currency code.
    currency: Currency,
}

/// Supported currencies.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
}

impl Money {
    /// Create from cents.
    #[must_use]
    pub const fn from_cents(cents: u64, currency: Currency) -> Self {
        Self { cents, currency }
    }

    /// Create from dollars (or major currency unit).
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "dollars * 100 overflows only above 1.8e17 dollars, which is not a valid monetary input"
    )]
    pub const fn from_dollars(dollars: u64, currency: Currency) -> Self {
        Self {
            cents: dollars * 100,
            currency,
        }
    }

    /// USD convenience constructor.
    #[must_use]
    pub const fn usd(cents: u64) -> Self {
        Self::from_cents(cents, Currency::USD)
    }

    /// USD from dollars.
    #[must_use]
    pub const fn usd_dollars(dollars: u64) -> Self {
        Self::from_dollars(dollars, Currency::USD)
    }

    #[must_use]
    pub const fn cents(&self) -> u64 {
        self.cents
    }

    #[must_use]
    pub const fn currency(&self) -> Currency {
        self.currency
    }

    /// Convert to f64 dollars for display. NOT for calculations.
    #[must_use]
    #[allow(
        clippy::as_conversions,
        reason = "intentional lossy cast for display-only use; caller is warned not to use for calculations"
    )]
    pub fn as_dollars_f64(&self) -> f64 {
        self.cents as f64 / 100.0
    }

    /// Multiply by a quantity (e.g., price * units).
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "financial multiplication; callers are responsible for ensuring quantity does not cause overflow in their billing context"
    )]
    pub const fn times(self, quantity: u64) -> Self {
        Self {
            cents: self.cents * quantity,
            currency: self.currency,
        }
    }

    /// Apply a percentage (basis points: 100 = 1%, 10000 = 100%).
    /// Commission calculation: amount.percent_bps(800) = 8%.
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "bps is bounded 0..=10_000 by convention; intermediate product overflows only for amounts exceeding u64::MAX / 10_000"
    )]
    pub const fn percent_bps(self, bps: u64) -> Self {
        Self {
            cents: self.cents * bps / 10_000,
            currency: self.currency,
        }
    }

    /// Zero amount in same currency.
    #[must_use]
    pub const fn zero(currency: Currency) -> Self {
        Self { cents: 0, currency }
    }

    /// Check if zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.cents == 0
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        debug_assert_eq!(
            self.currency, rhs.currency,
            "cannot add different currencies"
        );
        Self {
            cents: self.cents.saturating_add(rhs.cents),
            currency: self.currency,
        }
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        debug_assert_eq!(
            self.currency, rhs.currency,
            "cannot subtract different currencies"
        );
        Self {
            cents: self.cents.saturating_sub(rhs.cents),
            currency: self.currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self.currency {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
        };
        write!(f, "{}{}.{:02}", symbol, self.cents / 100, self.cents % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_from_cents() {
        let m = Money::usd(12345);
        assert_eq!(m.cents(), 12345);
        assert_eq!(m.as_dollars_f64(), 123.45);
    }

    #[test]
    fn money_from_dollars() {
        let m = Money::usd_dollars(250);
        assert_eq!(m.cents(), 25000);
    }

    #[test]
    fn money_display() {
        assert_eq!(Money::usd(12345).to_string(), "$123.45");
        assert_eq!(Money::usd(100).to_string(), "$1.00");
        assert_eq!(Money::usd(5).to_string(), "$0.05");
        assert_eq!(Money::from_cents(999, Currency::EUR).to_string(), "€9.99");
    }

    #[test]
    fn money_add() {
        let a = Money::usd(100);
        let b = Money::usd(250);
        assert_eq!((a + b).cents(), 350);
    }

    #[test]
    fn money_sub_saturating() {
        let a = Money::usd(100);
        let b = Money::usd(250);
        assert_eq!((a - b).cents(), 0); // saturating
    }

    #[test]
    fn money_times() {
        let price = Money::usd(1); // $0.01 per compound
        let total = price.times(1000); // 1000 compounds
        assert_eq!(total.cents(), 1000);
        assert_eq!(total.to_string(), "$10.00");
    }

    #[test]
    fn commission_percent_bps() {
        let order_value = Money::usd_dollars(10_000); // $10,000 CRO order
        // 5% commission = 500 basis points
        let commission = order_value.percent_bps(500);
        assert_eq!(commission.cents(), 50_000); // $500
        assert_eq!(commission.to_string(), "$500.00");

        // 8% commission = 800 basis points
        let commission_8 = order_value.percent_bps(800);
        assert_eq!(commission_8.cents(), 80_000); // $800
    }

    #[test]
    fn platform_pricing_matches_architecture() {
        // $0.01/compound scored
        let compound_price = Money::usd(1);
        let batch_100 = compound_price.times(100);
        assert_eq!(batch_100.to_string(), "$1.00");

        // $0.05/ML prediction
        let ml_price = Money::usd(5);
        let predictions_1000 = ml_price.times(1000);
        assert_eq!(predictions_1000.to_string(), "$50.00");

        // $0.10/GB/month storage overage
        let storage_price = Money::usd(10);
        let overage_50gb = storage_price.times(50);
        assert_eq!(overage_50gb.to_string(), "$5.00");
    }
}
