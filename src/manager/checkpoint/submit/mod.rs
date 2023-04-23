mod timeout;

// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use serde::Serialize;

/// The submission strategy. Depending on the speed requirement, the strategy to submit the checkpoint
/// can vary. For example, we can have checkpoint submission without waiting for the message to be
/// executed, or we can have checkpoint submission that waits for a specific timeout period.
#[async_trait]
pub trait SubmissionStrategy {
    /// Submit the checkpoint to the subnet.
    ///
    /// # Argument
    /// `subnet_id` The subnet id to submit checkpoint to
    /// `epoch` The epoch that the checkpoint is targeted
    /// `validator` The validator that the checkpoint is submitted from
    /// `checkpoint` The checkpoint to submit
    async fn submit_checkpoint<T: Serialize>(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
        checkpoint: T,
    ) -> Result<()>;
}
