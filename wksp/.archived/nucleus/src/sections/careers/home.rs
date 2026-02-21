//! Careers home — career development hub

use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Career Development"</h1>
            <p class="mt-2 text-slate-400">"Tools and assessments for PV career advancement"</p>

            <div class="mt-8 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
                <a href="/careers/skills" class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-amber-500/30 transition-colors">
                    <h3 class="text-lg font-semibold text-amber-400">"Skills Overview"</h3>
                    <p class="mt-2 text-sm text-slate-400">"Map your competencies across all PV domains"</p>
                </a>
                <a href="/careers/assessments" class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-amber-500/30 transition-colors">
                    <h3 class="text-lg font-semibold text-amber-400">"Assessments"</h3>
                    <p class="mt-2 text-sm text-slate-400">"14 structured career planning tools"</p>
                </a>
                <a href="/community/members" class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-amber-500/30 transition-colors">
                    <h3 class="text-lg font-semibold text-amber-400">"Find a Mentor"</h3>
                    <p class="mt-2 text-sm text-slate-400">"Connect with experienced professionals"</p>
                </a>
            </div>

            <div class="mt-12">
                <h2 class="text-xl font-semibold text-white">"Career Paths in PV"</h2>
                <div class="mt-4 grid gap-4 md:grid-cols-2">
                    <CareerPathCard
                        title="Drug Safety Scientist"
                        level="Mid-Career"
                        domains="D08, D09, D10"
                        desc="Focus on signal detection, evaluation, and benefit-risk assessment"
                    />
                    <CareerPathCard
                        title="Case Processing Specialist"
                        level="Entry Level"
                        domains="D02, D03, D05"
                        desc="Handle ICSR intake, coding, assessment, and regulatory submission"
                    />
                    <CareerPathCard
                        title="Risk Management Lead"
                        level="Senior"
                        domains="D07, D10"
                        desc="Design and execute risk management plans and REMS programs"
                    />
                    <CareerPathCard
                        title="PV Technology Specialist"
                        level="Mid-Career"
                        domains="D08, D11"
                        desc="Build and maintain safety databases, signal detection tools, and automation"
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn CareerPathCard(
    title: &'static str,
    level: &'static str,
    domains: &'static str,
    desc: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
            <div class="flex items-center justify-between">
                <h3 class="font-semibold text-white">{title}</h3>
                <span class="rounded-full bg-amber-500/10 px-2.5 py-0.5 text-xs text-amber-400">{level}</span>
            </div>
            <p class="mt-2 text-sm text-slate-400">{desc}</p>
            <p class="mt-2 text-xs text-slate-500">"Domains: "{domains}</p>
        </div>
    }
}
