use client::{register_cli_command, register_server_routes, HealthCheckHandler};

use lazy_static::lazy_static;
use node::NodeHandler;
use std::sync::Arc;

// initialize your server RPC handlers here
lazy_static! {
    static ref HEALTH: Arc<HealthCheckHandler> = Arc::new(HealthCheckHandler {});
}

// define the routes here
register_server_routes!(
    {"health-check", HEALTH, HealthCheckHandler}
);

// register the cli command handlers here
register_cli_command!(
    // { COMMAND NAME, HANDLER }
    {HealthCheck, HealthCheckHandler},
    {Node, NodeHandler}
);

#[tokio::main]
async fn main() {
    env_logger::init();
    cli().await;
}
