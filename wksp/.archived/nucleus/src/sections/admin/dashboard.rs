//! Admin dashboard — platform overview with live stats and activity

use leptos::prelude::*;
use crate::api_client::HealthResponse;

/// Server function to fetch platform health stats
#[server(GetAdminStats, "/api")]
pub async fn get_admin_stats() -> Result<HealthResponse, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.health().await.map_err(ServerFnError::new)
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| get_admin_stats());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Admin Dashboard"</h1>
                    <p class="mt-1 text-slate-400">"Platform management and analytics"</p>
                </div>
                <div class="flex gap-2">
                    <a href="/admin/users" class="rounded-lg border border-slate-700 px-3 py-2 text-xs font-bold text-slate-400 hover:text-white hover:border-slate-600 transition-colors">"USERS"</a>
                    <a href="/admin/settings" class="rounded-lg border border-slate-700 px-3 py-2 text-xs font-bold text-slate-400 hover:text-white hover:border-slate-600 transition-colors">"SETTINGS"</a>
                </div>
            </div>

            // API Health
            <div class="mt-6">
                <Suspense fallback=|| view! {
                    <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-4 animate-pulse">
                        <div class="h-4 w-48 bg-slate-800 rounded"></div>
                    </div>
                }>
                    {move || stats.get().map(|result| match result {
                        Ok(health) => view! {
                            <div class="rounded-lg border border-emerald-500/20 bg-emerald-500/5 p-4 flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <div class="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></div>
                                    <span class="text-sm text-emerald-400 font-mono font-bold">"NexCore API: "{health.status.to_uppercase()}</span>
                                </div>
                                <span class="text-xs text-slate-500 font-mono">"v"{health.version}" · uptime "{health.uptime_secs}"s"</span>
                            </div>
                        }.into_any(),
                        Err(_) => view! {
                            <div class="rounded-lg border border-red-500/20 bg-red-500/5 p-4 flex items-center gap-3">
                                <div class="h-2 w-2 rounded-full bg-red-400"></div>
                                <span class="text-sm text-red-400 font-mono font-bold">"NexCore API: OFFLINE"</span>
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </div>

            // Key metrics
            <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <StatCard label="Total Users" value="0" change="+0 this week" color="text-cyan-400"/>
                <StatCard label="Active Learners" value="0" change="0 in courses" color="text-emerald-400"/>
                <StatCard label="Community Posts" value="0" change="0 this month" color="text-violet-400"/>
                <StatCard label="Revenue" value="$0" change="MRR" color="text-amber-400"/>
            </div>

            // Management sections
            <div class="mt-8">
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500">"Management"</h2>
                <div class="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    <AdminLink href="/admin/academy" title="Academy" desc="Courses, KSBs, learner progress" icon="σ"/>
                    <AdminLink href="/admin/community" title="Community" desc="Moderation, circles, badges" icon="Σ"/>
                    <AdminLink href="/admin/content" title="Content" desc="Articles, series, media library" icon="μ"/>
                    <AdminLink href="/admin/intelligence" title="Intelligence" desc="Reports, research, data sources" icon="ρ"/>
                    <AdminLink href="/admin/users" title="Users" desc="Accounts, roles, permissions" icon="∂"/>
                    <AdminLink href="/admin/settings" title="Settings" desc="Platform config, billing" icon="ς"/>
                    <AdminLink href="/admin/leads" title="Website Leads" desc="Contact form submissions, trials" icon="→"/>
                    <AdminLink href="/admin/media" title="Media" desc="Images, documents, uploads" icon="π"/>
                    <AdminLink href="/admin/research" title="Research" desc="PV research, data analysis" icon="∃"/>
                </div>
            </div>

            // Recent activity
            <div class="mt-8">
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500">"Recent Activity"</h2>
                <div class="mt-4 space-y-2">
                    <ActivityItem time="Just now" event="Admin session started" category="System"/>
                    <ActivityItem time="—" event="Awaiting user registrations" category="Users"/>
                    <ActivityItem time="—" event="Awaiting content submissions" category="Content"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatCard(label: &'static str, value: &'static str, change: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p>
            <p class=format!("mt-2 text-3xl font-bold font-mono {color}")>{value}</p>
            <p class="mt-1 text-xs text-slate-500">{change}</p>
        </div>
    }
}

#[component]
fn AdminLink(href: &'static str, title: &'static str, desc: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <a href=href class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-amber-500/30 transition-colors group">
            <div class="flex items-center gap-3">
                <span class="h-8 w-8 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-400 font-mono text-sm">{icon}</span>
                <h3 class="font-semibold text-white group-hover:text-amber-400 transition-colors">{title}</h3>
            </div>
            <p class="mt-2 text-sm text-slate-400">{desc}</p>
        </a>
    }
}

#[component]
fn ActivityItem(time: &'static str, event: &'static str, category: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 rounded-lg border border-slate-800/50 bg-slate-900/30 p-3">
            <span class="shrink-0 w-16 text-[10px] text-slate-600 font-mono">{time}</span>
            <span class="flex-1 text-sm text-slate-400">{event}</span>
            <span class="rounded bg-slate-800 px-2 py-0.5 text-[10px] text-slate-500 font-mono uppercase">{category}</span>
        </div>
    }
}
