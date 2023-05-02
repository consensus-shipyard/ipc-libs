// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod bottomup;
pub mod topdown;

use crate::config::Subnet;
use crate::lotus::LotusClient;
use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

/// Checkpoint manager that handles a specific parent - child - checkpoint type tuple.
/// For example, we might have `/root` subnet and `/root/t01` as child, one implementation of manager
/// is handling the top-down checkpoint submission for `/root` to `/root/t01`.
#[async_trait]
pub trait CheckpointManager {
    type LotusClient: LotusClient;

    /// The client of the parent
    fn parent_client(&self) -> &Self::LotusClient;

    /// Getter for the parent subnet this checkpoint manager is handling
    fn parent_subnet_id(&self) -> &SubnetID;

    /// Getter for the target subnet this checkpoint manager is handling
    fn child_subnet(&self) -> &Subnet;

    /// The checkpoint period that the current manager is submitting upon
    fn checkpoint_period(&self) -> ChainEpoch;

    /// Submit the checkpoint based on the current epoch to submit and the previous epoch that was
    /// already submitted.
    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        previous_epoch: ChainEpoch,
        validator: &Address,
    ) -> Result<()>;

    /// Derive the next epoch to submit checkpoint for the validator in the defined
    /// parent-child subnet pair.
    async fn next_submission_epoch(
        &self,
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>>;
}
