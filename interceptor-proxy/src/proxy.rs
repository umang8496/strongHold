// src/proxy.rs

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::sync::Arc;

use crate::config::Config;
use crate::interceptor::mutate_request_headers;
use crate::mutation::{is_json_content, mutate_json_payload, update_content_length};
use crate::resolver::resolve_destination;

pub type UpstreamClient = Client<HttpConnector, BoxBody<Bytes, hyper::Error>>;

pub fn build_upstream_client() -> UpstreamClient {
    Client::builder(hyper_util::rt::TokioExecutor::new()).build_http()
}

pub async fn handle_request(
    mut req: Request<Incoming>,
    config: Arc<Config>,
    client: UpstreamClient,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::convert::Infallible> {
    // 1. Resolve Target Destination
    let host_header = req
        .headers()
        .get(http::header::HOST)
        .and_then(|h| h.to_str().ok());

    let target_uri = match resolve_destination(
        req.uri(),
        host_header,
        config.mode,
        config.upstream.as_deref(),
    ) {
        Ok(uri) => uri,
        Err(err) => {
            eprintln!("[Routing Error]: {err}");
            let bad_request = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(
                    Full::new(Bytes::from(format!("Routing Error: {err}")))
                        .map_err(|e| match e {})
                        .boxed(),
                )
                .unwrap();
            return Ok(bad_request);
        }
    };

    // 2. Intercept and mutate request headers
    mutate_request_headers(&mut req);

    // 3. Rewrite Target URI and Host Header
    *req.uri_mut() = target_uri.clone();
    if let Some(target_host) = target_uri.authority() {
        if let Ok(host_val) = http::HeaderValue::from_str(target_host.as_str()) {
            req.headers_mut().insert(http::header::HOST, host_val);
        }
    }

    // 4. Inspect & Mutate Body if Content-Type is JSON and method allows payload
    let (mut parts, incoming_body) = req.into_parts();

    let final_body: BoxBody<Bytes, hyper::Error> = if should_inspect_body(&parts.method, &parts.headers) {
        // Collect incoming byte stream into memory
        match incoming_body.collect().await {
            Ok(collected) => {
                let raw_bytes = collected.to_bytes();

                // Mutate payload if valid JSON object
                if let Some(mutated_bytes) = mutate_json_payload(&raw_bytes) {
                    let new_len = mutated_bytes.len();
                    update_content_length(&mut parts.headers, new_len);
                    Full::new(Bytes::from(mutated_bytes))
                        .map_err(|e| match e {})
                        .boxed()
                } else {
                    // Fall back to original bytes if not a JSON object
                    Full::new(raw_bytes)
                        .map_err(|e| match e {})
                        .boxed()
                }
            }
            Err(err) => {
                eprintln!("[Body Ingestion Failed]: {err}");
                let err_res = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(
                        Full::new(Bytes::from("Failed reading request body"))
                            .map_err(|e| match e {})
                            .boxed(),
                    )
                    .unwrap();
                return Ok(err_res);
            }
        }
    } else {
        // Stream directly without in-memory buffering for non-JSON or bodyless requests
        incoming_body.boxed()
    };

    let upstream_req = Request::from_parts(parts, final_body);

    println!(
        " ↳ Forwarding [{}] {} -> {}",
        upstream_req.method(),
        upstream_req.uri().path(),
        target_uri
    );

    // 5. Dispatch upstream call
    match client.request(upstream_req).await {
        Ok(upstream_res) => {
            let (res_parts, res_body) = upstream_res.into_parts();
            Ok(Response::from_parts(res_parts, res_body.boxed()))
        }
        Err(err) => {
            eprintln!("[Upstream Connection Failed]: {err}");
            let error_response = Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(
                    Full::new(Bytes::from(format!("Upstream Gateway Error: {err}")))
                        .map_err(|e| match e {})
                        .boxed(),
                )
                .unwrap();
            Ok(error_response)
        }
    }
}

/// Determines if request semantics allow a request body and if it claims to be JSON.
fn should_inspect_body(method: &Method, headers: &http::HeaderMap) -> bool {
    let has_body_semantics = matches!(*method, Method::POST | Method::PUT | Method::PATCH);
    has_body_semantics && is_json_content(headers)
}