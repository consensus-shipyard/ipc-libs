//! Setup infra for integration testing

mod subnet;

use std::sync::atomic::AtomicU16;
use ipc_sdk::subnet_id::SubnetID;

const DEFAULT_IPC_AGENT_URL: &str = "http://localhost:3030/json_rpc";
pub(crate) const DEFAULT_NODE_API_BASE_PORT: u16 = 1235;

pub struct SubnetTopology {
    pub id: SubnetID,
    pub number_of_nodes: usize,
    pub eudico_binary_path: String,
    pub parent: Option<SubnetID>,

    port_starting_seq: AtomicU16,
    ipc_agent_url: Option<String>,
}

impl SubnetTopology {
    pub fn new(id: SubnetID, number_of_nodes: usize, eudico_binary_path: String, parent: Option<SubnetID>, port_starting_seq: u16, ipc_agent_url: Option<String>) -> Self {
        Self { id, number_of_nodes, eudico_binary_path, parent, port_starting_seq: , ipc_agent_url }
    }

    pub fn ipc_agent_url(&self) -> String {
        self.ipc_agent_url.clone().unwrap_or(DEFAULT_IPC_AGENT_URL.to_string())
    }
}

