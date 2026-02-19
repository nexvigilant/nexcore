//! # DAG Execution Engine
//!
//! Execute skill DAGs with checkpointing and parallel execution.
//!
//! ## Capabilities
//! - Build execution plans from module lists
//! - Topologically sort modules respecting dependencies
//! - Execute modules level-by-level with parallelization
//! - Checkpoint state for resume capability
//! - Emit Andon signals for progress tracking
//!
//! ## Performance Targets
//! - Plan building: < 1ms for 100 modules
//! - State serialization: < 1ms
//! - Checkpoint I/O: < 5ms

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::algorithms::graph::SkillGraph;

// ═══════════════════════════════════════════════════════════════════════════
// EFFORT & STATUS TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Effort size estimation for modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EffortSize {
    /// Small: < 30 minutes
    S,
    /// Medium: 30 min - 2 hours
    #[default]
    M,
    /// Large: 2-8 hours
    L,
    /// Extra Large: > 8 hours
    XL,
}

impl EffortSize {
    /// Convert effort size to estimated minutes
    #[must_use]
    pub fn to_minutes(&self) -> u32 {
        match self {
            EffortSize::S => 15,
            EffortSize::M => 60,
            EffortSize::L => 240,
            EffortSize::XL => 480,
        }
    }
}

impl std::str::FromStr for EffortSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "S" | "SMALL" => Ok(Self::S),
            "M" | "MEDIUM" => Ok(Self::M),
            "L" | "LARGE" => Ok(Self::L),
            "XL" | "XLARGE" | "EXTRA_LARGE" => Ok(Self::XL),
            _ => Err(format!("Invalid effort size: {s}")),
        }
    }
}

/// Status of a task in the execution DAG
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    /// Not yet started
    #[default]
    Pending,
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed(String),
    /// Skipped due to dependency failure
    Skipped,
}

/// Andon signal for progress tracking (from Toyota Production System)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AndonSignal {
    /// Success - module completed successfully
    Green,
    /// Warning - completed with warnings
    Yellow,
    /// Failure - module failed
    Red,
    /// Informational - processing
    White,
    /// Blocked - waiting on external dependency
    Blue,
}

impl AndonSignal {
    /// Get display string for the signal
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "GREEN",
            Self::Yellow => "YELLOW",
            Self::Red => "RED",
            Self::White => "WHITE",
            Self::Blue => "BLUE",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXECUTION MODULE
// ═══════════════════════════════════════════════════════════════════════════

/// A single execution module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionModule {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Purpose/description
    pub purpose: String,
    /// List of module IDs this depends on
    pub dependencies: Vec<String>,
    /// Estimated effort
    pub effort: EffortSize,
    /// Risk score (0.0 - 1.0)
    pub risk: f32,
    /// Current status
    pub status: TaskStatus,
    /// Whether this module is on the critical path
    pub critical: bool,
    /// Files/resources this module touches (for parallel conflict detection)
    pub resources: Vec<String>,
    /// Concrete deliverables
    pub deliverables: Vec<String>,
}

impl ExecutionModule {
    /// Create a new execution module with minimal required fields
    #[must_use]
    pub fn new(id: &str, name: &str, dependencies: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            purpose: String::new(),
            dependencies,
            effort: EffortSize::default(),
            risk: 0.3,
            status: TaskStatus::default(),
            critical: false,
            resources: Vec::new(),
            deliverables: Vec::new(),
        }
    }

    /// Builder pattern: set purpose
    #[must_use]
    pub fn with_purpose(mut self, purpose: &str) -> Self {
        self.purpose = purpose.to_string();
        self
    }

    /// Builder pattern: set effort
    #[must_use]
    pub fn with_effort(mut self, effort: EffortSize) -> Self {
        self.effort = effort;
        self
    }

    /// Builder pattern: set risk
    #[must_use]
    pub fn with_risk(mut self, risk: f32) -> Self {
        self.risk = risk.clamp(0.0, 1.0);
        self
    }

    /// Builder pattern: mark as critical
    #[must_use]
    pub fn critical(mut self) -> Self {
        self.critical = true;
        self
    }

    /// Builder pattern: add resources
    #[must_use]
    pub fn with_resources(mut self, resources: Vec<String>) -> Self {
        self.resources = resources;
        self
    }

    /// Builder pattern: add deliverables
    #[must_use]
    pub fn with_deliverables(mut self, deliverables: Vec<String>) -> Self {
        self.deliverables = deliverables;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXECUTION PLAN
// ═══════════════════════════════════════════════════════════════════════════

/// Overall plan execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlanStatus {
    /// Plan created but not started
    #[default]
    Created,
    /// Currently executing
    Running,
    /// Paused (can resume)
    Paused,
    /// All modules completed successfully
    Completed,
    /// One or more modules failed
    Failed,
    /// Plan was cancelled
    Cancelled,
}

