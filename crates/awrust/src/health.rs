use std::collections::HashMap;
use std::net::SocketAddr;

use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use tokio::net::TcpStream;

use crate::config::ServiceKind;

pub fn is_facade_health_check<B>(req: &Request<B>) -> bool {
    req.uri().path() == "/health"
        && !req.headers().contains_key("authorization")
        && !req
            .headers()
            .keys()
            .any(|k| k.as_str().starts_with("x-amz-"))
}

pub async fn check(
    targets: &HashMap<ServiceKind, SocketAddr>,
    init_done: bool,
) -> Response<http_body_util::Full<Bytes>> {
    let mut all_healthy = true;
    let mut entries = Vec::new();

    for (&kind, &addr) in targets {
        let healthy = TcpStream::connect(addr).await.is_ok();
        if !healthy {
            all_healthy = false;
        }
        let status = if healthy { "ok" } else { "unhealthy" };
        entries.push(format!(r#""{kind}":{{"status":"{status}"}}"#));
    }

    let (status_str, http_status) = if !init_done {
        ("initializing", StatusCode::SERVICE_UNAVAILABLE)
    } else if all_healthy {
        ("ok", StatusCode::OK)
    } else {
        ("degraded", StatusCode::SERVICE_UNAVAILABLE)
    };

    let body = format!(
        r#"{{"status":"{status_str}","services":{{{}}}}}"#,
        entries.join(",")
    );

    Response::builder()
        .status(http_status)
        .header("content-type", "application/json")
        .body(http_body_util::Full::new(Bytes::from(body)))
        .expect("valid response")
}
