// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::VecDeque;
use std::task::{Context, Poll};
use std::time::Duration;

use ipc_sdk::subnet_id::SubnetID;
use libp2p::core::connection::ConnectionId;
use libp2p::gossipsub::error::SubscriptionError;
use libp2p::gossipsub::{
    GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage, IdentTopic, MessageAuthenticity,
    MessageId, Topic, TopicHash,
};
use libp2p::identity::Keypair;
use libp2p::swarm::derive_prelude::FromSwarm;
use libp2p::swarm::{NetworkBehaviourAction, PollParameters};
use libp2p::Multiaddr;
use libp2p::{
    gossipsub::Gossipsub,
    swarm::{ConnectionHandler, IntoConnectionHandler, NetworkBehaviour},
    PeerId,
};
use log::{debug, error, warn};
use tokio::time::{Instant, Interval};

use crate::hash::blake2b_256;
use crate::provider_cache::{ProviderDelta, SubnetProviderCache};
use crate::provider_record::{ProviderRecord, SignedProviderRecord, Timestamp};

use super::NetworkConfig;

/// `Gossipsub` subnet membership topic identifier.
const PUBSUB_MEMBERSHIP: &str = "/ipc/membership";

/// Events emitted by the [`membership::Behaviour`] behaviour.
#[derive(Debug)]
pub enum Event {
    /// Indicate a change in the subnets a peer is known to support.
    Updated(PeerId, ProviderDelta),

    /// Indicate that we no longer treat a peer as routable and removed all their supported subnet associations.
    Removed(PeerId),

    /// We could not add a provider record to the cache because the chache hasn't
    /// been told yet that the provider peer is routable. This event can be used
    /// to trigger a lookup by the discovery module to learn the address.
    Skipped(PeerId),
}

/// Configuration for [`membership::Behaviour`].
#[derive(Clone, Debug)]
pub struct Config {
    /// User defined list of subnets which will never be pruned from the cache.
    pub static_subnets: Vec<SubnetID>,
    /// Maximum number of subnets to track in the cache.
    pub max_subnets: usize,
    /// Publish interval for supported subnets.
    pub publish_interval: Duration,
    /// Minimum time between publishing own provider record in reaction to new joiners.
    pub min_time_between_publish: Duration,
    /// Maximum age of provider records before the peer is removed without an update.
    pub max_provider_age: Duration,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("invalid network: {0}")]
    InvalidNetwork(String),
    #[error("invalid gossipsub config: {0}")]
    InvalidGossipsubConfig(String),
    #[error("error subscribing to topic")]
    Subscription(#[from] SubscriptionError),
}

/// A [`NetworkBehaviour`] internally using [`Gossipsub`] to learn which
/// peer is able to resolve CIDs in different subnets.
pub struct Behaviour {
    /// [`Gossipsub`] behaviour to spread the information about subnet membership.
    inner: Gossipsub,
    /// Events to return when polled.
    outbox: VecDeque<Event>,
    /// [`Keypair`] used to sign [`SignedProviderRecord`] instances.
    local_key: Keypair,
    /// Name of the [`Gossipsub`] topic where subnet memberships are published.
    membership_topic: IdentTopic,
    /// List of subnet IDs this agent is providing data for.
    subnet_ids: Vec<SubnetID>,
    /// Caching the latest state of subnet providers.
    provider_cache: SubnetProviderCache,
    /// Interval between publishing the currently supported subnets.
    ///
    /// This acts like a heartbeat; if a peer doesn't publish its snapshot for a long time,
    /// other agents can prune it from their cache and not try to contact for resolution.
    publish_interval: Interval,
    /// Minimum time between publishing own provider record in reaction to new joiners.
    min_time_between_publish: Duration,
    /// Last time we gossiped our own provider record.
    last_publish_timestamp: Timestamp,
    /// Next time we will gossip our own provider record.
    next_publish_timestamp: Timestamp,
    /// Maximum time a provider can be without an update before it's pruned from the cache.
    max_provider_age: Duration,
}

