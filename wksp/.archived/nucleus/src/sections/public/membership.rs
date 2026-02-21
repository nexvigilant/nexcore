//! Membership pricing page — 3-tier subscription grid

use leptos::prelude::*;

#[component]
pub fn MembershipPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-slate-950 selection:bg-cyan-500/30">
            // Hero
            <section class="relative py-24 px-6 text-center overflow-hidden">
                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-cyan-600/5 rounded-full blur-[100px]"></div>
                <div class="relative z-10 max-w-3xl mx-auto">
                    <h2 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.4em] mb-4">"// SELECT TIER"</h2>
                    <h1 class="text-5xl md:text-6xl font-black font-mono text-white uppercase tracking-tighter">"MEMBERSHIP"</h1>
                    <p class="mt-6 text-lg text-slate-400 font-mono max-w-xl mx-auto">
                        "JOIN THE COMMUNITY ADVANCING PHARMACEUTICAL INTELLIGENCE AND HEALTHCARE CAREERS."
                    </p>
                </div>
            </section>

            // Pricing grid
            <section class="mx-auto max-w-6xl px-6 pb-32">
                <div class="grid gap-8 md:grid-cols-3">
                    <PricingCard
                        tier="COMMUNITY"
                        price="$19"
                        desc="Connect with pharmacovigilance professionals worldwide."
                        features=vec![
                            "Community feed & circles",
                            "Members directory",
                            "Basic Academy access",
                            "Monthly intelligence digest",
                            "Discussion forums",
                        ]
                        plan="community"
                        accent="slate"
                    />
                    <PricingCard
                        tier="PROFESSIONAL"
                        price="$29"
                        desc="Full learning platform and career development toolkit."
                        features=vec![
                            "Everything in Community",
                            "Full Academy course library",
                            "KSB competency tracking",
                            "Career assessment tools",
                            "Peer mentoring access",
                            "Certificate programs",
                        ]
                        plan="professional"
                        accent="cyan"
                    />
                    <PricingCard
                        tier="ENTERPRISE"
                        price="$59"
                        desc="Complete access plus algorithmic intelligence tools."
                        features=vec![
                            "Everything in Professional",
                            "Vigilance dashboard",
                            "Signal detection (6 methods)",
                            "Causality assessment tools",
                            "Consulting priority queue",
                            "Custom reporting",
                            "API access",
                        ]
                        plan="enterprise"
                        accent="amber"
                    />
                </div>

                // FAQ
                <div class="mt-24 text-center">
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// COMMON QUERIES"</h3>
                    <div class="max-w-2xl mx-auto space-y-6 text-left">
                        <FaqItem
                            q="Can I cancel anytime?"
                            a="Yes. Cancel with one click from your profile settings. No lock-in, no penalties."
                        />
                        <FaqItem
                            q="Is there a free trial?"
                            a="New members get 7 days to explore. Cancel before trial ends and you will not be charged."
                        />
                        <FaqItem
                            q="Can I switch tiers?"
                            a="Upgrade or downgrade anytime. Changes take effect on your next billing cycle."
                        />
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
fn PricingCard(
    tier: &'static str,
    price: &'static str,
    desc: &'static str,
    features: Vec<&'static str>,
    plan: &'static str,
    accent: &'static str,
) -> impl IntoView {
    let is_featured = accent == "cyan";
    let border_class = if is_featured {
        "border-cyan-500/40 shadow-[0_0_30px_rgba(34,211,238,0.15)]"
    } else if accent == "amber" {
        "border-amber-500/20"
    } else {
        "border-slate-800"
    };

    let badge_class = match accent {
        "cyan" => "text-cyan-400 border-cyan-500/30 bg-cyan-500/5",
        "amber" => "text-amber-400 border-amber-500/30 bg-amber-500/5",
        _ => "text-slate-400 border-slate-700 bg-slate-800/30",
    };

    let price_color = match accent {
        "cyan" => "text-cyan-400",
        "amber" => "text-amber-400",
        _ => "text-white",
    };

    let btn_class = if is_featured {
        "w-full px-8 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)]"
    } else {
        "w-full px-8 py-4 border border-slate-700 text-slate-300 font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-slate-900 hover:border-slate-500"
    };

    let href = format!("/checkout?plan={plan}");

    view! {
        <div class=format!("relative rounded-2xl border bg-slate-900/50 backdrop-blur-sm p-8 flex flex-col {border_class}")>
            {is_featured.then(|| view! {
                <div class="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full border border-cyan-500/30 bg-cyan-500/10 text-[10px] font-mono font-bold text-cyan-400 uppercase tracking-widest">
                    "RECOMMENDED"
                </div>
            })}

            <div class=format!("inline-flex self-start items-center gap-2 px-3 py-1 rounded-full border text-[10px] font-mono font-bold uppercase tracking-[0.2em] {badge_class}")>
                {tier}
            </div>

            <div class="mt-6">
                <span class=format!("text-5xl font-black font-mono {price_color}")>{price}</span>
                <span class="text-slate-500 font-mono text-sm">" /mo"</span>
            </div>

            <p class="mt-3 text-sm text-slate-400 font-mono leading-relaxed">{desc}</p>

            <ul class="mt-6 flex-1 space-y-3">
                {features.into_iter().map(|f| view! {
                    <li class="flex items-start gap-2 text-sm font-mono text-slate-300">
                        <span class="text-cyan-500 mt-0.5">"+"</span>
                        <span>{f}</span>
                    </li>
                }).collect::<Vec<_>>()}
            </ul>

            <a href={href} class=format!("mt-8 block text-center {btn_class}")>
                "SELECT PLAN"
            </a>
        </div>
    }
}

#[component]
fn FaqItem(q: &'static str, a: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h4 class="text-sm font-mono font-bold text-white uppercase">{q}</h4>
            <p class="mt-2 text-sm text-slate-400 font-mono">{a}</p>
        </div>
    }
}
