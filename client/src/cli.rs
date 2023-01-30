///! This contains the CLI related helper functions

#[macro_export]
macro_rules! register_cli_command {
    ( $({$name:ident, $handler:ident}), *) => {
        use clap::{Parser, Subcommand};
        use client::CommandLineHandler;

        /// The overall command line struct
        #[derive(std::fmt::Debug, Parser)]
        #[command(
            name = "ipc",
            about = "The IPC node command line tool",
            version = "v0.0.1"
        )]
        #[command(propagate_version = true)]
        struct IPCNode {
            #[command(subcommand)]
            command: Commands,
        }

        /// The subcommand to be called
        #[derive(Debug, Subcommand)]
        enum Commands {
            $(
                $name(<$handler as CommandLineHandler>::Request),
            )*

        }

        pub async fn cli() {
            let args = IPCNode::parse();
            let r = match &args.command {
            $(
                Commands::$name(n) => <$handler as CommandLineHandler>::handle(n).await,
            )*
            };

            if r.is_err() {
                println!(
                    "process command: {:?} failed due to {:?}",
                    args.command,
                    r.unwrap_err()
                )
            }
        }
    };
}
