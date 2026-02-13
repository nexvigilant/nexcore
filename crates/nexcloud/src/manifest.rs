use crate::error::{NexCloudError, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Top-level cloud platform manifest parsed from TOML.
///
/// Tier: T3 (σ Sequence + μ Mapping + ∂ Boundary + π Persistence + ∃ Existence)
/// The manifest is the declarative specification for the entire platform.
#[derive(Debug, Clone, Deserialize)]
pub struct CloudManifest {
    pub platform: PlatformDef,
    pub proxy: ProxyDef,
    #[serde(default, rename = "service")]
    pub services: Vec<ServiceDef>,
    #[serde(default, rename = "route")]
    pub routes: Vec<RouteDef>,
}

/// Platform identity and global settings.
///
/// Tier: T1 (π Persistence) — identity that persists across restarts.
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformDef {
    pub name: String,
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,
}

fn default_log_dir() -> PathBuf {
    PathBuf::from("/var/log/nexcloud")
}

/// Reverse proxy configuration.
///
/// Tier: T2-P (∂ Boundary) — defines the network boundary.
#[derive(Debug, Clone, Deserialize)]
pub struct ProxyDef {
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_https_port")]
    pub https_port: u16,
    pub tls: Option<TlsDef>,
}

fn default_http_port() -> u16 {
    80
}

fn default_https_port() -> u16 {
    443
}

/// TLS certificate configuration.
///
/// Tier: T2-P (∂ Boundary) — cryptographic boundary.
#[derive(Debug, Clone, Deserialize)]
pub struct TlsDef {
    pub cert: PathBuf,
    pub key: PathBuf,
}

/// Restart policy for a service.
///
/// Tier: T2-P (ρ Recursion) — retry/backoff is recursive by nature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicyKind {
    Always,
    OnFailure,
    Never,
}

impl Default for RestartPolicyKind {
    fn default() -> Self {
        Self::OnFailure
    }
}

/// A service definition from the manifest.
///
/// Tier: T3 (σ Sequence + ς State + ∂ Boundary + μ Mapping + ρ Recursion + π Persistence)
/// Full domain type — the complete specification for a managed process.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub binary: PathBuf,
    pub port: u16,
    #[serde(default = "default_health_path")]
    pub health: String,
    #[serde(default)]
    pub restart: RestartPolicyKind,
    #[serde(default = "default_max_restarts")]
    pub max_restarts: u32,
    #[serde(default = "default_backoff_ms")]
    pub backoff_ms: u64,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_health_path() -> String {
    "/health".to_string()
}

fn default_max_restarts() -> u32 {
    10
}

fn default_backoff_ms() -> u64 {
    1000
}

/// A routing rule for the reverse proxy.
///
/// Tier: T2-C (μ Mapping + ∂ Boundary + λ Location)
/// Maps incoming requests to backend services based on host/path boundaries.
#[derive(Debug, Clone, Deserialize)]
pub struct RouteDef {
    #[serde(default)]
    pub match_host: Option<String>,
    #[serde(default)]
    pub match_prefix: Option<String>,
    pub backend: String,
    #[serde(default)]
    pub strip_prefix: bool,
}

/// SEC-012: Validate that an identifier contains only safe characters.
///
/// Allowed: `[a-zA-Z0-9_-]`, must not be empty, max 64 chars.
/// Prevents: command injection (SEC-001), path traversal (SEC-002),
/// log injection, and tracing format string issues.
fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

