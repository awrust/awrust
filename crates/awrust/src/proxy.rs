use std::collections::HashMap;
use std::net::SocketAddr;

use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

use crate::config::ServiceKind;

pub struct Proxy {
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Incoming>,
    targets: HashMap<ServiceKind, SocketAddr>,
}

impl Proxy {
    pub fn new(targets: HashMap<ServiceKind, SocketAddr>) -> Self {
        let client = Client::builder(TokioExecutor::new()).build_http();
        Self { client, targets }
    }

    pub async fn forward(
        &self,
        target: ServiceKind,
        req: Request<Incoming>,
    ) -> Result<Response<Incoming>, hyper_util::client::legacy::Error> {
        let addr = self.targets[&target];
        let req = rewrite_uri(req, addr);
        self.client.request(req).await
    }

    pub fn has(&self, kind: ServiceKind) -> bool {
        self.targets.contains_key(&kind)
    }

    pub fn targets(&self) -> &HashMap<ServiceKind, SocketAddr> {
        &self.targets
    }
}

fn rewrite_uri(mut req: Request<Incoming>, addr: SocketAddr) -> Request<Incoming> {
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    *req.uri_mut() = Uri::builder()
        .scheme("http")
        .authority(addr.to_string())
        .path_and_query(path_and_query)
        .build()
        .expect("valid uri");

    req
}

pub fn service_unavailable(service: ServiceKind) -> Response<http_body_util::Full<Bytes>> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("content-type", "application/json")
        .body(http_body_util::Full::new(Bytes::from(format!(
            r#"{{"error":"service_unavailable","service":"{service}"}}"#
        ))))
        .expect("valid response")
}
