//! Reporting page — generate and view safety reports

use leptos::prelude::*;
use crate::api_client::{ReportRequest, ReportType, ReportResponse};

/// Server function to trigger report generation
#[server(GenerateReport, "/api")]
pub async fn generate_report_action(report_type: ReportType) -> Result<ReportResponse, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = ReportRequest {
        report_type,
        start_date: None,
        end_date: None,
    };

    client.reporting_generate(&req).await
        .map_err(ServerFnError::new)
}

/// Server function to list reports
#[server(ListReports, "/api")]
pub async fn list_reports_action() -> Result<Vec<ReportResponse>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.reporting_list().await
        .map_err(ServerFnError::new)
}

#[component]
pub fn ReportingPage() -> impl IntoView {
    let reports = Resource::new(|| (), |_| list_reports_action());
    let generate_action = ServerAction::<GenerateReport>::new();
    
    Effect::new(move |_| {
        if generate_action.value().get().is_some() {
            reports.refetch();
        }
    });

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <header class="mb-10">
                <h1 class="text-3xl font-bold text-white font-mono tracking-tight uppercase">"Safety Reporting"</h1>
                <p class="mt-2 text-slate-400">"Generate automated regulatory compliance and audit documentation"</p>
            </header>

            <div class="grid gap-8 md:grid-cols-2">
                <ReportGenerationSection action=generate_action />
                <RecentReportsSection reports=reports />
            </div>
        </div>
    }
}

#[component]
fn ReportGenerationSection(action: ServerAction<GenerateReport>) -> impl IntoView {
    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-lg font-bold text-white font-mono uppercase mb-6 tracking-wide">"Generate"</h2>
            <div class="space-y-3">
                <ReportGeneratorButton 
                    label="Signal Summary" 
                    report_type=ReportType::SignalSummary 
                    description="Aggregate view of all active safety signals"
                    action=action 
                />
                <ReportGeneratorButton 
                    label="Audit Trail" 
                    report_type=ReportType::AuditTrail 
                    description="Full temporal record of system decisions"
                    action=action 
                />
                <ReportGeneratorButton 
                    label="Guardian Performance" 
                    report_type=ReportType::GuardianPerformance 
                    description="Homeostasis stability and response metrics"
                    action=action 
                />
            </div>
        </section>
    }
}

#[component]
fn RecentReportsSection(reports: Resource<Result<Vec<ReportResponse>, ServerFnError>>) -> impl IntoView {
    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-lg font-bold text-white font-mono uppercase mb-6 tracking-wide">"History"</h2>
            <div class="mt-4">
                <Suspense fallback=|| view! { <div class="animate-pulse space-y-2"><div class="h-10 bg-slate-800 rounded"></div><div class="h-10 bg-slate-800 rounded"></div></div> }>
                    {move || reports.get().map(|result| view! { <ReportResultView result=result /> })}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn ReportResultView(result: Result<Vec<ReportResponse>, ServerFnError>) -> impl IntoView {
    match result {
        Ok(list) => view! { <ReportListView list=list /> }.into_any(),
        Err(e) => view! { <div class="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">{e.to_string()}</div> }.into_any()
    }
}

#[component]
fn ReportListView(list: Vec<ReportResponse>) -> impl IntoView {
    if list.is_empty() {
        view! { <p class="text-sm text-slate-500 italic text-center py-8">"No reports generated in current session."</p> }.into_any()
    } else {
        view! {
            <ul class="space-y-3">
                {list.into_iter().rev().map(|r| view! { <ReportListItem report=r /> }).collect_view()}
            </ul>
        }.into_any()
    }
}

#[component]
fn ReportGeneratorButton(
    label: &'static str, 
    report_type: ReportType,
    description: &'static str,
    action: ServerAction<GenerateReport>
) -> impl IntoView {
    let on_click = move |_| {
        action.dispatch(GenerateReport { report_type: report_type.clone() });
    };

    let is_loading = action.pending();

    view! {
        <button 
            on:click=on_click
            disabled=is_loading
            class="w-full rounded-lg bg-slate-800/80 border border-slate-700/50 p-4 text-left group hover:border-cyan-500/50 hover:bg-slate-800 transition-all disabled:opacity-50"
        >
            <div class="flex justify-between items-center">
                <span class="font-bold text-slate-200 group-hover:text-cyan-400 transition-colors">{label}</span>
                <span class="text-slate-600 group-hover:text-cyan-500 transition-all transform group-hover:translate-x-1">"\u{2192}"</span>
            </div>
            <p class="text-xs text-slate-500 mt-1">{description}</p>
        </button>
    }
}

#[component]
fn ReportListItem(report: ReportResponse) -> impl IntoView {
    let type_name = match report.report_type {
        ReportType::SignalSummary => "SIGNAL SUMMARY",
        ReportType::AuditTrail => "AUDIT TRAIL",
        ReportType::GuardianPerformance => "GUARDIAN PERF",
    };

    view! {
        <li class="rounded-lg border border-slate-800 bg-slate-950/30 p-4 hover:bg-slate-950/50 transition-colors">
            <div class="flex justify-between items-start">
                <div>
                    <p class="text-xs font-bold text-cyan-500 font-mono tracking-wider">{type_name}</p>
                    <p class="text-sm text-slate-300 mt-1">{report.content}</p>
                </div>
                <div class="text-right">
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-green-500/10 text-green-400 border border-green-500/20 font-mono">
                        {report.status.to_uppercase()}
                    </span>
                    <p class="text-[10px] text-slate-600 mt-2 font-mono">{report.generated_at.format("%H:%M:%S").to_string()}</p>
                </div>
            </div>
        </li>
    }
}