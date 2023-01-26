mod health;
mod node;

use clap::{Arg, Command};

use crate::command::node::NODE_CONFIG_KEY;

pub use health::health_check;
pub use node::start_node;

pub const NODE: &str = "node";
pub const HEALTH_CHECK: &str = "healthcheck";

pub fn cli() -> Command {
    Command::new("ipc-client")
        .about("The one client to interact with IPC actors")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new(HEALTH_CHECK)
                .about("Checking the health of the node")
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new(NODE)
                .about("Starts a node to interact with IPC actors on chain")
                .arg(
                    Arg::new(NODE_CONFIG_KEY)
                        .long("config")
                        .help("The config file path for the IPC client node")
                        .env("IPC_CLIENT_NODE_CONFIG"),
                ),
        )
}
