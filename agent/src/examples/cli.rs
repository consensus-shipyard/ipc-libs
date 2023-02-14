use agent::create_cli;
use foo::FooCmd;

mod foo {
    use agent::cli::CommandLineHandler;
    use agent::error::Error;
    use async_trait::async_trait;
    use clap::Args;
    use std::fmt::Debug;

    #[derive(Debug, Args)]
    #[command(about = "Performs a foo operation")]
    pub struct FooArgs {
        #[arg(
            long,
            value_name = "FOO_PARAM",
            help = "The argument for foo",
            env = "FOO_PARAM_ENV"
        )]
        foo_param: Option<String>,
    }

    pub struct FooCmd {}

    #[async_trait]
    impl CommandLineHandler for FooCmd {
        type Request = FooArgs;

        async fn handle(_request: &Self::Request) -> Result<String, Error> {
            Ok(String::from("foo"))
        }
    }
}

create_cli!(
    {FooCli, FooCmd}
    // add more command line handlers here
    // {BarCli, BarCmd}
    // ...
);

#[tokio::main]
async fn main() {
    // The following should be printed:
    // The IPC agent command line tool
    //
    // Usage: cli <COMMAND>
    //
    // Commands:
    //   foo-cli  Performs a foo operation
    //   help     Print this message or the help of the given subcommand(s)
    //
    // Options:
    //   -h, --help     Print help
    //   -V, --version  Print version
    //
    // Process finished with exit code 2
    cli().await
}
