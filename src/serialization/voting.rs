//! The deserialization of Voting struct from lotus json rpc server

use crate::serialization::DeserializeFromJson;
use cid::Cid;
use fvm_shared::clock::ChainEpoch;
use ipc_actor_common::vote::Voting;
use primitives::TCid;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

impl<'de, T: DeserializeOwned> Deserialize<'de> for DeserializeFromJson<Voting<T>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The helper struct that the expected json rpc response from lotus for `Voting`.
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Inner {
            pub genesis_epoch: ChainEpoch,
            pub submission_period: ChainEpoch,
            pub last_voting_executed: ChainEpoch,
            // other fields are not included as we dont need them now
        }

        let inner = Inner::deserialize(deserializer)?;
        Ok(DeserializeFromJson(Voting {
            genesis_epoch: inner.genesis_epoch,
            submission_period: inner.submission_period,
            last_voting_executed_epoch: inner.last_voting_executed,
            executable_epoch_queue: None,
            epoch_vote_submissions: TCid::from(Cid::default()),
            threshold_ratio: (0, 0),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::serialization::DeserializeFromJson;
    use ipc_actor_common::vote::Voting;

    #[test]
    fn test_deserialization() {
        let raw = r#"
        {
            "GenesisEpoch": 10,
            "SubmissionPeriod": 10,
            "LastVotingExecuted": 20
        }
        "#;

        let deserialized = serde_json::from_str::<DeserializeFromJson<Voting<()>>>(raw).unwrap();
        assert_eq!(deserialized.genesis_epoch, 10);
        assert_eq!(deserialized.submission_period, 10);
        assert_eq!(deserialized.last_voting_executed_epoch, 20);
    }
}
