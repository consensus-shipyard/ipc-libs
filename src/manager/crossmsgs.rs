// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use anyhow::Result;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::CrossMsg;
use crate::jsonrpc::JsonRpcClient;

use crate::lotus::client::LotusJsonRPCClient;

// Prototype function for submitting topdown messages. This function is supposed to be called each
// Nth epoch of a parent subnet. It reads the topdown messages from the parent subnet and submits
// them to the child subnet. We use stubs instead of actual existing JSON-RPC methods.
fn submit_topdown_msgs<T: JsonRpcClient + Send + Sync>(
    parent_epoch: ChainEpoch,
    account: &Address,
    child_client: &LotusJsonRPCClient<T>,
    parent_client: &LotusJsonRPCClient<T>,
) -> Result<()> {
    // First, we read from the child subnet the nonce of the last topdown message there. We
    // increment the result by one to obtain the nonce of the first topdown message we want to
    // submit to the child subnet.
    let nonce = child_client_read_last_topdown_nonce()? + 1;

    // Then, we read from the parent subnet the topdown messages with nonce greater than or equal
    // to the nonce we just obtained.
    let topdown_msgs = parent_client.read_topdown_msgs(nonce)?;

    // Finally, we submit the topdown messages to the child subnet.
    child_client_submit_topdown_msgs(topdown_msgs)?;

    Ok(())
}

// Stubs
// Reads the nonce of the last applied topdown message on the child subnet.
fn child_client_read_last_topdown_nonce() -> Result<u64> {
    Ok(0)
}

// Reads the topdown messages with nonce greater than or equal to the given nonce from the parent.
fn parent_client_read_topdown_msgs() -> Result<Vec<CrossMsg>> {
    Ok(vec![])
}

// Submits the given topdown messages to the child subnet.
fn child_client_submit_topdown_msgs(msgs: Vec<CrossMsg>) -> Result<()> {
    Ok(())
}

