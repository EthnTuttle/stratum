//! # Metrics Exporter for SV2
//!
//! Provides Prometheus metrics collection and export for SV2 pool and translator roles.
//!
//! ## Architecture
//!
//! This crate uses a trait-based approach to collect metrics from SV2 components:
//! - Roles implement `MetricsProvider` to expose their current state
//! - `PrometheusExporter` converts snapshots to Prometheus format
//! - HTTP server exposes `/metrics` endpoint for scraping
//!
//! ## Usage
//!
//! ```rust,no_run
//! use metrics_exporter::{MetricsServer, PoolMetricsSnapshot};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Start metrics server
//!     let server = MetricsServer::new("127.0.0.1:9090".to_string());
//!
//!     // Spawn server in background
//!     tokio::spawn(async move {
//!         server.start().await.unwrap();
//!     });
//! }
//! ```

pub mod metrics;
pub mod server;
pub mod types;

pub use metrics::PrometheusExporter;
pub use server::MetricsServer;
pub use types::{
    ChannelMetrics, DownstreamMetrics, MetricsProvider, PoolMetricsSnapshot,
    TranslatorMetricsSnapshot,
};
