//! Business domain types: LLCs, transactions, and KPIs.

use chrono::{DateTime, NaiveDate, Utc};
use nexcore_id::NexId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::enums::TransactionType;

/// LLC entity model for multi-tenant support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Llc {
    /// Unique identifier.
    pub id: NexId,

    /// Legal business name.
    pub legal_name: String,

    /// Doing Business As name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dba_name: Option<String>,

    /// Employer Identification Number.
    pub ein: String,

    /// US state code of formation.
    pub state_of_formation: String,

    /// Date of formation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formation_date: Option<NaiveDate>,

    /// Fiscal year start month (1-12).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiscal_year_start: Option<u8>,

    /// Tax rate (e.g., 0.30 for 30%).
    #[serde(default = "default_tax_rate")]
    pub tax_rate: Decimal,

    /// Accounting method (Cash or Accrual).
    #[serde(default = "default_accounting_method")]
    pub accounting_method: AccountingMethod,

    /// Contact email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Contact phone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Address line 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line1: Option<String>,

    /// Address line 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line2: Option<String>,

    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// State.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// ZIP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip_code: Option<String>,

    /// Bank name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_name: Option<String>,

    /// Last 4 digits of bank account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_last4: Option<String>,

    /// Industry classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,

    /// NAICS code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naics_code: Option<String>,

    /// Annual report due date (MM-DD format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annual_report_due_date: Option<String>,

    /// Whether the LLC is active.
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_tax_rate() -> Decimal {
    Decimal::new(30, 2) // 0.30 = 30%
}

fn default_true() -> bool {
    true
}

fn default_accounting_method() -> AccountingMethod {
    AccountingMethod::Cash
}

/// Accounting method for tax purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AccountingMethod {
    /// Cash basis accounting.
    #[default]
    Cash,
    /// Accrual basis accounting.
    Accrual,
}

