/// Academy — courses, enrollments, KSB domains
/// Tier: T3 (π Persistence + μ Mapping + Σ Sum)
use leptos::prelude::*;

use crate::api::academy;
use crate::components::progress_bar::ProgressBar;

#[component]
pub fn AcademyPage() -> impl IntoView {
    let courses = LocalResource::new(|| academy::list_courses());
    let enrollments = LocalResource::new(|| academy::list_enrollments());
    let pathways = LocalResource::new(|| academy::list_pathways());

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"Academy"</h1>
                <p class="page-subtitle">"Professional Development"</p>
            </header>

            // Enrollments section
            <section class="academy-section">
                <h2 class="section-title">"My Progress"</h2>
                <Suspense fallback=move || view! { <div class="loading">"Loading..."</div> }>
                    {move || {
                        enrollments.read().as_ref().map(|result| {
                            match result {
                                Ok(items) if !items.is_empty() => {
                                    view! {
                                        <div class="enrollment-list">
                                            {items.iter().map(|e| {
                                                let pct = (e.progress * 100.0) as u32;
                                                view! {
                                                    <div class="enrollment-card">
                                                        <div class="enrollment-header">
                                                            <span class="enrollment-title">{e.course_title.clone()}</span>
                                                            <span class="enrollment-code">{e.course_code.clone()}</span>
                                                        </div>
                                                        <ProgressBar value=pct max=100 />
                                                        {e.completed_at.as_ref().map(|date| {
                                                            view! { <span class="enrollment-completed">"Completed "{date.clone()}</span> }
                                                        })}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                },
                                Ok(_) => view! {
                                    <div class="empty-state">
                                        <p>"No enrollments yet"</p>
                                        <p class="empty-hint">"Enroll in courses below to start learning"</p>
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="error-card">
                                        <div class="error-msg">{e.message.clone()}</div>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>

            // Courses section
            <section class="academy-section">
                <h2 class="section-title">"Courses"</h2>
                <Suspense fallback=move || view! { <div class="loading">"Loading..."</div> }>
                    {move || {
                        courses.read().as_ref().map(|result| {
                            match result {
                                Ok(items) => view! {
                                    <div class="course-grid">
                                        {items.iter().map(|c| {
                                            let tier_class = format!("course-tier tier-{}", c.tier.to_lowercase());
                                            view! {
                                                <div class="course-card">
                                                    <div class="course-header">
                                                        <span class="course-code">{c.code.clone()}</span>
                                                        <span class=tier_class>{c.tier.clone()}</span>
                                                    </div>
                                                    <h3 class="course-title">{c.title.clone()}</h3>
                                                    <p class="course-desc">{c.description.clone()}</p>
                                                    <span class="course-level">{c.level.clone()}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="error-card">
                                        <div class="error-msg">{e.message.clone()}</div>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>

            // Pathways section
            <section class="academy-section">
                <h2 class="section-title">"Learning Pathways"</h2>
                <Suspense fallback=move || view! { <div class="loading">"Loading..."</div> }>
                    {move || {
                        pathways.read().as_ref().map(|result| {
                            match result {
                                Ok(items) => view! {
                                    <div class="pathway-list">
                                        {items.iter().map(|p| {
                                            view! {
                                                <div class="pathway-card">
                                                    <h3 class="pathway-name">{p.name.clone()}</h3>
                                                    <div class="pathway-nodes">
                                                        {p.nodes.iter().map(|n| {
                                                            view! {
                                                                <div class="pathway-node">
                                                                    <span class="node-title">{n.title.clone()}</span>
                                                                    <span class="node-level">{n.level.clone()}</span>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="error-card">
                                        <div class="error-msg">{e.message.clone()}</div>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>
        </div>
    }
}
