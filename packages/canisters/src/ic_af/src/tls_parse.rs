use candid::CandidType;
use serde_derive::Serialize;

#[derive(Clone, CandidType, Serialize)]
pub struct ParsedHeader {
    name : String,
    value : Vec<u8>,
}

impl From<&httparse::Header<'_>> for ParsedHeader {
    fn from(header : &httparse::Header<'_>) -> Self {
        ParsedHeader {
            name : header.name.to_string(),
            value : header.value.to_vec(),
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
    fn from(req : httparse::Request<'_, '_>) -> Self {
        ParsedRequest {
            method : match req.method {
                Some(x) => Some(x.to_string()),
                None => None,
            },
            path : match req.path {
                Some(x) => Some(x.to_string()),
                None => None,
            },
            version : req.version,
            headers : req.headers.to_vec().iter().map(|v| ParsedHeader::from(v)).collect(),
        }
    }
}

#[derive(Clone, CandidType, Serialize)]
pub struct ParsedResponse {
    version: Option<u8>,
    code: Option<u16>,
    reason: Option<String>,
    headers: Vec<ParsedHeader>,
}

impl From<httparse::Response<'_, '_>> for ParsedResponse {
    fn from(res : httparse::Response<'_, '_>) -> Self {
        ParsedResponse {
            version : res.version,
            code : res.code,
            reason : match res.reason {
                Some(x) => Some(x.to_string()),
                None => None,
            },
            headers : res.headers.to_vec().iter().map(|v| ParsedHeader::from(v)).collect(),
        }
    }
}
