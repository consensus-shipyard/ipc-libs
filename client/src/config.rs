use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::net::SocketAddr;
use std::str::FromStr;

const DEFAULT_NODE_ADDR: &str = "127.0.0.1:3030";

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

/// The config struct used parsed from cli
#[derive(Deserialize, Debug, Default)]
pub(crate) struct NodeLaunchConfig {
    pub config_path: Option<String>,
}

impl NodeLaunchConfig {
    pub fn client_node_config(&self) -> ClientNodeConfig {
        self.config_path
            .as_ref()
            .map(|s| parse_yaml(s))
            .unwrap_or_default()
    }
}

fn parse_yaml<T: DeserializeOwned>(path: &str) -> T {
    let raw = std::fs::read_to_string(path).expect("cannot read config yaml");
    serde_yaml::from_str(&raw).expect("cannot parse yaml")
}
