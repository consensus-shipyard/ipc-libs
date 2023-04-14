// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::MethodNum;
use ipc_gateway::checkpoint::checkpoint_epoch;
use ipc_gateway::TopDownCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use tokio::select;
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::config::Subnet;
use crate::constants::GATEWAY_ACTOR_ADDRESS;
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::CHAIN_HEAD_REQUEST_PERIOD;

pub async fn manage_topdown_checkpoints(
    (child, parent): (Subnet, Subnet),
    stop_notify: Arc<Notify>,
) -> Result<()> {
    log::info!(
        "Starting top-down checkpoint manager for (child, parent) subnet pair ({}, {})",
        child.id,
        parent.id
    );

    let child_client = LotusJsonRPCClient::from_subnet(&child);
    let parent_client = LotusJsonRPCClient::from_subnet(&parent);

    let result: Result<()> = try {
        // Read the parent's chain head and obtain the tip set CID.
        log::debug!("Getting parent tipset");
        let parent_head = parent_client.chain_head().await?;
        let cid_map = parent_head.cids.first().unwrap().clone();
        let parent_tip_set = Cid::try_from(cid_map)?;

        // Read the parent's chain head and obtain the topdown checkpoint period.
        let state = parent_client.ipc_read_gateway_state(parent_tip_set).await?;
        let period = state.topdown_check_period;

        loop {
            let parent_head = parent_client.chain_head().await?;
            let curr_epoch: ChainEpoch = ChainEpoch::try_from(parent_head.height)?;
            // next epoch to submit.
            let epoch = checkpoint_epoch(curr_epoch, period);

            // First, we check which accounts are in the validator set. This is done by reading
            // the parent's chain head and requesting the state at that tip set.
            let parent_head = parent_client.chain_head().await?;
            assert_eq!(parent_head.cids.len(), 1);
            let cid_map = parent_head.cids.first().unwrap().clone();
            let parent_tip_set = Cid::try_from(cid_map)?;

            let subnet_actor_state = parent_client
                .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
                .await?;

            let mut validator_set: HashSet<Address, RandomState> = HashSet::new();
            match subnet_actor_state.validator_set.validators {
                None => {}
                Some(validators) => {
                    for v in validators {
                        validator_set.insert(Address::from_str(v.addr.deref())?);
                    }
                }
            };

            // For each account that we manage that is in the validator set, we submit a topdown
            // checkpoint.
            for account in child.accounts.iter() {
                if validator_set.contains(account) {
                    log::debug!("Submitting topdown checkpoint for account {}", account);
                    submit_topdown_checkpoint(
                        epoch,
                        account,
                        child.id.clone(),
                        &child_client,
                        &parent_client,
                    )
                    .await?;
                }
            }

            // Sleep for an appropriate amount of time before checking the chain head again or return
            // if a stop notification is received.
            select! {
                _ = sleep(CHAIN_HEAD_REQUEST_PERIOD) => { continue }
                _ = stop_notify.notified() => { break }
            }
        }
    };

    result.context(format!(
        "error in manage_topdown_checkpoints() for subnet pair ({}, {})",
        parent.id, child.id
    ))
}

// Prototype function for submitting topdown messages. This function is supposed to be called each
// Nth epoch of a parent subnet. It reads the topdown messages from the parent subnet and submits
// them to the child subnet.
async fn submit_topdown_checkpoint<T: JsonRpcClient + Send + Sync>(
    parent_epoch: ChainEpoch,
    account: &Address,
    child_subnet: SubnetID,
    child_client: &LotusJsonRPCClient<T>,
    parent_client: &LotusJsonRPCClient<T>,
) -> Result<()> {
    // First, we read from the child subnet the nonce of the last topdown message there. We
    // increment the result by one to obtain the nonce of the first topdown message we want to
    // submit to the child subnet.
    let child_head = parent_client.chain_head().await?;
    let cid_map = child_head.cids.first().unwrap().clone();
    let child_tip_set = Cid::try_from(cid_map)?;
    let state = child_client.ipc_read_gateway_state(child_tip_set).await?;
    let nonce = state.applied_topdown_nonce + 1;

    // Then, we read from the parent subnet the topdown messages with nonce greater than or equal
    // to the nonce we just obtained.
    let gateway_addr = Address::from_str(GATEWAY_ACTOR_ADDRESS)?;
    let top_down_msgs = parent_client
        .ipc_get_topdown_msgs(&child_subnet, gateway_addr, nonce)
        .await?;

    // Finally, we submit the topdown messages to the child subnet.
    let to = gateway_addr;
    let from = *account;
    let topdown_checkpoint = TopDownCheckpoint {
        epoch: parent_epoch,
        top_down_msgs,
    };
    let message = MpoolPushMessage::new(
        to,
        from,
        ipc_gateway::Method::SubmitTopDownCheckpoint as MethodNum,
        cbor::serialize(&topdown_checkpoint, "topdown_checkpoint")?.to_vec(),
    );
    parent_client.mpool_push_message(message).await?;

    Ok(())
}
