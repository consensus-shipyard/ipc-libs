// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::proof::create_proof;
use crate::checkpoint::CheckpointManager;
use crate::config::Subnet;
use crate::lotus::LotusClient;
use crate::manager::EthManager;
use anyhow::anyhow;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::{Display, Formatter};

/// Bottom up checkpoint manager. It reads the state of child subnet, FVM, and commits to parent subnet,
/// FEVM.
pub struct BottomUpCheckpointManager<ParentManager, ChildManager> {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    parent_fevm_manager: ParentManager,
    child_fvm_manager: ChildManager,
}

impl<P, M> Display for BottomUpCheckpointManager<P, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fvm to fevm bottom-up, parent: {:}, child: {:}",
            self.parent.id, self.child.id
        )
    }
}

#[async_trait]
impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> CheckpointManager
    for BottomUpCheckpointManager<P, C>
{
    fn parent_subnet(&self) -> &Subnet {
        &self.parent
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn child_validators(&self) -> anyhow::Result<Vec<Address>> {
        self.parent_fevm_manager.validators(&self.child.id).await
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_fevm_manager
            .gateway_last_voting_executed_epoch()
            .await
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_fvm_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        _validator: &Address,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Getting fevm to fvm checkpoint bottom-up template for {epoch:} in subnet: {:?}",
            self.child.id
        );

        let template = self.child_fvm_manager
            .ipc_get_checkpoint_template(&self.child.gateway_addr(), epoch)
            .await
            .map_err(|e| {
                anyhow!(
                    "error getting bottom-up checkpoint template for epoch:{epoch:} in subnet: {:?} due to {e:}",
                    self.child.id
                )
            })?;

        let mut checkpoint =
            crate::manager::evm::subnet_contract::BottomUpCheckpoint::try_from(template)?;

        let proof = create_proof(&self.child_fvm_manager, epoch).await?;
        let proof_bytes = cbor::serialize(&proof, "fevm-fvm bottom up checkpoint proof")?.to_vec();
        checkpoint.proof = ethers::types::Bytes::from(proof_bytes);

        let prev_epoch = epoch - self.checkpoint_period;
        checkpoint.prev_hash = self
            .parent_fevm_manager
            .prev_bottom_up_checkpoint_hash(prev_epoch)
            .await?;

        self.parent_fevm_manager
            .submit_bottom_up_checkpoint(checkpoint)
            .await?;
        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        self.parent_fevm_manager
            .has_voted_in_subnet(&self.child.id, epoch, validator)
            .await
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}