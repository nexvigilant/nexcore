//! SkillsUsed - Display skills invoked during session

use leptos::prelude::*;
use crate::server::get_adventure_state::SkillInfo;

#[component]
pub fn SkillsUsed(skills: Vec<SkillInfo>) -> impl IntoView {
    view! {
        <div class="skills_used bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold text-orange-400 mb-4">"⚡ Skills Used"</h2>
            <SkillList skills=skills />
        </div>
    }
}

#[component]
fn SkillList(skills: Vec<SkillInfo>) -> impl IntoView {
    view! {
        <div class="space-y-2">
            {skills.into_iter().map(render_skill).collect::<Vec<_>>()}
        </div>
    }
}

fn render_skill(skill: SkillInfo) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between p-3 bg-gray-700 rounded">
            <div class="flex items-center gap-2">
                <span class="text-cyan-400 font-mono text-sm">{format!("/{}", skill.name)}</span>
            </div>
            <span class="bg-orange-500 text-white text-xs px-2 py-1 rounded-full">
                {format!("×{}", skill.invocations)}
            </span>
        </div>
    }
}
