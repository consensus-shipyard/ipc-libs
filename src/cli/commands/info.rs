//! The Info command line handler that prints the info about IPC Agent.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::CommandLineHandler;
use crate::error::Error;

/// Sample echo command for testing purposes
pub(crate) struct Info;

#[async_trait]
impl CommandLineHandler for Info {
    type Request = InfoArgs;

    async fn handle(_request: &Self::Request) -> Result<String, Error> {
        Ok(format!("echo: {:}", request.to_echo))
    }
}

#[derive(Debug, Args)]
#[command(about = "Prints info of the the ipc agent")]
pub(crate) struct InfoArgs {
}
