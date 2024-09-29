use base64::{engine::general_purpose, Engine};
use chrono::Utc;
use serde::Deserialize;
use serde_json::from_slice;

#[derive(Debug, Deserialize)]
struct Claims {
    exp: usize,
}

fn decode_base64_segment(segment: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::URL_SAFE_NO_PAD.decode(segment.replace("-", "+").replace("_", "/"))
}


pub fn is_jwttoken_expired(jwt_token: String) -> bool {
    let segments: Vec<&str> = jwt_token.split('.').collect();

    let payload_segment = decode_base64_segment(segments[1]).expect("Invalid Base64 payload");
    let claims: Claims = from_slice(&payload_segment).expect("Invalid JSON payload");

    let exp_timestamp = claims.exp as i64;
    let now_timestamp = Utc::now().timestamp();

    return exp_timestamp < now_timestamp;
}