impl CloudManifest {
    /// Parse a manifest from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self> {
        let manifest: Self =
            toml::from_str(content).map_err(|e| NexCloudError::ManifestParse(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Parse a manifest from a file path.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Validate the manifest for internal consistency.
    ///
    /// SEC: Input validation boundary — every field that flows into shell commands,
    /// file paths, or network addresses is validated here at parse time.
    fn validate(&self) -> Result<()> {
        // SEC-012: Validate service names contain only safe characters
        // Names flow into: log file paths, SSH commands, tracing output, PID file names
        for svc in &self.services {
            if !is_safe_identifier(&svc.name) {
                return Err(NexCloudError::ManifestValidation(format!(
                    "service name '{}' contains invalid characters (allowed: a-z, A-Z, 0-9, -, _)",
                    svc.name
                )));
            }
        }

        // Validate route backend names
        for route in &self.routes {
            if !is_safe_identifier(&route.backend) {
                return Err(NexCloudError::ManifestValidation(format!(
                    "route backend '{}' contains invalid characters",
                    route.backend
                )));
            }
        }

        // Check service names are unique
        let mut names = HashSet::new();
        for svc in &self.services {
            if !names.insert(&svc.name) {
                return Err(NexCloudError::ManifestValidation(format!(
                    "duplicate service name: '{}'",
                    svc.name
                )));
            }
        }

        // Check ports are unique across services
        let mut ports = HashSet::new();
        for svc in &self.services {
            if !ports.insert(svc.port) {
                return Err(NexCloudError::ManifestValidation(format!(
                    "duplicate port {} used by service '{}'",
                    svc.port, svc.name
                )));
            }
        }

        // Check route backends reference existing services
        for route in &self.routes {
            if !names.contains(&route.backend) {
                return Err(NexCloudError::ManifestValidation(format!(
                    "route backend '{}' not found in services",
                    route.backend
                )));
            }
        }

        // Check routes have at least one match criterion
        for route in &self.routes {
            if route.match_host.is_none() && route.match_prefix.is_none() {
                return Err(NexCloudError::ManifestValidation(format!(
                    "route for backend '{}' has no match_host or match_prefix",
                    route.backend
                )));
            }
        }

        // Check dependency references exist
        for svc in &self.services {
            for dep in &svc.depends_on {
                if !names.contains(dep) {
                    return Err(NexCloudError::ManifestValidation(format!(
                        "service '{}' depends on unknown service '{dep}'",
                        svc.name
                    )));
                }
            }
        }

        // Detect dependency cycles via DFS
        self.check_cycles(&names)?;

        Ok(())
    }

    /// Detect dependency cycles using iterative DFS with coloring.
    fn check_cycles(&self, _names: &HashSet<&String>) -> Result<()> {
        let dep_map: HashMap<&str, &[String]> = self
            .services
            .iter()
            .map(|s| (s.name.as_str(), s.depends_on.as_slice()))
            .collect();

        // 0 = white (unvisited), 1 = gray (in-progress), 2 = black (done)
        let mut color: HashMap<&str, u8> = dep_map.keys().map(|&k| (k, 0)).collect();

        for &start in dep_map.keys() {
            if color[start] != 0 {
                continue;
            }
            let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
            color.insert(start, 1);

            while let Some((node, idx)) = stack.last_mut() {
                let deps = dep_map.get(*node).copied().unwrap_or(&[]);
                if *idx >= deps.len() {
                    color.insert(node, 2);
                    stack.pop();
                    continue;
                }
                let dep = deps[*idx].as_str();
                *idx += 1;

                match color.get(dep).copied().unwrap_or(0) {
                    0 => {
                        color.insert(dep, 1);
                        stack.push((dep, 0));
                    }
                    1 => {
                        // Gray → cycle detected. Build cycle path.
                        let cycle_path: Vec<&str> = stack
                            .iter()
                            .map(|(n, _)| *n)
                            .chain(std::iter::once(dep))
                            .collect();
                        return Err(NexCloudError::DependencyCycle {
                            cycle: cycle_path.join(" -> "),
                        });
                    }
                    _ => {} // Black — already fully explored
                }
            }
        }

        Ok(())
    }

    /// Topologically sort services by dependency order.
    /// Returns service names in the order they should be started.
    pub fn topo_sort(&self) -> Result<Vec<String>> {
        let dep_map: HashMap<&str, Vec<&str>> = self
            .services
            .iter()
            .map(|s| {
                (
                    s.name.as_str(),
                    s.depends_on.iter().map(|d| d.as_str()).collect(),
                )
            })
            .collect();

        let mut in_degree: HashMap<&str, usize> = dep_map.keys().map(|&k| (k, 0)).collect();
        for deps in dep_map.values() {
            for &dep in deps {
                *in_degree.entry(dep).or_insert(0) += 1;
            }
        }

        // Note: in_degree counts dependents, not dependencies.
        // We need the reverse: start services with no dependencies first.
        let mut in_degree2: HashMap<&str, usize> =
            dep_map.keys().map(|&k| (k, dep_map[k].len())).collect();

        let mut queue: Vec<&str> = in_degree2
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(k, _)| *k)
            .collect();
        queue.sort(); // deterministic ordering

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node.to_string());
            // Find services that depend on this node and decrement their count
            for (&svc, deps) in &dep_map {
                if deps.contains(&node) {
                    if let Some(deg) = in_degree2.get_mut(svc) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(svc);
                            queue.sort();
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Look up a service definition by name.
    pub fn service_by_name(&self, name: &str) -> Option<&ServiceDef> {
        self.services.iter().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_toml() -> &'static str {
        r#"
[platform]
name = "test"

[proxy]
http_port = 8080
https_port = 8443

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "web"
"#
    }

    #[test]
    fn parse_minimal_manifest() {
        let manifest = CloudManifest::from_toml(minimal_toml());
        assert!(manifest.is_ok());
        let m = manifest.ok().unwrap_or_else(|| panic!("parse failed"));
        assert_eq!(m.platform.name, "test");
        assert_eq!(m.services.len(), 1);
        assert_eq!(m.routes.len(), 1);
    }

    #[test]
    fn parse_full_manifest() {
        let toml = r#"
[platform]
name = "nexvigilant"

[proxy]
http_port = 80
https_port = 443

[proxy.tls]
cert = "/etc/nexcloud/cert.pem"
key = "/etc/nexcloud/key.pem"

[[service]]
name = "api"
binary = "./target/release/nexcore-api"
port = 3030
health = "/health"
restart = "always"
max_restarts = 10
backoff_ms = 1000
env = { RUST_LOG = "info" }

[[service]]
name = "metrics"
binary = "./target/release/claude-metrics"
port = 9090
health = "/metrics"
restart = "on-failure"
max_restarts = 5
depends_on = ["api"]

[[route]]
match_host = "api.nexvigilant.com"
backend = "api"

[[route]]
match_prefix = "/metrics"
backend = "metrics"
strip_prefix = true
"#;
        let manifest = CloudManifest::from_toml(toml);
        assert!(manifest.is_ok());
        let m = manifest.ok().unwrap_or_else(|| panic!("parse failed"));
        assert_eq!(m.services.len(), 2);
        assert_eq!(m.routes.len(), 2);
        assert!(m.proxy.tls.is_some());
        assert_eq!(m.services[1].depends_on, vec!["api"]);
    }

    #[test]
    fn reject_duplicate_service_names() {
        let toml = r#"
[platform]
name = "test"
[proxy]
http_port = 80
https_port = 443

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 3000

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 3001

[[route]]
match_prefix = "/"
backend = "web"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
        if let Err(e) = result {
            let err = format!("{e}");
            assert!(err.contains("duplicate service name"));
        }
    }

    #[test]
    fn reject_duplicate_ports() {
        let toml = r#"
[platform]
name = "test"
[proxy]
http_port = 80
https_port = 443

[[service]]
name = "a"
binary = "/usr/bin/true"
port = 3000

[[service]]
name = "b"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "a"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn reject_unknown_route_backend() {
        let toml = r#"
[platform]
name = "test"
[proxy]
http_port = 80
https_port = 443

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "nonexistent"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn reject_dependency_cycle() {
        let toml = r#"
[platform]
name = "test"
[proxy]
http_port = 80
https_port = 443

[[service]]
name = "a"
binary = "/usr/bin/true"
port = 3000
depends_on = ["b"]

[[service]]
name = "b"
binary = "/usr/bin/true"
port = 3001
depends_on = ["a"]

[[route]]
match_prefix = "/"
backend = "a"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn topo_sort_respects_deps() {
        let toml = r#"
[platform]
name = "test"
[proxy]
http_port = 80
https_port = 443

[[service]]
name = "db"
binary = "/usr/bin/true"
port = 5432

[[service]]
name = "api"
binary = "/usr/bin/true"
port = 3030
depends_on = ["db"]

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 8080
depends_on = ["api"]

[[route]]
match_prefix = "/"
backend = "web"
"#;
        let manifest = CloudManifest::from_toml(toml);
        assert!(manifest.is_ok());
        let m = manifest.ok().unwrap_or_else(|| panic!("parse failed"));
        let order = m.topo_sort();
        assert!(order.is_ok());
        let order = order.ok().unwrap_or_else(|| panic!("sort failed"));
        // db must come before api, api before web
        let db_pos = order.iter().position(|s| s == "db");
        let api_pos = order.iter().position(|s| s == "api");
        let web_pos = order.iter().position(|s| s == "web");
        assert!(db_pos < api_pos);
        assert!(api_pos < web_pos);
    }

    #[test]
    fn defaults_applied_correctly() {
        let toml = r#"
[platform]
name = "test"
[proxy]

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "web"
"#;
        let m = CloudManifest::from_toml(toml);
        assert!(m.is_ok());
        let m = m.ok().unwrap_or_else(|| panic!("parse failed"));
        assert_eq!(m.proxy.http_port, 80);
        assert_eq!(m.proxy.https_port, 443);
        assert_eq!(m.services[0].health, "/health");
        assert_eq!(m.services[0].max_restarts, 10);
        assert_eq!(m.services[0].backoff_ms, 1000);
        assert_eq!(m.services[0].restart, RestartPolicyKind::OnFailure);
    }

    #[test]
    fn reject_service_name_with_slash() {
        let toml = r#"
[platform]
name = "test"
[proxy]

[[service]]
name = "../etc/shadow"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "../etc/shadow"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(msg.contains("invalid characters"));
        }
    }

    #[test]
    fn reject_service_name_with_semicolon() {
        let toml = r#"
[platform]
name = "test"
[proxy]

[[service]]
name = "foo;rm -rf /"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "foo;rm -rf /"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_service_name() {
        let toml = r#"
[platform]
name = "test"
[proxy]

[[service]]
name = ""
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = ""
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn accept_valid_service_names() {
        let toml = r#"
[platform]
name = "test"
[proxy]

[[service]]
name = "nexcore-api_v2"
binary = "/usr/bin/true"
port = 3000

[[route]]
match_prefix = "/"
backend = "nexcore-api_v2"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_ok());
    }

    #[test]
    fn is_safe_identifier_rejects_path_traversal() {
        assert!(!super::is_safe_identifier("../etc/shadow"));
        assert!(!super::is_safe_identifier("foo/bar"));
        assert!(!super::is_safe_identifier("a;b"));
        assert!(!super::is_safe_identifier("a b"));
        assert!(!super::is_safe_identifier(""));
        assert!(!super::is_safe_identifier(&"a".repeat(65)));
    }

    #[test]
    fn is_safe_identifier_accepts_valid() {
        assert!(super::is_safe_identifier("web"));
        assert!(super::is_safe_identifier("nexcore-api"));
        assert!(super::is_safe_identifier("my_service_v2"));
        assert!(super::is_safe_identifier("API"));
        assert!(super::is_safe_identifier("a"));
        assert!(super::is_safe_identifier(&"a".repeat(64)));
    }

    #[test]
    fn reject_route_without_match() {
        let toml = r#"
[platform]
name = "test"
[proxy]

[[service]]
name = "web"
binary = "/usr/bin/true"
port = 3000

[[route]]
backend = "web"
"#;
        let result = CloudManifest::from_toml(toml);
        assert!(result.is_err());
    }
}
