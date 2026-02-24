//! Persistence layer for nexcore API
//!
//! Provides durable storage (π Persistence) for user settings,
//! reports, and system state.

pub mod firestore;

#[cfg(test)]
mod tests;

use crate::persistence::firestore::{FirestorePersistence, MockPersistence};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Record representing a telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEventRecord {
    pub id: String,
    pub event_type: String,
    pub user_id: String,
    pub metadata: serde_json::Value,
    pub timestamp: String,
}

/// Record representing a persisted report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: String,
    pub report_type: String,
    pub generated_at: DateTime<Utc>,
    pub content: String,
    pub status: String,
    pub user_id: Option<String>,
}

/// Record representing a community post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRecord {
    pub id: String,
    pub author: String,
    pub role: String,
    pub content: String,
    pub likes: u32,
    pub replies: u32,
    pub created_at: DateTime<Utc>,
}

/// Record representing a partnership inquiry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquiryRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub organization: String,
    pub interest: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
}

/// Record representing a course enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentRecord {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub progress: f64,
    pub enrolled_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Record representing a professional Circle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: u32,
    pub post_count: u32,
    pub created_at: DateTime<Utc>,
}

/// Record representing a Circle membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipRecord {
    pub id: String,
    pub user_id: String,
    pub circle_id: String,
    pub joined_at: DateTime<Utc>,
}

/// Record representing a private message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

/// Record representing a KSB (Knowledge, Skill, Behavior)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbRecord {
    pub id: String,
    pub ksb_type: String,
    pub title: String,
    pub description: String,
    pub domain: String,
}

/// Record representing a KSB Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbDomainRecord {
    pub code: String,
    pub name: String,
    pub ksb_count: u32,
    pub dominant_primitive: String,
    pub cognitive_primitive: String,
    pub transfer_confidence: f64,
    pub pvos_layer: Option<String>,
    pub example_ksbs: Vec<String>,
}

/// Unified persistence backend
pub enum Persistence {
    Firestore(FirestorePersistence),
    Mock(MockPersistence),
}

impl Persistence {
    pub async fn save_report(&self, report: &ReportRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_report(report).await,
            Self::Mock(m) => m.save_report(report).await,
        }
    }

    pub async fn list_reports(&self) -> nexcore_error::Result<Vec<ReportRecord>> {
        match self {
            Self::Firestore(f) => f.list_reports().await,
            Self::Mock(m) => m.list_reports().await,
        }
    }

    pub async fn save_post(&self, post: &PostRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_post(post).await,
            Self::Mock(m) => m.save_post(post).await,
        }
    }

    pub async fn list_posts(&self) -> nexcore_error::Result<Vec<PostRecord>> {
        match self {
            Self::Firestore(f) => f.list_posts().await,
            Self::Mock(m) => m.list_posts().await,
        }
    }

    pub async fn save_inquiry(&self, inquiry: &InquiryRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_inquiry(inquiry).await,
            Self::Mock(m) => m.save_inquiry(inquiry).await,
        }
    }

    pub async fn list_inquiries(&self) -> nexcore_error::Result<Vec<InquiryRecord>> {
        match self {
            Self::Firestore(f) => f.list_inquiries().await,
            Self::Mock(m) => m.list_inquiries().await,
        }
    }

    pub async fn update_inquiry_status(&self, id: &str, status: &str) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.update_inquiry_status(id, status).await,
            Self::Mock(m) => m.update_inquiry_status(id, status).await,
        }
    }

    pub async fn save_enrollment(
        &self,
        enrollment: &EnrollmentRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_enrollment(enrollment).await,
            Self::Mock(m) => m.save_enrollment(enrollment).await,
        }
    }

    pub async fn list_enrollments(&self) -> nexcore_error::Result<Vec<EnrollmentRecord>> {
        match self {
            Self::Firestore(f) => f.list_enrollments().await,
            Self::Mock(m) => m.list_enrollments().await,
        }
    }

    pub async fn save_circle(&self, circle: &CircleRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_circle(circle).await,
            Self::Mock(m) => m.save_circle(circle).await,
        }
    }

    pub async fn list_circles(&self) -> nexcore_error::Result<Vec<CircleRecord>> {
        match self {
            Self::Firestore(f) => f.list_circles().await,
            Self::Mock(m) => m.list_circles().await,
        }
    }

    pub async fn save_membership(
        &self,
        membership: &MembershipRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_membership(membership).await,
            Self::Mock(m) => m.save_membership(membership).await,
        }
    }

    pub async fn list_memberships(
        &self,
        user_id: &str,
    ) -> nexcore_error::Result<Vec<MembershipRecord>> {
        match self {
            Self::Firestore(f) => f.list_memberships(user_id).await,
            Self::Mock(m) => m.list_memberships(user_id).await,
        }
    }

    pub async fn save_message(&self, message: &MessageRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_message(message).await,
            Self::Mock(m) => m.save_message(message).await,
        }
    }

    pub async fn list_messages(&self, user_id: &str) -> nexcore_error::Result<Vec<MessageRecord>> {
        match self {
            Self::Firestore(f) => f.list_messages(user_id).await,
            Self::Mock(m) => m.list_messages(user_id).await,
        }
    }

    pub async fn save_ksb_domain(&self, domain: &KsbDomainRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_ksb_domain(domain).await,
            Self::Mock(m) => m.save_ksb_domain(domain).await,
        }
    }

    pub async fn list_ksb_domains(&self) -> nexcore_error::Result<Vec<KsbDomainRecord>> {
        match self {
            Self::Firestore(f) => f.list_ksb_domains().await,
            Self::Mock(m) => m.list_ksb_domains().await,
        }
    }

    pub async fn save_telemetry_event(
        &self,
        event: &TelemetryEventRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_telemetry_event(event).await,
            Self::Mock(m) => m.save_telemetry_event(event).await,
        }
    }

    pub async fn list_telemetry_events(&self) -> nexcore_error::Result<Vec<TelemetryEventRecord>> {
        match self {
            Self::Firestore(f) => f.list_telemetry_events().await,
            Self::Mock(m) => m.list_telemetry_events().await,
        }
    }
}
