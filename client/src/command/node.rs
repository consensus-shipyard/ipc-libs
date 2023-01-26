use crate::config::NodeLaunchConfig;
use crate::node::IPCClientNode;
use clap::ArgMatches;

pub(crate) const NODE_CONFIG_KEY: &str = "CONFIG_FILE";

pub async fn start_node(matches: &ArgMatches) {
    let cli_config = parse_node(matches);
    let node_config = cli_config.client_node_config();
    IPCClientNode::new(node_config).run().await;
}

fn parse_node(matches: &ArgMatches) -> NodeLaunchConfig {
    let node_config = matches
        .get_one::<String>(NODE_CONFIG_KEY)
        .map(|s| s.as_str());
    let config = NodeLaunchConfig {
        config_path: node_config.map(String::from),
    };

    log::info!("parsed node args: {:?}", config);

    config
}
