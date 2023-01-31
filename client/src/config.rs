use serde::Deserialize;
use std::net::SocketAddr;
use std::str::FromStr;

pub(crate) const DEFAULT_NODE_ADDR: &str = "127.0.0.1:3030";
pub(crate) const DEFAULT_PROTOCOL: &str = "http";
pub(crate) const DEFAULT_HEALTH_ENDPOINT: &str = "health-check";

/// The Client Node Configuration. Should be loaded from a static file.
#[derive(Deserialize, Debug, Default)]
pub struct ClientNodeConfig {
    /// The addr for this node, default to `DEFAULT_NODE_ADDR`
    addr: Option<String>,
}

impl ClientNodeConfig {
    pub fn addr(&self) -> SocketAddr {
        self.addr
            .as_ref()
            .map(|r| SocketAddr::from_str(r).expect("invalid socket addr"))
            .unwrap_or_else(|| SocketAddr::from_str(DEFAULT_NODE_ADDR).unwrap())
    }
}
