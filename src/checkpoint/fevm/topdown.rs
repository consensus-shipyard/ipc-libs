// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::checkpoint::CheckpointManager;
use crate::config::Subnet;
use crate::manager::EthManager;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::{Display, Formatter};

pub struct TopdownCheckpointManager<T> {
    parent_subnet: Subnet,
    parent_manager: T,
    child_subnet: Subnet,
    child_manager: T,
    checkpoint_period: ChainEpoch,
}

impl<T> Display for TopdownCheckpointManager<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fevm top-down, parent: {:}, child: {:}",
            self.parent_subnet.id, self.child_subnet.id
        )
    }
}

#[async_trait]
impl<T: EthManager + Send + Sync> CheckpointManager for TopdownCheckpointManager<T> {
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
        // Current solidity contract needs to support batch query
        // pending: https://github.com/LimeChain/filecoin-ipc-actors-fevm/issues/81
        todo!()
    }

    /// The last executed voting epoch for top down checkpoint, the value should be fetch from
    /// child gateway.
    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_manager
            .gateway_last_voting_executed_epoch()
            .await
    }

    /// Top down checkpoint submission, we should be focusing on the parent subnet's current block
    /// number/chain epoch
    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        _epoch: ChainEpoch,
        _validator: &Address,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn should_submit_in_epoch(
        &self,
        _validator: &Address,
        _epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        todo!()
    }
}
