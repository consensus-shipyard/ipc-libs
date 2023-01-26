use crate::config::ClientNodeConfig;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use warp::Filter;

lazy_static! {
    static ref DEFAULT_HEALTH_RESPONSE: HealthResponse = HealthResponse { is_healthy: true };
}

pub(crate) const DEFAULT_HEALTH_ENDPOINT: &str = "health";

pub struct IPCClientNode {
    config: ClientNodeConfig,
}

impl IPCClientNode {
    pub fn new(config: ClientNodeConfig) -> Self {
        Self { config }
    }

    /// Runs the node in the current thread
    pub async fn run(&self) {
        let json_rpc = warp::post()
            .and(warp::path("json_rpc"))
            .and(warp::body::bytes())
            .map(|bytes: bytes::Bytes| {
                log::info!("received bytes = {:?}", bytes);
                warp::reply::json(&())
            });

        let health = warp::get()
            .and(warp::path(DEFAULT_HEALTH_ENDPOINT))
            .map(|| {
                log::debug!("received health check request");
                // TODO: we might need to add more checks later
                warp::reply::json(DEFAULT_HEALTH_RESPONSE.deref())
            });

        let routes = json_rpc.or(health);

        warp::serve(routes).run(self.config.addr()).await;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    pub is_healthy: bool,
}
