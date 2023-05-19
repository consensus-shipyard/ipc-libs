use std::fs::File;
use std::os::fd::{FromRawFd, IntoRawFd};
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::infra::{SubnetTopology, DEFAULT_IPC_AGENT_FOLDER};
use anyhow::{anyhow, Result};
use ipc_agent::cli::CreateSubnetArgs;
use ipc_agent::config::json_rpc_methods;
use ipc_agent::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use ipc_agent::server::create::{CreateSubnetParams, CreateSubnetResponse};
use ipc_sdk::subnet_id::SubnetID;
use std::process::{Child, Command, Stdio};

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
    log::info!(
        "node up with net addresses: {:?}",
        first_node.network_addresses()?
    );

    Ok(())
}

/// Spawn the first node, then subsequent node will connect to this node.
fn spawn_first_node(topology: &SubnetTopology) -> anyhow::Result<SubnetNode> {
    let mut node = SubnetNode::new(
        topology.id.clone(),
        topology.ipc_root_folder.clone(),
        topology.next_port(),
        topology.next_port(),
        topology.eudico_binary_path.clone(),
        topology.ipc_agent_url(),
    );

    node.gen_genesis()?;
    node.spawn_node()?;

    Ok(node)
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
    api_port: u16,
    status: SubnetNodeSpawnStatus,
}

/// The subnet node spawn status
enum SubnetNodeSpawnStatus {
    Running {
        net_addr: Option<String>,
        process: Child,
    },
    Idle,
}

impl SubnetNode {
    pub fn new(
        id: SubnetID,
        ipc_root_folder: String,
        node_api_port: u16,
        validator_api_port: u16,
        eudico_binary_path: String,
        ipc_agent_url: String,
    ) -> Self {
        Self {
            id,
            ipc_root_folder,
            node: NodeInfo {
                api_port: node_api_port,
                status: SubnetNodeSpawnStatus::Idle,
            },
            validator: NodeInfo {
                api_port: validator_api_port,
                status: SubnetNodeSpawnStatus::Idle,
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
        format!("~/.lotus_subnet{:}", self.subnet_id_cli_string())
    }

    fn genesis_path(&self) -> String {
        format!(
            "{:}/subnet{:}.car",
            self.ipc_root_folder,
            self.subnet_id_cli_string()
        )
    }

    fn network_addresses(&self) -> Result<Vec<String>> {
        let output = Command::new(format!("{:} net listen", self.eudico_binary_path))
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        if output.status.success() {
            let s: String = String::from_utf8_lossy(&output.stdout).parse()?;
            Ok(s.split("\n").into_iter().map(|s| s.to_string()).collect())
        } else {
            Err(anyhow!("cannot create admin token in subnet:{:}", self.id))
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

        if output.status.success() {
            let wallet = String::from_utf8_lossy(&output.stdout).parse()?;
            self.wallet_address = Some(wallet);
            Ok(())
        } else {
            Err(anyhow!(
                "cannot create new wallet address in subnet:{:}",
                self.id
            ))
        }
    }

    pub fn gen_genesis(&self) -> Result<()> {
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

        let fd = File::create(format!("./{subnet_id:}.log"))
            .unwrap()
            .into_raw_fd();

        let child = Command::new(&self.eudico_binary_path)
            .args([
                "mir",
                "daemon",
                "--genesis",
                &self.genesis_path(),
                "--bootstrap",
                "false",
                "--api",
                &self.node.api_port.to_string(),
            ])
            .stdout(unsafe { Stdio::from_raw_fd(fd) })
            .env("LOTUS_PATH", self.lotus_path())
            .spawn()?;

        self.node.status = SubnetNodeSpawnStatus::Running {
            net_addr: None,
            process: child,
        };

        log::debug!("node spawn for subnet: {:}", self.id);

        Ok(())
    }

    pub fn connect_peer(&self, peer: &str) -> Result<()> {
        let status = Command::new(format!("{:} net connect", self.eudico_binary_path))
            .arg(peer)
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

    pub fn register_validator(&self) -> Result<()> {
        Ok(())
    }

    pub fn config_validator(&self) -> Result<()> {
        Ok(())
    }

    pub fn spawn_validator(&mut self) -> Result<()> {
        if !matches!(self.validator.status, SubnetNodeSpawnStatus::Idle) {
            return Err(anyhow!(
                "subnet node: {:} already running",
                self.id.to_string()
            ));
        }

        let child = Command::new(format!("{:} mir validator run", self.eudico_binary_path))
            .arg("--membership")
            .arg("onchain")
            .arg("--nosync")
            .arg("--ipcagent-url")
            .arg(&self.ipc_agent_url)
            .spawn()?;

        self.validator.status = SubnetNodeSpawnStatus::Running {
            net_addr: None,
            process: child,
        };

        log::debug!("validator spawn for subnet: {:}", self.id);

        Ok(())
    }

    pub async fn create_admin_token(&self) -> Result<String> {
        let output = Command::new(format!("{:} auth create-token", self.eudico_binary_path))
            .arg("--perm")
            .arg("admin")
            .env("LOTUS_PATH", self.lotus_path())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).parse()?)
        } else {
            Err(anyhow!("cannot create admin token in subnet:{:}", self.id))
        }
    }
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
        min_validator_stake: 1.0,
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
