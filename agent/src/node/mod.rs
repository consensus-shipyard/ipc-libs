//! The ipc-agent node RPC handler. Each rpc method provided by the node will have a corresponding
//! handler, represented by `RPCNodeHandler`. One must implement the `RPCNodeHandler` so that it
//! can be registered to the node using `register_server_routes!`.

mod config;
mod rpc;
mod types;

use async_trait::async_trait;

pub use crate::node::config::ClientNodeConfig;

/// The trait used to represent the method the RPC node is providing.
#[async_trait]
pub trait RPCNodeHandler: Send + Sync {
    /// The request to process.
    type Request;
    /// The output of the handler
    type Output: serde::Serialize;
    /// The error thrown
    type Error: std::fmt::Display;

    /// Handles the request and produces a response
    async fn handle(&self, request: &Self::Request) -> Result<Self::Output, Self::Error>;
}
