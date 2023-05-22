// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::fs;
use std::fs::File;
use crate::infra::{SubnetTopology, DEFAULT_MIN_STAKE};
use anyhow::{anyhow, Result};
use ipc_agent::config::json_rpc_methods;
use ipc_agent::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use ipc_agent::server::create::{CreateSubnetParams, CreateSubnetResponse};
use ipc_agent::server::join::JoinSubnetParams;
use ipc_sdk::subnet_id::SubnetID;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

/// Spawn child subnet according to the topology
pub async fn spawn_child_subnet(topology: &SubnetTopology) -> anyhow::Result<()> {
    if topology.number_of_nodes == 0 {
        log::info!("no nodes to spawn");
        return Ok(());
    }

    let parent = if let Some(p) = &topology.parent {
        p.to_string()
    } else {
        return Err(anyhow!("parent cannot be None"));
    };

    create_subnet(
        topology.ipc_agent_url(),
        topology.root_address.clone(),
        parent,
        topology.name.clone(),
        topology.number_of_nodes as u64,
    )
    .await?;
    log::info!("created subnet: {:}", topology.id);

    let first_node = spawn_first_node(topology)?;
    let mut nodes = spawn_other_nodes(topology, &first_node)?;

    nodes.push(first_node);

    fund_nodes(
        &topology.root_address,
        &topology.root_lotus_path,
        &nodes,
        10,
    )?;
    log::info!("funded nodes");

    for node in nodes.iter_mut() {
        node.config_validator()?;
        log::info!("configured validator for node: {:?}", node.validator.net_addr);

        node.join_subnet().await?;
        log::info!("validator: {:?} joined subnet: {:}", node.validator.net_addr, node.id);

        node.spawn_validator()?;
        log::info!("validator: {:?} spawned", node.validator.net_addr);
    }

    let accounts = nodes.iter()
        .map(|n| n.wallet_address.clone().unwrap())
        .map(|s| format!("\"{:}\"", s))
        .collect::<Vec<_>>()
        .join(",");
    println!("accounts: {accounts:?}");

    Ok(())
}

pub fn send_token(
    eudico_binary_path: &str,
    lotus_path: &str,
    addr: &str,
    amount: u8,
) -> Result<()> {
    let status = Command::new(eudico_binary_path)
        .args(["send", addr, &amount.to_string()])
        .env("LOTUS_PATH", lotus_path)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("cannot send token to wallet:{:}", addr))
    }
}

fn fund_nodes(root_addr: &str, lotus_path: &str, nodes: &[SubnetNode], amount: u8) -> Result<()> {
    for node in nodes.iter() {
        send_token(
            root_addr,
            lotus_path,
            node.wallet_address.as_ref().unwrap(),
            amount,
        )?;
    }
    Ok(())
}

fn node_from_topology(topology: &SubnetTopology) -> SubnetNode {
    SubnetNode::new(
        topology.id.clone(),
        topology.ipc_root_folder.clone(),
        topology.next_port(),
        topology.next_port(),
        topology.next_port(),
        topology.next_port(),
        topology.eudico_binary_path.clone(),
        topology.ipc_agent_url(),
    )
}

/// Spawn the first node, then subsequent node will connect to this node.
fn spawn_first_node(topology: &SubnetTopology) -> anyhow::Result<SubnetNode> {
    let mut node = node_from_topology(topology);
    node.gen_genesis()?;
    node.spawn_node()?;

    loop {
         match node.new_wallet_address() {
             Ok(_) => {
                 log::info!("one wallet created in node: {:?}", node.id);
                 break;
             }
             Err(e) => {
                 log::error!("cannot create wallet: {e:}, wait and sleep to retry");
                 sleep(Duration::from_secs(10))
             }
         }
    }
    node.config_default_wallet()?;
    Ok(node)
}

fn spawn_other_nodes(
    topology: &SubnetTopology,
    first: &SubnetNode,
) -> anyhow::Result<Vec<SubnetNode>> {
    let mut nodes = vec![];
    for _ in 1..topology.number_of_nodes {
        let mut node = node_from_topology(topology);

        node.spawn_node()?;
        node.new_wallet_address()?;
        node.config_default_wallet()?;

        nodes.push(node);
    }

    let addrs = loop {
        match first.network_addresses() {
            Ok(s) => {
                break s;
            }
            Err(e) => {
                log::warn!("first node not up, wait: {e:}");
                sleep(std::time::Duration::from_secs(5));
            }
        }
    };

    let first_node_addr = tcp_address(addrs)?;
    for node in &nodes {
        node.connect_peer(&first_node_addr)?;
    }

    Ok(nodes)
}

