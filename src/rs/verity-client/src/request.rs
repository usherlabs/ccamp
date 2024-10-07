use std::convert::TryFrom;

use http::{HeaderName, HeaderValue};
use reqwest::{header::HeaderMap, Body, Request, Response};
use serde::Serialize;

use crate::{client::VerityClient, error::Result};

/// A builder to construct the properties of a `Request`.
///
/// To construct a `RequestBuilder`, refer to the `Client` documentation.
#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder {
    pub(crate) client: VerityClient,
    pub(crate) inner: reqwest::RequestBuilder,
}

impl RequestBuilder {
    /// Add a `Header` to this Request.
    pub fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        RequestBuilder {
            inner: self.inner.header(key, value),
            ..self
        }
    }

    /// Add a set of Headers to the existing ones on this Request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers(self, headers: HeaderMap) -> Self {
        RequestBuilder {
            inner: self.inner.headers(headers),
            ..self
        }
    }

    /// Set the request body.
    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        RequestBuilder {
            inner: self.inner.body(body),
            ..self
        }
    }

    /// Send a JSON body.
    ///
    /// # Errors
    ///
    /// Serialization can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or if `T` contains a map with non-string keys.
    pub fn json<T: Serialize + ?Sized>(self, json: &T) -> Self {
        RequestBuilder {
            inner: self.inner.json(json),
            ..self
        }
    }

    /// Add a Redact instruction.
    ///
    /// Redact instructs Verity Prover on how to hide a sensitive data.
    pub fn redact(self, redact: String) -> Self {
        RequestBuilder {
            inner: self
                .inner
                .header("T-REDACTED", HeaderValue::from_str(&redact).unwrap()),
            ..self
        }
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `VerityClient::execute()`.
    pub fn build(self) -> reqwest::Result<Request> {
        self.inner.build()
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `VerityClient::execute()`.
    ///
    /// This is similar to [`RequestBuilder::build()`], but also returns the
    /// embedded `VerityClient`.
    pub fn build_split(self) -> (VerityClient, reqwest::Result<Request>) {
        let Self { inner, client, .. } = self;
        let (_, req) = inner.build_split();

        (client, req)
    }

    /// Constructs the Request and sends it to the target URL, returning a
    /// future Response.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Error;
    /// #
    /// # async fn run() -> Result<(), Error> {
    /// let mut rng = rand::thread_rng();
    /// let signing_key = SigningKey::random(&mut rng);
    ///
    /// let config = VerityClientConfig {
    ///     prover_url: String::from("http://127.0.0.1:8080"),
    ///     prover_zmq: String::from("tcp://127.0.0.1:8080"),
    ///     analysis_url: String::from("http://127.0.0.1:8000"),
    ///     signing_key,
    /// };
    ///
    /// let response = verity_client::VerityCLient::from(reqwest::Client::new())
    ///     .get("https://hyper.rs")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(self) -> Result<Response> {
        let (mut client, req) = self.build_split();
        client.execute_request(req?).await
    }
}
