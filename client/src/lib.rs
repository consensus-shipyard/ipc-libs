mod cli;
mod command;
mod config;
mod server;

use async_trait::async_trait;
use clap::Args;
use serde::{Deserialize, Serialize};

pub use command::health::HealthCheckHandler;
pub use config::ClientNodeConfig;

/// The common trait to handle command line functions
#[async_trait]
pub trait CommandLineHandler {
    /// The request to process.
    ///
    /// NOTE that this parameter is used to generate the command line arguments.
    /// Currently we are directly integrating with `clap` crate. In the future we can use our own
    /// implementation to abstract away external crates. But this should be good for now.
    type Request: std::fmt::Debug + Args;
    /// The error thrown
    type Error: std::fmt::Debug;

    /// Handles the request and produces a response
    async fn handle(request: &Self::Request) -> Result<(), Self::Error>;
}

/// The common trait for json-rpc handler
#[async_trait]
pub trait RPCNodeHandler {
    /// The request to process.
    type Request: std::fmt::Debug;
    /// The output of the handler
    type Output: std::fmt::Debug;
    /// The error thrown
    type Error: std::fmt::Debug + std::fmt::Display;

    /// Handles the request and produces a response
    async fn handle(&self, request: &Self::Request) -> Result<Self::Output, Self::Error>;
}

/// Follows: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Serialize, Deserialize)]
pub struct JSONRPCParam {
    pub id: u16,
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct JSONRPCResponse<T: Serialize> {
    pub id: u16,
    pub jsonrpc: String,
    pub result: T,
}
