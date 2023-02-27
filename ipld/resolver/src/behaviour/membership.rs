use std::task::Context;

use libp2p::core::connection::ConnectionId;
use libp2p::swarm::derive_prelude::FromSwarm;
use libp2p::swarm::{NetworkBehaviourAction, PollParameters};
use libp2p::Multiaddr;
use libp2p::{
    gossipsub::Gossipsub,
    swarm::{ConnectionHandler, IntoConnectionHandler, NetworkBehaviour},
    PeerId,
};

use crate::provider_record::SignedProviderRecord;

/// Events emitted by the [`membership::Behaviour`] behaviour.
#[derive(Debug)]
pub enum Event {
    /// Indicate that a given peer is able to serve data from a list of subnets.
    SubnetProvider(SignedProviderRecord),
}

/// A [`NetworkBehaviour`] internally using [`Gossipsub`] to learn which
/// peer is able to resolve CIDs in different subnets.
pub struct Behaviour {
    inner: Gossipsub,
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
        _cx: &mut Context<'_>,
        _params: &mut impl PollParameters,
    ) -> std::task::Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        // Poll Gossipsub for events; this is where we can handle Gossipsub messages and
        // store the associations from peers to subnets.
        todo!()
    }
}
