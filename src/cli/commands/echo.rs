//! Sample Echo command line handler. Can be removed when we have real implementations.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::CommandLineHandler;
use crate::error::Error;

/// Sample echo command for testing purposes
pub(crate) struct Echo;

#[async_trait]
impl CommandLineHandler for Echo {
    type Request = EchoArgs;

    async fn handle(request: &Self::Request) -> Result<String, Error> {
        Ok(format!("echo: {:}", request.to_echo))
    }
}

#[derive(Debug, Args)]
#[command(about = "Performs an echo operation")]
pub(crate) struct EchoArgs {
    #[arg(
        long,
        value_name = "TO_ECHO",
        help = "What do you want to echo?",
        env = "TO_ECHO_ENV"
    )]
    to_echo: String,
}
