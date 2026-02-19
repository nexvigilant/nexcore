//! History query — filter and search past pipeline runs.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: κ (Comparison) | Filter by status, date range |
//! | T1: σ (Sequence) | Sorted results |

use crate::error::BuildOrcResult;
use crate::history::store::HistoryStore;
use crate::pipeline::state::{PipelineRunState, RunStatus};

/// Query parameters for history search.
#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    /// Filter by pipeline status.
    pub status: Option<RunStatus>,
    /// Filter by pipeline definition name.
    pub definition: Option<String>,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Filter: runs after this time.
    pub after: Option<chrono::DateTime<chrono::Utc>>,
    /// Filter: runs before this time.
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

impl HistoryQuery {
    /// Create a query with no filters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by status.
    #[must_use]
    pub fn with_status(mut self, status: RunStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filter by definition name.
    #[must_use]
    pub fn with_definition(mut self, name: impl Into<String>) -> Self {
        self.definition = Some(name.into());
        self
    }

    /// Limit results.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Execute a query against the history store.
pub fn query_history(
    store: &HistoryStore,
    query: &HistoryQuery,
) -> BuildOrcResult<Vec<PipelineRunState>> {
    let all_runs = store.load_all()?;

    let filtered: Vec<PipelineRunState> = all_runs
        .into_iter()
        .filter(|run| {
            if let Some(status) = query.status {
                if run.status != status {
                    return false;
                }
            }
            if let Some(ref def) = query.definition {
                if run.definition_name != *def {
                    return false;
                }
            }
            if let Some(after) = query.after {
                if run.started_at < after {
                    return false;
                }
            }
            if let Some(before) = query.before {
                if run.started_at > before {
                    return false;
                }
            }
            true
        })
        .take(query.limit.unwrap_or(usize::MAX))
        .collect();

    Ok(filtered)
}
