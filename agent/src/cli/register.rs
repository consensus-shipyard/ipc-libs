/// This macro creates the command line method to be invoked by the caller, i.e. a method cli().
/// This macro takes a list of tuple of the form: {METHOD_NAME, Handler Struct}. The handler struct
/// implements `cli::CommandLineHandler`. The generated `cli` method will handle the invocation and
/// print the response to `log`.
///
/// TODO: this macro relies on `clap` directly. It is possible to create an abstraction on top of
/// `clap` so that this macro does not directly reply on external crates. But that would significantly
/// increase the complexity. Using clap should be enough for now.
///
/// # Examples
/// ```no_run
/// use agent::{create_cli, cli::CommandLineHandler, error::Error};
///
/// use std::fmt::Debug;
/// use async_trait::async_trait;
/// use clap::Args;
///
/// #[derive(Debug, Args)]
/// #[command(about = "Performs a foo operation")]
/// struct FooArgs {
///     #[arg(
///         long,
///         value_name = "FOO_PARAM",
///         help = "The argument for foo",
///         env = "FOO_PARAM_ENV"
///     )]
///     foo_param: Option<String>,
/// }
///
/// struct FooCmd {}
///
/// #[async_trait]
/// impl CommandLineHandler for FooCmd {
///     type Request = FooArgs;
///
///     async fn handle(request: &Self::Request) -> Result<String, Error> {
///         Ok(String::from("foo"))
///     }
/// }
///
/// create_cli!(
///     {FooCli, FooCmd}
/// );
///
/// #[tokio::main]
/// async fn main() {
///     cli().await;
/// }
/// ```
#[macro_export]
macro_rules! create_cli {
    ( $({$name:ident, $handler:ty}), *) => {
        use clap::{Parser, Subcommand};

        /// The overall command line struct
        #[derive(std::fmt::Debug, Parser)]
        #[command(
            name = "ipc",
            about = "The IPC agent command line tool",
            version = "v0.0.1"
        )]
        #[command(propagate_version = true)]
        struct IPCCliCommands {
            #[command(subcommand)]
            command: Commands,
        }

        /// The subcommand to be called, see clap's documentation for usage.
        #[derive(Debug, Subcommand)]
        enum Commands {
            $(
                $name(<$handler as $crate::cli::CommandLineHandler>::Request),
            )*

        }

        pub async fn cli() {
            let args = IPCCliCommands::parse();
            let r = match &args.command {
            $(
                Commands::$name(n) => <$handler as $crate::cli::CommandLineHandler>::handle(n).await,
            )*
            };

            if r.is_err() {
                log::error!(
                    "process command: {:?} failed due to error: {:?}",
                    args.command,
                    r.unwrap_err()
                )
            } else {
                log::info!("{}", r.unwrap())
            }
        }
    }
}
