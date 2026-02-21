//! Admin: Academy sub-pages — analytics, certificates, pipeline, etc.

use leptos::prelude::*;

/* ------------------------------------------------------------------ */
/*  Academy Analytics                                                  */
/* ------------------------------------------------------------------ */

#[component]
pub fn AcademyAnalyticsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Academy Analytics"</h1>
            <p class="mt-1 text-slate-400">"Detailed platform metrics and learning trends."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">"Enrollment Velocity"</p>
                    <p class="mt-2 text-2xl font-bold text-white font-mono">"+12% <span class='text-xs text-emerald-400 font-normal'>\u{2191}</span>"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">"Avg. Course Time"</p>
                    <p class="mt-2 text-2xl font-bold text-white font-mono">"4.2h"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">"Certificate Minted"</p>
                    <p class="mt-2 text-2xl font-bold text-white font-mono">"156"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">"Learner Retention"</p>
                    <p class="mt-2 text-2xl font-bold text-white font-mono">"82%"</p>
                </div>
            </div>

            <div class="mt-8 rounded-2xl border border-slate-800 bg-slate-900/50 p-12 text-center">
                <p class="text-slate-500 italic">"Charts and trend lines will be rendered here via nexcore-visuals."</p>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Academy Content Pipeline                                           */
/* ------------------------------------------------------------------ */

#[component]
pub fn AcademyPipelinePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Content Pipeline"</h1>
            <p class="mt-1 text-slate-400">"Automated course generation and content validation pipeline."</p>

            <div class="mt-8 space-y-4">
                <PipelineStep title="KSB Extraction" status="Completed" detail="15 domains processed"/>
                <PipelineStep title="Module Scaffolding" status="Running" detail="Generating draft for Signal Detection"/>
                <PipelineStep title="Validation" status="Pending" detail="Waiting for human review"/>
                <PipelineStep title="Publication" status="Pending" detail="—"/>
            </div>
        </div>
    }
}

#[component]
fn PipelineStep(title: &'static str, status: &'static str, detail: &'static str) -> impl IntoView {
    let status_color = match status {
        "Completed" => "text-emerald-400 bg-emerald-500/10",
        "Running" => "text-cyan-400 bg-cyan-500/10",
        _ => "text-slate-500 bg-slate-800",
    };
    view! {
        <div class="flex items-center justify-between rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div>
                <h3 class="font-bold text-white">{title}</h3>
                <p class="text-xs text-slate-500">{detail}</p>
            </div>
            <span class=format!("rounded-full px-3 py-1 text-[10px] font-bold uppercase {status_color}")>{status}</span>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Academy Certificates                                               */
/* ------------------------------------------------------------------ */

#[component]
pub fn AcademyCertificatesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Certificates Management"</h1>
            <p class="mt-1 text-slate-400">"Issue, revoke, and verify learner credentials."</p>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-800/50 text-slate-400 uppercase text-[10px] font-bold tracking-widest">
                        <tr>
                            <th class="px-6 py-3">"Learner"</th>
                            <th class="px-6 py-3">"Course"</th>
                            <th class="px-6 py-3">"Issued"</th>
                            <th class="px-6 py-3">"Status"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-800">
                        <CertRow learner="Alice Johnson" course="Signal Detection Pro" date="2026-02-01" status="Active"/>
                        <CertRow learner="Bob Smith" course="Aggregate Reporting Expert" date="2026-01-15" status="Revoked"/>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn CertRow(
    learner: &'static str,
    course: &'static str,
    date: &'static str,
    status: &'static str,
) -> impl IntoView {
    let status_cls = if status == "Active" {
        "text-emerald-400"
    } else {
        "text-red-400"
    };
    view! {
        <tr class="hover:bg-slate-800/30 transition-colors">
            <td class="px-6 py-4 font-medium text-white">{learner}</td>
            <td class="px-6 py-4 text-slate-400">{course}</td>
            <td class="px-6 py-4 text-slate-500 font-mono">{date}</td>
            <td class=format!("px-6 py-4 font-bold {status_cls}")>{status}</td>
        </tr>
    }
}
