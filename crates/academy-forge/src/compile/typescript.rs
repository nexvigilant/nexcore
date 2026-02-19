//! TypeScript code generation for Studio-compatible pathway files.
//!
//! Transforms forge pathway JSON (`tov-01.json`) into TypeScript source files
//! that export `CapabilityStage` objects matching the Studio type definitions
//! from `@/types/academy`.
//!
//! ## Output Structure
//!
//! ```text
//! output_dir/
//! ├── stages/
//! │   ├── 01-system-decomposition.ts   ← one per stage
//! │   └── 02-hierarchical-organization.ts
//! ├── config.ts                        ← pathway metadata
//! └── index.ts                         ← assembler + StaticPathway
//! ```
//!
//! ## Type Mapping
//!
//! | Forge JSON | Studio TypeScript |
//! |---|---|
//! | `stages[].activities[]` | `CapabilityStage.lessons[]` |
//! | `activities[].type` | Dropped |
//! | `activities[].quiz` | `PracticeActivity.assessment` |
//! | `stages[].passingScore` | `assessment.passingScore` on quiz activities |
//! | `estimatedDuration: "15 minutes"` | `estimatedDuration: 15` (number) |
//! | No `content` field | `content: \`## Title\n\nTODO\`` |
//! | `correctAnswer: true` (T/F) | `correctAnswer: 1` (as `0 \| 1`) |
//! | `correctAnswer: false` (T/F) | `correctAnswer: 0` (as `0 \| 1`) |

use crate::error::{ForgeError, ForgeResult};

// ═══════════════════════════════════════════════════════════════════════════
// String utility functions
// ═══════════════════════════════════════════════════════════════════════════

/// Convert a title string into a URL-safe slug.
///
/// Lowercases the input, replaces all non-alphanumeric characters with `-`,
/// and collapses consecutive dashes into a single dash.
///
/// # Examples
///
/// ```
/// use academy_forge::compile::typescript::slugify;
///
/// assert_eq!(slugify("System Decomposition"), "system-decomposition");
/// assert_eq!(slugify("Harm Types A through H"), "harm-types-a-through-h");
/// assert_eq!(slugify("A1 — Core Axiom!"), "a1-core-axiom");
/// ```
pub fn slugify(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut result = String::with_capacity(lower.len());
    let mut prev_dash = false;

    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            result.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            result.push('-');
            prev_dash = true;
        }
    }

    // Trim trailing dash
    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Parse a human-readable duration string into whole minutes.
///
/// Supports the formats produced by the scaffold generator:
/// - `"15 minutes"` → `15`
/// - `"1 hour"` → `60`
/// - `"2 hours"` → `120`
/// - `"12 hours"` → `720`
///
/// Falls back to `0` for unrecognised strings.
///
/// # Examples
///
/// ```
/// use academy_forge::compile::typescript::parse_duration_minutes;
///
/// assert_eq!(parse_duration_minutes("15 minutes"), 15);
/// assert_eq!(parse_duration_minutes("1 hour"), 60);
/// assert_eq!(parse_duration_minutes("2 hours"), 120);
/// assert_eq!(parse_duration_minutes("12 hours"), 720);
/// assert_eq!(parse_duration_minutes("unknown"), 0);
/// ```
pub fn parse_duration_minutes(s: &str) -> u32 {
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    if parts.len() < 2 {
        return 0;
    }

    let Ok(amount) = parts[0].parse::<u32>() else {
        return 0;
    };

    let unit = parts[1].trim().to_lowercase();
    if unit.starts_with("minute") {
        amount
    } else if unit.starts_with("hour") {
        amount * 60
    } else {
        0
    }
}

/// Escape a string for use inside a TypeScript single-quoted string literal.
///
/// Escapes backslash, single quote, and newline characters.
fn escape_ts_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out
}

