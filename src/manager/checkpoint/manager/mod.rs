// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod bottomup;
pub mod topdown;

use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use serde::Serialize;
use std::fmt::Debug;

/// Checkpoint manager that handles a specific parent - child - checkpoint type tuple.
/// For example, we might have `/root` subnet and `/root/t01` as child, one implementation of manager
/// is handling the top-down checkpoint submission for `/root` to `/root/t01`.
#[async_trait]
pub trait CheckpointManager {
    /// The type of the checkpoint to submit
    type Checkpoint: Debug + Serialize + Send;

    /// Getter for the parent subnet this checkpoint manager is handling
    fn parent_subnet(&self) -> &SubnetID;

    /// Getter for the target subnet this checkpoint manager is handling
    fn child_subnet(&self) -> &SubnetID;

    /// The checkpoint period to submit
    fn checkpoint_period(&self) -> ChainEpoch;

    async fn sync_checkpoint_period(&self) -> Result<()>;

    async fn obtain_validators(&self) -> Result<Vec<Address>>;

    /// Creates the checkpoint based on the current epoch to submit and the previous epoch that was
    /// already submitted.
    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        previous_epoch: ChainEpoch,
        validator: &Address,
    ) -> Result<Self::Checkpoint>;

    /// Derive the next epoch to submit checkpoint for the validator in the defined
    /// parent-child subnet pair.
    async fn next_submission_epoch(
        &self,
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>>;
}
