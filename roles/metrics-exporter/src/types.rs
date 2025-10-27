//! Type definitions for metrics snapshots
//!
//! Defines the trait and data structures for collecting metrics from SV2 roles.

use serde::{Deserialize, Serialize};

/// Trait for components that can provide metrics snapshots
pub trait MetricsProvider {
    /// The type of snapshot this provider produces
    type Snapshot: Serialize + for<'de> Deserialize<'de>;

    /// Get the current metrics snapshot
    fn get_metrics_snapshot(&self) -> Self::Snapshot;
}

/// Metrics snapshot for a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetricsSnapshot {
    /// Metrics for each downstream connection
    pub downstreams: Vec<DownstreamMetrics>,
    /// Template provider connection status
    pub template_provider_connected: bool,
    /// Total templates received from template provider
    pub templates_received_total: u64,
    /// Timestamp when snapshot was taken
    pub timestamp: u64,
}

/// Metrics for a single downstream connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownstreamMetrics {
    /// Unique downstream identifier
    pub downstream_id: u32,
    /// Channels for this downstream
    pub channels: Vec<ChannelMetrics>,
}

/// Metrics for a single channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetrics {
    /// Channel ID
    pub channel_id: u32,
    /// Channel type
    pub channel_type: ChannelType,
    /// Total shares accepted on this channel
    pub shares_accepted: u32,
    /// Last share sequence number
    pub last_sequence_number: u32,
    /// Cumulative share work (difficulty sum)
    pub share_work_sum: f64,
    /// Blocks found on this channel
    pub blocks_found: u32,
}

/// Type of SV2 channel
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// Standard mining channel
    Standard,
    /// Extended mining channel (with custom work selection)
    Extended,
    /// Group channel
    Group,
}

impl ChannelType {
    /// Convert to Prometheus label value
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Extended => "extended",
            Self::Group => "group",
        }
    }
}

/// Metrics snapshot for a translator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorMetricsSnapshot {
    /// Upstream pool connection status
    pub upstream_connected: bool,
    /// Upstream pool address
    pub upstream_address: Option<String>,
    /// Total shares submitted to upstream
    pub shares_submitted_upstream_total: u64,
    /// Number of active downstream SV1 connections
    pub downstream_connections_active: u32,
    /// Total shares received from downstreams
    pub shares_received_downstream_total: u64,
    /// Timestamp when snapshot was taken
    pub timestamp: u64,
}

/// Get current Unix timestamp in seconds
pub fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_labels() {
        assert_eq!(ChannelType::Standard.as_label(), "standard");
        assert_eq!(ChannelType::Extended.as_label(), "extended");
        assert_eq!(ChannelType::Group.as_label(), "group");
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = PoolMetricsSnapshot {
            downstreams: vec![DownstreamMetrics {
                downstream_id: 1,
                channels: vec![ChannelMetrics {
                    channel_id: 1,
                    channel_type: ChannelType::Standard,
                    shares_accepted: 42,
                    last_sequence_number: 42,
                    share_work_sum: 150000.5,
                    blocks_found: 1,
                }],
            }],
            template_provider_connected: true,
            templates_received_total: 100,
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: PoolMetricsSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.downstreams.len(), 1);
        assert_eq!(deserialized.templates_received_total, 100);
    }
}
