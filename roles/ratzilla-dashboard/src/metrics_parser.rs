//! Prometheus metrics parser
//!
//! Parses Prometheus text format from /metrics endpoint

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub share_metrics: Vec<ChannelShareMetrics>,
    pub connection_metrics: ConnectionMetrics,
    pub last_update: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelShareMetrics {
    pub downstream_id: u32,
    pub channel_id: u32,
    pub channel_type: String,
    pub shares_accepted: u64,
    pub share_work_sum: f64,
    pub sequence_number: u64,
    pub blocks_found: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub downstream_connections_active: u64,
    pub channels_by_type: HashMap<String, u64>,
    pub template_provider_connected: bool,
}

impl Default for MetricsData {
    fn default() -> Self {
        Self {
            share_metrics: Vec::new(),
            connection_metrics: ConnectionMetrics {
                downstream_connections_active: 0,
                channels_by_type: HashMap::new(),
                template_provider_connected: false,
            },
            last_update: 0,
        }
    }
}

/// Parse Prometheus text format
pub fn parse_prometheus_metrics(text: &str) -> anyhow::Result<MetricsData> {
    let mut share_metrics: HashMap<(u32, u32), ChannelShareMetrics> = HashMap::new();
    let mut connection_metrics = ConnectionMetrics {
        downstream_connections_active: 0,
        channels_by_type: HashMap::new(),
        template_provider_connected: false,
    };

    for line in text.lines() {
        // Skip comments and help text
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Parse metric line
        if let Some((metric_name, rest)) = line.split_once('{') {
            // Extract labels and value
            if let Some((labels_str, value_str)) = rest.split_once('}') {
                let value = value_str.trim().parse::<f64>().unwrap_or(0.0);
                let labels = parse_labels(labels_str);

                match metric_name.trim() {
                    "sv2_pool_shares_accepted_total" => {
                        if let (Some(downstream_id), Some(channel_id), Some(channel_type)) = (
                            labels.get("downstream_id"),
                            labels.get("channel_id"),
                            labels.get("channel_type"),
                        ) {
                            let key = (
                                downstream_id.parse().unwrap_or(0),
                                channel_id.parse().unwrap_or(0),
                            );
                            share_metrics
                                .entry(key)
                                .or_insert_with(|| ChannelShareMetrics {
                                    downstream_id: key.0,
                                    channel_id: key.1,
                                    channel_type: channel_type.to_string(),
                                    shares_accepted: 0,
                                    share_work_sum: 0.0,
                                    sequence_number: 0,
                                    blocks_found: 0,
                                })
                                .shares_accepted = value as u64;
                        }
                    }
                    "sv2_pool_share_work_sum" => {
                        if let (Some(downstream_id), Some(channel_id)) =
                            (labels.get("downstream_id"), labels.get("channel_id"))
                        {
                            let key = (
                                downstream_id.parse().unwrap_or(0),
                                channel_id.parse().unwrap_or(0),
                            );
                            if let Some(metrics) = share_metrics.get_mut(&key) {
                                metrics.share_work_sum = value;
                            }
                        }
                    }
                    "sv2_pool_share_sequence_number" => {
                        if let (Some(downstream_id), Some(channel_id)) =
                            (labels.get("downstream_id"), labels.get("channel_id"))
                        {
                            let key = (
                                downstream_id.parse().unwrap_or(0),
                                channel_id.parse().unwrap_or(0),
                            );
                            if let Some(metrics) = share_metrics.get_mut(&key) {
                                metrics.sequence_number = value as u64;
                            }
                        }
                    }
                    "sv2_pool_blocks_found_total" => {
                        if let (Some(downstream_id), Some(channel_id)) =
                            (labels.get("downstream_id"), labels.get("channel_id"))
                        {
                            let key = (
                                downstream_id.parse().unwrap_or(0),
                                channel_id.parse().unwrap_or(0),
                            );
                            if let Some(metrics) = share_metrics.get_mut(&key) {
                                metrics.blocks_found = value as u64;
                            }
                        }
                    }
                    "sv2_pool_channels_active" => {
                        if let Some(channel_type) = labels.get("channel_type") {
                            connection_metrics
                                .channels_by_type
                                .insert(channel_type.to_string(), value as u64);
                        }
                    }
                    _ => {}
                }
            }
        } else if line.contains("sv2_pool_downstream_connections_active") {
            // Metric without labels
            if let Some(value_str) = line.split_whitespace().last() {
                connection_metrics.downstream_connections_active =
                    value_str.parse().unwrap_or(0);
            }
        } else if line.contains("sv2_pool_template_provider_connected") {
            if let Some(value_str) = line.split_whitespace().last() {
                connection_metrics.template_provider_connected =
                    value_str.parse::<i32>().unwrap_or(0) == 1;
            }
        }
    }

    Ok(MetricsData {
        share_metrics: share_metrics.into_values().collect(),
        connection_metrics,
        last_update: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

fn parse_labels(labels_str: &str) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    for part in labels_str.split(',') {
        if let Some((key, value)) = part.split_once('=') {
            let value = value.trim_matches('"');
            labels.insert(key.trim().to_string(), value.to_string());
        }
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metrics() {
        let prometheus_text = r#"
# HELP sv2_pool_shares_accepted_total Total accepted shares
# TYPE sv2_pool_shares_accepted_total gauge
sv2_pool_shares_accepted_total{downstream_id="1",channel_id="1",channel_type="standard"} 42
sv2_pool_share_work_sum{downstream_id="1",channel_id="1",channel_type="standard"} 150000.5
sv2_pool_downstream_connections_active 3
sv2_pool_channels_active{channel_type="standard"} 5
sv2_pool_template_provider_connected 1
"#;

        let metrics = parse_prometheus_metrics(prometheus_text).unwrap();
        assert_eq!(metrics.share_metrics.len(), 1);
        assert_eq!(metrics.share_metrics[0].shares_accepted, 42);
        assert_eq!(metrics.connection_metrics.downstream_connections_active, 3);
    }
}
