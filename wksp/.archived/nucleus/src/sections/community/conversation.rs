//! Conversation — direct message thread

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ConversationPage() -> impl IntoView {
    let params = use_params_map();
    let conversation_id = move || params.get().get("conversationId").unwrap_or_default();

    let message_input = RwSignal::new(String::new());
    let (messages, set_messages) = signal(vec![
        MessageEntry {
            sender: "them",
            content: "Hi! I saw your post about PRR vs ROR in pediatric databases. We're dealing with the same issue at our company.",
            time: "10:23 AM",
        },
        MessageEntry {
            sender: "them",
            content: "Have you tried using the Multi-item Gamma Poisson Shrinker? We found it handles the sparse counts much better than standard PRR.",
            time: "10:24 AM",
        },
        MessageEntry {
            sender: "me",
            content: "That's interesting! We've been looking at BCPNN primarily but haven't explored MGPS in depth for pediatric subsets.",
            time: "10:31 AM",
        },
        MessageEntry {
            sender: "me",
            content: "Do you have any references for the MGPS approach specifically in pediatric populations? Most of the literature I've found focuses on general adult databases.",
            time: "10:32 AM",
        },
        MessageEntry {
            sender: "them",
            content: "Yes! The DuMouchel (2013) paper covers this. I can share the methodology we adapted from it. Would a call be helpful?",
            time: "10:45 AM",
        },
    ]);

    view! {
        <div class="mx-auto max-w-3xl px-4 py-8 h-[calc(100vh-8rem)] flex flex-col">
            /* Header */
            <div class="flex items-center gap-4 pb-6 border-b border-slate-800 shrink-0">
                <a href="/community/messages" class="text-xs font-bold text-slate-500 font-mono uppercase tracking-widest hover:text-white transition-colors">
                    "<-"
                </a>
                <div class="h-10 w-10 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-sm font-bold text-cyan-400 font-mono">
                    {move || conversation_id().chars().next().unwrap_or('?').to_string().to_uppercase()}
                </div>
                <div class="flex-1">
                    <h1 class="text-sm font-bold text-white font-mono uppercase tracking-widest">{move || conversation_id()}</h1>
                    <p class="text-[10px] text-emerald-400 font-mono">"ONLINE"</p>
                </div>
                <div class="flex items-center gap-2">
                    <button class="rounded-lg border border-slate-700 px-3 py-1.5 text-[10px] font-bold text-slate-400 font-mono uppercase tracking-widest hover:text-white hover:border-slate-600 transition-all">"PROFILE"</button>
                    <button class="rounded-lg border border-slate-700 px-3 py-1.5 text-[10px] font-bold text-slate-400 font-mono uppercase tracking-widest hover:text-white hover:border-slate-600 transition-all">"MUTE"</button>
                </div>
            </div>

            /* Message thread */
            <div class="flex-1 overflow-y-auto py-6 space-y-4">
                /* Date separator */
                <div class="flex items-center gap-4 my-4">
                    <div class="flex-1 h-px bg-slate-800" />
                    <span class="text-[10px] font-bold text-slate-600 font-mono uppercase tracking-widest">"TODAY"</span>
                    <div class="flex-1 h-px bg-slate-800" />
                </div>

                {move || messages.get().into_iter().map(|msg| {
                    let is_me = msg.sender == "me";
                    let align = if is_me { "justify-end" } else { "justify-start" };
                    let bubble_color = if is_me {
                        "bg-cyan-600/20 border-cyan-500/30 text-slate-100"
                    } else {
                        "bg-slate-800/40 border-slate-700 text-slate-300"
                    };

                    view! {
                        <div class=format!("flex {} w-full", align)>
                            <div class=format!("max-w-[70%] rounded-2xl px-4 py-3 border backdrop-blur-sm {}", bubble_color)>
                                <p class="text-sm leading-relaxed">{msg.content}</p>
                                <p class="text-[9px] text-slate-500 mt-1.5 font-mono">{msg.time}</p>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            /* Input area */
            <div class="pt-4 border-t border-slate-800 shrink-0">
                <div class="flex gap-3">
                    <input
                        type="text"
                        placeholder="Type a message..."
                        prop:value=move || message_input.get()
                        on:input=move |ev| message_input.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" && !message_input.get().trim().is_empty() {
                                let text = message_input.get();
                                let mut current = messages.get();
                                current.push(MessageEntry {
                                    sender: "me",
                                    content: Box::leak(text.into_boxed_str()),
                                    time: "now",
                                });
                                set_messages.set(current);
                                message_input.set(String::new());
                            }
                        }
                        class="flex-1 rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white placeholder:text-slate-600 focus:border-cyan-500 focus:outline-none transition-all"
                    />
                    <button
                        class="rounded-lg bg-cyan-600 px-6 py-3 text-xs font-bold text-white hover:bg-cyan-500 transition-all font-mono uppercase tracking-widest shadow-lg shadow-cyan-900/20 disabled:opacity-50"
                        disabled=move || message_input.get().trim().is_empty()
                        on:click=move |_| {
                            let text = message_input.get();
                            if !text.trim().is_empty() {
                                let mut current = messages.get();
                                current.push(MessageEntry {
                                    sender: "me",
                                    content: Box::leak(text.into_boxed_str()),
                                    time: "now",
                                });
                                set_messages.set(current);
                                message_input.set(String::new());
                            }
                        }
                    >
                        "SEND"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone)]
struct MessageEntry {
    sender: &'static str,
    content: &'static str,
    time: &'static str,
}
