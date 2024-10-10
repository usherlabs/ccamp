use std::{future::IntoFuture, str::FromStr};

use base64::prelude::*;
use base64::Engine;
use http::{HeaderValue, Method};
use k256::ecdsa::SigningKey;
use k256::ecdsa::{signature::Signer, Signature};
use k256::SecretKey;
use reqwest::{multipart::Part, IntoUrl, Response, Url};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::auth::is_jwttoken_expired;
use crate::request::RequestBuilder;
use crate::Error;

#[derive(Clone)]
pub struct AnalysisConfig {
    pub analysis_url: String,
    pub secret_key: SecretKey,
}

#[derive(Clone)]
pub struct VerityClientConfig {
    pub prover_url: String,
    pub prover_zmq: String,
    pub analysis: Option<AnalysisConfig>,
}

#[derive(Clone)]
pub struct VerityClient {
    pub(crate) inner: reqwest::Client,
    pub(crate) config: VerityClientConfig,
    pub(crate) session_id: Option<Uuid>,
    pub(crate) token: Option<String>,
}

pub struct VerityResponse {
    pub subject: Response,
    pub proof: String,
    pub notary_pub_key: String,
}

impl VerityClient {
    pub fn new(config: VerityClientConfig) -> Self {
        return Self {
            inner: reqwest::Client::new(),
            config,
            session_id: None,
            token: None,
        };
    }

    pub async fn auth(&mut self) {
        let analysis = self
            .config
            .analysis
            .as_ref()
            .expect("analysis config is required");

        let (session_id, challenge) = self.get_challenge().await;

        let signing_key: SigningKey = analysis.secret_key.clone().into();
        let signature: Signature = signing_key.sign(challenge.as_ref());
        let signature = BASE64_STANDARD.encode(signature.to_der().to_bytes());

        let token = self.post_challenge(session_id, signature).await;
        self.session_id = Some(session_id);
        self.token = Some(token);
    }

    async fn get_challenge(&self) -> (Uuid, Vec<u8>) {
        #[derive(Debug, Serialize)]
        struct Request {
            public_key_pem: String,
        }

        #[derive(Debug, Deserialize)]
        struct Response {
            session_id: Uuid,
            challenge: String,
        }

        let analysis = self
            .config
            .analysis
            .as_ref()
            .expect("analysis config is required");
        let url = format!("{}/auth", analysis.analysis_url);

        let client = reqwest::Client::new();

        let public_key = analysis.secret_key.public_key();

        let request = Request {
            public_key_pem: public_key.to_string(),
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

        (
            response.session_id,
            BASE64_STANDARD.decode(response.challenge.clone()).unwrap(),
        )
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

        let analysis = self
            .config
            .analysis
            .as_ref()
            .expect("analysis config is required");
        let url = format!("{}/auth", analysis.analysis_url);

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
            client: self.clone(),
            inner: self.inner.request(method, url),
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
    pub async fn execute(
        &mut self,
        request: reqwest::Request,
    ) -> Result<VerityResponse, crate::Error> {
        self.execute_request(request).await
    }

    pub async fn execute_request(
        &mut self,
        mut req: reqwest::Request,
    ) -> Result<VerityResponse, crate::Error> {
        let proxy_url = &String::from(req.url().as_str());
        let headers = req.headers_mut();

        if self.config.analysis.is_some() {
            if self.token.is_none() || is_jwttoken_expired(self.token.clone().unwrap()) {
                self.auth().await;
            }
        }

        let request_id = Uuid::new_v4();
        headers.append(
            "T-REQUEST-ID",
            HeaderValue::from_str(&format!("{}", request_id)).unwrap(),
        );

        headers.append("T-PROXY-URL", HeaderValue::from_str(proxy_url).unwrap());

        *req.url_mut() = Url::from_str(&format!("{}/proxy", self.config.prover_url)).unwrap();

        let req = reqwest::RequestBuilder::from_parts(self.inner.clone(), req);

        let proof_future = async { self.receive_proof(request_id.to_string()).await };

        let (response, proof) = tokio::join!(req.send(), proof_future);
        let subject = match response {
            Ok(response) => response,
            Err(e) => return Err(Error::Reqwest(e)),
        };

        let proof = match proof {
            Ok(proof) => proof,
            Err(e) => return Err(Error::Verity(e.into())),
        };

        let (notary_pub_key, proof) = proof;

        if self.config.analysis.is_some() {
            if let Err(e) = self.send_proof_to_analysis(&notary_pub_key, &proof).await {
                return Err(Error::Verity(e.into()));
            }
        }

        Ok(VerityResponse {
            subject,
            proof,
            notary_pub_key,
        })
    }

    fn receive_proof(&self, request_id: String) -> JoinHandle<(String, String)> {
        let prover_zmq = self.config.prover_zmq.clone();

        tokio::task::spawn_blocking(move || {
            let context = zmq::Context::new();
            let subscriber = context.socket(zmq::SUB).unwrap();
            assert!(subscriber.connect(prover_zmq.as_str()).is_ok());
            assert!(subscriber.set_subscribe(request_id.as_bytes()).is_ok());

            let proof = subscriber.recv_string(0).unwrap().unwrap();

            // TODO: Gracefully shutdown the ZMQ subscriber with the context
            subscriber.set_unsubscribe(b"").unwrap();

            // TODO: Better split session_id and the proof. See multipart ZMQ messaging.
            let parts: Vec<&str> = proof.splitn(4, "|").collect();

            (parts[1].to_string(), parts[2].to_string())
        })
        .into_future()
    }

    async fn send_proof_to_analysis(
        &self,
        notary_pub_key: &str,
        proof: &str,
    ) -> Result<Response, reqwest::Error> {
        let analysis_config = self
            .config
            .analysis
            .as_ref()
            .expect("analysis configuration not set")
            .clone();

        let url = format!("{}/verify", analysis_config.analysis_url);

        let client = reqwest::Client::new();

        let form = reqwest::multipart::Form::new()
            .part(
                "session_id",
                Part::text(self.session_id.unwrap().to_string()),
            )
            .part("proof", Part::bytes(proof.as_bytes().to_vec()))
            .part(
                "notary_pub_key",
                Part::bytes(notary_pub_key.as_bytes().to_vec()),
            );

        client
            .post(url)
            .bearer_auth(self.token.as_ref().unwrap())
            .multipart(form)
            .send()
            .await
    }
}
