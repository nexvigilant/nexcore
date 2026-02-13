//! Wolfram Alpha tools: Computational knowledge engine integration
//!
//! Provides access to Wolfram Alpha's 4 API endpoints:
//! - Full Results API (XML) - Comprehensive results with pods
//! - Short Answer API - Single-line answers
//! - Spoken Results API - Natural language responses
//! - Simple API - Image-based results

use crate::params::{
    WolframAstronomyParams, WolframCalculateParams, WolframChemistryParams, WolframConvertParams,
    WolframDataLookupParams, WolframDatetimeParams, WolframFinanceParams, WolframImageParams,
    WolframLinguisticsParams, WolframNutritionParams, WolframPhysicsParams, WolframPlotParams,
    WolframQueryFilteredParams, WolframQueryParams, WolframQueryWithAssumptionParams,
    WolframShortParams, WolframStatisticsParams, WolframStepByStepParams,
};
use quick_xml::Reader;
use quick_xml::events::Event;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::env;

// API endpoints — explicit HTTPS (no reliance on implicit redirect)
const FULL_RESULTS_URL: &str = "https://api.wolframalpha.com/v2/query";
const SHORT_ANSWER_URL: &str = "https://api.wolframalpha.com/v1/result";
const SPOKEN_URL: &str = "https://api.wolframalpha.com/v1/spoken";
const SIMPLE_URL: &str = "https://api.wolframalpha.com/v1/simple";

// Default timeout in seconds
const DEFAULT_TIMEOUT: u64 = 30;

/// Get API key from environment
fn get_api_key() -> String {
    env::var("WOLFRAM_API_KEY").unwrap_or_else(|_| "DEMO".to_string())
}

/// HTTP client for Wolfram Alpha API
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT + 5))
        .build()
        .unwrap_or_default()
}

// ============================================================================
// Full Results API (XML parsing)
// ============================================================================

/// Parsed pod from Full Results API
///
/// Tier: T3 (Domain-specific Wolfram response)
/// Grounds to T1 Concepts via String/bool/Vec fields
/// Ord: N/A (composite record)
#[derive(Debug, Default)]
struct Pod {
    id: String,
    title: String,
    primary: bool,
    subpods: Vec<Subpod>,
}

/// Parsed subpod
///
/// Tier: T3 (Domain-specific Wolfram response)
/// Grounds to T1 Concepts via String/Option fields
/// Ord: N/A (composite record)
#[derive(Debug, Default)]
struct Subpod {
    title: String,
    plaintext: String,
    image_src: Option<String>,
}

/// Parsed Full Results response
///
/// Tier: T3 (Domain-specific Wolfram response)
/// Grounds to T1 Concepts via bool/String/Vec fields
/// Ord: N/A (composite record)
#[derive(Debug, Default)]
struct FullResult {
    success: bool,
    error: bool,
    error_msg: Option<String>,
    pods: Vec<Pod>,
    assumptions: Vec<String>,
}

/// Query Full Results API and parse XML response
async fn query_full_results(
    query: &str,
    units: &str,
    location: Option<&str>,
    assumption: Option<&str>,
    include_pods: Option<&[String]>,
    exclude_pods: Option<&[String]>,
) -> Result<FullResult, String> {
    let app_id = get_api_key();

    let mut params = vec![
        ("appid", app_id.as_str()),
        ("input", query),
        ("format", "plaintext,image"),
        ("output", "xml"),
        ("units", units),
        ("reinterpret", "true"),
    ];

    let location_owned;
    if let Some(loc) = location {
        location_owned = loc.to_string();
        params.push(("location", &location_owned));
    }

    let assumption_owned;
    if let Some(assum) = assumption {
        assumption_owned = assum.to_string();
        params.push(("assumption", &assumption_owned));
    }

    let include_str;
    if let Some(pods) = include_pods {
        include_str = pods.join(",");
        params.push(("includepodid", &include_str));
    }

    let exclude_str;
    if let Some(pods) = exclude_pods {
        exclude_str = pods.join(",");
        params.push(("excludepodid", &exclude_str));
    }

    let url = format!(
        "{}?{}",
        FULL_RESULTS_URL,
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    );

    let response = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("API error: HTTP {status}"));
    }

    // Validate Content-Type before XML parsing — ∂ Boundary guard
    // Wolfram Full Results returns text/xml or application/xml
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.is_empty()
        && !content_type.contains("xml")
        && !content_type.contains("text/plain")
    {
        return Err(format!(
            "Unexpected Content-Type from Wolfram API: {content_type} (expected XML)"
        ));
    }

    let xml = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    parse_full_results_xml(&xml)
}

