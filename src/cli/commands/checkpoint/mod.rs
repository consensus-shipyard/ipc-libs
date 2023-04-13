// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::cli::commands::checkpoint::list_checkpoints::{ListCheckpoints, ListCheckpointsArgs};
use crate::cli::{CommandLineHandler, GlobalArguments};

use clap::{Args, Subcommand};

mod list_checkpoints;

#[derive(Debug, Args)]
#[command(name = "checkpoint", about = "checkpoint related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct CheckpointCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl CheckpointCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::List(args) => ListCheckpoints::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    List(ListCheckpointsArgs),
}