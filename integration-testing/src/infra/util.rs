use crate::infra::subnet::SubnetNode;
use crate::infra::DEFAULT_MIN_STAKE;
use anyhow::anyhow;
use fvm_shared::crypto::signature::SignatureType;
use ipc_agent::config::json_rpc_methods;
use ipc_agent::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use ipc_agent::lotus::message::wallet::WalletKeyType;
use ipc_agent::server::create::{CreateSubnetParams, CreateSubnetResponse};
use ipc_agent::server::join::JoinSubnetParams;
use ipc_agent::server::wallet::import::{WalletImportParams, WalletImportResponse};
use std::process::Command;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

pub async fn create_subnet(
    ipc_agent_url: String,
    from: String,
    parent: String,
    name: String,
    min_validators: u64,
) -> anyhow::Result<String> {
    let json_rpc_client = JsonRpcClientImpl::new(ipc_agent_url.parse()?, None);

    let params = CreateSubnetParams {
        from: Some(from),
        parent,
        name,
        min_validator_stake: DEFAULT_MIN_STAKE,
        min_validators,
        bottomup_check_period: 10,
        topdown_check_period: 10,
    };

    Ok(json_rpc_client
        .request::<CreateSubnetResponse>(
            json_rpc_methods::CREATE_SUBNET,
            serde_json::to_value(params)?,
        )
        .await?
        .address)
}

pub async fn join_subnet(
    ipc_agent_url: String,
    from: String,
    subnet: String,
    collateral: f64,
    validator_net_addr: String,
) -> anyhow::Result<()> {
    let json_rpc_client = JsonRpcClientImpl::new(ipc_agent_url.parse()?, None);

    // The json rpc server will handle directing the request to
    // the correct parent.
    let params = JoinSubnetParams {
        subnet,
        from: Some(from),
        collateral,
        validator_net_addr,
    };

    json_rpc_client
        .request::<()>(json_rpc_methods::JOIN_SUBNET, serde_json::to_value(params)?)
        .await?;
    Ok(())
}

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

pub fn fund_nodes(
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
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LotusJsonKeyType {
        r#type: String,
        private_key: String,
    }

    let json_rpc_client = JsonRpcClientImpl::new(ipc_agent_url.parse()?, None);

    let params: LotusJsonKeyType = serde_json::from_str(&private_key)?;
    json_rpc_client
        .request::<WalletImportResponse>(
            json_rpc_methods::WALLET_IMPORT,
            serde_json::to_value(WalletImportParams {
                key_type: SignatureType::try_from(WalletKeyType::from_str(&params.r#type)?)? as u8,
                private_key: params.private_key,
            })?,
        )
        .await?;

    Ok(())
}

/// Filter and get the tcp address, input must contain tcp address
pub fn tcp_address(addrs: Vec<String>) -> anyhow::Result<String> {
    addrs
        .into_iter()
        .filter(|a| a.contains("tcp"))
        .next()
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
