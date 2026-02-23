//! Firestore persistence implementation

use crate::persistence::{
    CircleRecord, EnrollmentRecord, InquiryRecord, KsbDomainRecord, MembershipRecord,
    MessageRecord, PostRecord, ReportRecord,
};
use serde_json::json;

/// Firestore REST API base URL
const FIRESTORE_BASE: &str = "https://firestore.googleapis.com/v1";

pub struct FirestorePersistence {
    project_id: String,
    collection: String, // Base collection, but we'll use specific ones below
    http: reqwest::Client,
    api_key: Option<String>,
}

impl FirestorePersistence {
    pub fn new(project_id: String, _collection: String, api_key: Option<String>) -> Self {
        Self {
            project_id,
            collection: "nexcore".to_string(), // Root collection
            http: reqwest::Client::new(),
            api_key,
        }
    }

    fn url(&self, collection: &str, doc_id: Option<&str>) -> String {
        let base = format!(
            "{FIRESTORE_BASE}/projects/{}/databases/(default)/documents/{}",
            self.project_id, collection
        );
        match doc_id {
            Some(id) => format!("{}/{}", base, id),
            None => base,
        }
    }

    pub async fn save_report(&self, report: &ReportRecord) -> nexcore_error::Result<()> {
        let url = self.url("reports", Some(&report.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": report.id },
                "report_type": { "stringValue": report.report_type },
                "generated_at": { "stringValue": report.generated_at.to_rfc3339() },
                "content": { "stringValue": report.content },
                "status": { "stringValue": report.status },
                "user_id": { "stringValue": report.user_id.clone().unwrap_or_default() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_reports(&self) -> nexcore_error::Result<Vec<ReportRecord>> {
        let url = self.url("reports", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_report_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_post(&self, post: &PostRecord) -> nexcore_error::Result<()> {
        let url = self.url("posts", Some(&post.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": post.id },
                "author": { "stringValue": post.author },
                "role": { "stringValue": post.role },
                "content": { "stringValue": post.content },
                "likes": { "integerValue": post.likes.to_string() },
                "replies": { "integerValue": post.replies.to_string() },
                "created_at": { "stringValue": post.created_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_posts(&self) -> nexcore_error::Result<Vec<PostRecord>> {
        let url = self.url("posts", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_post_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_inquiry(&self, inquiry: &InquiryRecord) -> nexcore_error::Result<()> {
        let url = self.url("inquiries", Some(&inquiry.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": inquiry.id },
                "name": { "stringValue": inquiry.name },
                "email": { "stringValue": inquiry.email },
                "organization": { "stringValue": inquiry.organization },
                "interest": { "stringValue": inquiry.interest },
                "message": { "stringValue": inquiry.message },
                "created_at": { "stringValue": inquiry.created_at.to_rfc3339() },
                "status": { "stringValue": inquiry.status },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_inquiries(&self) -> nexcore_error::Result<Vec<InquiryRecord>> {
        let url = self.url("inquiries", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_inquiry_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn update_inquiry_status(&self, id: &str, status: &str) -> nexcore_error::Result<()> {
        let url = self.url("inquiries", Some(id));
        let body = json!({
            "fields": {
                "status": { "stringValue": status },
            }
        });

        // Use mask to only update the status field
        let url_with_mask = format!("{}?updateMask.fieldPaths=status", url);
        self.patch(&url_with_mask, &body).await
    }

    pub async fn save_enrollment(&self, enrollment: &EnrollmentRecord) -> nexcore_error::Result<()> {
        let url = self.url("enrollments", Some(&enrollment.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": enrollment.id },
                "user_id": { "stringValue": enrollment.user_id },
                "course_id": { "stringValue": enrollment.course_id },
                "progress": { "doubleValue": enrollment.progress },
                "enrolled_at": { "stringValue": enrollment.enrolled_at.to_rfc3339() },
                "completed_at": { "stringValue": enrollment.completed_at.map(|d| d.to_rfc3339()).unwrap_or_default() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_enrollments(&self) -> nexcore_error::Result<Vec<EnrollmentRecord>> {
        let url = self.url("enrollments", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_enrollment_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_circle(&self, circle: &CircleRecord) -> nexcore_error::Result<()> {
        let url = self.url("circles", Some(&circle.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": circle.id },
                "name": { "stringValue": circle.name },
                "description": { "stringValue": circle.description },
                "member_count": { "integerValue": circle.member_count.to_string() },
                "post_count": { "integerValue": circle.post_count.to_string() },
                "created_at": { "stringValue": circle.created_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_circles(&self) -> nexcore_error::Result<Vec<CircleRecord>> {
        let url = self.url("circles", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_circle_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_membership(&self, membership: &MembershipRecord) -> nexcore_error::Result<()> {
        let url = self.url("memberships", Some(&membership.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": membership.id },
                "user_id": { "stringValue": membership.user_id },
                "circle_id": { "stringValue": membership.circle_id },
                "joined_at": { "stringValue": membership.joined_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_memberships(&self, _user_id: &str) -> nexcore_error::Result<Vec<MembershipRecord>> {
        let url = self.url("memberships", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_membership_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_message(&self, message: &MessageRecord) -> nexcore_error::Result<()> {
        let url = self.url("messages", Some(&message.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": message.id },
                "sender_id": { "stringValue": message.sender_id },
                "recipient_id": { "stringValue": message.recipient_id },
                "content": { "stringValue": message.content },
                "created_at": { "stringValue": message.created_at.to_rfc3339() },
                "read": { "booleanValue": message.read },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_messages(&self, _user_id: &str) -> nexcore_error::Result<Vec<MessageRecord>> {
        let url = self.url("messages", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_message_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_ksb_domain(&self, domain: &KsbDomainRecord) -> nexcore_error::Result<()> {
        let url = self.url("ksb_domains", Some(&domain.code));
        let body = json!({
            "fields": {
                "code": { "stringValue": domain.code },
                "name": { "stringValue": domain.name },
                "ksb_count": { "integerValue": domain.ksb_count.to_string() },
                "dominant_primitive": { "stringValue": domain.dominant_primitive },
                "cognitive_primitive": { "stringValue": domain.cognitive_primitive },
                "transfer_confidence": { "doubleValue": domain.transfer_confidence },
                "pvos_layer": { "stringValue": domain.pvos_layer.clone().unwrap_or_default() },
                "example_ksbs": { "arrayValue": { "values": domain.example_ksbs.iter().map(|k| json!({ "stringValue": k })).collect::<Vec<_>>() } }
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_ksb_domains(&self) -> nexcore_error::Result<Vec<KsbDomainRecord>> {
        let url = self.url("ksb_domains", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_ksb_domain_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    async fn patch(&self, url: &str, body: &serde_json::Value) -> nexcore_error::Result<()> {
        let mut req = self.http.patch(url).json(body);
        if let Some(ref key) = self.api_key {
            req = req.query(&[("key", key)]);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            nexcore_error::bail!("Firestore error: {}", err_text);
        }
        Ok(())
    }

    async fn get_list(&self, url: &str) -> nexcore_error::Result<Vec<serde_json::Value>> {
        let mut req = self.http.get(url);
        if let Some(ref key) = self.api_key {
            req = req.query(&[("key", key)]);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            nexcore_error::bail!("Firestore error: {}", err_text);
        }
        let body: serde_json::Value = resp.json().await?;
        Ok(body
            .get("documents")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default())
    }
}

fn parse_report_doc(doc: &serde_json::Value) -> nexcore_error::Result<ReportRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(ReportRecord {
        id: get_string(fields, "id"),
        report_type: get_string(fields, "report_type"),
        generated_at: get_timestamp(fields, "generated_at"),
        content: get_string(fields, "content"),
        status: get_string(fields, "status"),
        user_id: fields
            .get("user_id")
            .and_then(|v| v.get("stringValue"))
            .and_then(|s| s.as_str())
            .map(String::from),
    })
}

fn parse_post_doc(doc: &serde_json::Value) -> nexcore_error::Result<PostRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(PostRecord {
        id: get_string(fields, "id"),
        author: get_string(fields, "author"),
        role: get_string(fields, "role"),
        content: get_string(fields, "content"),
        likes: get_int(fields, "likes"),
        replies: get_int(fields, "replies"),
        created_at: get_timestamp(fields, "created_at"),
    })
}

fn parse_inquiry_doc(doc: &serde_json::Value) -> nexcore_error::Result<InquiryRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(InquiryRecord {
        id: get_string(fields, "id"),
        name: get_string(fields, "name"),
        email: get_string(fields, "email"),
        organization: get_string(fields, "organization"),
        interest: get_string(fields, "interest"),
        message: get_string(fields, "message"),
        created_at: get_timestamp(fields, "created_at"),
        status: get_string(fields, "status"),
    })
}

fn parse_enrollment_doc(doc: &serde_json::Value) -> nexcore_error::Result<EnrollmentRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(EnrollmentRecord {
        id: get_string(fields, "id"),
        user_id: get_string(fields, "user_id"),
        course_id: get_string(fields, "course_id"),
        progress: fields
            .get("progress")
            .and_then(|v| v.get("doubleValue"))
            .and_then(|n| n.as_f64())
            .unwrap_or(0.0),
        enrolled_at: get_timestamp(fields, "enrolled_at"),
        completed_at: fields
            .get("completed_at")
            .and_then(|v| v.get("stringValue"))
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse().ok()),
    })
}

fn parse_circle_doc(doc: &serde_json::Value) -> nexcore_error::Result<CircleRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(CircleRecord {
        id: get_string(fields, "id"),
        name: get_string(fields, "name"),
        description: get_string(fields, "description"),
        member_count: get_int(fields, "member_count"),
        post_count: get_int(fields, "post_count"),
        created_at: get_timestamp(fields, "created_at"),
    })
}

fn parse_membership_doc(doc: &serde_json::Value) -> nexcore_error::Result<MembershipRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(MembershipRecord {
        id: get_string(fields, "id"),
        user_id: get_string(fields, "user_id"),
        circle_id: get_string(fields, "circle_id"),
        joined_at: get_timestamp(fields, "joined_at"),
    })
}

fn parse_message_doc(doc: &serde_json::Value) -> nexcore_error::Result<MessageRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(MessageRecord {
        id: get_string(fields, "id"),
        sender_id: get_string(fields, "sender_id"),
        recipient_id: get_string(fields, "recipient_id"),
        content: get_string(fields, "content"),
        created_at: get_timestamp(fields, "created_at"),
        read: fields
            .get("read")
            .and_then(|v| v.get("booleanValue"))
            .and_then(|b| b.as_bool())
            .unwrap_or(false),
    })
}

fn parse_ksb_domain_doc(doc: &serde_json::Value) -> nexcore_error::Result<KsbDomainRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    let examples = fields
        .get("example_ksbs")
        .and_then(|v| v.get("arrayValue"))
        .and_then(|a| a.get("values"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| {
                    val.get("stringValue")
                        .and_then(|s| s.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(KsbDomainRecord {
        code: get_string(fields, "code"),
        name: get_string(fields, "name"),
        ksb_count: get_int(fields, "ksb_count"),
        dominant_primitive: get_string(fields, "dominant_primitive"),
        cognitive_primitive: get_string(fields, "cognitive_primitive"),
        transfer_confidence: fields
            .get("transfer_confidence")
            .and_then(|v| v.get("doubleValue"))
            .and_then(|n| n.as_f64())
            .unwrap_or(0.0),
        pvos_layer: fields
            .get("pvos_layer")
            .and_then(|v| v.get("stringValue"))
            .and_then(|s| s.as_str())
            .map(String::from),
        example_ksbs: examples,
    })
}

fn get_string(fields: &serde_json::Value, key: &str) -> String {
    fields
        .get(key)
        .and_then(|v| v.get("stringValue"))
        .and_then(|s| s.as_str())
        .unwrap_or_default()
        .to_string()
}

fn get_timestamp(fields: &serde_json::Value, key: &str) -> chrono::DateTime<chrono::Utc> {
    fields
        .get(key)
        .and_then(|v| v.get("stringValue"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(chrono::Utc::now)
}

fn get_int(fields: &serde_json::Value, key: &str) -> u32 {
    fields
        .get(key)
        .and_then(|v| v.get("integerValue"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Mock implementation for local development
pub struct MockPersistence {
    reports: std::sync::Arc<std::sync::Mutex<Vec<ReportRecord>>>,
    posts: std::sync::Arc<std::sync::Mutex<Vec<PostRecord>>>,
    inquiries: std::sync::Arc<std::sync::Mutex<Vec<InquiryRecord>>>,
    enrollments: std::sync::Arc<std::sync::Mutex<Vec<EnrollmentRecord>>>,
    circles: std::sync::Arc<std::sync::Mutex<Vec<CircleRecord>>>,
    memberships: std::sync::Arc<std::sync::Mutex<Vec<MembershipRecord>>>,
    messages: std::sync::Arc<std::sync::Mutex<Vec<MessageRecord>>>,
    ksb_domains: std::sync::Arc<std::sync::Mutex<Vec<KsbDomainRecord>>>,
}

impl MockPersistence {
    pub fn new() -> Self {
        Self {
            reports: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            posts: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            inquiries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            enrollments: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            circles: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            memberships: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            messages: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            ksb_domains: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn save_report(&self, report: &ReportRecord) -> nexcore_error::Result<()> {
        let mut reports = self
            .reports
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        reports.push(report.clone());
        Ok(())
    }

    pub async fn list_reports(&self) -> nexcore_error::Result<Vec<ReportRecord>> {
        let reports = self
            .reports
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(reports.clone())
    }

    pub async fn save_post(&self, post: &PostRecord) -> nexcore_error::Result<()> {
        let mut posts = self
            .posts
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        posts.push(post.clone());
        Ok(())
    }

    pub async fn list_posts(&self) -> nexcore_error::Result<Vec<PostRecord>> {
        let posts = self
            .posts
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(posts.clone())
    }

    pub async fn save_inquiry(&self, inquiry: &InquiryRecord) -> nexcore_error::Result<()> {
        let mut inquiries = self
            .inquiries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        inquiries.push(inquiry.clone());
        Ok(())
    }

    pub async fn list_inquiries(&self) -> nexcore_error::Result<Vec<InquiryRecord>> {
        let inquiries = self
            .inquiries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(inquiries.clone())
    }

    pub async fn update_inquiry_status(&self, id: &str, status: &str) -> nexcore_error::Result<()> {
        let mut inquiries = self
            .inquiries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(inquiry) = inquiries.iter_mut().find(|i| i.id == id) {
            inquiry.status = status.to_string();
            Ok(())
        } else {
            nexcore_error::bail!("Inquiry not found")
        }
    }

    pub async fn save_enrollment(&self, enrollment: &EnrollmentRecord) -> nexcore_error::Result<()> {
        let mut enrollments = self
            .enrollments
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        enrollments.push(enrollment.clone());
        Ok(())
    }

    pub async fn list_enrollments(&self) -> nexcore_error::Result<Vec<EnrollmentRecord>> {
        let enrollments = self
            .enrollments
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(enrollments.clone())
    }

    pub async fn save_circle(&self, circle: &CircleRecord) -> nexcore_error::Result<()> {
        let mut circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        circles.push(circle.clone());
        Ok(())
    }

    pub async fn list_circles(&self) -> nexcore_error::Result<Vec<CircleRecord>> {
        let circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(circles.clone())
    }

    pub async fn save_membership(&self, membership: &MembershipRecord) -> nexcore_error::Result<()> {
        let mut memberships = self
            .memberships
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        memberships.push(membership.clone());
        Ok(())
    }

    pub async fn list_memberships(&self, user_id: &str) -> nexcore_error::Result<Vec<MembershipRecord>> {
        let memberships = self
            .memberships
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(memberships
            .iter()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    pub async fn save_message(&self, message: &MessageRecord) -> nexcore_error::Result<()> {
        let mut messages = self
            .messages
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        messages.push(message.clone());
        Ok(())
    }

    pub async fn list_messages(&self, user_id: &str) -> nexcore_error::Result<Vec<MessageRecord>> {
        let messages = self
            .messages
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(messages
            .iter()
            .filter(|m| m.sender_id == user_id || m.recipient_id == user_id)
            .cloned()
            .collect())
    }

    pub async fn save_ksb_domain(&self, domain: &KsbDomainRecord) -> nexcore_error::Result<()> {
        let mut domains = self
            .ksb_domains
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        domains.push(domain.clone());
        Ok(())
    }

    pub async fn list_ksb_domains(&self) -> nexcore_error::Result<Vec<KsbDomainRecord>> {
        let domains = self
            .ksb_domains
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(domains.clone())
    }
}

impl Default for MockPersistence {
    fn default() -> Self {
        Self::new()
    }
}
