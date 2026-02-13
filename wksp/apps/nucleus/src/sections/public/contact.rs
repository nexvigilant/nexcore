//! Contact page — contact form + direct contact info

use leptos::prelude::*;

/// Server function: handle contact form submission
#[server(SubmitContactForm, "/api")]
pub async fn submit_contact_form(
    name: String,
    email: String,
    subject: String,
    message: String,
) -> Result<String, ServerFnError> {
    // MVP: log the submission. Production: send email or store in Firestore.
    tracing::info!(
        name = %name,
        email = %email,
        subject = %subject,
        "Contact form submission received ({} chars)",
        message.len()
    );
    Ok("Message received. We will respond within 24 hours.".to_string())
}

#[component]
pub fn ContactPage() -> impl IntoView {
    let contact_action = ServerAction::<SubmitContactForm>::new();
    let action_value = contact_action.value();
    let pending = contact_action.pending();

    view! {
        <div class="min-h-screen bg-slate-950 selection:bg-cyan-500/30">
            // Hero
            <section class="relative py-24 px-6 text-center overflow-hidden">
                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] bg-cyan-600/5 rounded-full blur-[100px]"></div>
                <div class="relative z-10 max-w-3xl mx-auto">
                    <h2 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.4em] mb-4">"// CONTACT"</h2>
                    <h1 class="text-5xl md:text-6xl font-black font-mono text-white uppercase tracking-tighter">"GET IN TOUCH"</h1>
                    <p class="mt-6 text-lg text-slate-400 font-mono max-w-xl mx-auto">
                        "QUESTIONS ABOUT MEMBERSHIP, CONSULTING, OR COLLABORATION? WE RESPOND WITHIN 24 HOURS."
                    </p>
                </div>
            </section>

            <section class="mx-auto max-w-4xl px-6 pb-32">
                <div class="grid gap-12 md:grid-cols-5">
                    // Contact form (3 cols)
                    <div class="md:col-span-3">
                        // Success message
                        {move || {
                            action_value.get().and_then(|r| r.ok()).map(|msg| view! {
                                <div class="mb-6 rounded-xl border border-emerald-500/30 bg-emerald-500/10 p-4 text-sm text-emerald-400 font-mono">
                                    {msg}
                                </div>
                            })
                        }}

                        // Error message
                        {move || {
                            action_value.get().and_then(|r| r.err()).map(|e| view! {
                                <div class="mb-6 rounded-xl border border-red-500/30 bg-red-500/10 p-4 text-sm text-red-400 font-mono">
                                    {e.to_string()}
                                </div>
                            })
                        }}

                        <ActionForm action=contact_action attr:class="space-y-5">
                            <div>
                                <label class="block text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest mb-2">"NAME"</label>
                                <input
                                    type="text"
                                    name="name"
                                    required=true
                                    class="w-full rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3 text-sm font-mono text-white placeholder:text-slate-600 focus:border-cyan-500/50 focus:outline-none focus:ring-1 focus:ring-cyan-500/30 transition-colors"
                                    placeholder="Your name"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest mb-2">"EMAIL"</label>
                                <input
                                    type="email"
                                    name="email"
                                    required=true
                                    class="w-full rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3 text-sm font-mono text-white placeholder:text-slate-600 focus:border-cyan-500/50 focus:outline-none focus:ring-1 focus:ring-cyan-500/30 transition-colors"
                                    placeholder="you@company.com"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest mb-2">"SUBJECT"</label>
                                <input
                                    type="text"
                                    name="subject"
                                    required=true
                                    class="w-full rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3 text-sm font-mono text-white placeholder:text-slate-600 focus:border-cyan-500/50 focus:outline-none focus:ring-1 focus:ring-cyan-500/30 transition-colors"
                                    placeholder="Consulting inquiry, membership question, etc."
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest mb-2">"MESSAGE"</label>
                                <textarea
                                    name="message"
                                    required=true
                                    rows="5"
                                    class="w-full rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3 text-sm font-mono text-white placeholder:text-slate-600 focus:border-cyan-500/50 focus:outline-none focus:ring-1 focus:ring-cyan-500/30 transition-colors resize-none"
                                    placeholder="Tell us about your needs..."
                                ></textarea>
                            </div>
                            <button
                                type="submit"
                                disabled=pending
                                class="w-full px-10 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)] disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {move || if pending.get() { "SENDING..." } else { "SEND MESSAGE" }}
                            </button>
                        </ActionForm>
                    </div>

                    // Contact info (2 cols)
                    <div class="md:col-span-2 space-y-6">
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                            <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-widest mb-4">"// DIRECT CONTACT"</h3>
                            <div class="space-y-4">
                                <div>
                                    <div class="text-[10px] font-mono text-slate-600 uppercase tracking-widest">"EMAIL"</div>
                                    <a href="mailto:matthew@nexvigilant.com" class="text-sm font-mono text-cyan-400 hover:text-cyan-300 transition-colors">
                                        "matthew@nexvigilant.com"
                                    </a>
                                </div>
                                <div>
                                    <div class="text-[10px] font-mono text-slate-600 uppercase tracking-widest">"LINKEDIN"</div>
                                    <a href="https://linkedin.com/in/matthewcampion" target="_blank" rel="noopener" class="text-sm font-mono text-cyan-400 hover:text-cyan-300 transition-colors">
                                        "linkedin.com/in/matthewcampion"
                                    </a>
                                </div>
                            </div>
                        </div>

                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                            <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-widest mb-4">"// RESPONSE TIME"</h3>
                            <p class="text-sm text-slate-400 font-mono">
                                "We typically respond within 24 hours. Consulting inquiries receive a detailed scope document within 48 hours."
                            </p>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}
