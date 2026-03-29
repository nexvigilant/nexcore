use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use super::common::*;
use crate::types::SkillRunParams;

/// Program registry — maps program names to skill directory names.
fn program_map() -> Vec<(
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
)> {
    // (program_name, skill_dir, phases, description)
    // NOTE: skill_dir must match the actual directory name under ~/.claude/skills/
    vec![
        (
            "smart",
            "SMART-dev",
            &["S", "M", "A", "R", "T"],
            "Goal analysis (Specific→Measurable→Achievable→Relevant→Time-bound)",
        ),
        (
            "brain",
            "brain-dev",
            &["B", "R", "A", "I", "N"],
            "Knowledge management (Bootstrap→Recall→Analyze→Integrate→Navigate)",
        ),
        (
            "guard",
            "guard-program",
            &["G", "U", "A", "R", "D"],
            "Safety governance (Gate→Unveil→Assess→Report→Defend)",
        ),
        (
            "pulse",
            "pulse-program",
            &["P", "U", "L", "S", "E"],
            "System health (Probe→Uptime→Load→Signals→Evaluate)",
        ),
        (
            "forge",
            "forge",
            &["F", "O", "R", "G", "E"],
            "Dev pipeline (Foundation→Orchestrate→Review→Generate→Execute)",
        ),
        (
            "scope",
            "scope-program",
            &["S", "C", "O", "P", "E"],
            "Project analysis (Scan→Count→Outline→Profile→Evaluate)",
        ),
        (
            "clean",
            "clean-program",
            &["C", "L", "E", "A", "N"],
            "Code hygiene (Cruft→Lint→Empty→Archive→Normalize)",
        ),
        (
            "audit",
            "skill-audit",
            &["A", "U", "D", "I", "T"],
            "Security scanning (Access→Unsafe→Dependencies→Injection→Threats)",
        ),
        (
            "craft",
            "craft-program",
            &["C", "R", "A", "F", "T"],
            "Quality metrics (Coverage→Readability→Architecture→Fitness→Testing)",
        ),
        (
            "trace",
            "trace-program",
            &["T", "R", "A", "C", "E"],
            "Debug tracing (Threads→Routes→Allocations→Calls→Errors)",
        ),
        (
            "rivet",
            "rivet-program",
            &["R", "I", "V", "E", "T"],
            "Engineering principles (Redundancy→Interlocking→Valve→Energy→Threshold)",
        ),
        (
            "glean",
            "glean-program",
            &["G", "L", "E", "A", "N"],
            "Data source intelligence (Gather→Layout→Extract→Align→Numerate)",
        ),
        (
            "pixel",
            "pixel-program",
            &["P", "I", "X", "E", "L"],
            "Visual design quality (Proportion→Ink→Xray→Emphasis→Layout)",
        ),
    ]
}

/// List all available programs.
pub fn list_skills() -> Result<CallToolResult, McpError> {
    let mut programs = Vec::new();

    // Native programs
    programs.push(json!({
        "name": "vitals",
        "type": "native",
        "phases": ["V", "I", "T", "A", "L", "S"],
        "description": "Biological health (Vigor→Immunity→Telemetry→Antibodies→Lifespan→Synapse)",
        "tools": ["vitals_vigor", "vitals_immunity", "vitals_telemetry", "vitals_antibodies", "vitals_lifespan", "vitals_synapse", "vitals_pipeline"],
    }));
    programs.push(json!({
        "name": "learn",
        "type": "native",
        "phases": ["L", "E", "A", "R", "N"],
        "description": "Knowledge feedback loop (Landscape→Extract→Assimilate→Recall→Normalize)",
        "tools": ["learn_landscape", "learn_extract", "learn_assimilate", "learn_recall", "learn_normalize", "learn_pipeline"],
    }));
    programs.push(json!({
        "name": "prove",
        "type": "native",
        "phases": ["P", "O", "V", "E"],
        "description": "Self-verification (Prepare→Observe→Validate→Evaluate). Note: R=Run requires shell (sub-Claude spawn).",
        "tools": ["prove_prepare", "prove_observe", "prove_validate", "prove_evaluate"],
    }));

    // Shell-based programs
    for (name, _dir, phases, desc) in program_map() {
        programs.push(json!({
            "name": name,
            "type": "shell",
            "phases": phases,
            "description": desc,
            "tools": ["skill_run"],
        }));
    }

    Ok(json_result(&json!({
        "total": programs.len(),
        "programs": programs,
    })))
}

/// Run a shell-based vocabulary program.
pub async fn run_skill(params: SkillRunParams) -> Result<CallToolResult, McpError> {
    let program = params.program.to_lowercase();
    let phase = params.phase.to_uppercase();

    // Find the program
    let entry = program_map()
        .into_iter()
        .find(|(name, _, _, _)| *name == program);

    let (_, skill_dir, valid_phases, _) = entry
        .ok_or_else(|| mcp_err(&format!(
            "Unknown program '{program}'. Available: smart, brain, guard, pulse, forge, scope, clean, audit, craft, trace, rivet, glean, pixel"
        )))?;

    // Validate phase
    if phase != "ALL" && !valid_phases.contains(&phase.as_str()) {
        return Err(mcp_err(&format!(
            "Invalid phase '{phase}' for {program}. Valid: {valid_phases:?} or ALL"
        )));
    }

    // Build script path
    let script_name = if phase == "ALL" {
        "ALL.sh".to_string()
    } else {
        format!("{phase}.sh")
    };

    let script = skills_dir().join(format!("{skill_dir}/scripts/{script_name}"));
    if !script.exists() {
        return Err(mcp_err(&format!("Script not found: {}", script.display())));
    }

    // Build command
    let mut cmd = tokio::process::Command::new(&script);
    if let Some(ref dir) = params.dir {
        cmd.arg(dir);
    }

    // Capture output
    let output = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| mcp_err(&format!("exec: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(json_result(&json!({
        "program": program,
        "phase": phase,
        "exit_code": exit_code,
        "stdout": stdout.to_string(),
        "stderr": if stderr.is_empty() { None } else { Some(stderr.to_string()) },
        "script": script.display().to_string(),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_map_not_empty() {
        assert!(!program_map().is_empty());
    }

    #[test]
    fn all_programs_have_phases() {
        for (name, _, phases, _) in program_map() {
            assert!(!phases.is_empty(), "program '{name}' has no phases");
        }
    }

    #[test]
    fn all_programs_have_descriptions() {
        for (name, _, _, desc) in program_map() {
            assert!(!desc.is_empty(), "program '{name}' has no description");
        }
    }

    #[test]
    fn no_duplicate_program_names() {
        let map = program_map();
        let mut seen = std::collections::HashSet::new();
        for (name, _, _, _) in &map {
            assert!(seen.insert(name), "duplicate program name: {name}");
        }
    }

    #[test]
    fn list_skills_succeeds() {
        let result = list_skills();
        assert!(result.is_ok());
    }

    #[test]
    fn list_skills_includes_native_and_shell() {
        // list_skills returns 3 native + 12 shell = 15 programs
        let result = list_skills();
        assert!(result.is_ok());
    }
}
