use async_trait::async_trait;
use clap::Args;
use serde::{de::DeserializeOwned, Deserialize};

use crate::common::config::ClientNodeConfig;
use crate::common::error::Error;
use crate::common::handlers::CommandLineHandler;

/// The config struct used parsed from cli
#[derive(Deserialize, Debug, Default, Args)]
#[command(about = "Launches the IPC node")]
pub struct NodeLaunch {
    #[arg(
        long = "config",
        value_name = "CONFIG_FILE_PATH",
        help = "The config file path for the IPC client node",
        env = "IPC_CLIENT_NODE_CONFIG"
    )]
    config_path: Option<String>,
}

impl NodeLaunch {
    pub fn client_node_config(&self) -> ClientNodeConfig {
        self.config_path
            .as_ref()
            .map(|s| parse_yaml(s))
            .unwrap_or_default()
    }
}

pub struct NodeCmd {}

#[async_trait]
impl CommandLineHandler for NodeCmd {
    type Request = NodeLaunch;

    async fn handle(request: &Self::Request) -> Result<String, Error> {
        let node_config = request.client_node_config();
        crate::node::IPCClientNode::new(node_config).run().await;
        Ok(String::from("node up"))
    }
}

fn parse_yaml<T: DeserializeOwned>(path: &str) -> T {
    let raw = std::fs::read_to_string(path).expect("cannot read config yaml");
    serde_yaml::from_str(&raw).expect("cannot parse yaml")
}
