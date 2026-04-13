//! Minimal HTTP echo server for nexcloud integration testing.
//!
//! Responds with 200 OK on all paths. Health-check-friendly.
//! Usage: PORT=8080 nexcloud-echo

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn handle(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();
    let body = serde_json::json!({
        "status": "ok",
        "path": path,
        "method": req.method().to_string(),
    });
    let json = serde_json::to_string(&body).unwrap_or_else(|_| r#"{"status":"ok"}"#.to_string());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("ok")))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    eprintln!("nexcloud-echo listening on http://{addr}");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                if !e.to_string().contains("connection closed") {
                    eprintln!("connection error: {e}");
                }
            }
        });
    }
}
