// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Setup infra for integration testing

pub mod subnet;
pub mod util;

use ipc_sdk::subnet_id::SubnetID;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

const DEFAULT_IPC_AGENT_URL: &str = "http://localhost:3030/json_rpc";
const DEFAULT_NODE_API_BASE_PORT: u16 = 1230;
const DEFAULT_MIN_STAKE: f64 = 1.0;

/// The configuration struct for the subnet to spawn
pub struct SubnetConfig {
    /// The id of the subnet. If not specified, will create a subnet first.
    pub id: Option<SubnetID>,
    /// Name of the subnet
    pub name: String,
    /// The parent of the subnet
    pub parent: SubnetID,
    /// Number of nodes in the subnet
    pub number_of_nodes: usize,
    /// The path to eudico binary. Since most of the operations are issued from
    /// command line, we need to point to the eudico binary path.
    pub eudico_binary_path: String,
    /// The parent subnet wallet address. This will be used to perform setups in the parent
    /// subnet, such as initial fund transfer to the validators so that validators can join
    /// the created subnet
    pub parent_wallet_address: String,
    /// The parent subnet eudico lotus path
    pub parent_lotus_path: String,
    /// The ipc agent root folder
    pub ipc_root_folder: String,

    ipc_agent_url: Option<String>,

    /// The monotonic sequential port number generator to assign to each validator
    port_starting_seq: Arc<AtomicU16>,
}

impl SubnetConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        parent_wallet_address: String,
        parent_lotus_path: String,
        ipc_root_folder: String,
        number_of_nodes: usize,
        eudico_binary_path: String,
        parent: SubnetID,
        port_starting_seq: Arc<AtomicU16>,
    ) -> Self {
        Self {
            id: None,
            name,
            number_of_nodes,
            eudico_binary_path,
            parent,
            parent_wallet_address,
            parent_lotus_path,
            ipc_root_folder,
            port_starting_seq,
            ipc_agent_url: None,
        }
    }

    pub fn ipc_agent_url(&self) -> String {
        self.ipc_agent_url
            .clone()
            .unwrap_or_else(|| DEFAULT_IPC_AGENT_URL.to_string())
    }

    pub fn next_port(&self) -> u16 {
        loop {
            let r = self.port_starting_seq.load(Ordering::SeqCst);
            if self
                .port_starting_seq
                .compare_exchange(r, r + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return r + DEFAULT_NODE_API_BASE_PORT;
            }
        }
    }
}
