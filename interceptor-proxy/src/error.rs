// src/error.rs

use thiserror::Error;

/// Core error types for the proxy-circuit application.
/// 
/// `thiserror::Error` derives the standard `std::error::Error` trait and allows
/// formatting human-readable error messages via `#[error("...")]`.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProxyError {
    /// Raised when reverse proxy mode is chosen but no upstream address is supplied.
    #[error("Reverse proxy mode requires an upstream target URL (use --upstream <URL>)")]
    MissingUpstream,

    /// Raised when an incoming HTTP request in forward mode lacks both an absolute URI and a Host header.
    #[error("Forward proxy mode requires an absolute URI or a valid Host header")]
    MissingTargetHost,

    /// Raised when a URI cannot be parsed into a valid `http::Uri`.
    #[error("Malformed URI string: {0}")]
    InvalidUri(String),

    /// Raised when an upstream URL contains an invalid scheme (e.g., not http/https).
    #[error("Unsupported scheme '{0}': only http and https are supported")]
    UnsupportedScheme(String),
}
