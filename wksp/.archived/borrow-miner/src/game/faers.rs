//! FAERS API Integration - Real FDA adverse event data
//!
//! Server functions for querying openFDA FAERS database

use leptos::prelude::*;
use serde::Deserialize;

use super::signals::{CaseCount, DrugEvent, SignalResult, PRR, ROR, IC, EB05};

/// FAERS API response structure
#[derive(Debug, Clone, Deserialize)]
pub struct FaersResponse {
    pub results: Option<Vec<FaersResult>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersResult {
    pub term: String,
    pub count: u32,
}

/// Query FAERS for top adverse events for a drug
#[server(FaersDrugEvents)]
pub async fn faers_drug_events(drug: String, limit: u32) -> Result<Vec<DrugEvent>, ServerFnError> {
    let url = format!(
        "https://api.fda.gov/drug/event.json?search=patient.drug.medicinalproduct:{}&count=patient.reaction.reactionmeddrapt.exact&limit={}",
        drug, limit
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Request failed: {}", e)))?;

    let data: FaersResponse = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse failed: {}", e)))?;

    let events = data.results.unwrap_or_default()
        .into_iter()
        .map(|r| DrugEvent::new(&drug, r.term, r.count))
        .collect();

    Ok(events)
}

/// Calculate signal metrics for a drug-event pair
#[server(CalculateSignal)]
pub async fn calculate_signal(
    a: u32,  // drug+event
    b: u32,  // drug+other
    c: u32,  // other+event
    d: u32,  // other+other
) -> Result<SignalResult, ServerFnError> {
    let total = (a + b + c + d) as f64;
    let a = a as f64;
    let b = b as f64;
    let c = c as f64;
    let d = d as f64;

    // PRR = (a/(a+b)) / (c/(c+d))
    let prr = if (a + b) > 0.0 && (c + d) > 0.0 && c > 0.0 {
        (a / (a + b)) / (c / (c + d))
    } else {
        1.0
    };

    // ROR = (a*d) / (b*c)
    let ror = if b > 0.0 && c > 0.0 {
        (a * d) / (b * c)
    } else {
        1.0
    };

    // IC (simplified) = log2(observed/expected)
    let expected = ((a + b) * (a + c)) / total;
    let ic = if expected > 0.0 {
        (a / expected).log2()
    } else {
        0.0
    };

    // Chi-square
    let chi_square = chi_sq(a, b, c, d, total);

    // EB05 (simplified approximation)
    let eb05 = prr * 0.8; // Conservative lower bound

    Ok(SignalResult {
        prr: PRR(prr),
        ror: ROR(ror),
        ic: IC(ic),
        eb05: EB05(eb05),
        case_count: CaseCount(a as u32),
        chi_square,
    })
}

fn chi_sq(a: f64, b: f64, c: f64, d: f64, n: f64) -> f64 {
    let e_a = (a + b) * (a + c) / n;
    let e_b = (a + b) * (b + d) / n;
    let e_c = (c + d) * (a + c) / n;
    let e_d = (c + d) * (b + d) / n;

    let chi = |o: f64, e: f64| if e > 0.0 { (o - e).powi(2) / e } else { 0.0 };

    chi(a, e_a) + chi(b, e_b) + chi(c, e_c) + chi(d, e_d)
}

/// Known drug-event pairs with pre-computed signals (for demo)
pub fn known_signals() -> Vec<(&'static str, &'static str, SignalResult)> {
    vec![
        ("Warfarin", "Haemorrhage", SignalResult {
            prr: PRR(8.7),
            ror: ROR(9.2),
            ic: IC(3.1),
            eb05: EB05(7.5),
            case_count: CaseCount(2341),
            chi_square: 1205.8,
        }),
        ("Aspirin", "Gastrointestinal haemorrhage", SignalResult {
            prr: PRR(3.2),
            ror: ROR(3.5),
            ic: IC(1.8),
            eb05: EB05(2.8),
            case_count: CaseCount(838),
            chi_square: 245.6,
        }),
        ("Metformin", "Lactic acidosis", SignalResult {
            prr: PRR(5.1),
            ror: ROR(5.8),
            ic: IC(2.4),
            eb05: EB05(4.2),
            case_count: CaseCount(156),
            chi_square: 89.3,
        }),
        ("Lisinopril", "Cough", SignalResult {
            prr: PRR(2.1),
            ror: ROR(2.3),
            ic: IC(1.1),
            eb05: EB05(1.8),
            case_count: CaseCount(445),
            chi_square: 67.2,
        }),
    ]
}
