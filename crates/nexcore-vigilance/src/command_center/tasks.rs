//! Task domain types: tasks, documents, and compliance items.

use nexcore_chrono::{Date, DateTime};
use nexcore_id::NexId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::enums::{ComplianceStatus, DocumentStatus, RecurrenceType, TaskPriority, TaskStatus};

/// Task model with recurring task support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier.
    pub id: NexId,

    /// Associated LLC ID.
    pub llc_id: NexId,

    /// User who created the task.
    pub created_by: NexId,

    /// User assigned to the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<NexId>,

    /// Task title.
    pub title: String,

    /// Task description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Task status.
    #[serde(default)]
    pub status: TaskStatus,

    /// Task priority.
    #[serde(default)]
    pub priority: TaskPriority,

    /// Due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<Date>,

    /// When completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime>,

    /// Recurrence type.
    #[serde(default)]
    pub recurrence: RecurrenceType,

    /// Recurrence interval (every N periods).
    #[serde(default = "default_one")]
    pub recurrence_interval: i32,

    /// Day of week for weekly recurrence (0-6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_day_of_week: Option<i32>,

    /// Day of month for monthly recurrence (1-31).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_day_of_month: Option<i32>,

    /// Parent task ID for recurring instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<NexId>,

    /// Category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_one() -> i32 {
    1
}

impl Task {
    /// Create a new task.
    #[must_use]
    pub fn new(llc_id: NexId, created_by: NexId, title: impl Into<String>) -> Self {
        Self {
            id: NexId::v4(),
            llc_id,
            created_by,
            assigned_to: None,
            title: title.into(),
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            due_date: None,
            completed_at: None,
            recurrence: RecurrenceType::None,
            recurrence_interval: 1,
            recurrence_day_of_week: None,
            recurrence_day_of_month: None,
            parent_task_id: None,
            category: None,
            tags: Vec::new(),
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Check if task is overdue.
    #[must_use]
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            let today = DateTime::now().date();
            due_date < today && !self.is_completed()
        } else {
            false
        }
    }

    /// Check if task is completed.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    /// Check if task is recurring.
    #[must_use]
    pub fn is_recurring(&self) -> bool {
        self.recurrence != RecurrenceType::None
    }

    /// Mark task as completed.
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(DateTime::now());
        self.updated_at = Some(DateTime::now());
    }
}

/// Document model with expiry tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier.
    pub id: NexId,

    /// Associated LLC ID.
    pub llc_id: NexId,

    /// User who created the document.
    pub created_by: NexId,

    /// Document title.
    pub title: String,

    /// Document type (License, Certificate, etc.).
    pub document_type: String,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Document status.
    #[serde(default)]
    pub status: DocumentStatus,

    /// Issue date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<Date>,

    /// Expiry date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<Date>,

    /// Renewal date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renewal_date: Option<Date>,

    /// Issuing authority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuing_authority: Option<String>,

    /// Authority contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_contact: Option<String>,

    /// Authority website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_website: Option<String>,

    /// Cloud storage URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,

    /// Original filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,

    /// File size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,

    /// MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Days before expiry to send reminder.
    #[serde(default = "default_reminder_days")]
    pub reminder_days_before: i32,

    /// When last reminder was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reminder_sent_at: Option<DateTime>,

    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_reminder_days() -> i32 {
    30
}

impl Document {
    /// Create a new document.
    #[must_use]
    pub fn new(
        llc_id: NexId,
        created_by: NexId,
        title: impl Into<String>,
        document_type: impl Into<String>,
    ) -> Self {
        Self {
            id: NexId::v4(),
            llc_id,
            created_by,
            title: title.into(),
            document_type: document_type.into(),
            description: None,
            status: DocumentStatus::Active,
            issue_date: None,
            expiry_date: None,
            renewal_date: None,
            issuing_authority: None,
            authority_contact: None,
            authority_website: None,
            file_url: None,
            file_name: None,
            file_size: None,
            mime_type: None,
            reminder_days_before: 30,
            last_reminder_sent_at: None,
            tags: Vec::new(),
            notes: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Calculate days until expiry.
    #[must_use]
    pub fn days_until_expiry(&self) -> Option<i64> {
        self.expiry_date.map(|expiry| {
            let today = DateTime::now().date();
            (expiry - today).num_days()
        })
    }

    /// Check if document is expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.days_until_expiry().is_some_and(|days| days < 0)
    }

    /// Check if document is expiring soon (within reminder window).
    #[must_use]
    pub fn is_expiring_soon(&self) -> bool {
        self.days_until_expiry()
            .is_some_and(|days| days >= 0 && days <= i64::from(self.reminder_days_before))
    }

    /// Update status based on expiry date.
    pub fn update_status(&mut self) {
        if self.is_expired() {
            self.status = DocumentStatus::Expired;
        } else if self.is_expiring_soon() {
            self.status = DocumentStatus::ExpiringSoon;
        }
        self.updated_at = Some(DateTime::now());
    }
}

/// Compliance item model for regulatory tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceItem {
    /// Unique identifier.
    pub id: NexId,

    /// Associated LLC ID.
    pub llc_id: NexId,

    /// User who created the item.
    pub created_by: NexId,

