//! Jupyter & Voila Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for Jupyter kernel management and Voila notebook rendering.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize};

/// Parameters for jupyter_kernels — list running Jupyter kernels
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct JupyterKernelsParams {
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for jupyter_kernelspecs — list available kernel specifications
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct JupyterKernelspecsParams {
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for jupyter_status — server health check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct JupyterStatusParams {
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for voila_render — render a notebook as a web app
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct VoilaRenderParams {
    /// Notebook path relative to Jupyter root (e.g. "analysis.ipynb" or "notebooks/report.ipynb")
    pub notebook: String,
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for voila_status — check if Voila extension is loaded
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct VoilaStatusParams {
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for voila_list — list renderable notebooks
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct VoilaListParams {
    /// Directory path to scan for notebooks (relative to Jupyter root). Defaults to root.
    #[serde(default)]
    pub path: Option<String>,
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

// ============================================================================
// Pipeline Tools
// ============================================================================

/// A single cell to include in a new notebook.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct NotebookCell {
    /// Cell type: "code" or "markdown"
    #[serde(rename = "type")]
    pub cell_type: String,
    /// Cell source content
    pub source: String,
}

/// Parameters for jupyter_notebook_create — create a notebook with cells in one call
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct JupyterNotebookCreateParams {
    /// Notebook path relative to Jupyter root (e.g. "reports/signal-analysis.ipynb")
    pub path: String,
    /// Ordered list of cells to include in the notebook
    pub cells: Vec<NotebookCell>,
    /// Kernel name. Defaults to "python3"
    #[serde(default)]
    pub kernel: Option<String>,
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for jupyter_notebook_execute — execute all cells in an existing notebook
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct JupyterNotebookExecuteParams {
    /// Notebook path relative to Jupyter root (e.g. "reports/signal-analysis.ipynb")
    pub path: String,
    /// Kernel name override. Defaults to the notebook's configured kernel.
    #[serde(default)]
    pub kernel: Option<String>,
    /// Per-cell execution timeout in seconds. Defaults to 120.
    #[serde(default)]
    pub timeout: Option<u64>,
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Parameters for jupyter_pipeline — full create→execute→render pipeline
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct JupyterPipelineParams {
    /// Notebook path relative to Jupyter root (e.g. "reports/signal-analysis.ipynb")
    pub path: String,
    /// Cells to create the notebook with. If omitted, the notebook must already exist.
    #[serde(default)]
    pub cells: Option<Vec<NotebookCell>>,
    /// Kernel name. Defaults to "python3"
    #[serde(default)]
    pub kernel: Option<String>,
    /// Whether to execute the notebook after creating/loading it. Defaults to true.
    #[serde(default = "default_true")]
    pub execute: bool,
    /// Whether to confirm Voila can render the notebook. Defaults to true.
    #[serde(default = "default_true")]
    pub render: bool,
    /// Per-cell execution timeout in seconds. Defaults to 120.
    #[serde(default)]
    pub timeout: Option<u64>,
    /// Jupyter server URL. Defaults to http://localhost:8888
    #[serde(default)]
    pub url: Option<String>,
    /// Authentication token. Auto-discovered from JUPYTER_TOKEN env or `jupyter server list`.
    #[serde(default)]
    pub token: Option<String>,
}

fn default_true() -> bool {
    true
}
