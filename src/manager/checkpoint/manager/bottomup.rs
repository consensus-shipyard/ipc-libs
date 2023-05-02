// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::Subnet;
use crate::lotus::client::DefaultLotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::CheckpointManager;
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::MethodNum;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use primitives::TCid;

pub struct BottomUpCheckpointManager {
    parent: SubnetID,
    parent_client: DefaultLotusJsonRPCClient,
    child_subnet: Subnet,
    child_client: DefaultLotusJsonRPCClient,

    checkpoint_period: ChainEpoch,
}

impl BottomUpCheckpointManager {
    pub fn new_with_period(
        parent_subnet: SubnetID,
        parent_client: DefaultLotusJsonRPCClient,
        child_subnet: Subnet,
        child_client: DefaultLotusJsonRPCClient,
        checkpoint_period: ChainEpoch,
    ) -> Self {
        Self {
            parent: parent_subnet,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
        }
    }

    pub async fn new(
        parent_client: DefaultLotusJsonRPCClient,
        parent: SubnetID,
        child_client: DefaultLotusJsonRPCClient,
        child_subnet: Subnet,
    ) -> anyhow::Result<Self> {
        let checkpoint_period = obtain_checkpoint_period(&child_subnet.id, &parent_client).await?;
        Ok(Self::new_with_period(
            parent,
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
            .ipc_read_subnet_actor_state(&self.child_subnet.id, parent_tip_set)
            .await?;

        Ok(subnet_actor_state
            .bottom_up_checkpoint_voting
            .last_voting_executed)
    }
}

#[async_trait]
impl CheckpointManager for BottomUpCheckpointManager {
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

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        _previous_epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        let mut checkpoint = BottomUpCheckpoint::new(self.child_subnet.id.clone(), epoch);

        // From the template on the gateway actor of the child subnet, we get the children checkpoints
        // and the bottom-up cross-net messages.
        log::debug!(
            "Getting checkpoint bottom-up template for {epoch:} in subnet: {:?}",
            self.child_subnet.id
        );
        let template = self
            .child_client
            .ipc_get_checkpoint_template(epoch)
            .await
            .map_err(|e| {
                log::error!(
                "error getting bottom-up checkpoint template for epoch:{epoch:} in subnet: {:?}",
                self.child_subnet.id
            );
                e
            })?;
        checkpoint.data.children = template.data.children;
        checkpoint.data.cross_msgs = template.data.cross_msgs;

        log::info!(
            "checkpoint at epoch {:} contains {:} number of cross messages",
            checkpoint.data.epoch,
            checkpoint
                .data
                .cross_msgs
                .cross_msgs
                .as_ref()
                .map(|s| s.len())
                .unwrap_or_default()
        );

        // Get the CID of previous checkpoint of the child subnet from the gateway actor of the parent
        // subnet.
        log::debug!(
            "Getting previous checkpoint bottom-up from parent gateway for {epoch:} in subnet: {:?}",
            self.child_subnet.id
        );
        let response = self
            .parent_client
            .ipc_get_prev_checkpoint_for_child(&self.child_subnet.id)
            .await
            .map_err(|e| {
                log::error!(
                "error getting previous bottom-up checkpoint for epoch:{epoch:} in subnet: {:?}",
                self.child_subnet.id
            );
                e
            })?;

        // if previous checkpoint is set
        if response.is_some() {
            let cid = Cid::try_from(response.unwrap())?;
            checkpoint.data.prev_check = TCid::from(cid);
        }

        let child_chain_head = self.child_client.chain_head().await?;
        let child_tip_set = child_chain_head
            .cids
            .first()
            .ok_or_else(|| anyhow!("chain head has empty cid: {:}", self.child_subnet.id))?;
        checkpoint.data.proof = Cid::try_from(child_tip_set)?.to_bytes();

        let to = self.child_subnet.id.subnet_actor();
        let message = MpoolPushMessage::new(
            to,
            *validator,
            ipc_subnet_actor::Method::SubmitCheckpoint as MethodNum,
            cbor::serialize(&checkpoint, "checkpoint")?.to_vec(),
        );
        let mem_push_response = self
            .parent_client
            .mpool_push_message(message)
            .await
            .map_err(|e| {
                log::error!(
                    "error submitting checkpoint for epoch {epoch:} in subnet: {:?}",
                    self.child_subnet.id
                );
                e
            })?;

        // wait for the checkpoint to be committed before moving on.
        let message_cid = mem_push_response.cid()?;
        log::debug!("checkpoint message published with cid: {message_cid:?}");

        Ok(())
    }

    async fn next_submission_epoch(
        &self,
        validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>> {
        log::debug!(
            "attempt to obtain the next submission epoch in bottom up checkpoint for subnet: {:?}",
            self.child_subnet.id
        );

        let current_epoch = self.child_client.current_epoch().await?;
        let last_executed_epoch = self.last_executed_epoch().await?;

        log::debug!(
            "latest epoch {:?}, last executed epoch: {:?} for bottom up checkpointing in subnet: {:?}",
            current_epoch,
            last_executed_epoch,
            self.child_subnet.id,
        );

        let next_submission_epoch = last_executed_epoch + self.checkpoint_period;
        if current_epoch < next_submission_epoch {
            log::info!("latest epoch {current_epoch:} lagging next submission epoch {next_submission_epoch:}");
            return Ok(None);
        }

        if self
            .parent_client
            .ipc_has_voted_bu_in_epoch(validator, &self.child_subnet.id, next_submission_epoch)
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
