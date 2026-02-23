use nexcore_error::{Context, Result, bail};
use toml_edit::{Array, DocumentMut, Formatted, InlineTable, Item, Table, Value};

/// Trait for resolving the version of an internal crate by reading its own Cargo.toml.
/// This is abstracted so tests can provide a fake resolver.
pub trait InternalDepResolver {
    fn resolve_version(&self, crate_name: &str) -> Result<String>;
}

/// Resolves internal crate versions by reading Cargo.toml files from disk.
pub struct FileSystemResolver {
    workspace_root: std::path::PathBuf,
}

impl FileSystemResolver {
    pub fn new(workspace_root: std::path::PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Given a path like "crates/nexcore-tov", read the Cargo.toml and extract the version.
    fn read_crate_version(&self, crate_path: &str) -> Result<String> {
        let joined = self.workspace_root.join(crate_path);
        let canonical = joined
            .canonicalize()
            .with_context(|| format!("Failed to resolve crate path: {}", joined.display()))?;
        let canonical_root = self.workspace_root.canonicalize().with_context(|| {
            format!(
                "Failed to resolve workspace root: {}",
                self.workspace_root.display()
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            bail!(
                "Crate path '{}' escapes the workspace root '{}'",
                crate_path,
                canonical_root.display()
            );
        }
        let cargo_path = canonical.join("Cargo.toml");
        let contents = std::fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read {}", cargo_path.display()))?;
        let doc: DocumentMut = contents
            .parse()
            .with_context(|| format!("Failed to parse {}", cargo_path.display()))?;

        // If the crate uses version.workspace = true, return the workspace version
        let version_item = &doc["package"]["version"];
        if let Some(table) = version_item.as_inline_table() {
            if table.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
                // This crate inherits from workspace — we need to read the workspace version
                return self.read_workspace_version();
            }
        }
        // Check for dotted key: version.workspace = true shows up differently in toml_edit
        if let Some(pkg_table) = doc["package"].as_table() {
            if let Some(version_item) = pkg_table.get("version") {
                if let Some(tbl) = version_item.as_table_like() {
                    if tbl.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
                        return self.read_workspace_version();
                    }
                }
            }
        }

        version_item.as_str().map(|s| s.to_string()).ok_or_else(|| {
            nexcore_error::nexerror!("No version string found in {}", cargo_path.display())
        })
    }

    fn read_workspace_version(&self) -> Result<String> {
        let root_toml = self.workspace_root.join("Cargo.toml");
        let contents = std::fs::read_to_string(&root_toml)?;
        let doc: DocumentMut = contents.parse()?;
        doc["workspace"]["package"]["version"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| nexcore_error::nexerror!("No workspace.package.version found"))
    }
}

impl FileSystemResolver {
    pub fn resolve_version_from_path(&self, crate_path: &str) -> Result<String> {
        self.read_crate_version(crate_path)
    }
}

/// Information about a workspace dependency entry.
struct WorkspaceDep {
    /// True if this is an internal dep (has a `path` field).
    is_internal: bool,
    /// The path field value (e.g., "crates/nexcore-tov") for internal deps.
    path: Option<String>,
    /// The package name if a `package` field is set (for renames).
    package: Option<String>,
    /// The version string (for external deps).
    version: Option<String>,
    /// Features defined at the workspace level.
    features: Vec<String>,
    /// default-features setting from workspace, if present.
    default_features: Option<bool>,
}

/// Parse workspace.dependencies to build a lookup table.
fn parse_workspace_deps(
    workspace_doc: &DocumentMut,
) -> Result<std::collections::HashMap<String, WorkspaceDep>> {
    let mut deps = std::collections::HashMap::new();

    let ws_deps = match workspace_doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
    {
        Some(d) => d,
        None => return Ok(deps),
    };

    let table = ws_deps
        .as_table_like()
        .context("workspace.dependencies is not a table")?;

    for (name, item) in table.iter() {
        let dep = match item {
            Item::Value(Value::String(s)) => {
                // Simple version string: dep = "1.0"
                WorkspaceDep {
                    is_internal: false,
                    path: None,
                    package: None,
                    version: Some(s.value().clone()),
                    features: Vec::new(),
                    default_features: None,
                }
            }
            Item::Value(Value::InlineTable(t)) => {
                let path = t
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let package = t
                    .get("package")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let version = t
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let features = t
                    .get("features")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let default_features = t.get("default-features").and_then(|v| v.as_bool());

                WorkspaceDep {
                    is_internal: path.is_some(),
                    path,
                    package,
                    version,
                    features,
                    default_features,
                }
            }
            _ => continue,
        };

        deps.insert(name.to_string(), dep);
    }

    Ok(deps)
}

/// Parse workspace.package fields.
fn parse_workspace_package(workspace_doc: &DocumentMut) -> Result<Table> {
    let pkg = workspace_doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.as_table())
        .context("workspace.package not found")?;
    Ok(pkg.clone())
}

