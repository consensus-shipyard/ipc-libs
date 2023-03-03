// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Test that a cluster of IPLD resolver can be started in memory,
//! that they bootstrap from  each other and are able to resolve CIDs.
//!
//! Run the tests as follows:
//! ```ignore
//! cargo test -p ipc_ipld_resolver --test smoke
//! ```

// For inspiration on testing libp2p look at:
// * https://github.com/libp2p/rust-libp2p/blob/v0.50.0/misc/multistream-select/tests/transport.rs
// * https://github.com/libp2p/rust-libp2p/blob/v0.50.0/protocols/ping/tests/ping.rs
// * https://github.com/libp2p/rust-libp2p/blob/v0.50.0/protocols/gossipsub/tests/smoke.rs
// They all use a different combination of `MemoryTransport` and executors.
// These tests attempt to use `MemoryTransport` so it's quicker, with `Swarm::with_tokio_executor`
// so we can leave the polling to the `Service` running in a `Task`, rather than do it from the test
// (although these might be orthogonal).

use std::time::Duration;

use ipc_ipld_resolver::{
    Client, Config, ConnectionConfig, DiscoveryConfig, MembershipConfig, NetworkConfig, Service,
};
use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{Boxed, MemoryTransport},
    },
    identity::Keypair,
    mplex,
    multiaddr::Protocol,
    plaintext::PlainText2Config,
    yamux, Multiaddr, PeerId, Transport,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

mod store;
use store::*;

struct Agent {
    config: Config,
    client: Client,
    store: TestBlockstore,
}

struct Cluster {
    agents: Vec<Agent>,
}

struct ClusterBuilder {
    size: u32,
    rng: StdRng,
    services: Vec<Service<TestStoreParams>>,
    agents: Vec<Agent>,
}

impl ClusterBuilder {
    fn new(size: u32, seed: u64) -> Self {
        Self {
            size,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            services: Default::default(),
            agents: Default::default(),
        }
    }

    /// Add a node with randomized address, optionally bootstrapping from an existing node.
    fn add_node(&mut self, bootstrap: Option<usize>) {
        let bootstrap_addr = bootstrap.map(|i| {
            let config = &self.agents[i].config;
            let peer_id = config.network.local_peer_id();
            let mut addr = config.connection.listen_addr.clone();
            addr.push(Protocol::P2p(peer_id.into()));
            addr
        });
        let config = make_config(&mut self.rng, self.size, bootstrap_addr);
        let (service, client, store) = make_service(config.clone());
        self.services.push(service);
        self.agents.push(Agent {
            config,
            client,
            store,
        });
    }

    /// Start running all services
    fn run(self) -> Cluster {
        for service in self.services {
            tokio::task::spawn(async move { service.run().await.expect("error running service") });
        }
        Cluster {
            agents: self.agents,
        }
    }
}

/// Start a cluster of 5 nodes from a single bootstrap node.
/// Mark one of them as the provider of some subnet.
/// Insert a CID of an array of CIDs into the store of the provider.
/// Ask for the CID to be resolved from by another peer.
/// Check that the CID is deposited into the store of the requestor.
#[tokio::test]
async fn single_bootstrap_single_provider_resolve_one() {
    // TODO: Get the seed from QuickCheck
    let mut builder = ClusterBuilder::new(5, 123456u64);

    for i in 0..builder.size {
        builder.add_node(if i == 0 { None } else { Some(0) });
    }

    let _cluster = builder.run();
}

fn make_service(config: Config) -> (Service<TestStoreParams>, Client, TestBlockstore) {
    let store = TestBlockstore::default();
    let (svc, cli) = Service::new_with_transport(config, store.clone(), build_transport).unwrap();
    (svc, cli, store)
}

fn make_config(rng: &mut StdRng, cluster_size: u32, bootstrap_addr: Option<Multiaddr>) -> Config {
    let config = Config {
        connection: ConnectionConfig {
            listen_addr: Multiaddr::from(Protocol::Memory(rng.gen::<u64>())),
            expected_peer_count: cluster_size,
            max_incoming: cluster_size,
        },
        network: NetworkConfig {
            local_key: Keypair::generate_secp256k1(),
            network_name: "smoke-test".to_owned(),
        },
        discovery: DiscoveryConfig {
            static_addresses: bootstrap_addr.iter().cloned().collect(),
            target_connections: cluster_size.try_into().unwrap(),
            enable_kademlia: bootstrap_addr.is_some(),
        },
        membership: MembershipConfig {
            static_subnets: vec![],
            max_subnets: 10,
            publish_interval: Duration::from_secs(5),
            max_provider_age: Duration::from_secs(60),
        },
    };

    config
}

/// Builds an in-memory transport for libp2p to communicate over.
fn build_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let auth_config = PlainText2Config {
        local_public_key: local_key.public(),
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

    MemoryTransport::default()
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(auth_config)
        .multiplex(mplex_config)
        .boxed()
}
