// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::infra::subnet::SubnetNode;
use crate::infra::DEFAULT_MIN_STAKE;
use anyhow::anyhow;
use ipc_agent::cli::wallet::WalletImport;
use ipc_agent::cli::{
    CommandLineHandler, CreateSubnet, CreateSubnetArgs, GlobalArguments, JoinSubnet, JoinSubnetArgs,
};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

/// Create a new subnet in the actor
pub async fn create_subnet(
    ipc_agent_url: String,
    from: String,
    parent: String,
    name: String,
    min_validators: u64,
) -> anyhow::Result<String> {
    let args = CreateSubnetArgs {
        ipc_agent_url: Some(ipc_agent_url),
        from: Some(from),
        parent,
        name,
        min_validator_stake: DEFAULT_MIN_STAKE,
        min_validators,
        bottomup_check_period: 10,
        topdown_check_period: 10,
    };

    CreateSubnet::create(&GlobalArguments::default(), &args).await
}

/// Join the subnet
pub async fn join_subnet(
    ipc_agent_url: String,
    from: String,
    subnet: String,
    collateral: f64,
    validator_net_addr: String,
) -> anyhow::Result<()> {
    let join = JoinSubnetArgs {
        ipc_agent_url: Some(ipc_agent_url),
        from: Some(from),
        subnet,
        collateral,
        validator_net_addr,
    };

    JoinSubnet::handle(&GlobalArguments::default(), &join).await
}

/// Send token to the target address. Not that the `from` wallet address is not specified as it is
/// implied from the `lotus_path`.
pub fn send_token(
    eudico_binary_path: &str,
    lotus_path: &str,
    addr: &str,
    amount: u8,
) -> anyhow::Result<()> {
    let status = Command::new(eudico_binary_path)
        .args(["send", addr, &amount.to_string()])
        .env("LOTUS_PATH", lotus_path)
        .status()?;

    if status.success() {
        log::info!("funded wallet: {:} with amount: {:} fil", addr, amount);
        Ok(())
    } else {
        Err(anyhow!("cannot send token to wallet:{:}", addr))
    }
}

/// Fund the wallet addresses associated with the nodes
pub fn fund_wallet_in_nodes(
    eudico_binary_path: &str,
    lotus_path: &str,
    nodes: &[SubnetNode],
    amount: u8,
) -> anyhow::Result<()> {
    for node in nodes.iter() {
        send_token(
            eudico_binary_path,
            lotus_path,
            node.wallet_address.as_ref().unwrap(),
            amount,
        )?;
        // for nonce to be updated
        sleep(Duration::from_secs(5));
    }
    Ok(())
}

/// Create a new wallet address for the node
pub fn create_wallet(node: &mut SubnetNode) -> anyhow::Result<()> {
    loop {
        match node.new_wallet_address() {
            Ok(_) => {
                log::info!("one wallet created in node: {:?}", node.id);
                break;
            }
            Err(e) => {
                log::warn!("cannot create wallet: {e:}, wait and sleep to retry");
                sleep(Duration::from_secs(10))
            }
        }
    }

    Ok(())
}

pub async fn import_wallet(ipc_agent_url: &str, private_key: String) -> anyhow::Result<()> {
    let params: ipc_agent::cli::wallet::LotusJsonKeyType = serde_json::from_str(&private_key)?;
    WalletImport::import(&params, ipc_agent_url.parse()?).await?;
    Ok(())
}

/// Filter and get the tcp address, input must contain tcp address
pub fn tcp_address(addrs: Vec<String>) -> anyhow::Result<String> {
    addrs
        .into_iter()
        .find(|a| a.contains("tcp"))
        .ok_or_else(|| anyhow!("no tcp address found"))
}

pub fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
