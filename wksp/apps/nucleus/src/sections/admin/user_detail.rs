//! Admin: User detail page — audit logs, permissions, history

use leptos::prelude::*;

#[component]
pub fn UserDetailPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center gap-4 mb-8">
                <a href="/admin/users" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"User Detail"</h1>
            </div>

            <div class="grid gap-8 lg:grid-cols-3">
                /* ---- Profile Summary ---- */
                <div class="lg:col-span-1 space-y-6">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <div class="h-20 w-20 rounded-full bg-slate-800 mx-auto mb-4 flex items-center justify-center text-3xl font-bold text-cyan-400 font-mono">
                            "A"
                        </div>
                        <h2 class="text-center text-xl font-bold text-white">"Alice Johnson"</h2>
                        <p class="text-center text-sm text-slate-500">"alice@example.com"</p>
                        <div class="mt-6 space-y-2">
                            <div class="flex justify-between text-xs">
                                <span class="text-slate-500">"Role"</span>
                                <span class="text-cyan-400 font-bold">"ADMIN"</span>
                            </div>
                            <div class="flex justify-between text-xs">
                                <span class="text-slate-500">"Status"</span>
                                <span class="text-emerald-400 font-bold">"ACTIVE"</span>
                            </div>
                            <div class="flex justify-between text-xs">
                                <span class="text-slate-500">"Joined"</span>
                                <span class="text-slate-300">"2025-10-12"</span>
                            </div>
                        </div>
                    </div>

                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-xs font-bold uppercase tracking-widest text-slate-500 mb-4">"Danger Zone"</h3>
                        <button class="w-full rounded border border-red-500/30 py-2 text-xs font-bold text-red-400 hover:bg-red-500/10 transition-colors">"SUSPEND ACCOUNT"</button>
                    </div>
                </div>

                /* ---- Audit Logs & Activity ---- */
                <div class="lg:col-span-2 space-y-6">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-xs font-bold uppercase tracking-widest text-slate-500 mb-6">"Audit Log"</h3>
                        <div class="space-y-4">
                            <LogEntry date="2026-02-14 14:30" action="Changed Role" detail="Updated User 'Bob' to 'Moderator'"/>
                            <LogEntry date="2026-02-12 09:15" action="Login" detail="IP: 192.168.1.1 (London, UK)"/>
                            <LogEntry date="2026-02-10 18:45" action="Content Moderation" detail="Rejected post ID #8821"/>
                        </div>
                    </div>

                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-xs font-bold uppercase tracking-widest text-slate-500 mb-6">"Course Progress"</h3>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <span class="text-sm text-white">"Signal Detection Mastery"</span>
                                <span class="text-xs text-emerald-400">"COMPLETED"</span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-sm text-white">"Regulatory EU Guidelines"</span>
                                <span class="text-xs text-slate-500">"45%"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn LogEntry(date: &'static str, action: &'static str, detail: &'static str) -> impl IntoView {
    view! {
        <div class="flex gap-4 text-xs">
            <span class="shrink-0 w-32 text-slate-500 font-mono">{date}</span>
            <div>
                <span class="font-bold text-white uppercase tracking-tight">{action}</span>
                <p class="text-slate-400 mt-1">{detail}</p>
            </div>
        </div>
    }
}
