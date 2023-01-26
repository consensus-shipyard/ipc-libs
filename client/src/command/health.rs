use crate::config::{DEFAULT_NODE_ADDR, DEFAULT_PROTOCOL};
use crate::node::{HealthResponse, DEFAULT_HEALTH_ENDPOINT};
use clap::ArgMatches;
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_URL: String = format!(
        "{}://{}/{}",
        DEFAULT_PROTOCOL, DEFAULT_NODE_ADDR, DEFAULT_HEALTH_ENDPOINT
    );
}

pub(crate) const HEALTH_CHECK_ENDPOINT_KEY: &str = "NODE_ENDPOINT";

pub async fn health_check(matches: &ArgMatches) {
    let node = matches
        .get_one::<String>(HEALTH_CHECK_ENDPOINT_KEY)
        .map(|s| s.as_str())
        .unwrap_or(&DEFAULT_URL);

    if is_health(node).await {
        println!("node: {:} is healthy", node);
    } else {
        println!("node: {:} is not healthy", node);
    }
}

async fn is_health(node: &str) -> bool {
    log::debug!("health check endpoint: {:}", node);

    let r = match reqwest::get(node).await {
        Err(e) => {
            log::error!("cannot query health endpoint: {:?} due to {:?}", node, e);
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
