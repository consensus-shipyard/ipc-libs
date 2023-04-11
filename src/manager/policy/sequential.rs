// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The sequential checkpoint policy. Only when the previous checkpoint is committed, it will attempt to
//! submit the next submittable checkpoint.

use crate::manager::lotus::DefaultSubnetManager;
use crate::manager::policy::CheckpointPolicy;
use crate::manager::subnet::BottomUpCheckpointManager;
use crate::manager::subnet::SubnetChainInfo;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use std::time::Duration;

static SUBMIT_CHECKPOINT_TIMEOUT: Duration = Duration::new(90, 0);

pub struct SequentialCheckpointPolicy<T> {
    parent: SubnetID,
    parent_manager: T,
    /// The child subnet id
    child: SubnetID,
    child_manager: T,
    /// The interval to submit checkpoints
    checkpoint_period: ChainEpoch,
}

impl<T: AsRef<DefaultSubnetManager>> SequentialCheckpointPolicy<T> {
    pub fn new(
        parent: SubnetID,
        child: SubnetID,
        parent_manager: T,
        child_manager: T,
        checkpoint_period: ChainEpoch,
    ) -> Self {
        Self {
            parent,
            parent_manager,
            child,
            child_manager,
            checkpoint_period,
        }
    }
}

#[async_trait]
impl<T: AsRef<DefaultSubnetManager> + Send + Sync> CheckpointPolicy
    for SequentialCheckpointPolicy<T>
{
    fn subnet(&self) -> &SubnetID {
        &self.child
    }

    async fn next_submission_epoch(
        &self,
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>> {
        let child_manager = self.child_manager.as_ref();
        let parent_manager = self.parent_manager.as_ref();
        let child = self.subnet();

        let latest_epoch = child_manager.as_ref().current_epoch(child).await?;
        let latest_executed = parent_manager.last_executed_epoch(child).await?;

        let next_submission_epoch = latest_executed + self.checkpoint_period;
        if latest_epoch < next_submission_epoch {
            return Ok(None);
        }

        if parent_manager
            .has_voted_in_epoch(child, next_submission_epoch, validator)
            .await?
        {
            return Ok(None);
        }

        Ok(Some(next_submission_epoch))
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
            .try_submit_checkpoint(
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
