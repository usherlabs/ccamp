use candid::CandidType;
use serde_derive::Serialize;

#[derive(Clone, CandidType, Serialize)]
pub struct ParsedHeader {
    name: String,
    value: String,
}

impl From<&httparse::Header<'_>> for ParsedHeader {
    fn from(header: &httparse::Header<'_>) -> Self {
        ParsedHeader {
            name: header.name.to_string(),
            value: String::from_utf8_lossy(header.value).to_string(),
        }
    }
}

#[derive(Clone, CandidType, Serialize)]
pub struct ParsedRequest {
    method: Option<String>,
    path: Option<String>,
    version: Option<u8>,
    headers: Vec<ParsedHeader>,
}

impl From<httparse::Request<'_, '_>> for ParsedRequest {
    fn from(req: httparse::Request<'_, '_>) -> Self {
        ParsedRequest {
            method: req.method.map(|m| m.to_string()),
            path: req.path.map(|p| p.to_string()),
            version: req.version,
            headers: req.headers.iter().map(ParsedHeader::from).collect(),
        }
    }
}

#[derive(Clone, CandidType, Serialize)]
pub struct ParsedResponse {
    version: Option<u8>,
    code: Option<u16>,
    reason: Option<String>,
    headers: Vec<ParsedHeader>,
    body: String,
}

impl From<(httparse::Response<'_, '_>, &[u8])> for ParsedResponse {
    fn from((res, body): (httparse::Response<'_, '_>, &[u8])) -> Self {
        ParsedResponse {
            version: res.version,
            code: res.code,
            reason: res.reason.map(|r| r.to_string()),
            headers: res.headers.iter().map(ParsedHeader::from).collect(),
            body: String::from_utf8_lossy(body).to_string(),
        }
    }
}