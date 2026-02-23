//! Firestore REST API client
//!
//! Provides typed access to Firestore documents and collections
//! via the REST API. Used for server-side data fetching.

use serde::de::DeserializeOwned;

/// Firestore REST API base URL template
const FIRESTORE_BASE: &str = "https://firestore.googleapis.com/v1";

/// Default Firebase project ID
const PROJECT_ID: &str = "nexvigilant-digital-clubhouse";

/// Firestore query filter operator
#[derive(Debug, Clone)]
pub enum FilterOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    ArrayContains,
    In,
}

/// Firestore query filter
#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub op: FilterOp,
    pub value: serde_json::Value,
}

/// Firestore REST client (requires `client` feature)
#[cfg(feature = "client")]
pub struct FirestoreClient {
    project_id: String,
    http: reqwest::Client,
}

#[cfg(feature = "client")]
impl FirestoreClient {
    /// Create client with default project
    pub fn new() -> Self {
        Self {
            project_id: PROJECT_ID.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Create client with custom project ID
    pub fn with_project(project_id: String) -> Self {
        Self {
            project_id,
            http: reqwest::Client::new(),
        }
    }

    /// Get a single document by collection and document ID
    pub async fn get_document<T: DeserializeOwned>(
        &self,
        collection: &str,
        doc_id: &str,
        token: &str,
    ) -> Result<T, String> {
        let url = format!(
            "{FIRESTORE_BASE}/projects/{}/databases/(default)/documents/{}/{}",
            self.project_id, collection, doc_id
        );

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;

        if resp.status().is_success() {
            let doc: serde_json::Value =
                resp.json().await.map_err(|e| format!("Parse error: {e}"))?;
            parse_firestore_document(&doc)
        } else {
            Err(format!("Firestore error: {}", resp.status()))
        }
    }

    /// List documents in a collection
    pub async fn list_documents<T: DeserializeOwned>(
        &self,
        collection: &str,
        token: &str,
        page_size: Option<u32>,
    ) -> Result<Vec<T>, String> {
        let mut url = format!(
            "{FIRESTORE_BASE}/projects/{}/databases/(default)/documents/{}",
            self.project_id, collection
        );

        if let Some(size) = page_size {
            url.push_str(&format!("?pageSize={size}"));
        }

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;

        if resp.status().is_success() {
            let body: serde_json::Value =
                resp.json().await.map_err(|e| format!("Parse error: {e}"))?;

            let documents = body
                .get("documents")
                .and_then(|d| d.as_array())
                .cloned()
                .unwrap_or_default();

            documents.iter().map(parse_firestore_document).collect()
        } else {
            Err(format!("Firestore error: {}", resp.status()))
        }
    }
}

#[cfg(feature = "client")]
impl Default for FirestoreClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a Firestore document JSON into a typed struct.
///
/// Firestore REST API returns documents in a specific format with
/// typed value wrappers (`stringValue`, `integerValue`, etc.).
/// This function extracts the fields and converts them to a flat JSON object.
pub fn parse_firestore_document<T: DeserializeOwned>(doc: &serde_json::Value) -> Result<T, String> {
    let fields = doc.get("fields").ok_or("Document missing 'fields'")?;

    let flat = convert_firestore_fields(fields);
    serde_json::from_value(flat).map_err(|e| format!("Deserialization error: {e}"))
}

/// Convert Firestore typed fields to plain JSON values
pub fn convert_firestore_fields(fields: &serde_json::Value) -> serde_json::Value {
    match fields {
        serde_json::Value::Object(map) => {
            let converted: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), convert_firestore_value(v)))
                .collect();
            serde_json::Value::Object(converted)
        }
        other => other.clone(),
    }
}

/// Convert a single Firestore typed value to a plain JSON value
pub fn convert_firestore_value(val: &serde_json::Value) -> serde_json::Value {
    if let Some(s) = val.get("stringValue") {
        return s.clone();
    }
    if let Some(n) = val.get("integerValue") {
        // Firestore returns integers as strings
        if let Some(s) = n.as_str() {
            if let Ok(i) = s.parse::<i64>() {
                return serde_json::Value::Number(i.into());
            }
        }
        return n.clone();
    }
    if let Some(b) = val.get("booleanValue") {
        return b.clone();
    }
    if let Some(d) = val.get("doubleValue") {
        return d.clone();
    }
    if let Some(arr) = val
        .get("arrayValue")
        .and_then(|a| a.get("values"))
        .and_then(|v| v.as_array())
    {
        let items: Vec<serde_json::Value> = arr.iter().map(convert_firestore_value).collect();
        return serde_json::Value::Array(items);
    }
    if let Some(map) = val.get("mapValue").and_then(|m| m.get("fields")) {
        return convert_firestore_fields(map);
    }
    if val.get("nullValue").is_some() {
        return serde_json::Value::Null;
    }
    if let Some(ts) = val.get("timestampValue") {
        return ts.clone();
    }
    // Fallback
    val.clone()
}