struct SubnetNode {
    id: SubnetID,
    ipc_root_folder: String,
    /// The node info
    node: NodeInfo,
    /// The info of the validator
    validator: NodeInfo,
    eudico_binary_path: String,
    ipc_agent_url: String,
    wallet_address: Option<String>,
}

struct NodeInfo {
    tcp_port: u16,
    quic_port: u16,
    status: SubnetNodeSpawnStatus,
    net_addr: Option<String>,
}

/// The subnet node spawn status
enum SubnetNodeSpawnStatus {
    Running { process: Child },
    Idle,
}

impl SubnetNode {
    pub fn new(
        id: SubnetID,
        ipc_root_folder: String,
        node_tcp_port: u16,
        node_quic_port: u16,
        validator_tcp_port: u16,
        validator_quic_port: u16,
        eudico_binary_path: String,
        ipc_agent_url: String,
    ) -> Self {
        Self {
            id,
            ipc_root_folder,
            node: NodeInfo {
                tcp_port: node_tcp_port,
                quic_port: node_quic_port,
                status: SubnetNodeSpawnStatus::Idle,
                net_addr: None,
            },
            validator: NodeInfo {
                tcp_port: validator_tcp_port,
                quic_port: validator_quic_port,
                status: SubnetNodeSpawnStatus::Idle,
                net_addr: None,
            },
            eudico_binary_path,
            ipc_agent_url,
            wallet_address: None,
        }
    }

    fn subnet_id_cli_string(&self) -> String {
        self.id.to_string().replacen("/", "_", 1000)
    }

    fn lotus_path(&self) -> String {
        format!("~/.lotus_subnet{:}_{:}", self.subnet_id_cli_string(), self.node.tcp_port)
    }

    fn genesis_path(&self) -> String {
        format!(
            "{:}/subnet{:}.car",
            self.ipc_root_folder,
            self.subnet_id_cli_string()
        )
    }

