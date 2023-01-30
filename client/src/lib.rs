mod cli;
mod command;
mod config;
mod server;

use async_trait::async_trait;
use clap::Args;

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
    type Error: std::fmt::Debug;

    /// Handles the request and produces a response
    async fn handle(&self, request: &Self::Request) -> Result<Self::Output, Self::Error>;
}
