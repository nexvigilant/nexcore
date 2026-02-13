use crate::manifest::RouteDef;

/// A resolved backend target for proxying.
///
/// Tier: T2-P (λ Location) — a specific network location.
#[derive(Debug, Clone)]
pub struct Backend {
    pub name: String,
    pub addr: String,
    pub strip_prefix: bool,
    pub prefix: Option<String>,
}

/// A compiled routing rule.
///
/// Tier: T2-C (μ Mapping + ∂ Boundary + λ Location)
/// Maps incoming requests across host/path boundaries to backend locations.
#[derive(Debug, Clone)]
struct RouteRule {
    match_host: Option<String>,
    match_prefix: Option<String>,
    backend: Backend,
}

/// The routing table: matches incoming requests to backends.
///
/// Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)
/// Ordered sequence of rules — first match wins.
#[derive(Debug, Clone)]
pub struct RoutingTable {
    rules: Vec<RouteRule>,
}

impl RoutingTable {
    /// Build a routing table from route definitions and a port lookup function.
    pub fn from_routes<F>(routes: &[RouteDef], port_lookup: F) -> Self
    where
        F: Fn(&str) -> Option<u16>,
    {
        let rules = routes
            .iter()
            .filter_map(|r| {
                let port = port_lookup(&r.backend)?;
                Some(RouteRule {
                    match_host: r.match_host.clone(),
                    match_prefix: r.match_prefix.clone(),
                    backend: Backend {
                        name: r.backend.clone(),
                        addr: format!("127.0.0.1:{port}"),
                        strip_prefix: r.strip_prefix,
                        prefix: r.match_prefix.clone(),
                    },
                })
            })
            .collect();

        Self { rules }
    }

    /// Look up the backend for a given request.
    /// Checks host header first, then path prefix. First match wins.
    pub fn route(&self, host: Option<&str>, path: &str) -> Option<Backend> {
        for rule in &self.rules {
            // Check host match
            if let Some(ref match_host) = rule.match_host {
                if let Some(req_host) = host {
                    // Strip port from host header if present
                    let req_host_bare = req_host.split(':').next().unwrap_or(req_host);
                    if req_host_bare == match_host {
                        return Some(rule.backend.clone());
                    }
                }
                continue; // host rule didn't match, skip
            }

            // Check prefix match
            if let Some(ref match_prefix) = rule.match_prefix {
                if path.starts_with(match_prefix.as_str()) {
                    return Some(rule.backend.clone());
                }
            }
        }

        None
    }

    /// Number of routes in the table.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::RouteDef;

    fn make_routes() -> Vec<RouteDef> {
        vec![
            RouteDef {
                match_host: Some("api.example.com".to_string()),
                match_prefix: None,
                backend: "api".to_string(),
                strip_prefix: false,
            },
            RouteDef {
                match_host: None,
                match_prefix: Some("/metrics".to_string()),
                backend: "metrics".to_string(),
                strip_prefix: true,
            },
            RouteDef {
                match_host: None,
                match_prefix: Some("/".to_string()),
                backend: "web".to_string(),
                strip_prefix: false,
            },
        ]
    }

    fn port_lookup(name: &str) -> Option<u16> {
        match name {
            "api" => Some(3030),
            "metrics" => Some(9090),
            "web" => Some(8080),
            _ => None,
        }
    }

