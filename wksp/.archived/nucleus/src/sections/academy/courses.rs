//! Academy courses page — view and enroll in courses

use leptos::prelude::*;
use crate::api_client::{Course, Enrollment, EnrollRequest};
use crate::auth::use_auth;

/// Server function to list academy courses
#[server(ListCourses, "/api")]
pub async fn list_courses_action() -> Result<Vec<Course>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.academy_list_courses().await
        .map_err(ServerFnError::new)
}

/// Server function to enroll in a course
#[server(EnrollAction, "/api")]
pub async fn enroll_action(course_id: String, user_id: String) -> Result<Enrollment, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = EnrollRequest { course_id, user_id };
    client.academy_enroll(&req).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn CoursesPage() -> impl IntoView {
    let courses = Resource::new(|| (), |_| list_courses_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"CURRICULUM"</h1>
                <p class="mt-2 text-slate-400">"Master pharmacovigilance through primitive-first competency pathways"</p>
            </header>

            <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                <Suspense fallback=|| view! { <LoadingGrid /> }>
                    {move || courses.get().map(|result| match result {
                        Ok(list) => view! { <CoursesListView list=list /> }.into_any(),
                        Err(e) => view! { <div class="col-span-full p-6 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 font-mono text-sm">{e.to_string()}</div> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn LoadingGrid() -> impl IntoView {
    view! {
        <div class="col-span-full grid gap-6 md:grid-cols-2 lg:grid-cols-3 w-full">
            {(0..6).map(|_| view! { 
                <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-6 h-64 animate-pulse">
                    <div class="h-4 w-20 bg-slate-800 rounded mb-4"></div>
                    <div class="h-6 w-48 bg-slate-800 rounded mb-2"></div>
                    <div class="h-4 w-full bg-slate-800 rounded mb-1"></div>
                    <div class="h-4 w-2/3 bg-slate-800 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

#[component]
fn CoursesListView(list: Vec<Course>) -> impl IntoView {
    if list.is_empty() {
        view! { <p class="col-span-full text-slate-500 italic text-center py-20 font-mono">"NO COURSES AVAILABLE AT THIS TIER"</p> }.into_any()
    } else {
        view! {
            {list.into_iter().map(|course| view! { <CourseCard course=course /> }).collect_view()}
        }.into_any()
    }
}

#[component]
fn CourseCard(course: Course) -> impl IntoView {
    let auth = use_auth();
    let enroll_action = ServerAction::<EnrollAction>::new();
    let result = enroll_action.value();
    let course_id = course.id.clone();

    let tier_color = match course.tier.as_str() {
        "T1" => "text-red-400 border-red-500/30 bg-red-500/5",
        "T2-P" => "text-amber-400 border-amber-500/30 bg-amber-500/5",
        _ => "text-cyan-400 border-cyan-500/30 bg-cyan-500/5",
    };

    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 flex flex-col hover:border-slate-700 transition-all group">
            <div class="flex justify-between items-start mb-4">
                <span class=format!("text-[10px] font-bold px-2 py-1 rounded border font-mono {}", tier_color)>
                    {course.tier.clone()}
                </span>
                <span class="text-[10px] font-bold text-slate-600 font-mono uppercase tracking-widest">
                    "LVL " {course.level}
                </span>
            </div>
            
            <h3 class="text-xl font-bold text-white font-mono tracking-tight group-hover:text-cyan-400 transition-colors mb-2">
                {course.title.clone()}
            </h3>
            
            <p class="text-sm text-slate-400 leading-relaxed flex-grow">
                {course.description.clone()}
            </p>

            <div class="mt-8 flex items-center justify-between pt-6 border-t border-slate-800/50">
                <span class="text-xs font-bold text-slate-500 font-mono uppercase">{course.code.clone()}</span>
                
                {move || match result.get() {
                    Some(Ok(_)) => view! { 
                        <span class="text-xs font-bold text-green-400 uppercase tracking-widest flex items-center gap-2">
                            "Enrolled" <span class="text-lg">"✓"</span>
                        </span>
                    }.into_any(),
                    _ => {
                        let cid = course_id.clone();
                        view! {
                            <button 
                                on:click=move |_| {
                                    let uid = auth.user.get().map(|u| u.uid).unwrap_or_else(|| "anonymous".to_string());
                                    enroll_action.dispatch(EnrollAction { 
                                        course_id: cid.clone(),
                                        user_id: uid,
                                    });
                                }
                                disabled=enroll_action.pending()
                                class="text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest flex items-center gap-2 disabled:opacity-50"
                            >
                                {move || if enroll_action.pending().get() { "Enrolling..." } else { "Enroll" }} 
                                <span class="text-lg">"\u{2192}"</span>
                            </button>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}