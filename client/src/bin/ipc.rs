use client::cli;

#[tokio::main]
async fn main() {
    env_logger::init();
    cli().await;
}
