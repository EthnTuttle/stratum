//! Prometheus metrics exporter
//!
//! Converts metrics snapshots into Prometheus format

use prometheus::{
    Encoder, GaugeVec, IntCounter, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder,
};
use std::sync::Arc;

use crate::types::{PoolMetricsSnapshot, TranslatorMetricsSnapshot};

/// Prometheus metrics exporter for SV2
pub struct PrometheusExporter {
    registry: Arc<Registry>,
    // Pool metrics
    pool_shares_accepted: IntGaugeVec,
    pool_share_work_sum: GaugeVec,
    pool_share_sequence_number: IntGaugeVec,
    pool_blocks_found: IntGaugeVec,
    pool_downstream_connections_active: IntGauge,
    pool_channels_active: IntGaugeVec,
    pool_template_provider_connected: IntGauge,
    pool_templates_received_total: IntCounter,
    // Translator metrics
    translator_upstream_connected: IntGaugeVec,
    translator_shares_submitted_upstream_total: IntCounter,
    translator_downstream_connections_active: IntGauge,
    translator_shares_received_downstream_total: IntCounter,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // Pool share accounting metrics
        let pool_shares_accepted = IntGaugeVec::new(
            Opts::new(
                "sv2_pool_shares_accepted_total",
                "Total accepted shares per downstream/channel",
            ),
            &["downstream_id", "channel_id", "channel_type"],
        )?;
        registry.register(Box::new(pool_shares_accepted.clone()))?;

        let pool_share_work_sum = GaugeVec::new(
            Opts::new(
                "sv2_pool_share_work_sum",
                "Cumulative share work (difficulty sum) per channel",
            ),
            &["downstream_id", "channel_id", "channel_type"],
        )?;
        registry.register(Box::new(pool_share_work_sum.clone()))?;

        let pool_share_sequence_number = IntGaugeVec::new(
            Opts::new(
                "sv2_pool_share_sequence_number",
                "Last share sequence number per channel",
            ),
            &["downstream_id", "channel_id", "channel_type"],
        )?;
        registry.register(Box::new(pool_share_sequence_number.clone()))?;

        let pool_blocks_found = IntGaugeVec::new(
            Opts::new(
                "sv2_pool_blocks_found_total",
                "Total blocks found per channel",
            ),
            &["downstream_id", "channel_id", "channel_type"],
        )?;
        registry.register(Box::new(pool_blocks_found.clone()))?;

        // Pool connection metrics
        let pool_downstream_connections_active = IntGauge::new(
            "sv2_pool_downstream_connections_active",
            "Number of active downstream connections",
        )?;
        registry.register(Box::new(pool_downstream_connections_active.clone()))?;

        let pool_channels_active = IntGaugeVec::new(
            Opts::new(
                "sv2_pool_channels_active",
                "Number of active channels by type",
            ),
            &["channel_type"],
        )?;
        registry.register(Box::new(pool_channels_active.clone()))?;

        // Pool template provider metrics
        let pool_template_provider_connected = IntGauge::new(
            "sv2_pool_template_provider_connected",
            "Template provider connection status (1 = connected, 0 = disconnected)",
        )?;
        registry.register(Box::new(pool_template_provider_connected.clone()))?;

        let pool_templates_received_total = IntCounter::new(
            "sv2_pool_templates_received_total",
            "Total templates received from template provider",
        )?;
        registry.register(Box::new(pool_templates_received_total.clone()))?;

        // Translator metrics
        let translator_upstream_connected = IntGaugeVec::new(
            Opts::new(
                "sv2_translator_upstream_connected",
                "Upstream pool connection status (1 = connected, 0 = disconnected)",
            ),
            &["pool_address"],
        )?;
        registry.register(Box::new(translator_upstream_connected.clone()))?;

        let translator_shares_submitted_upstream_total = IntCounter::new(
            "sv2_translator_shares_submitted_upstream_total",
            "Total shares submitted to upstream pool",
        )?;
        registry.register(Box::new(translator_shares_submitted_upstream_total.clone()))?;

        let translator_downstream_connections_active = IntGauge::new(
            "sv2_translator_downstream_connections_active",
            "Number of active downstream SV1 connections",
        )?;
        registry.register(Box::new(translator_downstream_connections_active.clone()))?;

        let translator_shares_received_downstream_total = IntCounter::new(
            "sv2_translator_shares_received_downstream_total",
            "Total shares received from downstream SV1 miners",
        )?;
        registry.register(Box::new(translator_shares_received_downstream_total.clone()))?;