impl Llc {
    /// Create a new LLC with required fields.
    #[must_use]
    pub fn new(
        legal_name: impl Into<String>,
        ein: impl Into<String>,
        state_of_formation: impl Into<String>,
    ) -> Self {
        Self {
            id: NexId::v4(),
            legal_name: legal_name.into(),
            dba_name: None,
            ein: ein.into(),
            state_of_formation: state_of_formation.into(),
            formation_date: None,
            fiscal_year_start: None,
            tax_rate: default_tax_rate(),
            accounting_method: AccountingMethod::Cash,
            email: None,
            phone: None,
            address_line1: None,
            address_line2: None,
            city: None,
            state: None,
            zip_code: None,
            bank_name: None,
            bank_account_last4: None,
            industry: None,
            naics_code: None,
            annual_report_due_date: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Get full address as a single string.
    #[must_use]
    pub fn full_address(&self) -> Option<String> {
        let parts: Vec<&str> = [
            self.address_line1.as_deref(),
            self.address_line2.as_deref(),
            self.city.as_deref(),
            self.state.as_deref(),
            self.zip_code.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }
}

/// Financial transaction model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier.
    pub id: NexId,

    /// Associated LLC ID.
    pub llc_id: NexId,

    /// User who created the transaction.
    pub created_by: NexId,

    /// User who reconciled (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconciled_by: Option<NexId>,

    /// Transaction type (revenue or expense).
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,

    /// Transaction amount.
    pub amount: Decimal,

    /// Transaction date.
    pub date: NaiveDate,

    /// Description.
    pub description: String,

    /// Category.
    pub category: String,

    /// Invoice number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,

    /// Receipt number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_number: Option<String>,

    /// Client or vendor name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_vendor_name: Option<String>,

    /// Payment method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,

    /// Bank account used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account: Option<String>,

    /// Reference number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_number: Option<String>,

    /// Whether tax deductible.
    #[serde(default)]
    pub is_tax_deductible: bool,

    /// Tax category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_category: Option<String>,

    /// Whether reconciled.
    #[serde(default)]
    pub is_reconciled: bool,

    /// When reconciled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconciled_at: Option<DateTime<Utc>>,

    /// Receipt URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_url: Option<String>,

    /// Notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Transaction {
    /// Create a new transaction.
    #[must_use]
    pub fn new(
        llc_id: NexId,
        created_by: NexId,
        transaction_type: TransactionType,
        amount: Decimal,
        date: NaiveDate,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: NexId::v4(),
            llc_id,
            created_by,
            reconciled_by: None,
            transaction_type,
            amount,
            date,
            description: description.into(),
            category: category.into(),
            invoice_number: None,
            receipt_number: None,
            client_vendor_name: None,
            payment_method: None,
            bank_account: None,
            reference_number: None,
            is_tax_deductible: false,
            tax_category: None,
            is_reconciled: false,
            reconciled_at: None,
            receipt_url: None,
            notes: None,
            tags: Vec::new(),
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Check if this is a revenue transaction.
    #[must_use]
    pub fn is_revenue(&self) -> bool {
        self.transaction_type == TransactionType::Revenue
    }

    /// Check if this is an expense transaction.
    #[must_use]
    pub fn is_expense(&self) -> bool {
        self.transaction_type == TransactionType::Expense
    }

    /// Get signed amount (positive for revenue, negative for expense).
    #[must_use]
    pub fn signed_amount(&self) -> Decimal {
        match self.transaction_type {
            TransactionType::Revenue => self.amount,
            TransactionType::Expense => -self.amount,
        }
    }
}

/// KPI snapshot model for time-series metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Kpi {
    /// Unique identifier.
    pub id: NexId,

    /// Associated LLC ID.
    pub llc_id: NexId,

    /// Snapshot date.
    pub snapshot_date: NaiveDate,

    /// Total revenue to date.
    #[serde(default)]
    pub total_revenue: Decimal,

    /// Total expenses to date.
    #[serde(default)]
    pub total_expenses: Decimal,

    /// Net profit.
    #[serde(default)]
    pub net_profit: Decimal,

    /// Profit margin percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_margin: Option<Decimal>,

    /// Monthly revenue.
    #[serde(default)]
    pub monthly_revenue: Decimal,

    /// Monthly expenses.
    #[serde(default)]
    pub monthly_expenses: Decimal,

    /// Monthly profit.
    #[serde(default)]
    pub monthly_profit: Decimal,

    /// Year-to-date revenue.
    #[serde(default)]
    pub ytd_revenue: Decimal,

    /// Year-to-date expenses.
    #[serde(default)]
    pub ytd_expenses: Decimal,

    /// Year-to-date profit.
    #[serde(default)]
    pub ytd_profit: Decimal,

    /// Tax set aside amount.
    #[serde(default)]
    pub tax_set_aside: Decimal,

    /// Estimated tax owed.
    #[serde(default)]
    pub estimated_tax_owed: Decimal,

    /// Number of active clients.
    #[serde(default)]
    pub active_clients: i32,

    /// Completed tasks count.
    #[serde(default)]
    pub completed_tasks: i32,

    /// Pending tasks count.
    #[serde(default)]
    pub pending_tasks: i32,

    /// Overdue tasks count.
    #[serde(default)]
    pub overdue_tasks: i32,

    /// Cash balance.
    #[serde(default)]
    pub cash_balance: Decimal,

    /// Accounts receivable.
    #[serde(default)]
    pub accounts_receivable: Decimal,

    /// Accounts payable.
    #[serde(default)]
    pub accounts_payable: Decimal,

    /// Compliance score (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance_score: Option<Decimal>,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Kpi {
    /// Create a new KPI snapshot.
    #[must_use]
    pub fn new(llc_id: NexId, snapshot_date: NaiveDate) -> Self {
        Self {
            id: NexId::v4(),
            llc_id,
            snapshot_date,
            created_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Calculate profit margin from current values.
    #[must_use]
    pub fn calculate_profit_margin(&self) -> Option<Decimal> {
        if self.total_revenue.is_zero() {
            None
        } else {
            Some((self.net_profit / self.total_revenue) * Decimal::from(100))
        }
    }

    /// Calculate working capital.
    #[must_use]
    pub fn working_capital(&self) -> Decimal {
        self.cash_balance + self.accounts_receivable - self.accounts_payable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llc_new() {
        let llc = Llc::new("Test LLC", "12-3456789", "DE");
        assert_eq!(llc.legal_name, "Test LLC");
        assert_eq!(llc.ein, "12-3456789");
        assert_eq!(llc.state_of_formation, "DE");
        assert!(llc.is_active);
    }

    #[test]
    fn test_transaction_signed_amount() {
        let llc_id = NexId::v4();
        let user_id = NexId::v4();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let revenue = Transaction::new(
            llc_id,
            user_id,
            TransactionType::Revenue,
            Decimal::from(1000),
            date,
            "Sale",
            "Revenue",
        );
        assert_eq!(revenue.signed_amount(), Decimal::from(1000));

        let expense = Transaction::new(
            llc_id,
            user_id,
            TransactionType::Expense,
            Decimal::from(500),
            date,
            "Purchase",
            "Supplies",
        );
        assert_eq!(expense.signed_amount(), Decimal::from(-500));
    }

    #[test]
    fn test_kpi_working_capital() {
        let llc_id = NexId::v4();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let mut kpi = Kpi::new(llc_id, date);
        kpi.cash_balance = Decimal::from(10000);
        kpi.accounts_receivable = Decimal::from(5000);
        kpi.accounts_payable = Decimal::from(3000);

        assert_eq!(kpi.working_capital(), Decimal::from(12000));
    }

    #[test]
    fn test_llc_full_address() {
        let mut llc = Llc::new("Test LLC", "12-3456789", "DE");
        assert!(llc.full_address().is_none());

        llc.address_line1 = Some("123 Main St".to_string());
        llc.city = Some("Wilmington".to_string());
        llc.state = Some("DE".to_string());
        llc.zip_code = Some("19801".to_string());

        assert_eq!(
            llc.full_address(),
            Some("123 Main St, Wilmington, DE, 19801".to_string())
        );
    }
}
