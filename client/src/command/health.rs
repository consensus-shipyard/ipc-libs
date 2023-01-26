use clap::ArgMatches;

pub async fn health_check(_matches: &ArgMatches) {
    log::info!("performing health check... OK!");
}
