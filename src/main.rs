mod cli;
mod config;
mod jsonrpc;
mod lotus;
mod server;

#[tokio::main]
async fn main() {
    cli::cli().await;
}
