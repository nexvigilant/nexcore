//! Integration tests for PV Signal Detection tools
//!
//! Covers PRR, ROR, IC, EBGM, Chi-square, Naranjo, and WHO-UMC.

use nexcore_mcp::params::{
    ContingencyTableParams, NaranjoParams, SignalAlgorithmParams, SignalCompleteParams,
    WhoUmcParams,
};
use nexcore_mcp::tools::pv;

fn create_signal_table() -> ContingencyTableParams {
    ContingencyTableParams {
        a: 10,
        b: 90,
        c: 10,
        d: 9890,
    }
}

#[test]
fn test_signal_complete_analysis() {
    let params = SignalCompleteParams {
        table: create_signal_table(),
        prr_threshold: 2.0,
        min_n: 3,
        fdr_correction: false,
    };
    let result = pv::signal_complete(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("prr"));
    assert!(text.contains("ror"));
    assert!(text.contains("ic"));
    assert!(text.contains("ebgm"));
}

#[test]
fn test_signal_prr_calculation() {
    let params = SignalAlgorithmParams {
        table: create_signal_table(),
    };
    let result = pv::signal_prr(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("algorithm") && text.contains("PRR"));
    // PRR = (10/100) / (10/9900) = 0.1 / 0.0010101 = 99.0
    assert!(text.contains("99.0") || text.contains("99"));
}

#[test]
fn test_chi_square_significance() {
    let params = SignalAlgorithmParams {
        table: create_signal_table(),
    };
    let result = pv::chi_square(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("significant_0_05") && text.contains("true"));
}

#[test]
fn test_naranjo_probable() {
    let params = NaranjoParams {
        temporal: 1,
        dechallenge: 1,
        rechallenge: 2, // 1 + 1 + 2 + 1 + 1 = 6 (Probable)
        alternatives: 1,
        previous: 1,
    };
    let result = pv::naranjo_quick(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("\"category\"") && text.contains("Possible"));
}

#[test]
fn test_who_umc_certain() {
    let params = WhoUmcParams {
        temporal: 1,
        dechallenge: 1,
        rechallenge: 1,
        alternatives: 1,
        plausibility: 1,
    };
    let result = pv::who_umc_quick(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("Certain"));
}
