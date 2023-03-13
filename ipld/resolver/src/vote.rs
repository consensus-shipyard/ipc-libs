use ipc_sdk::subnet_id::SubnetID;
use libipld::Cid;
use libp2p::{core::SignedEnvelope, identity::PublicKey};

use crate::Timestamp;

/// Vote by a validator about the validity/availability/finality
/// of a CID in a given subnet.
pub struct Vote {
    /// Public key of the validator.
    pub public_key: PublicKey,
    /// The subnet in which the vote is valid, to prevent a vote on the same CID
    /// in one subnet being replayed by an attacker on a different subnet.
    pub subnet_id: SubnetID,
    /// The CID of the content the vote is about.
    pub cid: Cid,
    /// The claim of the vote, in case there can be votes about multiple facets
    /// regarding the CID.
    pub claim: String,
    /// Timestamp to thwart potential replay attacks.
    pub timestamp: Timestamp,
}
