use crate::SignalAnalysisResult;
use duckdb::{params, Connection, Result};
use uuid::Uuid;

pub struct PersistenceManager {
    conn: Connection,
}

impl PersistenceManager {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let conn = open_connection(path)?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }

    pub fn save_result(&self, sam: &SignalAnalysisResult) -> Result<()> {
        self.conn.execute(
            "INSERT INTO signals (id, drug_name, event_name, timestamp, prr, ror, ebgm, risk_level)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                sam.id.to_string(),
                sam.drug_name,
                sam.event_name,
                sam.timestamp,
                sam.metrics.prr,
                sam.metrics.ror,
                sam.metrics.ebgm,
                sam.risk_level
            ],
        )?;
        Ok(())
    }

    pub fn get_recent_signals(&self, limit: usize) -> Result<Vec<SignalAnalysisResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, drug_name, event_name, timestamp, prr, ror, ebgm, risk_level 
             FROM signals ORDER BY timestamp DESC LIMIT ?",
        )?;

        let rows = stmt.query_map(params![limit], map_row_to_sam)?;

        let mut results = Vec::new();
        for row_res in rows {
            results.push(row_res?);
        }
        Ok(results)
    }
}

fn open_connection(path: Option<&str>) -> Result<Connection> {
    match path {
        Some(p) => Connection::open(p),
        None => Connection::open_in_memory(),
    }
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS signals (
            id TEXT PRIMARY KEY,
            drug_name TEXT,
            event_name TEXT,
            timestamp TIMESTAMP,
            prr DOUBLE,
            ror DOUBLE,
            ebgm DOUBLE,
            risk_level TEXT
        )",
        [],
    )?;
    Ok(())
}

fn map_row_to_sam(row: &duckdb::Row<'_>) -> Result<SignalAnalysisResult> {
    let id_str: String = row.get(0)?;
    let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());

    Ok(SignalAnalysisResult {
        id,
        drug_name: row.get(1)?,
        event_name: row.get(2)?,
        timestamp: row.get(3)?,
        metrics: crate::SignalMetrics {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            prr: row.get(4)?,
            ror: row.get(5)?,
            ebgm: row.get(6)?,
        },
        risk_level: row.get(7)?,
        recommended_actions: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_flow() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let manager = PersistenceManager::new(None)?;
        let sam = SignalAnalysisResult::new("TestDrug", "TestEvent", 10, 100, 5, 5000);

        manager.save_result(&sam)?;
        let recent = manager.get_recent_signals(10)?;

        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].drug_name, "TestDrug");
        Ok(())
    }
}
