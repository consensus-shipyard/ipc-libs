use crate::manager::checkpoint::submit::SubmissionStrategy;
use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug)]
pub enum CheckpointType {
    TopDown,
    BottomUp,
}

/// Checkpoint manager that handles a specific parent - child - checkpoint type tuple.
/// For example, we might have `/root` subnet and `/root/t01` as child, one implementation of manager
/// is handling the top-down checkpoint submission for `/root` to `/root/t01`.
#[async_trait]
pub trait CheckpointManager {
    /// The type of the checkpoint to submit
    type Checkpoint: Debug + Serialize + Send;
    /// The submission strategy used
    type SubmissionStrategy: SubmissionStrategy + Send + Sync;

    /// Getter for the parent subnet this checkpoint manager is handling
    fn parent_subnet(&self) -> &SubnetID;

    /// Getter for the target subnet this checkpoint manager is handling
    fn child_subnet(&self) -> &SubnetID;

    /// Get the checkpoint type
    fn checkpoint_type(&self) -> CheckpointType;

    /// The checkpoint period to submit
    fn checkpoint_period(&self) -> ChainEpoch;

    /// The submit to memory pool strategy, i.e. determines if the submission to memory pool will
    /// wait for the execution to complete or wait for a specific timeout.
    fn submission_strategy(&self) -> &Self::SubmissionStrategy;

    async fn sync_checkpoint_period(&self) -> Result<()>;

    async fn obtain_validators(&self) -> Result<Vec<Address>>;

    /// Creates the checkpoint based on the current epoch to submit and the previous epoch that was
    /// already submitted.
    async fn create_checkpoint(
        &self,
        epoch: ChainEpoch,
        previous_epoch: ChainEpoch,
    ) -> Result<Self::Checkpoint>;

    /// Derive the next epoch to submit checkpoint for the validator in the defined
    /// parent-child subnet pair.
    async fn next_submission_epoch(
        &self,
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>>;
}
