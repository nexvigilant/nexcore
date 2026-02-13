use crate::components::card::Card;
use leptos::prelude::*;

/// Causality assessment page — Naranjo, WHO-UMC
/// Tier: T3 (domain — PV causality)
#[component]
pub fn CausalityPage() -> impl IntoView {
    let active_tab = RwSignal::new("naranjo");

    view! {
        <div class="page causality">
            <h1 class="page-title">"Causality Assessment"</h1>

            <div class="tab-bar">
                <button
                    class=move || { if active_tab.get() == "naranjo" { "tab active" } else { "tab" } }
                    on:click=move |_| active_tab.set("naranjo")
                >"Naranjo"</button>
                <button
                    class=move || { if active_tab.get() == "who-umc" { "tab active" } else { "tab" } }
                    on:click=move |_| active_tab.set("who-umc")
                >"WHO-UMC"</button>
            </div>

            <Show when=move || active_tab.get() == "naranjo">
                <NaranjoSection/>
            </Show>

            <Show when=move || active_tab.get() == "who-umc">
                <WhoUmcSection/>
            </Show>
        </div>
    }
}

/// Naranjo algorithm section as its own component
#[component]
fn NaranjoSection() -> impl IntoView {
    let naranjo_score = RwSignal::new(0_i32);
    let naranjo_category = RwSignal::new(String::new());
    let show_result = RwSignal::new(false);

    let answers: Vec<RwSignal<i32>> = (0..10).map(|_| RwSignal::new(0_i32)).collect();

    let questions: &[&str] = &[
        "1. Previous conclusive reports?",
        "2. Event appeared after drug administered?",
        "3. Reaction improved when drug discontinued?",
        "4. Reaction reappeared on re-administration?",
        "5. Alternative causes possible?",
        "6. Reaction with placebo?",
        "7. Drug in blood at toxic concentration?",
        "8. Reaction worse with increased dose?",
        "9. Similar reaction to same/similar drug before?",
        "10. Confirmed by objective evidence?",
    ];

    let question_views = questions
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let answer = answers[i];
            let yes_class = move || {
                if answer.get() > 0 {
                    "q-btn active"
                } else {
                    "q-btn"
                }
            };
            let unk_class = move || {
                if answer.get() == 0 {
                    "q-btn active"
                } else {
                    "q-btn"
                }
            };
            let no_class = move || {
                if answer.get() < 0 {
                    "q-btn active"
                } else {
                    "q-btn"
                }
            };
            let yes_val = if i == 4 || i == 5 { -1 } else { 1 };
            let no_val = if i == 4 || i == 5 { 1 } else { -1 };
            let q_text = q.to_string();
            view! {
                <div class="naranjo-q">
                    <p class="q-text">{q_text}</p>
                    <div class="q-options">
                        <button class=yes_class
                            on:click=move |_| answer.set(yes_val)
                        >"Yes"</button>
                        <button class=unk_class
                            on:click=move |_| answer.set(0)
                        >"Unknown"</button>
                        <button class=no_class
                            on:click=move |_| answer.set(no_val)
                        >"No"</button>
                    </div>
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <Card title="Naranjo Algorithm">
            <div class="naranjo-questions">
                {question_views}
            </div>

            <button class="btn-primary"
                on:click=move |_| {
                    let total: i32 = answers.iter().map(|a| a.get()).sum();
                    naranjo_score.set(total);
                    let cat = match total {
                        9.. => "Definite",
                        5..=8 => "Probable",
                        1..=4 => "Possible",
                        _ => "Doubtful",
                    };
                    naranjo_category.set(cat.to_string());
                    show_result.set(true);
                }
            >"Calculate Score"</button>
        </Card>

        <Show when=move || show_result.get()>
            <Card title="Result">
                <div class="causality-result">
                    <span class="result-score">{move || naranjo_score.get()}</span>
                    <span class="result-category">{move || naranjo_category.get()}</span>
                </div>
            </Card>
        </Show>
    }
}

/// WHO-UMC section
#[component]
fn WhoUmcSection() -> impl IntoView {
    view! {
        <Card title="WHO-UMC Assessment">
            <p class="card-hint">"WHO-UMC causality assessment categories"</p>
            <div class="who-categories">
                <div class="who-cat">"Certain \u{2014} clear temporal, confirmed by rechallenge"</div>
                <div class="who-cat">"Probable \u{2014} reasonable temporal, unlikely alternative"</div>
                <div class="who-cat">"Possible \u{2014} reasonable temporal, possible alternative"</div>
                <div class="who-cat">"Unlikely \u{2014} improbable temporal relationship"</div>
                <div class="who-cat">"Conditional \u{2014} more data needed"</div>
                <div class="who-cat">"Unassessable \u{2014} insufficient information"</div>
            </div>
        </Card>
    }
}
