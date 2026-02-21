//! Learning player — interactive lesson experience with persistent sidebar
//!
//! Two-pane layout:
//! - Left: Module/Lesson navigation tree (ς State)
//! - Right: Content area (Markdown, Video, Labs) with completion logic

use crate::api_client::{CourseDetail, CourseModule, Enrollment, Lesson};
use crate::sections::academy::academy_components::StreakWidget;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/* ------------------------------------------------------------------ */
/*  Server functions                                                   */
/* ------------------------------------------------------------------ */

#[server(GetLearningSession, "/api")]
pub async fn get_learning_session(course_id: String) -> Result<CourseDetail, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client
        .academy_get_course(&course_id)
        .await
        .map_err(ServerFnError::new)
}

#[server(CompleteLesson, "/api")]
pub async fn complete_lesson_action(
    enrollment_id: String,
    lesson_id: String,
) -> Result<Enrollment, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client
        .academy_complete_lesson(&enrollment_id, &lesson_id)
        .await
        .map_err(ServerFnError::new)
}

#[server(GetModuleLessons, "/api")]
pub async fn get_module_lessons(module_id: String) -> Result<Vec<Lesson>, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client
        .academy_get_module_lessons(&module_id)
        .await
        .map_err(ServerFnError::new)
}

#[server(SubmitQuiz, "/api")]
pub async fn submit_quiz_action(
    enrollment_id: String,
    lesson_id: String,
    score: f32,
    passed: bool,
) -> Result<crate::api_client::QuizAttempt, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    let req = crate::api_client::SubmitQuizRequest {
        enrollment_id,
        lesson_id,
        score,
        passed,
    };
    client
        .academy_submit_quiz(&req)
        .await
        .map_err(ServerFnError::new)
}

#[server(GetQuizAttempts, "/api")]
pub async fn get_quiz_attempts(
    enrollment_id: String,
    lesson_id: String,
) -> Result<Vec<crate::api_client::QuizAttempt>, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client
        .academy_get_quiz_attempts(&enrollment_id, &lesson_id)
        .await
        .map_err(ServerFnError::new)
}

/* ------------------------------------------------------------------ */
/*  Page component                                                     */
/* ------------------------------------------------------------------ */

#[component]
pub fn LearnPage() -> impl IntoView {
    let params = use_params_map();
    let course_id = move || params.get().get("id").unwrap_or_default();

    let session = Resource::new(move || course_id(), |id| get_learning_session(id));

    /* Local state for current selection */
    let current_lesson = RwSignal::new(Option::<Lesson>::None);

    view! {
        <Suspense fallback=|| view! { <LearnLoadingSkeleton /> }>
            {move || session.get().map(|result| match result {
                Ok(data) => {
                    view! { <LearnPlayer data=data current_lesson=current_lesson /> }.into_any()
                },
                Err(e) => view! {
                    <div class="h-screen flex items-center justify-center p-8 text-center bg-slate-950">
                        <div class="max-w-md w-full glass-panel p-10 rounded-3xl border border-red-500/20">
                            <div class="h-16 w-16 rounded-full bg-red-500/10 flex items-center justify-center text-red-500 mx-auto mb-6 text-2xl font-mono">"!"</div>
                            <h2 class="text-xl font-bold text-white font-mono uppercase tracking-tight mb-2">"Session Interrupt"</h2>
                            <p class="text-slate-500 font-mono text-sm mb-8 leading-relaxed">{e.to_string()}</p>
                            <a href="/academy" class="inline-block px-8 py-3 rounded-xl bg-slate-900 border border-slate-800 text-xs font-bold text-cyan-400 hover:text-cyan-300 uppercase tracking-[0.2em] transition-all">
                                "RETURN TO COMMAND"
                            </a>
                        </div>
                    </div>
                }.into_any()
            })}
        </Suspense>
    }
}

/* ------------------------------------------------------------------ */
/*  The Player Core                                                    */
/* ------------------------------------------------------------------ */

