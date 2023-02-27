use std::task::Context;

use fvm_ipld_encoding::serde::{Deserialize, Serialize};
use ipc_sdk::subnet_id::SubnetID;
use libipld::multihash;
use libp2p::core::connection::ConnectionId;
use libp2p::core::{signed_envelope, DecodeError, SignedEnvelope};
use libp2p::swarm::derive_prelude::FromSwarm;
use libp2p::swarm::{NetworkBehaviourAction, PollParameters};
use libp2p::Multiaddr;
use libp2p::{
    gossipsub::Gossipsub,
    swarm::{ConnectionHandler, IntoConnectionHandler, NetworkBehaviour},
    PeerId,
};

const DOMAIN_SEP: &str = "ipc-membership";
const PROVIDER_RECORD_PAYLOAD_TYPE: &str = "/ipc/provider-record";

/// Unix timestamp in seconds since epoch, which we can use to select the
/// more recent message during gossiping.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Timestamp(u64);

/// Record of the ability to provide data from a list of subnets.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderRecord {
    /// The ID of the peer we can contact to pull data from.
    peer_id: PeerId,
    /// The IDs of the subnets they are participating in.
    subnet_ids: Vec<SubnetID>,
    /// Timestamp from when the peer published this record.
    ///
    /// We use a timestamp instead of just a nonce so that we
    /// can drop records which are too old, indicating that
    /// the peer has dropped off.
    timestamp: Timestamp,
}

/// A [`ProviderRecord`] with a [`SignedEnvelope`] proving that the
/// peer indeed is ready to provide the data for the listed subnets.
#[derive(Debug)]
pub struct SignedProviderRecord {
    /// The deserialized and validated [`ProviderRecord`].
    record: ProviderRecord,
    /// The [`SignedEnvelope`] from which the record was deserialized from.
    envelope: SignedEnvelope,
}

// Based on `libp2p_core::peer_record::PeerRecord`
impl SignedProviderRecord {
    pub fn from_signed_envelope(envelope: SignedEnvelope) -> Result<Self, FromEnvelopeError> {
        let (payload, signing_key) = envelope.payload_and_signing_key(
            String::from(DOMAIN_SEP),
            PROVIDER_RECORD_PAYLOAD_TYPE.as_bytes(),
        )?;

        let record = fvm_ipld_encoding::from_slice::<ProviderRecord>(payload)?;

        if record.peer_id != signing_key.to_peer_id() {
            return Err(FromEnvelopeError::MismatchedSignature);
        }

        Ok(Self { record, envelope })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FromEnvelopeError {
    /// Failed to extract the payload from the envelope.
    #[error("Failed to extract payload from envelope")]
    BadPayload(#[from] signed_envelope::ReadPayloadError),
    /// Failed to decode the provided bytes as a [`ProviderRecord`].
    #[error("Failed to decode bytes as ProviderRecord")]
    InvalidProviderRecord(#[from] fvm_ipld_encoding::Error),
    /// Failed to decode the peer ID.
    #[error("Failed to decode bytes as PeerId")]
    InvalidPeerId(#[from] multihash::Error),
    /// The signer of the envelope is different than the peer id in the record.
    #[error("The signer of the envelope is different than the peer id in the record")]
    MismatchedSignature,
}

/// Events emitted by the [`Membership`] behaviour.
#[derive(Debug)]
pub enum MembershipEvent {
    /// Indicate that a given peer is able to serve data from a list of subnets.
    SubnetProvider(SignedProviderRecord),
}

/// `Membership` is a [`NetworkBehaviour`] internally using [`Gossipsub`] to learn which
/// peer is able to resolve CIDs in different subnets.
pub struct Membership {
    inner: Gossipsub,
}

impl NetworkBehaviour for Membership {
    type ConnectionHandler = <Gossipsub as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = MembershipEvent;

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