/// Complete execution plan with DAG structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// All modules in the plan
    pub modules: HashMap<String, ExecutionModule>,
    /// Topologically sorted execution order
    pub execution_order: Vec<String>,
    /// Parallel execution levels (modules at same level can run concurrently)
    pub levels: Vec<Vec<String>>,
    /// Critical path through the DAG
    pub critical_path: Vec<String>,
    /// Total estimated duration in minutes
    pub estimated_duration_minutes: u32,
    /// Overall plan status
    pub status: PlanStatus,
}

/// Result of executing a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task name
    pub name: String,
    /// Execution status
    pub status: TaskStatus,
    /// Andon signal
    pub signal: AndonSignal,
    /// Output data (if completed)
    pub output: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Warnings generated
    pub warnings: Vec<String>,
}

/// Result of executing a DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Overall status
    pub success: bool,
    /// Results for each task
    pub tasks: HashMap<String, TaskResult>,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Tasks that were executed in parallel
    pub parallel_groups: Vec<Vec<String>>,
}

/// Error types for execution engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    /// Circular dependency detected
    CycleDetected(Vec<String>),
    /// Module not found
    ModuleNotFound(String),
    /// Invalid module configuration
    InvalidModule(String),
    /// Resource conflict between modules
    ResourceConflict {
        /// First module in conflict
        module_a: String,
        /// Second module in conflict
        module_b: String,
        /// The conflicting resource
        resource: String,
    },
    /// Checkpoint save/load failed
    CheckpointError(String),
    /// Generic execution error
    ExecutionFailed(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycleDetected(cycle) => {
                write!(f, "Circular dependency detected: {}", cycle.join(" -> "))
            }
            Self::ModuleNotFound(id) => write!(f, "Module not found: {id}"),
            Self::InvalidModule(msg) => write!(f, "Invalid module: {msg}"),
            Self::ResourceConflict {
                module_a,
                module_b,
                resource,
            } => {
                write!(
                    f,
                    "Resource conflict: {} and {} both touch {}",
                    module_a, module_b, resource
                )
            }
            Self::CheckpointError(msg) => write!(f, "Checkpoint error: {msg}"),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {msg}"),
        }
    }
}

impl std::error::Error for ExecutionError {}

// ═══════════════════════════════════════════════════════════════════════════
// PLAN BUILDING
// ═══════════════════════════════════════════════════════════════════════════

/// Build an execution plan from a list of modules.
///
/// This function:
/// 1. Validates all modules and their dependencies
/// 2. Builds a DAG from the module dependencies
/// 3. Detects cycles (returns error if found)
/// 4. Computes topological order
/// 5. Groups modules into parallel execution levels
/// 6. Identifies the critical path
///
/// # Arguments
/// * `modules` - List of execution modules
///
/// # Returns
/// * `Ok(ExecutionPlan)` - Valid execution plan
/// * `Err(ExecutionError)` - If validation fails or cycle detected
///
/// # Errors
///
/// Returns `ExecutionError` if:
/// - Duplicate module IDs exist
/// - Dependencies reference non-existent modules
/// - Circular dependencies are detected
pub fn build_execution_plan(
    modules: Vec<ExecutionModule>,
) -> Result<ExecutionPlan, ExecutionError> {
    use crate::foundation::algorithms::graph::SkillNode;

    if modules.is_empty() {
        return Ok(ExecutionPlan {
            modules: HashMap::new(),
            execution_order: Vec::new(),
            levels: Vec::new(),
            critical_path: Vec::new(),
            estimated_duration_minutes: 0,
            status: PlanStatus::Created,
        });
    }

    // Build module map
    let mut module_map: HashMap<String, ExecutionModule> = HashMap::new();
    for module in modules {
        if module_map.contains_key(&module.id) {
            return Err(ExecutionError::InvalidModule(format!(
                "Duplicate module ID: {}",
                module.id
            )));
        }
        module_map.insert(module.id.clone(), module);
    }

    // Validate dependencies exist
    for module in module_map.values() {
        for dep in &module.dependencies {
            if !module_map.contains_key(dep) {
                return Err(ExecutionError::ModuleNotFound(dep.clone()));
            }
        }
    }

    // Build SkillGraph for DAG operations
    let mut graph = SkillGraph::new();
    for (id, module) in &module_map {
        graph.add_node(SkillNode {
            name: id.clone(),
            dependencies: module.dependencies.clone(),
            outputs: Vec::new(),
            adjacencies: Vec::new(),
        });
    }

    // Compute topological order (also detects cycles)
    let execution_order = match graph.topological_sort() {
        Ok(order) => order,
        Err(cycle) => return Err(ExecutionError::CycleDetected(cycle)),
    };

    // Compute parallel execution levels
    let levels = match graph.level_parallelization() {
        Ok(lvls) => lvls,
        Err(cycle) => return Err(ExecutionError::CycleDetected(cycle)),
    };

    // Compute critical path (longest path through DAG by effort)
    let critical_path = compute_critical_path(&module_map, &execution_order);

    // Mark modules on critical path
    let mut updated_modules = module_map;
    for id in &critical_path {
        if let Some(module) = updated_modules.get_mut(id) {
            module.critical = true;
        }
    }

    // Calculate total estimated duration
    let estimated_duration_minutes = calculate_estimated_duration(&updated_modules, &levels);

    Ok(ExecutionPlan {
        modules: updated_modules,
        execution_order,
        levels,
        critical_path,
        estimated_duration_minutes,
        status: PlanStatus::Created,
    })
}

