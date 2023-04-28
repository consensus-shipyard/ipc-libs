// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::lotus::client::DefaultLotusJsonRPCClient;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::CheckpointManager;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;

pub struct BottomUpCheckpointManager {
    parent_subnet: SubnetID,
    parent_client: DefaultLotusJsonRPCClient,
    child_subnet: SubnetID,
    child_client: DefaultLotusJsonRPCClient,

    checkpoint_period: ChainEpoch,
}

impl BottomUpCheckpointManager {
    pub fn new_with_period(
        parent_subnet: SubnetID,
        parent_client: DefaultLotusJsonRPCClient,
        child_subnet: SubnetID,
        child_client: DefaultLotusJsonRPCClient,
        checkpoint_period: ChainEpoch,
    ) -> Self {
        Self {
            parent_subnet,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
        }
    }

    pub async fn new(
        parent_client: DefaultLotusJsonRPCClient,
        parent_subnet: SubnetID,
        child_client: DefaultLotusJsonRPCClient,
        child_subnet: SubnetID,
    ) -> anyhow::Result<Self> {
        let checkpoint_period = obtain_checkpoint_period(&child_subnet, &parent_client).await?;
        Ok(Self::new_with_period(
            parent_subnet,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
        ))
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let parent_head = self.parent_client.chain_head().await?;

        // A key assumption we make now is that each block has exactly one tip set. We panic
        // if this is not the case as it violates our assumption.
        // TODO: update this logic once the assumption changes (i.e., mainnet)
        assert_eq!(parent_head.cids.len(), 1);

        let cid_map = parent_head.cids.first().unwrap().clone();
        let parent_tip_set = Cid::try_from(cid_map)?;
        // get subnet actor state and last checkpoint executed
        let subnet_actor_state = self
            .parent_client
            .ipc_read_subnet_actor_state(&self.child_subnet, parent_tip_set)
            .await?;

        Ok(subnet_actor_state
            .bottom_up_checkpoint_voting
            .last_voting_executed)
    }
}

#[async_trait]
impl CheckpointManager for BottomUpCheckpointManager {
    type Checkpoint = BottomUpCheckpoint;

    async fn obtain_validators(&self) -> anyhow::Result<Vec<Address>> {
        todo!()
    }

    fn parent_subnet(&self) -> &SubnetID {
        &self.parent_subnet
    }

    fn child_subnet(&self) -> &SubnetID {
        &self.child_subnet
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
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>> {
        log::debug!(
            "attempt to obtain the next submission epoch in bottom up checkpoint for subnet: {:?}",
            self.child_subnet
        );

        let current_epoch = self.child_client.current_epoch().await?;
        let last_executed_epoch = self.last_executed_epoch().await?;

        log::debug!(
            "latest epoch {:?}, last executed epoch: {:?} for bottom up checkpointing in subnet: {:?}",
            current_epoch,
            last_executed_epoch,
            self.child_subnet,
        );

        let next_submission_epoch = last_executed_epoch + self.checkpoint_period;
        if current_epoch < next_submission_epoch {
            log::info!("latest epoch {current_epoch:} lagging next submission epoch {next_submission_epoch:}");
            return Ok(None);
        }

        if self
            .parent_client
            .ipc_has_voted_bu_in_epoch(validator, &self.child_subnet, next_submission_epoch)
            .await?
        {
            log::info!("next submission epoch {next_submission_epoch:} already executed");
            return Ok(None);
        }

        log::debug!("next submission epoch {next_submission_epoch:}");

        Ok(Some(next_submission_epoch))
    }
}

async fn obtain_checkpoint_period(
    subnet_id: &SubnetID,
    parent_client: &DefaultLotusJsonRPCClient,
) -> anyhow::Result<ChainEpoch> {
    log::debug!("Getting the bottom up checkpoint period for subnet: {subnet_id:?}");

    // Read the parent's chain head and obtain the tip set CID.
    let parent_head = parent_client.chain_head().await?;
    let cid_map = parent_head.cids.first().unwrap().clone();
    let parent_tip_set = Cid::try_from(cid_map)?;

    // Extract the checkpoint period from the state of the subnet actor in the parent.
    log::debug!("Get checkpointing period from subnet actor in parent");
    let state = parent_client
        .ipc_read_subnet_actor_state(subnet_id, parent_tip_set)
        .await
        .map_err(|e| {
            log::error!("error getting subnet actor state for {:?}", subnet_id);
            e
        })?;

    Ok(state.bottom_up_check_period)
}
