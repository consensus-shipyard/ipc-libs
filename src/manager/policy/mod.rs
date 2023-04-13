// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Submit checkpoint policies

mod sequential;

use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;

pub use sequential::SequentialCheckpointPolicy;

/// The different policies to submit checkpoints for each parent-child subnet pair
#[async_trait]
pub trait CheckpointPolicy {
    /// The subnet to submit votes to
    fn subnet(&self) -> &SubnetID;

    /// Derive the next epoch to submit checkpoint to
    async fn next_submission_epoch(
        &self,
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>>;

    /// Submit the checkpoint
    async fn submit_checkpoint(
        &self,
        validator: Address,
        checkpoint: BottomUpCheckpoint,
    ) -> anyhow::Result<()>;
}