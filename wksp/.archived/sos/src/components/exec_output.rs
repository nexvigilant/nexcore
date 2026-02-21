/// Execution output terminal — σ Sequence (output lines) + ς State (running/done)
use leptos::prelude::*;

use crate::api::skills::SkillExecResult;

#[component]
pub fn ExecOutput(result: SkillExecResult) -> impl IntoView {
    let exit_class = if result.exit_code == 0 {
        "exec-exit exec-success"
    } else {
        "exec-exit exec-failure"
    };

    view! {
        <div class="exec-output">
            <div class="exec-header">
                <span class={exit_class}>
                    {if result.exit_code == 0 { "PASS" } else { "FAIL" }}
                </span>
                <span class="exec-duration">{format!("{}ms", result.duration_ms)}</span>
            </div>
            {if !result.stdout.is_empty() {
                Some(view! {
                    <pre class="exec-terminal">{result.stdout.clone()}</pre>
                })
            } else {
                None
            }}
            {if !result.stderr.is_empty() {
                Some(view! {
                    <pre class="exec-terminal exec-stderr">{result.stderr.clone()}</pre>
                })
            } else {
                None
            }}
        </div>
    }
}
