// src/mutation.rs

use bytes::Bytes;
use http::header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE};
use http::HeaderMap;
use serde_json::{json, Value};

/// Evaluates if the incoming request declares a JSON content type.
///
/// Looks for `application/json` or structured vendor suffixes like `application/problem+json`.
pub fn is_json_content(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|val| val.to_str().ok())
        .map(|ct| {
            let lower = ct.to_ascii_lowercase();
            lower.starts_with("application/json") || lower.contains("+json")
        })
        .unwrap_or(false)
}

/// Mutates a JSON byte slice by injecting proxy metadata.
///
/// # Behavior
/// - If the raw bytes parse into a JSON Object (`{ ... }`), it injects:
///   `"_proxy_circuit": { "intercepted": true, "timestamp_epoch_ms": <ms> }`
/// - If the root JSON is an Array (`[ ... ]`) or primitive, it skips mutation to preserve structure.
/// - If the bytes are empty or malformed JSON, returns `None` so the caller can fall back to raw bytes.
pub fn mutate_json_payload(raw_bytes: &[u8]) -> Option<Vec<u8>> {
    if raw_bytes.is_empty() {
        return None;
    }

    // Attempt zero-copy parsing of the byte buffer into a dynamic JSON structure
    let mut parsed: Value = serde_json::from_slice(raw_bytes).ok()?;

    // Only inject fields if the root element is a JSON Map/Object
    if let Value::Object(ref mut map) = parsed {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        map.insert(
            "_proxy_circuit".to_string(),
            json!({
                "intercepted": true,
                "timestamp_epoch_ms": now_ms
            }),
        );

        // Serialize mutated JSON back to bytes
        serde_json::to_vec(&parsed).ok()
    } else {
        None
    }
}

/// Updates or recalculates the Content-Length header to reflect the newly mutated payload length.
pub fn update_content_length(headers: &mut HeaderMap, new_length: usize) {
    if let Ok(len_val) = HeaderValue::from_str(&new_length.to_string()) {
        headers.insert(CONTENT_LENGTH, len_val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_is_json_detection() {
        let mut headers = HeaderMap::new();
        assert!(!is_json_content(&headers));

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        assert!(is_json_content(&headers));

        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        assert!(is_json_content(&headers));

        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        assert!(is_json_content(&headers));

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
        assert!(!is_json_content(&headers));
    }

    #[test]
    fn test_injects_metadata_into_json_object() {
        let raw = br#"{"order_id": 1001, "item": "mechanical_switch"}"#;
        let mutated = mutate_json_payload(raw).expect("Expected successful mutation");

        let parsed: Value = serde_json::from_slice(&mutated).unwrap();
        assert_eq!(parsed["order_id"], 1001);
        assert_eq!(parsed["item"], "mechanical_switch");
        assert_eq!(parsed["_proxy_circuit"]["intercepted"], true);
        assert!(parsed["_proxy_circuit"]["timestamp_epoch_ms"].is_number());
    }

    #[test]
    fn test_skips_json_array() {
        let raw = br#"[1, 2, 3]"#;
        assert_eq!(mutate_json_payload(raw), None);
    }

    #[test]
    fn test_skips_invalid_json() {
        let raw = b"invalid plain text {";
        assert_eq!(mutate_json_payload(raw), None);
    }

    #[test]
    fn test_content_length_updated() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("40"));

        update_content_length(&mut headers, 128);

        assert_eq!(
            headers.get(CONTENT_LENGTH).unwrap().to_str().unwrap(),
            "128"
        );
    }
}