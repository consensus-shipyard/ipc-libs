// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::{gateway_state, CheckpointManager};
use crate::config::Subnet;
use crate::lotus::LotusClient;
use crate::manager::EthManager;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::{CrossMsg, TopDownCheckpoint};
use std::fmt::{Display, Formatter};

/// Top down checkpoint manager. It reads the state of parent subnet, FEVM, and commits to child subnet,
/// FVM.
#[warn(dead_code)]
pub struct TopDownCheckpointManager<ParentManager, ChildManager> {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    parent_fevm_manager: ParentManager,
    child_fvm_manager: ChildManager,
}

impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> Display
    for TopDownCheckpointManager<P, C>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fevm to fvm top-down, parent: {:}, child: {:}",
            self.parent.id, self.child.id
        )
    }
}

#[async_trait]
impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> CheckpointManager
    for TopDownCheckpointManager<P, C>
{
    fn parent_subnet(&self) -> &Subnet {
        &self.parent
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn child_validators(&self) -> anyhow::Result<Vec<Address>> {
        self.parent_fevm_manager.validators(&self.child.id).await
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let child_gw_state = gateway_state(&self.child_fvm_manager, &self.child).await?;
        Ok(child_gw_state
            .top_down_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_fevm_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        let msgs = self
            .parent_fevm_manager
            .top_down_msgs(&self.child.id, epoch)
            .await?;

        // we submit the topdown messages to the CHILD subnet.
        let topdown_checkpoint = TopDownCheckpoint {
            epoch,
            top_down_msgs: msgs
                .into_iter()
                .map(CrossMsg::try_from)
                .collect::<anyhow::Result<_>>()?,
        };
        let submitted_epoch = self
            .child_fvm_manager
            .ipc_submit_top_down_checkpoint(
                self.parent.gateway_addr(),
                validator,
                topdown_checkpoint,
            )
            .await?;

        log::debug!(
            "checkpoint at epoch {:} for manager: {:} published with at epoch: {:?}, executed",
            epoch,
            self,
            submitted_epoch,
        );

        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        let has_voted = self
            .child_fvm_manager
            .ipc_validator_has_voted_topdown(&self.child.gateway_addr(), epoch, validator)
            .await
            .map_err(|e| {
                anyhow!("error checking if validator has voted for manager: {self:} due to {e:}")
            })?;

        // we should vote only when the validator has not voted
        Ok(!has_voted)
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        self.parent_fevm_manager.gateway_initialized().await
    }
}