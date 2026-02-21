//! Course preview page — read-only preview of course content before enrollment

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn CoursePreviewPage() -> impl IntoView {
    let params = use_params_map();
    let _id = move || params.read().get("id").unwrap_or_default();

    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <div>
                <a href="/academy/courses" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Courses"</a>
                <h1 class="text-2xl font-bold text-white mt-2">"Course Preview"</h1>
            </div>

            <div class="bg-gradient-to-r from-cyan-900/30 to-blue-900/30 border border-cyan-800/30 rounded-xl p-8">
                <span class="text-xs text-cyan-400 font-mono uppercase">"Preview Mode"</span>
                <h2 class="text-2xl font-bold text-white mt-2">"Introduction to Pharmacovigilance"</h2>
                <p class="text-slate-300 mt-2">"A comprehensive foundation course covering the principles, regulations, and practices of drug safety monitoring"</p>
                <div class="flex items-center gap-6 mt-4 text-sm text-slate-400">
                    <span>"8 modules"</span>
                    <span>"24 lessons"</span>
                    <span>"~12 hours"</span>
                    <span>"Beginner"</span>
                </div>
                <button class="mt-6 px-6 py-3 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg font-medium transition-colors">
                    "Enroll Now"
                </button>
            </div>

            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Course Outline"</h2>
                <div class="space-y-2">
                    <ModulePreview number=1 title="What is Pharmacovigilance?" lessons=3 duration="90 min"/>
                    <ModulePreview number=2 title="Regulatory Framework" lessons=4 duration="120 min"/>
                    <ModulePreview number=3 title="Individual Case Safety Reports" lessons=3 duration="90 min"/>
                    <ModulePreview number=4 title="Signal Detection" lessons=3 duration="90 min"/>
                    <ModulePreview number=5 title="Aggregate Reporting" lessons=3 duration="90 min"/>
                    <ModulePreview number=6 title="Risk Management" lessons=3 duration="90 min"/>
                    <ModulePreview number=7 title="Benefit-Risk Assessment" lessons=2 duration="60 min"/>
                    <ModulePreview number=8 title="PV Systems & Quality" lessons=3 duration="90 min"/>
                </div>
            </div>

            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Learning Outcomes"</h2>
                <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                    <ul class="space-y-2 text-sm text-slate-300">
                        <li class="flex items-start gap-2">
                            <span class="text-cyan-400 mt-0.5">"+"</span>
                            "Understand the regulatory framework governing pharmacovigilance globally"
                        </li>
                        <li class="flex items-start gap-2">
                            <span class="text-cyan-400 mt-0.5">"+"</span>
                            "Process and assess individual case safety reports (ICSRs)"
                        </li>
                        <li class="flex items-start gap-2">
                            <span class="text-cyan-400 mt-0.5">"+"</span>
                            "Apply signal detection methods to identify potential safety concerns"
                        </li>
                        <li class="flex items-start gap-2">
                            <span class="text-cyan-400 mt-0.5">"+"</span>
                            "Prepare periodic safety update reports (PSURs/PBRERs)"
                        </li>
                        <li class="flex items-start gap-2">
                            <span class="text-cyan-400 mt-0.5">"+"</span>
                            "Conduct structured benefit-risk assessments"
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ModulePreview(
    number: u32,
    #[prop(into)] title: String,
    lessons: u32,
    #[prop(into)] duration: String,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800/50 border border-slate-700/50 rounded-lg p-4 flex items-center justify-between">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded bg-slate-700/50 flex items-center justify-center text-sm text-slate-400 font-mono">
                    {number.to_string()}
                </div>
                <span class="text-white text-sm font-medium">{title}</span>
            </div>
            <div class="flex items-center gap-4 text-xs text-slate-500">
                <span>{format!("{} lessons", lessons)}</span>
                <span>{duration}</span>
            </div>
        </div>
    }
}
