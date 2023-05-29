// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use ipc_agent::sdk::IpcAgentClient;

const IPC_AGENT_JSON_RPC_URL_ENV: &str = "IPC_AGENT_JSON_RPC_URL";
const CHILD_SUBNET_ID_STR_ENV: &str = "CHILD_SUBNET_ID_STR";
const FUND_ADDRESS_ENV: &str = "FUND_ADDRESS";

/// This is a check to ensure the validators have all registered themselves in the parent.
#[tokio::test]
async fn verify_child_subnet_memberships() {}

#[tokio::test]
async fn verify_checkpoints_submitted() {
    let url = std::env::var(IPC_AGENT_JSON_RPC_URL_ENV)
        .unwrap()
        .parse()
        .unwrap();
    let subnet = std::env::var(CHILD_SUBNET_ID_STR_ENV).unwrap();

    let ipc_client = IpcAgentClient::default_from_url(url);

    let epoch = ipc_client.last_top_down_executed(&subnet).await.unwrap();
    assert!(epoch > 0, "no top down message executed yet");

    // at least get the first 10 epoches, this should be the very first bottome up checkpoints
    let checkpoints = ipc_client
        .list_bottom_up_checkpoints(&subnet, 0, 10)
        .await
        .unwrap();
    assert!(
        !checkpoints.is_empty(),
        "no bottom up checkpoints executed yet"
    );
}

#[tokio::test]
async fn test_fund_and_release() {
    let url = std::env::var(IPC_AGENT_JSON_RPC_URL_ENV)
        .unwrap()
        .parse()
        .unwrap();
    let ipc_client = IpcAgentClient::default_from_url(url);

    let subnet = std::env::var(CHILD_SUBNET_ID_STR_ENV).unwrap();
    let addr = std::env::var(FUND_ADDRESS_ENV).unwrap();
    let amount = 2.5;

    let fund_epoch = ipc_client
        .fund(&subnet, Some(addr.clone()), amount)
        .await
        .unwrap();
    loop {
        let epoch = ipc_client.last_top_down_executed(&subnet).await.unwrap();
        if epoch > fund_epoch {
            log::info!("fund epoch reached: {epoch:}");
            break;
        }
    }

    let epoch = ipc_client
        .release(&subnet, Some(addr.clone()), amount)
        .await
        .unwrap();
    loop {
        let checkpoints = ipc_client
            .list_bottom_up_checkpoints(&subnet, epoch, epoch + 1)
            .await
            .unwrap();
        if !checkpoints.is_empty() {
            log::info!("released in epoch: {epoch:}");
            break;
        }
    }
}
