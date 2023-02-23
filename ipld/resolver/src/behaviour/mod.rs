use libipld::store::StoreParams;
use libp2p::{gossipsub::Gossipsub, identify, ping, swarm::NetworkBehaviour};
use libp2p_bitswap::Bitswap;

use self::discovery::Discovery;

mod discovery;

/// Libp2p behaviour to manage content resolution from other subnets, using:
///
/// * Kademlia for peer discovery
/// * Gossipsub to advertise subnet membership
/// * Bitswap to resolve CIDs
#[derive(NetworkBehaviour)]
pub struct IpldResolver<P: StoreParams> {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    discovery: Discovery,
    gossipsub: Gossipsub,
    bitswap: Bitswap<P>,
}

// Unfortunately by using `#[derive(NetworkBehaviour)]` we cannot easily inspects events
// from the inner behaviours, e.g. we cannot poll a behaviour and if it returns something
// of interest then call a method on another behaviour. We can do this in another wrapper
// where we manually implement `NetworkBehaviour`, or the outer service where we drive the
// Swarm; there we are free to call any of the behaviours.
