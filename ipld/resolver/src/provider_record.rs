// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime};

use fvm_ipld_encoding::serde::{Deserialize, Serialize};
use ipc_sdk::subnet_id::SubnetID;
use libp2p::identity::Keypair;
use libp2p::PeerId;

use crate::signed_record::{Record, SignedRecord};

/// Unix timestamp in seconds since epoch, which we can use to select the
/// more recent message during gossiping.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize, Default)]
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

impl Sub<Duration> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Self(self.as_secs().saturating_sub(rhs.as_secs()))
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(self.as_secs().saturating_add(rhs.as_secs()))
    }
}

/// Record of the ability to provide data from a list of subnets.
///
/// Note that each the record contains the snapshot of the currently provided
/// subnets, not a delta. This means that if there were two peers using the
/// same keys running on different addresses, e.g. if the same operator ran
/// something supporting subnet A on one address, and another process supporting
/// subnet B on a different address, these would override each other, unless
/// they have different public keys (and thus peer IDs) associated with them.
///
/// This should be okay, as in practice there is no significance to these
/// peer IDs, we can even generate a fresh key-pair every time we run the
/// resolver.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ProviderRecord {
    /// The ID of the peer we can contact to pull data from.
    pub peer_id: PeerId,
    /// The IDs of the subnets they are participating in.
    pub subnet_ids: Vec<SubnetID>,
    /// Timestamp from when the peer published this record.
    ///
    /// We use a timestamp instead of just a nonce so that we
    /// can drop records which are too old, indicating that
    /// the peer has dropped off.
    pub timestamp: Timestamp,
}

impl Record for ProviderRecord {
    fn domain_sep() -> &'static str {
        "ipc-membership"
    }

    fn payload_type() -> &'static str {
        "/ipc/provider-record"
    }

    fn check_signing_key(&self, key: &libp2p::identity::PublicKey) -> bool {
        self.peer_id == key.to_peer_id()
    }
}

pub type SignedProviderRecord = SignedRecord<ProviderRecord>;

impl ProviderRecord {
    /// Create a new [`SignedProviderRecord`] with the current timestamp
    /// and a signed envelope which can be shared with others.
    pub fn signed(
        key: &Keypair,
        subnet_ids: Vec<SubnetID>,
    ) -> anyhow::Result<SignedProviderRecord> {
        let timestamp = Timestamp::now();
        let peer_id = key.public().to_peer_id();
        let record = ProviderRecord {
            peer_id,
            subnet_ids,
            timestamp,
        };
        let signed = SignedRecord::new(key, record)?;
        Ok(signed)
    }
}

#[cfg(any(test, feature = "arb"))]
mod arb {
    use libp2p::identity::Keypair;
    use quickcheck::Arbitrary;

    use crate::arb::ArbSubnetID;

    use super::{ProviderRecord, SignedProviderRecord, Timestamp};

    impl Arbitrary for Timestamp {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self(u64::arbitrary(g).saturating_add(1))
        }
    }

    /// Create a valid [`SignedProviderRecord`] with a random key.
    impl Arbitrary for SignedProviderRecord {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // NOTE: Unfortunately the keys themselves are not deterministic, nor is the Timestamp.
            let key = match u8::arbitrary(g) % 2 {
                0 => Keypair::generate_ed25519(),
                _ => Keypair::generate_secp256k1(),
            };

            // Limit the number of subnets and the depth of keys so data generation doesn't take too long.
            let mut subnet_ids = Vec::new();
            for _ in 0..u8::arbitrary(g) % 5 {
                let subnet_id = ArbSubnetID::arbitrary(g);
                subnet_ids.push(subnet_id.0)
            }

            ProviderRecord::signed(&key, subnet_ids).expect("error creating signed envelope")
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::core::SignedEnvelope;
    use quickcheck_macros::quickcheck;

    use super::SignedProviderRecord;

    #[quickcheck]
    fn prop_roundtrip(signed_record: SignedProviderRecord) -> bool {
        let envelope_bytes = signed_record.envelope.into_protobuf_encoding();

        let envelope =
            SignedEnvelope::from_protobuf_encoding(&envelope_bytes).expect("envelope roundtrip");

        let signed_record2 =
            SignedProviderRecord::from_signed_envelope(envelope).expect("record roundtrip");

        signed_record2.record == signed_record.record
    }

    #[quickcheck]
    fn prop_tamper_proof(signed_record: SignedProviderRecord, idx: usize) -> bool {
        let mut envelope_bytes = signed_record.envelope.into_protobuf_encoding();
        // Do some kind of mutation to a random byte in the envelope; after that it should not validate.
        let idx = idx % envelope_bytes.len();
        envelope_bytes[idx] = u8::MAX - envelope_bytes[idx];

        match SignedEnvelope::from_protobuf_encoding(&envelope_bytes) {
            Err(_) => true, // Corrupted the protobuf itself.
            Ok(envelope) => SignedProviderRecord::from_signed_envelope(envelope).is_err(),
        }
    }
}
