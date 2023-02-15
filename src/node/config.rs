use serde::Deserialize;
use std::net::SocketAddr;
use std::str::FromStr;

pub const DEFAULT_JSON_RPC_ADDR: &str = "127.0.0.1:3030";
pub const DEFAULT_JSON_RPC_PROTOCOL: &str = "http";
pub const DEFAULT_JSON_RPC_ENDPOINT: &str = "json_rpc";
pub const DEFAULT_JSON_RPC_VERSION: &str = "2.0";

/// The IPC Json RPC client node Configuration. This can be loaded from a static file.
#[derive(Deserialize, Debug, Default)]
pub struct IPCJsonRPCNodeConfig {
    /// The addr for this node, default to `DEFAULT_NODE_ADDR`
    addr: Option<SocketAddr>,
}

impl IPCJsonRPCNodeConfig {
    pub fn addr(&self) -> SocketAddr {
        self.addr
            .unwrap_or_else(|| SocketAddr::from_str(DEFAULT_JSON_RPC_ADDR).unwrap())
    }
}
