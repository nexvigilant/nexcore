//! Grow hub — professional growth resources landing

use leptos::prelude::*;

#[component]
pub fn GrowPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Grow Your PV Career"</h1>
            <p class="mt-3 text-lg text-slate-400">
                "Structured pathways, assessments, and community support for every stage of your pharmacovigilance career."
            </p>

            <div class="mt-12 grid gap-6 md:grid-cols-3">
                <GrowCard
                    title="Academy"
                    desc="15 PV domains, 1,462 KSBs, structured courses with certificates"
                    href="/academy-preview"
                    color="cyan"
                />
                <GrowCard
                    title="Career Tools"
                    desc="Competency assessments, skills gap analysis, interview prep"
                    href="/careers-preview"
                    color="amber"
                />
                <GrowCard
                    title="Community"
                    desc="Connect with PV professionals, join circles, share knowledge"
                    href="/community-preview"
                    color="violet"
                />
            </div>
        </div>
    }
}

#[component]
fn GrowCard(
    title: &'static str,
    desc: &'static str,
    href: &'static str,
    color: &'static str,
) -> impl IntoView {
    let border = format!("rounded-xl border border-slate-800 bg-slate-900/50 p-8 hover:border-{color}-500/30 transition-colors");
    let title_class = format!("text-xl font-bold text-{color}-400");

    view! {
        <a href=href class=border>
            <h2 class=title_class>{title}</h2>
            <p class="mt-3 text-sm text-slate-400">{desc}</p>
        </a>
    }
}
