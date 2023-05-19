// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Setup infra for integration testing

pub mod subnet;

use ipc_sdk::subnet_id::SubnetID;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

const DEFAULT_IPC_AGENT_URL: &str = "http://localhost:3030/json_rpc";
const DEFAULT_IPC_AGENT_FOLDER: &str = "~/.ipc-agent";
const DEFAULT_NODE_API_BASE_PORT: u16 = 1230;

pub struct SubnetTopology {
    pub id: SubnetID,
    pub name: String,
    pub number_of_nodes: usize,
    pub eudico_binary_path: String,
    pub parent: Option<SubnetID>,
    pub root_address: String,

    port_starting_seq: Arc<AtomicU16>,
    ipc_agent_url: Option<String>,
}

impl SubnetTopology {
    pub fn new(
        id: SubnetID,
        name: String,
        root_address: String,
        number_of_nodes: usize,
        eudico_binary_path: String,
        parent: Option<SubnetID>,
        port_starting_seq: Arc<AtomicU16>,
    ) -> Self {
        Self {
            id,
            name,
            number_of_nodes,
            eudico_binary_path,
            parent,
            root_address,
            port_starting_seq,
            ipc_agent_url: None,
        }
    }

    pub fn ipc_agent_url(&self) -> String {
        self.ipc_agent_url
            .clone()
            .unwrap_or(DEFAULT_IPC_AGENT_URL.to_string())
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