    fn network_addresses(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.eudico_binary_path)
            .args(["net", "listen"])
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        if output.status.success() {
            let s: String = String::from_utf8_lossy(&output.stdout).parse()?;
            Ok(s.split("\n").into_iter().map(|s| s.to_string()).collect())
        } else {
            Err(anyhow!(
                "cannot get network addresses admin token in subnet:{:} with status: {:?}",
                self.id,
                output.status
            ))
        }
    }

    pub fn new_wallet_address(&mut self) -> Result<()> {
        if self.wallet_address.is_some() {
            return Ok(());
        }

        let output = Command::new(&self.eudico_binary_path)
            .args(["wallet", "new"])
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        log::debug!("wallet create status: {:?}", output.status);

        if output.status.success() {
            let wallet = String::from_utf8_lossy(&output.stdout).parse()?;
            self.wallet_address = Some(wallet);
            Ok(())
        } else {
            Err(anyhow!(
                "cannot create new wallet address in subnet:{:} with error: {:?}",
                self.id,
                String::from_utf8_lossy(&output.stderr).parse::<String>()?
            ))
        }
    }

    pub fn config_default_wallet(&self) -> Result<()> {
        if self.wallet_address.is_none() {
            return Err(anyhow!("wallet not created yet"));
        }

        let status = Command::new(&self.eudico_binary_path)
            .args([
                "wallet",
                "set-default",
                self.wallet_address.as_ref().unwrap(),
            ])
            .env("LOTUS_PATH", self.lotus_path())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "cannot set default wallet address in subnet:{:}",
                self.id
            ))
        }
    }

    pub fn gen_genesis(&self) -> Result<()> {
        let genesis_path = self.genesis_path();
        if fs::metadata(&genesis_path).is_ok() {
            return Ok(());
        }

        let status = Command::new(&self.eudico_binary_path)
            .args([
                "genesis",
                "new",
                "--subnet-id",
                &self.id.to_string(),
                "-out",
                &self.genesis_path(),
            ])
            .env("LOTUS_PATH", self.lotus_path())
            .status()?;

        log::debug!(
            "generate genesis for subnet: {:} with status: {:?}",
            self.id,
            status
        );

        if !status.success() {
            let msg = format!(
                "generate genesis for subnet: {:} failed with status: {:?}",
                self.id, status
            );
            return Err(anyhow!(msg));
        }
        Ok(())
    }

    pub fn spawn_node(&mut self) -> Result<()> {
        if !matches!(self.node.status, SubnetNodeSpawnStatus::Idle) {
            return Err(anyhow!(
                "subnet node: {:} already running",
                self.id.to_string()
            ));
        }

        let subnet_id = self.subnet_id_cli_string();

        let node_std_out =
            File::create(format!("./{subnet_id:}_node_{:}.log", self.node.tcp_port))?;
        let node_std_err =
            File::create(format!("./{subnet_id:}_node_{:}.err", self.node.tcp_port))?;

        let child = Command::new(&self.eudico_binary_path)
            .args([
                "mir",
                "daemon",
                "--genesis",
                &self.genesis_path(),
                "--bootstrap",
                "false",
                "--api",
                &self.node.tcp_port.to_string(),
            ])
            .stdout(node_std_out)
            .stderr(node_std_err)
            .env("LOTUS_PATH", self.lotus_path())
            .spawn()?;

        self.node.status = SubnetNodeSpawnStatus::Running { process: child };

        log::debug!("node spawn for subnet: {:}", self.id);

        Ok(())
    }

    pub fn connect_peer(&self, peer: &str) -> Result<()> {
        let status = Command::new(&self.eudico_binary_path)
            .args(["net", "connect", peer])
            .env("LOTUS_PATH", self.lotus_path())
            .status()?;

        if !status.success() {
            let msg = format!(
                "cannot connect to peer {peer:} genesis for subnet: {:} failed with status: {:}",
                self.id, status
            );
            return Err(anyhow!(msg));
        }
        Ok(())
    }

    pub async fn join_subnet(&self) -> Result<()> {
        join_subnet(
            self.ipc_agent_url.clone(),
            self.wallet_address.clone().unwrap(),
            self.id.to_string(),
            DEFAULT_MIN_STAKE,
            self.validator.net_addr.clone().unwrap(),
        )
        .await
    }

    pub fn config_validator(&mut self) -> Result<()> {
        let status = Command::new(&self.eudico_binary_path)
            .args(&[
                "mir",
                "validator",
                "config",
                "init",
                "--quic-libp2p-port",
                &self.validator.quic_port.to_string(),
                "--tcp-libp2p-port",
                &self.validator.tcp_port.to_string(),
            ])
            .status()?;

        if !status.success() {
            return Err(anyhow!("cannot init validator in subnet:{:}", self.id));
        }

        let output = Command::new(&self.eudico_binary_path)
            .args(&["mir", "validator", "config", "validator-addr"])
            .output()?;

        if output.status.success() {
            let raw_addresses = String::from_utf8_lossy(&output.stdout).to_string();
            let addresses = raw_addresses
                .split("\n")
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            let tcp_addr = tcp_address(addresses)?;
            self.validator.net_addr = Some(tcp_addr);

            Ok(())
        } else {
            Err(anyhow!(
                "cannot get validator addresses in subnet:{:}",
                self.id
            ))
        }
    }

    pub fn spawn_validator(&mut self) -> Result<()> {
        if !matches!(self.validator.status, SubnetNodeSpawnStatus::Idle) {
            return Err(anyhow!(
                "subnet node: {:} already running",
                self.id.to_string()
            ));
        }

        let subnet_id = self.subnet_id_cli_string();

        let validator_std_out =
            File::create(format!("./{subnet_id:}_validator_{:}.log", self.validator.tcp_port))?;
        let validator_std_err =
            File::create(format!("./{subnet_id:}_validator_{:}.err", self.validator.tcp_port))?;

        let child = Command::new(&self.eudico_binary_path)
            .args(&[
                "mir",
                "validator",
                "run",
                "--membership",
                "onchain",
                "--nosync",
                "--ipcagent-url",
                &self.ipc_agent_url,
            ])
            .stdout(validator_std_out)
            .stderr(validator_std_err)
            .spawn()?;

        self.validator.status = SubnetNodeSpawnStatus::Running { process: child };

        log::debug!("validator spawn for subnet: {:}", self.id);

        Ok(())
    }

    pub async fn create_admin_token(&self) -> Result<String> {
        let output = Command::new(&self.eudico_binary_path)
            .args(["auth", "create-token", "--perm", "admin"])
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).parse()?)
        } else {
            Err(anyhow!("cannot create admin token in subnet:{:}", self.id))
        }
    }
}

/// Filter and get the tcp address, input must contain tcp address
fn tcp_address(addrs: Vec<String>) -> Result<String> {
    addrs
        .into_iter()
        .filter(|a| a.contains("tcp"))
        .next()
        .ok_or_else(|| anyhow!("no tcp address found"))
}

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