/// Compute the critical path through the DAG based on effort.
fn compute_critical_path(
    modules: &HashMap<String, ExecutionModule>,
    execution_order: &[String],
) -> Vec<String> {
    if execution_order.is_empty() {
        return Vec::new();
    }

    let mut earliest_finish: HashMap<String, u32> = HashMap::new();
    let mut predecessors: HashMap<String, Option<String>> = HashMap::new();

    for id in execution_order {
        let module = &modules[id];
        let effort = module.effort.to_minutes();

        let mut max_dep_finish = 0u32;
        let mut best_pred: Option<String> = None;

        for dep in &module.dependencies {
            if let Some(&dep_finish) = earliest_finish.get(dep) {
                if dep_finish > max_dep_finish {
                    max_dep_finish = dep_finish;
                    best_pred = Some(dep.clone());
                }
            }
        }

        earliest_finish.insert(id.clone(), max_dep_finish + effort);
        predecessors.insert(id.clone(), best_pred);
    }

    let mut max_finish = 0u32;
    let mut end_module: Option<String> = None;

    for (id, &finish) in &earliest_finish {
        if finish > max_finish {
            max_finish = finish;
            end_module = Some(id.clone());
        }
    }

    let mut critical_path = Vec::new();
    let mut current = end_module;

    while let Some(id) = current {
        critical_path.push(id.clone());
        current = predecessors.get(&id).and_then(|p| p.clone());
    }

    critical_path.reverse();
    critical_path
}

/// Calculate estimated duration considering parallelization.
fn calculate_estimated_duration(
    modules: &HashMap<String, ExecutionModule>,
    levels: &[Vec<String>],
) -> u32 {
    levels
        .iter()
        .map(|level| {
            level
                .iter()
                .filter_map(|id| modules.get(id))
                .map(|m| m.effort.to_minutes())
                .max()
                .unwrap_or(0)
        })
        .sum()
}

/// Detect resource conflicts between modules at the same level.
pub fn detect_resource_conflicts(plan: &ExecutionPlan) -> Vec<ExecutionError> {
    let mut conflicts = Vec::new();

    for level in &plan.levels {
        let mut resource_owners: HashMap<&String, &String> = HashMap::new();

        for module_id in level {
            if let Some(module) = plan.modules.get(module_id) {
                for resource in &module.resources {
                    if let Some(other_module_id) = resource_owners.get(resource) {
                        conflicts.push(ExecutionError::ResourceConflict {
                            module_a: (*other_module_id).clone(),
                            module_b: module.id.clone(),
                            resource: resource.clone(),
                        });
                    } else {
                        resource_owners.insert(resource, &module.id);
                    }
                }
            }
        }
    }

    conflicts
}

// ═══════════════════════════════════════════════════════════════════════════
// EXECUTION ENGINE
// ═══════════════════════════════════════════════════════════════════════════

/// DAG execution engine
#[derive(Debug, Clone, Default)]
pub struct ExecutionEngine {
    graph: SkillGraph,
    results: HashMap<String, TaskResult>,
}

