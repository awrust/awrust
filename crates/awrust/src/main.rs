mod config;
mod dns;
mod health;
mod process;
mod proxy;
mod router;
mod tracing_init;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;

use config::Config;
use process::ProcessManager;
use proxy::Proxy;

type BoxBody = http_body_util::Either<Incoming, http_body_util::Full<bytes::Bytes>>;

struct AppState {
    proxy: Proxy,
    targets: std::collections::HashMap<config::ServiceKind, std::net::SocketAddr>,
}

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

    let manager = ProcessManager::start(&config.services, &config.base_domain).await;
    manager.wait_healthy(Duration::from_secs(15)).await;

    let state = Arc::new(AppState {
        proxy: Proxy::new(manager.targets()),
        targets: manager.targets(),
    });

    if let Some(dns_config) = config.dns {
        tokio::spawn(dns::serve(dns_config));
    }

    let listener = TcpListener::bind(config.listen_addr)
        .await
        .expect("bind listen address");
    tracing::info!(listen = %config.listen_addr, "accepting connections");

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted.expect("accept connection");
                let state = Arc::clone(&state);

                tokio::spawn(async move {
                    let service = service_fn(move |req: Request<Incoming>| {
                        let state = Arc::clone(&state);
                        async move { Ok::<_, Infallible>(handle(req, &state).await) }
                    });

                    if let Err(e) = Builder::new(TokioExecutor::new())
                        .serve_connection(TokioIo::new(stream), service)
                        .await
                    {
                        tracing::debug!(error = %e, "connection error");
                    }
                });
            }
            _ = &mut shutdown => {
                tracing::info!("shutting down");
                break;
            }
        }
    }

    manager.shutdown().await;
}

async fn handle(req: Request<Incoming>, state: &AppState) -> Response<BoxBody> {
    if health::is_facade_health_check(&req) {
        return health::check(&state.targets)
            .await
            .map(http_body_util::Either::Right);
    }

    let target = router::route(&req);

    if !state.proxy.has(target) {
        return proxy::service_unavailable(target).map(http_body_util::Either::Right);
    }

    match state.proxy.forward(target, req).await {
        Ok(resp) => resp.map(http_body_util::Either::Left),
        Err(e) => {
            tracing::error!(service = %target, error = %e, "proxy error");
            proxy::service_unavailable(target).map(http_body_util::Either::Right)
        }
    }
}