/// Parse XML response - single-pass streaming parser
fn parse_full_results_xml(xml: &str) -> Result<FullResult, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut result = FullResult::default();
    let mut current_pod: Option<Pod> = None;
    let mut current_subpod: Option<Subpod> = None;
    let mut in_plaintext = false;
    let mut in_assumption = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.name();
                // SAFETY: XML element names are always valid UTF-8 per spec;
                // empty fallback handles malformed edge cases gracefully
                let name_str = std::str::from_utf8(name.as_ref()).unwrap_or("");

                match name_str {
                    "queryresult" => parse_queryresult_attrs(&e, &mut result),
                    "pod" => current_pod = Some(parse_pod_attrs(&e)),
                    "subpod" => current_subpod = Some(parse_subpod_attrs(&e)),
                    "plaintext" => in_plaintext = true,
                    "img" => parse_img_attrs(&e, &mut current_subpod),
                    "assumption" => {
                        in_assumption = true;
                        parse_assumption_word(&e, &mut result.assumptions);
                    }
                    "value" if in_assumption => parse_assumption_value(&e, &mut result.assumptions),
                    "error" => result.error = true,
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name_bytes = e.name();
                // SAFETY: Same UTF-8 invariant as Start events
                let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");
                match name {
                    "pod" => {
                        if let Some(pod) = current_pod.take() {
                            result.pods.push(pod);
                        }
                    }
                    "subpod" => {
                        if let (Some(pod), Some(subpod)) = (&mut current_pod, current_subpod.take())
                        {
                            pod.subpods.push(subpod);
                        }
                    }
                    "plaintext" => in_plaintext = false,
                    "assumption" => in_assumption = false,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_plaintext {
                    if let Some(subpod) = &mut current_subpod {
                        subpod.plaintext = text;
                    }
                } else if result.error && result.error_msg.is_none() && !text.is_empty() {
                    result.error_msg = Some(text);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    Ok(result)
}

// Helper functions to avoid nested loops in XML parsing
fn parse_queryresult_attrs(e: &quick_xml::events::BytesStart<'_>, result: &mut FullResult) {
    result.success = get_attr(e, "success") == "true";
    result.error = get_attr(e, "error") == "true";
}

fn parse_pod_attrs(e: &quick_xml::events::BytesStart<'_>) -> Pod {
    Pod {
        id: get_attr(e, "id"),
        title: get_attr(e, "title"),
        primary: get_attr(e, "primary") == "true",
        subpods: Vec::new(),
    }
}

fn parse_subpod_attrs(e: &quick_xml::events::BytesStart<'_>) -> Subpod {
    Subpod {
        title: get_attr(e, "title"),
        plaintext: String::new(),
        image_src: None,
    }
}

fn parse_img_attrs(e: &quick_xml::events::BytesStart<'_>, subpod: &mut Option<Subpod>) {
    if let Some(sp) = subpod {
        sp.image_src = Some(get_attr(e, "src"));
    }
}

fn parse_assumption_word(e: &quick_xml::events::BytesStart<'_>, assumptions: &mut Vec<String>) {
    let word = get_attr(e, "word");
    if !word.is_empty() {
        assumptions.push(format!("Interpreted '{}' as:", word));
    }
}

fn parse_assumption_value(e: &quick_xml::events::BytesStart<'_>, assumptions: &mut Vec<String>) {
    let desc = get_attr(e, "desc");
    if !desc.is_empty() {
        assumptions.push(format!("  • {desc}"));
    }
}

/// Extract an XML attribute value by key.
///
/// SAFETY: `unwrap_or("")` is intentional — XML attribute keys/values are
/// specified as UTF-8 by the XML 1.0 spec. Non-UTF-8 bytes would indicate
/// a malformed document, in which case we skip the attribute gracefully
/// rather than panicking.
fn get_attr(e: &quick_xml::events::BytesStart<'_>, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| std::str::from_utf8(a.key.as_ref()).unwrap_or("") == key)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
        .unwrap_or_default()
}

/// Format Full Results for display using iterators
fn format_full_result(result: &FullResult, verbose: bool) -> String {
    if result.error {
        return format!(
            "❌ Error: {}",
            result.error_msg.as_deref().unwrap_or("Unknown error")
        );
    }

    if !result.success || result.pods.is_empty() {
        let mut msg = "❌ No results found. Try rephrasing your query.".to_string();
        if !result.assumptions.is_empty() {
            msg.push_str("\n\n**Did you mean:**\n");
            msg.push_str(&result.assumptions.join("\n"));
        }
        return msg;
    }

    // Use flat_map to avoid nested loops
    let pod_content: Vec<String> = result
        .pods
        .iter()
        .flat_map(|pod| format_pod(pod, verbose))
        .collect();

    let mut output = pod_content.join("\n");

    if verbose && !result.assumptions.is_empty() {
        output.push_str("\n\n---\n**Assumptions made:**\n");
        output.push_str(&result.assumptions.join("\n"));
    }

    if output.is_empty() {
        "No content in response.".to_string()
    } else {
        output
    }
}

/// Format a single pod - returns lines for this pod
fn format_pod(pod: &Pod, verbose: bool) -> Vec<String> {
    let marker = if pod.primary { " ⭐" } else { "" };
    let mut lines = vec![format!("\n## {}{}", pod.title, marker)];

    // Use extend with iterator instead of nested loop
    lines.extend(pod.subpods.iter().flat_map(|sp| format_subpod(sp, verbose)));

    lines
}

/// Format a single subpod - returns lines for this subpod
fn format_subpod(subpod: &Subpod, verbose: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if !subpod.title.is_empty() {
        lines.push(format!("### {}", subpod.title));
    }
    if !subpod.plaintext.is_empty() {
        lines.push(subpod.plaintext.clone()); // CLONE: Building output Vec requires owned strings
    }
    if verbose {
        if let Some(ref src) = subpod.image_src {
            lines.push(format!("[Image: {src}]"));
        }
    }
    lines
}

// ============================================================================
// Short Answer API
// ============================================================================

async fn query_short_answer(query: &str, units: &str) -> Result<String, String> {
    let app_id = get_api_key();
    let url = format!(
        "{}?appid={}&i={}&units={}",
        SHORT_ANSWER_URL,
        urlencoding::encode(&app_id),
        urlencoding::encode(query),
        units
    );

    let response = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status();
    if status.as_u16() == 501 {
        return Ok("No short answer available for this query.".to_string());
    }

    if !status.is_success() {
        return Err(format!("API error: {status}"));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))
}

// ============================================================================
// Spoken Results API
// ============================================================================

async fn query_spoken(query: &str, units: &str) -> Result<String, String> {
    let app_id = get_api_key();
    let url = format!(
        "{}?appid={}&i={}&units={}",
        SPOKEN_URL,
        urlencoding::encode(&app_id),
        urlencoding::encode(query),
        units
    );

    let response = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status();
    if status.as_u16() == 501 {
        return Ok("No spoken answer available for this query.".to_string());
    }

    if !status.is_success() {
        return Err(format!("API error: {status}"));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))
}

