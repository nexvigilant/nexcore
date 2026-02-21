//! Career assessments — 14 structured career tools

use leptos::prelude::*;

#[component]
pub fn AssessmentsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Career Assessments"</h1>
            <p class="mt-2 text-slate-400">"14 structured assessment tools for career planning"</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2">
                <AssessmentCard title="Competency Self-Assessment" category="Skills" time="15 min" desc="Rate yourself across all 15 PV domains to identify strengths and gaps"/>
                <AssessmentCard title="Interview Preparation" category="Job Search" time="30 min" desc="Practice common PV interview questions with model answers"/>
                <AssessmentCard title="Career Path Alignment" category="Planning" time="20 min" desc="Discover which PV career paths best match your profile"/>
                <AssessmentCard title="Skills Gap Analysis" category="Skills" time="25 min" desc="Compare your current skills to role requirements"/>
                <AssessmentCard title="Mentoring Readiness" category="Development" time="10 min" desc="Assess readiness to be a mentor or find the right mentor"/>
                <AssessmentCard title="Leadership Style" category="Development" time="15 min" desc="Understand your leadership tendencies in safety teams"/>
                <AssessmentCard title="Salary Benchmarking" category="Job Search" time="10 min" desc="Compare your compensation to market rates by region and role"/>
                <AssessmentCard title="Network Mapping" category="Planning" time="20 min" desc="Visualize and strengthen your professional network"/>
                <AssessmentCard title="Technical Writing" category="Skills" time="30 min" desc="Assess narrative writing and regulatory document quality"/>
                <AssessmentCard title="Regulatory Knowledge" category="Skills" time="25 min" desc="Test knowledge of ICH, EU, FDA regulatory frameworks"/>
                <AssessmentCard title="Work-Life Balance" category="Wellness" time="10 min" desc="Evaluate and improve your professional sustainability"/>
                <AssessmentCard title="Presentation Skills" category="Development" time="15 min" desc="Self-assess your ability to present safety findings"/>
                <AssessmentCard title="Cross-functional Skills" category="Skills" time="20 min" desc="Rate collaboration with clinical, regulatory, and medical teams"/>
                <AssessmentCard title="Innovation Readiness" category="Development" time="15 min" desc="Assess openness to new PV technologies and methodologies"/>
            </div>
        </div>
    }
}

#[component]
fn AssessmentCard(
    title: &'static str,
    category: &'static str,
    time: &'static str,
    desc: &'static str,
) -> impl IntoView {
    let cat_color = match category {
        "Skills" => "text-emerald-400 bg-emerald-500/10",
        "Job Search" => "text-amber-400 bg-amber-500/10",
        "Planning" => "text-cyan-400 bg-cyan-500/10",
        "Development" => "text-violet-400 bg-violet-500/10",
        "Wellness" => "text-pink-400 bg-pink-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-amber-500/30 transition-colors cursor-pointer">
            <div class="flex items-center gap-2">
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {cat_color}")>{category}</span>
                <span class="text-xs text-slate-500">{time}</span>
            </div>
            <h3 class="mt-2 font-semibold text-white">{title}</h3>
            <p class="mt-1 text-sm text-slate-400">{desc}</p>
        </div>
    }
}
