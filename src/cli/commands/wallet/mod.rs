use crate::cli::{CommandLineHandler, GlobalArguments};

use crate::cli::commands::wallet::send_value::{SendValue, SendValueArgs};
use crate::cli::commands::wallet::wallet::{WalletNew, WalletNewArgs};
use clap::{Args, Subcommand};

pub mod send_value;
pub mod wallet;

#[derive(Debug, Args)]
#[command(name = "wallet", about = "wallet related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct WalletCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl WalletCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Send(args) => SendValue::handle(global, args).await,
            Commands::New(args) => WalletNew::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Send(SendValueArgs),
    New(WalletNewArgs),
}
