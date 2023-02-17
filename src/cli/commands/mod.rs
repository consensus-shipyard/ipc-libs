//! The module that contains all the CLI commands.

mod info;

use crate::cli::CommandLineHandler;
use clap::{Parser, Subcommand};
use std::fmt::Debug;

/// The `cli` method exposed to handle all the cli commands, ideally from main.
///
/// Consider using macro_rules! to handle the registration of new CLI handlers.
///
/// # Examples
/// To register a new command, add the command to
/// ```ignore
/// pub async fn cli() {
///
///     // ... other code
///
///     let r = match &args.command {
///         // ... other existing commands
///         Commands::NewCommand => <NewCommand as CommandLineHandler>::handle(n).await,
///     };
///
///     // ... other code
/// ```
/// Also add this type to Command enum.
/// ```ignore
/// enum Commands {
///     NewCommand(<NewCommand as CommandLineHandler>::Arguments),
/// }
/// ```
pub async fn cli() {
    // parse the arguments
    let args = IPCAgentCliCommands::parse();

    let r = match &args.command {
        Commands::Info(n) => <info::Info as CommandLineHandler>::handle(n).await,
    };

    if let Err(e) = r {
        log::error!(
            "process command: {:?} failed due to error: {:?}",
            args.command,
            e
        )
    }
}

/// The collection of all subcommands to be called, see clap's documentation for usage. Internal
/// to the current mode. Register a new command accordingly.
#[derive(Debug, Subcommand)]
enum Commands {
    Info(<info::Info as CommandLineHandler>::Arguments),
}

/// The overall command line struct to be used by `clap`.
#[derive(Debug, Parser)]
#[command(
    name = "ipc",
    about = "The IPC agent command line tool",
    version = "v0.0.1"
)]
#[command(propagate_version = true)]
struct IPCAgentCliCommands {
    #[command(subcommand)]
    command: Commands,
}
