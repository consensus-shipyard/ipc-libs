use crate::cli::health::HealthCheckHandler;
use crate::cli::node::NodeHandler;
use async_trait::async_trait;
use clap::Args;

pub mod health;
pub mod node;
mod util;

// Registers the handler to the command line.
//
// First attribute is the name of the command.
// Second attribute is the handler.
// Third attribute is the description.
crate::register_command!(
    // { COMMAND NAME, HANDLER, DESCRIPTION }
    {HealthCheck, HealthCheckHandler, "Performs a health check of the running IPC node"},
    {Node, NodeHandler, "Launch the IPC node"}
);

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
