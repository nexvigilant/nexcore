//! Dashboard - Main Adventure HUD Component

use leptos::prelude::*;

use crate::server::get_adventure_state::{get_adventure_state, AdventureState, Milestone};
use crate::components::{TaskTracker, SessionStats, SkillsUsed, SignalCascadeVisualizer};

/// Main dashboard view
#[component]
pub fn Dashboard() -> impl IntoView {
    let adventure = Resource::new(|| (), |_| get_adventure_state());

    view! {
        <div class="dashboard min-h-screen bg-gray-900 text-white p-6">
            <DashboardHeader adventure=adventure />
            <DashboardGrid adventure=adventure />
            <SignalCascadeVisualizer />
            <MilestonesSection adventure=adventure />
        </div>
    }
}

/// Header with session name
#[component]
fn DashboardHeader(adventure: Resource<Result<AdventureState, ServerFnError>>) -> impl IntoView {
    view! {
        <header class="mb-8">
            <h1 class="text-3xl font-bold text-cyan-400">"🗺️ Adventure HUD"</h1>
            <Suspense fallback=|| view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || adventure.get().map(|r| r.map(|s| s.session_name).unwrap_or_default())}
            </Suspense>
        </header>
    }
}

/// Three-column grid layout
#[component]
fn DashboardGrid(adventure: Resource<Result<AdventureState, ServerFnError>>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <StatsColumn adventure=adventure />
            <TasksColumn adventure=adventure />
            <SkillsColumn adventure=adventure />
        </div>
    }
}

#[component]
fn StatsColumn(adventure: Resource<Result<AdventureState, ServerFnError>>) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <div class="animate-pulse bg-gray-800 h-48 rounded-lg"></div> }>
            {move || adventure.get().and_then(|r| r.ok()).map(render_stats)}
        </Suspense>
    }
}

fn render_stats(s: AdventureState) -> impl IntoView {
    view! { <SessionStats 
        duration=s.duration_mins 
        tools=s.tools_called 
        tokens=s.tokens_used 
        level=s.level
        xp=s.basis_xp
        prestige=s.reuse_prestige
        velocity=s.compound_velocity
    /> }
}

#[component]
fn TasksColumn(adventure: Resource<Result<AdventureState, ServerFnError>>) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <div class="animate-pulse bg-gray-800 h-96 rounded-lg"></div> }>
            {move || adventure.get().and_then(|r| r.ok()).map(|s| view! { <TaskTracker tasks=s.tasks /> })}
        </Suspense>
    }
}

#[component]
fn SkillsColumn(adventure: Resource<Result<AdventureState, ServerFnError>>) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <div class="animate-pulse bg-gray-800 h-48 rounded-lg"></div> }>
            {move || adventure.get().and_then(|r| r.ok()).map(|s| view! { <SkillsUsed skills=s.skills_used /> })}
        </Suspense>
    }
}

#[component]
fn MilestonesSection(adventure: Resource<Result<AdventureState, ServerFnError>>) -> impl IntoView {
    view! {
        <div class="mt-8 bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold text-yellow-400 mb-4">"🏆 Milestones"</h2>
            <Suspense fallback=|| view! { <div></div> }>
                {move || adventure.get().and_then(|r| r.ok()).map(render_milestones)}
            </Suspense>
        </div>
    }
}

fn render_milestones(s: AdventureState) -> impl IntoView {
    s.milestones.into_iter().map(render_milestone).collect::<Vec<_>>()
}

fn render_milestone(m: Milestone) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 p-3 bg-gray-700 rounded mb-2">
            <span class="text-2xl">"✨"</span>
            <div>
                <p class="font-medium text-green-400">{m.name}</p>
                <p class="text-sm text-gray-400">{m.description}</p>
            </div>
        </div>
    }
}
