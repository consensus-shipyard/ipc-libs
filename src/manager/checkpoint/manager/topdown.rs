use std::fmt::{Display, Formatter};
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::manager::checkpoint::CheckpointManager;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

pub struct TopDownCheckpointManager;
use crate::config::Subnet;
use crate::lotus::client::DefaultLotusJsonRPCClient;
use async_trait::async_trait;

impl Display for TopDownCheckpointManager {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[async_trait]
impl CheckpointManager for TopDownCheckpointManager {
    type LotusClient = DefaultLotusJsonRPCClient;

    fn parent_client(&self) -> &Self::LotusClient {
        todo!()
    }

    fn parent_subnet_id(&self) -> &SubnetID {
        todo!()
    }

    fn child_subnet(&self) -> &Subnet {
        todo!()
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        todo!()
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn submit_checkpoint(
        &self,
        _epoch: ChainEpoch,
        _previous_epoch: ChainEpoch,
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
}
