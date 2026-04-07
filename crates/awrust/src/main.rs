mod config;
mod process;
mod tracing_init;

use std::time::Duration;

use config::Config;
use process::ProcessManager;

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

    let manager = ProcessManager::start(&config.services).await;
    manager.wait_healthy(Duration::from_secs(15)).await;

    tracing::info!("all services healthy, awaiting shutdown signal");
    tokio::signal::ctrl_c().await.expect("listen for ctrl-c");

    tracing::info!("shutting down");
    manager.shutdown().await;
}
