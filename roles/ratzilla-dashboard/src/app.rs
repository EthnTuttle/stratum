//! Application state management

use crate::metrics_parser::{MetricsData, parse_prometheus_metrics};
use anyhow::Result;

pub struct App {
    pub metrics: MetricsData,
    pub metrics_url: String,
    pub error: Option<String>,
}

impl App {
    pub fn new(metrics_url: String) -> Self {
        Self {
            metrics: MetricsData::default(),
            metrics_url,
            error: None,
        }
    }

    /// Fetch metrics from Prometheus endpoint
    pub async fn fetch_metrics(&mut self) -> Result<()> {
        match reqwest::get(&self.metrics_url).await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => {
                        match parse_prometheus_metrics(&text) {
                            Ok(metrics) => {
                                self.metrics = metrics;
                                self.error = None;
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to parse metrics: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to read response: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to fetch metrics: {}", e));
            }
        }
        Ok(())
    }
}
