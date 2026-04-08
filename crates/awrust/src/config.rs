use std::fmt;
use std::net::{Ipv4Addr, SocketAddr};

use crate::dns;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceKind {
    S3,
}

impl ServiceKind {
    const ALL: &[(&'static str, Self)] = &[("s3", Self::S3)];

    pub fn binary_name(self) -> String {
        format!("awrust-{self}-server")
    }

    pub fn listen_env_var(self) -> String {
        format!("AWRUST_{}_LISTEN_ADDR", self.as_str().to_ascii_uppercase())
    }

    pub fn base_domain_env_var(self) -> String {
        format!("AWRUST_{}_BASE_DOMAIN", self.as_str().to_ascii_uppercase())
    }

    pub fn from_sigv4_name(name: &str) -> Option<Self> {
        Self::ALL.iter().find(|(n, _)| *n == name).map(|(_, k)| *k)
    }

    fn from_str(s: &str) -> Option<Self> {
        Self::ALL.iter().find(|(n, _)| *n == s).map(|(_, k)| *k)
    }

    fn as_str(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(_, k)| *k == self)
            .map(|(n, _)| *n)
            .expect("variant in ALL")
    }
}

impl fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct Config {
    pub listen_addr: SocketAddr,
    pub services: Vec<ServiceKind>,
    pub log_filter: String,
    pub base_domain: String,
    pub dns: Option<dns::DnsConfig>,
}

impl Config {
    pub fn from_env() -> Self {
        let listen_addr = std::env::var("AWRUST_LISTEN_ADDR")
            .unwrap_or_else(|_| "[::]:4566".to_string())
            .parse()
            .expect("valid AWRUST_LISTEN_ADDR");

        let services = std::env::var("AWRUST_SERVICES")
            .unwrap_or_else(|_| "s3".to_string())
            .split(',')
            .map(|s| {
                ServiceKind::from_str(s.trim()).unwrap_or_else(|| panic!("unknown service: {s}"))
            })
            .collect();

        let log_filter = std::env::var("AWRUST_LOG").unwrap_or_else(|_| "info".to_string());

        let base_domain =
            std::env::var("AWRUST_BASE_DOMAIN").unwrap_or_else(|_| "localhost".to_string());

        let dns = parse_dns_config(&base_domain);

        Self {
            listen_addr,
            services,
            log_filter,
            base_domain,
            dns,
        }
    }
}

fn parse_dns_config(base_domain: &str) -> Option<dns::DnsConfig> {
    let enabled = std::env::var("AWRUST_DNS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if !enabled {
        return None;
    }

    let listen_addr = std::env::var("AWRUST_DNS_ADDR")
        .unwrap_or_else(|_| "[::]:53".to_string())
        .parse()
        .expect("valid AWRUST_DNS_ADDR");

    let resolve_ip = std::env::var("AWRUST_DNS_RESOLVE_IP")
        .ok()
        .map(|v| v.parse::<Ipv4Addr>().expect("valid AWRUST_DNS_RESOLVE_IP"))
        .unwrap_or_else(dns::detect_resolve_ip);

    let upstream = std::env::var("AWRUST_DNS_UPSTREAM")
        .ok()
        .map(|v| v.parse::<SocketAddr>().expect("valid AWRUST_DNS_UPSTREAM"))
        .unwrap_or_else(dns::detect_upstream);

    Some(dns::DnsConfig {
        listen_addr,
        resolve_ip,
        base_domain: base_domain.to_string(),
        upstream,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_kind_binary_name() {
        assert_eq!(ServiceKind::S3.binary_name(), "awrust-s3-server");
    }

    #[test]
    fn service_kind_listen_env_var() {
        assert_eq!(ServiceKind::S3.listen_env_var(), "AWRUST_S3_LISTEN_ADDR");
    }

    #[test]
    fn service_kind_base_domain_env_var() {
        assert_eq!(
            ServiceKind::S3.base_domain_env_var(),
            "AWRUST_S3_BASE_DOMAIN"
        );
    }

    #[test]
    fn service_kind_from_str() {
        assert_eq!(ServiceKind::from_str("s3"), Some(ServiceKind::S3));
        assert_eq!(ServiceKind::from_str("invalid"), None);
    }

    #[test]
    fn service_kind_display() {
        assert_eq!(format!("{}", ServiceKind::S3), "s3");
    }

    #[test]
    #[should_panic(expected = "unknown service")]
    fn unknown_service_panics() {
        ServiceKind::from_str("invalid").unwrap_or_else(|| panic!("unknown service: invalid"));
    }
}
