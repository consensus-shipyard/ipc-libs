//! The Info command line handler that prints the info about IPC Agent.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::CommandLineHandler;

/// Sample echo command for testing purposes
pub(crate) struct Info;

#[async_trait]
impl CommandLineHandler for Info {
    type Arguments = InfoArgs;

    async fn handle(_arguments: &Self::Arguments) -> anyhow::Result<()> {
        println!(
            r#"
            Implementation of an IPC agent.
            Version: v0.0.1
            "#
        );
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Prints info of the the ipc agent")]
pub(crate) struct InfoArgs {
}
