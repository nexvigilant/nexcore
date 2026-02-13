//! Root application component with mobile router
//!
//! Provides: auth context, online/offline status, bottom tab navigation
//! Routes: 9 page routes + 404 fallback
//!
//! Tier: T3 (full application shell)

use leptos::prelude::*;
use leptos_meta::{ provide_meta_context, Link, Stylesheet, Title };
use leptos_router::{ components::{ Route, Router, Routes }, StaticSegment };

use crate::auth::provide_auth_context;
use crate::components::nav_bar::NavBar;
use crate::pages;

/// Root application component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_context();

    view! {
        <Title text="NCOS — NexCore Operating System"/>
        <Stylesheet id="main" href="/style/main.css"/>
        <Link rel="manifest" href="/manifest.json"/>
        <Link rel="apple-touch-icon" href="/icons/icon-192.png"/>

        <Router>
            <div class="app-shell">
                <main class="app-content">
                    <Routes fallback=|| view! { <NotFound/> }>
                        <Route path=StaticSegment("") view=pages::dashboard::DashboardPage/>
                        <Route path=StaticSegment("store") view=pages::store::StorePage/>
                        <Route path=StaticSegment("signals") view=pages::signals::SignalsPage/>
                        <Route path=StaticSegment("guardian") view=pages::guardian::GuardianPage/>
                        <Route path=StaticSegment("brain") view=pages::brain::BrainPage/>
                        <Route path=StaticSegment("causality") view=pages::causality::CausalityPage/>
                        <Route path=StaticSegment("pvdsl") view=pages::pvdsl::PvdslPage/>
                        <Route path=StaticSegment("skills") view=pages::skills::SkillsPage/>
                        <Route path=StaticSegment("benefit-risk") view=pages::benefit_risk::BenefitRiskPage/>
                        <Route path=StaticSegment("settings") view=pages::settings::SettingsPage/>
                        <Route path=StaticSegment("more") view=pages::more::MorePage/>
                    </Routes>
                </main>
                <NavBar/>
            </div>
        </Router>
    }
}

/// 404 page
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="page not-found">
            <h1>"404"</h1>
            <p>"Page not found"</p>
        </div>
    }
}
