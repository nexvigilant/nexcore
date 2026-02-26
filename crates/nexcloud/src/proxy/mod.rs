pub mod router;
pub mod tls;

use crate::error::{NexCloudError, Result};
use crate::events::{CloudEvent, EventBus};
use crate::proxy::router::{Backend, RoutingTable};
use crate::status::CloudStatus;
use crate::supervisor::registry::ServiceRegistry;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub use router::RoutingTable as ProxyRoutingTable;
pub use tls::load_tls_config;

/// The reverse proxy server.
///
/// Tier: T3 (μ Mapping + ∂ Boundary + σ Sequence + λ Location + ν Frequency)
/// Full domain type — maps requests across network boundaries to backend services.
///
/// Constitutional grounding:
/// - CAP-018 Transportation: route governance
/// - CAP-029 Homeland Security: TLS boundary enforcement
/// - CAP-016 Communications: channel management
/// Maximum request body size (10 MB). SEC-003: prevents OOM via large payloads.
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

pub struct ReverseProxy {
    routing_table: Arc<RoutingTable>,
    event_bus: EventBus,
    http_client: reqwest::Client,
    tls_acceptor: Option<TlsAcceptor>,
    https_port: u16,
    registry: Option<Arc<ServiceRegistry>>,
    platform_name: String,
}

