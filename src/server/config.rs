use crate::server::DEFAULT_JSON_RPC_SERVER_ADDR;
use serde::Deserialize;
use std::net::SocketAddr;
use std::str::FromStr;

/// The IPC Json RPC agent node Configuration. This can be loaded from a static file.
#[derive(Deserialize, Debug, Default)]
pub struct JsonRPCServerConfig {
    /// The addr for this node, default to `DEFAULT_NODE_ADDR`
    addr: Option<SocketAddr>,
}

impl JsonRPCServerConfig {
    pub fn addr(&self) -> SocketAddr {
        self.addr
            .unwrap_or_else(|| SocketAddr::from_str(DEFAULT_JSON_RPC_SERVER_ADDR).unwrap())
    }
}