#[component]
fn LearnPlayer(data: CourseDetail, current_lesson: RwSignal<Option<Lesson>>) -> impl IntoView {
    let course = data.course;
    let modules = data.modules.clone();
    let enrollment = RwSignal::new(data.enrollment.clone());

    view! {
        <div class="flex h-[calc(100vh-4rem)] overflow-hidden bg-slate-950">
            /* ---- Left Pane: Navigation Tree ---- */
            <aside class="w-80 shrink-0 border-r border-slate-800 bg-slate-900/30 flex flex-col">
                <div class="p-6 border-b border-slate-800">
                    <span class="text-[10px] font-bold text-cyan-500 font-mono uppercase tracking-widest">"Capability Pathway"</span>
                    <h2 class="text-sm font-bold text-white mt-1 line-clamp-1">{course.title.clone()}</h2>

                    <div class="mt-4 flex justify-between items-center mb-1">
                        <span class="text-[9px] font-bold text-slate-500 font-mono uppercase">"Progress"</span>
                        <span class="text-[9px] font-bold text-cyan-400 font-mono">
                            {move || format!("{:.0}%", enrollment.get().map(|e| e.progress * 100.0).unwrap_or(0.0))}
                        </span>
                    </div>
                    <div class="h-1 w-full bg-slate-800 rounded-full overflow-hidden">
                        <div class="h-full bg-cyan-500 transition-all duration-700"
                             style=move || format!("width: {}%", enrollment.get().map(|e| e.progress * 100.0).unwrap_or(0.0))></div>
                    </div>
                </div>

                <nav class="flex-1 overflow-y-auto p-4 space-y-6 custom-scrollbar">
                    {modules.into_iter().map(|m| {
                        let enrollment_inner = enrollment;
                        view! {
                            <ModuleNode module=m current_lesson=current_lesson enrollment=enrollment_inner />
                        }
                    }).collect_view()}
                </nav>

                <div class="p-4 border-t border-slate-800 bg-slate-950/50">
                    <StreakWidget streak=5 />
                </div>
            </aside>

            /* ---- Right Pane: Content Area ---- */
            <div class="flex-1 flex flex-col relative h-full">
                <main class="flex-1 overflow-y-auto scroll-smooth pb-32">
                    {move || match current_lesson.get() {
                        Some(lesson) => {
                            let enrollment_inner = enrollment;
                            view! { <LessonRenderer lesson=lesson enrollment=enrollment_inner /> }.into_any()
                        },
                        None => view! { <LearnWelcome course_title=course.title.clone() /> }.into_any(),
                    }}
                </main>

                /* ---- Fixed Bottom Progress Bar ---- */
                {move || match current_lesson.get() {
                    Some(lesson) => {
                        let enrollment_inner = enrollment;
                        view! { <LessonProgressBar lesson=lesson enrollment=enrollment_inner /> }.into_any()
                    },
                    None => view! { <div/> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn ModuleNode(
    module: CourseModule,
    current_lesson: RwSignal<Option<Lesson>>,
    enrollment: RwSignal<Option<Enrollment>>,
) -> impl IntoView {
    let lessons = Resource::new(move || module.id.clone(), |id| get_module_lessons(id));

    view! {
        <div class="space-y-2">
            <div class="flex items-center gap-2 mb-2">
                <span class="text-[10px] font-mono text-slate-500 group-hover:text-cyan-500 transition-colors">
                    {format!("{:02}", module.order)}
                </span>
                <h3 class="text-[11px] font-bold text-slate-400 uppercase tracking-widest leading-tight">{module.title}</h3>
            </div>

            <div class="space-y-1 ml-2 border-l border-slate-800 pl-3">
                <Suspense fallback=|| view! {
                    <div class="space-y-2">
                        <div class="h-4 w-full bg-slate-900 rounded animate-pulse"></div>
                        <div class="h-4 w-2/3 bg-slate-900 rounded animate-pulse"></div>
                    </div>
                }>
                    {move || lessons.get().map(|result| match result {
                        Ok(list) => {
                            list.into_iter().map(|lesson| {
                                let enrollment_inner = enrollment;
                                view! {
                                    <LessonItem lesson=lesson current_lesson=current_lesson enrollment=enrollment_inner />
                                }
                            }).collect_view().into_any()
                        },
                        Err(_) => view! { <p class="text-[10px] text-red-500 italic">"Load failed"</p> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn LessonItem(
    lesson: Lesson,
    current_lesson: RwSignal<Option<Lesson>>,
    enrollment: RwSignal<Option<Enrollment>>,
) -> impl IntoView {
    let id_for_class = lesson.id.clone();
    let id_for_status_class = lesson.id.clone();
    let id_for_status_icon = lesson.id.clone();

    view! {
        <button
            on:click=move |_| current_lesson.set(Some(lesson.clone()))
            class=move || {
                let base = "w-full text-left px-3 py-2 rounded-lg text-xs transition-all relative group";
                let is_active = current_lesson
                    .get()
                    .map(|l| l.id == id_for_class)
                    .unwrap_or(false);
                if is_active {
                    format!("{base} bg-cyan-500/10 text-cyan-400 font-bold border border-cyan-500/20 shadow-[0_0_15px_-3px_rgba(34,211,238,0.1)]")
                } else {
                    format!("{base} text-slate-500 hover:text-slate-300 hover:bg-slate-900/50")
                }
            }
        >
            <div class="flex items-center gap-3">
                <span class=move || {
                    let is_completed = enrollment
                        .get()
                        .map(|e| e.completed_lesson_ids.contains(&id_for_status_class))
                        .unwrap_or(false);
                    if is_completed {
                        "text-emerald-500"
                    } else {
                        let is_active = current_lesson
                            .get()
                            .map(|l| l.id == id_for_status_class)
                            .unwrap_or(false);
                        if is_active {
                            "text-cyan-400"
                        } else {
                            "text-slate-700 group-hover:text-slate-500"
                        }
                    }
                }>
                    {move || {
                        if enrollment
                            .get()
                            .map(|e| e.completed_lesson_ids.contains(&id_for_status_icon))
                            .unwrap_or(false)
                        {
                            "✓"
                        } else {
                            "▤"
                        }
                    }}
                </span>
                <span class="truncate">{lesson.title.clone()}</span>
            </div>
        </button>
    }
}

#[component]
fn LessonRenderer(lesson: Lesson, enrollment: RwSignal<Option<Enrollment>>) -> impl IntoView {
    let complete_action = ServerAction::<CompleteLesson>::new();

    /* Refetch enrollment on completion */
    Effect::new(move |_| {
        if let Some(Ok(new_e)) = complete_action.value().get() {
            enrollment.set(Some(new_e));
        }
    });

    view! {
        <div class="max-w-4xl mx-auto px-8 py-12 md:py-20 animate-in fade-in slide-in-from-bottom-4 duration-700">
            /* Module Progress Mini-Card */
            <div class="mb-12 p-1.5 rounded-2xl bg-slate-900/50 border border-slate-800 w-fit inline-flex items-center gap-4 pr-6 text-left text-left">
                <div class="h-10 w-10 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400 font-black font-mono">
                    "σ"
                </div>
                <div>
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-[0.2em] font-mono">"Current Activity"</p>
                    <h4 class="text-[11px] font-bold text-slate-300 uppercase tracking-wider">{lesson.title.clone()}</h4>
                </div>
            </div>

            <header class="mb-12">
                <div class="flex flex-wrap items-center gap-3 mb-6">
                    <span class="px-2.5 py-1 rounded-full bg-cyan-500/10 text-[9px] font-bold font-mono text-cyan-400 uppercase border border-cyan-500/20 tracking-widest">
                        {lesson.lesson_type.to_uppercase()}
                    </span>
                    {lesson.duration_minutes.map(|d| view! {
                        <span class="text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono">{d} " MIN DURATION"</span>
                    })}
                </div>
                <h1 class="text-4xl md:text-5xl font-black text-white font-mono uppercase tracking-tighter leading-none mb-6">{lesson.title.clone()}</h1>
                <div class="h-0.5 w-20 bg-cyan-500/30 rounded-full"></div>
            </header>

            <article class="prose prose-invert prose-slate max-w-none prose-headings:font-mono prose-headings:uppercase prose-headings:tracking-tight prose-p:leading-relaxed prose-p:text-slate-400 prose-li:text-slate-400 prose-strong:text-cyan-400 prose-blockquote:border-cyan-500/30 prose-blockquote:bg-cyan-500/5 prose-blockquote:rounded-2xl prose-blockquote:p-6">
                /* Content from the API */
                <p>{lesson.content.clone()}</p>

                /* Conditional Interactive Lab */
                {if lesson.content.contains("[LAB:SIGNAL_DETECTION]") {
                    view! { <InteractiveLab kind="signal_detection" /> }.into_any()
                } else {
                    view! { <div/> }.into_any()
                }}
            </article>
        </div>
    }
}

#[component]
fn LessonProgressBar(lesson: Lesson, enrollment: RwSignal<Option<Enrollment>>) -> impl IntoView {
    let complete_action = ServerAction::<CompleteLesson>::new();
    let lesson_id_for_completed = lesson.id.clone();
    let lesson_id_for_dispatch = lesson.id.clone();
    let is_completed = move || {
        enrollment
            .get()
            .map(|e| e.completed_lesson_ids.contains(&lesson_id_for_completed))
            .unwrap_or(false)
    };

    view! {
        <div class="absolute bottom-0 left-0 right-0 p-6 bg-slate-950/80 backdrop-blur-md border-t border-slate-800/50 flex items-center justify-between z-20">
            <div class="flex items-center gap-6">
                <button class="h-10 w-10 rounded-xl border border-slate-800 bg-slate-900/50 flex items-center justify-center text-slate-500 hover:text-white hover:border-slate-700 transition-all group">
                    <span class="text-lg group-hover:-translate-x-0.5 transition-transform">"\u{2190}"</span>
                </button>
                <div class="hidden md:block">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono">"Next Up"</p>
                    <p class="text-xs font-bold text-slate-400 uppercase tracking-wider">"Proceed to Next Activity"</p>
                </div>
            </div>

            <div class="flex items-center gap-4">
                {move || if is_completed() {
                    view! {
                        <div class="px-6 py-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center gap-3">
                            <span class="text-emerald-400 text-sm">"✓"</span>
                            <span class="text-[10px] font-bold text-emerald-400 font-mono uppercase tracking-widest">"Validated"</span>
                        </div>
                    }.into_any()
                } else {
                    let lid = lesson_id_for_dispatch.clone();
                    view! {
                        <button
                            on:click=move |_| {
                                if let Some(ref e) = enrollment.get() {
                                    complete_action.dispatch(CompleteLesson {
                                        enrollment_id: e.id.clone(),
                                        lesson_id: lid.clone(),
                                    });
                                }
                            }
                            disabled=complete_action.pending()
                            class="px-10 py-3 rounded-xl bg-cyan-600 text-white font-black font-mono uppercase tracking-[0.2em] text-xs hover:bg-cyan-500 hover:shadow-glow-cyan transition-all disabled:opacity-50"
                        >
                            {move || if complete_action.pending().get() { "SYNCING..." } else { "COMPLETE" }}
                        </button>
                    }.into_any()
                }}

                <button class="h-10 w-10 rounded-xl border border-slate-800 bg-slate-900/50 flex items-center justify-center text-slate-500 hover:text-white hover:border-slate-700 transition-all group">
                    <span class="text-lg group-hover:translate-x-0.5 transition-transform">"\u{2192}"</span>
                </button>
            </div>
        </div>
    }
}

#[server(RunLabSignalDetection, "/api")]
pub async fn run_lab_signal_detection(
    a: u64,
    b: u64,
    c: u64,
    d: u64,
) -> Result<crate::api_client::SignalResult, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);
    let req = crate::api_client::SignalRequest { a, b, c, d };
    client
        .signal_complete(&req)
        .await
        .map_err(ServerFnError::new)
}

#[component]
fn InteractiveLab(kind: &'static str) -> impl IntoView {
    match kind {
        "signal_detection" => view! { <SignalLabContent /> }.into_any(),
        _ => view! { <div class="p-4 bg-slate-900 rounded-lg border border-slate-800 text-slate-500 italic">"Unknown Lab Type"</div> }.into_any(),
    }
}

#[component]
fn SignalLabContent() -> impl IntoView {
    let a = RwSignal::new(String::from("10"));
    let b = RwSignal::new(String::from("100"));
    let c = RwSignal::new(String::from("5"));
    let d = RwSignal::new(String::from("1000"));

    let analysis = Action::new(move |_: &()| {
        let a = a.get().parse().unwrap_or(0);
        let b = b.get().parse().unwrap_or(0);
        let c = c.get().parse().unwrap_or(0);
        let d = d.get().parse().unwrap_or(0);
        async move { run_lab_signal_detection(a, b, c, d).await }
    });

    view! {
        <div class="my-10 p-8 rounded-3xl border border-cyan-500/30 bg-cyan-500/5 glass-panel">
            <div class="flex items-center gap-3 mb-6">
                <span class="text-2xl">"🧪"</span>
                <h4 class="text-xl font-bold text-white font-mono uppercase tracking-tight m-0">"Interactive Lab: Signal Analysis"</h4>
            </div>

            <div class="grid sm:grid-cols-2 gap-8">
                <div class="space-y-4">
                    <p class="text-xs text-slate-400 font-mono uppercase tracking-widest mb-4">"Input Data Matrix"</p>
                    <div class="grid grid-cols-2 gap-3">
                        <LabInput label="a (Drug+AE)" signal=a />
                        <LabInput label="b (Drug+No AE)" signal=b />
                        <LabInput label="c (Other+AE)" signal=c />
                        <LabInput label="d (Other+No AE)" signal=d />
                    </div>
                    <button
                        on:click=move |_| {
                            analysis.dispatch(());
                        }
                        disabled=analysis.pending()
                        class="w-full mt-4 py-3 rounded-xl bg-cyan-600 text-white font-bold font-mono uppercase tracking-widest hover:bg-cyan-500 transition-all disabled:opacity-50"
                    >
                        {move || if analysis.pending().get() { "COMPUTING..." } else { "RUN ANALYSIS" }}
                    </button>
                </div>

                <div class="bg-slate-950/50 rounded-2xl border border-slate-800 p-6 flex flex-col">
                    <p class="text-xs text-slate-500 font-mono uppercase tracking-widest mb-4">"Telemetry Result"</p>
                    <div class="flex-1 flex flex-col justify-center">
                        {move || match analysis.value().get() {
                            Some(Ok(res)) => view! {
                                <div class="space-y-3">
                                    <div class=format!("text-center py-2 rounded-lg font-black font-mono text-sm border {}", if res.any_signal { "text-red-400 border-red-500/20 bg-red-500/5" } else { "text-emerald-400 border-emerald-500/20 bg-emerald-500/5" })>
                                        {if res.any_signal { "SIGNAL DETECTED" } else { "STABLE" }}
                                    </div>
                                    <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-[10px] font-mono">
                                        <span class="text-slate-500">"PRR"</span>
                                        <span class=if res.prr_signal { "text-red-400" } else { "text-slate-300" }>{format!("{:.2}", res.prr)}</span>
                                        <span class="text-slate-500">"ROR"</span>
                                        <span class=if res.ror_signal { "text-red-400" } else { "text-slate-300" }>{format!("{:.2}", res.ror)}</span>
                                        <span class="text-slate-500">"CHI\u{00b2}"</span>
                                        <span class=if res.chi_signal { "text-red-400" } else { "text-slate-300" }>{format!("{:.2}", res.chi_square)}</span>
                                    </div>
                                </div>
                            }.into_any(),
                            Some(Err(e)) => view! { <p class="text-red-500 text-[10px] font-mono">{e.to_string()}</p> }.into_any(),
                            None => view! { <p class="text-slate-600 text-xs italic font-mono text-center">"Awaiting execution..."</p> }.into_any(),
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn LabInput(label: &'static str, signal: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <label class="text-[9px] font-bold text-slate-500 uppercase ml-1 font-mono">{label}</label>
            <input
                type="number"
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
                class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-2 text-xs text-white focus:border-cyan-500 focus:outline-none font-mono"
            />
        </div>
    }
}

#[component]
fn LearnWelcome(course_title: String) -> impl IntoView {
    view! {
        <div class="h-full flex flex-col items-center justify-center p-12 text-center">
            <div class="h-24 w-24 rounded-3xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-5xl mb-8">
                "σ"
            </div>
            <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tight mb-4">{course_title}</h1>
            <p class="text-slate-500 max-w-md mx-auto leading-relaxed">
                "Welcome to the Learning Engine. Select a lesson from the sidebar to begin your capability development session."
            </p>

            <div class="mt-12 grid grid-cols-3 gap-8 w-full max-w-lg">
                <WelcomeStat label="Modules" value="08" />
                <WelcomeStat label="Hours" value="~12" />
                <WelcomeStat label="Skills" value="04" />
            </div>
        </div>
    }
}

#[component]
fn WelcomeStat(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div>
            <p class="text-[10px] font-bold text-slate-600 uppercase tracking-[0.2em] mb-1 font-mono">{label}</p>
            <p class="text-xl font-bold text-slate-300 font-mono">{value}</p>
        </div>
    }
}

#[component]
fn LearnLoadingSkeleton() -> impl IntoView {
    view! {
        <div class="flex h-[calc(100vh-4rem)] bg-slate-950 animate-pulse">
            <div class="w-80 border-r border-slate-800 p-6">
                <div class="h-4 w-24 bg-slate-800 rounded mb-4"></div>
                <div class="h-8 w-full bg-slate-800 rounded mb-10"></div>
                <div class="space-y-6">
                    {(0..4).map(|_| view! { <div class="h-20 w-full bg-slate-900 rounded-xl border border-slate-800"></div> }).collect_view()}
                </div>
            </div>
            <div class="flex-1 p-12">
                <div class="h-10 w-96 bg-slate-900 rounded mb-8"></div>
                <div class="space-y-4">
                    <div class="h-4 w-full bg-slate-900 rounded"></div>
                    <div class="h-4 w-full bg-slate-900 rounded"></div>
                    <div class="h-4 w-2/3 bg-slate-900 rounded"></div>
                </div>
            </div>
        </div>
    }
}
