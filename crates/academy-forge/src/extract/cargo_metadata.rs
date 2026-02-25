//! Parse `Cargo.toml` for crate metadata.

use std::path::Path;

use crate::error::ForgeError;

/// Metadata extracted from Cargo.toml.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CargoMetadata {
    /// Crate name.
    pub name: String,
    /// Crate version.
    pub version: String,
    /// Crate description.
    pub description: String,
    /// Workspace dependencies referenced.
    pub dependencies: Vec<String>,
}

/// Parse a Cargo.toml file into metadata.
pub fn parse_cargo_toml(crate_path: &Path) -> Result<CargoMetadata, ForgeError> {
    let cargo_path = crate_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path).map_err(|e| ForgeError::IoError {
        path: cargo_path.clone(),
        source: e,
    })?;

    let doc: toml::Value = toml::from_str(&content).map_err(|e| ForgeError::CargoTomlError {
        path: cargo_path,
        message: e.to_string(),
    })?;

    let package = doc.get("package");

    let name = package
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let version = package
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();

    let description = package
        .and_then(|p| p.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Extract dependency names (both [dependencies] and workspace = true refs)
    let mut dependencies = Vec::new();
    if let Some(deps) = doc.get("dependencies") {
        if let Some(table) = deps.as_table() {
            for key in table.keys() {
                // Only include nexcore-* workspace deps
                if key.starts_with("nexcore-") || key.starts_with("stem") {
                    dependencies.push(key.clone());
                }
            }
        }
    }

    Ok(CargoMetadata {
        name,
        version,
        description,
        dependencies,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_toml() {
        // Use the academy-forge's own Cargo.toml as test input
        let crate_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let meta = parse_cargo_toml(crate_path).unwrap();
        assert_eq!(meta.name, "academy-forge");
        assert_eq!(meta.version, "1.0.0");
        assert!(!meta.description.is_empty());
    }
}
