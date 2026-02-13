//! Pro-Drop Inference (∅ Contextual Elision)
//!
//! Latin drops subject pronouns because the verb already encodes
//! person and number. We drop parameters that are inferrable from context.
//!
//! If you're in `~/nexcore`, the `path` parameter defaults to `~/nexcore`.
//! If the last tool was `caesura_scan`, `caesura` without mode = `report`.
//!
//! ## Primitive Grounding
//! ∅ Void (dominant) + μ Mapping + ς State = T2-C

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tier: T2-C — ∅ Void + μ Mapping + ς State
///
/// Context state for pro-drop inference. Tracks what can be elided
/// because the context already encodes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProDropContext {
    /// Current working directory (inferred `path` parameter).
    pub cwd: Option<String>,
    /// Last tool invoked (inferred mode/family for next call).
    pub last_tool: Option<String>,
    /// Last tool family stem (for inflection inference).
    pub last_stem: Option<String>,
    /// Accumulated parameter defaults from session context.
    pub defaults: HashMap<String, String>,
}

impl Default for ProDropContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ProDropContext {
    /// Create empty context.
    pub fn new() -> Self {
        Self {
            cwd: None,
            last_tool: None,
            last_stem: None,
            defaults: HashMap::new(),
        }
    }

    /// Create context with CWD.
    pub fn with_cwd(cwd: String) -> Self {
        let mut ctx = Self::new();
        ctx.cwd = Some(cwd);
        ctx
    }

    /// Record a tool invocation, updating context for future inference.
    pub fn record_invocation(&mut self, tool_name: &str, params: &HashMap<String, String>) {
        self.last_tool = Some(tool_name.to_string());

        // Extract stem (everything before first `_`)
        if let Some(stem) = tool_name.split('_').next() {
            self.last_stem = Some(stem.to_string());
        }

        // Absorb params as defaults for the family
        for (k, v) in params {
            self.defaults.insert(k.clone(), v.clone());
        }
    }

    /// Resolve a parameter: return the provided value, or infer from context.
    ///
    /// Pro-drop rule: if the parameter is elided (None), check context.
    pub fn resolve(&self, param_name: &str, provided: Option<&str>) -> Option<String> {
        // If explicitly provided, use it (no pro-drop needed)
        if let Some(val) = provided {
            return Some(val.to_string());
        }

        // Inference rules (ordered by specificity):
        match param_name {
            // Path parameters default to CWD
            "path" | "dir" | "directory" => self.cwd.clone(),

            // File path defaults to last used file
            "file_path" | "file" => self
                .defaults
                .get("file_path")
                .or_else(|| self.defaults.get("file"))
                .cloned(),

            // Sensitivity defaults to last used value
            "sensitivity" | "sigma" => self
                .defaults
                .get("sensitivity")
                .or_else(|| self.defaults.get("sigma"))
                .cloned(),

            // Crate name defaults to last used
            "crate_name" | "crate" => self
                .defaults
                .get("crate_name")
                .or_else(|| self.defaults.get("crate"))
                .cloned(),

            // Generic: check defaults map
            _ => self.defaults.get(param_name).cloned(),
        }
    }

    /// Count how many parameters can be pro-dropped given current context.
    pub fn droppable_params(&self, param_names: &[&str]) -> usize {
        param_names
            .iter()
            .filter(|&&name| self.resolve(name, None).is_some())
            .count()
    }

    /// Compute elision ratio: droppable / total.
    pub fn elision_ratio(&self, param_names: &[&str]) -> f64 {
        if param_names.is_empty() {
            return 0.0;
        }
        self.droppable_params(param_names) as f64 / param_names.len() as f64
    }
}

/// Tier: T2-P — ∅ + N
///
/// Analysis of pro-drop potential for a tool invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProDropAnalysis {
    /// Tool name.
    pub tool: String,
    /// Total parameters.
    pub total_params: usize,
    /// Parameters that can be elided.
    pub droppable: usize,
    /// Elision ratio (0.0–1.0).
    pub elision_ratio: f64,
    /// Which params can be dropped and what they'd resolve to.
    pub resolutions: HashMap<String, String>,
}

/// Analyze pro-drop potential for a tool given current context.
pub fn analyze_prodrop(
    tool_name: &str,
    param_names: &[&str],
    context: &ProDropContext,
) -> ProDropAnalysis {
    let mut resolutions = HashMap::new();

    for &param in param_names {
        if let Some(resolved) = context.resolve(param, None) {
            resolutions.insert(param.to_string(), resolved);
        }
    }

    let droppable = resolutions.len();
    let total = param_names.len();

    ProDropAnalysis {
        tool: tool_name.to_string(),
        total_params: total,
        droppable,
        elision_ratio: if total > 0 {
            droppable as f64 / total as f64
        } else {
            0.0
        },
        resolutions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_explicit() {
        let ctx = ProDropContext::new();
        let result = ctx.resolve("path", Some("/home/matthew"));
        assert_eq!(result, Some("/home/matthew".to_string()));
    }

    #[test]
    fn test_resolve_cwd_inference() {
        let ctx = ProDropContext::with_cwd("/home/matthew/nexcore".to_string());
        let result = ctx.resolve("path", None);
        assert_eq!(result, Some("/home/matthew/nexcore".to_string()));
    }

    #[test]
    fn test_resolve_no_context() {
        let ctx = ProDropContext::new();
        let result = ctx.resolve("path", None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_record_invocation() {
        let mut ctx = ProDropContext::new();
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/home/matthew/nexcore/src".to_string());
        params.insert("sensitivity".to_string(), "1.5".to_string());

        ctx.record_invocation("caesura_scan", &params);

        assert_eq!(ctx.last_tool, Some("caesura_scan".to_string()));
        assert_eq!(ctx.last_stem, Some("caesura".to_string()));
        assert_eq!(ctx.resolve("sensitivity", None), Some("1.5".to_string()));
    }

    #[test]
    fn test_droppable_count() {
        let ctx = ProDropContext::with_cwd("/home/matthew/nexcore".to_string());
        let params = ["path", "sensitivity", "strata"];
        assert_eq!(ctx.droppable_params(&params), 1); // only path
    }

    #[test]
    fn test_elision_ratio() {
        let mut ctx = ProDropContext::with_cwd("/home/matthew/nexcore".to_string());
        let mut params = HashMap::new();
        params.insert("sensitivity".to_string(), "2.0".to_string());
        ctx.record_invocation("caesura_scan", &params);

        let param_names = ["path", "sensitivity", "strata"];
        let ratio = ctx.elision_ratio(&param_names);
        // path (from cwd) + sensitivity (from last invocation) = 2/3
        assert!((ratio - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analyze_prodrop() {
        let ctx = ProDropContext::with_cwd("/home/matthew/nexcore".to_string());
        let analysis = analyze_prodrop("caesura_scan", &["path", "strata"], &ctx);
        assert_eq!(analysis.droppable, 1);
        assert_eq!(analysis.total_params, 2);
        assert!(analysis.resolutions.contains_key("path"));
    }

    #[test]
    fn test_empty_params() {
        let ctx = ProDropContext::new();
        let analysis = analyze_prodrop("tool", &[], &ctx);
        assert_eq!(analysis.droppable, 0);
        assert!((analysis.elision_ratio - 0.0).abs() < f64::EPSILON);
    }
}
