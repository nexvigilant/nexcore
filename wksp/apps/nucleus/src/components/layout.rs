//! App shell layout — header, sidebar, main content area

use leptos::prelude::*;
use crate::auth::use_auth;

/// App shell wrapping authenticated pages
#[component]
pub fn AppShell(children: Children) -> impl IntoView {
    view! {
        <div class="flex min-h-screen flex-col bg-slate-950 text-slate-100">
            <Header/>
            <div class="flex flex-1 overflow-hidden">
                <DesktopSidebar/>
                <main class="flex-1 overflow-y-auto relative custom-scrollbar">
                    // Scanline overlay for that tech feel
                    <div class="fixed inset-0 pointer-events-none opacity-[0.03] bg-[linear-gradient(rgba(18,16,16,0)_50%,rgba(0,0,0,0.25)_50%),linear-gradient(90deg,rgba(255,0,0,0.06),rgba(0,255,0,0.02),rgba(0,0,255,0.06))] bg-[length:100%_2px,3px_100%] z-50"></div>
                    
                    <div class="relative z-10">
                        {children()}
                    </div>
                </main>
            </div>
            <MobileNav/>
        </div>
    }
}

/// Public layout without sidebar (for marketing pages)
#[component]
pub fn PublicLayout(children: Children) -> impl IntoView {
    view! {
        <div class="flex min-h-screen flex-col bg-slate-950 text-slate-100">
            <Header/>
            <main class="flex-1">
                {children()}
            </main>
            <Footer/>
        </div>
    }
}

/// Top navigation header — auth-aware
#[component]
fn Header() -> impl IntoView {
    let auth = use_auth();

    view! {
        <header class="sticky top-0 z-[60] border-b border-slate-800/50 bg-slate-950/80 backdrop-blur-xl">
            <div class="flex h-16 items-center justify-between px-6">
                <div class="flex items-center gap-8">
                    <a href="/" class="flex items-center gap-2 group">
                        <div class="h-8 w-8 rounded bg-cyan-500 flex items-center justify-center shadow-[0_0_15px_rgba(34,211,238,0.4)] group-hover:shadow-[0_0_20px_rgba(34,211,238,0.6)] transition-all">
                            <span class="text-slate-950 font-black text-xl font-mono">"λ"</span>
                        </div>
                        <span class="text-xl font-black text-white font-mono tracking-tighter uppercase group-hover:text-cyan-400 transition-colors">"NUCLEUS"</span>
                    </a>
                    
                    <nav class="hidden gap-6 lg:flex">
                        <TopNavLink href="/academy" label="ACADEMY"/>
                        <TopNavLink href="/community" label="COMMUNITY"/>
                        <TopNavLink href="/careers" label="CAREERS"/>
                        <TopNavLink href="/vigilance" label="VIGILANCE"/>
                    </nav>
                </div>

                <div class="flex items-center gap-4">
                    {move || if auth.is_authenticated.get() {
                        let user_display = auth.user.get()
                            .and_then(|u| u.display_name.clone().or(Some(u.email.clone())))
                            .unwrap_or_else(|| "OPERATOR".to_string());
                        let auth_clone = auth.clone();
                        view! {
                            <div class="flex items-center gap-4">
                                <div class="hidden sm:flex flex-col items-end">
                                    <span class="text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"Active Session"</span>
                                    <span class="text-xs font-mono font-bold text-cyan-400 uppercase tracking-tight">{user_display}</span>
                                </div>
                                <a href="/profile" class="h-10 w-10 rounded-full border border-slate-800 bg-slate-900 flex items-center justify-center hover:glow-border-cyan transition-all overflow-hidden">
                                    <span class="text-xs font-mono font-bold text-slate-500">"ID"</span>
                                </a>
                                <button
                                    class="rounded-lg border border-red-500/30 px-3 py-1.5 text-[10px] font-mono font-bold text-red-400 hover:bg-red-500/10 transition-all uppercase tracking-widest"
                                    on:click=move |_| auth_clone.sign_out()
                                >
                                    "Eject"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <a href="/signin" class="rounded-lg bg-cyan-600 px-6 py-2 text-xs font-mono font-bold text-white hover:bg-cyan-500 transition-all uppercase tracking-widest shadow-lg shadow-cyan-900/20">"INITIALIZE"</a>
                        }.into_any()
                    }}
                </div>
            </div>
        </header>
    }
}

#[component]
fn TopNavLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a href=href class="text-[11px] font-mono font-bold text-slate-500 hover:text-cyan-400 transition-colors uppercase tracking-[0.2em]">
            {label}
        </a>
    }
}

