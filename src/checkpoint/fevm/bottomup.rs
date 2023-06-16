// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::CheckpointManager;
use crate::config::Subnet;
use crate::manager::EthManager;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::{Display, Formatter};

pub struct BottomUpCheckpointManager<T> {
    parent_subnet: Subnet,
    parent_manager: T,
    child_subnet: Subnet,
    child_manager: T,
    checkpoint_period: ChainEpoch,
}

impl<T> Display for BottomUpCheckpointManager<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fevm bottom-up, parent: {:}, child: {:}",
            self.parent_subnet.id, self.child_subnet.id
        )
    }
}

#[async_trait]
impl<T: EthManager + Send + Sync> CheckpointManager for BottomUpCheckpointManager<T> {
    fn parent_subnet(&self) -> &Subnet {
        &self.parent_subnet
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child_subnet
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn child_validators(&self) -> anyhow::Result<Vec<Address>> {
        self.child_manager.validators(&self.child_subnet.id).await
    }

    /// The last executed voting epoch for bottom up checkpoint, the value should be fetch from
    /// parent gateway.
    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_manager
            .gateway_last_voting_executed_epoch()
            .await
    }

    /// Bottom up checkpoint submission, we should be focusing on the child subnet's current block
    /// number/chain epoch
    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        // TODO: when we support more wallet addresses, we need this variable
        _validator: &Address,
    ) -> anyhow::Result<()> {
        let checkpoint = self.child_manager.bottom_up_checkpoint(epoch).await?;
        self.child_manager
            .submit_bottom_up_checkpoint(checkpoint)
            .await?;
        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        self.parent_manager
            .has_voted_in_subnet(&self.child_subnet.id, epoch, validator)
            .await
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}
