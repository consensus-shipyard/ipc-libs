//! The optimistic checkpoint policy. When the previous checkpoint is submitted, it will attempt to
//! submit the next submittable checkpoint.

use crate::jsonrpc::JsonRpcClient;
use crate::manager::policy::CheckpointPolicy;
use crate::manager::subnet::BottomUpCheckpointManager;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::{BottomUpCheckpoint, Checkpoint};
use ipc_sdk::subnet_id::SubnetID;
use std::thread::sleep;
use std::time::Duration;

static SUBMIT_CHECKPOINT_TIMEOUT: Duration = Duration::new(90, 0);
static PER_EPOCH_WAIT_SEC: u64 = 3;

pub struct OptimisticCheckpointPolicy<T> {
    parent_manager: T,
    /// The child subnet id
    child: SubnetID,
    child_manager: T,
    /// The interval to submit checkpoints
    checkpoint_period: ChainEpoch,
}

impl<T> OptimisticCheckpointPolicy<T> {
    pub fn new(
        child: SubnetID,
        parent_manager: T,
        child_manager: T,
        checkpoint_period: ChainEpoch,
    ) -> Self {
        Self {
            parent_manager,
            child,
            child_manager,
            checkpoint_period,
        }
    }
}

impl<M: BottomUpCheckpointManager + Send + Sync, T: AsRef<M>> CheckpointPolicy
    for OptimisticCheckpointPolicy<T>
{
    fn subnet(&self) -> &SubnetID {
        &self.child
    }

    async fn next_submission_epoch(&self, validator: &Address) -> anyhow::Result<ChainEpoch> {
        let child_manager = self.child_manager.as_ref();
        let parent_manager = self.parent_manager.as_ref();
        let child = self.subnet();

        let next_submission_epoch = loop {
            let latest_epoch = child_manager.as_ref().current_epoch(child).await?;
            let latest_executed = parent_manager.last_executed_epoch(child).await?;

            let next_submission_epoch = latest_executed + self.checkpoint_period;
            if latest_epoch < next_submission_epoch {
                let timeout = PER_EPOCH_WAIT_SEC * (next_submission_epoch - latest_epoch);

                log::info!(
                    "wait for next submission epoch in subnet {:?}, current epoch {:?}",
                    self.subnet(),
                    latest_epoch,
                );

                tokio::time::sleep(Duration::from_secs(timeout)).await;

                continue;
            }

            if parent_manager
                .has_voted_in_epoch(child, next_submission_epoch, validator)
                .await?
            {
                continue;
            }

            break next_submission_epoch;
        };

        Ok(next_submission_epoch)
    }

    /// Just push to subnet, wait for 90 seconds
    async fn submit_checkpoint(
        &self,
        validator: Address,
        checkpoint: BottomUpCheckpoint,
    ) -> anyhow::Result<()> {
        let checkpoint_cid = checkpoint.cid();

        match self
            .parent_manager
            .as_ref()
            .try_submit_bu_checkpoint(
                self.parent.clone(),
                validator,
                checkpoint,
                SUBMIT_CHECKPOINT_TIMEOUT,
            )
            .await?
        {
            None => {
                log::info!("checkpoint: {checkpoint_cid:} submitted");
            }
            Some(message_cid) => {
                log::info!("pushed checkpoint: {checkpoint_cid:} to mem pool with message cid: {message_cid:}");
            }
        }
        Ok(())
    }
}
