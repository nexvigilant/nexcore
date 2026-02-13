//! Messages — private direct communications

use leptos::prelude::*;
use crate::api_client::{Message, SendMessageRequest};
use crate::auth::use_auth;

/// Server function to list messages
#[server(ListMessages, "/api")]
pub async fn list_messages_action() -> Result<Vec<Message>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.community_list_messages().await
        .map_err(ServerFnError::new)
}

/// Server function to send a message
#[server(SendMessage, "/api")]
pub async fn send_message_action(recipient_id: String, content: String) -> Result<Message, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    // TODO: Extract sender_id from real auth context
    let req = SendMessageRequest {
        sender_id: "anonymous".to_string(),
        recipient_id,
        content,
    };

    client.community_send_message(&req).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn MessagesPage() -> impl IntoView {
    let messages = Resource::new(|| (), |_| list_messages_action());
    let (selected_id, set_selected_id) = signal(Option::<String>::None);
    
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 h-[calc(100vh-8rem)]">
            <header class="mb-8">
                <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tighter">"MESSAGING"</h1>
            </header>

            <div class="grid gap-4 lg:grid-cols-3 h-full overflow-hidden">
                <div class="lg:col-span-1 flex flex-col glass-panel rounded-2xl overflow-hidden">
                    <div class="p-4 border-b border-slate-800 bg-slate-900/30">
                        <input
                            type="text"
                            placeholder="SEARCH ENCRYPTED..."
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-xs font-mono text-cyan-400 placeholder-slate-700 focus:border-cyan-500 focus:outline-none transition-all"
                        />
                    </div>
                    <div class="flex-1 overflow-y-auto">
                        <Suspense fallback=|| view! { <div class="p-4 animate-pulse">"SYNCHRONIZING..."</div> }>
                            {move || messages.get().map(|res| match res {
                                Ok(list) => view! { <ConversationList list=list selected=selected_id set_selected=set_selected_id /> }.into_any(),
                                Err(e) => view! { <div class="p-4 text-red-500 text-xs font-mono">{e.to_string()}</div> }.into_any()
                            })}
                        </Suspense>
                    </div>
                </div>

                <div class="lg:col-span-2 flex flex-col glass-panel rounded-2xl overflow-hidden relative">
                    {move || match selected_id.get() {
                        Some(id) => view! { <MessageThread id=id messages=messages /> }.into_any(),
                        None => view! { <EmptyThreadPlaceholder /> }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn ConversationList(
    list: Vec<Message>,
    selected: ReadSignal<Option<String>>,
    set_selected: WriteSignal<Option<String>>
) -> impl IntoView {
    if list.is_empty() {
        view! { <p class="p-8 text-center text-xs font-mono text-slate-600 uppercase tracking-widest">"NO COMMS RECORDED"</p> }.into_any()
    } else {
        view! {
            <div class="divide-y divide-slate-800/50">
                {list.into_iter().map(|m| {
                    let id = m.recipient_id.clone();
                    let id_for_active = id.clone();
                    let id_for_click = id.clone();
                    let id_for_initial = id.clone();
                    let content_preview = m.content.clone();
                    let time_str = m.created_at.format("%H:%M").to_string();
                    
                    view! {
                        <button 
                            on:click=move |_| set_selected.set(Some(id_for_click.clone()))
                            class=move || {
                                let base = "w-full p-4 text-left hover:bg-slate-800/30 transition-all flex items-center gap-3";
                                if selected.get().as_ref() == Some(&id_for_active) { 
                                    format!("{} bg-cyan-500/5 border-l-2 border-cyan-500", base) 
                                } else { 
                                    base.to_string() 
                                }
                            }
                        >
                            <div class="h-10 w-10 rounded-full bg-slate-800 flex items-center justify-center text-xs font-bold text-slate-500 font-mono border border-slate-700">
                                {id_for_initial.chars().next().unwrap_or('?').to_string().to_uppercase()}
                            </div>
                            <div class="flex-1 overflow-hidden">
                                <p class="text-xs font-bold text-slate-200 uppercase truncate">{id.clone()}</p>
                                <p class="text-[10px] text-slate-500 font-mono truncate mt-1">{content_preview.clone()}</p>
                            </div>
                            <span class="text-[9px] text-slate-700 font-mono">{time_str.clone()}</span>
                        </button>
                    }
                }).collect_view()}
            </div>
        }.into_any()
    }
}

#[component]
fn MessageThread(id: String, messages: Resource<Result<Vec<Message>, ServerFnError>>) -> impl IntoView {
    let content = RwSignal::new(String::new());
    let send_action = ServerAction::<SendMessage>::new();
    let thread_id_for_header = id.clone();
    let thread_id_for_filter = id.clone();
    let thread_id_for_keydown = id.clone();
    let thread_id_for_click = id.clone();

    view! {
        <div class="flex-1 flex flex-col h-full bg-slate-950/20">
            <div class="p-4 border-b border-slate-800 bg-slate-900/30 flex items-center gap-3">
                <div class="h-8 w-8 rounded-full bg-slate-800 flex items-center justify-center text-[10px] font-bold text-cyan-400 font-mono border border-cyan-500/30">
                    {thread_id_for_header.chars().next().unwrap_or('?').to_string().to_uppercase()}
                </div>
                <h3 class="text-xs font-bold text-white font-mono uppercase tracking-widest">{thread_id_for_header.clone()}</h3>
            </div>

            <div class="flex-1 overflow-y-auto p-6 space-y-4">
                {move || {
                    let tid = thread_id_for_filter.clone();
                    messages.get().map(|res| match res {
                        Ok(list) => {
                            view! {
                                {list.into_iter()
                                    .filter(|m| m.recipient_id == tid || m.sender_id == tid)
                                    .map(|m| view! { <MessageBubble message=m /> })
                                    .collect_view()}
                            }.into_any()
                        },
                        Err(_) => view! { <span class="text-red-400 text-xs font-mono">"Error loading messages"</span> }.into_any()
                    })
                }}
            </div>

            <div class="p-4 border-t border-slate-800 bg-slate-900/30">
                <div class="flex gap-3">
                    <input 
                        type="text"
                        placeholder="INPUT DIRECTIVE..."
                        prop:value=move || content.get()
                        on:input=move |ev| content.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" && !content.get().is_empty() {
                                let msg = content.get();
                                send_action.dispatch(SendMessage { 
                                    recipient_id: thread_id_for_keydown.clone(), 
                                    content: msg 
                                });
                                content.set(String::new());
                            }
                        }
                        class="flex-1 rounded-lg border border-slate-700 bg-slate-950 px-4 py-2 text-sm font-mono text-white focus:border-cyan-500 focus:outline-none transition-all"
                    />
                    <button 
                        on:click=move |_| {
                            if !content.get().is_empty() {
                                let msg = content.get();
                                send_action.dispatch(SendMessage { 
                                    recipient_id: thread_id_for_click.clone(), 
                                    content: msg 
                                });
                                content.set(String::new());
                            }
                        }
                        class="rounded-lg bg-cyan-600 px-6 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-all font-mono uppercase tracking-widest shadow-lg shadow-cyan-900/20"
                    >
                        "SEND"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MessageBubble(message: Message) -> impl IntoView {
    // Simplified sender check
    let is_me = message.sender_id == "anonymous";
    let align = if is_me { "justify-end" } else { "justify-start" };
    let color = if is_me { "bg-cyan-600/20 border-cyan-500/30 text-slate-100" } else { "bg-slate-800/40 border-slate-700 text-slate-300" };
    let content = message.content.clone();
    let time_str = message.created_at.format("%H:%M").to_string();

    view! {
        <div class=format!("flex {} w-full", align)>
            <div class=format!("max-w-[70%] rounded-2xl px-4 py-2 border backdrop-blur-sm {}", color)>
                <p class="text-sm leading-relaxed">{content}</p>
                <p class="text-[9px] text-slate-500 mt-1 font-mono">{time_str}</p>
            </div>
        </div>
    }
}

#[component]
fn EmptyThreadPlaceholder() -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col items-center justify-center text-center p-12">
            <div class="h-16 w-16 rounded-full bg-slate-900 flex items-center justify-center mb-6 border border-slate-800">
                <span class="text-slate-700 text-2xl">"//"</span>
            </div>
            <h3 class="text-sm font-bold text-slate-400 font-mono uppercase tracking-[0.2em]">"COMMS SECURED"</h3>
            <p class="mt-2 text-xs text-slate-600 font-mono">"SELECT SIGNAL SOURCE TO VIEW DIRECTIVE STREAM"</p>
        </div>
    }
}