/// Parse workspace.lints sections.
fn parse_workspace_lints(workspace_doc: &DocumentMut) -> Option<&Item> {
    workspace_doc.get("workspace").and_then(|w| w.get("lints"))
}

/// Check if a crate Cargo.toml has any workspace references.
fn has_workspace_references(doc: &DocumentMut) -> bool {
    // Check package fields
    if let Some(pkg) = doc.get("package").and_then(|p| p.as_table()) {
        for (_key, item) in pkg.iter() {
            if is_workspace_true(item) {
                return true;
            }
        }
    }

    // Check dependency sections
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if section_has_workspace_refs(doc.get(section)) {
            return true;
        }
    }

    // Check target-specific deps
    if let Some(target) = doc.get("target").and_then(|t| t.as_table_like()) {
        for (_target_name, target_item) in target.iter() {
            for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(target_table) = target_item.as_table_like() {
                    if section_has_workspace_refs(target_table.get(section)) {
                        return true;
                    }
                }
            }
        }
    }

    // Check lints
    if let Some(lints) = doc.get("lints") {
        if let Some(tbl) = lints.as_table_like() {
            if tbl.get("workspace").is_some() {
                return true;
            }
        }
    }

    // Check path deps (not workspace but still need conversion)
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if section_has_path_deps(doc.get(section)) {
            return true;
        }
    }

    if let Some(target) = doc.get("target").and_then(|t| t.as_table_like()) {
        for (_target_name, target_item) in target.iter() {
            for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(target_table) = target_item.as_table_like() {
                    if section_has_path_deps(target_table.get(section)) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn section_has_workspace_refs(section: Option<&Item>) -> bool {
    let table = match section.and_then(|s| s.as_table_like()) {
        Some(t) => t,
        None => return false,
    };
    for (_name, item) in table.iter() {
        if is_workspace_true(item) {
            return true;
        }
    }
    false
}

fn section_has_path_deps(section: Option<&Item>) -> bool {
    let table = match section.and_then(|s| s.as_table_like()) {
        Some(t) => t,
        None => return false,
    };
    for (_name, item) in table.iter() {
        if has_path_field(item) {
            return true;
        }
    }
    false
}

fn is_workspace_true(item: &Item) -> bool {
    match item {
        Item::Value(Value::InlineTable(t)) => {
            t.get("workspace").and_then(|v| v.as_bool()) == Some(true)
        }
        // Dotted key syntax (e.g., `dep.workspace = true`) is parsed as a Table by toml_edit
        Item::Table(t) => t.get("workspace").and_then(|i| i.as_bool()) == Some(true),
        _ => false,
    }
}

fn has_path_field(item: &Item) -> bool {
    match item {
        Item::Value(Value::InlineTable(t)) => t.get("path").is_some(),
        Item::Table(t) => t.get("path").is_some(),
        _ => false,
    }
}

/// Convert a crate's Cargo.toml from workspace-dependent to standalone.
///
/// # Arguments
/// * `crate_toml` - The crate's Cargo.toml content
/// * `workspace_toml` - The root workspace Cargo.toml content
/// * `registry_name` - The name of the private registry for internal deps
/// * `resolver` - Implementation to resolve internal crate versions
///
/// # Returns
/// The converted Cargo.toml content as a string.
pub fn convert_cargo_toml(
    crate_toml: &str,
    workspace_toml: &str,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<String> {
    if registry_name.is_empty() {
        bail!("Registry name must not be empty");
    }
    if !registry_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!(
            "Registry name '{}' contains invalid characters; \
             only alphanumeric characters, hyphens, and underscores are allowed",
            registry_name
        );
    }

    let mut doc: DocumentMut = crate_toml
        .parse()
        .context("Failed to parse crate Cargo.toml")?;
    let workspace_doc: DocumentMut = workspace_toml
        .parse()
        .context("Failed to parse workspace Cargo.toml")?;

    if !has_workspace_references(&doc) {
        return Ok(crate_toml.to_string());
    }

    let ws_package = parse_workspace_package(&workspace_doc)?;
    let ws_deps = parse_workspace_deps(&workspace_doc)?;

    // Step 1: Inline workspace package fields
    inline_package_fields(&mut doc, &ws_package)?;

    // Step 2: Convert dependency sections
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        convert_dep_section(&mut doc, section, &ws_deps, registry_name, resolver)?;
    }

    // Step 3: Convert target-specific dependency sections
    convert_target_deps(&mut doc, &ws_deps, registry_name, resolver)?;

    // Step 4: Inline workspace lints
    if let Some(ws_lints) = parse_workspace_lints(&workspace_doc) {
        inline_lints(&mut doc, ws_lints)?;
    }

    Ok(doc.to_string())
}

/// Replace `field.workspace = true` with the actual value from workspace.package.
fn inline_package_fields(doc: &mut DocumentMut, ws_package: &Table) -> Result<()> {
    let pkg = doc["package"]
        .as_table_mut()
        .context("No [package] table found")?;

    let fields_to_inline: Vec<String> = pkg
        .iter()
        .filter(|(_key, item)| {
            // Detect dotted syntax: version.workspace = true
            // In toml_edit, this appears as a Table with a "workspace" key
            if let Some(tbl) = item.as_table_like() {
                return tbl.get("workspace").and_then(|v| v.as_bool()) == Some(true);
            }
            false
        })
        .map(|(key, _)| key.to_string())
        .collect();

    for field in &fields_to_inline {
        let ws_value = ws_package.get(field).with_context(|| {
            format!("workspace.package.{field} not found but crate references it")
        })?;

        // Replace the workspace reference with the actual value
        pkg.insert(field, ws_value.clone());
    }

    Ok(())
}

/// Convert all deps in a dependency section.
fn convert_dep_section(
    doc: &mut DocumentMut,
    section: &str,
    ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<()> {
    if doc.get(section).and_then(|s| s.as_table_like()).is_none() {
        return Ok(());
    }

    // Collect dep names that need conversion
    let dep_names: Vec<String> = doc[section]
        .as_table_like()
        .unwrap()
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    for dep_name in dep_names {
        let item = &doc[section][&dep_name];

        // Check if this is a workspace = true dep
        if is_workspace_true(item) {
            convert_workspace_dep(doc, section, &dep_name, ws_deps, registry_name, resolver)?;
        } else if has_path_field(item) {
            convert_path_dep(doc, section, &dep_name, ws_deps, registry_name, resolver)?;
        }
    }

    Ok(())
}

/// Convert target-specific deps.
fn convert_target_deps(
    doc: &mut DocumentMut,
    ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<()> {
    let target_names: Vec<String> = match doc.get("target").and_then(|t| t.as_table_like()) {
        Some(t) => t.iter().map(|(name, _)| name.to_string()).collect(),
        None => return Ok(()),
    };

    for target_name in &target_names {
        for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
            let dep_names: Vec<String> = match doc
                .get("target")
                .and_then(|t| t.get(target_name.as_str()))
                .and_then(|t| t.get(*section))
                .and_then(|s| s.as_table_like())
            {
                Some(t) => t.iter().map(|(name, _)| name.to_string()).collect(),
                None => continue,
            };

            for dep_name in dep_names {
                let item = &doc["target"][target_name.as_str()][*section][&dep_name];
                if is_workspace_true(item) {
                    convert_workspace_dep_in_target(
                        doc,
                        target_name,
                        section,
                        &dep_name,
                        ws_deps,
                        registry_name,
                        resolver,
                    )?;
                } else if has_path_field(item) {
                    convert_path_dep_in_target(
                        doc,
                        target_name,
                        section,
                        &dep_name,
                        ws_deps,
                        registry_name,
                        resolver,
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Convert a `dep = { workspace = true, ... }` or `dep.workspace = true` to its resolved form.
fn convert_workspace_dep(
    doc: &mut DocumentMut,
    section: &str,
    dep_name: &str,
    ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<()> {
    let ws_dep = ws_deps
        .get(dep_name)
        .with_context(|| format!("{dep_name} not found in workspace.dependencies"))?;

    let new_item = build_resolved_dep(doc, section, dep_name, ws_dep, registry_name, resolver)?;
    doc[section][dep_name] = new_item;

    Ok(())
}

fn convert_workspace_dep_in_target(
    doc: &mut DocumentMut,
    target_name: &str,
    section: &str,
    dep_name: &str,
    ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<()> {
    let ws_dep = ws_deps
        .get(dep_name)
        .with_context(|| format!("{dep_name} not found in workspace.dependencies"))?;

    let new_item = build_resolved_dep_for_target(
        doc,
        target_name,
        section,
        dep_name,
        ws_dep,
        registry_name,
        resolver,
    )?;
    doc["target"][target_name][section][dep_name] = new_item;

    Ok(())
}

/// Build the resolved Item for a workspace dep.
fn build_resolved_dep(
    doc: &DocumentMut,
    section: &str,
    dep_name: &str,
    ws_dep: &WorkspaceDep,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<Item> {
    // Get the crate-level overrides (features, optional, default-features)
    let crate_item = &doc[section][dep_name];
    build_resolved_dep_from_item(crate_item, dep_name, ws_dep, registry_name, resolver)
}

fn build_resolved_dep_for_target(
    doc: &DocumentMut,
    target_name: &str,
    section: &str,
    dep_name: &str,
    ws_dep: &WorkspaceDep,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<Item> {
    let crate_item = &doc["target"][target_name][section][dep_name];
    build_resolved_dep_from_item(crate_item, dep_name, ws_dep, registry_name, resolver)
}

fn build_resolved_dep_from_item(
    crate_item: &Item,
    dep_name: &str,
    ws_dep: &WorkspaceDep,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<Item> {
    let crate_features = extract_features(crate_item);
    let crate_optional = extract_bool_field(crate_item, "optional");
    let crate_default_features = extract_bool_field(crate_item, "default-features");

    let mut new_table = InlineTable::new();

    if ws_dep.is_internal {
        // Internal dep: use version from resolver, add registry
        let package_name = ws_dep.package.as_deref().unwrap_or(dep_name);
        let version = resolver.resolve_version(package_name)?;
        new_table.insert("version", Value::String(Formatted::new(version)));
        new_table.insert(
            "registry",
            Value::String(Formatted::new(registry_name.to_string())),
        );
        // Preserve package rename if present
        if let Some(pkg) = &ws_dep.package {
            new_table.insert("package", Value::String(Formatted::new(pkg.clone())));
        }
    } else {
        // External dep: use version from workspace
        let version = ws_dep
            .version
            .as_ref()
            .with_context(|| format!("No version for external dep {dep_name}"))?;
        new_table.insert("version", Value::String(Formatted::new(version.clone())));
    }

    // Handle default-features
    if let Some(false) = crate_default_features {
        new_table.insert("default-features", Value::Boolean(Formatted::new(false)));
    } else if let Some(false) = ws_dep.default_features {
        // Only apply workspace default-features if crate doesn't override
        if crate_default_features.is_none() {
            new_table.insert("default-features", Value::Boolean(Formatted::new(false)));
        }
    }

    // Merge features: workspace features + crate-level features (deduplicated)
    let merged_features = if crate_default_features == Some(false) {
        // When crate sets default-features = false, only use crate-level features
        // (don't inherit workspace features since the crate is being explicit about what it wants)
        merge_features(&[], &crate_features)
    } else {
        merge_features(&ws_dep.features, &crate_features)
    };

    if !merged_features.is_empty() {
        let mut arr = Array::new();
        for f in &merged_features {
            arr.push(f.as_str());
        }
        new_table.insert("features", Value::Array(arr));
    }

    // Handle optional
    if let Some(true) = crate_optional {
        new_table.insert("optional", Value::Boolean(Formatted::new(true)));
    }

    Ok(Item::Value(Value::InlineTable(new_table)))
}

/// Convert a path dep (not workspace) to a registry dep.
fn convert_path_dep(
    doc: &mut DocumentMut,
    section: &str,
    dep_name: &str,
    ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<()> {
    let new_item = build_path_dep_replacement(
        &doc[section][dep_name],
        dep_name,
        ws_deps,
        registry_name,
        resolver,
    )?;
    doc[section][dep_name] = new_item;
    Ok(())
}

fn convert_path_dep_in_target(
    doc: &mut DocumentMut,
    target_name: &str,
    section: &str,
    dep_name: &str,
    ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<()> {
    let new_item = build_path_dep_replacement(
        &doc["target"][target_name][section][dep_name],
        dep_name,
        ws_deps,
        registry_name,
        resolver,
    )?;
    doc["target"][target_name][section][dep_name] = new_item;
    Ok(())
}

fn build_path_dep_replacement(
    item: &Item,
    dep_name: &str,
    _ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
    registry_name: &str,
    resolver: &dyn InternalDepResolver,
) -> Result<Item> {
    let mut new_table = InlineTable::new();

    // If there's already a version, use it; otherwise resolve via the resolver
    let existing_version = extract_string_field(item, "version");
    let version = match existing_version {
        Some(v) => v,
        None => resolver.resolve_version(dep_name)?,
    };

    new_table.insert("version", Value::String(Formatted::new(version)));
    new_table.insert(
        "registry",
        Value::String(Formatted::new(registry_name.to_string())),
    );

    // Preserve features if present
    let features = extract_features(item);
    if !features.is_empty() {
        let mut arr = Array::new();
        for f in &features {
            arr.push(f.as_str());
        }
        new_table.insert("features", Value::Array(arr));
    }

    // Preserve optional if present
    if let Some(true) = extract_bool_field(item, "optional") {
        new_table.insert("optional", Value::Boolean(Formatted::new(true)));
    }

    // Preserve default-features if present
    if let Some(false) = extract_bool_field(item, "default-features") {
        new_table.insert("default-features", Value::Boolean(Formatted::new(false)));
    }

    // Preserve package rename if present
    if let Some(pkg) = extract_string_field(item, "package") {
        new_table.insert("package", Value::String(Formatted::new(pkg)));
    }

    Ok(Item::Value(Value::InlineTable(new_table)))
}

/// Extract features from a dep item.
fn extract_features(item: &Item) -> Vec<String> {
    let features_value = match item {
        Item::Value(Value::InlineTable(t)) => t.get("features").and_then(|v| v.as_array()),
        Item::Table(t) => t.get("features").and_then(|i| i.as_array()),
        _ => None,
    };

    features_value
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract a boolean field from a dep item.
fn extract_bool_field(item: &Item, field: &str) -> Option<bool> {
    match item {
        Item::Value(Value::InlineTable(t)) => t.get(field).and_then(|v| v.as_bool()),
        Item::Table(t) => t.get(field).and_then(|i| i.as_bool()),
        _ => None,
    }
}

/// Extract a string field from a dep item.
fn extract_string_field(item: &Item, field: &str) -> Option<String> {
    match item {
        Item::Value(Value::InlineTable(t)) => {
            t.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
        }
        Item::Table(t) => t.get(field).and_then(|i| i.as_str()).map(|s| s.to_string()),
        _ => None,
    }
}

/// Merge two feature lists, deduplicating.
fn merge_features(ws_features: &[String], crate_features: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for f in ws_features.iter().chain(crate_features.iter()) {
        if seen.insert(f.clone()) {
            result.push(f.clone());
        }
    }

    result
}

/// Replace `[lints] workspace = true` with concrete lint config from workspace.
fn inline_lints(doc: &mut DocumentMut, ws_lints: &Item) -> Result<()> {
    let lints = match doc.get("lints").and_then(|l| l.as_table_like()) {
        Some(t) => t,
        None => return Ok(()),
    };

    // Check if there's a `workspace = true` in [lints]
    let has_workspace = lints
        .get("workspace")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !has_workspace {
        return Ok(());
    }

    // Replace the entire [lints] section with workspace lints
    let ws_lints_table = ws_lints
        .as_table()
        .context("workspace.lints is not a table")?;

    // Build a new lints table
    let mut new_lints = Table::new();
    new_lints.set_implicit(true);
    for (category, items) in ws_lints_table.iter() {
        if let Some(category_table) = items.as_table() {
            let mut new_category = Table::new();
            for (lint_name, lint_value) in category_table.iter() {
                new_category.insert(lint_name, lint_value.clone());
            }
            new_lints.insert(category, Item::Table(new_category));
        }
    }

    doc.insert("lints", Item::Table(new_lints));

    Ok(())
}

/// Convert a single crate's Cargo.toml using the filesystem for resolution.
pub fn convert_crate_file(
    crate_cargo_path: &std::path::Path,
    workspace_root: &std::path::Path,
    registry_name: &str,
) -> Result<String> {
    let crate_toml = std::fs::read_to_string(crate_cargo_path)
        .with_context(|| format!("Failed to read {}", crate_cargo_path.display()))?;
    let workspace_toml = std::fs::read_to_string(workspace_root.join("Cargo.toml"))
        .with_context(|| "Failed to read workspace Cargo.toml".to_string())?;

    let workspace_doc: DocumentMut = workspace_toml.parse()?;
    let ws_deps = parse_workspace_deps(&workspace_doc)?;
    let fs_resolver = FileSystemResolver::new(workspace_root.to_path_buf());

    // Build a resolver that maps crate names to their versions using the filesystem
    let mapping_resolver = MappingResolver::new(&ws_deps, &fs_resolver)?;

    convert_cargo_toml(
        &crate_toml,
        &workspace_toml,
        registry_name,
        &mapping_resolver,
    )
}

/// A resolver that pre-builds a name->version mapping using the filesystem resolver.
struct MappingResolver {
    versions: std::collections::HashMap<String, String>,
}

impl MappingResolver {
    fn new(
        ws_deps: &std::collections::HashMap<String, WorkspaceDep>,
        fs_resolver: &FileSystemResolver,
    ) -> Result<Self> {
        let mut versions = std::collections::HashMap::new();

        for (name, dep) in ws_deps {
            if dep.is_internal {
                if let Some(path) = &dep.path {
                    let version = fs_resolver.resolve_version_from_path(path)?;
                    // Map by package name (which is the actual crate name)
                    let package_name = dep.package.as_deref().unwrap_or(name.as_str());
                    versions.insert(package_name.to_string(), version);
                }
            }
        }

        Ok(Self { versions })
    }
}

impl InternalDepResolver for MappingResolver {
    fn resolve_version(&self, crate_name: &str) -> Result<String> {
        self.versions.get(crate_name).cloned().ok_or_else(|| {
            nexcore_error::nexerror!("Internal crate version not found: {crate_name}")
        })
    }
}
