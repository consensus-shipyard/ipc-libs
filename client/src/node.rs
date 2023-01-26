use crate::config::ClientNodeConfig;
use warp::Filter;

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

        warp::serve(json_rpc).run(self.config.addr()).await;
    }
}
