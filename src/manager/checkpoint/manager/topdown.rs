// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::manager::checkpoint::CheckpointManager;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::TopDownCheckpoint;
use ipc_sdk::subnet_id::SubnetID;

pub struct TopDownCheckpointManager;
use async_trait::async_trait;

#[async_trait]
impl CheckpointManager for TopDownCheckpointManager {
    type Checkpoint = TopDownCheckpoint;

    async fn obtain_validators(&self) -> anyhow::Result<Vec<Address>> {
        todo!()
    }

    fn parent_subnet(&self) -> &SubnetID {
        todo!()
    }

    fn child_subnet(&self) -> &SubnetID {
        todo!()
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        todo!()
    }

    async fn submit_checkpoint(
        &self,
        _epoch: ChainEpoch,
        _previous_epoch: ChainEpoch,
        _validator: &Address,
    ) -> anyhow::Result<Self::Checkpoint> {
        todo!()
    }

    async fn next_submission_epoch(
        &self,
        _validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>> {
        todo!()
    }
}
