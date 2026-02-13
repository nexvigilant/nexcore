/// Skill card — ∂ Boundary (compliance) + σ Sequence (tags)
use leptos::prelude::*;

use crate::api::skills::SkillSummary;

#[component]
pub fn SkillCard(skill: SkillSummary, on_tap: impl Fn() + 'static) -> impl IntoView {
    let compliance_class = match skill.compliance.to_lowercase().as_str() {
        "diamond" => "compliance-diamond",
        "platinum" => "compliance-platinum",
        "gold" => "compliance-gold",
        "silver" => "compliance-silver",
        _ => "compliance-bronze",
    };

    view! {
        <button class="skill-card" on:click=move |_| on_tap()>
            <div class="skill-card-header">
                <span class="skill-name">{skill.name.clone()}</span>
                <span class={format!("compliance-badge {compliance_class}")}>{skill.compliance.clone()}</span>
            </div>
            <p class="skill-desc">{skill.description.clone()}</p>
            <div class="skill-tags">
                {skill.tags.iter().take(3).map(|t| {
                    view! { <span class="skill-tag">{t.clone()}</span> }
                }).collect::<Vec<_>>()}
            </div>
        </button>
    }
}
