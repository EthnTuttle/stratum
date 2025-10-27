//! HTTP server for exposing Prometheus metrics
//!
//! Provides a simple HTTP server that serves metrics at /metrics endpoint

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use tokio::net::TcpListener;
use tracing::{error, info};
use bytes::Bytes;
use std::sync::Arc;

use crate::metrics::PrometheusExporter;

/// HTTP server for exposing Prometheus metrics
pub struct MetricsServer {
    listen_address: String,
    exporter: Arc<PrometheusExporter>,
}

impl MetricsServer {
    /// Create a new metrics server
    ///
    /// # Arguments
    /// * `listen_address` - Address to bind the HTTP server (e.g., "127.0.0.1:9090")
    pub fn new(listen_address: String) -> Self {
        let exporter = Arc::new(PrometheusExporter::new().expect("Failed to create exporter"));
        Self {
            listen_address,
            exporter,
        }
    }

    /// Create a new metrics server with a custom exporter
    pub fn with_exporter(listen_address: String, exporter: PrometheusExporter) -> Self {
        Self {
            listen_address,
            exporter: Arc::new(exporter),
        }
    }

    /// Get a reference to the exporter
    pub fn exporter(&self) -> Arc<PrometheusExporter> {
        Arc::clone(&self.exporter)
    }

    /// Start the HTTP server
    ///
    /// This will block the current task and serve metrics on the configured address.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.listen_address).await?;
        info!("📊 Prometheus metrics server listening on http://{}/metrics", self.listen_address);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let exporter = Arc::clone(&self.exporter);

            tokio::task::spawn(async move {
                let service = service_fn(move |req| {
                    let exporter = Arc::clone(&exporter);
                    async move { handle_request(req, exporter).await }
                });

                if let Err(err) = http1::Builder::new()
                    .keep_alive(true)
                    .serve_connection(io, service)
                    .await
                {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn handle_request(
    req: Request<Incoming>,
    exporter: Arc<PrometheusExporter>,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            match exporter.render() {
                Ok(metrics) => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/plain; version=0.0.4")
                        .body(Full::new(Bytes::from(metrics)))
                        .unwrap_or_else(|_| {
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Full::new(Bytes::from("Failed to build response")))
                                .unwrap()
                        })
                }
                Err(e) => {
                    error!("Failed to render metrics: {:?}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Failed to render metrics")))
                        .unwrap()
                }
            }
        }
        (&Method::GET, "/") | (&Method::GET, "/health") => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(
                    r#"<html>
<head><title>Metrics Exporter</title></head>
<body>
<h1>SV2 Metrics Exporter</h1>
<p><a href="/metrics">Metrics</a></p>
</body>
</html>"#,
                )))
                .unwrap()
        }
        _ => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap()
        }
    };
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = MetricsServer::new("127.0.0.1:9090".to_string());
        assert_eq!(server.listen_address, "127.0.0.1:9090");
    }

    #[tokio::test]
    async fn test_server_with_custom_exporter() {
        let exporter = PrometheusExporter::new().unwrap();
        let server = MetricsServer::with_exporter("127.0.0.1:9091".to_string(), exporter);

        // Verify we can get a reference to the exporter
        let _ = server.exporter();
    }
}
