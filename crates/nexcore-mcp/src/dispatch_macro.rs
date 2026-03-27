//! `dispatch_table!` — declarative macro to replace hand-written match arms in unified.rs.
//!
//! Eliminates 1,361 manual match arms with a structured dispatch table.
//! Uses `@style` prefix tokens to distinguish dispatch modes inline.
//!
//! # Usage
//!
//! ```ignore
//! dispatch_table! {
//!     command, params, server;
//!     @sync "foundation_levenshtein" => tools::foundation::calc_levenshtein;
//!     @sync "foundation_sha256" => tools::foundation::sha256;
//!     @async "api_health" => tools::api::health;
//!     @server "nexcore_health" => unified_health();
//!     @raw "help" => help_catalog();
//! }
//! ```

/// Core dispatch table macro using `@style` prefixed entries.
///
/// Each entry is a semicolon-terminated line with a style prefix:
/// - `@sync` — wraps with `typed(params, handler)`
/// - `@async` — wraps with `typed_async(params, handler).await`
/// - `@server` — calls `server.method()`
/// - `@raw` — passes expression directly
macro_rules! dispatch_table {
    // Entry point: collect all arms via recursive tt munching
    (
        $command:ident, $params:ident, $server:ident;
        $($rest:tt)*
    ) => {
        dispatch_table!(@collect $command, $params, $server; [] $($rest)*)
    };

    // Base case: no more entries, emit the match
    (@collect $command:ident, $params:ident, $server:ident; [$($arms:tt)*]) => {
        match $command {
            $($arms)*
            _ => Err(McpError::invalid_params(
                format!("Unknown command: '{}'. Use 'help' for catalog.", $command),
                None,
            ))
        }
    };

    // @sync entry
    (@collect $command:ident, $params:ident, $server:ident;
        [$($arms:tt)*]
        @sync $name:literal => $handler:expr;
        $($rest:tt)*
    ) => {
        dispatch_table!(@collect $command, $params, $server;
            [$($arms)* $name => typed($params, $handler),]
            $($rest)*
        )
    };

    // @async entry
    (@collect $command:ident, $params:ident, $server:ident;
        [$($arms:tt)*]
        @async $name:literal => $handler:expr;
        $($rest:tt)*
    ) => {
        dispatch_table!(@collect $command, $params, $server;
            [$($arms)* $name => typed_async($params, $handler).await,]
            $($rest)*
        )
    };

    // @server entry
    (@collect $command:ident, $params:ident, $server:ident;
        [$($arms:tt)*]
        @server $name:literal => $method:ident ($($arg:tt)*);
        $($rest:tt)*
    ) => {
        dispatch_table!(@collect $command, $params, $server;
            [$($arms)* $name => $server.$method($($arg)*),]
            $($rest)*
        )
    };

    // @raw entry
    (@collect $command:ident, $params:ident, $server:ident;
        [$($arms:tt)*]
        @raw $name:literal => $handler:expr;
        $($rest:tt)*
    ) => {
        dispatch_table!(@collect $command, $params, $server;
            [$($arms)* $name => $handler,]
            $($rest)*
        )
    };
}

pub(crate) use dispatch_table;

#[cfg(test)]
mod tests {
    use super::dispatch_table;
    use rmcp::ErrorData as McpError;
    use rmcp::model::{CallToolResult, Content};
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Deserialize)]
    struct StubParams {
        #[allow(dead_code)]
        value: Option<String>,
    }

    fn stub_sync(_p: StubParams) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("ok".to_string())]))
    }

    async fn stub_async(_p: StubParams) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("ok".to_string())]))
    }

    fn typed<T, F>(params: Value, f: F) -> Result<CallToolResult, McpError>
    where
        T: serde::de::DeserializeOwned,
        F: FnOnce(T) -> Result<CallToolResult, McpError>,
    {
        let p: T = serde_json::from_value(params)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;
        f(p)
    }

    async fn typed_async<T, F, Fut>(params: Value, f: F) -> Result<CallToolResult, McpError>
    where
        T: serde::de::DeserializeOwned,
        F: FnOnce(T) -> Fut,
        Fut: std::future::Future<Output = Result<CallToolResult, McpError>>,
    {
        let p: T = serde_json::from_value(params)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;
        f(p).await
    }

    struct FakeServer;
    impl FakeServer {
        fn health(&self) -> Result<CallToolResult, McpError> {
            Ok(CallToolResult::success(vec![Content::text("healthy".to_string())]))
        }
    }

    fn help_catalog() -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("help".to_string())]))
    }

    #[tokio::test]
    async fn test_dispatch_sync() {
        let command = "test_sync";
        let params = serde_json::json!({});
        let server = FakeServer;

        let result: Result<CallToolResult, McpError> = dispatch_table! {
            command, params, server;
            @sync "test_sync" => stub_sync;
            @server "test_health" => health();
        };

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_async() {
        let command = "test_async";
        let params = serde_json::json!({});
        let server = FakeServer;

        let result: Result<CallToolResult, McpError> = dispatch_table! {
            command, params, server;
            @sync "test_sync" => stub_sync;
            @async "test_async" => stub_async;
            @server "test_health" => health();
        };

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_server() {
        let command = "test_health";
        let params = serde_json::json!({});
        let server = FakeServer;

        let result: Result<CallToolResult, McpError> = dispatch_table! {
            command, params, server;
            @server "test_health" => health();
        };

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_raw() {
        let command = "help";
        let params = serde_json::json!({});
        let server = FakeServer;

        let result: Result<CallToolResult, McpError> = dispatch_table! {
            command, params, server;
            @raw "help" => help_catalog();
            @sync "test_sync" => stub_sync;
        };

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_unknown() {
        let command = "nonexistent";
        let params = serde_json::json!({});
        let server = FakeServer;

        let result: Result<CallToolResult, McpError> = dispatch_table! {
            command, params, server;
            @sync "test_sync" => stub_sync;
        };

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dispatch_mixed_all_styles() {
        let command = "test_async";
        let params = serde_json::json!({});
        let server = FakeServer;

        let result: Result<CallToolResult, McpError> = dispatch_table! {
            command, params, server;
            @raw "help" => help_catalog();
            @sync "test_sync" => stub_sync;
            @async "test_async" => stub_async;
            @server "test_health" => health();
        };

        assert!(result.is_ok());
    }
}
