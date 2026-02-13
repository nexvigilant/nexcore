//! Rust source code generation for compound skills.

use std::path::{Path, PathBuf};

use crate::error::{CompilerError, Result};
use crate::spec::{CompositionStrategy, CompoundSpec, MergeStrategy};

/// Output of the codegen stage.
#[derive(Debug)]
pub struct GeneratedCrate {
    /// Root directory of the generated crate.
    pub root: PathBuf,
    /// Path to generated main.rs.
    pub main_rs: PathBuf,
    /// Path to generated Cargo.toml.
    pub cargo_toml: PathBuf,
    /// Path to generated SKILL.md.
    pub skill_md: PathBuf,
}

/// Generate a compound skill crate from a spec.
///
/// # Errors
///
/// Returns `CompilerError::CodegenFailed` on I/O or generation failures.
pub fn generate(spec: &CompoundSpec, output_dir: &Path) -> Result<GeneratedCrate> {
    let crate_dir = output_dir.join(&spec.compound.name);
    let src_dir = crate_dir.join("src");

    std::fs::create_dir_all(&src_dir).map_err(|e| CompilerError::CodegenFailed {
        stage: "mkdir".into(),
        message: e.to_string(),
    })?;

    let main_rs = src_dir.join("main.rs");
    let cargo_toml = crate_dir.join("Cargo.toml");
    let skill_md = crate_dir.join("SKILL.md");

    std::fs::write(&main_rs, generate_main_rs(spec)).map_err(|e| CompilerError::CodegenFailed {
        stage: "write main.rs".into(),
        message: e.to_string(),
    })?;
    std::fs::write(&cargo_toml, generate_cargo_toml(spec)).map_err(|e| {
        CompilerError::CodegenFailed {
            stage: "write Cargo.toml".into(),
            message: e.to_string(),
        }
    })?;
    std::fs::write(&skill_md, generate_skill_md(spec)).map_err(|e| {
        CompilerError::CodegenFailed {
            stage: "write SKILL.md".into(),
            message: e.to_string(),
        }
    })?;

    Ok(GeneratedCrate {
        root: crate_dir,
        main_rs,
        cargo_toml,
        skill_md,
    })
}

fn generate_main_rs(spec: &CompoundSpec) -> String {
    let skill_entries = spec
        .skills
        .iter()
        .map(|s| {
            format!(
                "    SkillStep {{ name: \"{}\".into(), required: {}, timeout_secs: {} }}",
                s.name, s.required, s.timeout_seconds,
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let merge_fn = match spec.threading.as_ref().map(|t| t.merge_strategy) {
        Some(MergeStrategy::Overwrite) | None => "overwrite_merge",
        Some(MergeStrategy::DeepMerge) => "deep_merge",
    };

    let execution_body = match spec.compound.strategy {
        CompositionStrategy::Sequential => gen_sequential(merge_fn),
        CompositionStrategy::Parallel => gen_parallel(merge_fn),
        CompositionStrategy::FeedbackLoop => {
            let fb = spec.feedback.as_ref();
            let max_iter = fb.map_or(5, |f| f.max_iterations);
            let field = fb.map_or("quality_score", |f| f.convergence_field.as_str());
            let threshold = fb.map_or(0.85, |f| f.convergence_threshold);
            gen_feedback(merge_fn, max_iter, field, threshold)
        }
    };

    let name = &spec.compound.name;
    let strategy = &spec.compound.strategy;

    format!(
        r##"//! Auto-generated compound skill: {name} (strategy: {strategy})
use std::collections::HashMap;
use std::io::Read;
use std::process::{{Command, Stdio}};
use std::time::Instant;
use serde::Serialize;
use serde_json::{{Value, json}};

#[derive(Debug, Clone)]
struct SkillStep {{ name: String, required: bool, timeout_secs: u64 }}

#[derive(Debug, Serialize)]
struct StepDetail {{
    skill: String,
    status: String,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}}

fn main() {{
    let skills = vec![
{skill_entries}
    ];

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    let input_json: Value = serde_json::from_str(&input).unwrap_or_else(|_| json!({{}}));

    let mut accumulator: HashMap<String, Value> = HashMap::new();
    accumulator.insert("_input".into(), input_json);
    let mut step_details: Vec<StepDetail> = Vec::new();
    let mut steps_completed: usize = 0;
    let mut failed = false;

{execution_body}

    let output = json!({{
        "compound": "{name}",
        "status": if failed {{ "failed" }} else {{ "completed" }},
        "steps_completed": steps_completed,
        "results": accumulator,
        "step_details": step_details,
    }});
    println!("{{}}", output);
    if failed {{ std::process::exit(1); }}
}}

fn execute_skill(name: &str, input: &Value, _timeout_secs: u64) -> Result<(Value, u64), String> {{
    let start = Instant::now();
    let base = std::env::var("SKILL_BASE_DIR").ok()
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".claude").join("skills")))
        .unwrap_or_default();
    let skill_path = base.join(name);
    let scripts_dir = skill_path.join("scripts");
    let binary = scripts_dir.join("target").join("release").join(name);
    let shell = scripts_dir.join(format!("{{name}}.sh"));

    let (program, args): (String, Vec<String>) = if binary.exists() {{
        (binary.to_string_lossy().into(), vec![])
    }} else if shell.exists() {{
        ("bash".into(), vec![shell.to_string_lossy().into()])
    }} else {{
        return Err(format!("No executable found for skill '{{name}}'"));
    }};

    let mut child = Command::new(&program)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("SKILL_NAME", name)
        .env("SKILL_PATH", &skill_path)
        .env("SKILL_PARAMS", input.to_string())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{{name}}': {{e}}"))?;

    if let Some(mut stdin) = child.stdin.take() {{
        use std::io::Write;
        stdin.write_all(input.to_string().as_bytes()).ok();
    }}

    let out = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for '{{name}}': {{e}}"))?;
    let elapsed = start.elapsed().as_millis() as u64;

    if !out.status.success() {{
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("Skill '{{name}}' exit {{}}: {{stderr}}", out.status.code().unwrap_or(-1)));
    }}

    let stdout = String::from_utf8_lossy(&out.stdout);
    let result: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| json!({{"raw_output": stdout.to_string()}}));
    Ok((result, elapsed))
}}

