use std::fmt;
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceKind {
    S3,
}

impl ServiceKind {
    pub fn binary_name(self) -> &'static str {
        match self {
            Self::S3 => "awrust-s3-server",
        }
    }

    pub fn listen_env_var(self) -> &'static str {
        match self {
            Self::S3 => "AWRUST_S3_LISTEN_ADDR",
        }
    }

    pub fn from_sigv4_name(name: &str) -> Option<Self> {
        match name {
            "s3" => Some(Self::S3),
            _ => None,
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "s3" => Some(Self::S3),
            _ => None,
        }
    }
}

impl fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::S3 => write!(f, "s3"),
        }
    }
}

pub struct Config {
    pub listen_addr: SocketAddr,
    pub services: Vec<ServiceKind>,
    pub log_filter: String,
}

impl Config {
    pub fn from_env() -> Self {
        let listen_addr = std::env::var("AWRUST_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:4566".to_string())
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

        Self {
            listen_addr,
            services,
            log_filter,
        }
    }
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