// ============================================================================
// Simple API (Image URL)
// ============================================================================

fn get_simple_image_url(query: &str) -> String {
    let app_id = get_api_key();
    format!(
        "{}?appid={}&i={}",
        SIMPLE_URL,
        urlencoding::encode(&app_id),
        urlencoding::encode(query)
    )
}

// ============================================================================
// Tool Implementations
// ============================================================================

fn format_result(text: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

fn format_error(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(format!(
        "❌ {msg}"
    ))]))
}

/// Query Wolfram Alpha with full results
pub async fn query(params: WolframQueryParams) -> Result<CallToolResult, McpError> {
    match query_full_results(
        &params.query,
        &params.units,
        params.location.as_deref(),
        None,
        None,
        None,
    )
    .await
    {
        Ok(result) => format_result(format_full_result(&result, params.verbose)),
        Err(e) => format_error(&e),
    }
}

/// Get concise single-line answer
pub async fn short_answer(params: WolframShortParams) -> Result<CallToolResult, McpError> {
    match query_short_answer(&params.query, &params.units).await {
        Ok(answer) => format_result(answer),
        Err(e) => format_error(&e),
    }
}

/// Get natural language answer
pub async fn spoken_answer(params: WolframShortParams) -> Result<CallToolResult, McpError> {
    match query_spoken(&params.query, &params.units).await {
        Ok(answer) => format_result(answer),
        Err(e) => format_error(&e),
    }
}

