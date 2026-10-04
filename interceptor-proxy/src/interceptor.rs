// src/interceptor.rs

use http::header::{HeaderName, HeaderValue};
use http::{HeaderMap, Request};
use uuid::Uuid;

/// Constant header keys used by proxy-circuit
pub const HEADER_REQUEST_ID: &str = "x-request-id";
pub const HEADER_PROXY_CIRCUIT: &str = "x-proxy-circuit";
pub const HEADER_FORWARDED_HOST: &str = "x-forwarded-host";

/// Enriches an incoming request with tracing, proxy provenance, and routing metadata.
///
/// # Invariants
/// - If `x-request-id` is already present, it is preserved (to support distributed trace propagation).
/// - If absent, a new UUIDv4 is generated and injected.
/// - Injects `x-proxy-circuit: active` to signal interception.
/// - Sets `x-forwarded-host` based on the original `Host` header if present.
pub fn mutate_request_headers<B>(req: &mut Request<B>) {
    let headers: &mut HeaderMap = req.headers_mut();

    // 1. Ensure a correlation/trace ID is present
    if !headers.contains_key(HEADER_REQUEST_ID) {
        let req_id = Uuid::new_v4().to_string();
        if let Ok(val) = HeaderValue::from_str(&req_id) {
            headers.insert(HeaderName::from_static(HEADER_REQUEST_ID), val);
        }
    }

    // 2. Inject provenance header
    headers.insert(
        HeaderName::from_static(HEADER_PROXY_CIRCUIT),
        HeaderValue::from_static("active"),
    );

    // 3. Inject X-Forwarded-Host if the incoming request carried a Host header
    if let Some(host) = headers.get(http::header::HOST).cloned() {
        headers.insert(HeaderName::from_static(HEADER_FORWARDED_HOST), host);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;

    #[test]
    fn test_injects_request_id_when_missing() {
        let mut req = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();

        mutate_request_headers(&mut req);

        let headers = req.headers();
        assert!(headers.contains_key(HEADER_REQUEST_ID));
        assert_eq!(headers.get(HEADER_PROXY_CIRCUIT).unwrap(), "active");
    }

    #[test]
    fn test_preserves_existing_request_id() {
        let existing_id = "trace-123-abc";
        let mut req = Request::builder()
            .uri("/test")
            .header(HEADER_REQUEST_ID, existing_id)
            .body(())
            .unwrap();

        mutate_request_headers(&mut req);

        let headers = req.headers();
        assert_eq!(headers.get(HEADER_REQUEST_ID).unwrap(), existing_id);
    }

    #[test]
    fn test_injects_forwarded_host_if_host_header_present() {
        let mut req = Request::builder()
            .uri("/test")
            .header(http::header::HOST, "api.internal.local:8080")
            .body(())
            .unwrap();

        mutate_request_headers(&mut req);

        let headers = req.headers();
        assert_eq!(
            headers.get(HEADER_FORWARDED_HOST).unwrap(),
            "api.internal.local:8080"
        );
    }
}
