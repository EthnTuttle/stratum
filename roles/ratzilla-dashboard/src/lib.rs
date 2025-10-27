//! Ratzilla Dashboard for SV2 Metrics
//!
//! Terminal-style web dashboard for visualizing Stratum V2 mining metrics

pub mod metrics_parser;
pub mod app;
pub mod ui;

pub use app::App;
pub use metrics_parser::MetricsData;
