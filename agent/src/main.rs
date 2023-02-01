use agent::command::health::HealthCheckCmd;
use agent::{register_cli_command, register_server_routes};

// define the routes here
register_server_routes!(
    // initialize your server RPC handlers here, returns the handlers as a tuple
    init: {
        use agent::command::health::HealthCheckCmd;

        let h1 = HealthCheckCmd {};
        let h2 = HealthCheckCmd {};

        (h1, h2)
    },
    commands: health_check, health_check2
);

// register the cli command handlers here
register_cli_command!(
    // { COMMAND NAME, HANDLER }
    {HealthCheck, HealthCheckCmd},
    {Node, node::NodeCmd}
);

#[tokio::main]
async fn main() {
    env_logger::init();
    cli().await;
}
