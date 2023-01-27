use crate::cli::CommandLineHandler;
use crate::config::ClientNodeConfig;
use crate::node::IPCClientNode;
use async_trait::async_trait;
use clap::Args;
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// The config struct used parsed from cli
#[derive(Deserialize, Debug, Default, Args)]
#[command(about = "Launchs the IPC node")]
pub(crate) struct NodeLaunch {
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

pub(crate) struct NodeHandler {}

#[async_trait]
impl CommandLineHandler for NodeHandler {
    type Request = NodeLaunch;
    type Error = ();

    async fn handle(request: &Self::Request) -> Result<(), Self::Error> {
        let node_config = request.client_node_config();
        IPCClientNode::new(node_config).run().await;
        Ok(())
    }
}

fn parse_yaml<T: DeserializeOwned>(path: &str) -> T {
    let raw = std::fs::read_to_string(path).expect("cannot read config yaml");
    serde_yaml::from_str(&raw).expect("cannot parse yaml")
}