        Ok(Self {
            registry,
            pool_shares_accepted,
            pool_share_work_sum,
            pool_share_sequence_number,
            pool_blocks_found,
            pool_downstream_connections_active,
            pool_channels_active,
            pool_template_provider_connected,
            pool_templates_received_total,
            translator_upstream_connected,
            translator_shares_submitted_upstream_total,
            translator_downstream_connections_active,
            translator_shares_received_downstream_total,
        })
    }

    /// Update metrics from a pool snapshot
    pub fn update_from_pool_snapshot(&self, snapshot: &PoolMetricsSnapshot) {
        // Reset gauges before updating
        self.pool_shares_accepted.reset();
        self.pool_share_work_sum.reset();
        self.pool_share_sequence_number.reset();
        self.pool_blocks_found.reset();
        self.pool_channels_active.reset();

        // Count active connections
        self.pool_downstream_connections_active
            .set(snapshot.downstreams.len() as i64);

        // Channel type counters
        let mut standard_count = 0;
        let mut extended_count = 0;
        let mut group_count = 0;

        // Per-channel metrics
        for downstream in &snapshot.downstreams {
            let downstream_id = downstream.downstream_id.to_string();

            for channel in &downstream.channels {
                let channel_id = channel.channel_id.to_string();
                let channel_type = channel.channel_type.as_label();

                // Share accounting
                self.pool_shares_accepted
                    .with_label_values(&[&downstream_id, &channel_id, channel_type])
                    .set(channel.shares_accepted as i64);

                self.pool_share_work_sum
                    .with_label_values(&[&downstream_id, &channel_id, channel_type])
                    .set(channel.share_work_sum);

                self.pool_share_sequence_number
                    .with_label_values(&[&downstream_id, &channel_id, channel_type])
                    .set(channel.last_sequence_number as i64);

                self.pool_blocks_found
                    .with_label_values(&[&downstream_id, &channel_id, channel_type])
                    .set(channel.blocks_found as i64);

                // Count by type
                match channel.channel_type {
                    crate::types::ChannelType::Standard => standard_count += 1,
                    crate::types::ChannelType::Extended => extended_count += 1,
                    crate::types::ChannelType::Group => group_count += 1,
                }
            }
        }

        // Update channel counts by type
        self.pool_channels_active
            .with_label_values(&["standard"])
            .set(standard_count);
        self.pool_channels_active
            .with_label_values(&["extended"])
            .set(extended_count);
        self.pool_channels_active
            .with_label_values(&["group"])
            .set(group_count);

        // Template provider status
        self.pool_template_provider_connected
            .set(if snapshot.template_provider_connected { 1 } else { 0 });
    }

    /// Update metrics from a translator snapshot
    pub fn update_from_translator_snapshot(&self, snapshot: &TranslatorMetricsSnapshot) {
        // Reset upstream connection gauge
        self.translator_upstream_connected.reset();

        // Upstream connection status
        if let Some(addr) = &snapshot.upstream_address {
            self.translator_upstream_connected
                .with_label_values(&[addr])
                .set(if snapshot.upstream_connected { 1 } else { 0 });
        }

        // Downstream connections
        self.translator_downstream_connections_active
            .set(snapshot.downstream_connections_active as i64);
    }

    /// Render metrics in Prometheus text format
    pub fn render(&self) -> Result<Vec<u8>, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(buffer)
    }

    /// Get the Prometheus registry
    pub fn registry(&self) -> Arc<Registry> {
        Arc::clone(&self.registry)
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new().expect("Failed to create PrometheusExporter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChannelMetrics, ChannelType, DownstreamMetrics};

    #[test]
    fn test_exporter_creation() {
        let exporter = PrometheusExporter::new();
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_pool_snapshot_update() {
        let exporter = PrometheusExporter::new().unwrap();

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

        exporter.update_from_pool_snapshot(&snapshot);

        let output = exporter.render().unwrap();
        let output_str = String::from_utf8_lossy(&output);

        // Verify metrics are present
        assert!(output_str.contains("sv2_pool_shares_accepted_total"));
        assert!(output_str.contains("sv2_pool_share_work_sum"));
        assert!(output_str.contains("sv2_pool_downstream_connections_active 1"));
    }

    #[test]
    fn test_translator_snapshot_update() {
        let exporter = PrometheusExporter::new().unwrap();

        let snapshot = TranslatorMetricsSnapshot {
            upstream_connected: true,
            upstream_address: Some("pool.example.com:3333".to_string()),
            shares_submitted_upstream_total: 100,
            downstream_connections_active: 5,
            shares_received_downstream_total: 100,
            timestamp: 1234567890,
        };

        exporter.update_from_translator_snapshot(&snapshot);

        let output = exporter.render().unwrap();
        let output_str = String::from_utf8_lossy(&output);

        // Verify metrics are present
        assert!(output_str.contains("sv2_translator_upstream_connected"));
        assert!(output_str.contains("sv2_translator_downstream_connections_active 5"));
    }
}
