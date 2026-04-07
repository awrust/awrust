mod config;
mod tracing_init;

use config::Config;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    tracing_init::init(&config.log_filter);

    let services: Vec<String> = config.services.iter().map(|s| s.to_string()).collect();
    tracing::info!(
        listen = %config.listen_addr,
        services = ?services,
        "awrust starting"
    );
}
