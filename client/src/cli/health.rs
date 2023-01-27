use async_trait::async_trait;
use clap::Args;
use lazy_static::lazy_static;

use crate::cli::CommandLineHandler;
use crate::config::{DEFAULT_HEALTH_ENDPOINT, DEFAULT_NODE_ADDR, DEFAULT_PROTOCOL};
use crate::node::HealthResponse;

lazy_static! {
    static ref DEFAULT_URL: String = format!(
        "{}://{}/{}",
        DEFAULT_PROTOCOL, DEFAULT_NODE_ADDR, DEFAULT_HEALTH_ENDPOINT
    );
}

#[derive(Debug, Args)]
#[command(about = "Performs a health check of the running IPC node")]
pub(crate) struct HealthCheck {
    #[arg(
        long,
        value_name = "NODE_ENDPOINT",
        help = "The node endpoint to test health",
        env = "CHECK_NODE_ENDPOINT"
    )]
    node_endpoint: Option<String>,
}

pub(crate) struct HealthCheckHandler {}

#[async_trait]
impl CommandLineHandler for HealthCheckHandler {
    type Request = HealthCheck;
    type Error = ();

    async fn handle(request: &Self::Request) -> Result<(), Self::Error> {
        let node = request.node_endpoint.as_ref().unwrap_or(&DEFAULT_URL);
        if is_health(node).await {
            println!("node: {:} is healthy", node);
        } else {
            println!("node: {:} is not healthy", node);
        }
        Ok(())
    }
}

async fn is_health(node: &str) -> bool {
    log::debug!("health check endpoint: {:}", node);

    let r = match reqwest::get(node).await {
        Err(e) => {
            log::debug!("cannot query health endpoint: {:?} due to {:?}", node, e);
            return false;
        }
        Ok(r) => r,
    };

    r.json::<HealthResponse>()
        .await
        .map(|n| n.is_healthy)
        // this would be a parsing error, which we will treat as unhealthy
        // should be quite rare for this to happen
        .unwrap_or(false)
}