    #[test]
    fn route_by_host() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let backend = table.route(Some("api.example.com"), "/anything");
        assert!(backend.is_some());
        let b = backend.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.name, "api");
        assert_eq!(b.addr, "127.0.0.1:3030");
    }

    #[test]
    fn route_by_host_with_port() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let backend = table.route(Some("api.example.com:443"), "/anything");
        assert!(backend.is_some());
        let b = backend.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.name, "api");
    }

    #[test]
    fn route_by_prefix() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let backend = table.route(Some("other.com"), "/metrics/something");
        assert!(backend.is_some());
        let b = backend.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.name, "metrics");
        assert!(b.strip_prefix);
    }

    #[test]
    fn route_catchall() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let backend = table.route(Some("other.com"), "/some/page");
        assert!(backend.is_some());
        let b = backend.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.name, "web");
    }

    #[test]
    fn no_route_matches_wrong_host_only() {
        // If only host routes exist and none match, returns None
        let routes = vec![RouteDef {
            match_host: Some("specific.com".to_string()),
            match_prefix: None,
            backend: "api".to_string(),
            strip_prefix: false,
        }];
        let table = RoutingTable::from_routes(&routes, port_lookup);
        let backend = table.route(Some("other.com"), "/");
        assert!(backend.is_none());
    }

    #[test]
    fn table_len() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        assert_eq!(table.len(), 3);
        assert!(!table.is_empty());
    }

    #[test]
    fn empty_routing_table() {
        let table = RoutingTable::from_routes(&[], port_lookup);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        assert!(table.route(Some("anything"), "/").is_none());
    }

    #[test]
    fn no_host_header_falls_to_prefix() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        // No host header — host rules skip, prefix rules match
        let backend = table.route(None, "/metrics/endpoint");
        assert!(backend.is_some());
        let b = backend.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.name, "metrics");
    }

    #[test]
    fn no_host_no_prefix_match() {
        let routes = vec![RouteDef {
            match_host: Some("specific.com".to_string()),
            match_prefix: None,
            backend: "api".to_string(),
            strip_prefix: false,
        }];
        let table = RoutingTable::from_routes(&routes, port_lookup);
        // No host header and only host rules → no match
        assert!(table.route(None, "/anything").is_none());
    }

    #[test]
    fn unknown_backend_filtered_out() {
        let routes = vec![RouteDef {
            match_host: None,
            match_prefix: Some("/".to_string()),
            backend: "nonexistent".to_string(),
            strip_prefix: false,
        }];
        let table = RoutingTable::from_routes(&routes, port_lookup);
        // port_lookup returns None for "nonexistent" → route filtered out
        assert!(table.is_empty());
    }

    #[test]
    fn host_match_is_case_sensitive() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        // Host matching is case-sensitive
        let backend = table.route(Some("API.EXAMPLE.COM"), "/");
        // Should not match "api.example.com"
        assert!(backend.is_some()); // Falls through to "/" catchall
        let b = backend.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.name, "web"); // Matched catchall, not api
    }

    #[test]
    fn prefix_exact_match() {
        let routes = vec![RouteDef {
            match_host: None,
            match_prefix: Some("/api".to_string()),
            backend: "api".to_string(),
            strip_prefix: false,
        }];
        let table = RoutingTable::from_routes(&routes, port_lookup);
        assert!(table.route(None, "/api").is_some());
        assert!(table.route(None, "/api/v1").is_some());
        assert!(table.route(None, "/other").is_none());
    }

    #[test]
    fn first_match_wins_ordering() {
        let routes = vec![
            RouteDef {
                match_host: None,
                match_prefix: Some("/api".to_string()),
                backend: "api".to_string(),
                strip_prefix: false,
            },
            RouteDef {
                match_host: None,
                match_prefix: Some("/".to_string()),
                backend: "web".to_string(),
                strip_prefix: false,
            },
        ];
        let table = RoutingTable::from_routes(&routes, port_lookup);
        let b = table.route(None, "/api/test");
        assert!(b.is_some());
        let backend = b.unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(backend.name, "api"); // first match wins
    }

    #[test]
    fn backend_addr_format() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let b = table
            .route(Some("api.example.com"), "/")
            .unwrap_or_else(|| panic!("expected backend"));
        assert_eq!(b.addr, "127.0.0.1:3030");
    }

    #[test]
    fn strip_prefix_flag_preserved() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let b = table
            .route(None, "/metrics/x")
            .unwrap_or_else(|| panic!("expected"));
        assert!(b.strip_prefix);
        assert_eq!(b.prefix.as_deref(), Some("/metrics"));
    }

    #[test]
    fn routing_table_clone() {
        let table = RoutingTable::from_routes(&make_routes(), port_lookup);
        let cloned = table.clone();
        assert_eq!(cloned.len(), table.len());
    }
}