/// Desktop sidebar navigation
#[component]
fn DesktopSidebar() -> impl IntoView {
    view! {
        <aside class="hidden w-64 flex-shrink-0 border-r border-slate-800/50 bg-slate-950 p-6 lg:block overflow-y-auto custom-scrollbar">
            <nav class="space-y-8">
                <SidebarSection title="OPERATIONS" items=vec![
                    ("/academy", "DASHBOARD"),
                    ("/academy/courses", "CURRICULUM"),
                    ("/academy/pathways", "PATHWAYS"),
                    ("/academy/skills", "KSB TAXONOMY"),
                    ("/academy/portfolio", "PORTFOLIO"),
                ] />

                <SidebarSection title="COMMUNICATIONS" items=vec![
                    ("/community", "DATA FEED"),
                    ("/community/circles", "CIRCLES"),
                    ("/community/messages", "MESSAGES"),
                    ("/community/discover", "DISCOVERY"),
                ] />

                <SidebarSection title="VIGILANCE" items=vec![
                    ("/vigilance", "CORE HUD"),
                    ("/vigilance/signals", "SIGNAL SCAN"),
                    ("/vigilance/guardian", "GUARDIAN-AV"),
                    ("/vigilance/pvdsl", "PVDSL STUDIO"),
                    ("/vigilance/reporting", "AUDIT TRAIL"),
                ] />

                <SidebarSection title="ADMIN" items=vec![
                    ("/admin", "SYSTEM STATUS"),
                    ("/admin/ventures", "VENTURE GOV"),
                    ("/admin/users", "IDENTITY MGT"),
                    ("/admin/settings", "PROTOCOLS"),
                ] />
            </nav>
            
            <div class="mt-12 pt-8 border-t border-slate-800/50">
                <div class="flex items-center gap-3 text-slate-600">
                    <span class="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    <span class="text-[9px] font-mono font-bold uppercase tracking-widest">"System Link Stable"</span>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn SidebarSection(title: &'static str, items: Vec<(&'static str, &'static str)>) -> impl IntoView {
    view! {
        <div>
            <h3 class="mb-4 text-[10px] font-mono font-bold uppercase tracking-[0.3em] text-slate-600">
                "// " {title}
            </h3>
            <div class="space-y-1">
                {items.into_iter().map(|(href, label)| view! {
                    <SidebarLink href=href label=label />
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn SidebarLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a href=href class="group flex items-center justify-between rounded-lg px-3 py-2 text-[11px] font-mono font-bold text-slate-400 hover:bg-slate-900 hover:text-cyan-400 transition-all border border-transparent hover:border-slate-800/50">
            <span>{label}</span>
            <span class="opacity-0 group-hover:opacity-100 transition-opacity text-cyan-600">" >>"</span>
        </a>
    }
}

/// Mobile bottom navigation
#[component]
fn MobileNav() -> impl IntoView {
    view! {
        <nav class="fixed bottom-0 left-0 right-0 z-50 border-t border-slate-800/50 bg-slate-950/90 backdrop-blur-xl pb-[env(safe-area-inset-bottom)] lg:hidden">
            <div class="flex h-16 items-center justify-around px-2">
                <MobileNavLink href="/academy" label="LEARN" />
                <MobileNavLink href="/community" label="CONNECT" />
                <div class="h-10 w-10 rounded-xl bg-cyan-600 flex items-center justify-center shadow-lg shadow-cyan-900/30">
                    <span class="text-white font-mono font-black">"λ"</span>
                </div>
                <MobileNavLink href="/vigilance" label="VIGIL" />
                <MobileNavLink href="/profile" label="USER" />
            </div>
        </nav>
    }
}

#[component]
fn MobileNavLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a href=href class="flex flex-col items-center justify-center text-[9px] font-mono font-bold text-slate-500 hover:text-cyan-400 transition-colors tracking-widest">
            {label}
        </a>
    }
}

/// Public footer
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-slate-800/50 bg-slate-950 py-16">
            <div class="mx-auto max-w-6xl px-6">
                <div class="grid grid-cols-2 gap-12 md:grid-cols-4">
                    <FooterSection title="PLATFORM" items=vec![
                        ("/academy", "Academy"),
                        ("/community", "Community"),
                        ("/careers", "Careers"),
                        ("/vigilance", "Vigilance"),
                    ] />
                    <FooterSection title="RESOURCES" items=vec![
                        ("/insights", "Insights Hub"),
                        ("/regulatory", "Regulatory"),
                        ("/solutions", "Solutions"),
                        ("/intelligence", "Intelligence"),
                    ] />
                    <FooterSection title="COMPANY" items=vec![
                        ("/about", "About"),
                        ("/consulting", "Consulting"),
                        ("/contact", "Contact"),
                        ("/changelog", "Changelog"),
                    ] />
                    <FooterSection title="PROTOCOLS" items=vec![
                        ("/privacy", "Privacy"),
                        ("/terms", "Terms"),
                        ("/doctrine", "Doctrine"),
                        ("/verify", "Verify"),
                    ] />
                </div>
                <div class="mt-16 pt-8 border-t border-slate-800/50 flex flex-col md:flex-row justify-between items-center gap-4">
                    <div class="flex items-center gap-2">
                        <div class="h-4 w-4 rounded-sm bg-slate-800 flex items-center justify-center">
                            <span class="text-[10px] text-slate-500 font-mono">"λ"</span>
                        </div>
                        <p class="text-xs font-mono font-bold text-slate-600 uppercase tracking-widest">
                            "NEXVIGILANT CORE // VER 0.2.0"
                        </p>
                    </div>
                    <p class="text-[10px] font-mono text-slate-700 uppercase tracking-[0.3em]">
                        "Empowerment Through Vigilance"
                    </p>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn FooterSection(title: &'static str, items: Vec<(&'static str, &'static str)>) -> impl IntoView {
    view! {
        <div>
            <h4 class="mb-6 text-[10px] font-mono font-bold uppercase tracking-[0.3em] text-slate-500">"// " {title}</h4>
            <div class="space-y-3">
                {items.into_iter().map(|(href, label)| view! {
                    <a href=href class="block text-xs font-mono text-slate-600 hover:text-cyan-400 transition-colors uppercase tracking-wider">
                        {label}
                    </a>
                }).collect_view()}
            </div>
        </div>
    }
}