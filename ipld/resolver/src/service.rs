// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::time::Duration;

use anyhow::anyhow;
use ipc_sdk::subnet_id::SubnetID;
use libipld::store::StoreParams;
use libipld::Cid;
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
use log::{debug, error, trace, warn};
use tokio::select;
use tokio::sync::oneshot::{self, Sender};

use crate::behaviour::{
    self, content, discovery, membership, Behaviour, BehaviourEvent, ConfigError, DiscoveryConfig,
    MembershipConfig, NetworkConfig,
};

/// Keeps track of where to send query responses to.
type QueryMap = HashMap<content::QueryId, oneshot::Sender<anyhow::Result<()>>>;

/// Result of attempting to resolve a CID.
pub type ResolveResult = anyhow::Result<()>;

#[derive(thiserror::Error, Debug)]
#[error("No known peers for subnet {0}")]
pub struct NoKnownPeers(SubnetID);

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

/// Internal requests to enqueue to the [`Service`]
enum Request {
    SetProvidedSubnets(Vec<SubnetID>),
    PinSubnets(Vec<SubnetID>),
    Resolve(Cid, SubnetID, oneshot::Sender<ResolveResult>),
}

/// A facade to the [`Service`] to provide a nicer interface than message passing would allow on its own.
#[derive(Clone)]
pub struct Client {
    request_tx: tokio::sync::mpsc::UnboundedSender<Request>,
}

impl Client {
    fn send_request(&self, req: Request) -> anyhow::Result<()> {
        self.request_tx
            .send(req)
            .map_err(|_| anyhow!("disconnected"))
    }

    pub fn set_provided_subnets(&self, subnet_ids: Vec<SubnetID>) -> anyhow::Result<()> {
        let req = Request::SetProvidedSubnets(subnet_ids);
        self.send_request(req)
    }

    pub fn pin_subnets(&self, subnet_ids: Vec<SubnetID>) -> anyhow::Result<()> {
        let req = Request::PinSubnets(subnet_ids);
        self.send_request(req)
    }

    /// Send a CID for resolution from a specific subnet, await its completion,
    /// then return the result, to be inspected by the caller.
    ///
    /// Upon success, the data should be found in the store.
    pub async fn resolve(&self, cid: Cid, subnet_id: SubnetID) -> anyhow::Result<ResolveResult> {
        let (tx, rx) = oneshot::channel();
        let req = Request::Resolve(cid, subnet_id, tx);
        self.send_request(req)?;
        let res = rx.await?;
        Ok(res)
    }
}

/// The `Service` handles P2P communication to resolve IPLD content by wrapping and driving a number of `libp2p` behaviours.
pub struct Service<P: StoreParams> {
    listen_addr: Multiaddr,
    swarm: Swarm<Behaviour<P>>,
    queries: QueryMap,
    request_rx: tokio::sync::mpsc::UnboundedReceiver<Request>,
}

impl<P: StoreParams> Service<P> {
    pub fn new<S>(config: Config, store: S) -> Result<(Self, Client), ConfigError>
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

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let service = Self {
            listen_addr: config.connection.listen_addr,
            swarm,
            queries: Default::default(),
            request_rx: rx,
        };

        let client = Client { request_tx: tx };

        Ok((service, client))
    }

    /// Start the swarm listening for incoming connections and drive the events forward.
    pub async fn run(mut self) -> anyhow::Result<()> {
        // Start the swarm.
        Swarm::listen_on(&mut self.swarm, self.listen_addr.clone())?;

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
                request = self.request_rx.recv() => match request {
                    // A Client sent us a request.
                    Some(req) => self.handle_request(req),
                    // All Client instances have been dropped.
                    // We could keep the Swarm alive to serve content to others,
                    // but we ourselves are unable to send requests. Let's treat
                    // this as time to quit.
                    None => { break; }
                }
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
        if let identify::Event::Error { peer_id, error } = event {
            warn!("Error identifying {peer_id}: {error}")
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

    /// Handle Bitswap lookup result.
    fn handle_content_event(&mut self, event: content::Event) {
        match event {
            content::Event::Complete(query_id, result) => {
                if let Some(tx) = self.queries.remove(&query_id) {
                    send_resolve_result(tx, result)
                } else {
                    warn!("query ID not found");
                }
            }
        }
    }

    /// Handle an internal request coming from a [`Client`].
    fn handle_request(&mut self, request: Request) {
        match request {
            Request::SetProvidedSubnets(ids) => {
                if let Err(e) = self.membership_mut().set_provided_subnets(ids) {
                    error!("error setting subnet providers: {e}")
                }
            }
            Request::PinSubnets(ids) => {
                for id in ids {
                    self.membership_mut().pin_subnet(id)
                }
            }
            Request::Resolve(cid, subnet_id, tx) => {
                let peers = self.membership_mut().providers_of_subnet(&subnet_id);
                if peers.is_empty() {
                    send_resolve_result(tx, Err(anyhow!(NoKnownPeers(subnet_id))));
                } else {
                    let query_id = self.content_mut().resolve(cid, peers);
                    self.queries.insert(query_id, tx);
                }
            }
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

fn send_resolve_result(tx: Sender<ResolveResult>, res: ResolveResult) {
    if tx.send(res).is_err() {
        error!("error sending resolve result; listener closed")
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
