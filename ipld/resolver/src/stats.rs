// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use lazy_static::lazy_static;
use prometheus::{Histogram, HistogramOpts, IntCounter, IntGauge, Registry};

lazy_static! {
    pub static ref PING_RTT: Histogram =
        Histogram::with_opts(HistogramOpts::new("ping_rtt", "Ping roundtrip time")).unwrap();
    pub static ref PING_TIMEOUT: IntCounter =
        IntCounter::new("ping_timeouts", "Number of timed out pings").unwrap();
    pub static ref PING_FAILURE: IntCounter =
        IntCounter::new("ping_failure", "Number of failed pings").unwrap();
    pub static ref PING_SUCCESS: IntCounter =
        IntCounter::new("ping_success", "Number of successful pings",).unwrap();
    pub static ref IDENTIFY_FAILURE: IntCounter =
        IntCounter::new("identify_failure", "Number of Identify errors",).unwrap();
    pub static ref IDENTIFY_RECEIVED: IntCounter =
        IntCounter::new("identify_received", "Number of Identify infos received",).unwrap();
    pub static ref DISCOVERY_BACKGROUND_LOOKUP: IntCounter = IntCounter::new(
        "discovery_background_lookup",
        "Number of background lookups started",
    )
    .unwrap();
    pub static ref DISCOVERY_CONNECTED_PEERS: IntGauge =
        IntGauge::new("discovery_connected_peers", "Number of connections",).unwrap();
    pub static ref MEMBERSHIP_SKIPPED_PEERS: IntCounter =
        IntCounter::new("membership_skipped_peers", "Number of providers skipped",).unwrap();
    pub static ref MEMBERSHIP_ROUTABLE_PEERS: IntGauge =
        IntGauge::new("membership_routable_peers", "Number of routable peers").unwrap();
    pub static ref MEMBERSHIP_PROVIDER_PEERS: IntGauge =
        IntGauge::new("membership_provider_peers", "Number of unique providers").unwrap();
    pub static ref MEMBERSHIP_UNKNOWN_TOPIC: IntCounter = IntCounter::new(
        "membership_unknown_topic",
        "Number of messages with unknown topic"
    )
    .unwrap();
    pub static ref MEMBERSHIP_INVALID_MESSAGE: IntCounter = IntCounter::new(
        "membership_invalid_message",
        "Number of invalid messages received"
    )
    .unwrap();
    pub static ref MEMBERSHIP_PUBLISH_SUCCESS: IntCounter =
        IntCounter::new("membership_publish_total", "Number of published messages").unwrap();
    pub static ref MEMBERSHIP_PUBLISH_FAILURE: IntCounter = IntCounter::new(
        "membership_publish_failure",
        "Number of failed publish attempts"
    )
    .unwrap();
    pub static ref CONTENT_RESOLVE_RUNNING: IntGauge = IntGauge::new(
        "content_resolve_running",
        "Number of currently running content resolutions"
    )
    .unwrap();
    pub static ref CONTENT_RESOLVE_NO_PEERS: IntCounter = IntCounter::new(
        "content_resolve_no_peers",
        "Number of resolutions with no known peers"
    )
    .unwrap();
    pub static ref CONTENT_RESOLVE_SUCCESS: IntCounter = IntCounter::new(
        "content_resolve_success",
        "Number of successful resolutions"
    )
    .unwrap();
    pub static ref CONTENT_RESOLVE_FAILURE: IntCounter =
        IntCounter::new("content_resolve_success", "Number of failed resolutions").unwrap();
    pub static ref CONTENT_RESOLVE_FALLBACK: IntCounter = IntCounter::new(
        "content_resolve_fallback",
        "Number of resolutions that fall back on secondary peers"
    )
    .unwrap();
    pub static ref CONTENT_RESOLVE_PEERS: Histogram = Histogram::with_opts(HistogramOpts::new(
        "content_resolve_peers",
        "Number of peers found for resolution from a subnet"
    ))
    .unwrap();
    pub static ref CONTENT_CONNECTED_PEERS: Histogram = Histogram::with_opts(HistogramOpts::new(
        "content_connected_peers",
        "Number of connected peers in a resolution"
    ))
    .unwrap();
}

pub fn register_metrics(registry: &Registry) -> anyhow::Result<()> {
    registry.register(Box::new(PING_RTT.clone()))?;
    registry.register(Box::new(PING_TIMEOUT.clone()))?;
    registry.register(Box::new(PING_FAILURE.clone()))?;
    registry.register(Box::new(PING_SUCCESS.clone()))?;
    registry.register(Box::new(IDENTIFY_FAILURE.clone()))?;
    registry.register(Box::new(IDENTIFY_RECEIVED.clone()))?;
    registry.register(Box::new(DISCOVERY_BACKGROUND_LOOKUP.clone()))?;
    registry.register(Box::new(DISCOVERY_CONNECTED_PEERS.clone()))?;
    registry.register(Box::new(MEMBERSHIP_SKIPPED_PEERS.clone()))?;
    registry.register(Box::new(MEMBERSHIP_ROUTABLE_PEERS.clone()))?;
    registry.register(Box::new(MEMBERSHIP_PROVIDER_PEERS.clone()))?;
    registry.register(Box::new(MEMBERSHIP_UNKNOWN_TOPIC.clone()))?;
    registry.register(Box::new(MEMBERSHIP_INVALID_MESSAGE.clone()))?;
    registry.register(Box::new(MEMBERSHIP_PUBLISH_SUCCESS.clone()))?;
    registry.register(Box::new(MEMBERSHIP_PUBLISH_FAILURE.clone()))?;
    registry.register(Box::new(CONTENT_RESOLVE_RUNNING.clone()))?;
    registry.register(Box::new(CONTENT_RESOLVE_NO_PEERS.clone()))?;
    registry.register(Box::new(CONTENT_RESOLVE_SUCCESS.clone()))?;
    registry.register(Box::new(CONTENT_RESOLVE_FAILURE.clone()))?;
    registry.register(Box::new(CONTENT_RESOLVE_FALLBACK.clone()))?;
    registry.register(Box::new(CONTENT_RESOLVE_PEERS.clone()))?;
    registry.register(Box::new(CONTENT_CONNECTED_PEERS.clone()))?;
    Ok(())
}
