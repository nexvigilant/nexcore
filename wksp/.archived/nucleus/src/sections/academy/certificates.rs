//! Certificates — earned capability verifications

use leptos::prelude::*;

#[component]
pub fn CertificatesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Certificates"</h1>
            <p class="mt-2 text-slate-400">"Your earned capability verifications"</p>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-12 text-center">
                <div class="text-4xl text-slate-600">"--"</div>
                <h3 class="mt-4 text-lg font-medium text-slate-200">"No certificates yet"</h3>
                <p class="mt-2 max-w-md mx-auto text-sm text-slate-400">
                    "Complete capability pathways and pass assessments to earn certificates recognized by the PV professional community."
                </p>
                <a href="/academy/courses" class="mt-6 inline-block rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-medium text-white hover:bg-cyan-500 transition-colors">
                    "Start a Pathway"
                </a>
            </div>

            <div class="mt-12">
                <h2 class="text-xl font-semibold text-white">"Available Certificates"</h2>
                <div class="mt-4 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    <CertPreview title="PV Foundations" domain="D01-D03" requirements="Complete 3 foundation pathways"/>
                    <CertPreview title="Signal Detection Specialist" domain="D08-D09" requirements="Complete signal detection + evaluation"/>
                    <CertPreview title="Causality Assessor" domain="D05" requirements="Pass all 6 causality EPAs"/>
                    <CertPreview title="Risk Management Professional" domain="D07, D10" requirements="Complete risk management + benefit-risk"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn CertPreview(title: &'static str, domain: &'static str, requirements: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h3 class="font-semibold text-white">{title}</h3>
            <p class="mt-1 text-xs text-cyan-400">{domain}</p>
            <p class="mt-2 text-sm text-slate-400">{requirements}</p>
        </div>
    }
}
