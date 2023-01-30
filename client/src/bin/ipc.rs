use client::{register_cli_command, register_server_routes, HealthCheckHandler};
use lazy_static::lazy_static;

// initialize your server RPC handlers here
lazy_static! {
    static ref HEALTH: Arc<HealthCheckHandler> = Arc::new(HealthCheckHandler {});
}

// define the routes here
register_server_routes!(
    {get, "/health-check", HEALTH}
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
