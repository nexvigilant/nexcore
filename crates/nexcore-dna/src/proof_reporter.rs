//! Proof Reporter: Extracts proof metadata from proofs.rs and generates reports.
//!
//! Zero external dependencies — manual JSON formatting.

/// Metadata for a single proof.
#[derive(Debug, Clone)]
pub struct ProofMetadata {
    pub id: String,
    pub algorithm: String,
    pub property: String,
    pub domain: String,
    pub status: String,
}

/// A structured report of all proofs in the codebase.
#[derive(Debug, Clone)]
pub struct ProofReport {
    pub total_proofs: usize,
    pub proofs: Vec<ProofMetadata>,
    pub proven_count: usize,
}

pub struct ProofReporter;

impl ProofReporter {
    /// Extract proof metadata by parsing the table in proofs.rs
    pub fn extract_from_source(source: &str) -> Vec<ProofMetadata> {
        let mut proofs = Vec::new();
        let mut in_table = false;

        for line in source.lines() {
            let line = line.trim();

            // Look for table header
            if line.contains("| ID  | Algorithm | Property | Domain |") {
                in_table = true;
                continue;
            }

            if in_table {
                if !line.starts_with("//! |") || line.contains("|-----|") {
                    if line.is_empty() || (!line.starts_with("//! |") && !line.contains("|-----|"))
                    {
                        // Table might have ended
                        continue;
                    }
                    continue;
                }

                // Parse row: //! | ID  | Algorithm | Property | Domain |
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    let id = parts[1].trim().to_string();
                    let algorithm = parts[2].trim().to_string();
                    let property = parts[3].trim().to_string();
                    let domain = parts[4].trim().to_string();

                    if id == "ID" || id.is_empty() {
                        continue;
                    }

                    proofs.push(ProofMetadata {
                        id,
                        algorithm,
                        property,
                        domain,
                        status: "proven".to_string(),
                    });
                }
            }
        }

        proofs
    }

    pub fn generate_report(source: &str) -> ProofReport {
        let proofs = Self::extract_from_source(source);
        let count = proofs.len();
        ProofReport {
            total_proofs: count,
            proofs,
            proven_count: count,
        }
    }

    pub fn to_json(&self, report: &ProofReport) -> String {
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"total_proofs\": {},\n", report.total_proofs));
        json.push_str(&format!("  \"proven_count\": {},\n", report.proven_count));
        json.push_str("  \"proofs\": [\n");

        for (i, p) in report.proofs.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"id\": \"{}\",\n", p.id));
            json.push_str(&format!("      \"algorithm\": \"{}\",\n", p.algorithm));
            json.push_str(&format!("      \"property\": \"{}\",\n", p.property));
            json.push_str(&format!("      \"domain\": \"{}\",\n", p.domain));
            json.push_str(&format!("      \"status\": \"{}\"\n", p.status));
            json.push_str("    }");
            if i < report.proofs.len() - 1 {
                json.push_str(",");
            }
            json.push_str("\n");
        }

        json.push_str("  ]\n");
        json.push_str("}");
        json
    }
}
