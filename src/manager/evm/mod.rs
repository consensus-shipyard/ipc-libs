// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

mod manager;
mod payload;

use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

use super::subnet::SubnetManager;
pub use manager::EthSubnetManager;
pub use payload::{BottomUpCheckpoint, TopdownCheckpoint};

#[async_trait]
pub trait EthManager: SubnetManager {
    /// Fetches the last executed epoch for voting in the gateway.
    async fn gateway_last_voting_executed_epoch(&self) -> anyhow::Result<ChainEpoch>;

    /// Fetches the last executed epoch for voting in the subnet actor.
    async fn subnet_last_voting_executed_epoch(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch>;

    /// The current epoch/block number of the blockchain that the manager connects to.
    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch>;

    /// Submit top down checkpoint the gateway.
    async fn submit_top_down_checkpoint(
        &self,
        checkpoint: TopdownCheckpoint,
    ) -> anyhow::Result<ChainEpoch>;

    /// Submit bottom up checkpoint to the subnet actor.
    async fn submit_bottom_up_checkpoint(
        &self,
        checkpoint: BottomUpCheckpoint,
    ) -> anyhow::Result<ChainEpoch>;
}
