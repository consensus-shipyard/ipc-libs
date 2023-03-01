// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use libipld::store::StoreParams;
use libp2p::{
    identify,
    identity::{Keypair, PublicKey},
    ping,
    swarm::NetworkBehaviour,
    PeerId,
};
use libp2p_bitswap::BitswapStore;

mod content;
mod discovery;
mod membership;

pub use discovery::Config as DiscoveryConfig;
pub use membership::Config as MembershipConfig;

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// Cryptographic key used to sign messages.
    pub local_key: Keypair,
    /// Network name to be differentiate this peer group.
    pub network_name: String,
}

impl NetworkConfig {
    pub fn local_public_key(&self) -> PublicKey {
        self.local_key.public()
    }
    pub fn local_peer_id(&self) -> PeerId {
        self.local_public_key().to_peer_id()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Error in the discovery configuration")]
    Discovery(#[from] discovery::ConfigError),
    #[error("Error in the membership configuration")]
    Membership(#[from] membership::ConfigError),
}

/// Libp2p behaviour bundle to manage content resolution from other subnets, using:
///
/// * Kademlia for peer discovery
/// * Gossipsub to advertise subnet membership
/// * Bitswap to resolve CIDs
#[derive(NetworkBehaviour)]
pub struct Behaviour<P: StoreParams> {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    discovery: discovery::Behaviour,
    membership: membership::Behaviour,
    content: content::Behaviour<P>,
}

// Unfortunately by using `#[derive(NetworkBehaviour)]` we cannot easily inspects events
// from the inner behaviours, e.g. we cannot poll a behaviour and if it returns something
// of interest then call a method on another behaviour. We can do this in the a wrapper
// where we manually implement `NetworkBehaviour`, or the outer service where we drive the
// Swarm; there we are free to call any of the behaviours.

impl<P: StoreParams> Behaviour<P> {
    pub fn new<S>(
        nc: NetworkConfig,
        dc: DiscoveryConfig,
        mc: MembershipConfig,
        store: S,
    ) -> Result<Self, ConfigError>
    where
        S: BitswapStore<Params = P>,
    {
        Ok(Self {
            ping: Default::default(),
            identify: identify::Behaviour::new(identify::Config::new(
                "ipfs/0.1.0".into(),
                nc.local_public_key(),
            )),
            discovery: discovery::Behaviour::new(nc.clone(), dc)?,
            membership: membership::Behaviour::new(nc, mc)?,
            content: content::Behaviour::new(store),
        })
    }
}
