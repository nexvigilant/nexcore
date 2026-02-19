//! Runtime configuration helpers for Nucleus.

const DEFAULT_NEXCORE_API_URL: &str = "http://localhost:3030";

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn normalize_base_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return DEFAULT_NEXCORE_API_URL.to_string();
    }
    trimmed.trim_end_matches('/').to_string()
}

/// Resolve NexCore API base URL from environment with safe defaults.
pub fn nexcore_api_url() -> String {
    let raw = non_empty_env("NEXCORE_API_URL").unwrap_or_else(|| DEFAULT_NEXCORE_API_URL.to_string());
    normalize_base_url(&raw)
}

/// Resolve optional NexCore API key from environment.
pub fn nexcore_api_key() -> Option<String> {
    non_empty_env("NEXCORE_API_KEY")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_base_url_trims_trailing_slash() {
        assert_eq!(normalize_base_url("http://localhost:3030/"), "http://localhost:3030");
        assert_eq!(normalize_base_url("https://api.example.com///"), "https://api.example.com");
    }

    #[test]
    fn normalize_base_url_uses_default_for_empty() {
        assert_eq!(normalize_base_url(""), DEFAULT_NEXCORE_API_URL);
        assert_eq!(normalize_base_url("   "), DEFAULT_NEXCORE_API_URL);
    }
}