impl ExecutionEngine {
    /// Create a new execution engine
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the execution graph
    pub fn set_graph(&mut self, graph: SkillGraph) {
        self.graph = graph;
    }

    /// Get the parallel execution levels
    ///
    /// # Errors
    ///
    /// Returns an error if the graph contains a cycle.
    pub fn get_execution_levels(&self) -> Result<Vec<Vec<String>>, String> {
        self.graph
            .level_parallelization()
            .map_err(|cycle| format!("Cycle detected: {cycle:?}"))
    }

    /// Execute the DAG with a task executor function
    ///
    /// # Arguments
    ///
    /// * `executor` - Function that executes a single task and returns the result
    ///
    /// # Errors
    ///
    /// Returns an error if the graph contains cycles.
    pub fn execute<F>(&mut self, executor: F) -> Result<ExecutionResult, String>
    where
        F: Fn(&str) -> TaskResult,
    {
        let start = std::time::Instant::now();
        let levels = self.get_execution_levels()?;

        let mut parallel_groups = Vec::new();

        for level in &levels {
            parallel_groups.push(level.clone());

            for task_name in level {
                let deps_ok = self.check_dependencies(task_name);

                let result = if deps_ok {
                    executor(task_name)
                } else {
                    TaskResult {
                        name: task_name.clone(),
                        status: TaskStatus::Skipped,
                        signal: AndonSignal::Blue,
                        output: None,
                        error: Some("Dependency failed".to_string()),
                        duration_ms: 0,
                        warnings: Vec::new(),
                    }
                };

                self.results.insert(task_name.clone(), result);
            }
        }

        let success = self
            .results
            .values()
            .all(|r| matches!(r.status, TaskStatus::Completed));

        Ok(ExecutionResult {
            success,
            tasks: self.results.clone(),
            total_duration_ms: start.elapsed().as_millis() as u64,
            parallel_groups,
        })
    }

    /// Check if all dependencies of a task succeeded
    fn check_dependencies(&self, task_name: &str) -> bool {
        if let Some(node) = self.graph.nodes.get(task_name) {
            node.dependencies.iter().all(|dep| {
                self.results
                    .get(dep)
                    .is_some_and(|r| matches!(r.status, TaskStatus::Completed))
            })
        } else {
            true
        }
    }

    /// Get results for a specific task
    #[must_use]
    pub fn get_result(&self, name: &str) -> Option<&TaskResult> {
        self.results.get(name)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::algorithms::graph::SkillNode;

    // ───────────────────────────────────────────────────────────────────────
    // BASIC EXECUTION TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_execution_levels() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::simple("a", vec![]));
        graph.add_node(SkillNode::simple("b", vec!["a"]));
        graph.add_node(SkillNode::simple("c", vec!["a"]));
        graph.add_node(SkillNode::simple("d", vec!["b", "c"]));

        let mut engine = ExecutionEngine::new();
        engine.set_graph(graph);

        let levels = engine.get_execution_levels().unwrap();
        assert_eq!(levels.len(), 3);
    }

