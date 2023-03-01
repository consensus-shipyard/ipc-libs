use std::time::Duration;

// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use libipld::store::StoreParams;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    identity::Keypair,
    mplex, noise,
    swarm::{ConnectionLimits, SwarmBuilder},
    yamux, Multiaddr, PeerId, Swarm, Transport,
};
use libp2p::{identify, ping};
use libp2p_bitswap::BitswapStore;
use log::{debug, trace, warn};
use tokio::select;

use crate::behaviour::{
    self, content, discovery, membership, Behaviour, BehaviourEvent, ConfigError, DiscoveryConfig,
    MembershipConfig, NetworkConfig,
};

pub struct ConnectionConfig {
    /// The address where we will listen to incoming connections.
    listen_addr: Multiaddr,
    /// Maximum number of incoming connections.
    max_incoming: u32,
}

pub struct Config {
    network: NetworkConfig,
    discovery: DiscoveryConfig,
    membership: MembershipConfig,
    connection: ConnectionConfig,
}

pub struct IpldResolverService<P: StoreParams> {
    listen_addr: Multiaddr,
    swarm: Swarm<Behaviour<P>>,
}

impl<P: StoreParams> IpldResolverService<P> {
    pub fn new<S>(config: Config, store: S) -> Result<Self, ConfigError>
    where
        S: BitswapStore<Params = P>,
    {
        let peer_id = config.network.local_peer_id();
        let transport = build_transport(config.network.local_key.clone());
        let behaviour = Behaviour::new(config.network, config.discovery, config.membership, store)?;

        // NOTE: Hardcoded values from Forest. Will leave them as is until we know we need to change.

        let limits = ConnectionLimits::default()
            .with_max_pending_incoming(Some(10))
            .with_max_pending_outgoing(Some(30))
            .with_max_established_incoming(Some(config.connection.max_incoming))
            .with_max_established_outgoing(None) // Allow bitswap to connect to subnets we did not anticipate when we started.
            .with_max_established_per_peer(Some(5));

        let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id)
            .connection_limits(limits)
            .notify_handler_buffer_size(std::num::NonZeroUsize::new(20).expect("Not zero"))
            .connection_event_buffer_size(64)
            .build();

        Ok(Self {
            listen_addr: config.connection.listen_addr,
            swarm,
        })
    }

    /// Start the swarm listening for incoming connections and drive the events forward.
    pub async fn run(mut self) -> anyhow::Result<()> {
        // Start the swarm.
        Swarm::listen_on(&mut self.swarm, self.listen_addr.clone())?;

        // TODO: Should there be some control channel to close the service?
        loop {
            select! {
                swarm_event = self.swarm.next() => match swarm_event {
                    // Events raised by our behaviours.
                    Some(SwarmEvent::Behaviour(event)) => {
                        self.handle_behaviour_event(
                            event)
                    },
                    // Connection events are handled by the behaviours, passed directly from the Swarm.
                    Some(_) => { },
                    // The connection is closed.
                    None => { break; },
                },
                // TODO: Add a channel for internal requests.
            };
        }
        Ok(())
    }

    /// Handle events that the [`NetworkBehaviour`] for our [`Behaviour`] macro generated, one for each field.
    fn handle_behaviour_event(&mut self, event: BehaviourEvent<P>) {
        match event {
            BehaviourEvent::Ping(e) => self.handle_ping_event(e),
            BehaviourEvent::Identify(e) => self.handle_identify_event(e),
            BehaviourEvent::Discovery(e) => self.handle_discovery_event(e),
            BehaviourEvent::Membership(e) => self.handle_membership_event(e),
            BehaviourEvent::Content(e) => self.handle_content_event(e),
        }
    }

    // Copied from Forest.
    fn handle_ping_event(&mut self, event: ping::Event) {
        let peer_id = event.peer.to_base58();
        match event.result {
            Ok(ping::Success::Ping { rtt }) => {
                trace!(
                    "PingSuccess::Ping rtt to {} is {} ms",
                    peer_id,
                    rtt.as_millis()
                );
            }
            Ok(ping::Success::Pong) => {
                trace!("PingSuccess::Pong from {peer_id}");
            }
            Err(ping::Failure::Timeout) => {
                debug!("PingFailure::Timeout from {peer_id}");
            }
            Err(ping::Failure::Other { error }) => {
                warn!("PingFailure::Other from {peer_id}: {error}");
            }
            Err(ping::Failure::Unsupported) => {
                warn!("Banning peer {peer_id} due to protocol error");
                self.swarm.ban_peer_id(event.peer);
            }
        }
    }

    fn handle_identify_event(&mut self, event: identify::Event) {
        match event {
            identify::Event::Error { peer_id, error } => {
                warn!("Error identifying {peer_id}: {error}")
            }
            _ => {}
        }
    }

    fn handle_discovery_event(&mut self, event: discovery::Event) {
        match event {
            discovery::Event::Added(peer_id, _) => self.membership_mut().set_routable(peer_id),
            discovery::Event::Removed(peer_id) => self.membership_mut().set_unroutable(peer_id),
            discovery::Event::Connected(_, _) => {}
            discovery::Event::Disconnected(_, _) => {}
        }
    }

    fn handle_membership_event(&mut self, event: membership::Event) {
        match event {
            membership::Event::Skipped(peer_id) => self.discovery_mut().background_lookup(peer_id),
            membership::Event::Updated(_, _) => {}
            membership::Event::Removed(_) => {}
        }
    }

    fn handle_content_event(&mut self, event: content::Event) {
        match event {
            content::Event::Complete(_query_id, _result) => todo!("add book keeping"),
        }
    }

    // The following are helper functions because Rust Analyzer has trouble with recognising that `swarm.behaviour_mut()` is a legal call.

    fn discovery_mut(&mut self) -> &mut behaviour::discovery::Behaviour {
        self.swarm.behaviour_mut().discovery_mut()
    }
    fn membership_mut(&mut self) -> &mut behaviour::membership::Behaviour {
        self.swarm.behaviour_mut().membership_mut()
    }
    fn content_mut(&mut self) -> &mut behaviour::content::Behaviour<P> {
        self.swarm.behaviour_mut().content_mut()
    }
}

/// Builds the transport stack that libp2p will communicate over.
///
/// Based on the equivalent in Forest.
pub fn build_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let tcp_transport =
        || libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::new().nodelay(true));
    let transport = libp2p::dns::TokioDnsConfig::system(tcp_transport()).unwrap();
    let auth_config = {
        let dh_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&local_key)
            .expect("Noise key generation failed");

        noise::NoiseConfig::xx(dh_keys).into_authenticated()
    };

    let mplex_config = {
        let mut mplex_config = mplex::MplexConfig::new();
        mplex_config.set_max_buffer_size(usize::MAX);

        let mut yamux_config = yamux::YamuxConfig::default();
        yamux_config.set_max_buffer_size(16 * 1024 * 1024);
        yamux_config.set_receive_window_size(16 * 1024 * 1024);
        // yamux_config.set_window_update_mode(WindowUpdateMode::OnRead);
        libp2p::core::upgrade::SelectUpgrade::new(yamux_config, mplex_config)
    };

    transport
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(auth_config)
        .multiplex(mplex_config)
        .timeout(Duration::from_secs(20))
        .boxed()
}