/// Perform mathematical calculation - extracts priority pods
pub async fn calculate(params: WolframCalculateParams) -> Result<CallToolResult, McpError> {
    match query_full_results(&params.expression, "metric", None, None, None, None).await {
        Ok(result) => {
            let priority_titles = [
                "Result",
                "Solution",
                "Solutions",
                "Roots",
                "Root",
                "Exact result",
                "Decimal approximation",
                "Decimal form",
                "Definite integral",
                "Indefinite integral",
                "Limit",
                "Derivative",
                "Sum",
                "Product",
            ];

            let output: Vec<String> = result
                .pods
                .iter()
                .filter(|p| is_priority_pod(p, &priority_titles))
                .flat_map(|pod| format_calc_pod(pod))
                .collect();

            if output.is_empty() {
                format_result(format_full_result(&result, false))
            } else {
                format_result(output.join("\n"))
            }
        }
        Err(e) => format_error(&e),
    }
}

fn is_priority_pod(pod: &Pod, titles: &[&str]) -> bool {
    let title_lower = pod.title.to_lowercase();
    pod.primary
        || titles.iter().any(|t| t.eq_ignore_ascii_case(&pod.title))
        || title_lower.contains("result")
        || title_lower.contains("solution")
}

fn format_calc_pod(pod: &Pod) -> Vec<String> {
    let mut lines = vec![format!("\n## {}", pod.title)];
    lines.extend(
        pod.subpods
            .iter()
            .filter(|sp| !sp.plaintext.is_empty())
            .map(|sp| sp.plaintext.clone()),
    );
    lines
}

/// Solve with step-by-step explanations
pub async fn step_by_step(params: WolframStepByStepParams) -> Result<CallToolResult, McpError> {
    let query = format!("{} step by step", params.problem);
    match query_full_results(&query, "metric", None, None, None, None).await {
        Ok(result) => {
            let output: Vec<String> = std::iter::once("# Step-by-Step Solution\n".to_string())
                .chain(
                    result
                        .pods
                        .iter()
                        .filter(|p| is_step_pod(p))
                        .flat_map(|p| format_step_pod(p)),
                )
                .collect();

            if output.len() == 1 {
                format_result(format_full_result(&result, false))
            } else {
                format_result(output.join("\n"))
            }
        }
        Err(e) => format_error(&e),
    }
}

fn is_step_pod(pod: &Pod) -> bool {
    let title_lower = pod.title.to_lowercase();
    title_lower.contains("step") || title_lower.contains("solution") || pod.primary
}

