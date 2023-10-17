// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Bottom up checkpoint manager

use crate::config::Subnet;
use crate::manager::{BottomUpCheckpointRelayer, EthSubnetManager};
use anyhow::{anyhow, Result};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_identity::{EthKeyAddress, PersistentKeyStore};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Tracks the metadata required for both top down and bottom up checkpoint submissions, such as
/// parent/child subnet and checkpoint period.
pub struct CheckpointMetadata {
    parent: Subnet,
    child: Subnet,
    period: ChainEpoch,
}

pub struct BottomUpCheckpointManager<P, C> {
    metadata: CheckpointMetadata,
    parent_handler: P,
    child_handler: C,
}

impl<P: BottomUpCheckpointRelayer, C: BottomUpCheckpointRelayer> BottomUpCheckpointManager<P, C> {
    pub async fn new(
        parent: Subnet,
        child: Subnet,
        parent_handler: P,
        child_handler: C,
    ) -> Result<Self> {
        let period = parent_handler
            .checkpoint_period(&child.id)
            .await
            .map_err(|e| anyhow!("cannot get bottom up checkpoint period: {e}"))?;
        Ok(Self {
            metadata: CheckpointMetadata {
                parent,
                child,
                period,
            },
            parent_handler,
            child_handler,
        })
    }
}

impl BottomUpCheckpointManager<EthSubnetManager, EthSubnetManager> {
    pub async fn new_evm_manager(
        parent: Subnet,
        child: Subnet,
        keystore: Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>,
    ) -> Result<Self> {
        let parent_handler =
            EthSubnetManager::from_subnet_with_wallet_store(&parent, keystore.clone())?;
        let child_handler = EthSubnetManager::from_subnet_with_wallet_store(&child, keystore)?;
        Self::new(parent, child, parent_handler, child_handler).await
    }
}

impl<P: BottomUpCheckpointRelayer, C: BottomUpCheckpointRelayer> Display
    for BottomUpCheckpointManager<P, C>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bottom-up, parent: {:}, child: {:}",
            self.metadata.parent.id, self.metadata.child.id
        )
    }
}

impl<
        P: BottomUpCheckpointRelayer + Send + Sync + 'static,
        C: BottomUpCheckpointRelayer + Send + Sync + 'static,
    > BottomUpCheckpointManager<P, C>
{
    /// Getter for the parent subnet this checkpoint manager is handling
    pub fn parent_subnet(&self) -> &Subnet {
        &self.metadata.parent
    }

    /// Getter for the target subnet this checkpoint manager is handling
    pub fn child_subnet(&self) -> &Subnet {
        &self.metadata.child
    }

    /// The checkpoint period that the current manager is submitting upon
    pub fn checkpoint_period(&self) -> ChainEpoch {
        self.metadata.period
    }

    /// Run the bottom up checkpoint submission daemon in the background
    pub fn run(self, validator: Address, submission_interval: Duration) {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.submit_checkpoint(&validator).await {
                    log::error!("cannot submit checkpoint for validator: {validator} due to {e}");
                }

                tokio::time::sleep(submission_interval).await;
            }
        });
    }

    /// Submit the checkpoint from the target validator address
    pub async fn submit_checkpoint(&self, validator: &Address) -> Result<()> {
        let next_submission_height = self.next_submission_height().await?;
        let current_height = self.child_handler.current_epoch().await?;

        if current_height < next_submission_height {
            return Ok(());
        }

        let bundle = self
            .child_handler
            .checkpoint_bundle_at(next_submission_height)
            .await?;
        log::debug!("bottom up bundle: {bundle:?}");

        self.parent_handler
            .submit_checkpoint(validator, bundle)
            .await
            .map_err(|e| anyhow!("cannot submit bottom up checkpoint due to: {e:}"))?;

        Ok(())
    }

    async fn next_submission_height(&self) -> Result<ChainEpoch> {
        let last_checkpoint_epoch = self
            .parent_handler
            .last_bottom_up_checkpoint_height(&self.metadata.child.id)
            .await
            .map_err(|e| {
                anyhow!("cannot obtain the last bottom up checkpoint height due to: {e:}")
            })?;
        Ok(last_checkpoint_epoch + self.checkpoint_period())
    }
}
