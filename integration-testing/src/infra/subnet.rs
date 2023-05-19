use anyhow::{anyhow, Result};
use std::process::{Child, Command, ExitStatus};
use ipc_sdk::subnet_id::SubnetID;
use crate::infra::SubnetTopology;

/// Spawn child subnet according to the topology
pub fn spawn_child_subnet(topology: &SubnetTopology) -> anyhow::Result<()> {
    if topology.number_of_nodes == 0 {
        log::info!("no nodes to spawn");
        return Ok(())
    }

    Ok(())
}

/// Spawn the first node, then subsequent node will connect to this node.
fn spawn_first_node(topology: &SubnetTopology) -> anyhow::Result<SubnetNode> {
    let node = SubnetNode::new(
        topology.id.clone(),
        DEFAULT_NODE_API_BASE_PORT + topology.port_starting_seq,

    )
    Ok(())
}

struct SubnetNode {
    id: SubnetID,
    api_port: u16,
    status: SubnetNodeSpawnStatus,
    eudico_binary_path: String,
    ipc_agent_url: String,
    default_wallet_path: String,
}

/// The subnet node spawn status
enum SubnetNodeSpawnStatus {
    Running { rpc_url: String, net_addr: Option<String>, process: Child },
    Idle
}

impl SubnetNode {
    pub fn new(id: SubnetID, api_port: u16, eudico_binary_path: String, ipc_agent_url: String, default_wallet_path: String) -> Self {
        Self {
            id, api_port, status: SubnetNodeSpawnStatus::Idle, eudico_binary_path, ipc_agent_url, default_wallet_path
        }
    }

    fn subnet_id_cli_string(&self) -> String {
        self.id.to_string().replacen("/", "_", 1000)
    }

    fn lotus_path(&self) -> String {
        format!("~/.lotus_subnet_{:}", self.subnet_id_cli_string())
    }

    fn genesis_name(&self) -> String {
        format!("subnet_{:}.car", self.subnet_id_cli_string())
    }

    pub fn gen_genesis(&self) -> Result<()> {
        let status = Command::new(format!("{:} genesis new", self.eudico_binary_path))
            .arg("--subnet-id")
            .arg(&self.id.to_string())
            .arg("-out")
            .arg(self.genesis_name())
            .env("LOTUS_PATH", self.lotus_path())
            .status()?;

        log::debug!("generate genesis for subnet: {:} with status: {:}", self.id, status);

        if !status.success() {
            let msg = format!("generate genesis for subnet: {:} failed with status: {:}", self.id, status);
            return Err(anyhow!(msg))
        }
        Ok(())
    }

    pub fn spawn_node(&mut self) -> Result<()> {
        if !matches!(self.status, SubnetNodeSpawnStatus::Idle) {
            return Err(anyhow!("subnet node: {:} already running", self.id.to_string()));
        }

        let subnet_id = self.subnet_id_cli_string();

        let child = Command::new(format!("{:} mir daemon ", self.eudico_binary_path))
            .arg("--genesis")
            .arg(format!("subnet_{:}.car", subnet_id))
            .arg("--bootstrap")
            .arg("false")
            .arg("--api")
            .arg(&self.api_port.to_string())
            .env("LOTUS_PATH", self.lotus_path())
            .spawn()?;

        self.status = SubnetNodeSpawnStatus::Running {
            rpc_url: "".to_string(),
            net_addr: None,
            process: child
        };

        log::debug!("node spawn for subnet: {:}", self.id);

        Ok(())
    }

    pub fn connect_peer(&self, peer: &str) -> Result<()> {
        Ok(())
    }

    pub fn register_validator(&self) -> Result<()> {
        Ok(())
    }

    pub fn config_validator(&self) -> Result<()> {
        Ok(())
    }

    pub fn spawn_validator(&self) -> Result<()> {
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
