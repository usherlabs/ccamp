use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD as base64, Engine};
use http::{header, HeaderValue, Method};
use k256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use reqwest::{IntoUrl, Response, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{request::RequestBuilder, Result};

#[derive(Clone)]
pub struct VerityClientConfig {
    pub prover_url: String,
    pub prover_zmq: String,
    pub analysis_url: String,
    pub signing_key: SigningKey,
}

#[derive(Clone)]
pub struct VerityClient {
    pub(crate) inner: reqwest::Client,
    pub(crate) config: VerityClientConfig,
    pub(crate) token: Option<String>,
}

impl VerityClient {
    pub fn new(config: VerityClientConfig) -> Self {
        return Self {
            inner: reqwest::Client::new(),
            config,
            token: None,
        };
    }

    async fn auth(&mut self) {
        let (session_id, challenge) = self.get_challenge().await;

        let signature: Signature = self.config.signing_key.sign(challenge.as_bytes());
        let signature = base64.encode(signature.to_der().to_bytes());

        let token = self.post_challenge(session_id, signature).await;
        self.token = Some(token);
    }

    async fn get_challenge(&self) -> (Uuid, String) {
        #[derive(Debug, Serialize)]
        struct Request {
            public_key: String,
        }

        #[derive(Debug, Deserialize)]
        struct Response {
            session_id: Uuid,
            challenge: String,
        }

        let url = format!("{}/auth/challenge", self.config.analysis_url);
        let client = reqwest::Client::new();

        let verifying_key = VerifyingKey::from(&self.config.signing_key);

        let request = Request {
            public_key: base64.encode(verifying_key.to_sec1_bytes()),
        };

        let response = client
            .get(url)
            .json(&request)
            .send()
            .await
            .unwrap()
            .json::<Response>()
            .await
            .unwrap();

        (response.session_id, response.challenge)
    }

    async fn post_challenge(&self, session_id: Uuid, signature: String) -> String {
        #[derive(Debug, Serialize)]
        struct Request {
            session_id: Uuid,
            signature: String,
        }

        #[derive(Debug, Deserialize)]
        struct Response {
            token: String,
        }

        let url = format!("{}/auth/challenge", self.config.analysis_url);
        let client = reqwest::Client::new();

        let request = Request {
            session_id,
            signature,
        };

        let response = client
            .post(url)
            .json(&request)
            .send()
            .await
            .unwrap()
            .json::<Response>()
            .await
            .unwrap();

        response.token
    }

    /// Convenience method to make a `GET` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// Convenience method to make a `POST` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    /// Start building a `Request` with the `Method` and `Url`.
    ///
    /// Returns a `RequestBuilder`, which will allow setting headers and
    /// the request body before sending.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
            config: self.config.clone(),
        }
    }

    /// Executes a `Request`.
    ///
    /// A `Request` can be built manually with `Request::new()` or obtained
    /// from a RequestBuilder with `RequestBuilder::build()`.
    ///
    /// You should prefer to use the `RequestBuilder` and
    /// `RequestBuilder::send()`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    pub async fn execute(&mut self, request: reqwest::Request) -> Result<Response> {
        self.execute_request(request).await
    }

    pub async fn execute_request(&mut self, mut req: reqwest::Request) -> Result<Response> {
        if self.token.is_none() {
            self.auth().await;
        }

        let proxy_url = &String::from(req.url().as_str());

        let headers = req.headers_mut();
        headers.append("T-PROXY-URL", HeaderValue::from_str(proxy_url).unwrap());

        let header_value = &format!("Bearer {}", self.token.clone().unwrap());
        headers.append(
            header::AUTHORIZATION,
            HeaderValue::from_str(header_value).unwrap(),
        );

        *req.url_mut() = Url::from_str(&format!("{}/proxy", self.config.prover_url)).unwrap();

        let req = reqwest::RequestBuilder::from_parts(self.inner.clone(), req);

        req.send().await.map_err(crate::Error::Reqwest)
    }
}