impl Behaviour {
    pub fn new(nc: NetworkConfig, mc: Config) -> Result<Self, ConfigError> {
        if nc.network_name.is_empty() {
            return Err(ConfigError::InvalidNetwork(nc.network_name));
        }
        let membership_topic = Topic::new(format!("{}/{}", PUBSUB_MEMBERSHIP, nc.network_name));

        let mut gossipsub_config = GossipsubConfigBuilder::default();
        // Set the maximum message size to 2MB.
        gossipsub_config.max_transmit_size(2 << 20);
        gossipsub_config.message_id_fn(|msg: &GossipsubMessage| {
            let s = blake2b_256(&msg.data);
            MessageId::from(s)
        });

        let gossipsub_config = gossipsub_config
            .build()
            .map_err(|s| ConfigError::InvalidGossipsubConfig(s.into()))?;

        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(nc.local_key.clone()),
            gossipsub_config,
        )
        .map_err(|s| ConfigError::InvalidGossipsubConfig(s.into()))?;

        gossipsub
            .with_peer_score(
                scoring::build_peer_score_params(membership_topic.clone()),
                scoring::build_peer_score_thresholds(),
            )
            .map_err(ConfigError::InvalidGossipsubConfig)?;

        // Subscribe to the topic.
        gossipsub.subscribe(&membership_topic)?;

        // Don't publish immediately, it's empty. Let the creator call `set_subnet_ids` to trigger initially.
        let mut interval = tokio::time::interval(mc.publish_interval);
        interval.reset();

