//! Root application component with router
//!
//! Routes ~167 pages across 7 sections:
//! - public (26): marketing, legal, checkout
//! - academy (19): courses, pathways, KSBs, portfolio
//! - community (21): feed, circles, posts, messages
//! - careers (17): assessments, skills, mentoring
//! - vigilance (11): signals, guardian, PVDSL, brain
//! - admin (63): CRUD for all sections
//! - profile (4): user profile, settings

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::StaticSegment;

use crate::auth::provide_auth_context;
use crate::components::layout::{AppShell, PublicLayout};
use crate::components::guards::AuthGuard;
use crate::sections;

/// Root application component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_context();

    view! {
        <Title text="Nucleus — NexVigilant"/>

        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                // === Public Pages (with PublicLayout) ===
                <ParentRoute path=StaticSegment("") view=PublicShell>
                    <Route path=StaticSegment("") view=sections::public::LandingPage/>
                    <Route path=StaticSegment("about") view=sections::public::AboutPage/>
                    <Route path=StaticSegment("membership") view=sections::public::MembershipPage/>
                    <Route path=StaticSegment("services") view=sections::public::ServicesPage/>
                    <Route path=StaticSegment("consulting") view=sections::public::ConsultingPage/>
                    <Route path=StaticSegment("contact") view=sections::public::ContactPage/>
                    <Route path=StaticSegment("privacy") view=sections::public::PrivacyPage/>
                    <Route path=StaticSegment("terms") view=sections::public::TermsPage/>
                    <Route path=StaticSegment("intelligence") view=sections::public::ArticlesPage/>
                    <Route path=StaticSegment("intelligence/article") view=sections::public::ArticleDetailPage/>
                    <Route path=StaticSegment("intelligence/series") view=sections::public::SeriesPage/>
                    <Route path=StaticSegment("academy-preview") view=sections::public::AcademyPreviewPage/>
                    <Route path=StaticSegment("community-preview") view=sections::public::CommunityPreviewPage/>
                    <Route path=StaticSegment("careers-preview") view=sections::public::CareersPreviewPage/>
                    <Route path=StaticSegment("ventures") view=sections::public::VenturesPage/>
                    <Route path=StaticSegment("checkout") view=sections::public::CheckoutPage/>
                    <Route path=StaticSegment("checkout/success") view=sections::public::CheckoutSuccessPage/>
                    <Route path=StaticSegment("faq") view=sections::public::FaqPage/>
                </ParentRoute>

                // === Auth (standalone pages, no shell) ===
                <Route path=StaticSegment("signin") view=crate::auth::SignInPage/>
                <Route path=StaticSegment("signup") view=crate::auth::SignUpPage/>
                <Route path=StaticSegment("reset-password") view=crate::auth::ResetPasswordPage/>

                // === Academy (Protected with AppShell) ===
                <ParentRoute path=StaticSegment("academy") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::academy::DashboardPage/>
                    <Route path=StaticSegment("courses") view=sections::academy::CoursesPage/>
                    <Route path=StaticSegment("pathways") view=sections::academy::PathwaysPage/>
                    <Route path=StaticSegment("portfolio") view=sections::academy::PortfolioPage/>
                    <Route path=StaticSegment("certificates") view=sections::academy::CertificatesPage/>
                    <Route path=StaticSegment("progress") view=sections::academy::ProgressPage/>
                    <Route path=StaticSegment("skills") view=sections::academy::SkillsPage/>
                    <Route path=StaticSegment("skills/:id") view=sections::academy::KsbDetailPage/>
                    <Route path=StaticSegment("bookmarks") view=sections::academy::BookmarksPage/>
                </ParentRoute>

                // === Community (Protected) ===
                <ParentRoute path=StaticSegment("community") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::community::FeedPage/>
                    <Route path=StaticSegment("create-post") view=sections::community::CreatePostPage/>
                    <Route path=StaticSegment("circles") view=sections::community::CirclesPage/>
                    <Route path=StaticSegment("members") view=sections::community::MembersPage/>
                    <Route path=StaticSegment("messages") view=sections::community::MessagesPage/>
                    <Route path=StaticSegment("discover") view=sections::community::DiscoverPage/>
                    <Route path=StaticSegment("notifications") view=sections::community::NotificationsPage/>
                </ParentRoute>

                // === Careers (Protected) ===
                <ParentRoute path=StaticSegment("careers") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::careers::HomePage/>
                    <Route path=StaticSegment("skills") view=sections::careers::SkillsPage/>
                    <Route path=StaticSegment("assessments") view=sections::careers::AssessmentsPage/>
                    <Route path=StaticSegment("assessment-hub") view=sections::careers::AssessmentHubPage/>
                    <Route path=StaticSegment("competency") view=sections::careers::CompetencyPage/>
                    <Route path=StaticSegment("maturity") view=sections::careers::MaturityPage/>
                    <Route path=StaticSegment("value-proposition") view=sections::careers::ValuePropPage/>
                    <Route path=StaticSegment("interview-prep") view=sections::careers::InterviewPrepPage/>
                    <Route path=StaticSegment("mentoring") view=sections::careers::MentoringPage/>
                </ParentRoute>

                // === Vigilance (Protected) ===
                <ParentRoute path=StaticSegment("vigilance") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::vigilance::DashboardPage/>
                    <Route path=StaticSegment("signals") view=sections::vigilance::SignalsPage/>
                    <Route path=StaticSegment("guardian") view=sections::vigilance::GuardianPage/>
                    <Route path=StaticSegment("brain") view=sections::vigilance::BrainPage/>
                    <Route path=StaticSegment("causality") view=sections::vigilance::CausalityPage/>
                    <Route path=StaticSegment("pvdsl") view=sections::vigilance::PvdslPage/>
                    <Route path=StaticSegment("reporting") view=sections::vigilance::ReportingPage/>
                    <Route path=StaticSegment("skills") view=sections::vigilance::SkillsPage/>
                    <Route path=StaticSegment("benefit-risk") view=sections::vigilance::BenefitRiskPage/>
                    <Route path=StaticSegment("settings") view=sections::vigilance::SettingsPage/>
                </ParentRoute>

                // === Admin (Protected) ===
                <ParentRoute path=StaticSegment("admin") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::admin::DashboardPage/>
                    <Route path=StaticSegment("academy") view=sections::admin::AcademyAdminPage/>
                    <Route path=StaticSegment("community") view=sections::admin::CommunityAdminPage/>
                    <Route path=StaticSegment("content") view=sections::admin::ContentAdminPage/>
                    <Route path=StaticSegment("ventures") view=sections::admin::VenturesAdminPage/>
                    <Route path=StaticSegment("intelligence") view=sections::admin::IntelligenceAdminPage/>
                    <Route path=StaticSegment("users") view=sections::admin::UsersAdminPage/>
                    <Route path=StaticSegment("settings") view=sections::admin::SettingsAdminPage/>
                </ParentRoute>

                // === Profile (Protected) ===
                <ParentRoute path=StaticSegment("profile") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::profile::ProfilePage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}

/// Public page shell — header + content + footer
#[component]
fn PublicShell() -> impl IntoView {
    view! {
        <PublicLayout>
            <Outlet/>
        </PublicLayout>
    }
}

/// Protected page shell — auth guard + app shell (header + sidebar + content)
#[component]
fn ProtectedShell() -> impl IntoView {
    view! {
        <AuthGuard>
            <AppShell>
                <Outlet/>
            </AppShell>
        </AuthGuard>
    }
}

/// 404 page
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen">
            <h1 class="text-6xl font-bold text-slate-400">"404"</h1>
            <p class="mt-4 text-lg text-slate-500">"Page not found"</p>
            <a href="/" class="mt-6 text-cyan-400 hover:text-cyan-300 underline">"Back to Nucleus"</a>
        </div>
    }
}