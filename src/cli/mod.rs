use crate::error::Error;
use async_trait::async_trait;
use clap::Args;

/// The trait that represents the abstraction of a command line handler. To implement a new command
/// line operation, implement this trait and register it, see `register::register_cli_command` for
/// how to register.
///
/// Note that this trait does not support a stateful implementation as we assume CLI commands are all
/// constructed from scratch. Initialize the states in the `handle` method or pass in from the
/// implementation constructor.
#[async_trait]
pub trait CommandLineHandler {
    /// The request to process.
    ///
    /// NOTE that this parameter is used to generate the command line arguments.
    /// Currently we are directly integrating with `clap` crate. In the future we can use our own
    /// implementation to abstract away external crates. But this should be good for now.
    type Request: std::fmt::Debug + Args;

    /// Handles the request and produces a response string.
    async fn handle(request: &Self::Request) -> Result<String, Error>;
}
