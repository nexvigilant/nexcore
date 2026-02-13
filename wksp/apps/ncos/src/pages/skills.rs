use crate::components::card::{Card, CardLoading};
use leptos::prelude::*;

/// Skills registry browser
/// Tier: T3 (domain — Skills ecosystem)
#[component]
pub fn SkillsPage() -> impl IntoView {
    let skills = RwSignal::new(Vec::<(String, String, Vec<String>)>::new());
    let search = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    view! {
        <div class="page skills">
            <h1 class="page-title">"Skills Registry"</h1>

            <div class="search-bar">
                <input type="search" class="input-field"
                    placeholder="Search skills..."
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
                <button class="btn-primary"
                    disabled=move || loading.get()
                    on:click=move |_| loading.set(true)
                >"Load Skills"</button>
            </div>

            <Show when=move || loading.get()>
                <CardLoading/>
                <CardLoading/>
            </Show>

            <div class="skills-list">
                {move || {
                    let all_skills = skills.get();
                    let filter = search.get().to_lowercase();
                    let filtered: Vec<_> = if filter.is_empty() {
                        all_skills
                    } else {
                        all_skills.into_iter()
                            .filter(|(name, desc, _)| {
                                name.to_lowercase().contains(&filter)
                                || desc.to_lowercase().contains(&filter)
                            })
                            .collect()
                    };

                    if filtered.is_empty() {
                        view! { <p class="card-hint">"No skills found. Tap Load Skills."</p> }.into_any()
                    } else {
                        view! {
                            <div class="skills-grid">
                                {filtered.into_iter().map(|(name, desc, tags)| {
                                    view! {
                                        <Card title="">
                                            <h3 class="skill-name">{name}</h3>
                                            <p class="skill-desc">{desc}</p>
                                            <div class="skill-tags">
                                                {tags.into_iter().map(|t| {
                                                    view! { <span class="tag">{t}</span> }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </Card>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
