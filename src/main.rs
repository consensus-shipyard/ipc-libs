mod cli;
mod config;
mod jsonrpc;
mod lotus;

#[tokio::main]
async fn main() {
    cli::cli().await;
}
