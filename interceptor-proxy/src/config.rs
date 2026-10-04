// src/config.rs

use clap::{Parser, ValueEnum};
use std::net::SocketAddr;
use crate::error::ProxyError;

/// Defines the traffic routing strategy used by proxy-circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ProxyMode {
    /// In Reverse mode, incoming paths are appended to a static upstream base URL.
    Reverse,
    /// In Forward mode, targets are dynamically resolved from the request line or Host header.
    Forward,
}

/// Command-line configuration for `proxy-circuit`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "proxy-circuit",
    author,
    version,
    about = "An asynchronous intercepting and injecting Layer 7 proxy"
)]
pub struct Config {
    /// The local port on which proxy-circuit will listen for incoming TCP connections.
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// The local IPv4/IPv6 host interface to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// The operational proxy mode: 'reverse' or 'forward'.
    #[arg(short, long, value_enum, default_value_t = ProxyMode::Reverse)]
    pub mode: ProxyMode,

    /// The base target URL forwarded to in reverse proxy mode (e.g., 'http://127.0.0.1:3000').
    #[arg(short, long)]
    pub upstream: Option<String>,
}

impl Config {
    /// Validates configuration invariants prior to starting the network listener.
    ///
    /// # Invariants
    /// - If `mode` is `ProxyMode::Reverse`, `upstream` MUST be `Some(url)` and start with `http://` or `https://`.
    /// - `host` and `port` must parse into a valid `std::net::SocketAddr`.
    pub fn validate(&self) -> Result<(), ProxyError> {
        if self.mode == ProxyMode::Reverse {
            match &self.upstream {
                None => return Err(ProxyError::MissingUpstream),
                Some(url) => {
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return Err(ProxyError::UnsupportedScheme(url.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    /// Resolves the socket address to bind to.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_mode_requires_upstream() {
        let cfg = Config {
            port: 8080,
            host: "127.0.0.1".to_string(),
            mode: ProxyMode::Reverse,
            upstream: None,
        };

        assert_eq!(cfg.validate(), Err(ProxyError::MissingUpstream));
    }

    #[test]
    fn test_reverse_mode_rejects_non_http_schemes() {
        let cfg = Config {
            port: 8080,
            host: "127.0.0.1".to_string(),
            mode: ProxyMode::Reverse,
            upstream: Some("ftp://storage.lan".to_string()),
        };

        assert_eq!(
            cfg.validate(),
            Err(ProxyError::UnsupportedScheme("ftp://storage.lan".to_string()))
        );
    }

    #[test]
    fn test_valid_reverse_config() {
        let cfg = Config {
            port: 8080,
            host: "127.0.0.1".to_string(),
            mode: ProxyMode::Reverse,
            upstream: Some("http://localhost:3000".to_string()),
        };

        assert!(cfg.validate().is_ok());
        assert_eq!(
            cfg.socket_addr().unwrap(),
            "127.0.0.1:8080".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn test_forward_mode_does_not_require_upstream() {
        let cfg = Config {
            port: 8080,
            host: "127.0.0.1".to_string(),
            mode: ProxyMode::Forward,
            upstream: None,
        };

        assert!(cfg.validate().is_ok());
    }
}