fn format_step_pod(pod: &Pod) -> Vec<String> {
    let title_lower = pod.title.to_lowercase();
    let mut lines = vec![format!("\n## {}", pod.title)];
    lines.extend(
        pod.subpods
            .iter()
            .enumerate()
            .filter(|(_, sp)| !sp.plaintext.is_empty())
            .map(|(i, sp)| {
                if title_lower.contains("step") {
                    format!("**Step {}:** {}", i + 1, sp.plaintext)
                } else {
                    sp.plaintext.clone()
                }
            }),
    );
    lines
}

/// Generate mathematical plot
pub async fn plot(params: WolframPlotParams) -> Result<CallToolResult, McpError> {
    let query = params
        .range
        .as_ref()
        .map(|r| format!("plot {} {}", params.expression, r))
        .unwrap_or_else(|| format!("plot {}", params.expression));

    match query_full_results(&query, "metric", None, None, None, None).await {
        Ok(result) => {
            let output: Vec<String> = result
                .pods
                .iter()
                .filter(|p| p.title.to_lowercase().contains("plot"))
                .flat_map(|p| format_plot_pod(p))
                .collect();

            if output.is_empty() {
                format_result(format_full_result(&result, true))
            } else {
                format_result(output.join("\n\n"))
            }
        }
        Err(e) => format_error(&e),
    }
}

fn format_plot_pod(pod: &Pod) -> Vec<String> {
    pod.subpods
        .iter()
        .filter_map(|sp| {
            sp.image_src
                .as_ref()
                .map(|src| format!("**{}**\n![Plot]({})", pod.title, src))
        })
        .collect()
}

/// Convert between units
pub async fn convert(params: WolframConvertParams) -> Result<CallToolResult, McpError> {
    let query = format!(
        "convert {} {} to {}",
        params.value, params.from_unit, params.to_unit
    );
    match query_short_answer(&query, "metric").await {
        Ok(answer) => format_result(format!(
            "{} {} = {}",
            params.value, params.from_unit, answer
        )),
        Err(e) => format_error(&e),
    }
}

