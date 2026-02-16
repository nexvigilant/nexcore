//! Root application component with router
//!
//! 112 routes across 10 sections + auth:
//! - public (18): marketing, legal, checkout
//! - auth (3): signin, signup, reset-password
//! - academy (12): courses, pathways, KSBs, portfolio, review, assessments
//! - community (19): feed, circles, posts, messages, discover, members, analytics
//! - careers (18): assessments (14 tools), skills, mentoring
//! - vigilance (10): signals, guardian, PVDSL, brain, benefit-risk
//! - admin (25): CRUD for all sections + billing, notifications, onboarding
//! - insights (1): analytics dashboard
//! - regulatory (3): intelligence, frameworks, guidelines
//! - solutions (2): consulting hub, templates
//! - profile (1): user profile

use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::{ParamSegment, StaticSegment};

use crate::auth::provide_auth_context;
use crate::components::guards::AuthGuard;
use crate::components::layout::{AppShell, PublicLayout};
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
                /* === Public Pages (with PublicLayout) === */
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
                    <Route path=(StaticSegment("intelligence/article"), ParamSegment("slug")) view=sections::public::ArticleDetailPage/>
                    <Route path=StaticSegment("intelligence/series") view=sections::public::SeriesPage/>
                    <Route path=(StaticSegment("intelligence/series"), ParamSegment("slug")) view=sections::public::SeriesDetailPage/>
                    <Route path=StaticSegment("academy-preview") view=sections::public::AcademyPreviewPage/>
                    <Route path=StaticSegment("community-preview") view=sections::public::CommunityPreviewPage/>
                    <Route path=StaticSegment("careers-preview") view=sections::public::CareersPreviewPage/>
                    <Route path=StaticSegment("ventures") view=sections::public::VenturesPage/>
                    <Route path=StaticSegment("checkout") view=sections::public::CheckoutPage/>
                    <Route path=StaticSegment("checkout/success") view=sections::public::CheckoutSuccessPage/>
                    <Route path=StaticSegment("faq") view=sections::public::FaqPage/>
                    <Route path=StaticSegment("changelog") view=sections::public::ChangelogPage/>
                    <Route path=StaticSegment("contact/thanks") view=sections::public::ContactThankYouPage/>
                    <Route path=StaticSegment("grow") view=sections::public::GrowPage/>
                    <Route path=StaticSegment("schedule") view=sections::public::SchedulePage/>
                    <Route path=StaticSegment("trial") view=sections::public::TrialPage/>
                    <Route path=StaticSegment("doctrine") view=sections::public::DoctrinePage/>
                    <Route path=StaticSegment("guardian") view=sections::public::GuardianPublicPage/>
                    <Route path=StaticSegment("verify") view=sections::public::VerifyPage/>
                    <Route path=StaticSegment("enterprise-readiness") view=sections::public::EnterpriseReadinessPage/>
                </ParentRoute>

                /* === Auth (standalone pages, no shell) === */
                <Route path=StaticSegment("signin") view=crate::auth::SignInPage/>
                <Route path=StaticSegment("signup") view=crate::auth::SignUpPage/>
                <Route path=StaticSegment("reset-password") view=crate::auth::ResetPasswordPage/>

                /* === Legacy NCOS Routes (Protected, migrated to Nucleus) === */
                <ParentRoute path=StaticSegment("") view=ProtectedShell>
                    <Route path=StaticSegment("store") view=sections::tools::StorePage/>
                    <Route path=StaticSegment("more") view=sections::tools::HubPage/>
                    <Route path=StaticSegment("signals") view=sections::vigilance::SignalsPage/>
                    <Route path=StaticSegment("brain") view=sections::vigilance::BrainPage/>
                    <Route path=StaticSegment("causality") view=sections::vigilance::CausalityPage/>
                    <Route path=StaticSegment("pvdsl") view=sections::vigilance::PvdslPage/>
                    <Route path=StaticSegment("skills") view=sections::vigilance::SkillsPage/>
                    <Route path=StaticSegment("benefit-risk") view=sections::vigilance::BenefitRiskPage/>
                    <Route path=StaticSegment("settings") view=sections::vigilance::SettingsPage/>
                </ParentRoute>

                /* === Academy (Protected) === */
                <ParentRoute path=StaticSegment("academy") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::academy::DashboardPage/>
                    <Route path=StaticSegment("courses") view=sections::academy::CoursesPage/>
                    <Route path=StaticSegment("pathways") view=sections::academy::PathwaysPage/>
                    <Route path=StaticSegment("portfolio") view=sections::academy::PortfolioPage/>
                    <Route path=StaticSegment("certificates") view=sections::academy::CertificatesPage/>
                    <Route path=StaticSegment("progress") view=sections::academy::ProgressPage/>
                    <Route path=StaticSegment("skills") view=sections::academy::SkillsPage/>
                    <Route path=(StaticSegment("skills"), ParamSegment("id")) view=sections::academy::KsbDetailPage/>
                    <Route path=StaticSegment("bookmarks") view=sections::academy::BookmarksPage/>
                    <Route path=(StaticSegment("courses"), ParamSegment("slug")) view=sections::academy::CourseDetailPage/>
                    <Route path=(StaticSegment("learn"), ParamSegment("id")) view=sections::academy::LearnPage/>
                    <Route path=StaticSegment("assessments") view=sections::academy::AssessmentsPage/>
                    <Route path=StaticSegment("review") view=sections::academy::ReviewPage/>
                    <Route path=StaticSegment("pv-framework") view=sections::academy::PvFrameworkPage/>
                    <Route path=StaticSegment("gvp-modules") view=sections::academy::GvpModulesPage/>
                    <Route path=(StaticSegment("gvp-modules"), ParamSegment("code")) view=sections::academy::GvpModuleDetailPage/>
                    <Route path=StaticSegment("gvp-curriculum") view=sections::academy::GvpCurriculumPage/>
                    <Route path=StaticSegment("gvp-progress") view=sections::academy::GvpProgressPage/>
                    <Route path=StaticSegment("gvp-assessments") view=sections::academy::GvpAssessmentsPage/>
                    <Route path=StaticSegment("gvp-practicum") view=sections::academy::GvpPracticumPage/>
                </ParentRoute>

                /* === Community (Protected) === */
                <ParentRoute path=StaticSegment("community") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::community::FeedPage/>
                    <Route path=StaticSegment("for-you") view=sections::community::ForYouPage/>
                    <Route path=StaticSegment("search") view=sections::community::SearchPage/>
                    <Route path=StaticSegment("create-post") view=sections::community::CreatePostPage/>
                    <Route path=StaticSegment("analytics") view=sections::community::AnalyticsPage/>
                    <Route path=StaticSegment("find-your-home") view=sections::community::FindYourHomePage/>
                    <Route path=StaticSegment("create-circle") view=sections::community::CreateCirclePage/>
                    <ParentRoute path=StaticSegment("circles") view=Outlet>
                        <Route path=StaticSegment("") view=sections::community::CirclesPage/>
                        <Route path=ParamSegment("id") view=sections::community::CircleDetailPage/>
                        <Route path=(StaticSegment("post"), ParamSegment("postId")) view=sections::community::PostDetailPage/>
                    </ParentRoute>
                    <Route path=StaticSegment("members") view=sections::community::MembersPage/>
                    <Route path=(StaticSegment("members"), ParamSegment("userId")) view=sections::community::MemberProfilePage/>
                    <Route path=(StaticSegment("member"), ParamSegment("userId")) view=sections::community::MemberDetailPage/>
                    <Route path=StaticSegment("messages") view=sections::community::MessagesPage/>
                    <Route path=(StaticSegment("messages"), ParamSegment("conversationId")) view=sections::community::ConversationPage/>
                    <Route path=StaticSegment("discover") view=sections::community::DiscoverPage/>
                    <Route path=StaticSegment("discover/results") view=sections::community::DiscoverResultsPage/>
                    <Route path=StaticSegment("discover/matches") view=sections::community::DiscoverMatchesPage/>
                    <Route path=StaticSegment("notifications") view=sections::community::NotificationsPage/>
                    <Route path=StaticSegment("onboarding") view=sections::community::OnboardingPage/>
                </ParentRoute>

                /* === Careers (Protected) === */
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
                    <Route path=StaticSegment("advisory-readiness") view=sections::careers::AdvisoryReadinessPage/>
                    <Route path=StaticSegment("board-competencies") view=sections::careers::BoardCompetenciesPage/>
                    <Route path=StaticSegment("board-effectiveness") view=sections::careers::BoardEffectivenessPage/>
                    <Route path=StaticSegment("change-readiness") view=sections::careers::ChangeReadinessPage/>
                    <Route path=StaticSegment("fellowship-evaluator") view=sections::careers::FellowshipEvaluatorPage/>
                    <Route path=StaticSegment("hidden-job-market") view=sections::careers::HiddenJobMarketPage/>
                    <Route path=StaticSegment("performance-conditions") view=sections::careers::PerformanceConditionsPage/>
                    <Route path=StaticSegment("signal-decision") view=sections::careers::SignalDecisionPage/>
                    <Route path=StaticSegment("startup-health") view=sections::careers::StartupHealthPage/>
                </ParentRoute>

                /* === Vigilance (Protected) === */
                <ParentRoute path=StaticSegment("vigilance") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::vigilance::DashboardPage/>
                    <Route path=StaticSegment("signals") view=sections::vigilance::SignalsPage/>
                    <Route path=StaticSegment("faers") view=sections::vigilance::FaersPage/>
                    <Route path=StaticSegment("guardian") view=sections::vigilance::GuardianPage/>
                    <Route path=StaticSegment("brain") view=sections::vigilance::BrainPage/>
                    <Route path=StaticSegment("causality") view=sections::vigilance::CausalityPage/>
                    <Route path=StaticSegment("pvdsl") view=sections::vigilance::PvdslPage/>
                    <Route path=StaticSegment("reporting") view=sections::vigilance::ReportingPage/>
                    <Route path=StaticSegment("skills") view=sections::vigilance::SkillsPage/>
                    <Route path=StaticSegment("benefit-risk") view=sections::vigilance::BenefitRiskPage/>
                    <Route path=StaticSegment("drug-safety") view=sections::vigilance::DrugSafetyPage/>
                    <Route path=StaticSegment("seriousness") view=sections::vigilance::SeriousnessPage/>
                    <Route path=StaticSegment("settings") view=sections::vigilance::SettingsPage/>
                </ParentRoute>

                /* === Admin (Protected) === */
                <ParentRoute path=StaticSegment("admin") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::admin::DashboardPage/>
                    <ParentRoute path=StaticSegment("academy") view=Outlet>
                        <Route path=StaticSegment("") view=sections::admin::AcademyAdminPage/>
                        <Route path=StaticSegment("analytics") view=sections::admin::AcademyAnalyticsPage/>
                        <Route path=StaticSegment("pipeline") view=sections::admin::AcademyPipelinePage/>
                        <Route path=StaticSegment("certificates") view=sections::admin::AcademyCertificatesPage/>
                        <Route path=StaticSegment("courses") view=sections::admin::AcademyCoursesPage/>
                        <Route path=StaticSegment("courses-new") view=sections::admin::AcademyCoursesNewPage/>
                        <Route path=StaticSegment("courses-generate") view=sections::admin::AcademyCoursesGeneratePage/>
                        <Route path=StaticSegment("framework") view=sections::admin::AcademyFrameworkPage/>
                        <Route path=StaticSegment("framework-browser") view=sections::admin::AcademyFrameworkBrowserPage/>
                        <Route path=StaticSegment("ksb") view=sections::admin::AcademyKsbPage/>
                        <Route path=StaticSegment("ksb-builder") view=sections::admin::AcademyKsbBuilderPage/>
                        <Route path=StaticSegment("ksb-management") view=sections::admin::AcademyKsbManagementPage/>
                        <Route path=StaticSegment("learners") view=sections::admin::AcademyLearnersPage/>
                        <Route path=StaticSegment("my-work") view=sections::admin::AcademyMyWorkPage/>
                        <Route path=StaticSegment("operations") view=sections::admin::AcademyOperationsPage/>
                        <Route path=StaticSegment("content-pipeline") view=sections::admin::AcademyContentPipelinePage/>
                        <Route path=StaticSegment("pdc") view=sections::admin::AcademyPdcPage/>
                        <Route path=StaticSegment("pv-domains") view=sections::admin::AcademyPvDomainsPage/>
                        <Route path=StaticSegment("resources") view=sections::admin::AcademyResourcesPage/>
                        <Route path=StaticSegment("skills") view=sections::admin::AcademySkillsAdminPage/>
                    </ParentRoute>
                    <ParentRoute path=StaticSegment("community") view=Outlet>
                        <Route path=StaticSegment("") view=sections::admin::CommunityAdminPage/>
                        <Route path=StaticSegment("moderation") view=sections::admin::ModerationQueuePage/>
                        <Route path=StaticSegment("circles") view=sections::admin::CircleManagementPage/>
                        <Route path=StaticSegment("analytics") view=sections::admin::CommunityAnalyticsPage/>
                        <Route path=StaticSegment("badges") view=sections::admin::CommunityBadgesPage/>
                        <Route path=StaticSegment("circles-admin") view=sections::admin::CommunityCirclesPage/>
                        <Route path=StaticSegment("discovery") view=sections::admin::CommunityDiscoveryPage/>
                        <Route path=StaticSegment("messages") view=sections::admin::CommunityMessagesPage/>
                        <Route path=StaticSegment("moderation-detail") view=sections::admin::CommunityModerationPage/>
                        <Route path=StaticSegment("posts") view=sections::admin::CommunityPostsPage/>
                    </ParentRoute>
                    <ParentRoute path=StaticSegment("content") view=Outlet>
                        <Route path=StaticSegment("") view=sections::admin::ContentAdminPage/>
                        <Route path=StaticSegment("freshness") view=sections::admin::ContentFreshnessPage/>
                        <Route path=StaticSegment("validation") view=sections::admin::ContentValidationPage/>
                    </ParentRoute>
                    <Route path=StaticSegment("ventures") view=sections::admin::VenturesAdminPage/>
                    <ParentRoute path=StaticSegment("intelligence") view=Outlet>
                        <Route path=StaticSegment("") view=sections::admin::IntelligenceAdminPage/>
                        <Route path=StaticSegment("sources") view=sections::admin::DataSourcesPage/>
                        <Route path=StaticSegment("pipeline") view=sections::admin::SignalPipelinePage/>
                        <Route path=StaticSegment("new") view=sections::admin::IntelligenceNewPage/>
                        <Route path=StaticSegment("detail") view=sections::admin::IntelligenceDetailPage/>
                        <Route path=ParamSegment("slug") view=sections::admin::IntelligenceDetailPage/>
                    </ParentRoute>
                    <ParentRoute path=StaticSegment("leads") view=Outlet>
                        <Route path=StaticSegment("") view=sections::admin::WebsiteLeadsPage/>
                        <Route path=StaticSegment("consulting") view=sections::admin::LeadsConsultingPage/>
                        <Route path=StaticSegment("contact") view=sections::admin::LeadsContactPage/>
                        <Route path=StaticSegment("quiz-sessions") view=sections::admin::LeadsQuizSessionsPage/>
                    </ParentRoute>
                    <ParentRoute path=StaticSegment("users") view=Outlet>
                        <Route path=StaticSegment("") view=sections::admin::UsersAdminPage/>
                        <Route path=ParamSegment("userId") view=sections::admin::UserDetailPage/>
                    </ParentRoute>
                    <Route path=StaticSegment("settings") view=sections::admin::SettingsAdminPage/>
                    <Route path=StaticSegment("media") view=sections::admin::MediaPage/>
                    <Route path=StaticSegment("research") view=sections::admin::ResearchPage/>
                    <Route path=StaticSegment("billing") view=sections::admin::BillingAdminPage/>
                    <Route path=StaticSegment("careers") view=sections::admin::CareersAdminPage/>
                    <Route path=StaticSegment("vigilance") view=sections::admin::VigilanceAdminPage/>
                    <Route path=StaticSegment("regulatory") view=sections::admin::RegulatoryAdminPage/>
                    <Route path=StaticSegment("solutions") view=sections::admin::SolutionsAdminPage/>
                    <Route path=StaticSegment("insights") view=sections::admin::InsightsAdminPage/>
                    <Route path=StaticSegment("notifications") view=sections::admin::NotificationsAdminPage/>
                    <Route path=StaticSegment("onboarding") view=sections::admin::OnboardingAdminPage/>
                    <Route path=StaticSegment("waitlist") view=sections::admin::WaitlistPage/>
                    <Route path=StaticSegment("affiliate-applications") view=sections::admin::AffiliateApplicationsPage/>
                </ParentRoute>

                /* === Feature Pages (Protected) === */
                <ParentRoute path=StaticSegment("forge") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::forge::ForgePage/>
                </ParentRoute>
                <ParentRoute path=StaticSegment("tools") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::tools::HubPage/>
                    <Route path=StaticSegment("store") view=sections::tools::StorePage/>
                    <Route path=StaticSegment("codegen") view=sections::tools::CodeGenPage/>
                    <Route path=StaticSegment("debug") view=sections::tools::DebugPage/>
                    <Route path=StaticSegment("perf") view=sections::tools::PerfPage/>
                    <Route path=StaticSegment("api-explorer") view=sections::tools::ApiExplorerPage/>
                    <Route path=StaticSegment("visualizer") view=sections::tools::ArchVisualizerPage/>
                    <Route path=StaticSegment("storage") view=sections::tools::ArtifactManagerPage/>
                    <Route path=StaticSegment("registry") view=sections::tools::RegistryHudPage/>
                </ParentRoute>
                <ParentRoute path=StaticSegment("insights") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::insights::HubPage/>
                </ParentRoute>
                <ParentRoute path=StaticSegment("regulatory") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::regulatory::OverviewPage/>
                    <Route path=StaticSegment("guidelines") view=sections::regulatory::GuidelinesPage/>
                    <Route path=StaticSegment("live") view=sections::regulatory::LiveFeedPage/>
                    <Route path=StaticSegment("dashboard") view=sections::regulatory::DashboardPage/>
                    <Route path=StaticSegment("directory") view=sections::regulatory::DirectoryPage/>
                    <Route path=StaticSegment("timelines") view=sections::regulatory::TimelinesPage/>
                    <Route path=StaticSegment("glossary") view=sections::regulatory::GlossaryPage/>
                </ParentRoute>
                <ParentRoute path=StaticSegment("solutions") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::solutions::HubPage/>
                    <Route path=StaticSegment("templates") view=sections::solutions::TemplatesPage/>
                </ParentRoute>

                /* === Profile (Protected) === */
                <ParentRoute path=StaticSegment("profile") view=ProtectedShell>
                    <Route path=StaticSegment("") view=sections::profile::ProfilePage/>
                    <Route path=StaticSegment("settings") view=sections::profile::SettingsPage/>
                    <Route path=StaticSegment("subscription") view=sections::profile::SubscriptionPage/>
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