        Ok(Self {
            inner: gossipsub,
            outbox: Default::default(),
            local_key: nc.local_key,
            membership_topic,
            subnet_ids: Default::default(),
            provider_cache: SubnetProviderCache::new(mc.max_subnets, mc.static_subnets),
            publish_interval: interval,
            min_time_between_publish: mc.min_time_between_publish,
            last_publish_timestamp: Timestamp::default(),
            next_publish_timestamp: Timestamp::now() + mc.publish_interval,
            max_provider_age: mc.max_provider_age,
        })
    }

    /// Set all the currently supported subnet IDs, then publish the updated list.
    pub fn set_provided_subnets(&mut self, subnet_ids: Vec<SubnetID>) -> anyhow::Result<()> {
        self.subnet_ids = subnet_ids;
        self.publish_membership()
    }

    /// Add a subnet to the list of supported subnets, then publish the updated list.
    pub fn add_provided_subnet(&mut self, subnet_id: SubnetID) -> anyhow::Result<()> {
        if self.subnet_ids.contains(&subnet_id) {
            return Ok(());
        }
        self.subnet_ids.push(subnet_id);
        self.publish_membership()
    }

    /// Remove a subnet from the list of supported subnets, then publish the updated list.
    pub fn remove_provided_subnet(&mut self, subnet_id: SubnetID) -> anyhow::Result<()> {
        if !self.subnet_ids.contains(&subnet_id) {
            return Ok(());
        }
        self.subnet_ids.retain(|id| id != &subnet_id);
        self.publish_membership()
    }

    /// Make sure a subnet is not pruned.
    ///
    /// This method could be called in a parent subnet when the ledger indicates
    /// there is a known child subnet, so we make sure this subnet cannot be
    /// crowded out during the initial phase of bootstrapping the network.
    pub fn pin_subnet(&mut self, subnet_id: SubnetID) {
        self.provider_cache.pin_subnet(subnet_id)
    }

    /// Make a subnet pruneable.
    pub fn unpin_subnet(&mut self, subnet_id: &SubnetID) {
        self.provider_cache.unpin_subnet(subnet_id)
    }

    /// Send a message through Gossipsub to let everyone know about the current configuration.
    fn publish_membership(&mut self) -> anyhow::Result<()> {
        let record = SignedProviderRecord::new(&self.local_key, self.subnet_ids.clone())?;
        let data = record.into_envelope().into_protobuf_encoding();
        let _msg_id = self.inner.publish(self.membership_topic.clone(), data)?;
        self.last_publish_timestamp = Timestamp::now();
        self.next_publish_timestamp = self.last_publish_timestamp + self.publish_interval.period();
        self.publish_interval.reset(); // In case the change wasn't tiggered by the schedule.
        Ok(())
    }

    /// Mark a peer as routable in the cache.
    ///
    /// Call this method when the discovery service learns the address of a peer.
    pub fn set_routable(&mut self, peer_id: PeerId) {
        self.provider_cache.set_routable(peer_id);
        self.publish_for_new_peer(peer_id);
    }

    /// Mark a peer as unroutable in the cache.
    ///
    /// Call this method when the discovery service forgets the address of a peer.
    pub fn set_unroutable(&mut self, peer_id: PeerId) {
        self.provider_cache.set_unroutable(peer_id);
        self.outbox.push_back(Event::Removed(peer_id))
    }

    /// List the current providers of a subnet.
    ///
    /// Call this method when looking for a peer to resolve content from.
    pub fn providers_of_subnet(&self, subnet_id: &SubnetID) -> Vec<PeerId> {
        self.provider_cache.providers_of_subnet(subnet_id)
    }

    /// Parse and handle a [`GossipsubMessage`]. If it's from the expected topic,
    /// then raise domain event to let the rest of the application know about a
    /// provider. Also update all the book keeping in the behaviour that we use
    /// to answer future queries about the topic.
    fn handle_message(&mut self, msg: GossipsubMessage) {
        if msg.topic == self.membership_topic.hash() {
            match SignedProviderRecord::from_bytes(&msg.data).map(|r| r.into_record()) {
                Ok(record) => self.handle_provider_record(record),
                Err(e) => {
                    warn!(
                        "Gossip message from peer {:?} could not be deserialized: {e}",
                        msg.source
                    );
                }
            }
        } else {
            warn!(
                "unknown gossipsub topic in message from {:?}: {}",
                msg.source, msg.topic
            );
        }
    }

    /// Try to add a provider record to the cache.
    ///
    /// If this is the first time we receive a record from the peer,
    /// reciprocate by publishing our own.
    fn handle_provider_record(&mut self, record: ProviderRecord) {
        let is_new = !self.provider_cache.has_timestamp(&record.peer_id);
        let (event, publish) = match self.provider_cache.add_provider(&record) {
            None => (Some(Event::Skipped(record.peer_id)), false),
            Some(d) if d.is_empty() => (None, false),
            Some(d) => (Some(Event::Updated(record.peer_id, d)), is_new),
        };

        if let Some(event) = event {
            self.outbox.push_back(event);
        }

        if publish {
            self.publish_for_new_peer(record.peer_id)
        }
    }

    /// Handle new subscribers to the membership topic.
    fn handle_subscriber(&mut self, peer_id: PeerId, topic: TopicHash) {
        if topic == self.membership_topic.hash() {
            self.publish_for_new_peer(peer_id)
        } else {
            warn!(
                "unknown gossipsub topic in subscription from {}: {}",
                peer_id, topic
            )
        }
    }

    /// Publish our provider record when we encounter a new peer, unless we have recently done so.
    fn publish_for_new_peer(&mut self, peer_id: PeerId) {
        if self.subnet_ids.is_empty() {
            // We have nothing, so there's no need for them to know this ASAP.
            // The reason we shouldn't disable periodic publishing of empty
            // records completely is because it would also remove one of
            // triggers for non-connected peers to eagerly publish their
            // subnets when they see our empty records. Plus they could
            // be good to show on metrics, to have a single source of
            // the cluster size available on any node.
            return;
        }
        let now = Timestamp::now();
        if self.last_publish_timestamp > now - self.min_time_between_publish {
            debug!("recently published, not publishing again for peer {peer_id}");
        } else if self.next_publish_timestamp <= now + self.min_time_between_publish {
            debug!("publishing soon for new peer {peer_id}"); // don't let new joiners delay it forever by hitting the next block
        } else {
            debug!("publishing for new peer {peer_id}");
            // Create a new timer, rather than publish and reset. This way we don't repeat error handling.
            // Give some time for Kademlia and Identify to do their bit on both sides. Works better in tests.
            let delayed = Instant::now() + self.min_time_between_publish;
            self.next_publish_timestamp = now + self.min_time_between_publish;
            self.publish_interval =
                tokio::time::interval_at(delayed, self.publish_interval.period())
        }
    }

    /// Remove any membership record that hasn't been updated for a long time.
    fn prune_membership(&mut self) {
        let cutoff_timestamp = Timestamp::now() - self.max_provider_age;
        let pruned = self.provider_cache.prune_providers(cutoff_timestamp);
        for peer_id in pruned {
            self.outbox.push_back(Event::Removed(peer_id))
        }
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = <Gossipsub as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = Event;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        self.inner.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.inner.addresses_of_peer(peer_id)
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        self.inner.on_swarm_event(event)
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <<Self::ConnectionHandler as IntoConnectionHandler>::Handler as ConnectionHandler>::OutEvent,
    ) {
        self.inner
            .on_connection_handler_event(peer_id, connection_id, event)
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> std::task::Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        // Emit own events first.
        if let Some(ev) = self.outbox.pop_front() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
        }

        // Republish our current peer record snapshot and prune old records.
        if self.publish_interval.poll_tick(cx).is_ready() {
            if let Err(e) = self.publish_membership() {
                warn!("failed to publish membership: {e}")
            };
            self.prune_membership();
        }

        // Poll Gossipsub for events; this is where we can handle Gossipsub messages and
        // store the associations from peers to subnets.
        while let Poll::Ready(ev) = self.inner.poll(cx, params) {
            match ev {
                NetworkBehaviourAction::GenerateEvent(ev) => {
                    match ev {
                        // NOTE: We could (ab)use the Gossipsub mechanism itself to signal subnet membership,
                        // however I think the information would only spread to our nearest neighbours we are
                        // connected to. If we assume there are hundreds of agents in each subnet which may
                        // or may not overlap, and each agent is connected to ~50 other agents, then the chance
                        // that there are subnets from which there are no or just a few connections is not
                        // insignificant. For this reason I oped to use messages instead, and let the content
                        // carry the information, spreading through the Gossipsub network regardless of the
                        // number of connected peers.
                        GossipsubEvent::Subscribed { peer_id, topic } => {
                            self.handle_subscriber(peer_id, topic)
                        }

                        GossipsubEvent::Unsubscribed { .. } => {}
                        // Log potential misconfiguration.
                        GossipsubEvent::GossipsubNotSupported { peer_id } => {
                            debug!("peer {peer_id} doesn't support gossipsub");
                        }
                        GossipsubEvent::Message { message, .. } => {
                            self.handle_message(message);
                        }
                    }
                }
                other => {
                    return Poll::Ready(other.map_out(|_| unreachable!("already handled")));
                }
            }
        }

        Poll::Pending
    }
}

// Forest has Filecoin specific values copied from Lotus. Not sure what values to use,
// so I'll leave everything on default for now. Or maybe they should be left empty?
mod scoring {

    use libp2p::gossipsub::{IdentTopic, PeerScoreParams, PeerScoreThresholds, TopicScoreParams};

    pub fn build_peer_score_params(membership_topic: IdentTopic) -> PeerScoreParams {
        let mut params = PeerScoreParams::default();
        params
            .topics
            .insert(membership_topic.hash(), TopicScoreParams::default());
        params
    }

    pub fn build_peer_score_thresholds() -> PeerScoreThresholds {
        PeerScoreThresholds::default()
    }
}