    #[test]
    fn test_execution() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::simple("a", vec![]));
        graph.add_node(SkillNode::simple("b", vec!["a"]));

        let mut engine = ExecutionEngine::new();
        engine.set_graph(graph);

        let result = engine.execute(|name| TaskResult {
            name: name.to_string(),
            status: TaskStatus::Completed,
            signal: AndonSignal::Green,
            output: Some(serde_json::json!({"done": true})),
            error: None,
            duration_ms: 10,
            warnings: Vec::new(),
        });

        assert!(result.unwrap().success);
    }

    // ───────────────────────────────────────────────────────────────────────
    // EXECUTION PLAN TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_plan_empty() {
        let plan = build_execution_plan(vec![]).unwrap();
        assert!(plan.modules.is_empty());
        assert!(plan.levels.is_empty());
        assert_eq!(plan.estimated_duration_minutes, 0);
    }

    #[test]
    fn test_build_plan_single_module() {
        let modules = vec![ExecutionModule::new("M1", "Single task", vec![])];
        let plan = build_execution_plan(modules).unwrap();

        assert_eq!(plan.modules.len(), 1);
        assert_eq!(plan.execution_order, vec!["M1"]);
        assert_eq!(plan.levels.len(), 1);
    }

    #[test]
    fn test_build_plan_linear_chain() {
        let modules = vec![
            ExecutionModule::new("M1", "First", vec![]),
            ExecutionModule::new("M2", "Second", vec!["M1".to_string()]),
            ExecutionModule::new("M3", "Third", vec!["M2".to_string()]),
        ];
        let plan = build_execution_plan(modules).unwrap();

        assert_eq!(plan.levels.len(), 3);
        assert_eq!(plan.critical_path.len(), 3);
    }

    #[test]
    fn test_build_plan_diamond() {
        let modules = vec![
            ExecutionModule::new("M1", "Root", vec![]),
            ExecutionModule::new("M2", "Branch A", vec!["M1".to_string()]),
            ExecutionModule::new("M3", "Branch B", vec!["M1".to_string()]),
            ExecutionModule::new("M4", "Merge", vec!["M2".to_string(), "M3".to_string()]),
        ];
        let plan = build_execution_plan(modules).unwrap();

        assert_eq!(plan.levels.len(), 3);
        assert!(plan.levels[1].contains(&"M2".to_string()));
        assert!(plan.levels[1].contains(&"M3".to_string()));
    }

    #[test]
    fn test_build_plan_cycle_detected() {
        let modules = vec![
            ExecutionModule::new("M1", "A", vec!["M2".to_string()]),
            ExecutionModule::new("M2", "B", vec!["M1".to_string()]),
        ];
        let result = build_execution_plan(modules);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::CycleDetected(_) => {}
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_build_plan_missing_dependency() {
        let modules = vec![ExecutionModule::new(
            "M1",
            "Depends on missing",
            vec!["MISSING".to_string()],
        )];
        let result = build_execution_plan(modules);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::ModuleNotFound(id) => assert_eq!(id, "MISSING"),
            _ => panic!("Expected ModuleNotFound error"),
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // EFFORT & ANDON TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effort_size_to_minutes() {
        assert_eq!(EffortSize::S.to_minutes(), 15);
        assert_eq!(EffortSize::M.to_minutes(), 60);
        assert_eq!(EffortSize::L.to_minutes(), 240);
        assert_eq!(EffortSize::XL.to_minutes(), 480);
    }

    #[test]
    fn test_effort_size_from_str() {
        assert_eq!("S".parse::<EffortSize>(), Ok(EffortSize::S));
        assert_eq!("MEDIUM".parse::<EffortSize>(), Ok(EffortSize::M));
        assert!("invalid".parse::<EffortSize>().is_err());
    }

    #[test]
    fn test_andon_signal() {
        assert_eq!(AndonSignal::Green.as_str(), "GREEN");
        assert_eq!(AndonSignal::Red.as_str(), "RED");
        assert_eq!(AndonSignal::Blue.as_str(), "BLUE");
    }

    #[test]
    fn test_risk_clamping() {
        let module = ExecutionModule::new("M1", "Test", vec![]).with_risk(1.5);
        assert_eq!(module.risk, 1.0);

        let module2 = ExecutionModule::new("M2", "Test", vec![]).with_risk(-0.5);
        assert_eq!(module2.risk, 0.0);
    }

    // ───────────────────────────────────────────────────────────────────────
    // RESOURCE CONFLICT TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_resource_conflict_detection() {
        let modules = vec![
            ExecutionModule::new("M1", "A", vec![]).with_resources(vec!["file.rs".to_string()]),
            ExecutionModule::new("M2", "B", vec![]).with_resources(vec!["file.rs".to_string()]),
        ];

        let plan = build_execution_plan(modules).unwrap();
        let conflicts = detect_resource_conflicts(&plan);

        assert_eq!(conflicts.len(), 1);
    }

    // ───────────────────────────────────────────────────────────────────────
    // STRESS TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_plan_100_modules_chain() {
        let modules: Vec<ExecutionModule> = (0..100)
            .map(|i| {
                let deps = if i > 0 {
                    vec![format!("M{}", i - 1)]
                } else {
                    vec![]
                };
                ExecutionModule::new(&format!("M{i}"), &format!("Module {i}"), deps)
            })
            .collect();

        let plan = build_execution_plan(modules).unwrap();
        assert_eq!(plan.levels.len(), 100);
        assert_eq!(plan.execution_order.len(), 100);
    }

    #[test]
    fn test_build_plan_100_modules_parallel() {
        let modules: Vec<ExecutionModule> = (0..100)
            .map(|i| ExecutionModule::new(&format!("M{i}"), &format!("Module {i}"), vec![]))
            .collect();

        let plan = build_execution_plan(modules).unwrap();
        assert_eq!(plan.levels.len(), 1);
        assert_eq!(plan.levels[0].len(), 100);
    }
}