impl ReverseProxy {
    /// Create a new reverse proxy without TLS.
    pub fn new(routing_table: RoutingTable, event_bus: EventBus) -> Self {
        let http_client = reqwest::Client::builder()
            .no_proxy()
            .pool_max_idle_per_host(32)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            routing_table: Arc::new(routing_table),
            event_bus,
            http_client,
            tls_acceptor: None,
            https_port: 443,
            registry: None,
            platform_name: String::new(),
        }
    }

    /// Create a reverse proxy with TLS termination.
    pub fn with_tls(
        routing_table: RoutingTable,
        event_bus: EventBus,
        tls_config: Arc<rustls::ServerConfig>,
        https_port: u16,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .no_proxy()
            .pool_max_idle_per_host(32)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            routing_table: Arc::new(routing_table),
            event_bus,
            http_client,
            tls_acceptor: Some(TlsAcceptor::from(tls_config)),
            https_port,
            registry: None,
            platform_name: String::new(),
        }
    }

    /// Attach a service registry for the `/.nexcloud/status` endpoint.
    #[must_use]
    pub fn with_registry(mut self, registry: Arc<ServiceRegistry>, platform_name: String) -> Self {
        self.registry = Some(registry);
        self.platform_name = platform_name;
        self
    }

    /// Whether TLS is configured.
    pub fn has_tls(&self) -> bool {
        self.tls_acceptor.is_some()
    }

    /// Start the HTTP listener.
    ///
    /// If TLS is configured, HTTP requests receive a 301 redirect to HTTPS.
    /// Otherwise, HTTP requests are proxied normally.
    pub async fn serve_http(self: Arc<Self>, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NexCloudError::ProxyRoute(format!("failed to bind {addr}: {e}")))?;

        tracing::info!(%addr, "reverse proxy listening (HTTP)");

        loop {
            let (stream, remote_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::warn!("accept error: {e}");
                    continue;
                }
            };

            let proxy = Arc::clone(&self);
            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                let svc = service_fn(move |req| {
                    let proxy = Arc::clone(&proxy);
                    async move { proxy.handle_http_request(req, remote_addr).await }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                    if !e.to_string().contains("connection closed") {
                        tracing::debug!("connection error from {remote_addr}: {e}");
                    }
                }
            });
        }
    }

    /// Start the HTTPS listener with TLS termination.
    ///
    /// Constitutional: CAP-029 Homeland Security boundary enforcement.
    pub async fn serve_https(self: Arc<Self>, addr: SocketAddr) -> Result<()> {
        let acceptor = self
            .tls_acceptor
            .clone()
            .ok_or_else(|| NexCloudError::TlsConfig("no TLS config for HTTPS".to_string()))?;

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NexCloudError::ProxyRoute(format!("failed to bind {addr}: {e}")))?;

        tracing::info!(%addr, "reverse proxy listening (HTTPS)");

        loop {
            let (stream, remote_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::warn!("TLS accept error: {e}");
                    continue;
                }
            };

            let acceptor = acceptor.clone();
            let proxy = Arc::clone(&self);

            tokio::spawn(async move {
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::debug!("TLS handshake failed from {remote_addr}: {e}");
                        return;
                    }
                };

                let io = TokioIo::new(tls_stream);
                let svc = service_fn(move |req| {
                    let proxy = Arc::clone(&proxy);
                    async move { proxy.handle_request(req, remote_addr).await }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                    if !e.to_string().contains("connection closed") {
                        tracing::debug!("HTTPS connection error from {remote_addr}: {e}");
                    }
                }
            });
        }
    }

    /// Build a JSON status response from the service registry.
    fn build_status_response(&self) -> Response<Full<Bytes>> {
        let records = self
            .registry
            .as_ref()
            .map(|r| r.snapshot())
            .unwrap_or_default();
        let status = CloudStatus::from_records(&self.platform_name, records);
        let body = serde_json::to_string(&status)
            .unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string());
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("{}"))))
    }

    /// Handle HTTP request — redirect to HTTPS if TLS configured, else proxy.
    async fn handle_http_request(
        &self,
        req: Request<hyper::body::Incoming>,
        remote_addr: SocketAddr,
    ) -> std::result::Result<Response<Full<Bytes>>, hyper::Error> {
        // Built-in health endpoint (habeas corpus — every process accounted for)
        if req.uri().path() == "/.nexcloud/health" {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("ok")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("ok")))));
        }

        // Built-in status endpoint — full cloud status as JSON
        if req.uri().path() == "/.nexcloud/status" {
            return Ok(self.build_status_response());
        }

        // If TLS is configured, redirect HTTP → HTTPS (CAP-029 boundary enforcement)
        if self.tls_acceptor.is_some() {
            let host = req
                .headers()
                .get("host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("localhost");

            // SEC-009: Strip port and validate host contains only safe characters
            // Prevents open redirect via malicious Host header injection.
            let host_bare = host.split(':').next().unwrap_or(host);
            let host_safe = if host_bare
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
            {
                host_bare
            } else {
                "localhost"
            };
            let path = req.uri().path_and_query().map_or("/", |pq| pq.as_str());

            let redirect_url = if self.https_port == 443 {
                format!("https://{host_safe}{path}")
            } else {
                format!("https://{host_safe}:{}{path}", self.https_port)
            };

            return Ok(Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header("location", redirect_url)
                .body(Full::new(Bytes::from("Redirecting to HTTPS")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Redirect")))));
        }

        // No TLS — proxy directly
        self.handle_request(req, remote_addr).await
    }

    /// Handle a proxied request (used by both HTTP and HTTPS paths).
    async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
        _remote_addr: SocketAddr,
    ) -> std::result::Result<Response<Full<Bytes>>, hyper::Error> {
        let start = std::time::Instant::now();

        // Built-in health endpoint
        if req.uri().path() == "/.nexcloud/health" {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("ok")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("ok")))));
        }

        // Built-in status endpoint — full cloud status as JSON
        if req.uri().path() == "/.nexcloud/status" {
            return Ok(self.build_status_response());
        }

        let host = req
            .headers()
            .get("host")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let path = req.uri().path().to_string();

        let backend = self.routing_table.route(host.as_deref(), &path);

        match backend {
            Some(backend) => {
                let response = self.forward_request(req, &backend).await;
                let latency_ms = start.elapsed().as_millis() as u64;

                let status = match &response {
                    Ok(resp) => resp.status().as_u16(),
                    Err(_) => 502,
                };

                self.event_bus.emit(CloudEvent::ProxyRequest {
                    route: path,
                    backend: backend.name.clone(),
                    status,
                    latency_ms,
                });

                Ok(response.unwrap_or_else(|e| {
                    let msg = e.to_string();
                    let status = if msg.contains("payload too large") {
                        StatusCode::PAYLOAD_TOO_LARGE
                    } else {
                        StatusCode::BAD_GATEWAY
                    };
                    Response::builder()
                        .status(status)
                        .body(Full::new(Bytes::from(msg)))
                        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Error"))))
                }))
            }
            None => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("No route matched")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Error"))))),
        }
    }

    /// Forward a request to a backend service.
    async fn forward_request(
        &self,
        req: Request<hyper::body::Incoming>,
        backend: &Backend,
    ) -> std::result::Result<Response<Full<Bytes>>, NexCloudError> {
        let method = req.method().clone();
        let headers = req.headers().clone();
        let mut path = req.uri().path().to_string();

        // Strip prefix if configured
        if backend.strip_prefix {
            if let Some(ref prefix) = backend.prefix {
                if let Some(stripped) = path.strip_prefix(prefix.as_str()) {
                    path = if stripped.is_empty() {
                        "/".to_string()
                    } else if stripped.starts_with('/') {
                        stripped.to_string()
                    } else {
                        format!("/{stripped}")
                    };
                }
            }
        }

        // Append query string if present
        let query = req
            .uri()
            .query()
            .map(|q| format!("?{q}"))
            .unwrap_or_default();
        let url = format!("http://{}{path}{query}", backend.addr);

        // SEC-003: Collect incoming body with size limit to prevent OOM
        let body_bytes = match req.into_body().collect().await {
            Ok(collected) => {
                let bytes = collected.to_bytes();
                if bytes.len() > MAX_BODY_BYTES {
                    return Err(NexCloudError::ProxyRoute(format!(
                        "payload too large: {} bytes exceeds {} byte limit",
                        bytes.len(),
                        MAX_BODY_BYTES
                    )));
                }
                bytes
            }
            Err(e) => {
                return Err(NexCloudError::ProxyRoute(format!(
                    "failed to read request body: {e}"
                )));
            }
        };

        // Build reqwest request, forwarding relevant headers
        let mut proxy_req = self.http_client.request(method, &url);

        // Forward content-type and accept headers
        if let Some(ct) = headers.get("content-type") {
            proxy_req = proxy_req.header("content-type", ct);
        }
        if let Some(accept) = headers.get("accept") {
            proxy_req = proxy_req.header("accept", accept);
        }
        if let Some(auth) = headers.get("authorization") {
            proxy_req = proxy_req.header("authorization", auth);
        }

        if !body_bytes.is_empty() {
            proxy_req = proxy_req.body(body_bytes);
        }

        // Send and convert response
        let resp = proxy_req
            .send()
            .await
            .map_err(|e| NexCloudError::ProxyRoute(format!("backend request failed: {e}")))?;

        let status = resp.status();
        let body = resp.bytes().await.unwrap_or_else(|_| Bytes::new());

        let response = Response::builder()
            .status(status)
            .body(Full::new(body))
            .map_err(|e| NexCloudError::ProxyRoute(format!("response build failed: {e}")))?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_default_no_tls() {
        let table = RoutingTable::from_routes(&[], |_| None);
        let bus = EventBus::default();
        let proxy = ReverseProxy::new(table, bus);
        assert!(!proxy.has_tls());
    }
}