/// Escape a string for use inside a TypeScript template literal (backtick string).
///
/// Escapes backslash, backtick, and `${` interpolation sequences.
fn escape_ts_template(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                out.push_str("\\\\");
                i += 1;
            }
            '`' => {
                out.push_str("\\`");
                i += 1;
            }
            '$' if i + 1 < chars.len() && chars[i + 1] == '{' => {
                out.push_str("\\${");
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage rendering
// ═══════════════════════════════════════════════════════════════════════════

/// Render a single stage JSON value as a TypeScript source file.
///
/// Produces a file exporting `const stageNN: CapabilityStage = { ... }`.
///
/// # Parameters
///
/// - `stage` — The JSON object for one stage from the pathway's `stages[]` array.
/// - `var_name` — The TypeScript variable name to export (e.g. `"stage01"`).
/// - `passing_score` — The stage-level passing score, applied to quiz activities.
///
/// # Errors
///
/// Returns [`ForgeError::ParseError`] if required fields are missing or malformed.
pub fn render_stage(
    stage: &serde_json::Value,
    var_name: &str,
    passing_score: u64,
) -> ForgeResult<String> {
    let stage_id = stage["id"].as_str().ok_or_else(|| ForgeError::ParseError {
        file: "stage".to_string(),
        message: "stage missing 'id' field".to_string(),
    })?;
    let title = stage["title"]
        .as_str()
        .ok_or_else(|| ForgeError::ParseError {
            file: stage_id.to_string(),
            message: "stage missing 'title' field".to_string(),
        })?;
    let description = stage["description"].as_str().unwrap_or("");

    let activities = stage["activities"]
        .as_array()
        .ok_or_else(|| ForgeError::ParseError {
            file: stage_id.to_string(),
            message: "stage missing 'activities' array".to_string(),
        })?;

    let mut lessons = Vec::with_capacity(activities.len());
    for activity in activities {
        lessons.push(render_lesson(activity, stage_id, passing_score)?);
    }

    let lessons_joined = lessons.join(",\n");

    Ok(format!(
        r"/**
 * Stage: {title}
 *
 * Generated from tov-01.json by academy-forge compile module.
 * Edit the source JSON to update content, then re-run forge_compile.
 */

import type {{ CapabilityStage }} from '@/types/academy';

export const {var_name}: CapabilityStage = {{
  id: '{stage_id_escaped}',
  title: '{title_escaped}',
  description: '{description_escaped}',
  lessons: [
{lessons_joined}
  ],
}};
",
        title = escape_ts_string(title),
        var_name = var_name,
        stage_id_escaped = escape_ts_string(stage_id),
        title_escaped = escape_ts_string(title),
        description_escaped = escape_ts_string(description),
        lessons_joined = lessons_joined,
    ))
}

/// Render a single activity JSON object as a TypeScript `PracticeActivity` literal.
fn render_lesson(
    activity: &serde_json::Value,
    stage_id: &str,
    passing_score: u64,
) -> ForgeResult<String> {
    let activity_id = activity["id"]
        .as_str()
        .ok_or_else(|| ForgeError::ParseError {
            file: stage_id.to_string(),
            message: "activity missing 'id' field".to_string(),
        })?;
    let title = activity["title"]
        .as_str()
        .ok_or_else(|| ForgeError::ParseError {
            file: activity_id.to_string(),
            message: "activity missing 'title' field".to_string(),
        })?;
    let duration_str = activity["estimatedDuration"]
        .as_str()
        .unwrap_or("0 minutes");
    let duration_num = parse_duration_minutes(duration_str);

    let activity_type = activity["type"].as_str().unwrap_or("reading");
    let is_quiz = activity_type == "quiz";

    // Build content block
    let content_line = build_content_line(title, is_quiz);

    // Build assessment block if quiz
    let assessment_block = if is_quiz {
        if let Some(quiz) = activity.get("quiz") {
            let rendered = render_assessment(quiz, activity_id, passing_score)?;
            format!("      assessment: {rendered},\n")
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let content_id = format!("      id: '{}',\n", escape_ts_string(activity_id));
    let content_title = format!("      title: '{}',\n", escape_ts_string(title));
    let content_desc = format!("      description: '{}',\n", escape_ts_string(title));
    let content_duration = format!("      estimatedDuration: {duration_num},\n");

    Ok(format!(
        "    {{\n\
         {content_id}\
         {content_title}\
         {content_desc}\
         {content_line}\
         {content_duration}\
         {assessment_block}\
         }}"
    ))
}

/// Build the `content:` line for a lesson.
///
/// Quiz activities get an empty content string; other activities get a
/// placeholder markdown template with the activity title.
fn build_content_line(title: &str, is_quiz: bool) -> String {
    if is_quiz {
        "      content: '',\n".to_string()
    } else {
        let escaped = escape_ts_template(title);
        format!("      content: `## {escaped}\\n\\nTODO: Add content for this activity.`,\n")
    }
}

/// Render an assessment block from a quiz JSON object.
fn render_assessment(
    quiz: &serde_json::Value,
    activity_id: &str,
    passing_score: u64,
) -> ForgeResult<String> {
    let questions_val = quiz["questions"]
        .as_array()
        .ok_or_else(|| ForgeError::ParseError {
            file: activity_id.to_string(),
            message: "quiz missing 'questions' array".to_string(),
        })?;

    let mut rendered_questions = Vec::with_capacity(questions_val.len());
    for q in questions_val {
        rendered_questions.push(render_question(q, activity_id)?);
    }

    let questions_joined = rendered_questions.join(",\n");

    Ok(format!(
        "{{\n\
         {indent}  type: 'quiz',\n\
         {indent}  passingScore: {passing_score},\n\
         {indent}  questions: [\n\
         {questions_joined}\n\
         {indent}  ],\n\
         {indent}}}",
        indent = "      ",
        passing_score = passing_score,
        questions_joined = questions_joined,
    ))
}

/// Render a single quiz question JSON object as a TypeScript question literal.
fn render_question(q: &serde_json::Value, activity_id: &str) -> ForgeResult<String> {
    let q_id = q["id"].as_str().ok_or_else(|| ForgeError::ParseError {
        file: activity_id.to_string(),
        message: "question missing 'id' field".to_string(),
    })?;
    let q_type = q["type"].as_str().unwrap_or("multiple-choice");
    let question_text = q["question"].as_str().unwrap_or("");
    let explanation = q["explanation"].as_str().unwrap_or("");
    let points = q["points"].as_u64().unwrap_or(1);

    let type_specific = match q_type {
        "true-false" => {
            // Convert boolean correctAnswer to 0|1
            let correct: u8 = u8::from(q["correctAnswer"].as_bool().unwrap_or(false));
            format!(
                "        type: 'true-false',\n\
                 {indent}correctAnswer: {correct} as 0 | 1,\n",
                indent = "        ",
                correct = correct,
            )
        }
        "multiple-select" => {
            // correctAnswer is an array of indices
            let correct_arr = if let Some(arr) = q["correctAnswer"].as_array() {
                let indices: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_u64())
                    .map(|n| n.to_string())
                    .collect();
                format!("[{}]", indices.join(", "))
            } else {
                "[]".to_string()
            };
            let options_str = render_options_array(q, "        ");
            format!(
                "        type: 'multiple-select',\n\
                 {options_str}\
                 {indent}correctAnswer: {correct_arr},\n",
                indent = "        ",
                options_str = options_str,
                correct_arr = correct_arr,
            )
        }
        _ => {
            // multiple-choice (default)
            let correct = q["correctAnswer"].as_u64().unwrap_or(0);
            let options_str = render_options_array(q, "        ");
            format!(
                "        type: 'multiple-choice',\n\
                 {options_str}\
                 {indent}correctAnswer: {correct},\n",
                indent = "        ",
                options_str = options_str,
                correct = correct,
            )
        }
    };

    let id_line = format!("          id: '{}',\n", escape_ts_string(q_id));
    let question_line = format!(
        "          question: '{}',\n",
        escape_ts_string(question_text)
    );
    let explanation_line = if explanation.is_empty() {
        String::new()
    } else {
        format!(
            "          explanation: '{}',\n",
            escape_ts_string(explanation)
        )
    };
    let points_line = format!("          points: {points},\n");

    Ok(format!(
        "        {{\n\
         {id_line}\
         {type_specific}\
         {question_line}\
         {explanation_line}\
         {points_line}\
         {indent}}}",
        indent = "        ",
    ))
}

/// Render the `options: [...]` array for a question, indented at the given prefix.
fn render_options_array(q: &serde_json::Value, indent: &str) -> String {
    if let Some(arr) = q["options"].as_array() {
        let items: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| format!("{indent}  '{}',", escape_ts_string(s)))
            .collect();
        format!(
            "{indent}options: [\n{}\n{indent}],\n",
            items.join("\n"),
            indent = indent,
        )
    } else {
        format!("{indent}options: [],\n", indent = indent)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Config rendering
// ═══════════════════════════════════════════════════════════════════════════

/// Render a `config.ts` file containing pathway-level metadata.
///
/// # Errors
///
/// Returns [`ForgeError::ParseError`] if required top-level fields are absent.
pub fn render_config(pathway: &serde_json::Value) -> ForgeResult<String> {
    let id = pathway["id"]
        .as_str()
        .ok_or_else(|| ForgeError::ParseError {
            file: "pathway".to_string(),
            message: "pathway missing 'id' field".to_string(),
        })?;
    let title = pathway["title"]
        .as_str()
        .ok_or_else(|| ForgeError::ParseError {
            file: id.to_string(),
            message: "pathway missing 'title' field".to_string(),
        })?;
    let description = pathway["description"].as_str().unwrap_or("");
    let domain = pathway["domain"].as_str().unwrap_or("vigilance");
    let component_count = pathway["componentCount"].as_u64().unwrap_or(0);

    // Parse estimatedDuration to minutes for metadata
    let duration_str = pathway["estimatedDuration"].as_str().unwrap_or("0 minutes");
    let duration_minutes = parse_duration_minutes(duration_str);

    Ok(format!(
        r"/**
 * Pathway Configuration
 *
 * Generated from {id}.json by academy-forge compile module.
 * Edit the source JSON to update metadata, then re-run forge_compile.
 */

export const PATHWAY_CONFIG = {{
  id: '{id_escaped}',
  title: '{title_escaped}',
  description: '{description_escaped}',
  topic: '{title_escaped}',
  domain: '{domain_escaped}',
  status: 'published' as const,
  visibility: 'public' as const,
  qualityScore: 90,
  targetAudience: 'Practitioners and learners building expertise in {title_escaped}.',
  difficulty: 'intermediate' as const,
  metadata: {{
    estimatedDuration: {duration_minutes},
    componentCount: {component_count},
  }},
  instructor: {{
    name: 'NexVigilant Academy',
    bio: 'Empowerment Through Vigilance',
  }},
  version: 1,
}} as const;
",
        id = id,
        id_escaped = escape_ts_string(id),
        title_escaped = escape_ts_string(title),
        description_escaped = escape_ts_string(description),
        domain_escaped = escape_ts_string(domain),
        duration_minutes = duration_minutes,
        component_count = component_count,
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
// Index rendering
// ═══════════════════════════════════════════════════════════════════════════

/// Render an `index.ts` file that assembles the full `StaticPathway`.
///
/// # Parameters
///
/// - `stage_imports` — List of `(var_name, file_stem)` pairs for each stage,
///   e.g. `[("stage01", "01-system-decomposition"), ...]`.
/// - `stage_count` — Total number of stages (used for comment annotation).
///
/// # Errors
///
/// Returns [`ForgeError::ParseError`] if the stage list is empty.
pub fn render_index(stage_imports: &[(String, String)], stage_count: usize) -> ForgeResult<String> {
    if stage_imports.is_empty() {
        return Err(ForgeError::ParseError {
            file: "index".to_string(),
            message: "cannot render index.ts with zero stages".to_string(),
        });
    }

    let mut import_lines = Vec::with_capacity(stage_imports.len());
    for (var_name, file_stem) in stage_imports {
        import_lines.push(format!(
            "import {{ {var_name} }} from './stages/{file_stem}';"
        ));
    }

    let stages_array: Vec<String> = stage_imports
        .iter()
        .map(|(var_name, _)| format!("  {var_name},"))
        .collect();

    Ok(format!(
        r"/**
 * Pathway Assembler
 *
 * Auto-assembles the full CapabilityPathway from {stage_count} stage files.
 * Generated by academy-forge compile module.
 * To update content, edit the stage files in ./stages/.
 * To update pathway metadata, edit ./config.ts.
 *
 * @example
 * ```tsx
 * import {{ pathway }} from './index';
 * ```
 */

import type {{ CapabilityPathway }} from '@/types/academy';

import {{ PATHWAY_CONFIG }} from './config';
{imports}

/**
 * Static pathway type — excludes Firestore runtime fields.
 */
export type StaticPathway = Omit<CapabilityPathway, 'userId' | 'createdAt' | 'updatedAt'>;

/**
 * All stages in order. Add or reorder stages here.
 */
const stages = [
{stages_array}
] as const;

/**
 * The complete assembled pathway.
 */
export const pathway: StaticPathway = {{
  id: PATHWAY_CONFIG.id,
  title: PATHWAY_CONFIG.title,
  description: PATHWAY_CONFIG.description,
  topic: PATHWAY_CONFIG.topic,
  modules: [...stages],
  status: PATHWAY_CONFIG.status,
  visibility: PATHWAY_CONFIG.visibility,
  qualityScore: PATHWAY_CONFIG.qualityScore,
  domain: PATHWAY_CONFIG.domain,
  targetAudience: PATHWAY_CONFIG.targetAudience,
  difficulty: PATHWAY_CONFIG.difficulty,
  metadata: PATHWAY_CONFIG.metadata,
  instructor: PATHWAY_CONFIG.instructor,
  version: PATHWAY_CONFIG.version,
}};

/**
 * Get a specific stage by index (0-based).
 */
export function getStage(index: number) {{
  return stages[index];
}}

/**
 * Get total activity count across all stages.
 */
export function getTotalActivities(): number {{
  return stages.reduce((sum, stage) => sum + stage.lessons.length, 0);
}}

/**
 * Get total estimated duration in minutes.
 */
export function getTotalDuration(): number {{
  return stages.reduce(
    (sum, stage) =>
      sum + stage.lessons.reduce((s, l) => s + (l.estimatedDuration ?? 0), 0),
    0
  );
}}

export {{ stages, PATHWAY_CONFIG }};
",
        stage_count = stage_count,
        imports = import_lines.join("\n"),
        stages_array = stages_array.join("\n"),
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────
    // slugify
    // ───────────────────────────────────────────────

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("System Decomposition"), "system-decomposition");
        assert_eq!(slugify("Harm Types A through H"), "harm-types-a-through-h");
    }

    #[test]
    fn test_slugify_special_chars() {
        // Trailing non-alphanumeric chars produce a dash that gets trimmed
        assert_eq!(slugify("A1 — Core Axiom!"), "a1-core-axiom");
        assert_eq!(
            slugify("Conservation Laws in Practice"),
            "conservation-laws-in-practice"
        );
    }

    #[test]
    fn test_slugify_already_clean() {
        assert_eq!(slugify("hello-world"), "hello-world");
        assert_eq!(slugify("abc123"), "abc123");
    }

    #[test]
    fn test_slugify_collapses_dashes() {
        // Multiple consecutive non-alphanumeric chars → single dash
        assert_eq!(slugify("foo  bar"), "foo-bar");
        assert_eq!(slugify("foo -- bar"), "foo-bar");
    }

    // ───────────────────────────────────────────────
    // parse_duration_minutes
    // ───────────────────────────────────────────────

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration_minutes("15 minutes"), 15);
        assert_eq!(parse_duration_minutes("45 minutes"), 45);
        assert_eq!(parse_duration_minutes("1 hour"), 60);
        assert_eq!(parse_duration_minutes("2 hours"), 120);
        assert_eq!(parse_duration_minutes("12 hours"), 720);
    }

    #[test]
    fn test_parse_duration_unknown() {
        assert_eq!(parse_duration_minutes("unknown"), 0);
        assert_eq!(parse_duration_minutes(""), 0);
        assert_eq!(parse_duration_minutes("abc minutes"), 0);
    }

    // ───────────────────────────────────────────────
    // render_stage (no quiz)
    // ───────────────────────────────────────────────

    #[test]
    fn test_render_stage_basic() {
        let stage = serde_json::json!({
            "id": "tov-01-01",
            "title": "System Decomposition",
            "description": "Learn about A1.",
            "passingScore": 70,
            "estimatedDuration": "45 minutes",
            "activities": [
                {
                    "id": "tov-01-01-a01",
                    "title": "Reading the Basics",
                    "type": "reading",
                    "estimatedDuration": "15 minutes"
                },
                {
                    "id": "tov-01-01-a02",
                    "title": "Interactive Practice",
                    "type": "interactive",
                    "estimatedDuration": "20 minutes"
                }
            ]
        });

        let result = render_stage(&stage, "stage01", 70).unwrap();

        // Must export the correct variable
        assert!(result.contains("export const stage01: CapabilityStage = {"));
        // Must have correct id and title
        assert!(result.contains("id: 'tov-01-01'"));
        assert!(result.contains("title: 'System Decomposition'"));
        // Activities become lessons
        assert!(result.contains("lessons: ["));
        assert!(result.contains("id: 'tov-01-01-a01'"));
        assert!(result.contains("estimatedDuration: 15,"));
        assert!(result.contains("estimatedDuration: 20,"));
        // Non-quiz activities get TODO content
        assert!(result.contains("TODO: Add content for this activity."));
        // No assessment block for non-quiz
        assert!(!result.contains("assessment:"));
    }

    // ───────────────────────────────────────────────
    // render_stage (with quiz)
    // ───────────────────────────────────────────────

    #[test]
    fn test_render_stage_with_quiz() {
        let stage = serde_json::json!({
            "id": "tov-01-01",
            "title": "System Decomposition",
            "description": "Learn about A1.",
            "passingScore": 70,
            "estimatedDuration": "45 minutes",
            "activities": [
                {
                    "id": "tov-01-01-a03",
                    "title": "Assessment",
                    "type": "quiz",
                    "estimatedDuration": "15 minutes",
                    "quiz": {
                        "questions": [
                            {
                                "id": "tov-01-01-q01",
                                "type": "multiple-choice",
                                "question": "What is A1?",
                                "options": ["Decomposition", "Hierarchy", "Conservation", "Manifold"],
                                "correctAnswer": 0,
                                "points": 2,
                                "explanation": "A1 is System Decomposition."
                            },
                            {
                                "id": "tov-01-01-q02",
                                "type": "true-false",
                                "question": "A1 has no dependencies.",
                                "correctAnswer": true,
                                "points": 1,
                                "explanation": "Correct, A1 is a root axiom."
                            },
                            {
                                "id": "tov-01-01-q03",
                                "type": "true-false",
                                "question": "A5 is a root axiom.",
                                "correctAnswer": false,
                                "points": 1,
                                "explanation": "A5 depends on A2 and A4."
                            }
                        ]
                    }
                }
            ]
        });

        let result = render_stage(&stage, "stage01", 70).unwrap();

        // Assessment block present
        assert!(result.contains("assessment: {"));
        assert!(result.contains("type: 'quiz'"));
        assert!(result.contains("passingScore: 70"));

        // MC question rendered with options
        assert!(result.contains("type: 'multiple-choice'"));
        assert!(result.contains("correctAnswer: 0,"));
        assert!(result.contains("'Decomposition'"));

        // T/F true → 1
        assert!(result.contains("correctAnswer: 1 as 0 | 1"));
        // T/F false → 0
        assert!(result.contains("correctAnswer: 0 as 0 | 1"));

        // Quiz activity content should be empty string
        assert!(result.contains("content: '',"));
    }

    // ───────────────────────────────────────────────
    // render_stage with multiple-select
    // ───────────────────────────────────────────────

    #[test]
    fn test_render_stage_multiple_select() {
        let stage = serde_json::json!({
            "id": "tov-01-03",
            "title": "Conservation",
            "description": "A3.",
            "passingScore": 75,
            "estimatedDuration": "60 minutes",
            "activities": [
                {
                    "id": "tov-01-03-a04",
                    "title": "Conservation Quiz",
                    "type": "quiz",
                    "estimatedDuration": "20 minutes",
                    "quiz": {
                        "questions": [
                            {
                                "id": "tov-01-03-q02",
                                "type": "multiple-select",
                                "question": "Which are conservation laws?",
                                "options": ["Catalyst Invariance", "Signal Amplification", "Entropy Increase", "Momentum"],
                                "correctAnswer": [0, 2, 3],
                                "points": 3,
                                "explanation": "Laws 5, 6, and 7."
                            }
                        ]
                    }
                }
            ]
        });

        let result = render_stage(&stage, "stage03", 75).unwrap();

        assert!(result.contains("type: 'multiple-select'"));
        assert!(result.contains("correctAnswer: [0, 2, 3]"));
        assert!(result.contains("'Catalyst Invariance'"));
    }

    // ───────────────────────────────────────────────
    // render_config
    // ───────────────────────────────────────────────

    #[test]
    fn test_render_config() {
        let pathway = serde_json::json!({
            "id": "tov-01",
            "title": "Theory of Vigilance: Foundations",
            "description": "Master the five axioms.",
            "domain": "vigilance",
            "componentCount": 39,
            "estimatedDuration": "12 hours"
        });

        let result = render_config(&pathway).unwrap();

        assert!(result.contains("export const PATHWAY_CONFIG = {"));
        assert!(result.contains("id: 'tov-01'"));
        assert!(result.contains("title: 'Theory of Vigilance: Foundations'"));
        assert!(result.contains("domain: 'vigilance'"));
        // 12 hours → 720 minutes
        assert!(result.contains("estimatedDuration: 720,"));
        assert!(result.contains("componentCount: 39,"));
        assert!(result.contains("status: 'published' as const"));
    }

    // ───────────────────────────────────────────────
    // render_index
    // ───────────────────────────────────────────────

    #[test]
    fn test_render_index() {
        let imports = vec![
            ("stage01".to_string(), "01-system-decomposition".to_string()),
            (
                "stage02".to_string(),
                "02-hierarchical-organization".to_string(),
            ),
        ];

        let result = render_index(&imports, 2).unwrap();

        // Imports present
        assert!(result.contains("import { stage01 } from './stages/01-system-decomposition';"));
        assert!(
            result.contains("import { stage02 } from './stages/02-hierarchical-organization';")
        );

        // Stages array
        assert!(result.contains("stage01,"));
        assert!(result.contains("stage02,"));

        // StaticPathway export
        assert!(result.contains("export const pathway: StaticPathway = {"));
        assert!(result.contains("modules: [...stages]"));

        // Helper functions
        assert!(result.contains("export function getStage"));
        assert!(result.contains("export function getTotalActivities"));
        assert!(result.contains("export function getTotalDuration"));
    }

    #[test]
    fn test_render_index_empty_stages_errors() {
        let result = render_index(&[], 0);
        assert!(result.is_err());
    }

    // ───────────────────────────────────────────────
    // escape functions (internal)
    // ───────────────────────────────────────────────

    #[test]
    fn test_escape_ts_string() {
        assert_eq!(escape_ts_string("it's fine"), "it\\'s fine");
        assert_eq!(escape_ts_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_ts_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_escape_ts_template() {
        assert_eq!(escape_ts_template("hello `world`"), "hello \\`world\\`");
        assert_eq!(escape_ts_template("value: ${x}"), "value: \\${x}");
        assert_eq!(escape_ts_template("back\\slash"), "back\\\\slash");
    }
}
