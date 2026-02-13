/// Skills browser + executor — σ Sequence (execution) + ∂ Boundary (compliance)
/// Tier: T3 (searchable list + detail + execute)
use leptos::prelude::*;

use crate::api::skills::{self, SkillDetail, SkillExecResult};
use crate::components::exec_output::ExecOutput;
use crate::components::skill_card::SkillCard;

#[component]
pub fn SkillsPage() -> impl IntoView {
    let skill_list = LocalResource::new(|| skills::list_skills());
    let (search, set_search) = signal(String::new());
    let (selected, set_selected) = signal(None::<String>);
    let (detail, set_detail) = signal(None::<Result<SkillDetail, String>>);
    let (exec_result, set_exec_result) = signal(None::<Result<SkillExecResult, String>>);
    let (executing, set_executing) = signal(false);

    // Load detail when selected changes
    let load_detail = move |name: String| {
        set_selected.set(Some(name.clone()));
        set_detail.set(None);
        set_exec_result.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            match skills::get_skill(&name).await {
                Ok(d) => set_detail.set(Some(Ok(d))),
                Err(e) => set_detail.set(Some(Err(e.message))),
            }
        });
    };

    let run_skill = move |_| {
        if let Some(name) = selected.get() {
            set_executing.set(true);
            set_exec_result.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match skills::execute_skill(&name).await {
                    Ok(r) => set_exec_result.set(Some(Ok(r))),
                    Err(e) => set_exec_result.set(Some(Err(e.message))),
                }
                set_executing.set(false);
            });
        }
    };

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"Skills"</h1>
                <p class="page-subtitle">"94 Capabilities"</p>
            </header>

            // Search bar
            <div class="input-group" style="margin-bottom: 16px">
                <input
                    class="input-field"
                    type="text"
                    placeholder="Search skills..."
                    prop:value=search
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let val = ev.target().map(|t| t.unchecked_into::<web_sys::HtmlInputElement>().value()).unwrap_or_default();
                        set_search.set(val);
                    }
                />
            </div>

            // Detail panel (if selected)
            {move || {
                detail.get().map(|d| match d {
                    Ok(det) => {
                        let det2 = det.clone();
                        view! {
                            <div class="skill-detail-panel">
                                <div class="skill-detail-header">
                                    <h2 class="skill-detail-name">{det.name.clone()}</h2>
                                    <button class="btn-close" on:click=move |_| {
                                        set_selected.set(None);
                                        set_detail.set(None);
                                        set_exec_result.set(None);
                                    }>"X"</button>
                                </div>
                                <p class="skill-desc">{det.description.clone()}</p>
                                <div class="skill-meta-grid">
                                    <div class="skill-meta-item">
                                        <span class="skill-meta-label">"Compliance"</span>
                                        <span class="skill-meta-value">{det.compliance.clone()}</span>
                                    </div>
                                    <div class="skill-meta-item">
                                        <span class="skill-meta-label">"SMST Score"</span>
                                        <span class="skill-meta-value">{format!("{:.1}", det.smst_score)}</span>
                                    </div>
                                    <div class="skill-meta-item">
                                        <span class="skill-meta-label">"Scripts"</span>
                                        <span class="skill-meta-value">{if det.has_scripts { "Yes" } else { "No" }}</span>
                                    </div>
                                    <div class="skill-meta-item">
                                        <span class="skill-meta-label">"References"</span>
                                        <span class="skill-meta-value">{if det2.has_references { "Yes" } else { "No" }}</span>
                                    </div>
                                </div>

                                {if det2.has_scripts {
                                    Some(view! {
                                        <button
                                            class="btn-primary btn-execute"
                                            on:click=run_skill
                                            disabled=executing
                                        >
                                            {move || if executing.get() { "Executing..." } else { "Execute Skill" }}
                                        </button>
                                    })
                                } else {
                                    None
                                }}

                                {move || {
                                    exec_result.get().map(|r| match r {
                                        Ok(res) => view! { <ExecOutput result=res /> }.into_any(),
                                        Err(msg) => view! {
                                            <div class="error-card">
                                                <div class="error-msg">{msg.clone()}</div>
                                            </div>
                                        }.into_any(),
                                    })
                                }}
                            </div>
                        }.into_any()
                    },
                    Err(msg) => view! {
                        <div class="error-card">
                            <div class="error-msg">{msg.clone()}</div>
                        </div>
                    }.into_any(),
                })
            }}

            // Skill list
            <Suspense fallback=move || view! { <div class="loading">"Loading skills..."</div> }>
                {move || {
                    skill_list.read().as_ref().map(|result| {
                        match result {
                            Ok(items) => {
                                let query = search.get().to_lowercase();
                                let filtered: Vec<_> = items.iter()
                                    .filter(|s| {
                                        query.is_empty()
                                            || s.name.to_lowercase().contains(&query)
                                            || s.description.to_lowercase().contains(&query)
                                            || s.tags.iter().any(|t| t.to_lowercase().contains(&query))
                                    })
                                    .cloned()
                                    .collect();

                                view! {
                                    <div class="skill-count">{format!("{} skills", filtered.len())}</div>
                                    <div class="skill-list">
                                        {filtered.into_iter().map(|s| {
                                            let name = s.name.clone();
                                            let load = load_detail.clone();
                                            view! {
                                                <SkillCard skill=s on_tap=move || load(name.clone()) />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <div class="error-card">
                                    <div class="error-icon">"!"</div>
                                    <div class="error-msg">{e.message.clone()}</div>
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
