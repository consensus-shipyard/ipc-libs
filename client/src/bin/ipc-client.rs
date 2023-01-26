use client::{cli, health_check, start_node, HEALTH_CHECK, NODE};

#[tokio::main]
async fn main() {
    env_logger::init();

    let cmd = cli();
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some((NODE, matches)) => start_node(matches).await,
        Some((HEALTH_CHECK, matches)) => health_check(matches).await,
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