fn deep_merge(base: &mut HashMap<String, Value>, key: &str, value: Value) {{
    if let Some(existing) = base.get_mut(key) {{
        if let (Some(a), Some(b)) = (existing.as_object_mut(), value.as_object()) {{
            for (k, v) in b {{ a.insert(k.clone(), v.clone()); }}
            return;
        }}
    }}
    base.insert(key.into(), value);
}}

fn overwrite_merge(base: &mut HashMap<String, Value>, key: &str, value: Value) {{
    base.insert(key.into(), value);
}}
"##
    )
}

fn gen_sequential(merge_fn: &str) -> String {
    format!(
        r#"    for skill in &skills {{
        match execute_skill(&skill.name, &json!(accumulator), skill.timeout_secs) {{
            Ok((result, elapsed)) => {{
                {merge_fn}(&mut accumulator, &skill.name, result);
                steps_completed += 1;
                step_details.push(StepDetail {{ skill: skill.name.clone(), status: "completed".into(), duration_ms: elapsed, error: None }});
            }}
            Err(e) => {{
                step_details.push(StepDetail {{ skill: skill.name.clone(), status: "failed".into(), duration_ms: 0, error: Some(e.clone()) }});
                if skill.required {{ eprintln!("Required skill '{{}}' failed: {{e}}", skill.name); failed = true; break; }}
                eprintln!("Optional skill '{{}}' failed: {{e}}", skill.name);
            }}
        }}
    }}"#
    )
}

fn gen_parallel(merge_fn: &str) -> String {
    format!(
        r#"    let handles: Vec<_> = skills.iter().map(|skill| {{
        let nm = skill.name.clone(); let req = skill.required; let t = skill.timeout_secs;
        let inp = json!(accumulator);
        std::thread::spawn(move || (nm.clone(), req, execute_skill(&nm, &inp, t)))
    }}).collect();
    for handle in handles {{
        match handle.join() {{
            Ok((nm, req, Ok((result, elapsed)))) => {{
                {merge_fn}(&mut accumulator, &nm, result);
                steps_completed += 1;
                step_details.push(StepDetail {{ skill: nm, status: "completed".into(), duration_ms: elapsed, error: None }});
            }}
            Ok((nm, req, Err(e))) => {{
                step_details.push(StepDetail {{ skill: nm.clone(), status: "failed".into(), duration_ms: 0, error: Some(e.clone()) }});
                if req {{ eprintln!("Required skill '{{nm}}' failed: {{e}}"); failed = true; }}
            }}
            Err(_) => {{ eprintln!("Thread panicked"); failed = true; }}
        }}
    }}"#
    )
}

fn gen_feedback(merge_fn: &str, max_iter: u32, field: &str, threshold: f64) -> String {
    format!(
        r#"    for _iteration in 0..{max_iter}u32 {{
        for skill in &skills {{
            match execute_skill(&skill.name, &json!(accumulator), skill.timeout_secs) {{
                Ok((result, elapsed)) => {{
                    {merge_fn}(&mut accumulator, &skill.name, result);
                    steps_completed += 1;
                    step_details.push(StepDetail {{ skill: skill.name.clone(), status: "completed".into(), duration_ms: elapsed, error: None }});
                }}
                Err(e) => {{
                    step_details.push(StepDetail {{ skill: skill.name.clone(), status: "failed".into(), duration_ms: 0, error: Some(e.clone()) }});
                    if skill.required {{ eprintln!("Required '{{}}' failed: {{e}}", skill.name); failed = true; break; }}
                }}
            }}
        }}
        if failed {{ break; }}
        let converged = accumulator.values().any(|v| v.get("{field}").and_then(|f| f.as_f64()).is_some_and(|s| s >= {threshold}));
        if converged {{ accumulator.insert("_converged".into(), json!(true)); break; }}
    }}"#
    )
}

fn generate_cargo_toml(spec: &CompoundSpec) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde = {{ version = \"1.0\", features = [\"derive\"] }}\nserde_json = \"1.0\"\ndirs = \"6.0\"\n",
        spec.compound.name,
    )
}

fn generate_skill_md(spec: &CompoundSpec) -> String {
    let nested = spec
        .skills
        .iter()
        .map(|s| format!("  - {}", s.name))
        .collect::<Vec<_>>()
        .join("\n");
    let tags = if spec.compound.tags.is_empty() {
        "compound, generated".to_string()
    } else {
        let mut t = spec.compound.tags.clone();
        if !t.contains(&"compound".to_string()) {
            t.push("compound".into());
        }
        t.join(", ")
    };
    let table = spec
        .skills
        .iter()
        .map(|s| format!("| {} | {} | {}s |", s.name, s.required, s.timeout_seconds))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "---\nname: {name}\nintent: \"{desc}\"\ndomain: compound\ntags: [{tags}]\nnested-skills:\n{nested}\nmodel: opus\nstrategy: {strat}\n---\n\n# {name}\n\n{desc}\n\n## Sub-Skills\n\n| Skill | Required | Timeout |\n|-------|----------|--------|\n{table}\n",
        name = spec.compound.name,
        desc = spec.compound.description,
        tags = tags,
        nested = nested,
        strat = spec.compound.strategy,
        table = table,
    )
}
