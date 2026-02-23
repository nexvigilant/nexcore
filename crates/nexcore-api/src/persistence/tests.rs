#[cfg(test)]
mod tests {
    use crate::persistence::{Persistence, ReportRecord, firestore::MockPersistence};
    use chrono::Utc;

    #[tokio::test]
    async fn test_mock_persistence() -> nexcore_error::Result<()> {
        let persistence = MockPersistence::new();

        let report = ReportRecord {
            id: "test-id".to_string(),
            report_type: "signal_summary".to_string(),
            generated_at: Utc::now(),
            content: "Test content".to_string(),
            status: "completed".to_string(),
            user_id: None,
        };

        // Save
        persistence.save_report(&report).await?;

        // List
        let reports = persistence.list_reports().await?;
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].id, "test-id");
        assert_eq!(reports[0].content, "Test content");

        Ok(())
    }
}
