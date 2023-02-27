use std::time::SystemTime;

use fvm_ipld_encoding::serde::{Deserialize, Serialize};
use ipc_sdk::subnet_id::SubnetID;
use libipld::multihash;
use libp2p::core::{signed_envelope, SignedEnvelope};
use libp2p::identity::Keypair;
use libp2p::PeerId;

const DOMAIN_SEP: &str = "ipc-membership";
const PAYLOAD_TYPE: &str = "/ipc/provider-record";

/// Unix timestamp in seconds since epoch, which we can use to select the
/// more recent message during gossiping.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Current timestamp.
    pub fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("now() is never before UNIX_EPOCH")
            .as_secs();
        Self(secs)
    }

    /// Seconds elapsed since Unix epoch.
    pub fn as_secs(&self) -> u64 {
        self.0
    }
}

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

impl ProviderRecord {
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
    pub fn subnet_ids(&self) -> &[SubnetID] {
        self.subnet_ids.as_slice()
    }
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
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
    /// Create a new [`SignedProviderRecord`] with the current timestamp
    /// and a signed envelope which can be shared with others.
    pub fn new(key: &Keypair, subnet_ids: Vec<SubnetID>) -> anyhow::Result<Self> {
        let timestamp = Timestamp::now();
        let peer_id = key.public().to_peer_id();
        let record = ProviderRecord {
            peer_id,
            subnet_ids,
            timestamp,
        };
        let payload = fvm_ipld_encoding::to_vec(&record)?;
        let envelope = SignedEnvelope::new(
            key,
            DOMAIN_SEP.to_owned(),
            PAYLOAD_TYPE.as_bytes().to_vec(),
            payload,
        )?;
        Ok(Self { record, envelope })
    }

    pub fn from_signed_envelope(envelope: SignedEnvelope) -> Result<Self, FromEnvelopeError> {
        let (payload, signing_key) =
            envelope.payload_and_signing_key(DOMAIN_SEP.to_owned(), PAYLOAD_TYPE.as_bytes())?;

        let record = fvm_ipld_encoding::from_slice::<ProviderRecord>(payload)?;

        if record.peer_id != signing_key.to_peer_id() {
            return Err(FromEnvelopeError::MismatchedSignature);
        }

        Ok(Self { record, envelope })
    }

    pub fn record(&self) -> &ProviderRecord {
        &self.record
    }

    pub fn envelope(&self) -> &SignedEnvelope {
        &self.envelope
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
