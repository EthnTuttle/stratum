//! Metrics integration for pool
//!
//! Implements the MetricsProvider trait to expose pool state as Prometheus metrics

use metrics_exporter::types::{
    ChannelMetrics, ChannelType, DownstreamMetrics, MetricsProvider, PoolMetricsSnapshot,
    unix_timestamp,
};

use crate::channel_manager::ChannelManager;

impl MetricsProvider for ChannelManager {
    type Snapshot = PoolMetricsSnapshot;

    fn get_metrics_snapshot(&self) -> Self::Snapshot {
        self.channel_manager_data
            .super_safe_lock(|channel_manager_data| {
                let mut downstreams = Vec::new();

                // Iterate over all downstream connections
                for (downstream_id, downstream) in &channel_manager_data.downstream {
                    let mut channels = Vec::new();

                    // Extract metrics from downstream channels
                    downstream
                        .downstream_data
                        .super_safe_lock(|downstream_data| {
                            // Standard channels
                            for (channel_id, standard_channel) in &downstream_data.standard_channels
                            {
                                let share_accounting = standard_channel.get_share_accounting();
                                channels.push(ChannelMetrics {
                                    channel_id: *channel_id,
                                    channel_type: ChannelType::Standard,
                                    shares_accepted: share_accounting.get_shares_accepted(),
                                    last_sequence_number: share_accounting
                                        .get_last_share_sequence_number(),
                                    share_work_sum: share_accounting.get_share_work_sum(),
                                    blocks_found: 0, // TODO: Track blocks found per channel
                                });
                            }

                            // Extended channels
                            for (channel_id, extended_channel) in &downstream_data.extended_channels
                            {
                                let share_accounting = extended_channel.get_share_accounting();
                                channels.push(ChannelMetrics {
                                    channel_id: *channel_id,
                                    channel_type: ChannelType::Extended,
                                    shares_accepted: share_accounting.get_shares_accepted(),
                                    last_sequence_number: share_accounting
                                        .get_last_share_sequence_number(),
                                    share_work_sum: share_accounting.get_share_work_sum(),
                                    blocks_found: 0, // TODO: Track blocks found per channel
                                });
                            }

                            // Group channel (if present)
                            // Note: GroupChannel doesn't have share accounting, skip for now
                            // TODO: Investigate if GroupChannels need separate metrics tracking
                            if downstream_data.group_channels.is_some() {
                                // Group channel exists, but we don't track its metrics yet
                            }
                        });

                    downstreams.push(DownstreamMetrics {
                        downstream_id: *downstream_id,
                        channels,
                    });
                }

                PoolMetricsSnapshot {
                    downstreams,
                    template_provider_connected: channel_manager_data
                        .last_future_template
                        .is_some(),
                    templates_received_total: 0, // TODO: Track template count
                    timestamp: unix_timestamp(),
                }
            })
    }
}
