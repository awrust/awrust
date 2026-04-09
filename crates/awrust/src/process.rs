use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::process::{Child, Command};
use tokio::time;

use crate::config::ServiceKind;

struct ManagedProcess {
    kind: ServiceKind,
    addr: SocketAddr,
    child: Child,
}

pub struct ProcessManager {
    processes: Vec<ManagedProcess>,
}

impl ProcessManager {
    pub async fn start(services: &[ServiceKind], base_domain: &str) -> Self {
        let mut processes = Vec::with_capacity(services.len());

        for &kind in services {
            let addr = allocate_port().await;
            let child = spawn(kind, addr, base_domain);
            tracing::info!(service = %kind, addr = %addr, "spawned");
            processes.push(ManagedProcess { kind, addr, child });
        }

        Self { processes }
    }

    pub fn targets(&self) -> HashMap<ServiceKind, SocketAddr> {
        self.processes.iter().map(|p| (p.kind, p.addr)).collect()
    }

    pub async fn wait_healthy(&self, timeout: Duration) {
        for proc in &self.processes {
            wait_for_health(proc.kind, proc.addr, timeout).await;
        }
    }

    pub async fn shutdown(mut self) {
        for proc in &mut self.processes {
            let _ = proc.child.kill().await;
        }
        for proc in &mut self.processes {
            let _ = proc.child.wait().await;
        }
    }
}

async fn allocate_port() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    listener.local_addr().expect("local addr")
}

fn spawn(kind: ServiceKind, addr: SocketAddr, base_domain: &str) -> Child {
    let mut cmd = Command::new(kind.binary_name());
    cmd.env(kind.listen_env_var(), addr.to_string());

    if std::env::var(kind.base_domain_env_var()).is_err() {
        cmd.env(
            kind.base_domain_env_var(),
            kind.qualified_base_domain(base_domain),
        );
    }

    cmd.kill_on_drop(true)
        .spawn()
        .unwrap_or_else(|e| panic!("{} not found on PATH: {e}", kind.binary_name()))
}

async fn wait_for_health(kind: ServiceKind, addr: SocketAddr, timeout: Duration) {
    let deadline = time::Instant::now() + timeout;
    let mut interval = time::interval(Duration::from_millis(100));

    loop {
        interval.tick().await;

        if time::Instant::now() > deadline {
            panic!("{kind} did not become healthy within {timeout:?}");
        }

        if TcpStream::connect(addr).await.is_ok() {
            tracing::info!(service = %kind, "healthy");
            return;
        }
    }
}
