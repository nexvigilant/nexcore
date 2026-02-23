//! Walk `lib.rs` mod declarations to build module hierarchy.

use std::path::Path;

use crate::error::ForgeError;
use crate::ir::ModuleInfo;

/// Discover all Rust source files in a crate by walking the directory.
///
/// Returns `ModuleInfo` entries for each `.rs` file found.
pub fn discover_modules(crate_src: &Path) -> Result<Vec<ModuleInfo>, ForgeError> {
    let mut modules = Vec::new();

    if !crate_src.exists() {
        return Err(ForgeError::CrateNotFound(crate_src.to_path_buf()));
    }

    for entry in nexcore_fs::walk::WalkDir::new(crate_src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs") && e.file_type().is_file())
    {
        let path = entry.path();

        // Build module path from file path relative to src/
        let relative = path.strip_prefix(crate_src).unwrap_or(path);

        let module_path = relative
            .with_extension("")
            .to_string_lossy()
            .replace('/', "::")
            .replace("::mod", "");

        // Skip lib.rs as a module name
        let module_path = if module_path == "lib" {
            "crate".to_string()
        } else {
            module_path
        };

        let content = std::fs::read_to_string(path).map_err(|e| ForgeError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let line_count = content.lines().count();

        let file_path = relative.to_string_lossy().to_string();

        modules.push(ModuleInfo {
            path: module_path,
            doc_comment: None,        // Filled by syn_parser
            public_items: Vec::new(), // Filled by syn_parser
            file_path,
            line_count,
        });
    }

    modules.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(modules)
}