/// Look up chemical compound information
pub async fn chemistry(params: WolframChemistryParams) -> Result<CallToolResult, McpError> {
    let query = params
        .property
        .as_ref()
        .map(|p| format!("{} {}", params.compound, p))
        .unwrap_or_else(|| format!("{} chemical properties", params.compound));
    match query_full_results(&query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Query physics constants and calculations
pub async fn physics(params: WolframPhysicsParams) -> Result<CallToolResult, McpError> {
    match query_full_results(&params.query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Query astronomical data
pub async fn astronomy(params: WolframAstronomyParams) -> Result<CallToolResult, McpError> {
    match query_full_results(
        &params.query,
        "metric",
        params.location.as_deref(),
        None,
        None,
        None,
    )
    .await
    {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Perform statistical analysis
pub async fn statistics(params: WolframStatisticsParams) -> Result<CallToolResult, McpError> {
    match query_full_results(&params.query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Look up real-world data
pub async fn data_lookup(params: WolframDataLookupParams) -> Result<CallToolResult, McpError> {
    match query_full_results(&params.query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Query with specific interpretation
pub async fn query_with_assumption(
    params: WolframQueryWithAssumptionParams,
) -> Result<CallToolResult, McpError> {
    match query_full_results(
        &params.query,
        "metric",
        None,
        Some(&params.assumption),
        None,
        None,
    )
    .await
    {
        Ok(result) => format_result(format_full_result(&result, true)),
        Err(e) => format_error(&e),
    }
}

/// Query with pod filtering
pub async fn query_filtered(
    params: WolframQueryFilteredParams,
) -> Result<CallToolResult, McpError> {
    match query_full_results(
        &params.query,
        "metric",
        None,
        None,
        params.include_pods.as_deref(),
        params.exclude_pods.as_deref(),
    )
    .await
    {
        Ok(result) => {
            if result.pods.is_empty() && !result.error {
                if let Ok(full) =
                    query_full_results(&params.query, "metric", None, None, None, None).await
                {
                    let pod_ids: String = full
                        .pods
                        .iter()
                        .map(|p| format!("  • {} ({})", p.id, p.title))
                        .collect::<Vec<_>>()
                        .join("\n");
                    return format_result(format!(
                        "{}\n\n**Available pod IDs for this query:**\n{}",
                        format_full_result(&full, false),
                        pod_ids
                    ));
                }
            }
            format_result(format_full_result(&result, false))
        }
        Err(e) => format_error(&e),
    }
}

/// Get visual/image result
pub async fn image_result(params: WolframImageParams) -> Result<CallToolResult, McpError> {
    let url = get_simple_image_url(&params.query);
    format_result(format!(
        "![Wolfram Alpha Result]({})\n\nDirect URL: {}",
        url, url
    ))
}

/// Calculate dates, times, and durations
pub async fn datetime(params: WolframDatetimeParams) -> Result<CallToolResult, McpError> {
    match query_full_results(
        &params.query,
        "metric",
        params.location.as_deref(),
        None,
        None,
        None,
    )
    .await
    {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Look up nutritional information
pub async fn nutrition(params: WolframNutritionParams) -> Result<CallToolResult, McpError> {
    let query = params
        .amount
        .as_ref()
        .map(|a| format!("{} {} nutrition facts", a, params.food))
        .unwrap_or_else(|| format!("{} nutrition facts", params.food));
    match query_full_results(&query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Financial calculations and data
pub async fn finance(params: WolframFinanceParams) -> Result<CallToolResult, McpError> {
    match query_full_results(&params.query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

/// Language and word information
pub async fn linguistics(params: WolframLinguisticsParams) -> Result<CallToolResult, McpError> {
    match query_full_results(&params.query, "metric", None, None, None, None).await {
        Ok(result) => format_result(format_full_result(&result, false)),
        Err(e) => format_error(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<queryresult success='true' error='false' numpods='1'>
    <pod id='Result' title='Result' primary='true'>
        <subpod title=''>
            <plaintext>4</plaintext>
        </subpod>
    </pod>
</queryresult>"#;

        let result = parse_full_results_xml(xml).expect("hardcoded valid XML"); // INVARIANT: Test XML is hardcoded valid
        assert!(result.success);
        assert!(!result.error);
        assert_eq!(result.pods.len(), 1);
        assert_eq!(result.pods[0].title, "Result");
        assert!(result.pods[0].primary);
        assert_eq!(result.pods[0].subpods[0].plaintext, "4");
    }

    #[test]
    fn test_parse_error_xml() {
        let xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<queryresult success='false' error='true'>
    <error>
        <msg>Invalid query</msg>
    </error>
</queryresult>"#;

        let result = parse_full_results_xml(xml).expect("hardcoded valid XML"); // INVARIANT: Test XML is hardcoded valid
        assert!(!result.success);
        assert!(result.error);
    }

    #[test]
    fn test_format_full_result() {
        let result = FullResult {
            success: true,
            error: false,
            error_msg: None,
            pods: vec![Pod {
                id: "Result".to_string(),
                title: "Result".to_string(),
                primary: true,
                subpods: vec![Subpod {
                    title: String::new(),
                    plaintext: "42".to_string(),
                    image_src: None,
                }],
            }],
            assumptions: vec![],
        };

        let formatted = format_full_result(&result, false);
        assert!(formatted.contains("Result"));
        assert!(formatted.contains("42"));
        assert!(formatted.contains("⭐"));
    }

    #[test]
    fn test_simple_image_url() {
        let url = get_simple_image_url("2+2");
        assert!(url.starts_with(SIMPLE_URL));
        assert!(url.contains("2%2B2"));
    }
}
