//! CEP Pipeline Activator (UserPromptSubmit)
//!
//! Detects CEP-related commands in user prompts and injects pipeline context.
//! Triggers: /cep, /cep-pipeline, /knowledge-discovery, /translate, /domain-translate
//!
//! Patent: NV-2026-001

use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};

const CEP_TRIGGERS: &[&str] = &[
    "/cep",
    "/cep-pipeline",
    "/knowledge-discovery",
    "/domain-translate",
    "/cross-domain",
    "run cep on",
    "extract knowledge from",
    "translate concept",
    "cross-domain translate",
    "primitive extraction",
];

const CEP_CONTEXT: &str = r#"<cep-pipeline-context>
CEP (Cognitive Evolution Pipeline) detected. 8-stage methodology:

| Stage | Name | Purpose |
|-------|------|---------|
| 1 | SEE | Observe phenomena without prejudice |
| 2 | SPEAK | Articulate into structured vocabulary |
| 3 | DECOMPOSE | Extract T1/T2/T3 primitives via DAG |
| 4 | COMPOSE | Synthesize structures from primitives |
| 5 | TRANSLATE | Bidirectional cross-domain mapping |
| 6 | VALIDATE | Verify coverage ≥0.95, minimality ≥0.90, independence ≥0.90 |
| 7 | DEPLOY | Generate operational artifacts |
| 8 | IMPROVE | Aggregate feedback → new SEE cycle |

Validation Thresholds (Patent NV-2026-002):
- Coverage: ≥ 0.95 (concepts expressible from primitives)
- Minimality: ≥ 0.90 (absence of redundant primitives)
- Independence: ≥ 0.90 (absence of implied relationships)

Tier Classification:
- T1 Universal: ≥10 domains (confidence = 1.0)
- T2 Cross-Domain: 2-9 domains (confidence 0.85-0.95)
- T3 Domain-Specific: 1 domain (confidence 0.80-1.0)

MCP Tools: cep_execute_stage, cep_pipeline_stages, cep_validate_extraction, cep_classify_primitive
Skills: /cep-pipeline, /domain-translator, /primitive-extractor

🤖 **SUBAGENT RECOMMENDATION**: For full 8-stage pipeline, use Task tool with:
   subagent_type: "cep-orchestrator"
   For translation only: subagent_type: "domain-bridger"
</cep-pipeline-context>"#;

const DOMAIN_TRIGGERS: &[&str] = &[
    "pharmacovigilance",
    "pv domain",
    "finance domain",
    "aviation",
    "cybersecurity",
    "what are the primitives",
    "building blocks of",
    "decompose this domain",
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let prompt = input.get_prompt().unwrap_or("").to_lowercase();

    // Check if any CEP trigger is present
    let is_cep_request = CEP_TRIGGERS.iter().any(|trigger| prompt.contains(trigger));

    // Check for domain analysis requests (Accelerator 3)
    let is_domain_request = DOMAIN_TRIGGERS
        .iter()
        .any(|trigger| prompt.contains(trigger));

    if is_cep_request || is_domain_request {
        exit_with_context(CEP_CONTEXT);
    }

    exit_skip_prompt();
}
