mod command;
mod common;

use crate::command::health::HealthCheckCmd;
use crate::command::node::NodeCmd;

// define the routes here
register_server_routes!(
    // initialize your server RPC handlers here, returns the handlers as a tuple
    init: {
        use crate::command::health::HealthCheckCmd;

        let h1 = HealthCheckCmd {};
        let h2 = HealthCheckCmd {};

        (h1, h2)
    },
    // health_check method is associated with h1
    // health_check2 method is associated with h2
    // TODO: find a clearer way to association and not by convention
    commands: health_check, health_check2
);

// register the cli command handlers here
register_cli_command!(
    // { COMMAND NAME, HANDLER }
    {HealthCheck, HealthCheckCmd},
    {Node, NodeCmd}
);

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    cli().await;
}
