use async_trait::async_trait;
use clap::Args;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::common::config::{DEFAULT_NODE_ADDR, DEFAULT_PROTOCOL, DEFAULT_RPC_ENDPOINT};
use crate::common::error::Error;
use crate::common::handlers::{CommandLineHandler, RPCNodeHandler};
use crate::common::rpc::{JSONRPCParam, JSONRPCResponse};

lazy_static! {
    static ref DEFAULT_URL: String = format!(
        "{}://{}/{}",
        DEFAULT_PROTOCOL, DEFAULT_NODE_ADDR, DEFAULT_RPC_ENDPOINT
    );
}

#[derive(Debug, Args)]
#[command(about = "Performs a health check of the running IPC node")]
pub struct HealthCheck {
    #[arg(
        long,
        value_name = "NODE_ENDPOINT",
        help = "The node endpoint to test health",
        env = "CHECK_NODE_ENDPOINT"
    )]
    node_endpoint: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    pub is_healthy: bool,
}

pub struct HealthCheckCmd {}

#[async_trait]
impl CommandLineHandler for HealthCheckCmd {
    type Request = HealthCheck;

    async fn handle(request: &Self::Request) -> Result<String, Error> {
        let node = request.node_endpoint.as_ref().unwrap_or(&DEFAULT_URL);
        if is_health(node).await {
            Ok(format!("node: {:} is healthy", node))
        } else {
            Err(Error::Custom(format!("node: {:} is down", node)))
        }
    }
}

#[async_trait]
impl RPCNodeHandler for HealthCheckCmd {
    type Request = ();
    type Output = HealthResponse;
    type Error = String;

    async fn handle(&self, _request: &Self::Request) -> Result<Self::Output, Self::Error> {
        Ok(HealthResponse { is_healthy: true })
    }
}

async fn is_health(node: &str) -> bool {
    log::debug!("health check endpoint: {:}", node);

    let client = reqwest::Client::new();
    match client
        .post(node)
        .json(&JSONRPCParam::new(
            0,
            "health_check".to_string(),
            serde_json::Value::Null,
        ))
        .send()
        .await
    {
        Err(e) => {
            log::debug!("cannot query health endpoint: {:?} due to {:?}", node, e);
            false
        }
        Ok(r) => {
            r.json::<JSONRPCResponse<HealthResponse>>()
                .await
                .map(|n| n.result.is_healthy)
                // this would be a parsing error, which we will treat as unhealthy
                // should be quite rare for this to happen
                .unwrap_or(false)
        }
    }
}
