// src/resolver.rs

use http::Uri;
use std::str::FromStr;
use crate::config::ProxyMode;
use crate::error::ProxyError;

/// Resolves the destination URI for an in-flight request.
///
/// # Arguments
/// * `incoming_uri` - The URI extracted from the incoming HTTP request line.
/// * `host_header` - Optional value from the HTTP `Host` header.
/// * `mode` - Whether the proxy is operating in Reverse or Forward mode.
/// * `configured_upstream` - The configured base target (only used in Reverse mode).
pub fn resolve_destination(
    incoming_uri: &Uri,
    host_header: Option<&str>,
    mode: ProxyMode,
    configured_upstream: Option<&str>,
) -> Result<Uri, ProxyError> {
    match mode {
        ProxyMode::Reverse => {
            let upstream_base = configured_upstream.ok_or(ProxyError::MissingUpstream)?;

            // Extract path and query parameters (e.g., "/api/v1/resource?id=1")
            // Default to "/" if the path is empty.
            let path_and_query = incoming_uri
                .path_and_query()
                .map(|pq| pq.as_str())
                .unwrap_or("/");

            // Strip any trailing slash from upstream base to avoid double-slashes ("//")
            let sanitized_base = upstream_base.trim_end_matches('/');

            let full_url = format!("{}{}", sanitized_base, path_and_query);
            Uri::from_str(&full_url).map_err(|_| ProxyError::InvalidUri(full_url))
        }

        ProxyMode::Forward => {
            // Case A: The client sent an absolute URI (standard HTTP proxy spec: GET http://target.com/path)
            if incoming_uri.scheme().is_some() && incoming_uri.authority().is_some() {
                return Ok(incoming_uri.clone());
            }

            // Case B: The client sent a relative URI (GET /path). We must extract the host from the Host header.
            let host = host_header.ok_or(ProxyError::MissingTargetHost)?;
            let path_and_query = incoming_uri
                .path_and_query()
                .map(|pq| pq.as_str())
                .unwrap_or("/");

            let full_url = format!("http://{}{}", host, path_and_query);
            Uri::from_str(&full_url).map_err(|_| ProxyError::InvalidUri(full_url))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_proxy_resolution_with_query_params() {
        let incoming: Uri = "/orders?status=active".parse().unwrap();
        let upstream = "http://127.0.0.1:3000";

        let target = resolve_destination(&incoming, None, ProxyMode::Reverse, Some(upstream)).unwrap();

        assert_eq!(target.to_string(), "http://127.0.0.1:3000/orders?status=active");
    }

    #[test]
    fn test_reverse_proxy_strips_trailing_slash_overlap() {
        let incoming: Uri = "/metrics".parse().unwrap();
        let upstream = "http://127.0.0.1:3000/"; // Trailing slash present

        let target = resolve_destination(&incoming, None, ProxyMode::Reverse, Some(upstream)).unwrap();

        assert_eq!(target.to_string(), "http://127.0.0.1:3000/metrics");
    }

    #[test]
    fn test_forward_proxy_with_absolute_uri() {
        let incoming: Uri = "http://example.com/index.html".parse().unwrap();

        let target = resolve_destination(&incoming, None, ProxyMode::Forward, None).unwrap();

        assert_eq!(target.to_string(), "http://example.com/index.html");
    }

    #[test]
    fn test_forward_proxy_fallback_to_host_header() {
        let incoming: Uri = "/api/v1/ping".parse().unwrap();
        let host = "service.internal:9000";

        let target = resolve_destination(&incoming, Some(host), ProxyMode::Forward, None).unwrap();

        assert_eq!(target.to_string(), "http://service.internal:9000/api/v1/ping");
    }

    #[test]
    fn test_forward_proxy_fails_without_host() {
        let incoming: Uri = "/relative/path".parse().unwrap();

        let err = resolve_destination(&incoming, None, ProxyMode::Forward, None).unwrap_err();

        assert_eq!(err, ProxyError::MissingTargetHost);
    }
}
