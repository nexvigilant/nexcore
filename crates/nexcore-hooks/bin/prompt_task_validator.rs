//! Task Quality Validator - UserPromptSubmit Event
//!
//! Validates task creation quality when TaskCreate appears to be intended.
//! Injects context to guide better task creation.
//!
//! Validates (via guidance):
//! - Subject is actionable (starts with verb)
//! - Description has acceptance criteria
//! - activeForm is present continuous
//! - No duplicate tasks exist
//!
//! Action: Inject validation context into prompt

use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};

/// Keywords that suggest task creation is about to happen
const TASK_KEYWORDS: &[&str] = &[
    "create task",
    "add task",
    "new task",
    "taskcreate",
    "task list",
    "todo",
    "track this",
];

/// Good action verbs for task subjects
const ACTION_VERBS: &[&str] = &[
    "implement",
    "add",
    "create",
    "fix",
    "update",
    "remove",
    "refactor",
    "test",
    "validate",
    "configure",
    "deploy",
    "migrate",
    "optimize",
    "document",
    "review",
    "debug",
    "investigate",
    "analyze",
    "design",
    "build",
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let prompt = match input.get_prompt() {
        Some(p) => p.to_lowercase(),
        None => exit_skip_prompt(),
    };

    // Check if this looks like task creation
    let is_task_related = TASK_KEYWORDS.iter().any(|k| prompt.contains(k));

    if !is_task_related {
        exit_skip_prompt();
    }

    // Build guidance context
    let verbs_list = ACTION_VERBS[..10].join(", ");

    let context = format!(
        r#"📋 **TASK CREATION GUIDANCE**

When creating tasks, ensure high quality:

**Subject (required):**
- Start with an action verb: {}
- Be specific and measurable
- Good: "Implement user authentication with JWT"
- Bad: "Auth stuff"

**Description (required):**
- Include acceptance criteria
- Define what "done" looks like
- List dependencies if any

**activeForm (recommended):**
- Present continuous for status display
- Example: "Implementing user authentication"

**Example quality task:**
```
Subject: "Fix login timeout error on slow connections"
Description: "The login form times out after 5 seconds on slow 3G connections.
  Acceptance: Login works on 3G speeds with 30s timeout.
  Test: Throttle to 3G in DevTools and verify login completes."
activeForm: "Fixing login timeout error"
```

Create tasks that clearly communicate progress and completion criteria.
"#,
        verbs_list
    );

    exit_with_context(&context);
}
