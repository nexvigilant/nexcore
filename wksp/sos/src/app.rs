/// Root application component — Router + 8 routes
/// Tier: T3 (σ Sequence of routes + ς State management)
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::nav::BottomNav;
use crate::pages::{
    academy::AcademyPage, brain::BrainPage, guardian::GuardianPage, home::HomePage,
    pvos::PvosPage, settings::SettingsPage, signals::SignalsPage, skills::SkillsPage,
};

/// Root <App/> — mounts router with bottom tab navigation
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="app-shell">
                <main class="app-content">
                    <Routes fallback=|| view! { <div class="page-center">"404 — Not Found"</div> }>
                        <Route path=path!("/") view=HomePage />
                        <Route path=path!("/signals") view=SignalsPage />
                        <Route path=path!("/guardian") view=GuardianPage />
                        <Route path=path!("/brain") view=BrainPage />
                        <Route path=path!("/settings") view=SettingsPage />
                        <Route path=path!("/pvos") view=PvosPage />
                        <Route path=path!("/skills") view=SkillsPage />
                        <Route path=path!("/academy") view=AcademyPage />
                    </Routes>
                </main>
                <BottomNav />
            </div>
        </Router>
    }
}
