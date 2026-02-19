//! Admin: Content validation and quality checks

use leptos::prelude::*;

struct ValidationCheck {
    name: &'static str,
    desc: &'static str,
    status: &'static str,
    checked: u32,
    passed: u32,
    failed: u32,
    last_run: &'static str,
}

const CHECKS: &[ValidationCheck] = &[
    ValidationCheck {
        name: "Regulatory Accuracy",
        desc: "Cross-reference claims against ICH/EMA/FDA guidelines",
        status: "Passing",
        checked: 142,
        passed: 142,
        failed: 0,
        last_run: "2h ago",
    },
    ValidationCheck {
        name: "Citation Verification",
        desc: "Verify all cited sources exist and are accessible",
        status: "Warning",
        checked: 238,
        passed: 234,
        failed: 4,
        last_run: "3h ago",
    },
    ValidationCheck {
        name: "Link Integrity",
        desc: "Check all internal and external links for 404s",
        status: "Passing",
        checked: 456,
        passed: 456,
        failed: 0,
        last_run: "1h ago",
    },
    ValidationCheck {
        name: "Terminology Consistency",
        desc: "Ensure MedDRA and WHO-DD terms are current and correct",
        status: "Passing",
        checked: 389,
        passed: 389,
        failed: 0,
        last_run: "4h ago",
    },
    ValidationCheck {
        name: "Accessibility Compliance",
        desc: "WCAG 2.1 AA compliance for all content pages",
        status: "Warning",
        checked: 135,
        passed: 128,
        failed: 7,
        last_run: "6h ago",
    },
    ValidationCheck {
        name: "Image Alt Text",
        desc: "Verify all images have descriptive alternative text",
        status: "Failing",
        checked: 89,
        passed: 71,
        failed: 18,
        last_run: "2h ago",
    },
    ValidationCheck {
        name: "Code Sample Validation",
        desc: "Test all embedded code samples compile and run correctly",
        status: "Passing",
        checked: 34,
        passed: 34,
        failed: 0,
        last_run: "12h ago",
    },
    ValidationCheck {
        name: "Cross-Reference Integrity",
        desc: "Verify internal cross-references between courses and modules",
        status: "Passing",
        checked: 267,
        passed: 265,
        failed: 2,
        last_run: "5h ago",
    },
];

#[component]
pub fn ContentValidationPage() -> impl IntoView {
    let total_checked: u32 = CHECKS.iter().map(|c| c.checked).sum();
    let total_failed: u32 = CHECKS.iter().map(|c| c.failed).sum();
    let all_passing = CHECKS.iter().all(|c| c.status == "Passing");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Content Validation"</h1>
                    <p class="mt-1 text-slate-400">"Automated quality checks, accuracy verification, and compliance validation."</p>
                </div>
                <a href="/admin/content" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Content Admin"</a>
            </div>

            /* Summary */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Checks"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{CHECKS.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Items Checked"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{total_checked.to_string()}</p>
                </div>
                <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-5">
                    <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest font-mono">"Issues Found"</p>
                    <p class="text-2xl font-black text-red-400 font-mono mt-2">{total_failed.to_string()}</p>
                </div>
                <div class=move || {
                    if all_passing {
                        "rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5"
                    } else {
                        "rounded-xl border border-amber-500/20 bg-amber-500/5 p-5"
                    }
                }>
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Overall Status"</p>
                    <p class=move || {
                        if all_passing {
                            "text-2xl font-black text-emerald-400 font-mono mt-2"
                        } else {
                            "text-2xl font-black text-amber-400 font-mono mt-2"
                        }
                    }>
                        {if all_passing { "PASSING" } else { "WARNINGS" }}
                    </p>
                </div>
            </div>

            /* Checks */
            <div class="mt-8 space-y-3">
                {CHECKS.iter().map(|check| {
                    let (icon, icon_cls) = match check.status {
                        "Passing" => ("\u{2713}", "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"),
                        "Warning" => ("!", "text-amber-400 bg-amber-500/10 border-amber-500/20"),
                        _ => ("\u{2717}", "text-red-400 bg-red-500/10 border-red-500/20"),
                    };
                    let left_border = match check.status {
                        "Passing" => "border-l-emerald-500",
                        "Warning" => "border-l-amber-500",
                        _ => "border-l-red-500",
                    };
                    view! {
                        <div class=format!("rounded-xl border border-slate-800 bg-slate-900/50 p-5 border-l-2 {left_border}")>
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-4">
                                    <span class=format!("h-8 w-8 rounded-lg border flex items-center justify-center text-xs font-bold shrink-0 {icon_cls}")>{icon}</span>
                                    <div>
                                        <h3 class="text-sm font-bold text-white">{check.name}</h3>
                                        <p class="text-[10px] text-slate-500 mt-0.5">{check.desc}</p>
                                    </div>
                                </div>
                                <div class="text-right shrink-0">
                                    <p class="text-xs text-slate-400 font-mono">{format!("{}/{} passed", check.passed, check.checked)}</p>
                                    {(check.failed != 0).then(|| view! {
                                        <p class="text-xs text-red-400 font-mono font-bold">{format!("{} issues", check.failed)}</p>
                                    })}
                                    <p class="text-[10px] text-slate-600 font-mono mt-1">{check.last_run}</p>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="mt-6 flex justify-end">
                <button class="rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase tracking-widest">
                    "Run Full Validation"
                </button>
            </div>
        </div>
    }
}
