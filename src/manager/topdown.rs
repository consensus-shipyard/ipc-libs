// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::str::FromStr;

use anyhow::Result;
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::MethodNum;
use ipc_gateway::TopDownCheckpoint;
use ipc_sdk::subnet_id::SubnetID;

use crate::constants::GATEWAY_ACTOR_ADDRESS;
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;

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
    let top_down_msgs = parent_client
        .ipc_get_topdown_msgs(child_subnet, nonce)
        .await?;

    // Finally, we submit the topdown messages to the child subnet.
    let to = Address::from_str(GATEWAY_ACTOR_ADDRESS)?;
    let from = *account;
    let topdown_checkpoint = TopDownCheckpoint { epoch: parent_epoch, top_down_msgs };
    let message = MpoolPushMessage::new(
        to,
        from,
        ipc_gateway::Method::SubmitTopDownCheckpoint as MethodNum,
        cbor::serialize(&topdown_checkpoint, "topdown_checkpoint")?.to_vec(),
    );
    parent_client.mpool_push_message(message).await?;

    Ok(())
}