    /// Title.
    pub title: String,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Compliance status.
    #[serde(default)]
    pub status: ComplianceStatus,

    /// Compliance type (Tax, Annual Report, License, etc.).
    pub compliance_type: String,

    /// Regulatory authority (IRS, State, County, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulatory_authority: Option<String>,

    /// Requirement level (Federal, State, Local).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_level: Option<String>,

    /// Due date.
    pub due_date: Date,

    /// When filed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filed_date: Option<Date>,

    /// Next due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_due_date: Option<Date>,

    /// Whether recurring.
    #[serde(default)]
    pub is_recurring: bool,

    /// Recurrence frequency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_frequency: Option<String>,

    /// Form number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_number: Option<String>,

    /// Filing method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filing_method: Option<String>,

    /// Filing URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filing_url: Option<String>,

    /// Filing fee.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filing_fee: Option<Decimal>,

    /// Penalty amount if late.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penalty_amount: Option<Decimal>,

    /// Whether complete.
    #[serde(default)]
    pub is_complete: bool,

    /// Completion notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_notes: Option<String>,

    /// Days before due to send reminder.
    #[serde(default = "default_reminder_days")]
    pub reminder_days_before: i32,

    /// When last reminder was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reminder_sent_at: Option<DateTime>,

    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

impl ComplianceItem {
    /// Create a new compliance item.
    #[must_use]
    pub fn new(
        llc_id: NexId,
        created_by: NexId,
        title: impl Into<String>,
        compliance_type: impl Into<String>,
        due_date: Date,
    ) -> Self {
        Self {
            id: NexId::v4(),
            llc_id,
            created_by,
            title: title.into(),
            description: None,
            status: ComplianceStatus::Compliant,
            compliance_type: compliance_type.into(),
            regulatory_authority: None,
            requirement_level: None,
            due_date,
            filed_date: None,
            next_due_date: None,
            is_recurring: false,
            recurrence_frequency: None,
            form_number: None,
            filing_method: None,
            filing_url: None,
            filing_fee: None,
            penalty_amount: None,
            is_complete: false,
            completion_notes: None,
            reminder_days_before: 30,
            last_reminder_sent_at: None,
            tags: Vec::new(),
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Calculate days until due.
    #[must_use]
    pub fn days_until_due(&self) -> i64 {
        let today = DateTime::now().date();
        (self.due_date - today).num_days()
    }

    /// Check if item is overdue.
    #[must_use]
    pub fn is_overdue(&self) -> bool {
        self.days_until_due() < 0 && !self.is_complete
    }

    /// Check if item is due soon.
    #[must_use]
    pub fn is_due_soon(&self) -> bool {
        let days = self.days_until_due();
        days >= 0 && days <= i64::from(self.reminder_days_before) && !self.is_complete
    }

    /// Update status based on due date.
    pub fn update_status(&mut self) {
        if self.is_complete {
            self.status = ComplianceStatus::Filed;
        } else if self.is_overdue() {
            self.status = ComplianceStatus::Overdue;
        } else if self.is_due_soon() {
            self.status = ComplianceStatus::DueSoon;
        } else {
            self.status = ComplianceStatus::Compliant;
        }
        self.updated_at = Some(DateTime::now());
    }

    /// Mark as filed.
    pub fn mark_filed(&mut self) {
        self.is_complete = true;
        self.filed_date = Some(DateTime::now().date());
        self.status = ComplianceStatus::Filed;
        self.updated_at = Some(DateTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new() {
        let llc_id = NexId::v4();
        let user_id = NexId::v4();

        let task = Task::new(llc_id, user_id, "Test Task");
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(!task.is_recurring());
    }

    #[test]
    fn test_task_complete() {
        let llc_id = NexId::v4();
        let user_id = NexId::v4();

        let mut task = Task::new(llc_id, user_id, "Test Task");
        assert!(!task.is_completed());

        task.complete();
        assert!(task.is_completed());
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_document_expiry() {
        let llc_id = NexId::v4();
        let user_id = NexId::v4();

        let mut doc = Document::new(llc_id, user_id, "License", "Business License");

        // Set expiry to 10 days from now
        let expiry = DateTime::now().date() + nexcore_chrono::Duration::days(10);
        doc.expiry_date = Some(expiry);

        assert!(!doc.is_expired());
        assert!(doc.is_expiring_soon()); // Within 30-day window

        // Set expiry to 60 days from now
        let expiry = DateTime::now().date() + nexcore_chrono::Duration::days(60);
        doc.expiry_date = Some(expiry);

        assert!(!doc.is_expired());
        assert!(!doc.is_expiring_soon()); // Outside 30-day window
    }

    #[test]
    fn test_compliance_item_status() {
        let llc_id = NexId::v4();
        let user_id = NexId::v4();
        let due_date = DateTime::now().date() + nexcore_chrono::Duration::days(15);

        let mut item =
            ComplianceItem::new(llc_id, user_id, "Annual Report", "Annual Report", due_date);

        assert!(item.is_due_soon()); // Within 30-day window

        item.mark_filed();
        assert!(item.is_complete);
        assert_eq!(item.status, ComplianceStatus::Filed);
    }
}
