// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The bottom up checkpoint related code

use crate::manager::evm::manager::{call_with_premium_estimation, contract_address_from_subnet};
use crate::manager::subnet::BottomUpCheckpointRelayer;
use crate::manager::EthSubnetManager;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_actors_abis::subnet_actor_manager_facet;
use ipc_sdk::checkpoint::{BottomUpCheckpoint, BottomUpCheckpointBundle};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::evm::payload_to_evm_address;
use ipc_sdk::subnet_id::SubnetID;
use std::sync::Arc;

#[async_trait]
impl BottomUpCheckpointRelayer for EthSubnetManager {
    async fn submit_checkpoint(
        &self,
        submitter: &Address,
        bundle: BottomUpCheckpointBundle,
    ) -> anyhow::Result<()> {
        let BottomUpCheckpointBundle {
            checkpoint,
            signatures,
            signatories,
            cross_msgs,
        } = bundle;

        let address = contract_address_from_subnet(&checkpoint.subnet_id)?;
        log::info!(
            "submit bottom up checkpoint: {checkpoint:?} in evm subnet contract: {address:}"
        );

        let signatures = signatures
            .into_iter()
            .map(ethers::types::Bytes::from)
            .collect::<Vec<_>>();
        let signatories = signatories
            .into_iter()
            .map(|addr| payload_to_evm_address(addr.payload()))
            .collect::<Result<Vec<_>, _>>()?;
        let cross_msgs = cross_msgs
            .into_iter()
            .map(subnet_actor_manager_facet::CrossMsg::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let checkpoint = subnet_actor_manager_facet::BottomUpCheckpoint::try_from(checkpoint)?;

        let signer = Arc::new(self.get_signer(submitter)?);
        let contract = subnet_actor_manager_facet::SubnetActorManagerFacet::new(
            address,
            signer.clone(),
        );
        let call = contract.submit_checkpoint(checkpoint, cross_msgs, signatories, signatures);
        call_with_premium_estimation(signer, call)
            .await?
            .send()
            .await?;

        Ok(())
    }

    async fn last_bottom_up_checkpoint_height(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn checkpoint_bundle_at(
        &self,
        subnet_id: &SubnetID,
        height: ChainEpoch,
    ) -> anyhow::Result<BottomUpCheckpointBundle> {
        todo!()
    }
}
