//! This mod contains the different command line implementations.

mod server;

use crate::cli::CommandLineHandler;
use clap::{Parser, Subcommand};
use std::fmt::Debug;

/// The `cli` method exposed to handle all the cli commands, ideally from main.
///
/// Consider using macro_rules! to handle the registration of new CLI handlers.
///
/// # Examples
/// Sample usage:
/// ```ignore
/// # to start the daemon with
/// ipc-client daemon ./examples/sample_config.toml
/// ```
///
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
        Commands::Daemon(args) => <server::LaunchJsonRPC as CommandLineHandler>::handle(args).await,
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
    /// Launch the ipc agent daemon.
    ///
    /// Note that, technically speaking, this just launches the ipc agent node and runs in the foreground
    /// and not in the background as what daemon processes are. Still, this struct contains `Daemon`
    /// due to the convention from `lotus` and the expected behavior from the filecoin user group.
    Daemon(<server::LaunchJsonRPC as CommandLineHandler>::Arguments),
}

/// The overall command line struct to be used by `clap`.
#[derive(Debug, Parser)]
#[command(name = "ipc", about = "The IPC agent command line tool")]
#[command(propagate_version = true)]
struct IPCAgentCliCommands {
    #[command(subcommand)]
    command: Commands,
}
