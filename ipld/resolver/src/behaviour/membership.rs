use std::collections::{HashMap, VecDeque};
use std::task::{Context, Poll};

use ipc_sdk::subnet_id::SubnetID;
use libp2p::core::connection::ConnectionId;
use libp2p::gossipsub::{GossipsubEvent, GossipsubMessage, IdentTopic, Topic};
use libp2p::identity::Keypair;
use libp2p::swarm::derive_prelude::FromSwarm;
use libp2p::swarm::{NetworkBehaviourAction, PollParameters};
use libp2p::Multiaddr;
use libp2p::{
    gossipsub::Gossipsub,
    swarm::{ConnectionHandler, IntoConnectionHandler, NetworkBehaviour},
    PeerId,
};
use log::debug;
use tokio::time::Interval;

use crate::provider_record::{SignedProviderRecord, Timestamp};

/// `Gossipsub` subnet membership topic identifier.
const PUBSUB_MEMBERSHIP: &str = "/ipc/membership";

struct Config {
    /// Network name to be combined into the Gossipsub topic.
    network_name: String,
}

/// Events emitted by the [`membership::Behaviour`] behaviour.
#[derive(Debug)]
pub enum Event {
    /// Indicate that a given peer is able to serve data from a list of subnets.
    ///
    /// Note that each the event contains the snapshot of the currently provided
    /// subnets, not a delta. This means that if there were two peers using the
    /// same keys running on different addresses, e.g. if the same operator ran
    /// something supporting subnet A on one address, and another process supporting
    /// subnet B on a different address, these would override each other, unless
    /// they have different public keys (and thus peer IDs) associated with them.
    ///
    /// This should be okay, as in practice there is no significance to these
    /// peer IDs, we can even generate a fresh key-pair every time we run the
    /// resolver.
    SubnetProvider(SignedProviderRecord),
}

/// A [`NetworkBehaviour`] internally using [`Gossipsub`] to learn which
/// peer is able to resolve CIDs in different subnets.
pub struct Behaviour {
    /// [`Gossipsub`] behaviour to spread the information about subnet membership.
    inner: Gossipsub,
    /// Events to return when polled.
    outbox: VecDeque<Event>,
    /// [`Keypair`] used to construct [`SignedProviderRecord`] instances.
    keypair: Keypair,
    /// Name of the [`Gossipsub`] topic where subnet memberships are published.
    membership_topic: IdentTopic, // Topic::new(format!("{}/{}", PUBSUB_MEMBERSHIP, network_name)
    /// List of subnet IDs this agent is providing data for.
    subnet_ids: Vec<SubnetID>,
    /// List of peer IDs supporting each individual subnet.
    ///
    /// TODO: Limit the number of peers and topic we track. How do we limit the subnets?
    /// An agent will probably only be asked to resolve topics from the current subnet and
    /// its children, but how do we know which are legitimate child subnets?
    ///
    /// TODO: How do we protect against DoS attacks trying to fill the memory with bogus data?
    /// We could differentiate by tracking which peers we are actually connected to, and first
    /// prune the ones we just heard about, but we don't know if they are legit.
    subnet_membership: HashMap<SubnetID, Vec<PeerId>>,
    /// Timestamp of the last record received about a peer.
    ///
    /// TODO: Add more meta-data, such as whether the peer was connected to at any point in time,
    /// or whether it served data to us.
    peer_timestamps: HashMap<PeerId, Timestamp>,
    /// Interval between publishing the currently supported subnets.
    ///
    /// This acts like a heartbeat; if a peer doesn't publish its snapshot for a long time,
    /// other agents can prune it from their cache and not try to contact for resolution.
    publish_interval: Interval,
}

impl Behaviour {
    /// Set all the currently supported subnet IDs, then publish the updated list.
    pub fn set_subnet_ids(&mut self, subnet_ids: Vec<SubnetID>) -> anyhow::Result<()> {
        self.subnet_ids = subnet_ids;
        self.publish_membership()
    }

    /// Add a subnet to the list of supported subnets, then publish the updated list.
    pub fn add_subnet_id(&mut self, subnet_id: SubnetID) -> anyhow::Result<()> {
        if self.subnet_ids.contains(&subnet_id) {
            return Ok(());
        }
        self.subnet_ids.push(subnet_id);
        self.publish_membership()
    }

    /// Remove a subnet from the list of supported subnets, then publish the updated list.
    pub fn remove_subnet_id(&mut self, subnet_id: SubnetID) -> anyhow::Result<()> {
        if !self.subnet_ids.contains(&subnet_id) {
            return Ok(());
        }
        self.subnet_ids.retain(|id| id != &subnet_id);
        self.publish_membership()
    }

    /// Send a message through Gossipsub to let everyone know about the current configuration.
    fn publish_membership(&mut self) -> anyhow::Result<()> {
        let record = SignedProviderRecord::new(&self.keypair, self.subnet_ids.clone())?;
        let data = record.into_envelope().into_protobuf_encoding();
        let _msg_id = self.inner.publish(self.membership_topic.clone(), data)?;
        Ok(())
    }

    /// Remove any membership record that hasn't been updated for a long time.
    fn prune_membership(&mut self) {}

    /// Parse and handle a [`GossipsubMessage`]. If it's from the expected topic,
    /// then raise domain event to let the rest of the application know about a
    /// provider. Also update all the book keeping in the behaviour that we use
    /// to answer future queries about the topic.
    fn handle_message(&mut self, msg: GossipsubMessage) -> Option<Event> {
        todo!()
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
            self.publish_membership();
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
                        GossipsubEvent::Subscribed { .. } | GossipsubEvent::Unsubscribed { .. } => {
                        }
                        // Log potential misconfiguration.
                        GossipsubEvent::GossipsubNotSupported { peer_id } => {
                            debug!("peer {peer_id} doesn't support gossipsub");
                        }
                        GossipsubEvent::Message {
                            propagation_source,
                            message_id,
                            message,
                        } => {
                            if let Some(ev) = self.handle_message(message) {
                                return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
                            }
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
