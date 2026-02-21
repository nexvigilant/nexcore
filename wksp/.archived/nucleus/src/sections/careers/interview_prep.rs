//! Interview preparation — practice PV interview questions with reveal answers

use leptos::prelude::*;

#[component]
pub fn InterviewPrepPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Interview Preparation"</h1>
            <p class="mt-2 text-slate-400">"Practice common pharmacovigilance interview questions. Click to reveal model answers."</p>

            <div class="mt-8 space-y-4">
                <QuestionCard
                    category="Technical"
                    question="Explain the difference between a signal and a risk in pharmacovigilance."
                    answer="A signal is information arising from observations that suggests a new potentially causal association (or a new aspect of a known association) between an intervention and an adverse event. A risk is the probability of harm occurring in a defined population. A signal may or may not become a confirmed risk after further evaluation, including statistical analysis, clinical review, and sometimes epidemiological studies."
                />
                <QuestionCard
                    category="Technical"
                    question="What is the PRR and how is it calculated?"
                    answer="The Proportional Reporting Ratio (PRR) compares the proportion of a specific adverse event for a drug of interest with the same proportion for all other drugs. PRR = (a/(a+b)) / (c/(c+d)), where a = target drug + target event, b = target drug + other events, c = other drugs + target event, d = other drugs + other events. A PRR >= 2 with chi-squared >= 4 and N >= 3 suggests a signal."
                />
                <QuestionCard
                    category="Behavioral"
                    question="Describe a time you identified a safety signal that others missed."
                    answer="Use the STAR method: Situation (database/context), Task (your role), Action (analytical approach — what data did you examine, what methodology?), Result (outcome — was a signal confirmed? What was the regulatory impact?). Focus on the analytical rigor and cross-functional communication."
                />
                <QuestionCard
                    category="Regulatory"
                    question="What are the key differences between EU and US pharmacovigilance requirements?"
                    answer="Key differences: (1) EU requires a QPPV (Qualified Person for PV) while US requires a PV responsible person without specific qualification mandate. (2) EU uses EudraVigilance; US uses FAERS. (3) EU requires PSURs/PBRERs at defined intervals; US periodic reports differ in format. (4) EU GVP modules provide detailed guidance; US relies on CFR Title 21 and FDA guidances. (5) RMPs are mandatory in EU; REMS are case-by-case in US."
                />
                <QuestionCard
                    category="Scenario"
                    question="A physician reports a serious unexpected reaction. Walk through your ICSR processing steps."
                    answer="1. Receive and triage (assess 4 minimum criteria: identifiable reporter, identifiable patient, suspect product, adverse event). 2. Classify seriousness per ICH E2A criteria (death, life-threatening, hospitalization, disability, congenital anomaly, medically important). 3. Assess expectedness against reference safety information (SmPC/label). 4. Code using MedDRA (PT and SOC). 5. Assess causality (WHO-UMC or Naranjo). 6. Submit expedited report to authorities within 15 calendar days (serious unexpected). 7. Follow-up for additional information. 8. Include in aggregate analyses (PSUR/PBRER)."
                />
            </div>
        </div>
    }
}

#[component]
fn QuestionCard(category: &'static str, question: &'static str, answer: &'static str) -> impl IntoView {
    let revealed = RwSignal::new(false);

    let cat_color = match category {
        "Technical" => "text-cyan-400 bg-cyan-500/10",
        "Behavioral" => "text-violet-400 bg-violet-500/10",
        "Regulatory" => "text-amber-400 bg-amber-500/10",
        "Scenario" => "text-emerald-400 bg-emerald-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div
            class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors cursor-pointer"
            on:click=move |_| revealed.update(|v| *v = !*v)
        >
            <div class="flex items-center justify-between">
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {cat_color}")>{category}</span>
                <span class="text-xs text-slate-600 font-mono">
                    {move || if revealed.get() { "HIDE" } else { "REVEAL" }}
                </span>
            </div>
            <p class="mt-3 font-medium text-white">{question}</p>
            {move || if revealed.get() {
                view! {
                    <div class="mt-4 rounded-lg bg-slate-950 border border-slate-800 p-4">
                        <p class="text-sm text-slate-300 leading-relaxed">{answer}</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <p class="mt-2 text-xs text-slate-500 italic">"Click to reveal model answer"</p>
                }.into_any()
            }}
        </div>
    }
}
