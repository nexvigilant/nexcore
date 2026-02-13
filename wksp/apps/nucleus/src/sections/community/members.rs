//! Members directory

use leptos::prelude::*;

#[component]
pub fn MembersPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Members"</h1>
            <p class="mt-2 text-slate-400">"Connect with healthcare professionals worldwide"</p>

            <div class="mt-6">
                <input
                    type="text"
                    placeholder="Search by name, role, or expertise..."
                    class="w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none focus:ring-1 focus:ring-cyan-500"
                />
            </div>

            <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <MemberCard name="Dr. Sarah Chen" role="Signal Detection Lead" org="Major Pharma Co" domains="D08, D09"/>
                <MemberCard name="James Wilson" role="PV Specialist" org="CRO International" domains="D02, D03, D05"/>
                <MemberCard name="Maria Santos" role="Regulatory Affairs" org="Biotech Startup" domains="D06, D07"/>
                <MemberCard name="Dr. Ahmed Hassan" role="Chief Safety Officer" org="Global Pharma" domains="D07, D08, D10"/>
                <MemberCard name="Lisa Park" role="Literature Reviewer" org="Safety Consultants" domains="D04"/>
                <MemberCard name="Robert Kim" role="Risk Manager" org="Large Pharma" domains="D07, D10"/>
            </div>
        </div>
    }
}

#[component]
fn MemberCard(
    name: &'static str,
    role: &'static str,
    org: &'static str,
    domains: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
            <div class="flex items-center gap-3">
                <div class="h-12 w-12 rounded-full bg-slate-700 flex items-center justify-center text-lg font-bold text-white">
                    {&name[..1]}
                </div>
                <div>
                    <p class="font-medium text-white">{name}</p>
                    <p class="text-xs text-slate-400">{role}</p>
                    <p class="text-xs text-slate-500">{org}</p>
                </div>
            </div>
            <div class="mt-3 flex flex-wrap gap-1">
                {domains.split(", ").map(|d| view! {
                    <span class="rounded bg-slate-800 px-2 py-0.5 text-xs text-slate-400">{d.to_string()}</span>
                }).collect::<Vec<_>>()}
            </div>
            <button class="mt-3 text-xs text-cyan-400 hover:text-cyan-300 transition-colors">"Connect"</button>
        </div>
    }
}
