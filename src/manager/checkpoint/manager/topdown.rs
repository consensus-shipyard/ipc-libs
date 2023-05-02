use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicI64, Ordering};
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::manager::checkpoint::CheckpointManager;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

use crate::config::Subnet;
use crate::lotus::client::DefaultLotusJsonRPCClient;
use crate::lotus::message::ipc::{IPCReadGatewayStateResponse, IPCReadSubnetActorStateResponse};
use crate::lotus::LotusClient;
use async_trait::async_trait;
use cid::Cid;

pub struct TopDownCheckpointManager {
    parent: SubnetID,
    parent_client: DefaultLotusJsonRPCClient,
    child_subnet: Subnet,
    child_client: DefaultLotusJsonRPCClient,

    checkpoint_period: ChainEpoch,

    // some cache states
    current_epoch: AtomicI64,
}

impl TopDownCheckpointManager {
    pub async fn new(
        parent_client: DefaultLotusJsonRPCClient,
        parent: SubnetID,
        child_client: DefaultLotusJsonRPCClient,
        child_subnet: Subnet,
    ) -> anyhow::Result<Self> {
        let checkpoint_period = obtain_checkpoint_period(&child_subnet.id, &child_client).await?;
        Ok(Self {
            parent,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
            current_epoch: Default::default(),
        })
    }
}

impl Display for TopDownCheckpointManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "top-down, parent: {:}, child: {:}",
            self.parent, self.child_subnet.id
        )
    }
}

impl TopDownCheckpointManager {
    async fn sync_subnet_gateway_state(&self) -> anyhow::Result<IPCReadGatewayStateResponse> {
        let child_head = self.child_client.chain_head().await?;
        let cid_map = child_head.cids.first().unwrap().clone();
        let child_tip_set = Cid::try_from(cid_map)?;

        self.child_client
            .ipc_read_gateway_state(child_tip_set)
            .await
    }
}

#[async_trait]
impl CheckpointManager for TopDownCheckpointManager {
    type LotusClient = DefaultLotusJsonRPCClient;

    fn parent_client(&self) -> &Self::LotusClient {
        &self.parent_client
    }

    fn parent_subnet_id(&self) -> &SubnetID {
        &self.parent
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child_subnet
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let child_gw_state = self.sync_subnet_gateway_state().await?;
        Ok(child_gw_state
            .top_down_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let parent_head = self.parent_client.chain_head().await?;
        Ok(ChainEpoch::try_from(parent_head.height)?)
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

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        let state = self.sync_subnet_gateway_state().await?;
        Ok(state.initialized)
    }
}

async fn obtain_checkpoint_period(
    subnet_id: &SubnetID,
    child_client: &DefaultLotusJsonRPCClient,
) -> anyhow::Result<ChainEpoch> {
    log::debug!("Getting the top down checkpoint period for subnet: {subnet_id:?}");

    // Read the child's chain head and obtain the tip set CID.
    log::debug!("Getting child tipset and starting top-down checkpointing manager");
    let child_head = child_client.chain_head().await?;
    let cid_map = child_head.cids.first().unwrap().clone();
    let child_tip_set = Cid::try_from(cid_map)?;

    // Read the child's chain head and obtain the topdown checkpoint period
    // and genesis epoch.
    let state = child_client.ipc_read_gateway_state(child_tip_set).await?;
    Ok(state.top_down_check_period)
}
