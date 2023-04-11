//! Submit checkpoint policies

mod optimistic;

use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;

pub use optimistic::OptimisticCheckpointPolicy;

/// The different policies to submit checkpoints for each parent-child subnet pair
#[async_trait]
pub trait CheckpointPolicy {
    /// The subnet to submit votes to
    fn subnet(&self) -> &SubnetID;

    /// Derive the next epoch to submit checkpoint to
    async fn next_submission_epoch(&self, validator: &Address) -> anyhow::Result<ChainEpoch>;

    /// Submit the checkpoint
    async fn submit_checkpoint(
        &self,
        validator: Address,
        checkpoint: Checkpoint,
    ) -> anyhow::Result<()>;
}
