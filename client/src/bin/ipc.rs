use client::{register_cli_command, register_server_routes, HealthCheckHandler};

// define the routes here
register_server_routes!(
    // initialize your server RPC handlers here, returns the handlers as a tuple
    init: {
        use client::HealthCheckHandler;

        let h1 = HealthCheckHandler {};
        let h2 = HealthCheckHandler {};

        (h1, h2)
    },
    commands: health_check, health_check2
);

// register the cli command handlers here
register_cli_command!(
    // { COMMAND NAME, HANDLER }
    {HealthCheck, HealthCheckHandler},
    {Node, node::NodeHandler}
);

#[tokio::main]
async fn main() {
    env_logger::init();
    cli().await;
}
