//! Portfolio — capability evidence and achievements

use leptos::prelude::*;
use crate::api_client::Enrollment;
use crate::auth::use_auth;

/// Server function to list student enrollments
#[server(ListStudentEnrollments, "/api")]
pub async fn list_student_enrollments_action() -> Result<Vec<Enrollment>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.academy_list_enrollments().await
        .map_err(ServerFnError::new)
}

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let enrollments = Resource::new(|| (), |_| list_student_enrollments_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"PORTFOLIO"</h1>
                <p class="mt-2 text-slate-400">"Validated capability evidence and structural grounding."</p>
            </header>

            <div class="grid gap-8 lg:grid-cols-3">
                <div class="lg:col-span-2">
                    <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// EVIDENCE STREAM"</h2>
                    
                    <Suspense fallback=|| view! { <div class="p-12 animate-pulse text-slate-500">"SCANNING RECORDS..."</div> }>
                        {move || enrollments.get().map(|result| match result {
                            Ok(list) => view! { <EvidenceListView list=list /> }.into_any(),
                            Err(e) => view! { <div class="p-8 text-red-400 font-mono text-sm">{e.to_string()}</div> }.into_any()
                        })}
                    </Suspense>
                </div>

                <div>
                    <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// DOMAIN COVERAGE"</h2>
                    <div class="glass-panel p-6 rounded-2xl border border-slate-800 space-y-6">
                        <DomainProgress title="D01 Foundation" percent=85 color="bg-cyan-500" />
                        <DomainProgress title="D08 Detection" percent=42 color="bg-amber-500" />
                        <DomainProgress title="D03 Processing" percent=12 color="bg-slate-700" />
                        <DomainProgress title="D05 Assessment" percent=0 color="bg-slate-800" />
                        
                        <div class="pt-6 border-t border-slate-800/50">
                            <p class="text-[10px] font-mono text-slate-500 leading-relaxed uppercase">
                                "Structural integrity is maintained through continuous grounding to Lex Primitiva."
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn EvidenceListView(list: Vec<Enrollment>) -> impl IntoView {
    if list.is_empty() {
        view! {
            <div class="glass-panel p-12 rounded-2xl border border-slate-800 text-center relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-cyan-500/5 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-1000"></div>
                <p class="text-slate-500 font-mono text-sm tracking-widest uppercase mb-4">"NO EVIDENCE DETECTED"</p>
                <p class="text-xs text-slate-600 font-mono max-w-md mx-auto leading-relaxed">
                    "COMPLETE COMPETENCY ASSESSMENTS TO POPULATE THIS STREAM WITH VALIDATED CAPABILITY RECORDS."
                </p>
                <a href="/academy/courses" class="inline-block mt-8 text-[11px] font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-[0.2em] font-mono">
                    "ENGAGE CURRICULUM >>"
                </a>
            </div>
        }.into_any()
    } else {
        view! {
            <div class="space-y-4">
                {list.into_iter().map(|e| view! { <EvidenceItem enrollment=e /> }).collect_view()}
            </div>
        }.into_any()
    }
}

#[component]
fn EvidenceItem(enrollment: Enrollment) -> impl IntoView {
    view! {
        <div class="glass-panel p-6 rounded-xl border border-slate-800 flex items-center justify-between group hover:border-slate-700 transition-all">
            <div class="flex items-center gap-4">
                <div class="h-10 w-10 rounded-full bg-slate-900 border border-slate-800 flex items-center justify-center text-xs font-mono text-cyan-500 font-bold">
                    "E"
                </div>
                <div>
                    <h4 class="text-sm font-bold text-white font-mono uppercase tracking-tight">{enrollment.course_id}</h4>
                    <p class="text-[10px] text-slate-500 font-mono">"Validated: " {enrollment.enrolled_at.format("%Y-%m-%d").to_string()}</p>
                </div>
            </div>
            <div class="text-right">
                <p class="text-[9px] font-mono font-bold text-slate-600 uppercase mb-1">"PROGRESS"</p>
                <p class="text-sm font-black text-cyan-400 font-mono">{format!("{:.0}%", enrollment.progress)}</p>
            </div>
        </div>
    }
}

#[component]
fn DomainProgress(title: &'static str, percent: u32, color: &'static str) -> impl IntoView {
    view! {
        <div class="space-y-2">
            <div class="flex justify-between items-end">
                <span class="text-[10px] font-mono font-bold text-slate-400 uppercase tracking-widest">{title}</span>
                <span class="text-[10px] font-mono font-bold text-slate-500 uppercase">{format!("{percent}%")}</span>
            </div>
            <div class="h-1 bg-slate-900 rounded-full overflow-hidden">
                <div 
                    class=format!("h-full rounded-full transition-all duration-1000 {}", color)
                    style=format!("width: {percent}%")
                ></div>
            </div>
        </div>
    }
}