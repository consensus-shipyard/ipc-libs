use std::time::Duration;

// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use libipld::store::StoreParams;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    identity::Keypair,
    mplex, noise,
    swarm::{ConnectionLimits, SwarmBuilder},
    yamux, Multiaddr, PeerId, Swarm, Transport,
};
use libp2p_bitswap::BitswapStore;

use crate::behaviour::{Behaviour, ConfigError, DiscoveryConfig, MembershipConfig, NetworkConfig};

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

        Ok(Self { swarm })
    }

    /// Start the swarm listening for incoming connections and drive the events forward.
    pub async fn run(self) -> anyhow::Result<()> {
        todo!("IPC-37")
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
