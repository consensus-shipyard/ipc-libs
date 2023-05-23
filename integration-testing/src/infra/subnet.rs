// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::infra::util::{import_wallet, trim_newline};
use crate::infra::{util, SubnetTopology, DEFAULT_MIN_STAKE};
use anyhow::{anyhow, Result};
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::fs;
use std::fs::File;
use std::process::{Child, Command};
use std::str::FromStr;
use std::thread::sleep;

/// Spawn child subnet according to the topology
pub async fn spawn_child_subnet(topology: &mut SubnetTopology) -> anyhow::Result<()> {
    if topology.number_of_nodes == 0 {
        log::info!("no nodes to spawn");
        return Ok(());
    }

    let parent = topology.parent.to_string();

    let actor_addr = util::create_subnet(
        topology.ipc_agent_url(),
        topology.root_address.clone(),
        parent,
        topology.name.clone(),
        topology.number_of_nodes as u64,
    )
    .await?;

    topology.id = Some(SubnetID::new_from_parent(
        &topology.parent,
        Address::from_str(&actor_addr)?,
    ));

    log::info!("created subnet: {:?}", topology.id);

    let first_node = spawn_first_node(topology)?;
    let mut nodes = spawn_other_nodes(topology, &first_node)?;

    nodes.push(first_node);

    util::fund_nodes(
        &topology.eudico_binary_path,
        &topology.root_lotus_path,
        &nodes,
        10,
    )?;

    for node in nodes.iter_mut() {
        node.config_validator()?;
        log::info!(
            "configured validator for node: {:?}",
            node.validator.net_addr
        );

        node.export_wallet_to_ipc_key_store().await?;
        node.join_subnet().await?;
        log::info!(
            "validator: {:?} joined subnet: {:}",
            node.validator.net_addr,
            node.id
        );

        sleep(std::time::Duration::from_secs(5));

        node.spawn_validator()?;
        log::info!("validator: {:?} spawned", node.validator.net_addr);
    }

    let accounts = nodes
        .iter()
        .map(|n| n.wallet_address.clone().unwrap())
        .collect::<Vec<_>>()
        .join(",");

    println!("accounts = [{accounts:?}]");

    let mut admin_token = nodes[0].create_admin_token().await?;
    trim_newline(&mut admin_token);
    println!("auth_token = \"{admin_token:}\"");

    println!(
        "jsonrpc_api_http = \"http://127.0.0.1:{:}/rpc/v1\"",
        nodes[0].node.tcp_port
    );

    Ok(())
}

fn node_from_topology(topology: &SubnetTopology) -> SubnetNode {
    SubnetNode::new(
        topology.id.clone().unwrap(),
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

    util::create_wallet(&mut node)?;
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

        util::create_wallet(&mut node)?;

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

    let mut first_node_addr = util::tcp_address(addrs)?;
    util::trim_newline(&mut first_node_addr);

    log::info!("first node net addr: {:?}", first_node_addr);

    for node in &nodes {
        node.connect_peer(&first_node_addr)?;
    }

    Ok(nodes)
}

pub struct SubnetNode {
    pub id: SubnetID,
    ipc_root_folder: String,
    /// The node info
    node: NodeInfo,
    /// The info of the validator
    validator: NodeInfo,
    eudico_binary_path: String,
    ipc_agent_url: String,
    pub(crate) wallet_address: Option<String>,
}

struct NodeInfo {
    tcp_port: u16,
    quic_port: u16,
    status: SubnetNodeSpawnStatus,
    net_addr: Option<String>,
}

/// The subnet node spawn status
enum SubnetNodeSpawnStatus {
    Running {
        #[allow(dead_code)]
        process: Child,
    },
    Idle,
}

impl SubnetNode {
    #[allow(clippy::too_many_arguments)]
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
        self.id.to_string().replacen('/', "_", 1000)
    }

    fn lotus_path(&self) -> String {
        format!(
            "{:}/.lotus_subnet{:}_{:}",
            std::env::var("HOME").unwrap(),
            self.subnet_id_cli_string(),
            self.node.tcp_port
        )
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
            Ok(s.split('\n').into_iter().map(|s| s.to_string()).collect())
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
            let mut wallet = String::from_utf8_lossy(&output.stdout).parse()?;
            util::trim_newline(&mut wallet);
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

    pub async fn export_wallet_to_ipc_key_store(&mut self) -> Result<()> {
        if self.wallet_address.is_none() {
            return Err(anyhow!("wallet not created"));
        }

        let output = Command::new(&self.eudico_binary_path)
            .args([
                "wallet",
                "export",
                "--lotus-json",
                self.wallet_address.as_ref().unwrap(),
            ])
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        log::debug!("wallet export status: {:?}", output.status);

        if output.status.success() {
            let mut private_key_json: String = String::from_utf8_lossy(&output.stdout).parse()?;
            util::trim_newline(&mut private_key_json);
            import_wallet(&self.ipc_agent_url, private_key_json).await
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

        log::info!(
            "setting wallet: {:} as default",
            self.wallet_address.as_ref().unwrap()
        );

        let status = Command::new(&self.eudico_binary_path)
            .args([
                "wallet",
                "set-default",
                self.wallet_address.as_ref().unwrap(),
            ])
            .env("LOTUS_PATH", self.lotus_path())
            .status()?;

        if status.success() {
            log::info!(
                "set wallet: {:} as default",
                self.wallet_address.as_ref().unwrap()
            );
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

        log::info!(
            "spawning node with api: {:}, genesis: {:}, lotus path: {:}",
            self.node.tcp_port,
            self.genesis_path(),
            self.lotus_path()
        );

        let child = Command::new(&self.eudico_binary_path)
            .args([
                "mir",
                "daemon",
                &format!("--genesis={:}", self.genesis_path()),
                &format!("--api={:}", self.node.tcp_port),
                "--bootstrap=false",
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
        util::join_subnet(
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
            .args([
                "mir",
                "validator",
                "config",
                "init",
                "--quic-libp2p-port",
                &self.validator.quic_port.to_string(),
                "--tcp-libp2p-port",
                &self.validator.tcp_port.to_string(),
                "-f",
            ])
            .env("LOTUS_PATH", self.lotus_path())
            .status()?;

        if !status.success() {
            return Err(anyhow!("cannot init validator in subnet:{:}", self.id));
        }

        let output = Command::new(&self.eudico_binary_path)
            .args(["mir", "validator", "config", "validator-addr"])
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        if output.status.success() {
            let raw_addresses = String::from_utf8_lossy(&output.stdout).to_string();

            log::debug!("raw addresses: {:?}", raw_addresses);

            let addresses = raw_addresses.lines().map(|s| s.to_string()).collect();

            let mut tcp_addr = util::tcp_address(addresses)?;
            util::trim_newline(&mut tcp_addr);

            // the net address starts with wallet address, need to trim it
            let parts = tcp_addr.split('@').collect::<Vec<_>>();
            self.validator.net_addr = Some(parts[1].to_string());

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

        let validator_std_out = File::create(format!(
            "./{subnet_id:}_validator_{:}.log",
            self.validator.tcp_port
        ))?;
        let validator_std_err = File::create(format!(
            "./{subnet_id:}_validator_{:}.err",
            self.validator.tcp_port
        ))?;

        let child = Command::new(&self.eudico_binary_path)
            .args([
                "mir",
                "validator",
                "run",
                "--membership=onchain",
                "--nosync",
                &format!("--ipcagent-url={:}", self.ipc_agent_url),
            ])
            .env("LOTUS_PATH", self.lotus_path())
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
